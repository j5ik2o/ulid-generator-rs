// Copyright 2020 Developers of the `ulid-generator-rs` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::ULID;
use uuid::Uuid;

impl From<Uuid> for ULID {
  fn from(uuid: Uuid) -> Self {
    Self(uuid.as_u128())
  }
}

impl From<ULID> for Uuid {
  fn from(ulid: ULID) -> Self {
    Uuid::from_u128(ulid.0)
  }
}

#[cfg(test)]
mod test {
  use crate::ULID;
  use uuid::Uuid;

  #[test]
  fn uuid() {
    let uuid_expected: Uuid = Uuid::new_v4();
    let ulid: ULID = uuid_expected.into();
    let uuid: Uuid = ulid.into();
    assert_eq!(uuid, uuid_expected);
  }
}
