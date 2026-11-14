//! `canonical`: one form for every spelling of the same `mimi://` URI (RFC 3986 §6.2.2), and `parse` taking the
//! scheme in any case.

use super::*;

#[test]
fn scheme_and_authority_case_and_unreserved_escapes_come_to_one_form() {
    for spelling in [
        "mimi://a.example/u/alice",
        "MIMI://A.Example/u/alice",
        "Mimi://a.EXAMPLE/u/%61lice",
        "mimi://a.example/u/%61%6C%69%63%65",
        "mimi://a.example/u/%61%6c%69%63%65",
    ] {
        assert_eq!(
            canonical(spelling),
            "mimi://a.example/u/alice",
            "{spelling}"
        );
    }
}

#[test]
fn the_path_keeps_its_case_and_reserved_escapes_keep_their_meaning() {
    assert_eq!(
        canonical("mimi://a.example/u/Alice"),
        "mimi://a.example/u/Alice"
    );
    // `/` encoded stays encoded (it is reserved); its hex digits become uppercase.
    assert_eq!(
        canonical("mimi://a.example/r/a%2fb"),
        "mimi://a.example/r/a%2Fb"
    );
    // A broken escape is kept as it is.
    assert_eq!(
        canonical("mimi://a.example/u/50%"),
        "mimi://a.example/u/50%"
    );
    assert_eq!(
        canonical("mimi://a.example/u/a%zz"),
        "mimi://a.example/u/a%zz"
    );
}

#[test]
fn a_path_outside_ascii_comes_through_unchanged() {
    assert_eq!(
        canonical("mimi://A.example/u/zoë"),
        "mimi://a.example/u/zoë"
    );
}

#[test]
fn what_is_not_a_mimi_uri_is_returned_as_given() {
    assert_eq!(canonical("alice"), "alice");
    assert_eq!(
        canonical("https://a.example/u/alice"),
        "https://a.example/u/alice"
    );
    assert_eq!(canonical(""), "");
}

#[test]
fn canonical_is_idempotent_and_parse_takes_any_scheme_case() {
    let once = canonical("MIMI://A.Example/r/%74able");
    assert_eq!(canonical(&once), once);
    let parsed = MimiUri::parse("MIMI://a.example/u/alice").unwrap();
    assert_eq!(parsed.kind, Some(MimiKind::User));
}

#[test]
fn a_host_outside_ascii_is_refused_and_its_a_label_form_is_not() {
    for uri in [
        "mimi://例え.jp/u/alice",
        "mimi://\u{FEFF}/u/alice",
        "mimi://\u{FFFE}/r/room",
    ] {
        assert!(
            matches!(MimiUri::parse(uri), Err(UriError::NonAsciiAuthority(_))),
            "{uri}"
        );
    }
    assert!(MimiUri::parse("mimi://xn--r8jz45g.jp/u/alice").is_ok());
    // The path may hold any script.
    assert!(MimiUri::parse("mimi://a.example/u/例え").is_ok());
}
