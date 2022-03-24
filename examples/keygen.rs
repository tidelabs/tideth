use ethcontract::PrivateKey;
use rand::Rng;
use std::str::FromStr;

fn main() {
  let random_bytes = rand::thread_rng().gen::<[u8; 32]>();
  let privkey = hex::encode(random_bytes);
  println!("PRIV {}", privkey);
  let k = PrivateKey::from_str(&privkey);
  println!("P {:?}", k);
}
