use std::cmp::Ordering;
use std::convert::TryFrom;
use std::str::FromStr;

use chrono::{DateTime, Duration, TimeZone, Utc};
use rand::Rng;
use rand::rngs::ThreadRng;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ULIDError {
  #[error("generate random error: msg = {msg}")]
  GenerateRandomError { msg: String },
  #[error("invalid length")]
  InvalidLength,
  #[error("invalid the char: {0}")]
  InvalidChar(char),
  #[error("data type overflow")]
  DataTypeOverflow,
  #[error("data must be 16 bytes in length!")]
  InvalidByteArrayError,
  #[error("ulidString must not exceed '7ZZZZZZZZZZZZZZZZZZZZZZZZZ'!")]
  TimestampOverflowError,
}

const ULID_STRING_LENGTH: u32 = 26;
const ULID_BYTES_LENGTH: u32 = 16;
const TIMESTAMP_OVERFLOW_MASK: u64 = 0xffff000000000000;

#[rustfmt::skip]
static ENCODING_DIGITS: [char; 32] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K',
    'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'V', 'W', 'X',
    'Y', 'Z',
  ];

#[rustfmt::skip]
static DECODING_DIGITS: [Option<u8>; 123] = [
  // 0
  None, None, None, None, None, None, None, None,
  // 8
  None, None, None, None, None, None, None, None,
  // 16
  None, None, None, None, None, None, None, None,
  // 24
  None, None, None, None, None, None, None, None,
  // 32
  None, None, None, None, None, None, None, None,
  // 40
  None, None, None, None, None, None, None, None,
  // 48
  Some(0), Some(1), Some(2), Some(3), Some(4), Some(5), Some(6), Some(7),
  // 56
  Some(8), Some(9), None, None, None, None, None, None,
  // 64
  None, Some(10), Some(11), Some(12), Some(13), Some(14), Some(15), Some(16),
  // 72
  Some(17), Some(1), Some(18), Some(19), Some(1), Some(20), Some(21), Some(0),
  // 80
  Some(22), Some(23), Some(24), Some(25), Some(26), None, Some(27), Some(28),
  // 88
  Some(29), Some(30), Some(31), None, None, None, None, None,
  // 96
  None, Some(10), Some(11), Some(12), Some(13), Some(14), Some(15), Some(16),
  // 104
  Some(17), Some(1), Some(18), Some(19), Some(1), Some(20), Some(21), Some(0),
  // 112
  Some(22), Some(23), Some(24), Some(25), Some(26), None, Some(27), Some(28),
  // 120
  Some(29), Some(30), Some(31),
];

fn resolve_u64_value_for_char(c: char) -> Result<u64, ULIDError> {
  let index = c as usize;
  if index < DECODING_DIGITS.len() {
    if let Some(u8_value) = DECODING_DIGITS[index] {
      return Ok(u64::from(u8_value));
    }
  }
  Err(ULIDError::InvalidChar(c))
}

fn resolve_u128_value_for_char(c: char) -> Result<u128, ULIDError> {
  let index = c as usize;
  if index < DECODING_DIGITS.len() {
    if let Some(u8_value) = DECODING_DIGITS[index] {
      return Ok(u128::from(u8_value));
    }
  }
  Err(ULIDError::InvalidChar(c))
}

