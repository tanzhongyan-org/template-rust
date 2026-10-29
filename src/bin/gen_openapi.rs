//! Prints the service OpenAPI document as pretty JSON.
//!
//! Used by `make openapi` and `make openapi-check`, which write or diff
//! `openapi/openapi.json`. It is not part of the running service.

use template_rust::api::openapi::ApiDoc;
use utoipa::OpenApi;

fn main() -> anyhow::Result<()> {
    println!("{}", ApiDoc::openapi().to_pretty_json()?);
    Ok(())
}
