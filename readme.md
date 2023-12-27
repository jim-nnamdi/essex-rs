# Essex Chain
Essex chain is a simple proof of concept blockchain built in Rust specifically for research purposes. it implements block creation and validation and addition of blocks to the chain which is dependent on the current state of the chain. Apparently since a blockchain is a state machine which stays in its current state except acted on say by means on adding new data which is made possible after validation and consensus. Essex tends to incorporate these mechanism into this research work.

# Block Nodes
When the Blockchain is started, the multi node discovery service attributes open and available ports which are TCP based to a particular Node or Peer and then keeps lurking around until it finds a peer and then can start chain communication. in event of no peers (or a single Node or peer in the chain) the swarm behaviour would throw a message of insufficient peers.

```shell 
🆚 Chain Verx: v1.0.0
👨🏾‍💻 Chain Devx: Jim Nnamdi
🚀 Chain Specs: random specs
🧰 Chain Role: authority
🛢 Chain DBX: /local/db/essex.db
🎱 Operating system: MacOS m1
🧶 Architecture: amd 64 intel
🌈 Node Listener: /ip4/169.254.28.84/udp/52993/quic-v1
discovered peer 12D3KooWFaByC5sCBw6t3GfEo1NGvEb2xBdLFEiwnzEx3wCkAHHU
discovered peer 12D3KooWFaByC5sCBw6t3GfEo1NGvEb2xBdLFEiwnzEx3wCkAHHU
discovered peer 12D3KooWFaByC5sCBw6t3GfEo1NGvEb2xBdLFEiwnzEx3wCkAHHU
discovered peer 12D3KooWFaByC5sCBw6t3GfEo1NGvEb2xBdLFEiwnzEx3wCkAHHU
```

The block returns the chain version and developer and the specifications and the local DB location and Operating system architecture with the Node listener data and discovered peers.

# Account Twist Fix
(wip: might remove)
Account chain Linking: still working on proof of concept
```shell 

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
```

# Essex Features
- [x] Block generation
- [x] Chain generation
- [x] Account implementation
- [x] Transaction implementation
- [x] Secure cryptographic Algorithms
- [x] P2P Networking & Discovery

# Essex Todo
- [x] Linking Accounts with Blocks
- [x] Linking Transactions & Accounts
- [x] Linking Accounts and Transactions
- [x] Publish transactions on Local Explorer