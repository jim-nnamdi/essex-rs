use crate::block;
use anyhow::{self, Ok, Result};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use std::{mem, time::SystemTime};

type Block8 = block::block::Block;
pub struct Blockchain {
    pub chain: Vec<Block8>,
    pub timestamp:SystemTime
}

impl Default for Blockchain {
    fn default() -> Self {
        Blockchain { chain: Vec::new() , timestamp:SystemTime::now()}
    }
}

impl Blockchain {
    pub fn new() -> Self {
        Blockchain::default()
    }
    pub fn add_block(&mut self, block: Block8) -> Result<Blockchain> {
        if self.chain.len() < 20 {
            let block_valid = block.valid.clone();
            if block_valid {
                let mut block_data = [block.block_data.get(0).unwrap().as_str().as_bytes()].concat();
                let block_bytes = String::from_utf8(mem::take(&mut block_data))?;
                let mut rng = rand::thread_rng();
                let bits = 2048;
                let secret = RsaPrivateKey::new(&mut rng, bits)?;
                let public = RsaPublicKey::from(&secret);
                let enc_block =
                    public.encrypt(&mut rng, Pkcs1v15Encrypt, block_bytes.as_bytes())?;
                let dec_block = secret.decrypt(Pkcs1v15Encrypt, &enc_block)?;
                assert_eq!(&block_data[..], &dec_block[..]);
                let new_blockchain = Blockchain{chain:vec![block.clone()], timestamp:SystemTime::now()};
                return Ok(new_blockchain);
            }
        }
        // for every default blockchain returned
        // invalidate the network resource ...
        Ok(Blockchain::default())
    }
}
