#![allow(clippy::unwrap_used, clippy::expect_used)]

//! The committed OpenAPI document lists every route the router serves.
//! `make openapi-check` separately proves the committed files match the
//! generated output; this test proves the annotations cover every route.

use std::collections::BTreeMap;

#[test]
fn committed_spec_lists_every_route() {
    let spec: serde_json::Value =
        serde_json::from_str(include_str!("../openapi/openapi.json")).unwrap();
    let paths = spec["paths"].as_object().unwrap();
    let mut actual: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (path, item) in paths {
        let methods: Vec<&str> = item
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        actual.insert(path, methods);
    }
    let expected: BTreeMap<&str, Vec<&str>> = BTreeMap::from([
        ("/health", vec!["get"]),
        ("/notes", vec!["get", "post"]),
        ("/notes/{id}", vec!["delete", "get", "put"]),
    ]);
    assert_eq!(actual, expected);
    assert_eq!(spec["info"]["title"], "template-rust");
    assert_eq!(spec["info"]["version"], env!("CARGO_PKG_VERSION"));
}
