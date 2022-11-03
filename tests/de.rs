use bt_bencode::{Error, Value};
use serde::Deserialize;
use serde_derive::Deserialize;

static TORRENT_BYTES: &[u8] = include_bytes!("ubuntu-20.04.4-live-server-amd64.iso.torrent");

#[derive(Debug, Deserialize, Clone, Eq, Hash, PartialEq)]
struct TorrentFile {
    announce: String,
}

#[test]
fn test_deserialize_torrent_file_via_type() -> Result<(), Error> {
    let torrent_file: TorrentFile = bt_bencode::from_slice(TORRENT_BYTES)?;

    assert_eq!(torrent_file.announce, "https://torrent.ubuntu.com/announce");

    Ok(())
}

#[test]
fn test_deserialize_torrent_file_via_value() -> Result<(), Error> {
    let decoded_value: Value = bt_bencode::from_slice(TORRENT_BYTES)?;

    let announce = decoded_value
        .get("announce")
        .and_then(bt_bencode::Value::as_byte_str)
        .map(|v| v.to_vec());

    assert_eq!(
        announce,
        Some(String::from("https://torrent.ubuntu.com/announce").into_bytes())
    );

    Ok(())
}

#[test]
fn test_deserialize_torrent_file_via_value_index() -> Result<(), Error> {
    let decoded_value: Value = bt_bencode::from_slice(TORRENT_BYTES)?;

    let announce = decoded_value["announce"].as_str();

    assert_eq!(announce, Some("https://torrent.ubuntu.com/announce"));

    Ok(())
}

#[test]
fn test_deserialize_info_hash() -> Result<(), Error> {
    use sha1::Digest;

    #[derive(Deserialize)]
    struct Metainfo {
        info: serde_bytes::ByteBuf,
    }

    let metainfo: Metainfo = bt_bencode::from_slice(TORRENT_BYTES)?;

    let mut hasher = sha1::Sha1::new();
    hasher.update(&metainfo.info);
    let orig_info_hash = hasher.finalize();

    assert_eq!(
        orig_info_hash.as_slice(),
        &[
            0xb4, 0x4a, 0x0e, 0x20, 0xfa, 0x5b, 0x7c, 0xec, 0xb7, 0x71, 0x56, 0x33, 0x3b, 0x42,
            0x68, 0xdf, 0xd7, 0xc3, 0x0a, 0xfb
        ]
    );

    let info: Value = bt_bencode::from_slice(&metainfo.info).unwrap();

    // Need to verify the value is actually a dictionary. The ByteBuf could have been any value.
    assert!(info.is_dict());

    // Verify that a round-trip decoding and encoding produces the same info hash.
    // The re-encoding ensures that the original data was encoded correctly
    // according to bencode rules (ordering of keys, no leading zeros, etc.)
    let re_encoded_bytes: Vec<u8> = bt_bencode::to_vec(&info).unwrap();
    let mut hasher = sha1::Sha1::new();
    hasher.update(&re_encoded_bytes);
    let re_encoded_info_hash = hasher.finalize();
    assert_eq!(orig_info_hash, re_encoded_info_hash);

    assert_eq!(
        info.get("piece length").and_then(bt_bencode::Value::as_u64),
        Some(262_144)
    );
    assert_eq!(
        info.get("pieces")
            .and_then(bt_bencode::Value::as_byte_str)
            .map(|v| v.len()),
        Some(101_600)
    );
    assert_eq!(
        info.get("length").and_then(bt_bencode::Value::as_u64),
        Some(1_331_691_520)
    );

    Ok(())
}

#[test]
fn test_deserialize_info_hash_borrowed() -> Result<(), Error> {
    use sha1::Digest;

    #[derive(Deserialize)]
    struct Metainfo<'a> {
        info: &'a [u8],
    }

    #[derive(Deserialize)]
    struct Info<'a> {
        name: Option<&'a str>,
        pieces: &'a [u8],
    }

    let metainfo: Metainfo = bt_bencode::from_slice(TORRENT_BYTES)?;

    let mut hasher = sha1::Sha1::new();
    hasher.update(metainfo.info);
    let orig_info_hash = hasher.finalize();

    assert_eq!(
        orig_info_hash.as_slice(),
        &[
            0xb4, 0x4a, 0x0e, 0x20, 0xfa, 0x5b, 0x7c, 0xec, 0xb7, 0x71, 0x56, 0x33, 0x3b, 0x42,
            0x68, 0xdf, 0xd7, 0xc3, 0x0a, 0xfb
        ]
    );

    let info: Value = bt_bencode::from_slice(metainfo.info).unwrap();

    // Need to verify the value is actually a dictionary. The ByteBuf could have been any value.
    assert!(info.is_dict());

    // Verify that a round-trip decoding and encoding produces the same info hash.
    // The re-encoding ensures that the original data was encoded correctly
    // according to bencode rules (ordering of keys, no leading zeros, etc.)
    let re_encoded_bytes: Vec<u8> = bt_bencode::to_vec(&info).unwrap();
    let mut hasher = sha1::Sha1::new();
    hasher.update(&re_encoded_bytes);
    let re_encoded_info_hash = hasher.finalize();
    assert_eq!(orig_info_hash, re_encoded_info_hash);

    assert_eq!(
        info.get("piece length").and_then(bt_bencode::Value::as_u64),
        Some(262_144)
    );
    assert_eq!(
        info.get("pieces")
            .and_then(bt_bencode::Value::as_byte_str)
            .map(|v| v.len()),
        Some(101_600)
    );
    assert_eq!(
        info.get("length").and_then(bt_bencode::Value::as_u64),
        Some(1_331_691_520)
    );

    let info = Info::deserialize(&info)?;

    assert_eq!(info.name, Some("ubuntu-20.04.4-live-server-amd64.iso"));
    assert_eq!(info.pieces.len(), 101_600);

    Ok(())
}
