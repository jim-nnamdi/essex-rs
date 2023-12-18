use crate::block;
use anyhow::{self, Ok, Result};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use tokio::{io::{self, AsyncBufReadExt}, select};
use tracing_subscriber::EnvFilter;
use std::{mem, time::{SystemTime, Duration}, collections::hash_map::DefaultHasher};
use libp2p::{swarm::NetworkBehaviour,swarm::SwarmEvent, gossipsub, mdns, tcp, noise, yamux};
use std::hash::{Hash, Hasher};

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

#[derive(NetworkBehaviour)]
struct BlockNode {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour
}

#[tokio::main]
async fn _essex_sim() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|key| {
            let msg_fn_id = |message: &gossipsub::Message| {
                let mut s = DefaultHasher::new();
                message.data.hash(&mut s);
                gossipsub::MessageId::from(s.finish().to_string())
            };
            let goss_config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(10))
                .validation_mode(gossipsub::ValidationMode::Strict)
                .message_id_fn(msg_fn_id)
                .build()
                .map_err(|x| print!("{}", x))
                .unwrap();
            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                goss_config,
            )?;
            let mdns =
                mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())
                    .unwrap();
            Ok(BlockNode { gossipsub, mdns })
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();
    let gtopic = gossipsub::IdentTopic::new("test-net");
    let _ = swarm.behaviour_mut().gossipsub.subscribe(&gtopic);
    let mut stdin = io::BufReader::new(io::stdin()).lines();
    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    println!("any messages sent would be sent to peers");

    loop {
        select! {
            Ok(Some(line)) = stdin.next_line() => {
                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(gtopic.clone(),line.as_bytes()){
                    println!("{e}")
                }
            }
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(BlockNodeEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for(peer_id, _multiaddr) in list {
                        println!("discovered peer {peer_id}");
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(BlockNodeEvent::Mdns(mdns::Event::Expired(list))) => {
                    for(peer_id, _multiaddr) in list {
                        println!("mdns discover expired: {peer_id}");
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(BlockNodeEvent::Gossipsub(gossipsub::Event::Message{
                    propagation_source:peer_id,
                    message_id:id,
                    message
                })) => {
                    println!("Got message {} with id: {id} from peer: {peer_id}", String::from_utf8_lossy(&message.data))
                },
                SwarmEvent::NewListenAddr {address, ..} => {
                    println!("local node is listening on {address}")
                }
                _ => {},
            }
        }
    }
}