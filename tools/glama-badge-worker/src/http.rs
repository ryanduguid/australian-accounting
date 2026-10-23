use futures_util::StreamExt;
use worker::*;

pub(crate) fn reply(status: u16, body: String, age: Option<u64>, head: bool) -> Result<Response> {
    let mut response = Response::ok(if head { String::new() } else { body })?.with_status(status);
    let headers = response.headers_mut();
    headers.set(
        "content-type",
        if status == 200 {
            "image/svg+xml; charset=utf-8"
        } else {
            "text/plain; charset=utf-8"
        },
    )?;
    headers.set("content-security-policy", "default-src 'none'; sandbox")?;
    headers.set("x-content-type-options", "nosniff")?;
    headers.set("referrer-policy", "no-referrer")?;
    headers.set(
        "cache-control",
        &age.map_or_else(
            || "no-store".to_owned(),
            |age| format!("public, max-age={age}, must-revalidate"),
        ),
    )?;
    Ok(response)
}

pub(crate) async fn bounded_body(response: &mut Response, limit: usize) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut stream = response.stream()?;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if chunk.len() > limit - bytes.len() {
            return Err(Error::RustError("Badge exceeds byte limit".into()));
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}
