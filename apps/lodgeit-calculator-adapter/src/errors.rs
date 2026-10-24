//! `errors.py`: the failure kinds, kept apart from one another on purpose.
//!
//! A caller has to be able to tell "the provider refused" from "the provider
//! is broken" from "we refused to send". Collapsing them is how a timeout
//! becomes a zero. Each is a `pyops::Exc` class; `Exc::is(ADAPTER_ERROR)` is
//! `except AdapterError`.

pub const ADAPTER_ERROR: &str = "lodgeitadapter.errors.AdapterError";
pub const NOT_ENABLED: &str = "lodgeitadapter.errors.NotEnabledError";
pub const CONFIGURATION: &str = "lodgeitadapter.errors.ConfigurationError";
pub const DISALLOWED_TARGET: &str = "lodgeitadapter.errors.DisallowedTargetError";
pub const CONTRACT: &str = "lodgeitadapter.errors.ContractError";
