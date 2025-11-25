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
use std::net::SocketAddr;

use crate::entropy::UniversalEntropy;

pub const ROOT: &[u8] = b"hey";

/// CHUNK_SIZE for reading from stdin and UDP socket.
pub const SIZE: usize = 4096;

/// A leaf encodes as an 8-byte block of all zeros or all ones.
const ZERO_BLOCK: &[u8] = &[0u8; 1];
const ONE_BLOCK: &[u8] = &[0xFFu8; 1];

/// Bit-level buffer type used throughout.
pub type Bits = BitVec<u8, Msb0>;

/// - Bit(Zero/One): terminal leaf
/// - Compound(left, right): internal Node
/// - A compound is : C = [[1, [...A]], [1, [1, ...B]]]
#[derive(Debug, Clone)]
pub enum Node {
    Compound(Box<(Node, Node)>),
    Bit(bool),
}

impl Node {
    pub fn fold(&self, entropy: &mut UniversalEntropy, pos: usize) -> bool {
        let expected = entropy.bit(pos);
        let bit = match self {
            Node::Bit(b) => *b,
            Node::Compound(compound) => {
                let (left, right) = compound.as_ref();
                let left_bit = left.fold(entropy, pos);
                let right_bit = right.fold(entropy, pos);
                left_bit ^ right_bit
            }
        };
        bit ^ expected
    }

    pub fn reflect<'input>(&mut self, entropy: &mut UniversalEntropy, input: &'input Node) -> Node {
        // let branch = Node::Compound::(self, input);
        let branch = self.op(entropy, input).into();
        let node = match branch {
            Node::Bit(bit) => Node::Compound(Box::new((Node::Bit(bit), branch.into()))),
            Node::Compound(compound) => {
                let (left, right) = *compound;
                Node::Compound(Box::new((left.into(), right.into())))
            }
        };
        node
    }

    // pub fn op(&self, entropy: &mut UniversalEntropy, other: &Node) -> Bits {
    //     // let size = self.size().max(other.size());

    //     let mut result = Node::from(Box::new((self.clone(), other.clone())));

    //     result
    // }
    /// Perform a three-way XOR between two Nodes & their expected entropical value.
    pub fn op(&self, entropy: &mut UniversalEntropy, other: &Node) -> Bits {
        let size = self.size().max(other.size());
        let mut result: Bits = BitVec::with_capacity(size);

        for pos in 0..size {
            let a_bit = self.fold(entropy, pos);
            let b_bit = other.fold(entropy, pos);
            let expected = entropy.bit(pos);
            let res_bit = a_bit ^ b_bit ^ expected;
            result.push(res_bit);
        }

        result
    }

    pub fn size(&self) -> usize {
        match self {
            Node::Bit(_) => 1,
            Node::Compound(compound) => {
                let (left, right) = compound.as_ref();
                left.size() + right.size()
            }
        }
    }
}

impl Into<bool> for Node {
    fn into(self) -> bool {
        let entropy = &mut UniversalEntropy::new();
        match self {
            Node::Bit(_) => self.into(),
            Node::Compound(compound) => {
                let (left, right) = *compound;
                let left_bit: bool = left.into();
                let right_bit: bool = right.into();
                left_bit ^ right_bit ^ entropy.bit(0)
            }
        }
    }
}

/// Flatten leaf or compound node into bytes.
impl From<Node> for Vec<u8> {
    fn from(n: Node) -> Self {
        match n {
            Node::Bit(false) => ZERO_BLOCK.to_vec(),
            Node::Bit(true) => ONE_BLOCK.to_vec(),
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

impl From<&Node> for Bits {
    fn from(n: &Node) -> Self {
        match n {
            Node::Bit(false) => BitVec::from_slice(ZERO_BLOCK),
            Node::Bit(true) => BitVec::from_slice(ONE_BLOCK),
            Node::Compound(value) => {
                let (left, right) = value.as_ref();
                let mut out = Bits::new();
                let left_bits: Bits = Bits::from(left);
                let right_bits: Bits = Bits::from(right);
                out.extend_from_bitslice(&left_bits.to_owned());
                out.extend_from_bitslice(&right_bits.to_owned());
                out
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
            return Node::Bit(if ones >= zeros { true } else { false });
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
            Node::Bit(false) => BitVec::from_slice(ZERO_BLOCK),
            Node::Bit(true) => BitVec::from_slice(ONE_BLOCK),
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
