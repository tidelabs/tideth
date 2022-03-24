use ethcontract::{
  transport::DynTransport,
  web3::{
    signing::{Key as Web3Key, Signature, SigningError},
    types::{Address as Web3Address, H256},
  },
  Account, Http, Web3, H160,
};
use std::{convert::TryInto, str::FromStr};
use tideth::{router::RouterClient, safe::SafeClient};

ethcontract::contract!("artifacts/contracts/RustCoin.sol/RustCoin.json");

const ERC20_ADDRESS: Option<&str> = None;
const ROUTER_ADDRESS: Option<&str> = None;
const SAFE_ADDRESS: Option<&str> = None;

// const ERC20_ADDRESS: Option<&str> = Some("0x95401dc811bb5740090279ba06cfa8fcf6113778");
// const ROUTER_ADDRESS: Option<&str> = Some("0xf5059a5d33d5853360d16c683c16e67980206f36");
// const SAFE_ADDRESS: Option<&str> = Some("0x4826533b4897376654bb4d4ad88b7fafd0c98528");

const ETH_URL: &str = "http://localhost:8545";

#[tokio::test]
async fn main() {
  let web3 = Web3::new(DynTransport::new(
    Http::new(ETH_URL).expect("couldnt setup web3"),
  ));

  let accounts = web3.eth().accounts().await.expect("getAccounts failed");
  let coin_amt = 10;
  let send_amt = 9; //

  let zero_account = Account::Local(accounts[0], None);

  let mut router = RouterClient::new(&web3, ROUTER_ADDRESS).expect("derp");
  if let None = ROUTER_ADDRESS {
    router
      .deploy(zero_account.clone())
      .await
      .expect("Didnt deploy");
  }

  let erc20 = if let Some(addy) = ERC20_ADDRESS {
    RustCoin::at(
      &web3,
      H160::from_str(addy).expect("couldnt parse erc20 addy"),
    )
  } else {
    RustCoin::builder(&web3)
      //.gas(1_000_000_u64.into())
      .deploy()
      .await
      .expect("Couldn't deploy the ERC20")
  };
  println!("erc20 address: {:?}", erc20.address());

  let bal1: u128 = erc20
    .balance_of(router.address())
    .call()
    .await
    .expect("Couldn't get router's balance")
    .as_u128();
  erc20
    .transfer(router.address(), coin_amt.into())
    .send()
    .await
    .expect("Couldn't send the ERC20 to the router");
  let bal2: u128 = erc20
    .balance_of(router.address())
    .call()
    .await
    .expect("Couldn't get router's balance")
    .as_u128();
  assert_eq!(bal2, bal1 + coin_amt as u128);

  let user_bal1: u128 = erc20
    .balance_of(accounts[3])
    .call()
    .await
    .expect("Couldn't get user's balance")
    .as_u128();

  println!("Router balance = {:?}! {:?}", coin_amt, router.address());

  let mut safe = SafeClient::new(&web3, SAFE_ADDRESS).expect("derp");
  if let None = SAFE_ADDRESS {
    safe
      .deploy(zero_account.clone())
      .await
      .expect("Didnt deploy");
    let owners = vec![accounts[0], accounts[1], accounts[2]];
    safe
      .setup(zero_account.clone(), owners, 2)
      .await
      .expect("Couldnt setup");
  }

  let mut nonce = safe.nonce().await.expect("couldnt get safe nonce");
  println!("NONCE {:?}", nonce);

  // return ();

  let current_owner = router.owner().await.expect("couldnt get currenct owner");
  // println!("CURRENT OWNER {:?}", current_owner);
  // println!("SAFE ADDY {:?}", safe.address());

  if current_owner != safe.address() {
    router
      .transfer_ownership(zero_account.clone(), safe.address())
      .await
      .expect("Couldnt transfer router ownership to safe");
    let claimdata = router
      .claim_ownership_data()
      .expect("could make claim_ownership_data");
    let tx_data = safe
      .encode_data(router.address(), 0, claimdata.clone(), nonce)
      .await
      .expect("couldnt encode claim data");

    let signatures = all_sigs(tx_data.0, accounts.clone());
    safe
      .exec_with_data(
        zero_account.clone(),
        router.address(),
        claimdata,
        signatures,
        0,
      )
      .await
      .expect("Couldn't execute the Claim");
    // increment the nonce
    nonce = nonce + 1;
    println!("safe owns router!");
  } else {
    println!("safe already owns router");
  }

  let own = router.owner().await.expect("couldnt call owner");
  println!("router owner: {:?}", own);

  let withdrawaldata = router
    .erc20_withdrawal_data(accounts[3], erc20.address(), send_amt)
    .expect("Couldnt build withdrawal data");

  // must increment nonce by one each EXEC
  let wtxdata = safe
    .encode_data(router.address(), 0, withdrawaldata.clone(), nonce)
    .await
    .expect("couldnt encode withdrawal data");

  let signatures = all_sigs(wtxdata.0, accounts.clone());

  safe
    .exec_with_data(
      zero_account,
      router.address(),
      withdrawaldata,
      signatures,
      0,
    )
    .await
    .expect("Couldn't execute the TX");

  println!("exectued withdrawal!");
  println!("BAL2 {}", bal2);
  let bal3: u128 = erc20
    .balance_of(router.address())
    .call()
    .await
    .expect("Couldn't get router's balance")
    .as_u128();
  assert_eq!(bal2 - send_amt, bal3);
  println!("router has {:?} erc20 left!", (coin_amt - send_amt));

  let user_bal2: u128 = erc20
    .balance_of(accounts[3])
    .call()
    .await
    .expect("Couldn't get user's balance")
    .as_u128();
  assert_eq!(user_bal1 + send_amt, user_bal2);
  println!("accounts[3] has {:?} erc20!", user_bal2);

  // Ok(())
}

fn all_sigs(data: Vec<u8>, accounts: Vec<H160>) -> Vec<u8> {
  let tx_hash = keccak256(data.as_ref());
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

  signatures
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
