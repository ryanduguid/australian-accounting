// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

use super::xml::strip;
use rmcp::{
    ServiceExt,
    model::{ReadResourceRequestParams, ResourceContents, Tool},
};
use std::{path::Path, process::Stdio, time::Duration};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

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
async fn load_context(
    transport: (ChildStdout, ChildStdin),
    deadline: Duration,
) -> Result<String, String> {
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
