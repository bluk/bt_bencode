# CHANGELOG

## v0.6.0

### Added

* Allow serialization when no_std.

  Adds `Write` trait and implementations.

  Thanks [@bheesham](https://github.com/bheesham).

## v0.5.1

### Updated

* Use `Bytes` for `Values::Dict` index access instead of allocating a `ByteBuf`.

## v0.5.0

### Updated

* Update to `itoa` version `1.0.1`.

## v0.4.0

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

## v0.3.0

### Added

* `Read` trait and helper implementations `IoRead` and `SliceRead` are made public.
* Add `Value` `as_number()`.
* Add multiple `From` implementations for all the common primitive signed and
  unsigned integers to `Number`.

## v0.2.0

### Added

* `Value` type and related functions.

## v0.1.0

### Added

* `Serializer`, `Deserializer`, and related functions.
