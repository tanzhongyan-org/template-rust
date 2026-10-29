//! The service OpenAPI document, generated from the handler annotations.
//!
//! `openapi/openapi.json` is the committed contract: `make openapi` regenerates
//! it together with the TypeScript client, and `make openapi-check` fails when
//! the committed files differ from the generated output.

use utoipa::{
    Modify, OpenApi,
    openapi::{
        Components,
        security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme},
    },
};

use crate::api::{health, notes};
use crate::core::error::{ErrorBody, ErrorEnvelope};

#[derive(OpenApi)]
#[openapi(
    paths(
        health::health_handler,
        notes::list_notes,
        notes::get_note,
        notes::create_note,
        notes::update_note,
        notes::delete_note,
    ),
    components(schemas(
        health::HealthResponse,
        notes::Note,
        notes::CreateNoteRequest,
        notes::UpdateNoteRequest,
        ErrorEnvelope,
        ErrorBody,
    )),
    tags(
        (name = "notes", description = "Example slice: notes owned by the caller"),
        (name = "health", description = "Service probe"),
    ),
    modifiers(&SecurityAddon),
    info(description = "Rust web service template: Axum, SQLx and fail-closed identity.")
)]
pub struct ApiDoc;

/// Registers the two caller credentials: the product JWT and the dev header.
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Components::new);
        components.add_security_scheme(
            "bearer",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some("Product JWT validated against the product JWKS"))
                    .build(),
            ),
        );
        components.add_security_scheme(
            "identity_header",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::with_description(
                "x-user-id",
                "Dev-only caller header, honoured only with TRUST_IDENTITY_HEADER=true",
            ))),
        );
    }
}
