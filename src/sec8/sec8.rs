use rand::rngs::OsRng;
use rsa::{RsaPrivateKey, RsaPublicKey, Pkcs1v15Encrypt};
use secp256k1::{SecretKey, PublicKey};

use anyhow::{Ok,Result};

pub fn sec8_tx_id_hash() -> Result<(SecretKey, PublicKey)> {
  let secp = secp256k1::Secp256k1::new();
  let (sk, pk) = secp.generate_keypair(&mut OsRng);
  Ok((sk, pk))
}

pub fn sec8_block_id_hash()-> Result<(SecretKey, PublicKey)> {
  let secp = secp256k1::Secp256k1::new();
  let (sk, pk) = secp.generate_keypair(&mut OsRng);
  Ok((sk, pk))
}

pub fn sec8_encrypt_block_data(bits_size: usize, block_data:&[u8]) -> Result<Vec<u8>>{
  let mut rng = rand::thread_rng();
  let sec8_private = RsaPrivateKey::new(&mut rng, bits_size)?;
  let sec8_public = RsaPublicKey::from(&sec8_private);
  let sec8_data_encrypt = sec8_public.encrypt(&mut rng, Pkcs1v15Encrypt,block_data)?;
  Ok(sec8_data_encrypt)
}

pub fn sec8_decrypt_block_data(bits_size: usize, block_data:Vec<u8>) -> Result<Vec<u8>>{
  let mut rng = rand::thread_rng();
  let sec8_private = RsaPrivateKey::new(&mut rng, bits_size)?;
  let sec8_data_decrypt = sec8_private.decrypt(Pkcs1v15Encrypt,&block_data)?;
  Ok(sec8_data_decrypt)
}
