#[macro_use]
extern crate serde;

mod de;
mod error;
mod read;

pub use de::{from_reader, from_slice, Deserializer};
pub use error::{Error, Result};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
