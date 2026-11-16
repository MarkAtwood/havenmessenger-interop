//! RFC 9457 problem details for the refusals that carry one. Each problem type is named by a URI into
//! `docs/problems.md`, the page that says what it means. A problem body names the condition only: never a user, a
//! client or a room.

use axum::{
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Serialize;

/// The media type of a problem body (RFC 9457 §3).
pub const PROBLEM_JSON: &str = "application/problem+json";

/// The page each problem type's URI points into.
const PROBLEMS_PAGE: &str = "https://github.com/havenmessenger/interop/blob/main/docs/problems.md";

/// A problem type: its anchor on the problems page, its title and the status it is answered with.
pub struct ProblemType {
    pub anchor: &'static str,
    pub title: &'static str,
    pub status: StatusCode,
    pub detail: &'static str,
}

/// A `notify` (protocol-07 §5.5) carrying an application message or a Commit for a room hosted on another hub: this hub
/// does not deliver those yet, so it refuses them instead of accepting what it would not deliver.
pub const REMOTE_ROOM_NOT_DELIVERED: ProblemType = ProblemType {
    anchor: "remote-room-not-delivered",
    title: "Not delivered: the room is hosted on another hub",
    status: StatusCode::UNPROCESSABLE_ENTITY,
    detail: "This hub does not yet deliver application messages or Commits for rooms hosted on another hub.",
};

#[derive(Serialize)]
struct Body {
    #[serde(rename = "type")]
    kind: String,
    title: &'static str,
    status: u16,
    detail: &'static str,
}

impl ProblemType {
    /// The problem type's URI.
    pub fn uri(&self) -> String {
        format!("{PROBLEMS_PAGE}#{}", self.anchor)
    }

    /// The answer: the problem's status, `Content-Type: application/problem+json`, and the body.
    pub fn response(&self) -> Response {
        let body = Body {
            kind: self.uri(),
            title: self.title,
            status: self.status.as_u16(),
            detail: self.detail,
        };
        let json = serde_json::to_vec(&body).unwrap_or_default();
        let mut response = (self.status, json).into_response();
        response
            .headers_mut()
            .insert(header::CONTENT_TYPE, HeaderValue::from_static(PROBLEM_JSON));
        response
    }
}

#[cfg(test)]
#[path = "problem_tests.rs"]
mod tests;
