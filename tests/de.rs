#[macro_use]
extern crate serde_derive;

use bt_bencode::Error;

#[derive(Debug, Deserialize, Clone, Eq, Hash, PartialEq)]
struct TorrentFile {
    announce: String,
}

#[test]
fn test_deserialize_torrent_file() -> Result<(), Error> {
    let torrent_bytes = include_bytes!("ubuntu-18.04.3-live-server-amd64.iso.torrent");
    let torrent_file: TorrentFile = bt_bencode::from_slice(&torrent_bytes[..])?;

    assert_eq!(torrent_file.announce, "https://torrent.ubuntu.com/announce");

    Ok(())
}
