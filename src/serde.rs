use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{ULID, ULID_STRING_LENGTH};

impl Serialize for ULID {
  fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
  where
    S: Serializer,
  {
    let mut buffer = [0; ULID_STRING_LENGTH];
    let text = self.to_str(&mut buffer).unwrap();
    text.serialize(serializer)
  }
}

impl<'de> Deserialize<'de> for ULID {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let deserialized_str = String::deserialize(deserializer)?;
    Self::from_string(&deserialized_str).map_err(serde::de::Error::custom)
  }
}

#[cfg(all(feature = "uuid", feature = "serde"))]
pub mod ulid_as_uuid {
  use serde::{Deserialize, Deserializer, Serialize, Serializer};

  use crate::ULID;

  /// Converts the ULID to a UUID and serializes it as a string.
  pub fn serialize<S>(value: &ULID, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let uuid: Uuid = (*value).into();
    uuid.to_string().serialize(serializer)
  }

  /// Deserializes a ULID from a string containing a UUID.
  pub fn deserialize<'de, D>(deserializer: D) -> Result<ULID, D::Error>
  where
    D: Deserializer<'de>,
  {
    let de_string = String::deserialize(deserializer)?;
    let de_uuid = Uuid::parse_str(&de_string).map_err(serde::de::Error::custom)?;
    Ok(Ulid::from(de_uuid))
  }
}
