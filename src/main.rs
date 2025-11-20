/*
 * hey,
 *
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
use std::io::{self, Read};

mod node;
use node::{Bits, Node, ROOT, SIZE};

fn main() -> io::Result<()> {
    let mut handle = io::stdin().lock();
    let mut buf = [0u8; SIZE];

    // Starting state from ROOT.

    println!("=== {}- (AGPLv3) ===\n", String::from_utf8_lossy(ROOT));
    let mut node: Node = Node::from(BitVec::from_slice(ROOT));

    loop {
        // Show current state.

        let n = handle.read(&mut buf)?;
        if n == 0 {
            break;
        }

        let chunk = &buf[..n];

        match std::str::from_utf8(chunk) {
            Ok(s) => println!("[IN::Ok] Text: {:?}", s),
            Err(_) => println!("[IN::Err] Bytes: {:?}", chunk),
        }
        println!();

        // Fold the new chunk into the evolving entropical state.
        let input_bits: Bits = BitVec::from_slice(chunk);
        let input_node: Node = Node::from(input_bits);

        node = node.next(input_node, true);
    }

    Ok(())
}