pub fn parse_crockford_u64_tuple(input: &str) -> Result<(u64, u64), ULIDError> {
  let length = input.len();
  if length != ULID_STRING_LENGTH as usize {
    return Err(ULIDError::InvalidLength);
  }

  let mut chars = input.chars();
  let highest = resolve_u64_value_for_char(chars.next().unwrap())?;
  if highest > 7 {
    return Err(ULIDError::DataTypeOverflow);
  }

  let mut high: u64 = highest << 61;
  high |= resolve_u64_value_for_char(chars.next().unwrap())? << 56;
  high |= resolve_u64_value_for_char(chars.next().unwrap())? << 51;
  high |= resolve_u64_value_for_char(chars.next().unwrap())? << 46;
  high |= resolve_u64_value_for_char(chars.next().unwrap())? << 41;
  high |= resolve_u64_value_for_char(chars.next().unwrap())? << 36;
  high |= resolve_u64_value_for_char(chars.next().unwrap())? << 31;
  high |= resolve_u64_value_for_char(chars.next().unwrap())? << 26;
  high |= resolve_u64_value_for_char(chars.next().unwrap())? << 21;
  high |= resolve_u64_value_for_char(chars.next().unwrap())? << 16;
  high |= resolve_u64_value_for_char(chars.next().unwrap())? << 11;
  high |= resolve_u64_value_for_char(chars.next().unwrap())? << 6;
  high |= resolve_u64_value_for_char(chars.next().unwrap())? << 1;

  let split = resolve_u64_value_for_char(chars.next().unwrap())?;
  high |= split >> 4;

  let mut low: u64 = split << 60;

  low |= resolve_u64_value_for_char(chars.next().unwrap())? << 55;
  low |= resolve_u64_value_for_char(chars.next().unwrap())? << 50;
  low |= resolve_u64_value_for_char(chars.next().unwrap())? << 45;
  low |= resolve_u64_value_for_char(chars.next().unwrap())? << 40;
  low |= resolve_u64_value_for_char(chars.next().unwrap())? << 35;
  low |= resolve_u64_value_for_char(chars.next().unwrap())? << 30;
  low |= resolve_u64_value_for_char(chars.next().unwrap())? << 25;
  low |= resolve_u64_value_for_char(chars.next().unwrap())? << 20;
  low |= resolve_u64_value_for_char(chars.next().unwrap())? << 15;
  low |= resolve_u64_value_for_char(chars.next().unwrap())? << 10;
  low |= resolve_u64_value_for_char(chars.next().unwrap())? << 5;
  low |= resolve_u64_value_for_char(chars.next().unwrap())?;

  Ok((high, low))
}

pub fn parse_crockford_u128(input: &str) -> Result<u128, ULIDError> {
  let length = input.len();
  if length != 26 {
    return Err(ULIDError::InvalidLength);
  }

  let mut chars = input.chars();

  let highest = resolve_u128_value_for_char(chars.next().unwrap())?;
  if highest > 7 {
    return Err(ULIDError::DataTypeOverflow);
  }

  let mut result: u128 = highest << 125;
  result |= resolve_u128_value_for_char(chars.next().unwrap())? << 120;
  result |= resolve_u128_value_for_char(chars.next().unwrap())? << 115;
  result |= resolve_u128_value_for_char(chars.next().unwrap())? << 110;
  result |= resolve_u128_value_for_char(chars.next().unwrap())? << 105;
  result |= resolve_u128_value_for_char(chars.next().unwrap())? << 100;
  result |= resolve_u128_value_for_char(chars.next().unwrap())? << 95;
  result |= resolve_u128_value_for_char(chars.next().unwrap())? << 90;
  result |= resolve_u128_value_for_char(chars.next().unwrap())? << 85;
  result |= resolve_u128_value_for_char(chars.next().unwrap())? << 80;
  result |= resolve_u128_value_for_char(chars.next().unwrap())? << 75;
  result |= resolve_u128_value_for_char(chars.next().unwrap())? << 70;
  result |= resolve_u128_value_for_char(chars.next().unwrap())? << 65;
  result |= resolve_u128_value_for_char(chars.next().unwrap())? << 60;
  result |= resolve_u128_value_for_char(chars.next().unwrap())? << 55;
  result |= resolve_u128_value_for_char(chars.next().unwrap())? << 50;
  result |= resolve_u128_value_for_char(chars.next().unwrap())? << 45;
  result |= resolve_u128_value_for_char(chars.next().unwrap())? << 40;
  result |= resolve_u128_value_for_char(chars.next().unwrap())? << 35;
  result |= resolve_u128_value_for_char(chars.next().unwrap())? << 30;
  result |= resolve_u128_value_for_char(chars.next().unwrap())? << 25;
  result |= resolve_u128_value_for_char(chars.next().unwrap())? << 20;
  result |= resolve_u128_value_for_char(chars.next().unwrap())? << 15;
  result |= resolve_u128_value_for_char(chars.next().unwrap())? << 10;
  result |= resolve_u128_value_for_char(chars.next().unwrap())? << 5;
  result |= resolve_u128_value_for_char(chars.next().unwrap())?;

  Ok(result)
}

