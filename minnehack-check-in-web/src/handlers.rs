use std::io::Read as IoRead;
use std::net::SocketAddr;

use iron::{IronResult, Plugin, Request, Response};
use iron::method::Method;
use iron::status;
use minnehack_check_in::p2p::PeerState;
use minnehack_check_in::util::vec_to_arrayvec;
use persistent::Read;

use util::{json_response, Client};

pub fn mine(req: &mut Request) -> IronResult<Response> {
    if req.method != Method::Post {
        return Ok(Response::with(status::NotFound));
    }

    let mut buf = Vec::new();
    itry!(req.body.read_to_end(&mut buf));
    if buf.len() >= 256 {
        return Ok(Response::with(status::BadRequest));
    }

    let client = itry!(req.get::<Read<Client>>());
    let data = iexpect!(vec_to_arrayvec(buf));
    json_response(status::Ok, client.mine(data))
}

pub fn status(req: &mut Request) -> IronResult<Response> {
    #[derive(Debug, Serialize)]
    struct Status<'a> {
        chain: Vec<&'a [u8]>,
        tip_index: u64,
        peers: Vec<(SocketAddr, &'static str, usize, Option<u64>)>,
    }

    if req.method != Method::Get {
        return Ok(Response::with(status::NotFound));
    }

    let client = itry!(req.get::<Read<Client>>());
    client.with_chain(|chain| {
        client.with_peers(|peers| {
            let peers = peers
                .values()
                .map(|peer| {
                    let (state, tip_index) = match peer.state {
                        PeerState::Speculative => ("speculative", None),
                        PeerState::Existent => ("existent", None),
                        PeerState::Confirmed(n, _) => ("confirmed", Some(n)),
                        PeerState::Ignore => ("ignore", None),
                    };
                    (peer.addr, state, peer.karma, tip_index)
                })
                .collect();
            let chain_data = chain
                .into_iter()
                .map(|block| block.data.as_slice())
                .collect();

            json_response(
                status::Ok,
                Status {
                    chain: chain_data,
                    tip_index: chain.tip().index,
                    peers,
                },
            )
        })
    })
}
