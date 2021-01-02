# ulid-generator-rs

A Rust crate for generating ULIDs.

[![crates.io](https://img.shields.io/crates/v/ulid-generator-rs.svg)](https://crates.io/crates/ulid-generator-rs)
[![docs.rs](https://docs.rs/ulid-generator-rs/badge.svg)](https://docs.rs/ulid-generator-rs)
[![Workflow Status](https://github.com/j5ik2o/ulid-generator-rs/workflows/Rust/badge.svg)](https://github.com/j5ik2o/ulid-generator-rs/actions?query=workflow%3A%22Rust%22)

## Install to Cargo.toml

Add this to your `Cargo.toml`:

```toml
[dependencies]
ulid-generator-rs = "0.0.2"
```

## About ULID

ULID is Universally Unique Lexicographically Sortable Identifier.

For more information, please check the following specifications.
[ULID Spec](https://github.com/ulid/spec)

## Usage

```rust
use ulid_generator_rs::{ULIDGenerator, ULID};

let ulid: ULID = ULIDGenerator::new().generate().unwrap();
let str: String = ulid.to_string();
```

## Alternative crates

- https://github.com/dylanhart/ulid-rs
- https://github.com/huxi/rusty_ulid
- https://github.com/suyash/ulid-rs

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
