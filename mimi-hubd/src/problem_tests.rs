use axum::http::{header, StatusCode};

use super::*;

#[tokio::test]
async fn a_problem_is_answered_with_its_status_media_type_and_body() {
    let response = REMOTE_ROOM_NOT_DELIVERED.response();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        PROBLEM_JSON
    );
    let bytes = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        body["type"],
        "https://github.com/havenmessenger/interop/blob/main/docs/problems.md#remote-room-not-delivered"
    );
    assert_eq!(body["status"], 422);
    assert_eq!(body["title"], REMOTE_ROOM_NOT_DELIVERED.title);
    assert_eq!(body["detail"], REMOTE_ROOM_NOT_DELIVERED.detail);
}

#[test]
fn every_problem_anchor_is_on_the_problems_page() {
    let page = include_str!("../../docs/problems.md");
    for problem in [&REMOTE_ROOM_NOT_DELIVERED] {
        assert!(
            page.contains(&format!("## {}\n", problem.anchor)),
            "docs/problems.md has no heading for {}",
            problem.anchor
        );
        assert!(
            page.contains(&problem.uri()),
            "docs/problems.md does not give {}'s URI",
            problem.anchor
        );
        assert!(
            page.contains(problem.title),
            "docs/problems.md does not give {}'s title",
            problem.anchor
        );
    }
}
