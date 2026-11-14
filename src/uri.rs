//! MIMI identifier URIs (draft-ietf-mimi-protocol-07 §4) - the ADDRESSING primitive.
//!
//! A MIMI identifier is a URI whose **authority is the destination provider** and whose first path
//! segment is the entity kind. This is how a provider knows WHERE a message goes (the authority) and
//! WHAT it addresses (user / room / device) - the foundation of inter-provider routing. Discovery
//! (fetching the authority's /.well-known/mimi-protocol-directory) is a separate step (see the provider).
//!
//!   provider : mimi://a.example
//!   user     : mimi://a.example/u/alice-smith
//!   room     : mimi://a.example/r/engineering_team
//!   device   : mimi://a.example/d/3b52249d-68f9-45ce-8bf5-c799f3cad7ec/0003   (multi-segment path)
//!
//! Verified against the URI forms used in the draft + its examples/ instance documents.
//! This is the conformance-grade parser: KATs round-trip the official example URIs byte-exact.

use std::fmt;

/// Typed parse errors for [`MimiUri::parse`] (`thiserror` per-module enum - the
/// library convention, vs `anyhow` which is app-idiom). Each variant is a distinct malformed-input class.
#[derive(Debug, thiserror::Error)]
pub enum UriError {
    /// The string did not start with the `mimi://` scheme.
    #[error("not a mimi:// URI: {0:?}")]
    NotMimiScheme(String),
    /// `mimi://` with no authority (host) before the first `/`.
    #[error("mimi URI has an empty authority: {0:?}")]
    EmptyAuthority(String),
    /// An entity tag (`/u`, `/r`, `/d`) with no following path segment.
    #[error("mimi URI has an entity tag but no path: {0:?}")]
    TagWithoutPath(String),
    /// The first path segment was not a known entity tag (`u`/`r`/`d`).
    #[error("unknown MIMI entity tag {tag:?} in {uri:?}")]
    UnknownEntityTag { tag: String, uri: String },
    /// The entity tag was present and recognized, but the path after it was empty.
    #[error("mimi URI entity path is empty: {0:?}")]
    EmptyEntityPath(String),
    /// The authority has a character outside ASCII. Protocol-07 names a provider by an RFC 3986 host, which is ASCII:
    /// an internationalized domain is written in A-labels (`xn--`).
    #[error("mimi URI authority is not ASCII: {0:?}")]
    NonAsciiAuthority(String),
}

/// The entity a MIMI URI addresses. The first path segment selects it (`u` / `r` / `d`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MimiKind {
    /// `/u/...` - a user.
    User,
    /// `/r/...` - a room.
    Room,
    /// `/d/...` - a device/client.
    Device,
}

impl MimiKind {
    const fn tag(self) -> &'static str {
        match self {
            Self::User => "u",
            Self::Room => "r",
            Self::Device => "d",
        }
    }

    fn from_tag(s: &str) -> Option<Self> {
        match s {
            "u" => Some(Self::User),
            "r" => Some(Self::Room),
            "d" => Some(Self::Device),
            _ => None,
        }
    }
}

/// A parsed MIMI identifier URI. `authority` is the destination provider (the routing key); `path` is
/// everything after `/{kind}/` (kept verbatim - may contain multiple `/`-separated segments, as device
/// URIs do). A provider-only URI (`mimi://a.example`, no entity) parses with `kind == None`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MimiUri {
    pub authority: String,
    pub kind: Option<MimiKind>,
    pub path: String,
}

impl MimiUri {
    /// Parse a `mimi://` URI. Rejects: a non-`mimi` scheme, an empty authority, an unknown entity tag,
    /// and an entity tag with no following path. A bare provider URI (`mimi://a.example`, optionally a
    /// trailing slash) is accepted with `kind == None`.
    pub fn parse(s: &str) -> Result<Self, UriError> {
        // The scheme compares without case (RFC 3986 §3.1); the rest of the parse reads the URI as given.
        let rest = s
            .get(..7)
            .filter(|scheme| scheme.eq_ignore_ascii_case("mimi://"))
            .map(|_| &s[7..])
            .ok_or_else(|| UriError::NotMimiScheme(s.to_string()))?;
        // authority = up to the first '/', the remainder (if any) is the path part.
        let (authority, after) = rest
            .find('/')
            .map_or((rest, ""), |i| (&rest[..i], &rest[i + 1..]));
        if authority.is_empty() {
            return Err(UriError::EmptyAuthority(s.to_string()));
        }
        if !authority.is_ascii() {
            return Err(UriError::NonAsciiAuthority(s.to_string()));
        }
        // Bare provider URI: mimi://a.example  or  mimi://a.example/
        if after.is_empty() {
            return Ok(Self {
                authority: authority.to_string(),
                kind: None,
                path: String::new(),
            });
        }
        // entity: /{tag}/{path...}
        let (tag, path) = match after.split_once('/') {
            Some((t, p)) => (t, p),
            None => return Err(UriError::TagWithoutPath(s.to_string())),
        };
        let kind = MimiKind::from_tag(tag).ok_or_else(|| UriError::UnknownEntityTag {
            tag: tag.to_string(),
            uri: s.to_string(),
        })?;
        if path.is_empty() {
            return Err(UriError::EmptyEntityPath(s.to_string()));
        }
        Ok(Self {
            authority: authority.to_string(),
            kind: Some(kind),
            path: path.to_string(),
        })
    }

    /// True when this URI's authority is the given provider (case-insensitive host compare). The routing
    /// decision: a provider serves local ops only for URIs whose authority is itself.
    pub fn is_local_to(&self, provider_authority: &str) -> bool {
        self.authority.eq_ignore_ascii_case(provider_authority)
    }
}

