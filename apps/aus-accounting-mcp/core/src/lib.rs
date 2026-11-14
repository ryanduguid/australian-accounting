// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The native server's tool call without a transport, for the 6 tools the hosted
//! connector serves: argument checks, the effective input models, the engine
//! adapters and the declared output schemas. No clock, file, network or MCP SDK.
pub mod adapters;
pub mod contracts;
mod validation;

use serde_json::Value;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// One `tools/call` result, as the native server's `AccountingServer::call` builds it.
pub fn call(name: &str, arguments: &Value) -> Value {
    if let Err(error) = contracts::check_arguments(name, arguments) {
        return contracts::error(error);
    }
    let args = match validation::arguments(name, arguments) {
        Ok(args) => args,
        Err(error) => return contracts::error(format!("Error executing tool {name}: {error}")),
    };
    match adapters::evaluate(name, &args).and_then(|result| {
        contracts::validate_output(name, &result)?;
        Ok(result)
    }) {
        Ok(result) => contracts::success(result),
        Err(error) => contracts::error(format!("Error executing tool {name}: {error}")),
    }
}
