# BtBencode

BtBencode is a library which can help with [Bencode][wikipedia_bencode]
encoding/decoding.  Bencode is primarily used in [BitTorrent][bep_0003] related
applications.

It uses the [Serde][serde] library to serialize and deserialize Bencode data.

## Documentation

* [Latest API Docs][docs_rs_bt_bencode]

## Installation

```toml
[dependencies]
bt_bencode = "0.6.0"
```

## Examples

An example serializing from a standard Rust collection type into a custom type:

```rust
# use bt_bencode::Error;
# use std::collections::BTreeMap;
#
# fn main() -> Result<(), Error> {
use serde_bytes::ByteBuf;
use serde_derive::Deserialize;

let mut dict: BTreeMap<String, String> = BTreeMap::new();
dict.insert(String::from("url"), String::from("https://example.com/"));

let serialized_bytes = bt_bencode::to_vec(&dict)?;

#[derive(Deserialize)]
struct Info {
    url: String,
}

let info: Info = bt_bencode::from_slice(&serialized_bytes)?;
assert_eq!(info.url, "https://example.com/");
#   Ok(())
# }
```

An example deserializing from an unknown slice of bytes and then into a custom type.

```rust
# use bt_bencode::Error;
# use std::collections::BTreeMap;
#
# fn main() -> Result<(), Error> {
use bt_bencode::Value;
use serde_bytes::ByteBuf;
use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Info {
    t: String,
    url: String,
}

let serialized_bytes = bt_bencode::to_vec(&Info {
    t: String::from("query"),
    url: String::from("https://example.com/"),
})?;

let value: Value = bt_bencode::from_slice(&serialized_bytes)?;
assert_eq!(value["t"].as_str().ok_or(Error::UnsupportedType)?, "query");

let info: Info = bt_bencode::from_value(value)?;
assert_eq!(info.url, "https://example.com/");
#   Ok(())
# }
```

## License

Licensed under either of [Apache License, Version 2.0][LICENSE_APACHE] or [MIT
License][LICENSE_MIT] at your option.

### Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[LICENSE_APACHE]: LICENSE-APACHE
[LICENSE_MIT]: LICENSE-MIT
[wikipedia_bencode]: https://en.wikipedia.org/wiki/Bencode
[bep_0003]: http://www.bittorrent.org/beps/bep_0003.html
[serde]: https://serde.rs
[docs_rs_bt_bencode]: https://docs.rs/bt_bencode/latest/bt_bencode/
