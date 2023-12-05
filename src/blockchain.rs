use crate::block;
use anyhow::{self, Ok, Result};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use std::{mem, rc::Rc};

type Block8 = block::Block;
pub struct Blockchain {
    pub chain: Vec<Block8>,
}

impl Default for Blockchain {
    fn default() -> Self {
        Blockchain { chain: Vec::new() }
    }
}

impl Blockchain {
    pub fn new() -> Self {
        Blockchain::default()
    }
    pub fn add_block(&mut self, block: Block8) -> Result<bool> {
        if self.chain.len() < 20 {
            let block_valid = Rc::try_unwrap(block.valid.clone()).unwrap_or_else(|_| panic!(""));
            if block_valid {
                let mut block_data = [
                    block.block_data.get(0).unwrap().as_str().as_bytes(),
                    block.block_hash,
                ]
                .concat();
                let block_bytes = String::from_utf8(mem::take(&mut block_data))?;
                let mut rng = rand::thread_rng();
                let bits = 2048;
                let secret = RsaPrivateKey::new(&mut rng, bits)?;
                let public = RsaPublicKey::from(&secret);
                let enc_block =
                    public.encrypt(&mut rng, Pkcs1v15Encrypt, block_bytes.as_bytes())?;
                let dec_block = secret.decrypt(Pkcs1v15Encrypt, &enc_block)?;
                assert_eq!(&block_data[..], &dec_block[..]);
                self.chain.push(block);
                return Ok(true);
            }
        }
        Ok(false)
    }
}
