// Copyright 2020 Developers of the `ulid-generator-rs` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ulid_generator_rs::ULIDGenerator;

fn main() {
  let ulid = ULIDGenerator::new().generate().unwrap();
  println!("{}", ulid);
}
