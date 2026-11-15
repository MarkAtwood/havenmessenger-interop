//! The `room_metadata` app component (protocol-07 §10, component `0x0023`): a room's name, descriptions, avatar,
//! subject and mood. It is display data: read with bounds and kept as sent, and it governs nothing a hub enforces.
//!
//! A room whose members set any of it requires every member to support the component (mls-extensions §5), so a
//! provider that refuses it cannot have its users in such a room.

use tls_codec::{Serialize as _, VLBytes};

use crate::room_policy::RoomPolicyError;
use crate::room_policy_wire::{refuse, Reader};

/// The component id (protocol-07 §10.4).
pub const ROOM_METADATA_COMPONENT_ID: u16 = 0x0023;
/// The private proposal type an AppDataUpdate to `room_metadata` is read as, beside the participant list (`0xF7A0`) and
/// the two room-policy components (`0xF7A3`, `0xF7A4`).
pub const MIMI_ROOM_METADATA_PROPOSAL_TYPE: u16 = 0xF7A5;
/// The largest `RoomMetaData` a hub keeps.
pub const MAX_ROOM_METADATA: usize = 16 * 1024;
const MAX_URI: usize = 2048;
const MAX_TEXT: usize = 1024;
const MAX_MEDIA_TYPE: usize = 128;
const MAX_LANGUAGE_TAG: usize = 64;
const MAX_DESCRIPTION: usize = 4096;
const MAX_DESCRIPTIONS: usize = 16;

/// One `RichDescription`: its media type (empty for UTF-8 plain text), language tag and content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RichDescription {
    pub media_type: String,
    pub language_tag: String,
    pub content: Vec<u8>,
}

/// A room's `RoomMetaData`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomMetadata {
    pub room_uri: String,
    pub room_name: String,
    pub descriptions: Vec<RichDescription>,
    /// An `https` URI of the room's avatar image, or empty.
    pub avatar: String,
    pub subject: String,
    pub mood: String,
}

/// Read a `RoomMetaData` (the `update` of an AppDataUpdate to this component). Refused when it is larger than
/// [`MAX_ROOM_METADATA`], when a field passes its bound or is not UTF-8 without NUL, when the avatar is neither empty
/// nor an `https` URI, or when bytes are left over.
pub fn decode_room_metadata(bytes: &[u8]) -> Result<RoomMetadata, RoomPolicyError> {
    const WHAT: &str = "RoomMetaData";
    if bytes.len() > MAX_ROOM_METADATA {
        return Err(refuse(
            WHAT,
            format!("{} octets, at most {MAX_ROOM_METADATA}", bytes.len()),
        ));
    }
    let mut reader = Reader::new(bytes);
    let room_uri = text(&mut reader, "RoomMetaData.room_uri", MAX_URI)?;
    let room_name = text(&mut reader, "RoomMetaData.room_name", MAX_TEXT)?;
    let mut list = reader.run("RoomMetaData.room_descriptions", MAX_ROOM_METADATA)?;
    let mut descriptions = Vec::new();
    while !list.is_empty() {
        if descriptions.len() == MAX_DESCRIPTIONS {
            return Err(refuse(
                WHAT,
                format!("more than {MAX_DESCRIPTIONS} descriptions"),
            ));
        }
        let media_type = text(&mut list, "RichDescription.media_type", MAX_MEDIA_TYPE)?;
        let language_tag = text(&mut list, "RichDescription.language_tag", MAX_LANGUAGE_TAG)?;
        let content = list
            .opaque("RichDescription.description_content", MAX_DESCRIPTION)?
            .to_vec();
        descriptions.push(RichDescription {
            media_type,
            language_tag,
            content,
        });
    }
    let avatar = text(&mut reader, "RoomMetaData.room_avatar", MAX_URI)?;
    if !avatar.is_empty() && !avatar.starts_with("https://") {
        return Err(refuse(WHAT, "an avatar that is not an https URI"));
    }
    let subject = text(&mut reader, "RoomMetaData.room_subject", MAX_TEXT)?;
    let mood = text(&mut reader, "RoomMetaData.room_mood", MAX_TEXT)?;
    if !reader.is_empty() {
        return Err(refuse(WHAT, "bytes after the last field"));
    }
    Ok(RoomMetadata {
        room_uri,
        room_name,
        descriptions,
        avatar,
        subject,
        mood,
    })
}

/// Write a `RoomMetaData`, the form [`decode_room_metadata`] reads.
pub fn encode_room_metadata(metadata: &RoomMetadata) -> Vec<u8> {
    let vl = |bytes: &[u8]| {
        VLBytes::new(bytes.to_vec())
            .tls_serialize_detached()
            .expect("a vector this size encodes")
    };
    let mut descriptions = Vec::new();
    for d in &metadata.descriptions {
        descriptions.extend(vl(d.media_type.as_bytes()));
        descriptions.extend(vl(d.language_tag.as_bytes()));
        descriptions.extend(vl(&d.content));
    }
    [
        vl(metadata.room_uri.as_bytes()),
        vl(metadata.room_name.as_bytes()),
        vl(&descriptions),
        vl(metadata.avatar.as_bytes()),
        vl(metadata.subject.as_bytes()),
        vl(metadata.mood.as_bytes()),
    ]
    .concat()
}

fn text(
    reader: &mut Reader<'_>,
    what: &'static str,
    limit: usize,
) -> Result<String, RoomPolicyError> {
    let bytes = reader.opaque(what, limit)?;
    if bytes.contains(&0) {
        return Err(refuse(what, "a NUL"));
    }
    String::from_utf8(bytes.to_vec()).map_err(|_| refuse(what, "not valid UTF-8"))
}

#[cfg(test)]
#[path = "room_metadata_tests.rs"]
mod tests;
