// Copyright 2020 Developers of the `ulid-generator-rs` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # About ULID
//!
//! ULID is Universally Unique Lexicographically Sortable Identifier.
//!
//! For more information, please check the following specifications.
//! - [ULID Spec](https://github.com/ulid/spec)
//!
//! # Usage
//!
//! ```rust
//! use ulid_generator_rs::{ULIDGenerator, ULID};
//!
//! let mut generator: ULIDGenerator = ULIDGenerator::new();
//! let ulid: ULID = generator.generate().unwrap();
//! let str: String = ulid.to_string();
//! println!("{}", str); // "01ETGRM6448X1HM0PYWG2KT648"
//! ```
//!
//! # Alternative crates
//!
//! - https://github.com/dylanhart/ulid-rs
//! - https://github.com/huxi/rusty_ulid
//! - https://github.com/suyash/ulid-rs
#![allow(dead_code)]

use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Local, TimeZone, Utc};
use rand::rngs::ThreadRng;
use rand::Rng;
use std::cmp::Ordering;
use thiserror::Error;

#[cfg(feature = "serde")]
pub mod serde;
#[cfg(feature = "uuid")]
pub mod uuid;

type ByteArray = Vec<u8>;

/// The error types of [ULID]
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

const ENCODING_DIGITS: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

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

#[inline]
fn resolve_value_for_char<T>(c: char) -> Result<T, ULIDError>
where
  T: From<u8>, {
  let index = c as usize;
  if index < DECODING_DIGITS.len() {
    if let Some(u8_value) = DECODING_DIGITS[index] {
      return Ok(T::from(u8_value));
    }
  }
  Err(ULIDError::InvalidChar(c))
}

#[inline]
fn parse_crockford_u64_tuple(input: &str) -> Result<(u64, u64), ULIDError> {
  let length = input.len();
  if length != ULID_STRING_LENGTH as usize {
    return Err(ULIDError::InvalidLength);
  }
  let mut chars = input.chars();
  let highest: u64 = resolve_value_for_char::<u64>(chars.next().unwrap())?;
  if highest > 7 {
    return Err(ULIDError::DataTypeOverflow);
  }

  let mut high: u64 = highest << 61;
  high |= resolve_value_for_char::<u64>(chars.next().unwrap())? << 56;
  high |= resolve_value_for_char::<u64>(chars.next().unwrap())? << 51;
  high |= resolve_value_for_char::<u64>(chars.next().unwrap())? << 46;
  high |= resolve_value_for_char::<u64>(chars.next().unwrap())? << 41;
  high |= resolve_value_for_char::<u64>(chars.next().unwrap())? << 36;
  high |= resolve_value_for_char::<u64>(chars.next().unwrap())? << 31;
  high |= resolve_value_for_char::<u64>(chars.next().unwrap())? << 26;
  high |= resolve_value_for_char::<u64>(chars.next().unwrap())? << 21;
  high |= resolve_value_for_char::<u64>(chars.next().unwrap())? << 16;
  high |= resolve_value_for_char::<u64>(chars.next().unwrap())? << 11;
  high |= resolve_value_for_char::<u64>(chars.next().unwrap())? << 6;
  high |= resolve_value_for_char::<u64>(chars.next().unwrap())? << 1;

  let split: u64 = resolve_value_for_char::<u64>(chars.next().unwrap())?;
  high |= split >> 4;

  let mut low: u64 = split << 60;
  low |= resolve_value_for_char::<u64>(chars.next().unwrap())? << 55;
  low |= resolve_value_for_char::<u64>(chars.next().unwrap())? << 50;
  low |= resolve_value_for_char::<u64>(chars.next().unwrap())? << 45;
  low |= resolve_value_for_char::<u64>(chars.next().unwrap())? << 40;
  low |= resolve_value_for_char::<u64>(chars.next().unwrap())? << 35;
  low |= resolve_value_for_char::<u64>(chars.next().unwrap())? << 30;
  low |= resolve_value_for_char::<u64>(chars.next().unwrap())? << 25;
  low |= resolve_value_for_char::<u64>(chars.next().unwrap())? << 20;
  low |= resolve_value_for_char::<u64>(chars.next().unwrap())? << 15;
  low |= resolve_value_for_char::<u64>(chars.next().unwrap())? << 10;
  low |= resolve_value_for_char::<u64>(chars.next().unwrap())? << 5;
  low |= resolve_value_for_char::<u64>(chars.next().unwrap())?;

  Ok((high, low))
}

