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

    async fn fetch(&self, identity: &str) -> Result<String, &'static str> {
        let mut response = self
            .client
            .get(format!("{}{identity}/badges/score.svg", self.upstream))
            .header("Accept", "image/svg+xml")
            .send()
            .await
            .map_err(|_| "Glama request failed")?;
        if !response.status().is_success() {
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
            let mut cache = service.cache.lock().expect("cache mutex");
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
                    svg: svg.clone(),
                    fetched: Instant::now(),
                },
            );
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
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tower::ServiceExt;

    const SVG: &str = include_str!("../tests/fixtures/server-a-b-d.svg");
    const KEY: &str = "servers/ryanduguid/australian-accounting";
    const ROUTE: &str = "/glama/servers/ryanduguid/australian-accounting.svg";

    struct Mock {
        service: Arc<Service>,
        hits: Arc<AtomicUsize>,
        task: tokio::task::JoinHandle<()>,
    }

    impl Drop for Mock {
        fn drop(&mut self) {
            self.task.abort();
        }
    }

    async fn mock(status: StatusCode, mime: &'static str, body: Vec<u8>) -> Mock {
        let hits = Arc::new(AtomicUsize::new(0));
        let requests = hits.clone();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let router = Router::new().fallback(move |headers: axum::http::HeaderMap| {
            assert!(!headers.contains_key("authorization"));
            assert!(!headers.contains_key("cookie"));
            assert!(!headers.contains_key("x-forwarded-for"));
            let body = body.clone();
            requests.fetch_add(1, Ordering::SeqCst);
            async move {
                (
                    status,
                    [("content-type", mime), ("location", "/redirect-target")],
                    body,
                )
            }
        });
        let task = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        let mut service = Service::new().unwrap();
        service.client = client(false).unwrap();
        service.upstream = Box::leak(format!("http://{address}/mcp/").into_boxed_str());
        Mock {
            service: Arc::new(service),
            hits,
            task,
        }
    }

    async fn request(service: &Arc<Service>, route: &str, method: Method) -> Response {
        app(service.clone())
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(route)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn live_response_preserves_source_and_uses_bounded_cache() {
        let m = mock(
            StatusCode::OK,
            "image/svg+xml; charset=utf-8",
            SVG.as_bytes().to_vec(),
        )
        .await;
        for _ in 0..2 {
            let response = request(&m.service, ROUTE, Method::GET).await;
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(response.headers()["x-content-type-options"], "nosniff");
            assert_eq!(
                response.headers()["content-security-policy"],
                "default-src 'none'; sandbox"
            );
            assert!(
                response.headers()["cache-control"]
                    .to_str()
                    .unwrap()
                    .ends_with("must-revalidate")
            );
            assert_eq!(
                to_bytes(response.into_body(), MAX_BYTES).await.unwrap(),
                glama_badge_server::restyle(SVG).unwrap()
            );
        }
        assert_eq!(m.hits.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn unknown_urls_queries_and_wrong_methods_never_fetch() {
        let m = mock(StatusCode::OK, "image/svg+xml", SVG.as_bytes().to_vec()).await;
        for route in [
            "/",
            "/glama/servers/unknown.svg",
            "/glama/https://localhost.svg",
            "/glama/servers/../x.svg",
            "/glama/servers/ryanduguid/australian-accounting.svg?url=http://localhost",
            "/glama/servers/%72yanduguid/australian-accounting.svg",
        ] {
            let response = request(&m.service, route, Method::GET).await;
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "{route}");
            assert_eq!(response.headers()["cache-control"], "no-store");
        }
        let response = request(&m.service, ROUTE, Method::POST).await;
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(response.headers()["allow"], "GET, HEAD");
        assert_eq!(m.hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn head_has_no_body() {
        let m = mock(StatusCode::OK, "image/svg+xml", SVG.as_bytes().to_vec()).await;
        let response = request(&m.service, ROUTE, Method::HEAD).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            to_bytes(response.into_body(), MAX_BYTES)
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn caller_credentials_and_forwarding_headers_are_not_sent_upstream() {
        let m = mock(StatusCode::OK, "image/svg+xml", SVG.as_bytes().to_vec()).await;
        let request = Request::builder()
            .uri(ROUTE)
            .header("authorization", "Bearer synthetic-test-value")
            .header("cookie", "synthetic=value")
            .header("x-forwarded-for", "192.0.2.1")
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            app(m.service.clone())
                .oneshot(request)
                .await
                .unwrap()
                .status(),
            StatusCode::OK
        );
        assert_eq!(m.hits.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn expired_downstream_cache_age_never_extends_the_source_window() {
        let m = mock(StatusCode::OK, "image/svg+xml", SVG.as_bytes().to_vec()).await;
        m.service.cache.lock().unwrap().insert(
            KEY.to_owned(),
            Entry {
                svg: "cached fixture".into(),
                fetched: Instant::now() - Duration::from_secs(295),
            },
        );
        let response = request(&m.service, ROUTE, Method::GET).await;
        let value = response.headers()["cache-control"].to_str().unwrap();
        assert!(
            value == "public, max-age=4, must-revalidate"
                || value == "public, max-age=3, must-revalidate"
        );
        assert_eq!(m.hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn stalled_provider_hits_total_deadline_without_caching() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            let (_socket, _) = listener.accept().await.unwrap();
            std::future::pending::<()>().await;
        });
        let mut service = Service::new().unwrap();
        service.client = client(false).unwrap();
        service.upstream = Box::leak(format!("http://{address}/mcp/").into_boxed_str());
        let service = Arc::new(service);
        let result = tokio::time::timeout(
            Duration::from_secs(10),
            request(&service, ROUTE, Method::GET),
        )
        .await;
        task.abort();
        assert_eq!(result.unwrap().status(), StatusCode::BAD_GATEWAY);
        assert!(service.cache.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn chunked_oversized_provider_body_is_rejected_without_content_length() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = [0; 2048];
            let _ = socket.read(&mut request).await.unwrap();
            let body = "x".repeat(MAX_BYTES + 1);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: image/svg+xml\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n{:x}\r\n{body}\r\n0\r\n\r\n",
                body.len()
            );
            let _ = socket.write_all(response.as_bytes()).await;
        });
        let mut service = Service::new().unwrap();
        service.client = client(false).unwrap();
        service.upstream = Box::leak(format!("http://{address}/mcp/").into_boxed_str());
        let service = Arc::new(service);
        let result = request(&service, ROUTE, Method::GET).await;
        task.abort();
        assert_eq!(result.status(), StatusCode::BAD_GATEWAY);
        assert!(service.cache.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn provider_errors_and_redirects_fail_without_following_or_caching() {
        for status in [
            StatusCode::UNAUTHORIZED,
            StatusCode::NOT_FOUND,
            StatusCode::TOO_MANY_REQUESTS,
            StatusCode::INTERNAL_SERVER_ERROR,
            StatusCode::FOUND,
        ] {
            let m = mock(status, "image/svg+xml", SVG.as_bytes().to_vec()).await;
            let response = request(&m.service, ROUTE, Method::GET).await;
            assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
            assert_eq!(response.headers()["cache-control"], "no-store");
            assert_eq!(m.hits.load(Ordering::SeqCst), 1);
            assert!(m.service.cache.lock().unwrap().is_empty());
        }
    }

    #[tokio::test]
    async fn invalid_media_oversized_non_utf8_and_active_svg_fail_closed() {
        for (mime, body) in [
            ("text/html", SVG.as_bytes().to_vec()),
            ("image/svg+xml", vec![b'a'; MAX_BYTES + 1]),
            ("image/svg+xml", vec![255]),
            (
                "image/svg+xml",
                b"<svg><script>alert(1)</script></svg>".to_vec(),
            ),
        ] {
            let m = mock(StatusCode::OK, mime, body).await;
            assert_eq!(
                request(&m.service, ROUTE, Method::GET).await.status(),
                StatusCode::BAD_GATEWAY
            );
            assert!(m.service.cache.lock().unwrap().is_empty());
        }
    }

    #[tokio::test]
    async fn expired_positive_result_is_not_served_after_upstream_failure() {
        let m = mock(
            StatusCode::SERVICE_UNAVAILABLE,
            "text/plain",
            b"offline".to_vec(),
        )
        .await;
        m.service.cache.lock().unwrap().insert(
            KEY.to_owned(),
            Entry {
                svg: "old positive grade".into(),
                fetched: Instant::now() - TTL,
            },
        );
        let response = request(&m.service, ROUTE, Method::GET).await;
        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
        assert_eq!(m.hits.load(Ordering::SeqCst), 1);
        assert!(m.service.cache.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn fetch_capacity_is_bounded_without_a_wait_queue() {
        let m = mock(StatusCode::OK, "image/svg+xml", SVG.as_bytes().to_vec()).await;
        let _permits = m
            .service
            .requests
            .clone()
            .acquire_many_owned(4)
            .await
            .unwrap();
        assert_eq!(
            request(&m.service, ROUTE, Method::GET).await.status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(m.hits.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn full_cache_evicts_oldest_entry_before_inserting_new_result() {
        let m = mock(StatusCode::OK, "image/svg+xml", SVG.as_bytes().to_vec()).await;
        let keys: Vec<_> = m
            .service
            .allowed
            .iter()
            .copied()
            .filter(|key| *key != KEY)
            .take(MAX_CACHE)
            .collect();
        {
            let mut cache = m.service.cache.lock().unwrap();
            for (index, key) in keys.iter().enumerate() {
                cache.insert(
                    (*key).to_owned(),
                    Entry {
                        svg: "cached fixture".into(),
                        fetched: Instant::now()
                            - Duration::from_secs(if index == 0 { 30 } else { 1 }),
                    },
                );
            }
        }
        assert_eq!(
            request(&m.service, ROUTE, Method::GET).await.status(),
            StatusCode::OK
        );
        let cache = m.service.cache.lock().unwrap();
        assert_eq!(cache.len(), MAX_CACHE);
        assert!(!cache.contains_key(keys[0]));
        assert!(cache.contains_key(KEY));
    }

    #[tokio::test]
    async fn four_real_fetches_exhaust_capacity_and_fifth_miss_does_not_queue() {
        let hits = Arc::new(AtomicUsize::new(0));
        let requests = hits.clone();
        let gate = Arc::new(Semaphore::new(0));
        let upstream_gate = gate.clone();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let router = Router::new().fallback(move || {
            let gate = upstream_gate.clone();
            requests.fetch_add(1, Ordering::SeqCst);
            async move {
                let _permit = gate.acquire().await.unwrap();
                (StatusCode::OK, [("content-type", "image/svg+xml")], SVG)
            }
        });
        let task = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        let mut service = Service::new().unwrap();
        service.client = client(false).unwrap();
        service.upstream = Box::leak(format!("http://{address}/mcp/").into_boxed_str());
        let m = Mock {
            service: Arc::new(service),
            hits,
            task,
        };
        let mut fetches = Vec::new();
        for _ in 0..4 {
            let service = m.service.clone();
            fetches.push(tokio::spawn(async move {
                request(&service, ROUTE, Method::GET).await.status()
            }));
        }
        tokio::time::timeout(Duration::from_secs(3), async {
            while m.hits.load(Ordering::SeqCst) < 4 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert_eq!(
            request(&m.service, ROUTE, Method::GET).await.status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(m.hits.load(Ordering::SeqCst), 4);
        gate.add_permits(4);
        for fetch in fetches {
            assert_eq!(fetch.await.unwrap(), StatusCode::OK);
        }
    }

    #[test]
    fn allowlist_contains_only_observed_public_glama_routes() {
        let service = Service::new().unwrap();
        assert_eq!(service.allowed.len(), 2947);
        assert!(service.allowed.contains(KEY));
        for key in &service.allowed {
            let parts: Vec<_> = key.split('/').collect();
            assert!(matches!(parts[0], "servers" | "connectors"));
            assert!((2..=3).contains(&parts.len()));
            assert!(parts[1..].iter().all(|p| {
                !p.is_empty()
                    && *p != "."
                    && *p != ".."
                    && p.bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"-_.@".contains(&b))
            }));
        }
    }
}
