// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use super::xml::strip;
use rmcp::{
    RoleClient, ServiceExt,
    model::{ReadResourceRequestParams, ResourceContents, Tool},
    transport::IntoTransport,
};
use std::{path::Path, process::Stdio, time::Duration};
use tokio::process::{Child, Command};

const SCOPE: &str = "aus-accounting://scope";

fn schema(value: serde_json::Value) -> Result<String, String> {
    let mut value = value;
    value.sort_all_objects();
    let pretty = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
    // Python's json.dumps uses ensure_ascii=True, including UTF-16 surrogate pairs.
    let mut ascii = String::new();
    for c in pretty.chars() {
        if c.is_ascii() {
            ascii.push(c);
        } else {
            for unit in c.encode_utf16(&mut [0; 2]) {
                ascii.push_str(&format!("\\u{unit:04x}"));
            }
        }
    }
    Ok(ascii)
}
fn format_context(instructions: &str, tools: Vec<Tool>, scope: &str) -> Result<String, String> {
    let mut output = format!("# Server instructions\n\n{instructions}\n\n# Tools\n\n");
    for tool in tools {
        output.push_str(&format!(
            "## {}{}\n\n{}\n\nInput schema:\n\n{}\n\nOutput schema:\n\n{}\n\n",
            tool.name,
            tool.title
                .filter(|s| !s.is_empty())
                .map(|s| format!(" ({s})"))
                .unwrap_or_default(),
            strip(tool.description.as_deref().unwrap_or("")),
            schema(serde_json::to_value(tool.input_schema).map_err(|e| e.to_string())?)?,
            schema(serde_json::to_value(tool.output_schema).map_err(|e| e.to_string())?)?
        ));
    }
    output.push_str(&format!(
        "# Preloaded scope resource\n\n{SCOPE}\n\n{scope}\n\n"
    ));
    Ok(output)
}
async fn load_context<T, E, A>(transport: T, deadline: Duration) -> Result<String, String>
where
    T: IntoTransport<RoleClient, E, A>,
    E: std::error::Error + Send + Sync + 'static,
{
    tokio::time::timeout(deadline, async {
        let mut service = ().serve(transport).await.map_err(|e| e.to_string())?;
        let result = async {
            let instructions = service
                .peer_info()
                .and_then(|p| p.instructions.clone())
                .unwrap_or_default();
            let tools = service
                .list_tools(None)
                .await
                .map_err(|e| e.to_string())?
                .tools;
            let scope = service
                .read_resource(ReadResourceRequestParams::new(SCOPE))
                .await
                .map_err(|e| e.to_string())?;
            let Some(ResourceContents::TextResourceContents { text, .. }) = scope.contents.first()
            else {
                return Err("scope resource has no text".into());
            };
            format_context(&instructions, tools, text)
        }
        .await;
        service
            .close_with_timeout(Duration::from_secs(2))
            .await
            .map_err(|e| e.to_string())?;
        result
    })
    .await
    .map_err(|_| "native server context timed out".to_owned())
    .and_then(|r| r)
}
async fn reap(child: &mut Child) -> Result<(), String> {
    // Always reap the owned child, including failed handshakes and request timeouts.
    match tokio::time::timeout(Duration::from_secs(2), child.wait()).await {
        Ok(status) => {
            status.map_err(|e| e.to_string())?;
        }
        Err(_) => {
            child.kill().await.map_err(|e| e.to_string())?;
            child.wait().await.map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
pub(super) async fn describe(server: &Path, deadline: Duration) -> Result<String, String> {
    let mut child = Command::new(server)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("{}: {e}", server.display()))?;
    let transport = (child.stdout.take().unwrap(), child.stdin.take().unwrap());
    let result = load_context(transport, deadline).await;
    reap(&mut child).await?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::{
        model::{ClientJsonRpcMessage, ServerJsonRpcMessage},
        transport::Transport,
    };
    use serde_json::json;
    use std::{
        collections::VecDeque,
        future::Future,
        sync::{Arc, Mutex},
        task::{Context, Poll, Waker},
    };

    #[derive(Default)]
    struct Queue {
        messages: VecDeque<ServerJsonRpcMessage>,
        waker: Option<Waker>,
    }

    struct BlobScopeTransport(Arc<Mutex<Queue>>);

    fn reply_for(request: &serde_json::Value) -> Option<ServerJsonRpcMessage> {
        let id = request.get("id")?.clone();
        let method = request.get("method")?.as_str()?;
        let result = if method == "initialize" {
            json!({
                "protocolVersion": "2025-03-26",
                "capabilities": {"resources": {}, "tools": {}},
                "serverInfo": {"name": "stub", "version": "1"}
            })
        } else if method == "tools/list" {
            json!({"tools": []})
        } else {
            assert_eq!(method, "resources/read");
            json!({"contents": [{"uri": SCOPE, "blob": "AA=="}]})
        };
        serde_json::from_value(json!({"jsonrpc": "2.0", "id": id, "result": result})).ok()
    }

    impl Transport<RoleClient> for BlobScopeTransport {
        type Error = std::io::Error;

        fn send(
            &mut self,
            item: ClientJsonRpcMessage,
        ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'static {
            let queue = self.0.clone();
            let value = serde_json::to_value(&item).ok();
            async move {
                if let Some(reply) = value.as_ref().and_then(reply_for) {
                    let mut state = queue.lock().unwrap();
                    state.messages.push_back(reply);
                    let _ = state.waker.take().map(Waker::wake);
                }
                Ok(())
            }
        }

        fn receive(&mut self) -> impl Future<Output = Option<ServerJsonRpcMessage>> + Send {
            let queue = self.0.clone();
            std::future::poll_fn(move |cx: &mut Context<'_>| {
                let mut state = queue.lock().unwrap();
                if let Some(msg) = state.messages.pop_front() {
                    Poll::Ready(Some(msg))
                } else {
                    state.waker = Some(cx.waker().clone());
                    Poll::Pending
                }
            })
        }

        async fn close(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn load_context_refuses_non_text_scope_resource() {
        let transport = BlobScopeTransport(Arc::new(Mutex::new(Queue::default())));
        let error = load_context(transport, Duration::from_secs(5))
            .await
            .unwrap_err();
        assert_eq!(error, "scope resource has no text");
    }

    #[tokio::test]
    async fn reap_kills_and_waits_for_a_lingering_child() {
        #[cfg(unix)]
        let mut child = Command::new("sleep")
            .arg("30")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        #[cfg(windows)]
        let mut child = Command::new("ping")
            .args(["-n", "30", "127.0.0.1"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        assert_eq!(reap(&mut child).await, Ok(()));
        assert!(child.try_wait().unwrap().is_some());
    }
}
