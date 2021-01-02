// Copyright 2020 Developers of the `ulid-generator-rs` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

use crate::ULID;

impl Serialize for ULID {
  fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
  where
    S: Serializer, {
    let text = self.to_string();
    text.serialize(serializer)
  }
}

impl<'de> Deserialize<'de> for ULID {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>, {
    let deserialized_str = String::deserialize(deserializer)?;
    deserialized_str.parse::<ULID>().map_err(serde::de::Error::custom)
  }
}

pub mod ulid_as_u128 {
  use super::*;

  /// Serializes a ULID as a u128 type.
  pub fn serialize<S>(value: &ULID, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer, {
    value.0.serialize(serializer)
  }

  /// Deserializes a ULID from a u128 type.
  pub fn deserialize<'de, D>(deserializer: D) -> Result<ULID, D::Error>
  where
    D: Deserializer<'de>, {
    let deserialized_u128 = u128::deserialize(deserializer)?;
    Ok(ULID::from(deserialized_u128))
  }
}

#[cfg(all(feature = "uuid", feature = "serde"))]
pub mod ulid_as_uuid {
  use super::*;

  /// Converts the ULID to a UUID and serializes it as a string.
  pub fn serialize<S>(value: &ULID, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer, {
    let uuid: Uuid = value.clone().into();
    uuid.to_string().serialize(serializer)
  }

  /// Deserializes a ULID from a string containing a UUID.
  pub fn deserialize<'de, D>(deserializer: D) -> Result<ULID, D::Error>
  where
    D: Deserializer<'de>, {
    let de_string = String::deserialize(deserializer)?;
    let de_uuid = Uuid::parse_str(&de_string).map_err(serde::de::Error::custom)?;
    Ok(ULID::from(de_uuid))
  }
}
