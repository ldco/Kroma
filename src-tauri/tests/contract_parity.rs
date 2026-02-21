use std::collections::BTreeSet;

use pretty_assertions::assert_eq;

use kroma_backend_core::api::routes::route_catalog;
use kroma_backend_core::contract::openapi;
use kroma_backend_core::contract::RouteSpec;
use kroma_backend_core::default_openapi_contract_path;

#[test]
fn rust_route_catalog_matches_openapi_contract() {
    let openapi_routes = openapi::load_routes(default_openapi_contract_path().as_path())
        .expect("OpenAPI contract must parse for parity tests");
    let rust_routes: Vec<RouteSpec> = route_catalog()
        .into_iter()
        .map(|entry| entry.spec)
        .collect();

    let openapi_set: BTreeSet<RouteSpec> = openapi_routes.into_iter().collect();
    let rust_set: BTreeSet<RouteSpec> = rust_routes.into_iter().collect();

    let missing: Vec<String> = openapi_set
        .difference(&rust_set)
        .map(ToString::to_string)
        .collect();
    let extra: Vec<String> = rust_set
        .difference(&openapi_set)
        .map(ToString::to_string)
        .collect();

    assert!(
        missing.is_empty() && extra.is_empty(),
        "route contract mismatch\nmissing_in_rust={missing:?}\nextra_in_rust={extra:?}"
    );

    assert_eq!(rust_set, openapi_set);
}

#[test]
fn rust_route_catalog_entries_are_unique() {
    let routes = route_catalog();
    let set: BTreeSet<RouteSpec> = routes.iter().map(|entry| entry.spec.clone()).collect();
    assert_eq!(
        set.len(),
        routes.len(),
        "route catalog contains duplicate entries"
    );
}
