use uuid::Uuid;
use crate::ULID;

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