const MASK_U64: u64 = 0b11111;
const MASK_U128: u128 = 0b11111;

pub fn append_crockford_u64_tuple(value: (u64, u64), to_append_to: &mut String) {
  to_append_to.push(ENCODING_DIGITS[(value.0 >> 61) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value.0 >> 56) & MASK_U64) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value.0 >> 51) & MASK_U64) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value.0 >> 46) & MASK_U64) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value.0 >> 41) & MASK_U64) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value.0 >> 36) & MASK_U64) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value.0 >> 31) & MASK_U64) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value.0 >> 26) & MASK_U64) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value.0 >> 21) & MASK_U64) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value.0 >> 16) & MASK_U64) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value.0 >> 11) & MASK_U64) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value.0 >> 6) & MASK_U64) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value.0 >> 1) & MASK_U64) as usize]);

  let split = ((value.0 << 4) & MASK_U64) | ((value.1 >> 60) & MASK_U64);
  to_append_to.push(ENCODING_DIGITS[split as usize]);

  to_append_to.push(ENCODING_DIGITS[((value.1 >> 55) & MASK_U64) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value.1 >> 50) & MASK_U64) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value.1 >> 45) & MASK_U64) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value.1 >> 40) & MASK_U64) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value.1 >> 35) & MASK_U64) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value.1 >> 30) & MASK_U64) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value.1 >> 25) & MASK_U64) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value.1 >> 20) & MASK_U64) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value.1 >> 15) & MASK_U64) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value.1 >> 10) & MASK_U64) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value.1 >> 5) & MASK_U64) as usize]);
  to_append_to.push(ENCODING_DIGITS[(value.1 & MASK_U64) as usize]);
}

pub fn append_crockford_u128(value: u128, to_append_to: &mut String) {
  to_append_to.push(ENCODING_DIGITS[(value >> 125) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value >> 120) & MASK_U128) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value >> 115) & MASK_U128) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value >> 110) & MASK_U128) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value >> 105) & MASK_U128) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value >> 100) & MASK_U128) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value >> 95) & MASK_U128) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value >> 90) & MASK_U128) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value >> 85) & MASK_U128) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value >> 80) & MASK_U128) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value >> 75) & MASK_U128) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value >> 70) & MASK_U128) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value >> 65) & MASK_U128) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value >> 60) & MASK_U128) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value >> 55) & MASK_U128) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value >> 50) & MASK_U128) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value >> 45) & MASK_U128) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value >> 40) & MASK_U128) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value >> 35) & MASK_U128) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value >> 30) & MASK_U128) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value >> 25) & MASK_U128) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value >> 20) & MASK_U128) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value >> 15) & MASK_U128) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value >> 10) & MASK_U128) as usize]);
  to_append_to.push(ENCODING_DIGITS[((value >> 5) & MASK_U128) as usize]);
  to_append_to.push(ENCODING_DIGITS[(value & MASK_U128) as usize]);
}

#[derive(Debug, Clone, PartialEq)]
pub struct ULID {
  most_significant_bits: u64,
  least_significant_bits: u64,
}

impl PartialOrd for ULID {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    if self.most_significant_bits < other.most_significant_bits {
      Some(Ordering::Less)
    } else if self.most_significant_bits > other.most_significant_bits {
      Some(Ordering::Greater)
    } else if self.least_significant_bits < other.least_significant_bits {
      Some(Ordering::Less)
    } else if self.least_significant_bits > other.least_significant_bits {
      Some(Ordering::Greater)
    } else {
      Some(Ordering::Equal)
    }
  }
}

