use std::{
    io::Write,
    mem,
    time::{Duration, SystemTime},
};

use anyhow::{Ok, Result};

use secp256k1::hashes::sha256;
use secp256k1::{Message, Secp256k1};
use serde::{Deserialize, Serialize};

use crate::{
    account::Account,
    sec8::{self, sec8_block_id_hash},
};
const VMAX: u32 = 30;
pub trait _BlockT {
    fn new() -> Self;
    fn create_essex_block(block: Block, acc: Account, msg: &str) -> Result<Block>;
    fn validate_block(prevblock: Block) -> Result<bool>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub block_hash: String,
    pub prev_hash: String,
    pub validator: String,
    pub signature: String,
    pub block_data: Vec<String>,
    pub valid: bool,
    pub timestamp: SystemTime,
}

impl Default for Block {
    fn default() -> Self {
        let sec8_ks = sec8_block_id_hash().unwrap();
        let sec8_mess = Message::from_hashed_data::<sha256::Hash>("".as_bytes());
        let sec8_sg = secp256k1::Secp256k1::sign_ecdsa(&Secp256k1::new(), &sec8_mess, &sec8_ks.0);
        Block {
            block_hash: sec8_ks.0.display_secret().to_string(),
            prev_hash: sec8_ks.0.display_secret().to_string(),
            block_data: Vec::new(),
            validator: sec8_ks.1.to_string(),
            signature: sec8_sg.to_string(),
            valid: false,
            timestamp: SystemTime::now(),
        }
        .to_owned()
    }
}

impl _BlockT for Block {
    fn new() -> Self {
        Block::default()
    }

    // pass same message used in acc creation ...
    fn create_essex_block(mut block: Block, acc: Account, msg: &str) -> Result<Block> {
        // let's assume the genesis block was passed in the fn
        let check_validated = Self::validate_block(mem::take(&mut block)).is_ok();
        // by default the genesis block should pass this logic
        if !check_validated {
            log::error!("invalid previous block");
        }
        // secp256k1 algorithm is to be used for signatures
        let scp = secp256k1::Secp256k1::new();
        // msg is same as the one user had during acc creation
        let mex = Message::from_hashed_data::<sha256::Hash>(msg.as_bytes());
        let ubal = acc.acc_balance;
        // 0x1E min val a validator should have to create block
        let vok = ubal > VMAX;
        // validator loses coins because of illegal tx
        if !vok {
            log::error!("insufficient balance");
        }
        let essex_secret = sec8::sec8_block_id_hash()
            .unwrap()
            .0
            .display_secret()
            .to_string();
        let block = Block {
            block_hash: essex_secret,
            prev_hash: block.block_hash,
            validator: acc.acc_public.to_string(),
            signature: acc.acc_signed.to_string(),
            block_data: vec![],
            valid: true,
            timestamp: SystemTime::now(),
        };
        let data_store = std::fs::File::options().append(true).open("block.txt");
        match data_store {
            core::result::Result::Ok(mut data) => {
                let serialise_block = serde_json::to_string(&block);
                match serialise_block {
                    core::result::Result::Ok(resb) => {
                        data.write_all(resb.as_bytes())?;
                        return Ok(block);
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
            }
            Err(e) => {
                println!("could not save data = {}", e);
                return Ok(block.clone());
            }
        }
        // verification on the validator data
        scp.verify_ecdsa(&mex, &acc.acc_signed, &acc.acc_public)
            .unwrap();
        Ok(block)
    }

    fn validate_block(prevblock: Block) -> Result<bool> {
        let prev_valid = prevblock.valid;
        if prev_valid {
            // if previous block is valid get creation time
            let block_creation_time = prevblock.timestamp;
            // check_add to see that block is > 0x02 hours
            let ox02 = SystemTime::now()
                .checked_add(Duration::from_secs(7200))
                .ok_or("invalid time")
                .unwrap();
            // fetch the duration since the block creation time
            let block_time_diff = ox02.duration_since(block_creation_time)?;
            // if the block difference is < 0x02 hours validate!
            if block_time_diff < Duration::from_secs(7200) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
