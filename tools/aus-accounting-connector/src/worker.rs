// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The Workers glue: no storage, no outbound request and no logging.

use crate::{BODY_LIMIT, Reply, admit, allow_origin, respond, today};
use futures_util::StreamExt;
use worker::{Context, Date, Env, Request, Response, Result, event};

struct Incoming<'a>(&'a worker::Headers);

impl crate::Headers for Incoming<'_> {
    fn get(&self, name: &str) -> Option<String> {
        self.0.get(name).ok().flatten()
    }
}

fn send(reply: Reply) -> Result<Response> {
    let response = match reply.body {
        Some(body) => Response::from_bytes(body.into_bytes())?,
        None => Response::empty()?,
    };
    let mut response = response.with_status(reply.status);
    let headers = response.headers_mut();
    for (name, value) in reply.headers {
        headers.set(name, &value)?;
    }
    Ok(response)
}

/// The body, or `None` once it passes `BODY_LIMIT` bytes.
async fn bounded(request: &mut Request) -> Result<Option<Vec<u8>>> {
    let mut bytes = Vec::new();
    let mut stream = request.stream()?;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if chunk.len() > BODY_LIMIT - bytes.len() {
            return Ok(None);
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(Some(bytes))
}

#[event(fetch)]
async fn fetch(mut request: Request, env: Env, _context: Context) -> Result<Response> {
    let reply = handle(&mut request, &env).await?;
    send(allow_origin(reply, &Incoming(request.headers())))
}

async fn handle(request: &mut Request, env: &Env) -> Result<Reply> {
    let url = request.url()?;
    if let Err(reply) = admit(
        request.method().as_ref(),
        url.path(),
        &Incoming(request.headers()),
    ) {
        return Ok(reply);
    }
    // Fails closed: a missing or failing limiter refuses the request.
    let address = request
        .headers()
        .get("cf-connecting-ip")?
        .unwrap_or_default();
    let admitted = match env.rate_limiter("LIMITER") {
        Ok(limiter) => limiter.limit(address).await.map(|outcome| outcome.success),
        Err(error) => Err(error),
    };
    match admitted {
        Ok(true) => {}
        Ok(false) => return Ok(Reply::new(429, None).header("retry-after", "60")),
        Err(_) => return Ok(Reply::new(503, None)),
    }
    let Some(body) = bounded(request).await? else {
        return Ok(Reply::new(413, None));
    };
    let today = today(Date::now().as_millis());
    Ok(respond(&body, &Incoming(request.headers()), &today))
}