#[inline]
fn parse_crockford_u128(input: &str) -> Result<u128, ULIDError> {
  let length = input.len();
  if length != 26 {
    return Err(ULIDError::InvalidLength);
  }
  let mut chars = input.chars();
  let highest: u128 = resolve_value_for_char::<u128>(chars.next().unwrap())?;
  if highest > 7 {
    return Err(ULIDError::DataTypeOverflow);
  }
  let mut result: u128 = highest << 125;
  result |= resolve_value_for_char::<u128>(chars.next().unwrap())? << 120;
  result |= resolve_value_for_char::<u128>(chars.next().unwrap())? << 115;
  result |= resolve_value_for_char::<u128>(chars.next().unwrap())? << 110;
  result |= resolve_value_for_char::<u128>(chars.next().unwrap())? << 105;
  result |= resolve_value_for_char::<u128>(chars.next().unwrap())? << 100;
  result |= resolve_value_for_char::<u128>(chars.next().unwrap())? << 95;
  result |= resolve_value_for_char::<u128>(chars.next().unwrap())? << 90;
  result |= resolve_value_for_char::<u128>(chars.next().unwrap())? << 85;
  result |= resolve_value_for_char::<u128>(chars.next().unwrap())? << 80;
  result |= resolve_value_for_char::<u128>(chars.next().unwrap())? << 75;
  result |= resolve_value_for_char::<u128>(chars.next().unwrap())? << 70;
  result |= resolve_value_for_char::<u128>(chars.next().unwrap())? << 65;
  result |= resolve_value_for_char::<u128>(chars.next().unwrap())? << 60;
  result |= resolve_value_for_char::<u128>(chars.next().unwrap())? << 55;
  result |= resolve_value_for_char::<u128>(chars.next().unwrap())? << 50;
  result |= resolve_value_for_char::<u128>(chars.next().unwrap())? << 45;
  result |= resolve_value_for_char::<u128>(chars.next().unwrap())? << 40;
  result |= resolve_value_for_char::<u128>(chars.next().unwrap())? << 35;
  result |= resolve_value_for_char::<u128>(chars.next().unwrap())? << 30;
  result |= resolve_value_for_char::<u128>(chars.next().unwrap())? << 25;
  result |= resolve_value_for_char::<u128>(chars.next().unwrap())? << 20;
  result |= resolve_value_for_char::<u128>(chars.next().unwrap())? << 15;
  result |= resolve_value_for_char::<u128>(chars.next().unwrap())? << 10;
  result |= resolve_value_for_char::<u128>(chars.next().unwrap())? << 5;
  result |= resolve_value_for_char::<u128>(chars.next().unwrap())?;
  Ok(result)
}

const MASK_U64: u64 = 0b11111;
const MASK_U128: u128 = 0b11111;

