use async_std::{task};
use futures::{future, select, prelude::*};
use std::{task::{Context, Poll}, env};
use libp2p::ping::{Ping, PingEvent, PingSuccess, PingFailure, PingConfig};
use libp2p::kad::{Kademlia, KademliaEvent};
use libp2p::kad::store::MemoryStore;
use libp2p::swarm::{NetworkBehaviourEventProcess, NetworkBehaviour};
use libp2p::identify::{IdentifyEvent, Identify};
use libp2p::identity::Keypair;
use libp2p::{NetworkBehaviour, Swarm, identity};
use anyhow::Result;
use libp2p::core::{PeerId, Multiaddr};
use futures::channel::mpsc;
use libp2p::gossipsub::{Topic, Gossipsub, GossipsubEvent, GossipsubConfig, MessageAuthenticity};

#[derive(NetworkBehaviour)]
struct Behaviour {
    identify: Identify,
    ping: Ping,
    kademlia: Kademlia<MemoryStore>,
    gossipsub: Gossipsub,
}

impl NetworkBehaviourEventProcess<GossipsubEvent> for Behaviour {
    fn inject_event(&mut self, event: GossipsubEvent) {
        println!("gossip: {:?}", &event);
    }
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

pub enum Action {
    Bootstrap(PeerId, Multiaddr),
    Message(String),
}

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
        let (actions_tx, mut actions_rx) = mpsc::unbounded::<Action>();
        let (events_tx, events_rx) = mpsc::unbounded::<Event>();

        let peer_id = PeerId::from(self.keypair.public());
        println!("Local peer id: {:?}", peer_id);

        let transport = libp2p::build_development_transport(self.keypair.clone())?;

        let store = MemoryStore::new(peer_id.clone());
        // let mut behaviour = Kademlia::with_config(peer_id.clone(), store, cfg);

        let topic = Topic::new("global".into());
        let gossipsub_config = GossipsubConfig::default();
        let mut gossipsub = Gossipsub::new(MessageAuthenticity::Signed(self.keypair.clone()), gossipsub_config);
        gossipsub.subscribe(topic.clone());

        let behaviour = Behaviour {
            identify: Identify::new(
                "/civ5/0.1.0".into(),
                "civ5".into(),
                self.keypair.public(),
            ),
            ping: Ping::new(PingConfig::new()),
            kademlia: Kademlia::new(peer_id.clone(), store),
            gossipsub,
        };

        // Create a Swarm that establishes connections through the given transport
        // and applies the ping behaviour on each connection.
        let mut swarm = Swarm::new(transport, behaviour, peer_id.clone());

        // Tell the swarm to listen on all interfaces and a random, OS-assigned port.
        Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/0".parse()?)?;

        let mut listening = false;
        task::spawn(future::poll_fn(move |cx: &mut Context<'_>| {
            loop {
                match actions_rx.poll_next_unpin(cx) {
                    Poll::Ready(Some(action)) => {
                        println!("action!");
                        match action {
                            Action::Bootstrap(peer_id, addr) => {
                                swarm.kademlia.add_address(&peer_id, addr);
                                swarm.kademlia.bootstrap().unwrap();
                            }
                            Action::Message(msg) => {
                                swarm.gossipsub.publish(&topic, msg).unwrap();
                            }
                        }
                    }
                    Poll::Ready(None) => { println!("actions_rx channel closed"); }
                    Poll::Pending => break,
                };
            };
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
                        break;
                    }
                }
            }
            Poll::Pending
        }));

        Ok((actions_tx, events_rx))
    }
}
