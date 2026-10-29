//! Product JWT validation against the product's JWKS endpoint.
//!
//! When `PRODUCT_JWKS_URL` is set, a request carrying
//! `Authorization: Bearer <token>` is authenticated by validating the token
//! against the product's JSON Web Key Set. The token subject (`sub`) becomes
//! the caller identity. Only asymmetric algorithms are accepted: RS256 keys
//! (`kty: RSA`) and ES256 keys (`kty: EC`, curve P-256).

use std::time::{Duration, Instant};

use jsonwebtoken::jwk::{AlgorithmParameters, EllipticCurve, Jwk, JwkSet};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::core::config::AppConfig;
use crate::core::error::AppError;

/// How long a fetched JWKS stays in memory.
const JWKS_TTL: Duration = Duration::from_secs(600);
/// Timeout for one JWKS fetch.
const JWKS_TIMEOUT: Duration = Duration::from_secs(10);

/// The claims this service reads. Expiry, issuer and audience are enforced by
/// [`Validation`] on the raw claims, so they need no fields here.
#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
}

/// Why a token was rejected. Every variant maps to `401`; details stay in the logs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JwtError {
    /// The JWKS endpoint could not be reached or returned garbage.
    Fetch,
    /// The token is malformed, expired, wrongly scoped or badly signed.
    Invalid,
}

impl std::fmt::Display for JwtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JwtError::Fetch => write!(f, "could not fetch the product JWKS"),
            JwtError::Invalid => write!(f, "invalid bearer token"),
        }
    }
}

impl std::error::Error for JwtError {}

/// Validates product JWTs, caching the JWKS in memory.
#[derive(Debug)]
pub struct JwtValidator {
    jwks_url: String,
    issuer: Option<String>,
    audience: Option<String>,
    client: reqwest::Client,
    cache: RwLock<Option<(JwkSet, Instant)>>,
}

impl JwtValidator {
    /// Builds a validator from the application configuration, or `None` when
    /// no JWKS URL is configured and bearer tokens must be refused.
    pub fn from_config(config: &AppConfig) -> Result<Option<Self>, AppError> {
        let Some(url) = config.product_jwks_url.clone() else {
            return Ok(None);
        };
        let client = reqwest::Client::builder()
            .timeout(JWKS_TIMEOUT)
            .build()
            .map_err(|err| AppError::Config(format!("Cannot build the JWKS client: {err}")))?;
        Ok(Some(Self {
            jwks_url: url,
            issuer: config.product_jwt_issuer.clone(),
            audience: config.product_jwt_audience.clone(),
            client,
            cache: RwLock::new(None),
        }))
    }

    /// Validates `token` and returns its subject as the caller identity.
    pub async fn validate(&self, token: &str) -> Result<String, JwtError> {
        let kid = decode_header(token)
            .map_err(|_| JwtError::Invalid)?
            .kid
            .ok_or(JwtError::Invalid)?;
        let jwks = self.jwks_for(&kid).await?;
        let jwk = jwks.find(&kid).ok_or(JwtError::Invalid)?;
        let (key, algorithm) = decoding_key(jwk)?;
        let mut validation = Validation::new(algorithm);
        validation.set_required_spec_claims(&["exp", "sub"]);
        if let Some(issuer) = &self.issuer {
            validation.set_issuer(std::slice::from_ref(issuer));
        }
        if let Some(audience) = &self.audience {
            validation.set_audience(std::slice::from_ref(audience));
        }
        let data = decode::<Claims>(token, &key, &validation).map_err(|_| JwtError::Invalid)?;
        Ok(data.claims.sub)
    }

    /// Returns a cached JWKS containing `kid`, refreshing once when the cache
    /// misses so a rotation takes effect without waiting for the TTL.
    async fn jwks_for(&self, kid: &str) -> Result<JwkSet, JwtError> {
        if let Some(cached) = self.cached().await
            && cached.find(kid).is_some()
        {
            return Ok(cached);
        }
        self.refresh().await
    }

    async fn cached(&self) -> Option<JwkSet> {
        match self.cache.read().await.as_ref() {
            Some((jwks, fetched_at)) if fetched_at.elapsed() < JWKS_TTL => Some(jwks.clone()),
            _ => None,
        }
    }

    async fn refresh(&self) -> Result<JwkSet, JwtError> {
        let response = self
            .client
            .get(&self.jwks_url)
            .send()
            .await
            .map_err(|err| {
                tracing::warn!(error = %err, "JWKS fetch failed");
                JwtError::Fetch
            })?;
        let response = response.error_for_status().map_err(|err| {
            tracing::warn!(error = %err, "JWKS endpoint returned an error status");
            JwtError::Fetch
        })?;
        let jwks: JwkSet = response.json().await.map_err(|err| {
            tracing::warn!(error = %err, "JWKS endpoint returned invalid JSON");
            JwtError::Fetch
        })?;
        *self.cache.write().await = Some((jwks.clone(), Instant::now()));
        Ok(jwks)
    }
}

/// Builds the decoding key for one JWK, pinning the algorithm to the key type
/// so a token can never validate under a weaker algorithm than its key.
fn decoding_key(jwk: &Jwk) -> Result<(DecodingKey, Algorithm), JwtError> {
    let declared = match jwk.common.key_algorithm {
        Some(declared) => Some(Algorithm::try_from(declared).map_err(|_| JwtError::Invalid)?),
        None => None,
    };
    match &jwk.algorithm {
        AlgorithmParameters::RSA(params) => {
            if declared.is_some_and(|alg| alg != Algorithm::RS256) {
                return Err(JwtError::Invalid);
            }
            let key = DecodingKey::from_rsa_components(&params.n, &params.e)
                .map_err(|_| JwtError::Invalid)?;
            Ok((key, Algorithm::RS256))
        }
        AlgorithmParameters::EllipticCurve(params) => {
            if !matches!(params.curve, EllipticCurve::P256)
                || declared.is_some_and(|alg| alg != Algorithm::ES256)
            {
                return Err(JwtError::Invalid);
            }
            let key = DecodingKey::from_ec_components(&params.x, &params.y)
                .map_err(|_| JwtError::Invalid)?;
            Ok((key, Algorithm::ES256))
        }
        _ => Err(JwtError::Invalid),
    }
}