#[inline]
const fn append_crockford_u64_tuple(value: (u64, u64)) -> [u8; 26] {
  let mut ans = [0; 26];
  ans[0] = ENCODING_DIGITS[(value.0 >> 61) as usize];
  ans[1] = ENCODING_DIGITS[((value.0 >> 56) & MASK_U64) as usize];
  ans[2] = ENCODING_DIGITS[((value.0 >> 51) & MASK_U64) as usize];
  ans[3] = ENCODING_DIGITS[((value.0 >> 46) & MASK_U64) as usize];
  ans[4] = ENCODING_DIGITS[((value.0 >> 41) & MASK_U64) as usize];
  ans[5] = ENCODING_DIGITS[((value.0 >> 36) & MASK_U64) as usize];
  ans[6] = ENCODING_DIGITS[((value.0 >> 31) & MASK_U64) as usize];
  ans[7] = ENCODING_DIGITS[((value.0 >> 26) & MASK_U64) as usize];
  ans[8] = ENCODING_DIGITS[((value.0 >> 21) & MASK_U64) as usize];
  ans[9] = ENCODING_DIGITS[((value.0 >> 16) & MASK_U64) as usize];
  ans[10] = ENCODING_DIGITS[((value.0 >> 11) & MASK_U64) as usize];
  ans[11] = ENCODING_DIGITS[((value.0 >> 6) & MASK_U64) as usize];
  ans[12] = ENCODING_DIGITS[((value.0 >> 1) & MASK_U64) as usize];

  let split = ((value.0 << 4) & MASK_U64) | ((value.1 >> 60) & MASK_U64);
  ans[13] = ENCODING_DIGITS[split as usize];

  ans[14] = ENCODING_DIGITS[((value.1 >> 55) & MASK_U64) as usize];
  ans[15] = ENCODING_DIGITS[((value.1 >> 50) & MASK_U64) as usize];
  ans[16] = ENCODING_DIGITS[((value.1 >> 45) & MASK_U64) as usize];
  ans[17] = ENCODING_DIGITS[((value.1 >> 40) & MASK_U64) as usize];
  ans[18] = ENCODING_DIGITS[((value.1 >> 35) & MASK_U64) as usize];
  ans[19] = ENCODING_DIGITS[((value.1 >> 30) & MASK_U64) as usize];
  ans[20] = ENCODING_DIGITS[((value.1 >> 25) & MASK_U64) as usize];
  ans[21] = ENCODING_DIGITS[((value.1 >> 20) & MASK_U64) as usize];
  ans[22] = ENCODING_DIGITS[((value.1 >> 15) & MASK_U64) as usize];
  ans[23] = ENCODING_DIGITS[((value.1 >> 10) & MASK_U64) as usize];
  ans[24] = ENCODING_DIGITS[((value.1 >> 5) & MASK_U64) as usize];
  ans[25] = ENCODING_DIGITS[(value.1 & MASK_U64) as usize];
  ans
}

#[inline]
const fn append_crockford_u128(value: u128) -> [u8; 26] {
  let mut ans = [0; 26];
  ans[0] = ENCODING_DIGITS[(value >> 125) as usize];
  ans[1] = ENCODING_DIGITS[((value >> 120) & MASK_U128) as usize];
  ans[2] = ENCODING_DIGITS[((value >> 115) & MASK_U128) as usize];
  ans[3] = ENCODING_DIGITS[((value >> 110) & MASK_U128) as usize];
  ans[4] = ENCODING_DIGITS[((value >> 105) & MASK_U128) as usize];
  ans[5] = ENCODING_DIGITS[((value >> 100) & MASK_U128) as usize];
  ans[6] = ENCODING_DIGITS[((value >> 95) & MASK_U128) as usize];
  ans[7] = ENCODING_DIGITS[((value >> 90) & MASK_U128) as usize];
  ans[8] = ENCODING_DIGITS[((value >> 85) & MASK_U128) as usize];
  ans[9] = ENCODING_DIGITS[((value >> 80) & MASK_U128) as usize];
  ans[10] = ENCODING_DIGITS[((value >> 75) & MASK_U128) as usize];
  ans[11] = ENCODING_DIGITS[((value >> 70) & MASK_U128) as usize];
  ans[12] = ENCODING_DIGITS[((value >> 65) & MASK_U128) as usize];
  ans[13] = ENCODING_DIGITS[((value >> 60) & MASK_U128) as usize];
  ans[14] = ENCODING_DIGITS[((value >> 55) & MASK_U128) as usize];
  ans[15] = ENCODING_DIGITS[((value >> 50) & MASK_U128) as usize];
  ans[16] = ENCODING_DIGITS[((value >> 45) & MASK_U128) as usize];
  ans[17] = ENCODING_DIGITS[((value >> 40) & MASK_U128) as usize];
  ans[18] = ENCODING_DIGITS[((value >> 35) & MASK_U128) as usize];
  ans[19] = ENCODING_DIGITS[((value >> 30) & MASK_U128) as usize];
  ans[20] = ENCODING_DIGITS[((value >> 25) & MASK_U128) as usize];
  ans[21] = ENCODING_DIGITS[((value >> 20) & MASK_U128) as usize];
  ans[22] = ENCODING_DIGITS[((value >> 15) & MASK_U128) as usize];
  ans[23] = ENCODING_DIGITS[((value >> 10) & MASK_U128) as usize];
  ans[24] = ENCODING_DIGITS[((value >> 5) & MASK_U128) as usize];
  ans[25] = ENCODING_DIGITS[(value & MASK_U128) as usize];
  ans
}

