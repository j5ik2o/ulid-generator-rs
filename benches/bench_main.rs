use criterion::*;

use ulid_rs_j5ik2o::ULIDGenerator;
use chrono::Utc;
use rand::Rng;

#[inline]
fn j5ik2o_uild_gen(gen: &mut ULIDGenerator) {
  gen.generate();
}

#[inline]
fn j5ik2o_ulid_to_string(uild: &ulid_rs_j5ik2o::ULID) {
  uild.to_string();
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

/// huxi/rusty_ulid
#[inline]
fn huxi_rusty_ulid_gen_ulid() {
  rusty_ulid::Ulid::generate();
}

#[inline]
fn huxi_rusty_ulid_to_string(ulid: &rusty_ulid::Ulid) {
  ulid.to_string();
}

/// suyash/ulid-rs
#[inline]
fn suyash_ulid_rs_gen_ulid() {
  ulid_rs::Ulid::new(Utc::now().timestamp_millis() as u64, || {
    rand::thread_rng().gen::<u8>()
  });
}

#[inline]
fn suyash_ulid_rs_to_string(ulid: &ulid_rs::Ulid) {
  ulid.to_string();
}

fn criterion_benchmark(c: &mut Criterion) {
  let mut group = c.benchmark_group("gen_ulid_and_to_string");
  let mut gen = ULIDGenerator::new();
  let op = 0u8;
  group.bench_with_input(BenchmarkId::new("j5ik2o/ulid-rs/gen", op), &op, |b, i| {
    b.iter(|| j5ik2o_uild_gen(&mut gen))
  });
  group.bench_with_input(
    BenchmarkId::new("dylanhart/ulid-rs/gen", op),
    &op,
    |b, i| b.iter(|| dylanhart_ulid_rs_gen_ulid()),
  );
  group.bench_with_input(BenchmarkId::new("huxi/rusty_ulid/gen", op), &op, |b, i| {
    b.iter(|| huxi_rusty_ulid_gen_ulid())
  });
  group.bench_with_input(BenchmarkId::new("suyash/ulid-rs/gen", op), &op, |b, i| {
    b.iter(|| suyash_ulid_rs_gen_ulid())
  });
  // ---
  let ulid = gen.generate().unwrap();
  group.bench_with_input(BenchmarkId::new("j5ik2o/ulid-rs/str", op), &op, |b, i| {
    b.iter(|| j5ik2o_ulid_to_string(&ulid))
  });
  let ulid = ulid::Ulid::new();
  group.bench_with_input(
    BenchmarkId::new("dylanhart/ulid-rs/str", op),
    &op,
    |b, i| b.iter(|| dylanhart_ulid_rs_to_string(&ulid)),
  );
  let ulid = rusty_ulid::Ulid::generate();
  group.bench_with_input(BenchmarkId::new("huxi/rusty_ulid/str", op), &op, |b, i| {
    b.iter(|| huxi_rusty_ulid_to_string(&ulid))
  });
  let ulid = ulid_rs::Ulid::new(Utc::now().timestamp_millis() as u64, || {
    rand::thread_rng().gen::<u8>()
  });
  group.bench_with_input(BenchmarkId::new("suyash/ulid-rs/str", op), &op, |b, i| {
    b.iter(|| suyash_ulid_rs_to_string(&ulid))
  });
  group.finish();
}

criterion_group!(benches, criterion_benchmark);

criterion_main! {
benches,
}
