use async_std::task;
use futures::{future, prelude::*};
use std::{task::{Context, Poll}, env};
use std::time::Duration;
use libp2p::ping::{Ping, PingEvent, PingSuccess, PingFailure, PingConfig};
use libp2p::kad::{Kademlia, KademliaEvent};
use libp2p::kad::store::MemoryStore;
use libp2p::swarm::NetworkBehaviourEventProcess;
use libp2p::identify::{IdentifyEvent, Identify};
use libp2p::identity::Keypair;
use libp2p::{floodsub, Transport, tcp, dns, websocket, noise, yamux, mplex, NetworkBehaviour, Swarm, identity};
use anyhow::Result;
use libp2p::core::{PeerId, Multiaddr};
use std::str::FromStr;
use futures::channel::mpsc;

#[derive(NetworkBehaviour)]
struct Behaviour {
    // gossipsub: Gossipsub,
    identify: Identify,
    ping: Ping,
    kademlia: Kademlia<MemoryStore>,
}

impl NetworkBehaviourEventProcess<KademliaEvent> for Behaviour {
    fn inject_event(&mut self, event: KademliaEvent) {
        println!("kad: {:?}", &event);
    }
}

impl NetworkBehaviourEventProcess<IdentifyEvent> for Behaviour {
    // Called when `identify` produces an event.
    fn inject_event(&mut self, event: IdentifyEvent) {
        println!("identify: {:?}", event);
        match event {
            IdentifyEvent::Received { peer_id, info, .. } => {
                for addr in info.listen_addrs {
                    self.kademlia.add_address(&peer_id, addr);
                }
            }
            _ => {}
        }
    }
}

impl NetworkBehaviourEventProcess<PingEvent> for Behaviour {
    // Called when `ping` produces an event.
    fn inject_event(&mut self, event: PingEvent) {
        match event {
            PingEvent {
                peer,
                result: Ok(PingSuccess::Ping { rtt }),
            } => {
                println!(
                    "ping: rtt to {} is {} ms",
                    peer.to_base58(),
                    rtt.as_millis()
                );
            }
            PingEvent {
                peer,
                result: Ok(PingSuccess::Pong),
            } => {
                println!("ping: pong from {}", peer.to_base58());
            }
            PingEvent {
                peer,
                result: Err(PingFailure::Timeout),
            } => {
                println!("ping: timeout to {}", peer.to_base58());
            }
            PingEvent {
                peer,
                result: Err(PingFailure::Other { error }),
            } => {
                println!("ping: failure with {}: {}", peer.to_base58(), error);
            }
        }
    }
}

pub enum Action {}

pub enum Event {}

pub struct Civ5p2p {
    keypair: Keypair,
}

impl Civ5p2p {
    pub fn new(keypair: Keypair) -> Self {
        Self { keypair }
    }

    pub fn new_keypair() -> Keypair {
        Keypair::generate_ed25519()
    }

    pub async fn run(&self) -> Result<(mpsc::UnboundedSender<Action>, mpsc::UnboundedReceiver<Event>)> {
        let (actions_tx, actions_rx) = mpsc::unbounded::<Action>();
        let (events_tx, events_rx) = mpsc::unbounded::<Event>();

        let peer_id = PeerId::from(self.keypair.public());
        println!("Local peer id: {:?}", peer_id);

        // let secret = match &local_key {
        //     Keypair::Ed25519(ed) => { ed.encode() }
        //     _ => panic!("unknown key type")
        // };
        // dbg!(&secret);

        let transport = libp2p::build_development_transport(self.keypair.clone())?;

        let store = MemoryStore::new(peer_id.clone());
        // let mut behaviour = Kademlia::with_config(peer_id.clone(), store, cfg);

        let mut behaviour = Behaviour {
            // gossipsub: Gossipsub::new(MessageAuthenticity::Signed(local_key.clone()), gossipsub_config),
            identify: Identify::new(
                "/civ5/0.1.0".into(),
                "civ5".into(),
                self.keypair.public(),
            ),
            ping: Ping::new(PingConfig::new()),
            kademlia: Kademlia::new(peer_id.clone(), store),
        };

        // Create a Swarm that establishes connections through the given transport
        // and applies the ping behaviour on each connection.
        let mut swarm = Swarm::new(transport, behaviour, peer_id.clone());

        if let Some(bootstrap_addr) = env::args().nth(2) {
            let bootstrap_peer_id = env::args().nth(1).unwrap();
            let bootstrap_peer_id = PeerId::from_str(&bootstrap_peer_id)?;
            let bootstrap_addr: Multiaddr = bootstrap_addr.parse()?;
            swarm.kademlia.add_address(&bootstrap_peer_id, bootstrap_addr);
            // println!("Bootstrapping node to join DHT");
            swarm.kademlia.bootstrap()?;
        };

        // Order Kademlia to search for a peer.
        if let Some(peer_id) = env::args().nth(3) {
            let peer_id: PeerId = peer_id.parse()?;
            swarm.kademlia.get_closest_peers(peer_id);
        };

        // Dial the peer identified by the multi-address given as the second
        // command-line argument, if any.
        // if let Some(addr) = std::env::args().nth(1) {
        //     let remote = addr.parse()?;
        //     Swarm::dial_addr(&mut swarm, remote)?;
        //     println!("Dialed {}", addr)
        // }

        // Tell the swarm to listen on all interfaces and a random, OS-assigned port.
        Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/0".parse()?)?;

        let mut listening = false;
        task::spawn(future::poll_fn(move |cx: &mut Context<'_>| {
            loop {
                match swarm.poll_next_unpin(cx) {
                    Poll::Ready(Some(event)) => println!("{:?}", event),
                    Poll::Ready(None) => return Poll::Ready(()),
                    Poll::Pending => {
                        if !listening {
                            for addr in Swarm::listeners(&swarm) {
                                println!("Listening on {} {}", peer_id, addr);
                                listening = true;
                            }
                        }
                        return Poll::Pending;
                    }
                }
            }
        }));

        Ok((actions_tx, events_rx))
    }
}
