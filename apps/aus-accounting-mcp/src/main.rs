// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use aus_accounting_mcp::{VERSION, doctor, handler};
use rmcp::ServiceExt;

const LICENCE_NOTICE: &str = "\n\
    Copyright 2026 Ryan Duguid. No warranty.\n\
    AGPL-3.0-or-later with additional terms (see NOTICE).\n\
    Licence: https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/LICENSE\n";

#[tokio::main]
async fn main() {
    let args = aus_accounting_mcp::command_line::command_words();
    if args.len() == 1 && args[0] == "--version" {
        println!("aus-accounting-mcp {VERSION}");
        return;
    }
    if args.len() == 1 && (args[0] == "--help" || args[0] == "-h") {
        println!(
            "Run the Aus Accounting MCP server over stdio. An MCP client such as Claude Desktop, Claude Code, Cursor or Codex starts this command itself; run by hand, it waits silently for a client.\n\nUsage: aus-accounting-mcp [--version] [doctor]\n\nCommands:\n  doctor  {}\n\nOptional retrieval folders: set AUS_ACCOUNTING_LIBRARY_ROOT to a Markdown library, or AUS_ACCOUNTING_CORPUS_ROOT to a legislation corpus, in the client's server configuration, then run `aus-accounting-mcp doctor` with the same variables to check them. Setup guide: https://github.com/ryanduguid/australian-accounting/blob/main/apps/aus-accounting-mcp/docs/REFERENCE.md#client-setup{LICENCE_NOTICE}",
            doctor::HELP
        );
        return;
    }
    if args.len() == 1 && args[0] == "doctor" {
        let (report, code) = doctor::report();
        print!("{report}");
        std::process::exit(code);
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
