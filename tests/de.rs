#[macro_use]
extern crate serde_derive;

use bt_bencode::{Error, Value};
use serde_bytes::ByteBuf;

#[derive(Debug, Deserialize, Clone, Eq, Hash, PartialEq)]
struct TorrentFile {
    announce: String,
}

#[test]
fn test_deserialize_torrent_file_via_type() -> Result<(), Error> {
    let torrent_bytes = include_bytes!("ubuntu-18.04.3-live-server-amd64.iso.torrent");
    let torrent_file: TorrentFile = bt_bencode::from_slice(&torrent_bytes[..])?;

    assert_eq!(torrent_file.announce, "https://torrent.ubuntu.com/announce");

    Ok(())
}

#[test]
fn test_deserialize_torrent_file_via_value() -> Result<(), Error> {
    let torrent_bytes = include_bytes!("ubuntu-18.04.3-live-server-amd64.iso.torrent");
    let decoded_value: Value = bt_bencode::from_slice(&torrent_bytes[..])?;

    let announce = match decoded_value {
        Value::Dict(dict) => match dict.get(&ByteBuf::from(String::from("announce"))) {
            Some(Value::ByteStr(s)) => Some(s.clone().into_vec()),
            _ => None,
        },
        _ => None,
    };

    assert_eq!(
        announce,
        Some(String::from("https://torrent.ubuntu.com/announce").into_bytes())
    );

    Ok(())
}
