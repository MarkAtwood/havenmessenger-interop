//! Every request to the peer-facing listener is from an authenticated provider (protocol-07 §4.1,
//! draft-atwood-mimi-provider-auth §4 and §10): by the client certificate it presented in the TLS handshake, or, with no
//! certificate, by a signature from a key its provider publishes in DNS ([`crate::signature`]). A request with neither
//! is answered 403 and nothing more (§11). This is the one place the listener decides that, for every route but the
//! directory, which §5.1 has fetched before there is a peer to authenticate.
//!
//! A signature that verifies makes the request's [`PeerCert`] the signed domain, so each route's own question ("does
//! this peer speak for domain X?") is answered the same way for both proofs.

use std::sync::Arc;
use std::time::SystemTime;

use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Router;

use crate::peer::{PeerCert, PeerIdentity};
use crate::signature::{verify, PeerKeys, SignedRequest};

/// The most body a signed request may carry; the routes' own limits are smaller.
const MAX_SIGNED_BODY: usize = 16 * 1024 * 1024;

/// The one route open without a proof.
const DIRECTORY: &str = "/.well-known/mimi-protocol-directory";

/// `router` behind the provider-auth gate, with signatures checked against `keys`.
pub fn with_provider_auth(router: Router, keys: Arc<PeerKeys>) -> Router {
    router.layer(axum::middleware::from_fn(
        move |request: Request, next: Next| {
            let keys = keys.clone();
            async move { gate(&keys, request, next).await }
        },
    ))
}

async fn gate(keys: &PeerKeys, request: Request, next: Next) -> Response {
    let has_certificate = matches!(
        request.extensions().get::<PeerCert>(),
        Some(PeerCert(Some(_)))
    );
    if has_certificate || request.uri().path() == DIRECTORY {
        return next.run(request).await;
    }
    let (mut parts, body) = request.into_parts();
    let Ok(body) = axum::body::to_bytes(body, MAX_SIGNED_BODY).await else {
        return StatusCode::FORBIDDEN.into_response();
    };
    // Decided before anything is awaited, so no borrow of the request crosses an await.
    let verdict = {
        let host = parts
            .uri
            .authority()
            .map(|a| a.as_str().to_string())
            .or_else(|| {
                parts
                    .headers
                    .get(axum::http::header::HOST)
                    .and_then(|h| h.to_str().ok())
                    .map(str::to_string)
            })
            .unwrap_or_default();
        let path_and_query = parts
            .uri
            .path_and_query()
            .map(|p| p.as_str())
            .unwrap_or("/");
        let target_uri = format!("https://{host}{path_and_query}");
        let headers = parts.headers.clone();
        let header = move |name: &str| -> Option<String> {
            let values: Vec<&str> = headers
                .get_all(name)
                .iter()
                .filter_map(|v| v.to_str().ok())
                .map(str::trim)
                .collect();
            (!values.is_empty()).then(|| values.join(", "))
        };
        let signed = SignedRequest {
            method: parts.method.as_str(),
            target_uri: &target_uri,
            authority: &host,
            path: parts.uri.path(),
            query: parts.uri.query(),
            header: &header,
            body: &body,
        };
        verify(&signed, SystemTime::now(), keys)
    };
    match verdict {
        Ok(domain) => {
            parts
                .extensions
                .insert(PeerCert(Some(PeerIdentity::signed(&domain))));
            next.run(Request::from_parts(parts, Body::from(body))).await
        }
        Err(refusal) => {
            eprintln!(
                "[mimi-hubd] provider auth refused {} {}: {refusal:?}",
                parts.method,
                parts.uri.path()
            );
            StatusCode::FORBIDDEN.into_response()
        }
    }
}

#[cfg(test)]
#[path = "provider_auth_tests.rs"]
mod tests;
