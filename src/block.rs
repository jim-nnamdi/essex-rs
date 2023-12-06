use core::panic;
use std::{
    rc::Rc,
    time::{Duration, SystemTime}, mem
};

use anyhow::{Ok, Result};
use secp256k1::{Message, SecretKey,ecdsa::Signature, PublicKey, Secp256k1};

use crate::{account::Account, sec8::{self, sec8_block_id_hash}};
const vmax: u32 = 40;
trait _BlockT {
    fn new(&self) -> Self;
    fn create_block(&mut self, block:Block, acc:Account, msg:&str)->Result<Block>;
    fn validate_block(&mut self, prevblock: Block) -> Result<bool>;
}

#[derive(Debug, Clone)]
pub struct Block {
    pub block_hash: SecretKey,
    pub prev_hash: SecretKey,
    pub validator: PublicKey,
    pub signature: Signature,
    pub block_data: Vec<String>,
    pub valid: Rc<bool>,
    pub timestamp: SystemTime,
}

impl Default for Block {
    fn default() -> Self {
        let sec8_ks = sec8_block_id_hash().unwrap();
        let sec8_mess = Message::from_digest_slice("".as_bytes()).unwrap();
        let sec8_sg = secp256k1::Secp256k1::sign_ecdsa(&Secp256k1::new(),&sec8_mess, &sec8_ks.0);
        Block {
            block_hash: sec8_ks.0,
            prev_hash: sec8_ks.0,
            block_data: Vec::new(),
            validator: sec8_ks.1,
            signature: sec8_sg,
            valid: Rc::new(false),
            timestamp: SystemTime::now(),
        }
    }
}

impl _BlockT for Block {
    fn new(&self) -> Self {
        Block::default()
    }

    // pass same message used in acc creation ...
    fn create_block(&mut self,mut block:Block, acc:Account, msg:&str) -> Result<Block> {
        let check_validated = self.validate_block(mem::take(&mut block)).is_ok();
        if !check_validated {log::error!("invalid previous block");}
        let scp = secp256k1::Secp256k1::new();
        let mex = Message::from_digest_slice(msg.as_bytes()).unwrap();
        let ubal = acc.acc_balance;
        let vok = ubal > vmax;
        if !vok{log::error!("insufficient balance");}
        let block = Block{block_hash:sec8::sec8_block_id_hash().unwrap().0,prev_hash:block.block_hash,validator:acc.acc_public,signature:acc.acc_signed,block_data:vec![],valid:Rc::new(true),timestamp:SystemTime::now()};
        scp.verify_ecdsa(&mex, &acc.acc_signed, &acc.acc_public).unwrap();
        Ok(block)
    }

    fn validate_block(&mut self, prevblock: Block) -> Result<bool> {
        let prev_valid =
            Rc::try_unwrap(prevblock.valid).unwrap_or_else(|_| panic!("last elem was shared"));
        if prev_valid {
            if prevblock.block_hash.eq(&self.prev_hash) {
                log::info!("prev-hash = {:?}", self.prev_hash);
                let block_creation_time = self.timestamp;
                let two_hours_future = SystemTime::now()
                    .checked_add(Duration::from_secs(7200))
                    .unwrap();
                let block_time_diff = two_hours_future.duration_since(block_creation_time)?;
                if block_time_diff < Duration::from_secs(7200) {
                    let parse_bloc_data = self.block_data.get(0);
                    match parse_bloc_data {
                        Some(bloc_data) => {
                            // implement kafka sending & p2p here ...
                            log::info!("block validated {}", bloc_data);
                            return Ok(true);
                        }
                        None => {
                            log::warn!("no block validated");
                            return Ok(false);
                        }
                    }
                }
            }
        }
        Ok(false)
    }
}
