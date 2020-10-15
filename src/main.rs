use futures::{future, prelude::*};
use std::{task::{Context, Poll}, env};
use std::time::Duration;
use libp2p::ping::{Ping, PingEvent, PingSuccess, PingFailure, PingConfig};
use libp2p::kad::{Kademlia, KademliaEvent};
use libp2p::kad::store::MemoryStore;
use libp2p::swarm::NetworkBehaviourEventProcess;
use libp2p::identify::IdentifyEvent;
use libp2p::identity::Keypair;
use libp2p::{floodsub, Transport, tcp, dns, websocket, noise, yamux, mplex, NetworkBehaviour, Swarm, identity};
use anyhow::Result;
use tokio::{main, runtime};
use libp2p::tcp::TokioTcpConfig;
use libp2p::core::PeerId;

#[derive(NetworkBehaviour)]
struct MyBehaviour {
    // gossipsub: Gossipsub,
    // identify: Identify,
    ping: Ping,
    kademlia: Kademlia<MemoryStore>,
}

impl NetworkBehaviourEventProcess<KademliaEvent> for MyBehaviour {
    fn inject_event(&mut self, event: KademliaEvent) {
        dbg!(&event);
    }
}

impl NetworkBehaviourEventProcess<IdentifyEvent> for MyBehaviour {
    // Called when `identify` produces an event.
    fn inject_event(&mut self, event: IdentifyEvent) {
        println!("identify: {:?}", event);
    }
}

impl NetworkBehaviourEventProcess<PingEvent> for MyBehaviour {
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

#[tokio::main]
async fn main() -> Result<()> {
    // Create a random PeerId
    let keypair = Keypair::generate_ed25519();
    let peer_id = PeerId::from(keypair.public());
    println!("Local peer id: {:?}", peer_id);

    // let secret = match &local_key {
    //     Keypair::Ed25519(ed) => { ed.encode() }
    //     _ => panic!("unknown key type")
    // };
    // dbg!(&secret);

    // let transport = build_transport()?;
    let transport = libp2p::build_development_transport(keypair.clone())?;

    let store = MemoryStore::new(peer_id.clone());
    // let mut behaviour = Kademlia::with_config(peer_id.clone(), store, cfg);

    let mut behaviour = MyBehaviour {
        // gossipsub: Gossipsub::new(MessageAuthenticity::Signed(local_key.clone()), gossipsub_config),
        // identify: Identify::new(
        //     "/ipfs/0.1.0".into(),
        //     "rust-ipfs-example".into(),
        //     id_keys.public(),
        // ),
        ping: Ping::new(PingConfig::new()),
        kademlia: Kademlia::new(peer_id.clone(), store),
    };

    // Create a Swarm that establishes connections through the given transport
    // and applies the ping behaviour on each connection.
    let mut swarm = Swarm::new(transport, behaviour, peer_id);

    // Order Kademlia to search for a peer.
    let to_search: PeerId = if let Some(peer_id) = env::args().nth(1) {
        peer_id.parse()?
    } else {
        identity::Keypair::generate_ed25519().public().into()
    };

    println!("Searching for the closest peers to {:?}", to_search);
    // swarm.get_closest_peers(to_search);
    // behaviour.kademlia.get_closest_peers(to_search);
    swarm.kademlia.get_closest_peers(to_search);

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
    future::poll_fn(move |cx: &mut Context<'_>| {
        loop {
            match swarm.poll_next_unpin(cx) {
                Poll::Ready(Some(event)) => println!("event {:?}", event),
                Poll::Ready(None) => return Poll::Ready(()),
                Poll::Pending => {
                    if !listening {
                        for addr in Swarm::listeners(&swarm) {
                            println!("Listening on {}", addr);
                            listening = true;
                        }
                    }
                    return Poll::Pending;
                }
            }
        }
    }).await;

    Ok(())
}

// fn build_transport() -> Result<Box<dyn Transport>> {
//     let transport = {
//         let tcp = TokioTcpConfig::new().nodelay(true);
//         Ok(dns::DnsConfig::new(tcp)?)
//     };
//
//     let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
//         .into_authentic(&keypair)
//         .expect("Signing libp2p-noise static DH keypair failed.");
//
//     Ok(Box::new(transport
//         .upgrade(core::upgrade::Version::V1)
//         .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
//         .multiplex(core::upgrade::SelectUpgrade::new(yamux::Config::default(), mplex::MplexConfig::new()))
//         .map(|(peer, muxer), _| (peer, core::muxing::StreamMuxerBox::new(muxer)))
//         .timeout(std::time::Duration::from_secs(20))))
// }