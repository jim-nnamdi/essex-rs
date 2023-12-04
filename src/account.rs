use rand::rngs::OsRng;
use secp256k1::{SecretKey, Message, ecdsa::Signature, PublicKey};
use anyhow::{Result, Ok, Error};

#[derive(Debug)]
pub struct Account {
  pub acc_private: SecretKey,
  pub acc_public: PublicKey,
  pub acc_signed : Signature,
  pub acc_balance: u32
}

impl Account {
  pub fn create(msg:&str) -> Result<Account, Error> {
    let secp = secp256k1::Secp256k1::new();
    let (secret, public) = secp.generate_keypair(&mut OsRng);
    log::info!("secret: {:?} public: {:?}", secret, public);
    let mess = Message::from_digest_slice(msg.as_bytes())?;
    let sig = secp.sign_ecdsa(&mess, &secret);
    let new_acc = Account { acc_private: secret, acc_public: public, acc_signed: sig, acc_balance:0 };
    log::info!("new-acc created: {:?}", new_acc);
    Ok(new_acc)
  }
}