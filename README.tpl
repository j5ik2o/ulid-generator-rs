# {{crate}}

A Rust crate for generating ULIDs.

{{badges}}
[![crates.io](https://img.shields.io/crates/v/ulid-generator-rs.svg)](https://crates.io/crates/ulid-generator-rs)
[![docs.rs](https://docs.rs/ulid-generator-rs/badge.svg)](https://docs.rs/ulid-generator-rs)
[![dependency status](https://deps.rs/repo/github/j5ik2o/ulid-generator-rs/status.svg)](https://deps.rs/repo/github/j5ik2o/ulid-generator-rs)
[![tokei](https://tokei.rs/b1/github/j5ik2o/ulid-generator-rs)](https://github.com/XAMPPRocky/tokei)

## Install to Cargo.toml

Add this to your `Cargo.toml`:

```toml
[dependencies]
{{crate}} = "<<version>>"
```

{{readme}}

## Benchmarks

```
gen_ulid_and_to_string/j5ik2o/ulid-generator-rs/gen_to_str/0
time:   [117.15 ns 117.26 ns 117.39 ns]
change: [-1.7662% -0.9620% -0.3349%] (p = 0.00 < 0.05)
Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
2 (2.00%) high mild
1 (1.00%) high severe

gen_ulid_and_to_string/dylanhart/ulid-rs/gen_to_str/0
time:   [115.63 ns 115.81 ns 116.04 ns]
change: [-1.0856% -0.8741% -0.6850%] (p = 0.00 < 0.05)
Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
2 (2.00%) high mild
2 (2.00%) high severe

gen_ulid_and_to_string/huxi/rusty_ulid/gen_to_str/0
time:   [126.32 ns 126.46 ns 126.60 ns]
change: [-0.4696% -0.3016% -0.1476%] (p = 0.00 < 0.05)
Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
2 (2.00%) high mild

gen_ulid_and_to_string/suyash/ulid-rs/gen_to_str/0
time:   [157.22 ns 157.35 ns 157.49 ns]
change: [-1.6453% -1.4630% -1.2639%] (p = 0.00 < 0.05)
Performance has improved.
Found 4 outliers among 100 measurements (4.00%)
3 (3.00%) high mild
1 (1.00%) high severe
```

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
