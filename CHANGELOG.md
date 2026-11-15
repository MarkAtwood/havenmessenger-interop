# Changelog

This project uses 0.x semantic versions: a minor version changes the wire format or the public API, a patch changes
anything else. Each release names the drafts it implements and the divergences from them that remain open; the reasons
are in [DIVERGENCES.md](DIVERGENCES.md).

## 0.0.2 - 2026-11-15

Implements draft-ietf-mimi-protocol-07, draft-ietf-mimi-content-10, draft-ietf-mimi-room-policy-04,
draft-ietf-mls-extensions-10 and draft-atwood-mimi-provider-auth-00 (DNS-published keys).

### Protocol
- The hub follows protocol-07: wire answers carry the protocol media type; `update` takes an `UpdateRequest` and answers
  with an `UpdateRoomResponse`; a Commit may carry a participant-list change, a room-policy change, or one of each, with
  Add and Remove proposals beside it, and a member may leave through proposals the next Commit includes by reference.
- A Commit's epoch is checked, and a Commit refused for its changes says which rule refused it.
- Participant-list updates apply in the room's own order (protocol-07 §7.5).
- Room policy is read from `roles_list` and `base_room_policy` AppDataUpdates (room-policy-04).
- `identifierQuery` answers with an `IdentifierResponse`: a handle search takes the whole handle URI
  (`mimi://<domain>/u/<name>` or `im:<name>@<domain>`), a nick search the bare name, and every miss is the same
  `notFound` with HTTP 200.
- `keyMaterial` serves a KeyPackage that meets the request's ciphersuites and capabilities; KeyPackages are published
  only for this hub's own users or by the provider that holds them.
- A notify is accepted only from the hub of its room; a fanned-out Welcome is queued for the local user it names.
- content-10 is read and written; `content_check` decodes another implementation's content documents and compares the
  re-encoding.

### Identity and URIs
- URIs are compared in their canonical form (RFC 3986 §6.2.2), and a path written as an IRI is mapped to its URI
  (RFC 3987 §3.1), so a name written raw and the same name percent-encoded are one user. A found user is answered with
  the name percent-encoded. An existing store is rewritten to this form once, at its next start.
- An authority outside ASCII is refused.

### Provider authentication
- A peer is authenticated by a client certificate that chains to the configured CA, or, with no certificate, by an
  RFC 9421 signature from a key its provider publishes in DNS (draft-atwood-mimi-provider-auth §5). One gate decides
  this for every route but the directory; a request with neither is answered 403. Keys are read from the file named by
  `MIMI_PEER_KEYS`, which a resolver on the host keeps current.
- The fingerprint of each client CA is printed at startup.

### Operating the hub
- `mimi-hubd enroll` lets the operator enroll a handle for `identifierQuery` from the hub's host.
- An optional listener serves the directory and nothing else; an optional activity line logs each peer request's
  method, route template and status.
- The startup line names the version and the git commit it was built from.

### Tooling
- Protocol-07 vectors decode against any -07 implementation; `seed_fixtures` seeds a hub for a visiting implementation.
- CI refuses attribution trailers in this project's own commit messages; an outside contributor's message is kept as
  written.
- Dependencies: openmls on a main revision (GREASE KeyPackages pass the gate), rusqlite 0.41, rustls 0.23.46,
  rustls-webpki 0.103.16 (RUSTSEC-2026-0382).

### Open divergences
DIV-1 (no external commits, by design), DIV-2 (one ciphersuite), DIV-3 (no assets or OHTTP), DIV-4 (identifierQuery
by opt-in only), DIV-5 (no relay to other providers), DIV-6 (client identity not bound to the channel), DIV-8
(metadata-only abuse reports), DIV-9 (no franking), DIV-10 (role checks on a Commit's sender), DIV-11, DIV-12, DIV-13,
DIV-15, DIV-16, DIV-17. See DIVERGENCES.md for each.

## 0.0.1 - 2026-07-16

First release: the mimi-core library (protocol-06 wire codecs, consent, participant lists, room policy, content
codec) and the mimi-hubd reference hub over mutually authenticated TLS, with binaries, `.deb` packages and a container
image.
