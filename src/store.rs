use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

use bitvec::prelude::*;

use crate::node::Node; // adjust to your actual module path

/// Simple append-only log of Node frames.
/// File layout: [u32 len][len bytes of Node][u32 len][len bytes]...
pub struct Store {
    file: File,
}

impl Store {
    /// Open (or create) the log file.
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        Ok(Self { file })
    }

    /// Append a single Node as a frame.
    pub fn append_frame(&mut self, node: &Node) -> io::Result<()> {
        // Convert Node -> Vec<u8> (you already have impl From<Node> for Vec<u8>)
        let bytes: Vec<u8> = node.clone().into();
        let len = bytes.len() as u32;

        // Seek to end to keep it append-only.
        self.file.seek(SeekFrom::End(0))?;

        // Write length prefix (big-endian).
        self.file.write_all(&len.to_be_bytes())?;

        // Then write the raw bytes.
        self.file.write_all(&bytes)?;

        // Ensure it's on disk (optional but nice for durability).
        self.file.flush()?;

        Ok(())
    }

    /// Create an iterator over all frames from the beginning.
    pub fn iter(&mut self) -> io::Result<FrameIter> {
        // Rewind to start of file for reading
        self.file.seek(SeekFrom::Start(0))?;
        Ok(FrameIter {
            file: self.file.try_clone()?,
        })
    }
}

/// Iterator over frames in the log.
pub struct FrameIter {
    file: File,
}

impl Iterator for FrameIter {
    type Item = io::Result<Node>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut len_buf = [0u8; 4];

        // Try to read the length prefix
        match self.file.read_exact(&mut len_buf) {
            Ok(()) => {}
            Err(e) => {
                // If we hit EOF cleanly, stop the iteration
                if e.kind() == io::ErrorKind::UnexpectedEof {
                    return None;
                } else {
                    return Some(Err(e));
                }
            }
        }

        let len = u32::from_be_bytes(len_buf) as usize;
        let mut data = vec![0u8; len];

        if let Err(e) = self.file.read_exact(&mut data) {
            return Some(Err(e));
        }

        // data: Vec<u8> -> Bits -> Node
        let bits: BitVec<u8, Msb0> = BitVec::from_slice(&data);
        let node = Node::from(bits);

        Some(Ok(node))
    }
}
