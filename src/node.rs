/*
 * @name: Node
 * @description: A thermodynamically complete universal data layer, communicating entropical deltas.
 * @author: George Phillips<george.phillips@nanoly.cloud>
 *
 * Copyright (C) 2025 Free Software Foundation, Inc.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 **/
use bitvec::prelude::*;
use std::io::{self};
use std::net::{SocketAddr, UdpSocket}; // <-- NEW

pub const ROOT: &[u8] = b"hey";

/// CHUNK_SIZE for reading from stdin and UDP socket.
pub const SIZE: usize = 4096;

/// A leaf encodes as an 8-byte block of all zeros or all ones.
const ZERO_BLOCK: &[u8] = &[0u8; 1];
const ONE_BLOCK: &[u8] = &[0xFFu8; 1];

/// Bit-level buffer type used throughout.
pub type Bits = BitVec<u8, Msb0>;

/// Consider it the only network literal.
#[derive(Debug, Clone, Copy)]
pub enum BitVal {
    Zero,
    One,
}

/// A recursive binary structure:
/// - Bit(Zero/One): terminal leaf
/// - Compound(left, right): internal Node
/// - A compound is : C = [[1, [...A]], [1, [1, ...B]]]
#[derive(Debug, Clone)]
pub enum Node {
    Compound(Box<(Node, Node)>),
    Bit(BitVal),
}

impl Node {
    pub fn attach(&self) -> Result<(usize, UdpSocket), io::Error> {
        let start_port = SIZE as usize;
        let end_port = 65535;
        let ttl = end_port - start_port;
        let bits = Bits::from(self.clone());

        let port = (bits.len() % ttl) + SIZE;

        let socket = match UdpSocket::bind(format!("{}:{}", "0.0.0.0", port)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[UDP ERROR] Failed to bind to port {}: {}", port, e);
                return Err(e);
            }
        };

        socket.set_broadcast(true)?;
        println!("[UDP] Bound UDP socket to port {}", port);
        return Ok((port, socket));
    }

    pub fn next(&mut self, input: Node, listen: bool) -> Node {
        // UDP socket for Reading/Writing.

        let mut buf = [0u8; SIZE];

        let node = match input {
            Node::Bit(BitVal::Zero) | Node::Bit(BitVal::One) => {
                Node::Compound(Box::new((self.to_owned(), input.to_owned())))
            }
            Node::Compound(_) => {
                let mut v = Bits::new();

                let self_bits: Bits = self.clone().into();
                let input_bits: Bits = input.into();

                // your one-bit-renormalisation step:
                v.extend_from_bitslice(&self_bits[1..]);
                v.extend_from_bitslice(&input_bits);

                Node::from(v)
            }
        };

        let (port, socket) = match self.attach() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("  [BROADCAST ERROR] Failed to bind socket: {}", e);
                panic!("Cannot proceed without UDP socket.")
            }
        };

        let self_bits: &Vec<u8> = &self.clone().into();
        let addr = format!("0.0.0.0:{}", port);

        match socket.send_to(&self_bits, &addr) {
            Ok(_) => println!("  [SEND] To {}", addr),
            Err(e) => eprintln!("  [SEND ERROR] Failed to send: {}", e),
        }

        println!("  [LISTEN] Waiting to receive on port {}", port);
        if listen {
            loop {
                match socket.recv_from(&mut buf) {
                    Ok((size, _socket)) => {
                        println!("  [RECV] From {:?}: {} bytes", &buf[..size], size);
                    }
                    Err(e) => {
                        eprintln!("  [RECV ERROR] Failed to receive: {}", e);
                    }
                };
            }
        }

        node // [node, (socket)] ???
    }

    /// Perform a bitwise XOR between two Nodes.
    pub fn op(&self, b: &Node) -> Node {
        let a_bits: Bits = self.clone().into();
        let b_bits: Bits = b.clone().into();

        let max_len = usize::max(a_bits.len(), b_bits.len());
        let mut result_bits: Bits = BitVec::with_capacity(max_len);

        for i in 0..max_len {
            let a_bit = if i < a_bits.len() { a_bits[i] } else { false };
            let b_bit = if i < b_bits.len() { b_bits[i] } else { false };
            result_bits.push(a_bit ^ b_bit);
        }

        Node::from(result_bits)
    }
}

/// Flatten leaf or compound node into bytes.
impl From<Node> for Vec<u8> {
    fn from(n: Node) -> Self {
        match n {
            Node::Bit(BitVal::Zero) => ZERO_BLOCK.to_vec(),
            Node::Bit(BitVal::One) => ONE_BLOCK.to_vec(),
            Node::Compound(pair) => {
                let (l, r) = *pair;
                let mut bits: Bits = BitVec::new();
                bits.extend_from_bitslice(&Bits::from(l));
                bits.extend_from_bitslice(&Bits::from(r));
                bits.into()
            }
        }
    }
}

impl From<SocketAddr> for Node {
    fn from(socket: SocketAddr) -> Self {
        // Placeholder implementation
        let s = socket.to_string(); // "127.0.0.1:40000"
        let bytes = s.as_bytes();
        let bits: Bits = BitVec::from_slice(bytes);
        Node::from(bits)
    }
}

/// Build a Node from Bits by recursively splitting.
impl From<Bits> for Node {
    fn from(bv: Bits) -> Self {
        if bv.is_empty() {
            panic!("Node::from: empty bitvector.");
        }

        const LEAF_THRESHOLD: usize = 64;

        if bv.len() <= LEAF_THRESHOLD {
            let ones = bv.iter().by_vals().filter(|b| *b).count();
            let zeros = bv.len() - ones;
            return Node::Bit(if ones >= zeros {
                BitVal::One
            } else {
                BitVal::Zero
            });
        }

        let mid = bv.len() / 2;
        let left = Node::from(bv[..mid].to_bitvec());
        let right = Node::from(bv[mid..].to_bitvec());
        Node::Compound(Box::new((left, right)))
    }
}

/// Flatten node → Bits.
impl From<Node> for Bits {
    fn from(n: Node) -> Self {
        match n {
            Node::Bit(BitVal::Zero) => BitVec::from_slice(ZERO_BLOCK),
            Node::Bit(BitVal::One) => BitVec::from_slice(ONE_BLOCK),
            Node::Compound(pair) => {
                let (l, r) = *pair;
                let mut out = Bits::new();
                out.extend_from_bitslice(&Bits::from(l));
                out.extend_from_bitslice(&Bits::from(r));
                out
            }
        }
    }
}
