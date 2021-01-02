// Copyright 2020 Developers of the `ulid-generator-rs` project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(unused_must_use)]
#![allow(unused_variables)]
#![allow(dead_code)]

use criterion::*;

use chrono::Utc;
use rand::Rng;
use ulid_generator_rs::*;

#[inline]
fn j5ik2o_uild_gen(gen: &mut ULIDGenerator) {
  gen.generate();
}

#[inline]
fn j5ik2o_ulid_to_string(uild: &ULID) {
  uild.to_string();
}

#[inline]
fn j5ik2o_ulid_gen_to_string(gen: &mut ULIDGenerator) {
  gen.generate().unwrap().to_string();
}

/// dylanhart/ulid-rs
#[inline]
fn dylanhart_ulid_rs_gen_ulid() {
  ulid::Ulid::new();
}

#[inline]
fn dylanhart_ulid_rs_to_string(ulid: &ulid::Ulid) {
  ulid.to_string();
}

#[inline]
fn dylanhart_ulid_rs_gen_to_string() {
  ulid::Ulid::new().to_string();
}

/// huxi/rusty_ulid
#[inline]
fn huxi_rusty_ulid_gen_ulid() {
  rusty_ulid::Ulid::generate();
}

#[inline]
fn huxi_rusty_ulid_to_string(ulid: &rusty_ulid::Ulid) {
  ulid.to_string();
}

#[inline]
fn huxi_rusty_ulid_gen_to_string() {
  rusty_ulid::Ulid::generate().to_string();
}

/// suyash/ulid-rs
#[inline]
fn suyash_ulid_rs_gen_ulid() {
  ulid_rs::Ulid::new(Utc::now().timestamp_millis() as u64, || rand::thread_rng().gen::<u8>());
}

#[inline]
fn suyash_ulid_rs_to_string(ulid: &ulid_rs::Ulid) {
  ulid.to_string();
}

#[inline]
fn suyash_ulid_rs_gen_to_string() {
  ulid_rs::Ulid::new(Utc::now().timestamp_millis() as u64, || rand::thread_rng().gen::<u8>()).to_string();
}

fn criterion_benchmark(c: &mut Criterion) {
  let mut group = c.benchmark_group("gen_ulid_and_to_string");
  let mut gen = ULIDGenerator::new();
  let op = 0u8;
  // group.bench_with_input(BenchmarkId::new("j5ik2o/ulid-rs/gen", op), &op, |b, i| {
  //   b.iter(|| j5ik2o_uild_gen(&mut gen))
  // });
  // group.bench_with_input(
  //   BenchmarkId::new("dylanhart/ulid-rs/gen", op),
  //   &op,
  //   |b, i| b.iter(|| dylanhart_ulid_rs_gen_ulid()),
  // );
  // group.bench_with_input(BenchmarkId::new("huxi/rusty_ulid/gen", op), &op, |b, i| {
  //   b.iter(|| huxi_rusty_ulid_gen_ulid())
  // });
  // group.bench_with_input(BenchmarkId::new("suyash/ulid-rs/gen", op), &op, |b, i| {
  //   b.iter(|| suyash_ulid_rs_gen_ulid())
  // });
  // // ---
  // let ulid = gen.generate().unwrap();
  // group.bench_with_input(BenchmarkId::new("j5ik2o/ulid-rs/str", op), &op, |b, i| {
  //   b.iter(|| j5ik2o_ulid_to_string(&ulid))
  // });
  // let ulid = ulid::Ulid::new();
  // group.bench_with_input(
  //   BenchmarkId::new("dylanhart/ulid-rs/str", op),
  //   &op,
  //   |b, i| b.iter(|| dylanhart_ulid_rs_to_string(&ulid)),
  // );
  // let ulid = rusty_ulid::Ulid::generate();
  // group.bench_with_input(BenchmarkId::new("huxi/rusty_ulid/str", op), &op, |b, i| {
  //   b.iter(|| huxi_rusty_ulid_to_string(&ulid))
  // });
  // let ulid = ulid_rs::Ulid::new(Utc::now().timestamp_millis() as u64, || {
  //   rand::thread_rng().gen::<u8>()
  // });
  // group.bench_with_input(BenchmarkId::new("suyash/ulid-rs/str", op), &op, |b, i| {
  //   b.iter(|| suyash_ulid_rs_to_string(&ulid))
  // });
  // ---
  group.bench_with_input(BenchmarkId::new("j5ik2o/ulid-rs/gen_to_str", op), &op, |b, i| {
    b.iter(|| j5ik2o_ulid_gen_to_string(&mut gen))
  });
  group.bench_with_input(BenchmarkId::new("dylanhart/ulid-rs/gen_to_str", op), &op, |b, i| {
    b.iter(|| dylanhart_ulid_rs_gen_to_string())
  });
  group.bench_with_input(BenchmarkId::new("huxi/rusty_ulid/gen_to_str", op), &op, |b, i| {
    b.iter(|| huxi_rusty_ulid_gen_to_string())
  });
  group.bench_with_input(BenchmarkId::new("suyash/ulid-rs/gen_to_str", op), &op, |b, i| {
    b.iter(|| suyash_ulid_rs_gen_to_string())
  });
  group.finish();
}

criterion_group!(benches, criterion_benchmark);

criterion_main! {
benches,
}
