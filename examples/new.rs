use ulid_generator_rs::ULIDGenerator;

fn main() {
  let ulid = ULIDGenerator::new().generate().unwrap();
  println!("{}", ulid);
}