impl fmt::Display for MimiUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            Some(k) => write!(f, "mimi://{}/{}/{}", self.authority, k.tag(), self.path),
            None => write!(f, "mimi://{}", self.authority),
        }
    }
}

/// The canonical form of a `mimi://` URI (RFC 3986 §6.2.2): the scheme and the authority in lowercase, a percent-encoded
/// unreserved character (ALPHA, DIGIT, `-`, `.`, `_`, `~`) decoded, and any other percent-encoding's hex digits in
/// uppercase. The path keeps its case. Two URIs that name the same thing have the same canonical form, so a store keyed by
/// it finds `MIMI://A.Example/u/%61lice` under `mimi://a.example/u/alice`. Anything that is not a `mimi://` URI (a bare
/// user name, for one) is returned as given.
pub fn canonical(s: &str) -> String {
    let Some(rest) = s
        .get(..7)
        .filter(|scheme| scheme.eq_ignore_ascii_case("mimi://"))
        .map(|_| &s[7..])
    else {
        return s.to_string();
    };
    let (authority, path) = rest.find('/').map_or((rest, ""), |i| rest.split_at(i));
    let mut out = String::with_capacity(s.len());
    out.push_str("mimi://");
    out.push_str(&normalize_percent(authority).to_ascii_lowercase());
    out.push_str(&normalize_percent(path));
    out
}

/// Decode each percent-encoded unreserved character and uppercase the hex digits of every other percent-encoding. Bytes
/// outside a percent-encoding are kept as they are, so a path in any script comes through unchanged.
fn normalize_percent(part: &str) -> String {
    let bytes = part.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let escape = (bytes[i] == b'%')
            .then(|| bytes.get(i + 1..i + 3))
            .flatten()
            .filter(|hex| hex.iter().all(u8::is_ascii_hexdigit));
        match escape {
            Some(hex) => {
                let value =
                    u8::from_str_radix(std::str::from_utf8(hex).unwrap_or("00"), 16).unwrap_or(0);
                if value.is_ascii_alphanumeric() || matches!(value, b'-' | b'.' | b'_' | b'~') {
                    out.push(value);
                } else {
                    out.push(b'%');
                    out.extend(hex.iter().map(u8::to_ascii_uppercase));
                }
                i += 3;
            }
            None => {
                out.push(bytes[i]);
                i += 1;
            }
        }
    }
    // Only ASCII was written in place of an escape, so the result is as valid UTF-8 as the input.
    String::from_utf8(out).unwrap_or_else(|_| part.to_string())
}

#[cfg(test)]
#[path = "uri_canonical_tests.rs"]
mod canonical_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_official_example_uris() {
        let u = MimiUri::parse("mimi://example.com/u/alice-smith").unwrap();
        assert_eq!(u.authority, "example.com");
        assert_eq!(u.kind, Some(MimiKind::User));
        assert_eq!(u.path, "alice-smith");

        let r = MimiUri::parse("mimi://example.com/r/engineering_team").unwrap();
        assert_eq!(r.authority, "example.com");
        assert_eq!(r.kind, Some(MimiKind::Room));

        // device URI from examples/implied-original.cbor - multi-segment path kept verbatim.
        let d = MimiUri::parse("mimi://example.com/d/3b52249d-68f9-45ce-8bf5-c799f3cad7ec/0003")
            .unwrap();
        assert_eq!(d.authority, "example.com");
        assert_eq!(d.kind, Some(MimiKind::Device));
        assert_eq!(d.path, "3b52249d-68f9-45ce-8bf5-c799f3cad7ec/0003");
    }

    #[test]
    fn roundtrips_byte_exact() {
        for s in [
            "mimi://example.com/u/alice-smith",
            "mimi://example.com/r/engineering_team",
            "mimi://example.com/d/3b52249d-68f9-45ce-8bf5-c799f3cad7ec/0003",
            "mimi://a.example",
        ] {
            assert_eq!(
                MimiUri::parse(s).unwrap().to_string(),
                s,
                "round-trip must be byte-exact: {s}"
            );
        }
    }

    #[test]
    fn authority_is_the_destination() {
        let u = MimiUri::parse("mimi://mimi-b.havenmessenger.com/u/bob").unwrap();
        assert_eq!(u.authority, "mimi-b.havenmessenger.com");
        assert!(u.is_local_to("mimi-b.havenmessenger.com"));
        assert!(u.is_local_to("MIMI-B.HavenMessenger.com")); // host compare is case-insensitive
        assert!(!u.is_local_to("mimi.havenmessenger.com"));
    }

    #[test]
    fn bare_provider_uri_has_no_kind() {
        let p = MimiUri::parse("mimi://a.example").unwrap();
        assert_eq!(p.authority, "a.example");
        assert_eq!(p.kind, None);
        assert_eq!(MimiUri::parse("mimi://a.example/").unwrap(), p); // trailing slash == bare
    }

    #[test]
    fn rejects_malformed() {
        assert!(
            MimiUri::parse("https://example.com/u/alice").is_err(),
            "non-mimi scheme"
        );
        assert!(
            MimiUri::parse("mimi:///u/alice").is_err(),
            "empty authority"
        );
        assert!(
            MimiUri::parse("mimi://example.com/x/thing").is_err(),
            "unknown entity tag"
        );
        assert!(
            MimiUri::parse("mimi://example.com/u/").is_err(),
            "empty entity path"
        );
        assert!(
            MimiUri::parse("mimi://example.com/u").is_err(),
            "tag without path"
        );
        assert!(MimiUri::parse("not-a-uri").is_err(), "no scheme");
    }
}
