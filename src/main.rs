use rand::rngs::OsRng;
use rsa::{RsaPrivateKey, RsaPublicKey, Pkcs1v15Encrypt};
use secp256k1::Message;

use crate::block::block::_BlockT;

pub mod sec8;
pub mod block;
pub mod blockchain;
pub mod account;
pub mod transaction;

fn _check_rsa(msg:&str) {
    let mut rng = rand::thread_rng();
    let bits = 2048;
    let private_key =  RsaPrivateKey::new(&mut rng, bits).unwrap();
    let public_key = RsaPublicKey::from(&private_key);
    let encrypt_data = public_key.encrypt(&mut rng, Pkcs1v15Encrypt, msg.as_bytes()).unwrap();
    let decrypt_data = private_key.decrypt(Pkcs1v15Encrypt, &encrypt_data).unwrap();
    assert_ne!(msg.as_bytes(), encrypt_data);
    assert_eq!(msg.as_bytes(), &decrypt_data[..]);
}

fn _check_secp256k1(msg:&[u8]) {
    let secp = secp256k1::Secp256k1::new();
    let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
    println!("sec-key = {:?} pub-key = {:?}", secret_key, public_key);
    let message = Message::from_digest_slice(msg);
    match message {
        Ok(mesx) => {
            let sig = secp.sign_ecdsa(&mesx, &secret_key);
            assert!(secp.verify_ecdsa(&mesx, &sig, &public_key).is_ok());
        },
        Err(e) => {
            eprintln!("{:?}", e)
        }
    };
}

fn main() {
    let genesis = block::block::Block::new();
    let account = account::account::Account::create("hello").unwrap();
    let cb = <block::block::Block as _BlockT>::create_essex_block(genesis,account,"hello").unwrap();
    let der = serde_json::to_string(&cb).unwrap();
    println!("block={:?}",der);
}
