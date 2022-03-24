use ethcontract::{
  transport::DynTransport,
  web3::{
    signing::{Key as Web3Key, Signature, SigningError},
    types::{Address as Web3Address, H256},
  },
  Account, Http, Web3,
};
use std::convert::TryInto;
use tideth::safe::SafeClient;

ethcontract::contract!("./artifacts/contracts/RustCoin.sol/RustCoin.json");

#[tokio::test]
async fn main() {
  let web3 = Web3::new(DynTransport::new(
    Http::new("http://localhost:8545").expect("couldnt setup web3"),
  ));

  let accounts = web3.eth().accounts().await.expect("getAccounts failed");

  let zero_account = Account::Local(accounts[0], None);

  let mut safe = SafeClient::new(&web3, None).expect("derp");
  safe
    .deploy(zero_account.clone())
    .await
    .expect("Didnt deploy");

  let owners = vec![accounts[0], accounts[1], accounts[2]];
  safe
    .setup(zero_account.clone(), owners, 2)
    .await
    .expect("Couldnt setup");

  // Deploy an ERC20 and send 100 tokens to the safe
  let erc20 = RustCoin::builder(&web3)
    //.gas(1_000_000_u64.into())
    .deploy()
    .await
    .expect("Couldn't deploy the ERC20");

  erc20
    .transfer(safe.address(), 100_u64.into())
    .send()
    .await
    .expect("Couldn't send the ERC20 to the safe");
  assert_eq!(
    erc20
      .balance_of(safe.address())
      .call()
      .await
      .expect("Couldn't get safe's balance"),
    100_u64.into()
  );

  let nonce = 0;
  let tx_data = safe
    .encode_erc20_tx(erc20.address(), accounts[3], 100, nonce)
    .await
    .expect("couldnt build erc20 tx");

  let tx_hash = keccak256(tx_data.0.as_ref());
  // accounts[0]
  let sk1 = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
  let mut sk_bytes1 = [0u8; 32];
  hex::decode_to_slice(sk1, &mut sk_bytes1 as &mut [u8]).expect("couldnt decode hex");
  let secret1 = SecretKey::from_bytes(&sk_bytes1).expect("couldnt parse");
  let sig1 = SecretKeyRef(&secret1)
    .sign(&tx_hash, None)
    .expect("couldnt sign");

  // accounts[1]
  let sk2 = "59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
  let mut sk_bytes2 = [0u8; 32];
  hex::decode_to_slice(sk2, &mut sk_bytes2 as &mut [u8]).expect("couldnt decode hex");
  let secret2 = SecretKey::from_bytes(&sk_bytes2).expect("couldnt parse");
  let sig2 = SecretKeyRef(&secret2)
    .sign(&tx_hash, None)
    .expect("couldnt sign");

  // Gnosis defines signatures as r, s, v tuples
  // The addresses are accounts[1] and accounts[0]
  // They do need to be in a sorted order
  let sorted_sigs = if accounts[0] < accounts[1] {
    vec![sig1, sig2]
  } else {
    vec![sig2, sig1]
  };
  let mut signatures = vec![];
  for sig in sorted_sigs {
    signatures.append(&mut sig.r.as_ref().to_vec());
    signatures.append(&mut sig.s.as_ref().to_vec());
    signatures.push(sig.v as u8);
  }

  safe
    .exec_erc20_tx(
      zero_account.clone(),
      erc20.address(),
      accounts[3],
      100,
      signatures,
    )
    .await
    .expect("Couldn't execute the TX");
  assert_eq!(
    erc20
      .balance_of(safe.address())
      .call()
      .await
      .expect("Couldn't get safe's balance"),
    0_u64.into()
  );
  assert_eq!(
    erc20
      .balance_of(accounts[3])
      .call()
      .await
      .expect("Couldn't get safe's balance"),
    100_u64.into()
  );
  println!("Executed!");
}

pub struct SecretKey(libsecp256k1::SecretKey);

impl SecretKey {
  /// Get a secret key from a raw byte array.
  pub fn from_bytes(bytes: &[u8; 32]) -> std::result::Result<Self, String> {
    match libsecp256k1::SecretKey::parse(bytes) {
      Ok(k) => Ok(Self(k)),
      Err(_) => Err("couldnt parse".to_string()),
    }
  }
  /// Sign the specified hashed message.
  pub fn sign(&self, msg: &[u8; 32]) -> (libsecp256k1::Signature, libsecp256k1::RecoveryId) {
    libsecp256k1::sign(&libsecp256k1::Message::parse(msg), &self.0)
  }
  pub fn public_key(&self) -> libsecp256k1::PublicKey {
    libsecp256k1::PublicKey::from_secret_key(&self.0)
  }
}
struct SecretKeyRef<'a>(&'a SecretKey);

impl<'a> Web3Key for SecretKeyRef<'a> {
  fn sign(&self, message: &[u8], chain_id: Option<u64>) -> Result<Signature, SigningError> {
    let (signature, recovery_id) = self.0.sign(
      message[0..32]
        .try_into()
        .expect("secp256k1 message must contain exactly 32 bytes"),
    );

    let standard_v = recovery_id.serialize() as u64;

    let v = if let Some(chain_id) = chain_id {
      // When signing with a chain ID, add chain replay protection.
      standard_v + 35 + chain_id * 2
    } else {
      // Otherwise, convert to standard notation.
      standard_v + 27
    };
    let signature = signature.serialize();
    let r = H256::from_slice(&signature[..32]);
    let s = H256::from_slice(&signature[32..]);

    Ok(Signature { v, r, s })
  }

  fn sign_message(&self, message: &[u8]) -> Result<Signature, SigningError> {
    self.sign(message, None)
  }

  fn address(&self) -> Web3Address {
    let public_key = self.0.public_key();
    let public_key = public_key.serialize();

    debug_assert_eq!(public_key[0], 0x04);
    let hash = keccak256(&public_key[1..]);

    Web3Address::from_slice(&hash[12..])
  }
}

fn keccak256(bytes: &[u8]) -> [u8; 32] {
  use tiny_keccak::{Hasher, Keccak};
  let mut output = [0u8; 32];
  let mut hasher = Keccak::v256();
  hasher.update(bytes);
  hasher.finalize(&mut output);
  output
}
