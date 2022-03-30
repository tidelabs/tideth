// Copyright 2021-2022 Semantic Network Ltd.
// This file is part of tideth.

// tideth is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Tidechain is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with tideth.  If not, see <http://www.gnu.org/licenses/>.

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
