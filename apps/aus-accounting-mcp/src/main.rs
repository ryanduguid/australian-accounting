// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

use aus_accounting_mcp::{VERSION, handler};
use rmcp::ServiceExt;

#[tokio::main]
async fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() == 1 && args[0] == "--version" {
        println!("aus-accounting-mcp {VERSION}");
        return;
    }
    if args.len() == 1 && (args[0] == "--help" || args[0] == "-h") {
        println!(
            "Run the Aus Accounting MCP server over stdio. An MCP client such as Claude Desktop, Claude Code, Cursor or Codex starts this command itself; run by hand, it waits silently for a client.\n\nUsage: aus-accounting-mcp [--version]\n\nOptional retrieval folders: set AUS_ACCOUNTING_LIBRARY_ROOT to a Markdown library, or AUS_ACCOUNTING_CORPUS_ROOT to a legislation corpus, in the client's server configuration. Setup guide: https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/docs/REFERENCE.md#client-setup"
        );
        return;
    }
    if !args.is_empty() {
        eprintln!("aus-accounting-mcp: unrecognised arguments");
        std::process::exit(2);
    }
    let (service, transport) = handler::stdio();
    match service.serve(transport).await {
        Ok(service) => {
            if let Err(error) = service.waiting().await {
                eprintln!("MCP service failed: {error}");
                std::process::exit(1);
            }
        }
        Err(rmcp::service::ServerInitializeError::ConnectionClosed(_)) => {}
        Err(error) => {
            eprintln!("MCP initialization failed: {error}");
            std::process::exit(1);
        }
    }
}