/// This enum is the endian types.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Endian {
  /// Little endian.
  LE,
  /// Big endian.
  BE,
}

/// This struct is [ULID].
#[derive(Debug, Copy, Clone, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct ULID(u128);

unsafe impl Send for ULID {}
unsafe impl Sync for ULID {}

impl fmt::Display for ULID {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(&self.to_string())
  }
}

const RANDOM_MSB_MASK: u64 = 0xffff;
const TIMESTAMP_MSB_MASK: u64 = 0xffffffffffff0000;

/// implements for [ULID].
impl ULID {
  /// The Constructor for [ULID].
  ///
  /// # Example
  ///
  /// [`ULID::new()`] is used to create a [ULID] instance, but usually [ULIDGenerator] is used.
  ///
  /// ```rust
  /// use ulid_generator_rs::{ULID, ULIDGenerator};
  ///
  /// let ulid: ULID = ULID::new(1945530789360716160560926739305506752);
  ///
  /// // Use `ULIDGenerator::generate` as follows
  /// let mut generator: ULIDGenerator = ULIDGenerator::new();
  /// let ulid: ULID = generator.generate().unwrap();
  /// println!("{}", ulid); // "01ETGRM6448X1HM0PYWG2KT648"
  /// ```
  #[must_use]
  pub fn new(value: u128) -> Self {
    Self(value)
  }

  /// Converts a [ULID] to a string representation.
  ///
  /// # Example
  ///
  /// ```rust
  /// use ulid_generator_rs::{ULIDGenerator, ULID};
  ///
  /// let mut generator: ULIDGenerator = ULIDGenerator::new();
  /// let ulid: ULID = generator.generate().unwrap();
  /// let str: String = ulid.to_string();
  /// println!("{}", str); // "01ETGRM6448X1HM0PYWG2KT648"
  /// ```
  #[allow(clippy::unknown_clippy_lints)]
  #[allow(clippy::inherent_to_string_shadow_display)]
  #[must_use]
  pub fn to_string(&self) -> String {
    String::from_utf8(append_crockford_u128(self.0).to_vec()).unwrap()
  }

  /// Increment this [ULID].
  ///
  /// # Example
  ///
  /// ```rust
  /// use ulid_generator_rs::{ULIDGenerator, ULID};
  ///
  /// let mut generator: ULIDGenerator = ULIDGenerator::new();
  /// let ulid: ULID = generator.generate().unwrap();
  /// let next_ulid: ULID = ulid.increment();
  /// ```
  pub fn increment(&self) -> Self {
    let lsb = self.least_significant_bits();
    if lsb != 0xffffffffffffffff {
      ULID::from((self.most_significant_bits(), lsb + 1))
    } else {
      let msb = self.most_significant_bits();
      if (msb & RANDOM_MSB_MASK) != RANDOM_MSB_MASK {
        ULID::from((msb + 1, 0))
      } else {
        ULID::from((msb & TIMESTAMP_MSB_MASK, 0))
      }
    }
  }

  /// Most significant bits.
  ///
  /// # Example
  ///
  /// ```rust
  /// use ulid_generator_rs::{ULIDGenerator, ULID};
  ///
  /// let mut generator: ULIDGenerator = ULIDGenerator::new();
  /// let ulid: ULID = generator.generate().unwrap();
  /// let m: u64 = ulid.most_significant_bits();
  /// ```
  #[must_use]
  pub const fn most_significant_bits(&self) -> u64 {
    (self.0 >> 64) as u64
  }

