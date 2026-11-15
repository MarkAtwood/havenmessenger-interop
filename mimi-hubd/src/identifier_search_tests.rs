use super::enrolled_name;
use mimi_core::protocol_wire::SearchIdentifierType::{Email, Handle, Nick};

const HUB: &str = "hub.example";

fn handle(value: &str) -> Option<String> {
    enrolled_name(Handle, value, HUB)
}

#[test]
fn a_nick_is_the_bare_name() {
    assert_eq!(enrolled_name(Nick, "erin", HUB).as_deref(), Some("erin"));
    assert_eq!(enrolled_name(Nick, "", HUB), None);
    assert_eq!(enrolled_name(Nick, "mimi://hub.example/u/erin", HUB), None);
    assert_eq!(enrolled_name(Nick, "im:erin@hub.example", HUB), None);
}

#[test]
fn a_handle_is_a_mimi_user_uri_on_this_hub() {
    assert_eq!(handle("mimi://hub.example/u/erin").as_deref(), Some("erin"));
    assert_eq!(handle("MIMI://HUB.example/u/erin").as_deref(), Some("erin"));
    assert_eq!(
        handle("mimi://hub.example/u/%65rin").as_deref(),
        Some("erin")
    );
}

#[test]
fn a_handle_is_an_im_uri_on_this_hub() {
    assert_eq!(handle("im:erin@hub.example").as_deref(), Some("erin"));
    assert_eq!(handle("IM:erin@HUB.example").as_deref(), Some("erin"));
}

#[test]
fn a_handle_names_nobody_here_when_it_is_not_a_user_uri_of_this_hub() {
    // PLANT-HANDLE-FOREIGN: another provider's user is not ours, in either form.
    assert_eq!(
        handle("mimi://other.example/u/erin"),
        None,
        "PLANT-HANDLE-FOREIGN"
    );
    assert_eq!(
        handle("im:erin@other.example"),
        None,
        "PLANT-HANDLE-FOREIGN"
    );
    // A room or a device is not a user; a user path with more segments is not one name.
    assert_eq!(handle("mimi://hub.example/r/erin"), None);
    assert_eq!(handle("mimi://hub.example/d/erin/1"), None);
    assert_eq!(handle("mimi://hub.example/u/erin/x"), None);
    // An empty name, or a host smuggled into the name.
    assert_eq!(handle("im:@hub.example"), None);
    assert_eq!(handle("im:a@b@hub.example"), None);
}

#[test]
fn a_bare_name_is_not_a_handle() {
    // PLANT-HANDLE-BARE: a handle search matches the entire handle URI, so a bare name answers notFound.
    assert_eq!(handle("erin"), None, "PLANT-HANDLE-BARE");
}

#[test]
fn other_search_types_name_nobody() {
    assert_eq!(enrolled_name(Email, "erin@hub.example", HUB), None);
}
