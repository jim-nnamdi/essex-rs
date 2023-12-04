use std::time::{Duration, SystemTime};

use anyhow::{Ok, Result};

trait _BlockT {
    fn new() -> Self;
    fn validate_block(&mut self, prevblock: Block) -> Result<bool>;
}

pub struct Block {
    pub block_hash: &'static [u8],
    pub prev_hash: &'static [u8],
    pub validator: &'static str,
    pub signature: &'static [u8],
    pub block_data: Vec<String>,
    pub valid: bool,
    pub timestamp: SystemTime,
}

impl Default for Block {
    fn default() -> Self {
        let sec8ref = "genesis".as_bytes();
        Block {
            block_hash: sec8ref,
            prev_hash: sec8ref,
            block_data: Vec::new(),
            validator: "genesis",
            signature: sec8ref,
            valid: true,
            timestamp: SystemTime::now(),
        }
    }
}

impl _BlockT for Block {
    fn new() -> Self {
        Block::default()
    }
    fn validate_block(&mut self, prevblock: Block) -> Result<bool> {
        if prevblock.valid {
            if prevblock.block_hash.eq(self.prev_hash) {
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
                            if bloc_data.as_str().as_bytes().eq(self.block_hash) {
                                log::info!("block {} validated", bloc_data);
                                return Ok(true);
                            }
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
