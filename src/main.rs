/*
 * `hey,`
 *
 * @description: A thermodynamically complete universal data layer, communicating entropical deltas.
 * @author:
 *     George Phillips <george.phillips@nanoly.cloud>
 *
 * Copyright (C) 2025
 * GNU Affero General Public License v3 or later
 */

use bitvec::prelude::*;
use std::{
    collections::HashSet,
    io::{self, BufRead},
    net::{SocketAddr, UdpSocket},
    sync::mpsc,
    thread,
    time::Duration,
};

mod node;
use node::{BitVal, Bits, Node, ROOT, SIZE};

/// Map a Node to a UDP port.
pub fn to_port(node: &Node) -> u16 {
    let ttl: usize = 65535 - SIZE;

    let bits: Bits = node.clone().into();
    let port = (bits.len() % ttl) + SIZE;
    let port_u16 = port as u16;

    println!("[PORT] Using UDP port {}", port_u16);
    port_u16
}

/// Try to bind a UDP socket based on the current node state.
pub fn bind(node: &Node) -> Result<(u16, UdpSocket), io::Error> {
    let port = to_port(node);
    let addr = format!("0.0.0.0:{}", port);

    let socket = match UdpSocket::bind(&addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[UDP ERROR] Failed to bind to {}: {}", addr, e);
            return Err(e);
        }
    };

    socket.set_broadcast(true)?;
    println!("[UDP] Bound UDP socket to {}", addr);
    Ok((port, socket))
}

/// Once bound, sit and receive, perform minimal handshake based on ROOT,
/// and also read from stdin, folding input bytes into the node state and
/// sending them to all known peers.
fn begin(mut socket: UdpSocket, port: u16, node: &mut Node) -> io::Result<()> {
    println!(
        "[MESH] Bound successfully on port {} with node state: {:?}",
        port, node
    );

    socket.set_nonblocking(true)?;

    // Compute the canonical "root" port from ROOT state.
    let root_node = Node::from(BitVec::from_slice(&ROOT));
    let root_port = to_port(&root_node);

    // Known peers (can be many).
    let mut peers: HashSet<SocketAddr> = HashSet::new();

    // If we are NOT the root node, announce ourselves to the root port.
    if port != root_port {
        let bits: Bits = node.clone().into();
        let buf = bits.into_vec();
        let target = format!("127.0.0.1:{}", root_port);
        println!("[HANDSHAKE] Announcing to {}", target);
        let _ = socket.send_to(&buf, &target)?;
    } else {
        println!(
            "[HANDSHAKE] This node is the ROOT node (port {}).",
            root_port
        );
    }

    // === stdin reader thread ===
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    thread::spawn(move || {
        println!("[STDIN] stdin thread started; type and press ENTER.");
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        let mut line = String::new();

        loop {
            line.clear();
            match handle.read_line(&mut line) {
                Ok(0) => {
                    println!("[STDIN] EOF, exiting stdin thread.");
                    break;
                }
                Ok(_) => {
                    let data = line.as_bytes().to_vec();
                    if tx.send(data).is_err() {
                        println!("[STDIN] main loop gone, exiting stdin thread.");
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("[STDIN ERROR] {}", e);
                    break;
                }
            }
        }
    });

    let mut buf = [0u8; SIZE];

    loop {
        // === 1. Network side: UDP receive / handshake / chat ===
        match socket.recv_from(&mut buf) {
            Ok((n, src)) => {
                // HELLO is our explicit handshake ack
                if n == 5 && &buf[..n] == b"HELLO" {
                    println!("[HANDSHAKE] Received HELLO from {}", src);
                    if peers.insert(src) {
                        println!("[HANDSHAKE] Added new peer {}", src);
                    }
                    continue;
                }

                let payload = &buf[..n];
                println!(
                    "[NET] Received {} bytes from {}: {:?}",
                    n,
                    src,
                    String::from_utf8_lossy(payload)
                );

                // Track every sender as a peer.
                if peers.insert(src) {
                    println!("[HANDSHAKE] Learned new peer addr = {}", src);
                }

                // Fold payload into Node state (optional, but matches your model).
                let peer_node = Node::from(BitVec::from_slice(payload));
                *node = node.next(peer_node)?;
                println!("[MESH] Updated node state from peer: {:?}", node);

                // If we are the ROOT node and this looks like an announcement,
                // respond with HELLO so the sender learns us as a peer.
                if port == root_port {
                    println!("[HANDSHAKE] (ROOT) Sending HELLO to {}", src);
                    let _ = socket.send_to(b"HELLO", src)?;
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // No UDP this tick — that’s fine.
            }
            Err(e) => {
                eprintln!("[MESH ERROR] UDP receive error: {}", e);
                return Err(e);
            }
        }

        // === 2. Local side: stdin input → node state → send to all peers ===
        match rx.try_recv() {
            Ok(data) => {
                println!(
                    "[STDIN] Got {} bytes: {:?}",
                    data.len(),
                    String::from_utf8_lossy(&data)
                );

                // Fold stdin bytes into the evolving entropical state.
                let input_bits: Bits = BitVec::from_slice(&data);
                let input_node: Node = Node::from(input_bits);
                *node = node.next(input_node)?;
                println!("[MESH] Updated node state from stdin: {:?}", node);

                if peers.is_empty() {
                    println!("[CHAT] No peers known yet; not sending.");
                } else {
                    for peer in &peers {
                        println!("[CHAT] Sending {} bytes to {}", data.len(), peer);
                        let _ = socket.send_to(&data, peer)?;
                    }
                }
            }
            Err(mpsc::TryRecvError::Empty) => {
                // no stdin this tick
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                // stdin thread ended; just keep serving network
            }
        }

        // Small sleep so we don't busy-spin.
        thread::sleep(Duration::from_millis(20));
    }
}

/// `hey,` entry point.
/// Self discovering UDP network, communicating an evolving Node state
/// Note: how ROOT = 'hey'
fn main() -> io::Result<()> {
    println!("=== {} - (AGPLv3) ===\n", String::from_utf8_lossy(&ROOT));

    // Initial entropical state from ROOT.
    let mut node = Node::from(BitVec::from_slice(&ROOT));

    loop {
        // Attempt to bind using current node state.
        match bind(&node) {
            Ok((port, socket)) => {
                // Success: we have found this node’s place in the local mesh.
                // From here on, we just participate; main never returns.
                return begin(socket, port, &mut node);
            }
            Err(ref e) if e.kind() == io::ErrorKind::AddrInUse => {
                println!("[MESH] Port in use – encoding failure bit and hopping…");
                let bit = Node::Bit(BitVal::One);
                node = node.next(bit)?;
            }
            Err(e) => {
                // Any other error: also evolve and keep going.
                eprintln!("[MESH ERROR] {}", e);
                node = node.next(Node::Bit(BitVal::One))?;
            }
        }
    }
}
