use crate::block;
use anyhow::{self, Ok, Result};
use rand::rngs::OsRng;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use secp256k1::{hashes::sha256, Message};
use serde::{Deserialize, Serialize};
use std::{time::SystemTime, io::Write};

type Block8 = block::block::Block;

#[derive(Serialize, Deserialize, Debug)]
pub struct Blockchain {
    pub chain: Vec<Block8>,
    pub timestamp: SystemTime,
}

impl Default for Blockchain {
    fn default() -> Self {
        Blockchain {
            chain: vec![],
            timestamp: SystemTime::now(),
        }
    }
}

impl Blockchain {
    pub fn new() -> Self {
        Blockchain::default()
    }
    pub fn _add_block_to_chain(block: Block8) -> Self {
        // check to see if block is valid
        // pass in custom built function to check
        // and verify validity of the block
        if block.valid {
            // hash the entire blockchain data
            // using secp256k1 algorithm: see ECC
            let blck_fn = secp256k1::Secp256k1::new();
            let blck_kp = blck_fn.generate_keypair(&mut OsRng);
            // using the block hash we hash and generate a message digest
            // to completely hash the data ...
            let blck_msg = Message::from_hashed_data::<sha256::Hash>(block.block_hash.as_bytes());
            let blck_sig = blck_fn.sign_ecdsa(&blck_msg, &blck_kp.0);
            // verification of data signature to ensure chain validity
            let blck_ver = blck_fn.verify_ecdsa(&blck_msg, &blck_sig, &blck_kp.1);
            // ensure verification passes and then merge block
            // to the existing chain otherwise truncate user
            if blck_ver.is_ok() {
                let mut chain = Vec::new();
                chain.push(block);
                // we're using system time based on locale
                let bchain = Blockchain {
                    chain,
                    timestamp: SystemTime::now(),
                };
                // save this data to the local blockchain
                // database stored in user's system
                let storechain = std::fs::File::options().append(true).open("blockchain.json");
                match storechain {
                    core::result::Result::Ok(mut sc) => {
                        let serialise_chain = serde_json::to_string(&bchain);
                        match serialise_chain {
                            core::result::Result::Ok(scx) => {
                                let _ = sc.write_all(scx.as_bytes());
                                let x = std::fs::File::options().read(true).open("blockchain.json").unwrap();
                                dbg!("{:?}", x);
                            },
                            Err(scxe) => {
                                println!("cannot add data to local chain data = {}", scxe);
                            }
                        }
                    },
                    Err(sce) => {
                        println!("issue opening & appending to file = {}", sce);
                    }
                }
                return bchain;
            }
        }
        // at this point block is not valid
        // return or block & truncate user
        // but also return an empty chain
        Blockchain::default()
    }

    pub fn add_block(block: Block8) -> Result<Blockchain> {
        let mut blocks: Vec<Block8> = Vec::new();
        let block_valid = block.valid.clone();
        if block_valid {
            let block_data = block.block_data.get(0).unwrap().as_str().as_bytes();

            dbg!("block data", &block.block_data);
            let block_bytes = String::from_utf8(Vec::from(block_data))?;
            let mut rng = rand::thread_rng();
            let bits = 2048;
            let secret = RsaPrivateKey::new(&mut rng, bits)?;
            let public = RsaPublicKey::from(&secret);
            let enc_block = public.encrypt(&mut rng, Pkcs1v15Encrypt, block_bytes.as_bytes())?;
            let dec_block = secret.decrypt(Pkcs1v15Encrypt, &enc_block)?;
            assert_eq!(&block_data[..], &dec_block[..]);
            blocks.push(block.clone());
            let new_blockchain = Blockchain {
                chain: blocks,
                timestamp: SystemTime::now(),
            };
            return Ok(new_blockchain);
        }
        Ok(Blockchain::default())
    }
}
