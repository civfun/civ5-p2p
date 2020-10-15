async function main() {
    const Libp2p = require('libp2p')
    const Tcp = require('libp2p-tcp')
    const {NOISE} = require('libp2p-noise')
    const MPLEX = require('libp2p-mplex')
    const Bootstrap = require('libp2p-bootstrap')

// Known peers addresses
    const bootstrapMultiaddrs = [
        '/dns4/ams-1.bootstrap.libp2p.io/tcp/443/wss/p2p/QmSoLer265NRgSp2LA3dPaeykiS1J6DifTC88f5uVQKNAd',
        '/dns4/lon-1.bootstrap.libp2p.io/tcp/443/wss/p2p/QmSoLMeWqB7YGVLJN3pNLQpmmEk35v6wYtsMGLzSr5QBU3'
    ]

    const node = await Libp2p.create({
        modules: {
            transport: [Tcp],
            connEncryption: [NOISE],
            streamMuxer: [MPLEX],
            peerDiscovery: [Bootstrap]
        },
        config: {
            peerDiscovery: {
                autoDial: true, // Auto connect to discovered peers (limited by ConnectionManager minConnections)
                // The `tag` property will be searched when creating the instance of your Peer Discovery service.
                // The associated object, will be passed to the service when it is instantiated.
                [Bootstrap.tag]: {
                    enabled: true,
                    list: bootstrapMultiaddrs // provide array of multiaddrs
                }
            }
        }
    })

    node.on('peer:discovery', (peer: any) => {
        console.log('Discovered %s', peer.id.toB58String()) // Log discovered peer
    })

    node.connectionManager.on('peer:connect', (connection: any) => {
        console.log('Connected to %s', connection.remotePeer.toB58String()) // Log connected peer
    })

// start libp2p
    await node.start()
}

main()
    .then(text => {
        console.log(text);
    })
    // .catch(err => {});
