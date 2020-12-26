use std::cmp::Ordering;
use std::convert::TryFrom;
use std::str::FromStr;

use chrono::{DateTime, Duration, Local, TimeZone, Utc};
use rand::rngs::ThreadRng;
use rand::RngCore;

use crate::ULIDError::RandomGenError;

const ULID_BYTES_LENGTH: i32 = 16;
const TIMESTAMP_OVERFLOW_MASK: u64 = 0xffff000000000000;
const MASK_BITS: u32 = 5;
const MASK: u64 = 0x1f;
const ENCODING_CHARS: [char; 32] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J',
    'K', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'V', 'W', 'X', 'Y', 'Z',
];
const DECODING_CHARS: [i32; 123] = [
    // 0
    -1, -1, -1, -1, -1, -1, -1, -1, // 8
    -1, -1, -1, -1, -1, -1, -1, -1, // 16
    -1, -1, -1, -1, -1, -1, -1, -1, // 24
    -1, -1, -1, -1, -1, -1, -1, -1, // 32
    -1, -1, -1, -1, -1, -1, -1, -1, // 40
    -1, -1, -1, -1, -1, -1, -1, -1, // 48
    0, 1, 2, 3, 4, 5, 6, 7, // 56
    8, 9, -1, -1, -1, -1, -1, -1, // 64
    -1, 10, 11, 12, 13, 14, 15, 16, // 72
    17, 1, 18, 19, 1, 20, 21, 0, // 80
    22, 23, 24, 25, 26, -1, 27, 28, // 88
    29, 30, 31, -1, -1, -1, -1, -1, // 96
    -1, 10, 11, 12, 13, 14, 15, 16, // 104
    17, 1, 18, 19, 1, 20, 21, 0, // 112
    22, 23, 24, 25, 26, -1, 27, 28, // 120
    29, 30, 31,
];

fn internal_write_crockford(value: u64, count: u32) -> String {
    (0..count)
        .into_iter()
        .fold("".to_string(), |mut result, i| {
            let index = (value >> ((count - i - 1) * MASK_BITS)) & MASK;
            result.push(ENCODING_CHARS[index as usize]);
            result
        })
}

fn interal_parse_crockford(input: &str) -> u64 {
    let length = input.len();
    if length > 12 {
        panic!("input length must not exceed 12 but was {}!", length)
    }
    let result = input
        .chars()
        .enumerate()
        .into_iter()
        .fold(0u64, |result, (i, current)| {
            let value = if (current as usize) < DECODING_CHARS.len() {
                DECODING_CHARS[current as usize]
            } else {
                -1
            };
            if value < 0 {
                panic!("Illegal character '{}'!", current)
            }
            let factor = (length as u32) - 1u32 - (i as u32);
            let value = (value as u64) << (factor * MASK_BITS);
            result | value
        });
    result
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
        let mut result = internal_write_crockford(self.to_epoch_milli_as_long(), 10);
        result.push_str(&internal_write_crockford(
            (self.most_significant_bits & 0xffff) << 24 | self.least_significant_bits >> 40,
            8,
        ));
        result.push_str(&internal_write_crockford(self.least_significant_bits, 8));
        result
    }
}

impl ULID {
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

#[derive(Debug, Clone)]
pub enum ULIDError {
    RandomGenError { msg: String },
    InvalidByteArrayError,
    TimestampOverflowError,
}

type ByteArray = Vec<u8>;

impl FromStr for ULID {
    type Err = ULIDError;

    fn from_str(ulid_str: &str) -> Result<Self, Self::Err> {
        let len = ulid_str.len();
        let ts = interal_parse_crockford(&ulid_str[0..10]);
        if (ts & TIMESTAMP_OVERFLOW_MASK) != 0 {
            return Err(ULIDError::TimestampOverflowError);
        }
        let part1 = interal_parse_crockford(&ulid_str[10..18]);
        let part2 = interal_parse_crockford(&ulid_str[18..len]);

        let most_significant_bits = (ts << 16) | (part1 >> 24);
        let least_significant_bits = part2 | (part1 << 40);
        Ok(ULID::new(most_significant_bits, least_significant_bits))
    }
}

impl TryFrom<ByteArray> for ULID {
    type Error = ULIDError;

    fn try_from(value: ByteArray) -> Result<Self, Self::Error> {
        if value.len() != ULID_BYTES_LENGTH as usize {
            Err(ULIDError::InvalidByteArrayError)
        } else {
            let mut most_significant_bits = 0u64;
            for i in 0..8 {
                most_significant_bits = (most_significant_bits << 8) | (value[i] & 0xff) as u64;
            }
            let mut least_significant_bits = 0u64;
            for i in 8..16 {
                least_significant_bits = (least_significant_bits << 8) | (value[i] & 0xff) as u64;
            }
            Ok(ULID::new(most_significant_bits, least_significant_bits))
        }
    }
}

impl ULIDGenerator {
    pub fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
        }
    }

    pub fn generate(&mut self) -> ULID {
        let ts = Self::unix_time_stamp() as u64;
        Self::check_timestamp(ts);
        let (r1, r2) = Self::generate_random(|usize| self.random(usize).unwrap());
        let most_significant_bits = (ts << 16) | (r1 >> 24);
        let least_significant_bits = (r1 << 40) | r2;
        ULID::new(most_significant_bits, least_significant_bits)
    }

    fn check_timestamp(timestamp: u64) {
        if (timestamp & TIMESTAMP_OVERFLOW_MASK) != 0 {
            panic!("ULID does not support timestamps after +10889-08-02T05:31:50.655Z!")
        }
    }

    fn random(&mut self, size: usize) -> Result<ByteArray, ULIDError> {
        let mut b: Vec<u8> = Vec::with_capacity(size);
        b.resize(size, 0u8);
        let result = self.rng.try_fill_bytes(&mut b[..]);
        match result {
            Ok(_) => Ok(b),
            Err(e) => Err(RandomGenError { msg: e.to_string() }),
        }
    }

    fn unix_time_stamp() -> i64 {
        Utc::now().timestamp_millis()
    }

    #[inline]
    fn generate_random<F>(mut random_gen: F) -> (u64, u64)
    where
        F: FnMut(usize) -> ByteArray,
    {
        let bytes = random_gen(10);

        let mut random1 = ((bytes[0x0] & 0xff) as u64) << 32;
        random1 |= ((bytes[0x1] & 0xff) as u64) << 24;
        random1 |= ((bytes[0x2] & 0xff) as u64) << 16;
        random1 |= ((bytes[0x3] & 0xff) as u64) << 8;
        random1 |= (bytes[0x4] & 0xff) as u64;

        let mut random2 = ((bytes[0x5] & 0xff) as u64) << 32;
        random2 |= ((bytes[0x6] & 0xff) as u64) << 24;
        random2 |= ((bytes[0x7] & 0xff) as u64) << 16;
        random2 |= ((bytes[0x8] & 0xff) as u64) << 8;
        random2 |= (bytes[0x9] & 0xff) as u64;

        (random1, random2)
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use crate::{ULIDGenerator, ULID};

    #[test]
    fn it_works() {
        let mut ulid_generator = ULIDGenerator::new();
        let ulid: ULID = ulid_generator.generate();
        println!("{:?}", ulid);
        println!("{:?}", ulid.to_string());

        let s = ulid.to_string().parse::<ULID>();
        println!("{:?}", s);

        let b = ulid.to_bytes();
        println!("{:?}", b);
        let t = ULID::try_from(b).unwrap();
        println!("{:?}", t);
    }
}
