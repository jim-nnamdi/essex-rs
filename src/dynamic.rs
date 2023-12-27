use futures::stream::StreamExt;
use libp2p::{gossipsub, mdns, noise, swarm::NetworkBehaviour, swarm::SwarmEvent, tcp, yamux};
use libp2p::{Multiaddr, PeerId, Swarm};
use log::info;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::io::{self, AsyncBufReadExt};
use tokio::select;
use tracing_subscriber::EnvFilter;

use crate::account::account;
use crate::block::block::{self, _BlockT};
use crate::blockchain::blockchain;

#[derive(NetworkBehaviour)]
pub struct EssexBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

pub fn block_handler() {
    let genesis = block::Block::new();
    let account = account::Account::create("hello").unwrap();
    let cb = <block::Block as _BlockT>::create_essex_block(genesis, account, "hello").unwrap();
    let bk = blockchain::Blockchain::_add_block_to_chain(cb.clone());
    if bk.chain.len() > 0 {
        // this calls the latest block hash
        // on the first block on the chain
        println!(
            "🌈 Latest known block: {}",
            bk.chain.get(0).unwrap().block_hash
        );
    }
}

pub fn _get_nodes(swarm: &Swarm<EssexBehaviour>) -> Vec<String> {
    let nodes = swarm.behaviour().mdns.discovered_nodes();
    let mut unique_nodes = HashSet::new();
    for leafs in nodes {
        unique_nodes.insert(leafs);
    }
    unique_nodes.iter().map(|x| x.to_string()).collect()
}

pub fn _print_nodes(swarm: &Swarm<EssexBehaviour>) {
    let nodes = _get_nodes(swarm);
    nodes.iter().for_each(|x| info!("{}\n", x))
}

pub fn _blacklist_invalid_tx(leafval: i32, node: &PeerId, swarm: &mut Swarm<EssexBehaviour>) {
    let min_val = 5;
    if leafval < min_val {
        info!("blacklisted: {}\n", node);
        swarm.behaviour_mut().gossipsub.blacklist_peer(node);
    }
}

#[tokio::main]
pub async fn _essex_sim<'a>(
    enode_topic: &'a str,
    enode_addr: Multiaddr,
    enode_addr_2: Multiaddr,
) -> Result<(), Box<dyn std::error::Error>> {
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
            Ok(EssexBehaviour { gossipsub, mdns })
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();
    let gtopic = gossipsub::IdentTopic::new(enode_topic);
    let _ = swarm.behaviour_mut().gossipsub.subscribe(&gtopic);
    let mut stdin = io::BufReader::new(io::stdin()).lines();
    swarm.listen_on(enode_addr)?;
    swarm.listen_on(enode_addr_2)?;
    println!("any messages sent would be sent to peers");

    loop {
        select! {
            Ok(Some(line)) = stdin.next_line() => {
                // impl Topic<IdHash> && impl Into<Vec<u8>>
                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(gtopic.clone(),line.as_bytes()){
                    println!("{e}")
                }
            }
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(EssexBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for(peer_id, _multiaddr) in list {
                        println!("discovered peer {peer_id}");
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(EssexBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for(peer_id, _multiaddr) in list {
                        println!("mdns discover expired: {peer_id}");
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(EssexBehaviourEvent::Gossipsub(gossipsub::Event::Message{
                    propagation_source:peer_id,
                    message_id:id,
                    message
                })) => {
                    if message.data == "createchain".as_bytes() {
                        // we assume createchain is a command
                        // to simulate & prepare a new chain
                        // in the blockchain : _ts_demo()-x
                        block_handler();
                    }
                    println!("Got message {:?} with id: {id} from peer: {peer_id}", String::from_utf8_lossy(&message.data))
                },
                SwarmEvent::NewListenAddr {address, ..} => {
                    println!("🆚 Chain Verx: {}", "v1.0.0");
                    println!("👨🏾‍💻 Chain Devx: {}", "Jim Nnamdi");
                    println!("🚀 Chain Specs: {}", "random specs");
                    println!("🧰 Chain Role: {}", "authority");
                    println!("🛢 Chain DBX: {}", "/local/db/essex.db");
                    println!("🎱 Operating system: {}", "MacOS m1");
                    println!("🧶 Architecture: {}", "amd 64 intel");
                    println!("🌈 Node Listener: {}", address);
                }
                _ => {},
            }
        }
    }
}