  /// Least significant bits.
  ///
  /// # Example
  ///
  /// ```rust
  /// use ulid_generator_rs::{ULIDGenerator, ULID};
  ///
  /// let mut generator: ULIDGenerator = ULIDGenerator::new();
  /// let ulid: ULID = generator.generate().unwrap();
  /// let l: u64 = ulid.least_significant_bits();
  /// ```
  #[must_use]
  pub const fn least_significant_bits(&self) -> u64 {
    self.0 as u64
  }

  /// Converts a [ULID] to a epoch time as milli seconds.
  ///
  /// # Example
  ///
  /// ```rust
  /// use ulid_generator_rs::{ULIDGenerator, ULID};
  ///
  /// let mut generator: ULIDGenerator = ULIDGenerator::new();
  /// let ulid: ULID = generator.generate().unwrap();
  /// let epoch: i64 = ulid.to_epoch_milli_as_long();
  /// ```
  #[must_use]
  pub const fn to_epoch_milli_as_long(&self) -> i64 {
    (self.0 >> 80) as i64
  }

  /// Converts a [ULID] to a [`DateTime<Local>`]
  ///
  /// # Example
  ///
  /// ```rust
  /// use ulid_generator_rs::{ULIDGenerator, ULID};
  /// use chrono::{Local, DateTime};
  ///
  /// let mut generator: ULIDGenerator = ULIDGenerator::new();
  /// let ulid: ULID = generator.generate().unwrap();
  /// let date_time: DateTime<Local> = ulid.to_date_time();
  /// ```
  #[must_use]
  pub fn to_date_time(&self) -> DateTime<Local> {
    Local.timestamp_millis(self.to_epoch_milli_as_long())
  }

  /// Converts a [ULID] to a byte array.
  ///
  /// `endian` a [Endian] of byte array
  ///
  /// # Example
  ///
  /// ```rust
  /// use ulid_generator_rs::{ULIDGenerator, ULID, Endian};
  ///
  /// let mut generator: ULIDGenerator = ULIDGenerator::new();
  /// let ulid: ULID = generator.generate().unwrap();
  /// let ba: Vec<u8> = ulid.to_byte_array(Endian::BE);
  /// ```
  #[must_use]
  pub fn to_byte_array(&self, endian: Endian) -> ByteArray {
    let mut buf = vec![0; ULID_BYTES_LENGTH as usize];
    let bytes = match endian {
      Endian::LE => self.0.to_le_bytes(),
      Endian::BE => self.0.to_be_bytes(),
    };
    buf.copy_from_slice(&bytes);
    buf
  }

  /// Parse a byte array as [ULID].
  ///
  /// `byte_array` a byte array as [ULID].
  /// `endian` a [Endian] of `byte_array`.
  ///
  /// # Example
  ///
  /// ```rust
  /// use ulid_generator_rs::{ULIDGenerator, ULID, Endian};
  ///
  /// let mut generator: ULIDGenerator = ULIDGenerator::new();
  /// let ulid: ULID = generator.generate().unwrap();
  /// let ba: Vec<u8> = ulid.to_byte_array(Endian::BE);
  /// let ulid: ULID = ULID::parse_from_byte_array(ba, Endian::BE).unwrap();
  /// ```
  pub fn parse_from_byte_array(byte_array: ByteArray, endian: Endian) -> Result<Self, ULIDError> {
    if byte_array.len() != ULID_BYTES_LENGTH as usize {
      Err(ULIDError::InvalidByteArrayError)
    } else {
      let b: [u8; ULID_BYTES_LENGTH as usize] = byte_array.try_into().map_err(|_| ULIDError::InvalidByteArrayError)?;
      let result = if endian == Endian::BE {
        u128::from_be_bytes(b)
      } else {
        u128::from_le_bytes(b)
      };
      Ok(Self(result))
    }
  }
}

