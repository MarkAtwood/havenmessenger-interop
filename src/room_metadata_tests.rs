//! `RoomMetaData`: what is read and kept, and each bound that refuses it.

use super::*;

fn sample() -> RoomMetadata {
    RoomMetadata {
        room_uri: "mimi://a.example/r/one".into(),
        room_name: "Planning".into(),
        descriptions: vec![RichDescription {
            media_type: String::new(),
            language_tag: "en".into(),
            content: b"Where the table plans the week".to_vec(),
        }],
        avatar: "https://a.example/avatar.png".into(),
        subject: "IETF 127".into(),
        mood: String::new(),
    }
}

#[test]
fn a_room_metadata_reads_back_as_written() {
    let bytes = encode_room_metadata(&sample());
    assert_eq!(decode_room_metadata(&bytes).unwrap(), sample());
    let empty = RoomMetadata {
        room_uri: "mimi://a.example/r/one".into(),
        room_name: String::new(),
        descriptions: Vec::new(),
        avatar: String::new(),
        subject: String::new(),
        mood: String::new(),
    };
    assert_eq!(
        decode_room_metadata(&encode_room_metadata(&empty)).unwrap(),
        empty
    );
}

#[test]
fn each_bound_refuses() {
    let mut avatar = sample();
    avatar.avatar = "http://a.example/avatar.png".into();
    assert!(decode_room_metadata(&encode_room_metadata(&avatar)).is_err());
    let mut nul = sample();
    nul.room_name = "Plan\0ning".into();
    assert!(decode_room_metadata(&encode_room_metadata(&nul)).is_err());
    let mut long = sample();
    long.subject = "x".repeat(1025);
    assert!(decode_room_metadata(&encode_room_metadata(&long)).is_err());
    let mut many = sample();
    many.descriptions = vec![many.descriptions[0].clone(); 17];
    assert!(decode_room_metadata(&encode_room_metadata(&many)).is_err());
    let mut trailing = encode_room_metadata(&sample());
    trailing.push(0);
    assert!(decode_room_metadata(&trailing).is_err());
    assert!(decode_room_metadata(&vec![0u8; MAX_ROOM_METADATA + 1]).is_err());
    let mut not_utf8 = encode_room_metadata(&sample());
    let at = not_utf8.iter().position(|b| *b == b'P').unwrap();
    not_utf8[at] = 0xFF;
    assert!(decode_room_metadata(&not_utf8).is_err());
}
