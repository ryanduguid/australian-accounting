//! Finite hosting trial. The live provider handler is absent from this build.

use crate::{
    cache_age,
    http::{bounded_body, reply},
};
use worker::*;

const FIXTURE: &str = include_str!("../../glama-badge-server/tests/fixtures/server-a-b-d.svg");
const RENDERED_AT: &str = "x-synthetic-rendered-at";

fn window(env: &Env, now: u64) -> Option<(u64, u64)> {
    let start = env
        .var("SYNTHETIC_START_MS")
        .ok()?
        .to_string()
        .parse()
        .ok()?;
    let end = env.var("SYNTHETIC_END_MS").ok()?.to_string().parse().ok()?;
    let duration = u64::checked_sub(end, start)?;
    ((1..=3_600_000).contains(&duration) && start <= now && now < end).then_some((start, end))
}

fn render() -> Result<String> {
    let svg = glama_badge_server::restyle(FIXTURE)
        .map_err(|_| Error::RustError("Synthetic fixture could not be rendered".into()))?;
    let svg = svg.replacen("<svg ", "<svg x=\"65\" y=\"24\" ", 1);
    Ok(format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"240\" height=\"52\" viewBox=\"0 0 240 52\" role=\"img\" aria-labelledby=\"trial-title trial-desc\"><title id=\"trial-title\">Synthetic rendering trial</title><desc id=\"trial-desc\">Recorded fixture, not a live rating.</desc><path fill=\"#000000\" d=\"M0 0h240v52H0z\"/><text x=\"120\" y=\"16\" text-anchor=\"middle\" font-family=\"Verdana, sans-serif\" font-size=\"11\" fill=\"#FFD32A\">SYNTHETIC TEST DATA</text><g aria-hidden=\"true\">{svg}</g></svg>"
    ))
}

#[event(fetch)]
async fn fetch(request: Request, env: Env, context: Context) -> Result<Response> {
    let head = request.method() == Method::Head;
    let now = Date::now().as_millis();
    let Some((start, end)) = window(&env, now) else {
        return reply(410, "Synthetic trial inactive".into(), None, head);
    };
    if request.method() != Method::Get && !head {
        let mut response = reply(405, "GET or HEAD required".into(), None, false)?;
        response.headers_mut().set("allow", "GET, HEAD")?;
        return Ok(response);
    }
    let url = request.url()?;
    if url.query().is_some() || !matches!(url.path(), "/synthetic.svg" | "/failure.svg") {
        return reply(404, "Unknown synthetic trial route".into(), None, head);
    }
    if url.path() == "/failure.svg" {
        return reply(503, "Deliberate synthetic failure".into(), None, head);
    }
    // A local Cache API key only. No request is sent to this reserved domain.
    let key = format!("https://flat-glama-synthetic.invalid/v1/{start}/{end}.svg");
    let cache = Cache::default();
    if let Ok(Some(mut cached)) = cache.get(key.as_str(), false).await {
        let rendered = cached
            .headers()
            .get(RENDERED_AT)?
            .and_then(|v| v.parse().ok());
        if cached.status_code() == 200
            && let Ok(bytes) = bounded_body(&mut cached, 65_536).await
            && let Ok(svg) = String::from_utf8(bytes)
        {
            let now = Date::now().as_millis();
            if now < start || now >= end {
                return reply(410, "Synthetic trial inactive".into(), None, head);
            }
            if let Some(age) = rendered.and_then(|t| cache_age(t, now)) {
                let remaining = (end - now) / 1_000;
                let mut response = reply(200, svg, Some(age.min(remaining)), head)?;
                response.headers_mut().set("x-synthetic-cache", "HIT")?;
                return Ok(response);
            }
        }
    }
    let svg = render()?;
    let now = Date::now().as_millis();
    if now < start || now >= end {
        return reply(410, "Synthetic trial inactive".into(), None, head);
    }
    let remaining = (end - now) / 1_000;
    let mut stored = reply(200, svg.clone(), Some(300.min(remaining)), false)?;
    stored.headers_mut().set(RENDERED_AT, &now.to_string())?;
    context.wait_until(async move {
        let _ = cache.put(key.as_str(), stored).await;
    });
    let mut response = reply(200, svg, Some(60.min(remaining)), head)?;
    response.headers_mut().set("x-synthetic-cache", "MISS")?;
    Ok(response)
}