impl FromStr for ULID {
  type Err = ULIDError;

  /// ```rust
  /// use ulid_generator_rs::ULID;
  ///
  /// let ulid: ULID = "01ETGRM6448X1HM0PYWG2KT648".parse::<ULID>().unwrap();
  /// ```
  fn from_str(ulid_str: &str) -> Result<Self, Self::Err> {
    let value = parse_crockford_u128(ulid_str)?;
    Ok(Self(value))
  }
}

impl From<u128> for ULID {
  /// ```rust
  /// use ulid_generator_rs::ULID;
  ///
  /// let ulid: ULID = 1945530789360716160560926739305506752.into();
  /// ```
  fn from(value: u128) -> Self {
    Self::new(value)
  }
}

impl From<(u64, u64)> for ULID {
  /// ```rust
  /// use ulid_generator_rs::ULID;
  ///
  /// let ulid: ULID = (105449255778666307, 1874305465861347464).into();
  /// ```
  fn from((most_significant_bits, least_significant_bits): (u64, u64)) -> Self {
    let value: u128 = (most_significant_bits as u128) << 64 | least_significant_bits as u128;
    Self(value)
  }
}

impl TryFrom<ByteArray> for ULID {
  type Error = ULIDError;

  /// ```rust
  /// use ulid_generator_rs::{ULID, Endian};
  /// use std::convert::TryInto;
  ///
  /// let ulid: ULID = 1945530789360716160560926739305506752.into();
  /// let bytes: Vec<u8> = ulid.to_byte_array(Endian::BE);
  /// let ulid: ULID = bytes.try_into().unwrap();
  /// ```
  fn try_from(value: ByteArray) -> Result<Self, Self::Error> {
    Self::parse_from_byte_array(value, Endian::BE)
  }
}

/// This is the [ULID] Generator.
#[derive(Clone, Debug)]
pub struct ULIDGenerator {
  rng: ThreadRng,
}

unsafe impl Send for ULIDGenerator {}
unsafe impl Sync for ULIDGenerator {}

impl ULIDGenerator {
  /// The Constructor for [ULIDGenerator].
  #[must_use]
  pub fn new() -> Self {
    Self {
      rng: rand::thread_rng(),
    }
  }

  fn internal_generate<F>(&mut self, time_stamp_f: F) -> Result<ULID, ULIDError>
  where
    F: Fn() -> u64, {
    let timestamp = time_stamp_f();
    if (timestamp & TIMESTAMP_OVERFLOW_MASK) != 0 {
      Err(ULIDError::TimestampOverflowError)
    } else {
      let (most_rnd, least_significant_bits): (u16, u64) = self.rng.gen();
      let most_significant_bits = timestamp << 16 | u64::from(most_rnd);
      Ok(ULID::from((most_significant_bits, least_significant_bits)))
    }
  }

  /// Generate a [ULID].
  ///
  /// Generate a [ULID] based on the current time.
  ///
  /// # Example
  ///
  /// ```rust
  /// use ulid_generator_rs::ULIDGenerator;
  ///
  /// let mut generator = ULIDGenerator::new();
  /// let ulid = generator.generate().unwrap();
  /// ```
  pub fn generate(&mut self) -> Result<ULID, ULIDError> {
    self.internal_generate(|| Utc::now().timestamp_millis() as u64)
  }

  /// Generate a monotonic [ULID].
  ///
  /// Generate a [ULID] based on the current time,
  /// but if the current is the same timestamp as the `previous_id`,
  /// the lower 64 bits are incremented.
  ///
  /// # Example
  ///
  /// ```rust
  /// use ulid_generator_rs::ULIDGenerator;
  ///
  /// let mut generator = ULIDGenerator::new();
  /// let previous_id = generator.generate().unwrap();
  /// let ulid = generator.generate_monotonic(&previous_id).unwrap();
  /// ```
  pub fn generate_monotonic(&mut self, previous_id: &ULID) -> Result<ULID, ULIDError> {
    let timestamp = Utc::now().timestamp_millis();
    if previous_id.to_epoch_milli_as_long() == timestamp {
      Ok(previous_id.increment())
    } else {
      self.internal_generate(|| timestamp as u64)
    }
  }

