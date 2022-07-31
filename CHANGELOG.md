# CHANGELOG

## [0.7.0] - 2022-07-31

### Added

* Add `Deserializer::byte_offset()` to return the byte offset in the underlying source. It may be useful if there is trailing data.
* Serialize and deserialize tuples and tuple structs.
* Allow deserialization from a borrowed `Value`.
* Set supported Rust version to `1.36.0`. The MSRV is not guaranteed due to dependencies being free to bump their version.

### Updated

* In general, fewer allocations are made when parsing values.
* **Breaking change**: Refactored the `Read` trait to allow borrowing against the original data.

  ```
  #[derive(Deserialize)]
  struct Info<'a> {
      name: Option<&'a str>,
      pieces: &'a [u8],
  }
  ```

  should work now when using `from_slice`.
* **Breaking change**: Refactored the `Error` type.

  The `Error::byte_offset()` method can help hint where the error occurred at (usually only helpful for deserialization).

  Refactored to use `Box` to reduce the size of the return types. Rationale is
  influenced by Serde JSON issues/discussions where an allocation for an exceptional code path is acceptable.

## [0.6.1] - 2022-03-31

### Updated

* Fix wrong error returned when parsing an invalid list.
* Add documentation to more items
* Add #[must_use] to more functions

## [0.6.0] - 2022-03-21

### Added

* Allow serialization when no_std.

  Adds `Write` trait and implementations.

  Thanks [@bheesham](https://github.com/bheesham).

## [0.5.1] - 2022-03-14

### Updated

* Use `Bytes` for `Values::Dict` index access instead of allocating a `ByteBuf`.

## [0.5.0] - 2022-03-09

### Updated

* Update to `itoa` version `1.0.1`.

## [0.4.0] - 2021-05-27

### Added

* Allow deserialization of non-byte string values into raw byte buffers. In
  cases where a value is a non-byte string, a byte buffer can be used to capture
  the raw encoded value. For instance, assuming a dictionary with an `info`
  key which has a dictionary value:

  ```
  #[derive(Deserialize)]
  struct Metainfo {
      info: ByteBuf,
  }
  ```

  could be used to capture the raw bytes of the encoded `info` dictionary value.

  For untrusted input, the value should be verified as having the correct type
  (e.g. a dictionary) instead of a byte string which contains the raw encoded
  value.

## [0.3.0] - 2020-10-10

### Added

* `Read` trait and helper implementations `IoRead` and `SliceRead` are made public.
* Add `Value` `as_number()`.
* Add multiple `From` implementations for all the common primitive signed and
  unsigned integers to `Number`.

## [0.2.0] - 2020-02-20

### Added

* `Value` type and related functions.

## [0.1.0] - 2020-02-20

### Added

* `Serializer`, `Deserializer`, and related functions.


[Unreleased]: https://github.com/bluk/bt_bencode/compare/v0.7.0...HEAD
[0.7.0]: https://github.com/bluk/bt_bencode/compare/v0.6.1...v0.7.0
[0.6.1]: https://github.com/bluk/bt_bencode/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/bluk/bt_bencode/compare/v0.5.1...v0.6.0
[0.5.1]: https://github.com/bluk/bt_bencode/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/bluk/bt_bencode/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/bluk/bt_bencode/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/bluk/bt_bencode/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/bluk/bt_bencode/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/bluk/bt_bencode/releases/tag/v0.1.0