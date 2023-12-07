use rand::rngs::OsRng;
use secp256k1::{SecretKey, Message, ecdsa::Signature, PublicKey, hashes::sha256};
use anyhow::{Result, Ok, Error};

#[derive(Debug)]
pub struct Account {
  pub acc_private: SecretKey,
  pub acc_public: PublicKey,
  pub acc_signed : Signature,
  pub acc_balance: u32
}

impl Account {
  // user should store the msg 
  // msg would be needed to create block
  pub fn create(msg:&str) -> Result<Account, Error> {
    let secp = secp256k1::Secp256k1::new();
    let (secret, public) = secp.generate_keypair(&mut OsRng);
    log::info!("secret: {:?} public: {:?}", secret, public);
    let mess = Message::from_hashed_data::<sha256::Hash>(msg.as_bytes());
    let sig = secp.sign_ecdsa(&mess, &secret);
    let new_acc = Account { acc_private: secret, acc_public: public, acc_signed: sig, acc_balance:0 };
    log::info!("new-acc created: {:?}", new_acc);
    Ok(new_acc)
  }
}