  /// Generate a strictly monotonic [ULID].
  ///
  /// If the [ULID] generated by [`ULIDGenerator::generate_monotonic`] is smaller than `previous_id`,
  /// `None` is returned. Otherwise, the [ULID] will be returned wrapped in `Some`.
  ///
  /// # Example
  ///
  /// ```rust
  /// use ulid_generator_rs::ULIDGenerator;
  ///
  /// let mut generator = ULIDGenerator::new();
  /// let prev_id = generator.generate().unwrap();
  /// let ulid = generator.generate_monotonic(&prev_id).unwrap();
  /// ```
  pub fn generate_strictly_monotonic(&mut self, previous_id: &ULID) -> Result<Option<ULID>, ULIDError> {
    let result = self.generate_monotonic(previous_id)?;
    if result.partial_cmp(previous_id) == Some(Ordering::Less) {
      Ok(None)
    } else {
      Ok(Some(result))
    }
  }
}

impl Default for ULIDGenerator {
  fn default() -> Self {
    ULIDGenerator::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::convert::TryInto;

  #[test]
  fn generate() -> Result<(), ULIDError> {
    let now = Local::now().timestamp_millis();
    let ulid = ULIDGenerator::new().generate()?;
    assert!(now <= ulid.to_epoch_milli_as_long());
    Ok(())
  }

  #[test]
  fn new() {
    let ulid: ULID = (105449255778666307, 1874305465861347464).into();
    assert_eq!(ulid.to_string(), "01ETGRM6448X1HM0PYWG2KT648");
    let ulid: ULID = (105449255778666307, 1874305465861347465).into();
    assert_eq!(ulid.to_string(), "01ETGRM6448X1HM0PYWG2KT649");
    let ulid: ULID = (105449255778666307, 1874305465861347465).into();
    assert_eq!(ulid.to_string(), "01ETGRM6448X1HM0PYWG2KT649");
  }

  #[test]
  fn significant_bits() {
    let ulid: ULID = (105449255778666307, 1874305465861347464).into();
    assert_eq!(ulid.most_significant_bits(), 105449255778666307);
    assert_eq!(ulid.least_significant_bits(), 1874305465861347464);
    println!("ulid = {}", ulid);
  }

  #[test]
  fn to_date_time() {
    let ulid: ULID = 1945530789360716160560926739305506752.into();
    println!("ulid = {}", ulid);
    println!("date_time = {}", ulid.to_date_time());
  }

  #[test]
  fn bytes() -> Result<(), ULIDError> {
    let ulid_expected: ULID = ULIDGenerator::new().generate()?;
    let bytes: ByteArray = ulid_expected.to_byte_array(Endian::BE);
    let _ulid: ULID = ULID::parse_from_byte_array(bytes.clone(), Endian::BE)?;
    let ulid: ULID = bytes.try_into()?;
    println!("ulid = {}", ulid);
    assert_eq!(ulid, ulid_expected);
    Ok(())
  }

  #[test]
  fn parse_string() -> Result<(), ULIDError> {
    let s = "01ETGRM6448X1HM0PYWG2KT648";
    let ulid = s.parse::<ULID>()?;
    assert_eq!(ulid.to_string(), s);
    Ok(())
  }

  #[test]
  fn generate_monotonic() -> Result<(), ULIDError> {
    let mut generator = ULIDGenerator::new();
    let previous_id = generator.generate()?;
    let ulid = generator.generate_monotonic(&previous_id)?;
    assert!(previous_id < ulid);
    Ok(())
  }

  #[test]
  fn generate_strictly_monotonic() -> Result<(), ULIDError> {
    let mut generator = ULIDGenerator::new();
    let previous_id = generator.generate()?;
    let ulid = generator.generate_strictly_monotonic(&previous_id)?;
    assert!(previous_id < ulid.unwrap());
    Ok(())
  }
}
