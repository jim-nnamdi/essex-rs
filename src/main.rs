use std::{
    collections::hash_map::DefaultHasher,
    error::Error,
    hash::{Hash, Hasher},
    time::Duration,
};

use futures::StreamExt;
use libp2p::{gossipsub, mdns, noise, swarm::NetworkBehaviour, swarm::SwarmEvent, tcp, yamux};
use rand::rngs::OsRng;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use secp256k1::Message;
use tokio::{
    io::{self, AsyncBufReadExt},
    select,
};
use tracing_subscriber::EnvFilter;

use crate::block::block::_BlockT;

pub mod account;
pub mod block;
pub mod blockchain;
pub mod dynamic;
pub mod sec8;
pub mod transaction;

fn _check_rsa(msg: &str) {
    let mut rng = rand::thread_rng();
    let bits = 2048;
    let private_key = RsaPrivateKey::new(&mut rng, bits).unwrap();
    let public_key = RsaPublicKey::from(&private_key);
    let encrypt_data = public_key
        .encrypt(&mut rng, Pkcs1v15Encrypt, msg.as_bytes())
        .unwrap();
    let decrypt_data = private_key.decrypt(Pkcs1v15Encrypt, &encrypt_data).unwrap();
    assert_ne!(msg.as_bytes(), encrypt_data);
    assert_eq!(msg.as_bytes(), &decrypt_data[..]);
}

fn _check_secp256k1(msg: &[u8]) {
    let secp = secp256k1::Secp256k1::new();
    let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
    println!("sec-key = {:?} pub-key = {:?}", secret_key, public_key);
    let message = Message::from_digest_slice(msg);
    match message {
        Ok(mesx) => {
            let sig = secp.sign_ecdsa(&mesx, &secret_key);
            assert!(secp.verify_ecdsa(&mesx, &sig, &public_key).is_ok());
        }
        Err(e) => {
            eprintln!("{:?}", e)
        }
    };
}

fn _tsmain() {
    let genesis = block::block::Block::new();
    let account = account::account::Account::create("hello").unwrap();
    let cb =
        <block::block::Block as _BlockT>::create_essex_block(genesis, account, "hello").unwrap();
    let bk = blockchain::blockchain::Blockchain::_add_block_to_chain(cb.clone());
    println!("{:?}", bk);
}

#[derive(NetworkBehaviour)]
pub struct EssexBlockchain {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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
            let msg_fn = |message: &gossipsub::Message| {
                let mut sd = DefaultHasher::new();
                message.data.hash(&mut sd);
                gossipsub::MessageId::from(sd.finish().to_string())
            };
            let msg_cfg = gossipsub::ConfigBuilder::default()
                .heartbeat_initial_delay(Duration::from_secs(10))
                .validation_mode(gossipsub::ValidationMode::Strict)
                .message_id_fn(msg_fn)
                .build()
                .map_err(|msg| io::Error::new(io::ErrorKind::InvalidData, msg))
                .unwrap();
            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                msg_cfg,
            )
            .unwrap();
            let mdns =
                mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())
                    .unwrap();
            Ok(EssexBlockchain { gossipsub, mdns })
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();
    let topic = gossipsub::IdentTopic::new("essex-chain");
    let _ = swarm.behaviour_mut().gossipsub.subscribe(&topic);
    let mut stdin = io::BufReader::new(io::stdin()).lines();
    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    loop {
        select! {
            Ok(Some(line)) = stdin.next_line() => {
                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), line.as_bytes()) {
                    println!("Error publishing info on running Node ... {e}");
                }
            }
            // using futures::stream::StreamExt to handle
            // the select next some for Node events
            event  = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(EssexBlockchainEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for(peer_id, _multiaddr) in list {
                        println!("discovered peer {peer_id}");
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(EssexBlockchainEvent::Mdns(mdns::Event::Expired(list))) => {
                    for(peer_id, _multiaddr) in list {
                        println!("expired peer {peer_id}");
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(EssexBlockchainEvent::Gossipsub(gossipsub::Event::Message{
                    propagation_source:peer_id,
                    message_id:id,
                    message
                })) => {
                    println!("Got message {} with id: {id} from peer: {peer_id}", String::from_utf8_lossy(&message.data))
                },
                SwarmEvent::NewListenAddr {address, ..} => {
                    println!("Node listening on address {address}")
                }
                _ => {}
            }
        }
    }
}
