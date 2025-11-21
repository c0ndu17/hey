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
use std::net::SocketAddr;

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

/// - Bit(Zero/One): terminal leaf
/// - Compound(left, right): internal Node
/// - A compound is : C = [[1, [...A]], [1, [1, ...B]]]
#[derive(Debug, Clone)]
pub enum Node {
    Compound(Box<(Node, Node)>),
    Bit(BitVal),
}

impl Node {
    pub fn next(&mut self, input: Node) -> Result<Node, io::Error> {
        let node = match input {
            Node::Bit(bit) => &Node::Compound(Box::new((Node::Bit(bit), self.clone()))),
            Node::Compound(_) => &Node::from(self.op(&input)),
        };
        Ok(node.to_owned())
    }

    /// Perform a bitwise XOR between two Nodes.
    pub fn op(&self, other: &Node) -> Bits {
        let a_bits: Bits = self.clone().into();
        let b_bits: Bits = other.clone().into();

        a_bits
            .iter()
            .by_vals()
            .zip(b_bits.iter().by_vals())
            .map(|(a, b)| a ^ b)
            .collect()
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

        let node = Node::Compound(Box::new((left, right)));
        node
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
