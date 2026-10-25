use axum::{
    Router,
    extract::State,
    http::{Method, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::Semaphore;

const TTL: Duration = Duration::from_secs(300);
const MAX_BYTES: usize = 32_768;
const MAX_CACHE: usize = 256;
const UPSTREAM: &str = "https://glama.ai/mcp/";

struct Entry {
    svg: String,
    fetched: Instant,
}

struct Service {
    client: reqwest::Client,
    allowed: HashSet<&'static str>,
    cache: Mutex<HashMap<String, Entry>>,
    requests: Arc<Semaphore>,
    upstream: &'static str,
}

fn client(https_only: bool) -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .https_only(https_only)
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(8))
        .user_agent("glama-flat-badges/0.1")
        .build()
}

impl Service {
    fn new() -> Result<Self, reqwest::Error> {
        Ok(Self {
            client: client(true)?,
            allowed: include_str!("../allowed.txt").lines().collect(),
            cache: Mutex::new(HashMap::new()),
            requests: Arc::new(Semaphore::new(4)),
            upstream: UPSTREAM,
        })
    }

    fn cache_result(&self, identity: &str, svg: String) {
        let mut cache = self.cache.lock().expect("cache mutex");
        if cache.len() >= MAX_CACHE {
            let oldest = cache
                .iter()
                .min_by_key(|(_, entry)| entry.fetched)
                .map(|(key, _)| key.clone());
            if let Some(oldest) = oldest {
                cache.remove(&oldest);
            }
        }
        cache.insert(
            identity.to_owned(),
            Entry {
                svg,
                fetched: Instant::now(),
            },
        );
    }

    async fn fetch(&self, identity: &str) -> Result<String, &'static str> {
        let mut response = self
            .client
            .get(format!("{}{identity}/badges/score.svg", self.upstream))
            .header("Accept", "image/svg+xml")
            .send()
            .await
            .map_err(|_| "Glama request failed")?;
        if response.status() != StatusCode::OK {
            return Err("Glama returned an unsuccessful response");
        }
        let mime = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(';').next());
        if mime != Some("image/svg+xml") {
            return Err("Glama returned an unexpected media type");
        }
        if response
            .content_length()
            .is_some_and(|n| n > MAX_BYTES as u64)
        {
            return Err("Glama badge exceeds the size limit");
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|_| "Glama body failed")? {
            if chunk.len() > MAX_BYTES - bytes.len() {
                return Err("Glama badge exceeds the size limit");
            }
            bytes.extend_from_slice(&chunk);
        }
        let svg = std::str::from_utf8(&bytes).map_err(|_| "Glama badge is not UTF-8")?;
        glama_badge_server::restyle(svg).map_err(|_| "Glama badge format is unsupported")
    }
}

fn reply(status: StatusCode, body: String, max_age: Option<u64>) -> Response {
    let is_svg = status == StatusCode::OK;
    let mut response = (status, body).into_response();
    let headers = response.headers_mut();
    headers.insert(
        "content-type",
        if is_svg {
            "image/svg+xml; charset=utf-8"
        } else {
            "text/plain; charset=utf-8"
        }
        .parse()
        .expect("literal header"),
    );
    headers.insert(
        "x-content-type-options",
        "nosniff".parse().expect("literal header"),
    );
    headers.insert(
        "content-security-policy",
        "default-src 'none'; sandbox"
            .parse()
            .expect("literal header"),
    );
    headers.insert(
        "referrer-policy",
        "no-referrer".parse().expect("literal header"),
    );
    headers.insert(
        "cache-control",
        max_age
            .map_or_else(
                || "no-store".to_owned(),
                |n| format!("public, max-age={n}, must-revalidate"),
            )
            .parse()
            .expect("bounded cache age"),
    );
    response
}

async fn badge(State(service): State<Arc<Service>>, uri: Uri, method: Method) -> Response {
    if method != Method::GET && method != Method::HEAD {
        let mut response = reply(
            StatusCode::METHOD_NOT_ALLOWED,
            "GET or HEAD required".into(),
            None,
        );
        response
            .headers_mut()
            .insert("allow", "GET, HEAD".parse().expect("literal header"));
        return response;
    }
    let response = badge_response(service, uri).await;
    if method == Method::HEAD {
        response.map(|_| axum::body::Body::empty())
    } else {
        response
    }
}

async fn badge_response(service: Arc<Service>, uri: axum::http::Uri) -> Response {
    let identity = uri
        .path()
        .strip_prefix("/glama/")
        .and_then(|p| p.strip_suffix(".svg"));
    let Some(identity) =
        identity.filter(|id| uri.query().is_none() && service.allowed.contains(id))
    else {
        return reply(StatusCode::NOT_FOUND, "Unknown badge".into(), None);
    };
    {
        let mut cache = service.cache.lock().expect("cache mutex");
        cache.retain(|_, entry| entry.fetched.elapsed() < TTL);
        if let Some(entry) = cache.get(identity) {
            let remaining = TTL
                .saturating_sub(entry.fetched.elapsed())
                .as_secs()
                .min(60);
            return reply(StatusCode::OK, entry.svg.clone(), Some(remaining));
        }
    }
    let Ok(_permit) = service.requests.clone().try_acquire_owned() else {
        return reply(
            StatusCode::SERVICE_UNAVAILABLE,
            "Badge fetch capacity reached".into(),
            None,
        );
    };
    match service.fetch(identity).await {
        Ok(svg) => {
            service.cache_result(identity, svg.clone());
            reply(StatusCode::OK, svg, Some(60))
        }
        Err(message) => reply(StatusCode::BAD_GATEWAY, message.into(), None),
    }
}

fn app(service: Arc<Service>) -> Router {
    Router::new().fallback(badge).with_state(service)
}

#[tokio::main(worker_threads = 2)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Discard the executable name; only an explicit SocketAddr controls the bind address.
    // nosemgrep: rust.lang.security.args.args
    let args: Vec<String> = std::env::args().skip(1).collect();
    let address: SocketAddr = match args.as_slice() {
        [] => "127.0.0.1:8080".parse()?,
        [flag, address] if flag == "--listen" => address.parse()?,
        _ => return Err("Usage: glama-badge-server [--listen IP:PORT]".into()),
    };
    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, app(Arc::new(Service::new()?)))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests;