impl ToString for ULID {
  fn to_string(&self) -> String {
    let mut result = String::with_capacity(ULID_STRING_LENGTH as usize);
    append_crockford_u64_tuple(
      (self.most_significant_bits, self.least_significant_bits),
      &mut result,
    );
    result
  }
}

impl ULID {
  #[inline]
  pub fn new(most_significant_bits: u64, least_significant_bits: u64) -> Self {
    Self {
      most_significant_bits,
      least_significant_bits,
    }
  }

  pub fn to_epoch_milli_as_long(&self) -> u64 {
    self.most_significant_bits >> 16
  }

  pub fn to_epoch_milli_as_duration(&self) -> Duration {
    Duration::milliseconds(self.to_epoch_milli_as_long() as i64)
  }

  pub fn to_date_time(&self) -> DateTime<Utc> {
    Utc.timestamp_millis(self.to_epoch_milli_as_long() as i64)
  }

  pub fn to_bytes(&self) -> ByteArray {
    let mut result: ByteArray = Vec::with_capacity(16);
    result.resize(16, 0);
    for i in 0..8 {
      result[i] = ((self.most_significant_bits >> ((7 - i) * 8)) & 0xff) as u8;
    }
    for i in 8..16 {
      result[i] = ((self.least_significant_bits >> ((15 - i) * 8)) & 0xff) as u8;
    }
    result
  }
}

pub struct ULIDGenerator {
  rng: ThreadRng,
}

type ByteArray = Vec<u8>;

impl FromStr for ULID {
  type Err = ULIDError;

  fn from_str(ulid_str: &str) -> Result<Self, Self::Err> {
    let (m, l) = parse_crockford_u64_tuple(ulid_str)?;
    Ok(ULID::new(m, l))
  }
}

impl TryFrom<ByteArray> for ULID {
  type Error = ULIDError;

  fn try_from(value: ByteArray) -> Result<Self, Self::Error> {
    if value.len() != ULID_BYTES_LENGTH as usize {
      Err(ULIDError::InvalidByteArrayError)
    } else {
      let mut most_significant_bits: u64 = 0u64;
      for i in 0..8 {
        most_significant_bits = (most_significant_bits << 8) | (value[i] & 0xff) as u64;
      }
      let mut least_significant_bits: u64 = 0u64;
      for i in 8..16 {
        least_significant_bits = (least_significant_bits << 8) | (value[i] & 0xff) as u64;
      }
      Ok(ULID::new(most_significant_bits, least_significant_bits))
    }
  }
}

impl ULIDGenerator {
  #[inline]
  pub fn new() -> Self {
    Self {
      rng: rand::thread_rng(),
    }
  }

  pub fn generate(&mut self) -> Result<ULID, ULIDError> {
    let timestamp = Utc::now().timestamp_millis() as u64;
    if (timestamp & TIMESTAMP_OVERFLOW_MASK) != 0 {
      Err(ULIDError::TimestampOverflowError)
    } else {
      let (most_rnd, least_rnd): (u16, u64) = self.rng.gen();
      let most_significant_bits = timestamp << 16 | u64::from(most_rnd);
      let least_significant_bits = least_rnd;
      Ok(ULID::new(most_significant_bits, least_significant_bits))
    }
  }
}

#[cfg(test)]
mod tests {
  use std::convert::TryFrom;

  use crate::{ULID, ULIDError, ULIDGenerator};

  #[test]
  fn new() {
    let ulid = ULID::new(105449255778666307, 1874305465861347464);
    assert_eq!(ulid.to_string(), "01ETGRM6448X1HM0PYWG2KT648");
    let ulid = ULID::new(105449255778666307, 1874305465861347465);
    assert_eq!(ulid.to_string(), "01ETGRM6448X1HM0PYWG2KT649");
  }

  #[test]
  fn parse() -> Result<(), ULIDError> {
    let ulid = "01ETGRM6448X1HM0PYWG2KT648".parse::<ULID>()?;
    assert_eq!(ulid.to_string(), "01ETGRM6448X1HM0PYWG2KT648");
    Ok(())
  }
}
