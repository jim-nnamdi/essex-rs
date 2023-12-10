use anyhow::{Ok, Result};
use rsa::{RsaPrivateKey, RsaPublicKey};
use secp256k1::{PublicKey, SecretKey};
use std::time::SystemTime;

use crate::{account::account::Account, block::block::Block};

#[derive(Debug)]
pub struct TxHeader {
    pub transaction_id: RsaPrivateKey,
    pub transaction_hash: RsaPublicKey,
    pub transaction_valid: bool,
}

#[derive(Debug)]
pub struct Transaction {
    pub tx_user: Account,
    pub timestamp: SystemTime,
    pub tx_header: TxHeader,
    pub tx_block: Block,
    pub tx_amount: u32,
}

#[derive(Debug)]
pub struct TransactionPool {
    pub transactions: Vec<Transaction>,
}

impl Transaction {
    pub fn new(
        user: Account,
        user_secret: SecretKey,
        blockdata: Block,
        amount: u32,
    ) -> Result<Transaction> {
        let secp = secp256k1::Secp256k1::new();
        let user_ref = PublicKey::from_secret_key(&secp, &user_secret);
        if !user.acc_public.eq(&user_ref) || user.acc_balance.ge(&amount) {
            log::error!("invalid secret key : {:?}", user_secret);
            log::error!("insufficient balance: {:?}", user.acc_balance);
            println!("invalid secret key or insufficient balance");
        }
        let mut rng = rand::thread_rng();
        let bit_size = 512;
        let tx_id = RsaPrivateKey::new(&mut rng, bit_size).unwrap();
        let tx_hash = RsaPublicKey::from(&tx_id);
        let txheader = TxHeader {
            transaction_id: tx_id,
            transaction_hash: tx_hash,
            transaction_valid: true,
        };
        let newtx = Transaction {
            tx_user: user,
            timestamp: SystemTime::now(),
            tx_header: txheader,
            tx_block: blockdata,
            tx_amount: amount,
        };
        Ok(newtx)
    }
}

impl TransactionPool {
    pub fn fetch_txs(&self) {
        let alltx = self
            .transactions
            .iter()
            .map(|f| f.tx_block.clone())
            .collect::<Vec<_>>();
          log::info!("[txs]:{:?}",alltx);
          for tx in &self.transactions {
            println!("tx = {:?}", tx);
        }
    }
    pub fn add_to_pool(&mut self, tx:Transaction) ->Result<bool> {
       self.transactions.push(tx);
       Ok(true)
    }
}
