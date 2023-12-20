use anyhow::{Error, Ok, Result};
use rand::rngs::OsRng;
use secp256k1::{ecdsa::Signature, hashes::sha256, Message, PublicKey, SecretKey};

#[derive(Debug)]
pub struct Account {
    pub acc_private: SecretKey,
    pub acc_public: PublicKey,
    pub acc_signed: Signature,
    pub acc_balance: u32,
}

#[derive(Debug, Clone)]
pub struct ANode {
    pub values: Vec<i32>,
    pub childs: Vec<ANode>,
}

#[derive(Debug, Clone)]
pub struct LNode {
    pub dx: i32,
    pub dy: Box<Option<LNode>>,
}

#[derive(Debug, Clone)]
pub struct LNodeLeafs {
    pub head: Box<Option<LNode>>,
    pub tail: Box<Option<LNode>>,
}

pub struct LNodeLeafsIterator<'a> {
    pub head_leaf_iter: Box<dyn Iterator<Item = &'a LNode> + 'a>,
    pub tail_leaf_iter: Box<dyn Iterator<Item = &'a LNode> + 'a>,
}

impl<'a> Iterator for LNodeLeafsIterator<'a> {
    type Item = &'a LNode;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(x) = self.head_leaf_iter.next() {
            Some(x)
        } else {
            if let Some(y) = self.tail_leaf_iter.next() {
                self.head_leaf_iter = Box::new(y.dy.iter());
                self.next()
            } else {
                None
            }
        }
    }
}

impl LNode {
    pub fn zero_check<T: PartialOrd + From<i32>>(&self, v: T) -> bool {
        let data = T::from(0);
        if v == data {
            return true;
        }
        false
    }
    pub fn new(&self) -> LNode {
        LNode {
            dx: 0,
            dy: Box::new(None),
        }
    }
    pub fn new_lnode(&self, data: i32, mut nl: LNodeLeafs) -> Result<LNode> {
        let nlh = nl.head.clone().unwrap().dx;
        let nln = nl.head.clone().unwrap().dy;
        if nlh == 0 && nln.is_none() {
            nl.head.clone().unwrap().dx = data;
            nl.head.clone().unwrap().dy = Box::new(None).into();
            return Ok(nl.head.unwrap());
        } else {
            let mut newnode = self.new();
            newnode.dx = data;
            newnode.dy = Box::new(None);
            nl.head = Box::new(Some(newnode));
            newnode = nl.head.unwrap();
            dbg!(&newnode);
            Ok(newnode.clone())
        }
    }
    pub fn lnode_walk<'a>(&'a self, z: &'a LNodeLeafs) -> Box<dyn Iterator<Item = &'a i32> + 'a> {
        Box::new(
            z.head
                .iter()
                .chain(z.tail.iter())
                .map(|f| f.lnode_walk(z))
                .flatten(),
        )
    }
}

pub struct ANodeIterator<'a> {
    pub value_iter: Box<dyn Iterator<Item = &'a i32> + 'a>,
    pub child_iter: Box<dyn Iterator<Item = &'a ANode> + 'a>,
}

impl<'a> Iterator for ANodeIterator<'a> {
    type Item = &'a i32;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(v) = self.value_iter.next() {
            Some(v)
        } else {
            if let Some(vx) = self.child_iter.next() {
                self.value_iter = Box::new(vx.values.iter());
                self.next()
            } else {
                None
            }
        }
    }
}

impl ANode {
    pub fn anode_val<'a>(&'a self) -> Box<dyn Iterator<Item = &'a i32> + 'a> {
        Box::new(
            self.values
                .iter()
                .chain(self.childs.iter().map(|f| f.anode_val()).flatten()),
        )
    }
}

impl Account {
    // user should store the msg
    // msg would be needed to create block
    pub fn create(msg: &str) -> Result<Account, Error> {
        let secp = secp256k1::Secp256k1::new();
        let (secret, public) = secp.generate_keypair(&mut OsRng);
        log::info!("secret: {:?} public: {:?}", secret, public);
        let mess = Message::from_hashed_data::<sha256::Hash>(msg.as_bytes());
        let sig = secp.sign_ecdsa(&mess, &secret);
        let new_acc = Account {
            acc_private: secret,
            acc_public: public,
            acc_signed: sig,
            acc_balance: 0,
        };
        log::info!("new-acc created: {:?}", new_acc);
        Ok(new_acc)
    }
}
