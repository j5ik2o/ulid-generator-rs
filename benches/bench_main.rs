use criterion::*;

use ulid_rs::ULIDGenerator;

#[inline]
fn gen_ulid_and_then_to_string(gen: &mut ULIDGenerator) {
  gen.generate().to_string();
}

fn criterion_benchmark(c: &mut Criterion) {
  let mut gen = ULIDGenerator::new();
  c.bench_function("gen_ulid_and_then_to_string", |b| {
    b.iter(|| gen_ulid_and_then_to_string(&mut gen))
  });
}

criterion_group!(benches, criterion_benchmark);

criterion_main! {
benches,
}
