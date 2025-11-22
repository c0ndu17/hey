use bitvec::prelude::*;

/// Bit-level buffer type (same as your `Bits` alias).
pub type Bits = BitVec<u8, Msb0>;

/// Universal entropy source, conceptually "bits of e^(1/e)".
///
/// Internally this keeps:
/// - a mutable generator state `f`
/// - the current step index `n`
/// - a growing buffer of generated bits
///
/// Each step advances `f` using your iterative rule:
///     f_{n+1} = 1 + f_n / (n+1)
/// and extracts one bit from the floating-point representation.
/// The bits are deterministic and can be reproduced anywhere
/// by replaying the generator.
#[derive(Debug, Clone)]
pub struct UniversalEntropy {
    n: u64,
    f: f64,
    bits: Bits,
}

impl UniversalEntropy {
    /// Create a new universal entropy generator.
    ///
    /// `f` is initialised near e^(1/e) to make the intent explicit,
    /// but the important part is that the generator is deterministic
    /// and shared, not that it numerically equals e^(1/e).
    pub fn new() -> Self {
        let f0 = std::f64::consts::E.powf(1.0 / std::f64::consts::E); // ≈ 1.444667...
        UniversalEntropy {
            n: 1,
            f: f0,
            bits: Bits::new(),
        }
    }

    /// Advance the generator by one step, append one bit to the stream.
    ///
    /// Update rule (your e-generator interpreted iteratively):
    ///     f_{n+1} = 1 + f_n / (n+1)
    ///
    /// Then we take one bit from the floating-point representation
    /// of `f` to get a deterministic boolean.
    fn step(&mut self) {
        // advance n
        self.n += 1;

        // update f via your rule
        self.f = 1.0 + self.f / (self.n as f64);

        // derive a bit from the current f
        // using its IEEE-754 bit-pattern for determinism
        let raw = self.f.to_bits(); // u64
        let bit = (raw & 1) == 1;

        self.bits.push(bit);
    }

    /// Ensure we have generated at least `pos + 1` bits.
    fn ensure_pos(&mut self, pos: usize) {
        while self.bits.len() <= pos {
            self.step();
        }
    }

    /// Get the universal bit at a given pos.
    ///
    /// This is a lazy interface: if the internal buffer does not yet
    /// reach `pos`, it will generate as many bits as needed.
    pub fn bit(&mut self, pos: usize) -> bool {
        self.ensure_pos(pos);
        self.bits[pos]
    }

    /// Get a prefix of the universal bit stream of length `n_bits`.
    pub fn bits_to_pos(&mut self, n_bits: usize) -> Bits {
        self.ensure_pos(n_bits.saturating_sub(1));
        self.bits[..n_bits].to_bitvec()
    }

    /// Get a range [start, start + len) from the universal bit stream.
    pub fn bits_range(&mut self, start: usize, len: usize) -> Bits {
        if len == 0 {
            return Bits::new();
        }
        self.ensure_pos(start + len - 1);
        self.bits[start..start + len].to_bitvec()
    }
}
