use crate::{
    cache_age,
    http::{bounded_body, reply},
    identity,
};
use futures_util::future::{Either, select};
use std::{cell::Cell, time::Duration};
use worker::*;

const SOURCE_LIMIT: usize = 32_768;
const OUTPUT_LIMIT: usize = 65_536;
const FETCHED_HEADER: &str = "x-glama-source-fetched-at";
const MAX_FETCHES: usize = 32;

thread_local! {
    static IN_FLIGHT: Cell<usize> = const { Cell::new(0) };
}

// shortcut: this bounds one isolate; account-wide admission needs an edge rate limit.
struct FetchPermit;

impl FetchPermit {
    fn take() -> Option<Self> {
        IN_FLIGHT.with(|count| {
            let active = count.get();
            (active < MAX_FETCHES).then(|| {
                count.set(active + 1);
                Self
            })
        })
    }
}

impl Drop for FetchPermit {
    fn drop(&mut self) {
        IN_FLIGHT.with(|count| count.set(count.get() - 1));
    }
}

async fn source_svg(request: Request, signal: &AbortSignal) -> Result<String> {
    let mut response = Fetch::Request(request).send_with_signal(signal).await?;
    if response.status_code() != 200 {
        return Err(Error::RustError(
            "Glama returned an unsuccessful response".into(),
        ));
    }
    if response
        .headers()
        .get("content-type")?
        .as_deref()
        .and_then(|t| t.split(';').next())
        .map(str::trim)
        != Some("image/svg+xml")
    {
        return Err(Error::RustError("Unexpected Glama media type".into()));
    }
    if response
        .headers()
        .get("content-length")?
        .and_then(|n| n.parse::<u64>().ok())
        .is_some_and(|n| n > SOURCE_LIMIT as u64)
    {
        return Err(Error::RustError("Badge exceeds byte limit".into()));
    }
    let bytes = bounded_body(&mut response, SOURCE_LIMIT).await?;
    let svg = std::str::from_utf8(&bytes).map_err(|_| Error::RustError("Invalid UTF-8".into()))?;
    let svg = glama_badge_server::restyle(svg)
        .map_err(|_| Error::RustError("Unsupported Glama SVG".into()))?;
    if svg.len() > OUTPUT_LIMIT {
        return Err(Error::RustError("Rendered badge exceeds byte limit".into()));
    }
    Ok(svg)
}

async fn source(id: &str) -> Result<String> {
    let mut init = RequestInit::new();
    init.with_method(Method::Get)
        .with_redirect(RequestRedirect::Manual);
    init.headers.set("accept", "image/svg+xml")?;
    init.headers.set("user-agent", "glama-flat-badges/0.1")?;
    let request = Request::new_with_init(
        &format!("https://glama.ai/mcp/{id}/badges/score.svg"),
        &init,
    )?;
    let abort = AbortController::default();
    let signal = abort.signal();
    let work = source_svg(request, &signal);
    let result = match select(
        Box::pin(work),
        Box::pin(Delay::from(Duration::from_secs(8))),
    )
    .await
    {
        Either::Left((result, _)) => result,
        Either::Right(_) => Err(Error::RustError("Glama deadline exceeded".into())),
    };
    abort.abort();
    result
}

#[event(fetch)]
async fn fetch(request: Request, _env: Env, context: Context) -> Result<Response> {
    let head = request.method() == Method::Head;
    if request.method() != Method::Get && !head {
        let mut response = reply(405, "GET or HEAD required".into(), None, false)?;
        response.headers_mut().set("allow", "GET, HEAD")?;
        return Ok(response);
    }
    let url = request.url()?;
    let Some(id) = identity(url.path(), url.query().is_some()) else {
        return reply(404, "Unknown badge".into(), None, head);
    };
    // A local Cache API key only. This reserved domain is never fetched.
    let key = format!("https://flat-glama-badges.invalid/v1/{id}.svg");
    let cache = Cache::default();
    if let Ok(Some(mut cached)) = cache.get(key.as_str(), false).await {
        let fetched = cached
            .headers()
            .get(FETCHED_HEADER)?
            .and_then(|s| s.parse().ok());
        let age = fetched.and_then(|f| cache_age(f, Date::now().as_millis()));
        if cached.status_code() == 200
            && let Some(age) = age
            && let Ok(bytes) = bounded_body(&mut cached, OUTPUT_LIMIT).await
            && let Ok(svg) = String::from_utf8(bytes)
        {
            return reply(200, svg, Some(age), head);
        }
    }
    let Some(_permit) = FetchPermit::take() else {
        return reply(503, "Badge fetch capacity reached".into(), None, head);
    };
    match source(id).await {
        Ok(svg) => {
            let fetched = Date::now().as_millis();
            let mut stored = reply(200, svg.clone(), Some(300), false)?;
            stored
                .headers_mut()
                .set(FETCHED_HEADER, &fetched.to_string())?;
            context.wait_until(async move {
                let _ = cache.put(key.as_str(), stored).await;
            });
            reply(200, svg, Some(60), head)
        }
        Err(_) => reply(502, "Glama badge unavailable".into(), None, head),
    }
}
