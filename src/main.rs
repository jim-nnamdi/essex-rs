use rand::rngs::OsRng;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use secp256k1::Message;

use crate::block::block::_BlockT;

pub mod account;
pub mod block;
pub mod blockchain;
pub mod dynamic;
pub mod sec8;
pub mod transaction;

fn _check_rsa(msg: &str) {
    let mut rng = rand::thread_rng();
    let bits = 2048;
    let private_key = RsaPrivateKey::new(&mut rng, bits).unwrap();
    let public_key = RsaPublicKey::from(&private_key);
    let encrypt_data = public_key
        .encrypt(&mut rng, Pkcs1v15Encrypt, msg.as_bytes())
        .unwrap();
    let decrypt_data = private_key.decrypt(Pkcs1v15Encrypt, &encrypt_data).unwrap();
    assert_ne!(msg.as_bytes(), encrypt_data);
    assert_eq!(msg.as_bytes(), &decrypt_data[..]);
}

fn _check_secp256k1(msg: &[u8]) {
    let secp = secp256k1::Secp256k1::new();
    let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
    dbg!(&secret_key, &public_key);
    let message = Message::from_digest_slice(msg);
    match message {
        Ok(mesx) => {
            let sig = secp.sign_ecdsa(&mesx, &secret_key);
            assert!(secp.verify_ecdsa(&mesx, &sig, &public_key).is_ok());
        }
        Err(e) => {
            eprintln!("{:?}", e)
        }
    };
}

fn _tsmain() {
    let genesis = block::block::Block::new();
    let account = account::account::Account::create("hello").unwrap();
    let cb =
        <block::block::Block as _BlockT>::create_essex_block(genesis, account, "hello").unwrap();
    let bk = blockchain::blockchain::Blockchain::_add_block_to_chain(cb.clone());
    println!("{:?}", bk);
}


fn main(){
    let _ = dynamic::_essex_sim("test-net", "/ip4/0.0.0.0/udp/0/quic-v1".parse().unwrap(),"/ip4/0.0.0.0/tcp/0".parse().unwrap()).expect("essex conn error ...");
}