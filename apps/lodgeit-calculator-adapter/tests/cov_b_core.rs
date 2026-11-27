// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.
//! `CoreClient.list_clients` refusing before it would open a connection,
//! and `CoreConfig.enabled`. Each expected value is what the retired Python
//! `lodgeitadapter.core` gives under CPython 3.15: `list_clients` checks
//! `config.enabled`, then the page size, before it builds a request, so no
//! host is contacted here.
use lodgeit_calculator_adapter::core::{CoreCategory, CoreClient, CoreConfig};

const TOKEN: &str = "fabricated-token-for-offline-test";

#[test]
fn enabled_reads_the_flag_the_config_was_built_with() {
    assert!(CoreConfig::new(true, 20.0, 1024).unwrap().enabled());
    assert!(!CoreConfig::default().enabled());
}

#[test]
fn a_disabled_client_refuses_before_any_request() {
    let client = CoreClient {
        config: CoreConfig::default(),
    };
    let error = client.list_clients(TOKEN, 100, None).unwrap_err();
    assert_eq!(error.category, CoreCategory::Disabled);
}

#[test]
fn an_enabled_client_refuses_a_page_size_out_of_range_before_any_request() {
    let client = CoreClient {
        config: CoreConfig::new(true, 20.0, 1024).unwrap(),
    };
    for size in [0, 501] {
        let error = client.list_clients(TOKEN, size, None).unwrap_err();
        assert_eq!(error.category, CoreCategory::InvalidInput, "{size}");
    }
}
