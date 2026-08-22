use core::ops::Add;

use crate::{RangeSet, Tnum};

/// Reduced product of a bounded interval union and known/unknown bits.
///
/// Its concrete values satisfy both components. Reduction exchanges cheap
/// unsigned-bound and common-bit facts after construction and operations.
///
/// This follows the reduced-product pattern used by the Linux eBPF verifier,
/// which maintains signed bounds, unsigned bounds, and a tnum together:
/// <https://docs.kernel.org/bpf/verifier.html#register-value-tracking>
/// LLVM implements the analogous conversions between `ConstantRange` and
/// `KnownBits`:
/// <https://github.com/llvm/llvm-project/blob/main/llvm/lib/IR/ConstantRange.cpp>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitRange<const K: usize = 2> {
    ranges: RangeSet<K>,
    bits: Tnum,
}

impl<const K: usize> BitRange<K> {
    pub fn new(ranges: RangeSet<K>, bits: Tnum) -> Self {
        let mut result = Self { ranges, bits };
        result.reduce();
        result
    }

    pub fn from_value(value: u64) -> Self {
        Self::new(RangeSet::from_value(value), Tnum::from_value(value))
    }

    pub fn from_ranges(ranges: RangeSet<K>) -> Self {
        Self::new(ranges, Tnum::default())
    }

    pub fn from_bits(bits: Tnum) -> Self {
        Self::new(RangeSet::default(), bits)
    }

    pub fn ranges(&self) -> RangeSet<K> {
        self.ranges
    }
    pub fn bits(&self) -> Tnum {
        self.bits
    }

    pub fn contains_value(&self, value: u64) -> bool {
        self.ranges.contains_value(value) && self.bits.contains_value(value)
    }

    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty() || !self.bits.has_value()
    }

    pub fn union(self, other: Self) -> Self {
        if self.is_empty() {
            return other;
        }
        if other.is_empty() {
            return self;
        }
        Self::new(self.ranges.union(other.ranges), self.bits.union(other.bits))
    }

    pub fn intersection(self, other: Self) -> Self {
        Self::new(
            self.ranges.intersection(other.ranges),
            self.bits.intersection(other.bits),
        )
    }

    fn reduce(&mut self) {
        if self.ranges.is_empty() || !self.bits.has_value() {
            self.ranges = RangeSet::empty();
            self.bits = Tnum::empty();
            return;
        }

        // Every value in a linear interval shares the prefix above the most
        // significant bit on which its endpoints differ.
        let mut range_bits = Tnum::empty();
        for &(low, high) in self.ranges.ranges() {
            let differing = low ^ high;
            let unknown = if differing == 0 {
                0
            } else {
                u64::MAX >> differing.leading_zeros()
            };
            let piece = Tnum::from_parts(low & !unknown, unknown);
            range_bits = range_bits.union(piece);
        }
        self.bits = self.bits.intersection(range_bits);
        if !self.bits.has_value() {
            self.ranges = RangeSet::empty();
            return;
        }

        // A tnum's unsigned extrema are exact, even if there are holes.
        let (low, high) = self.bits.unsigned_bounds();
        self.ranges = self.ranges.intersection(RangeSet::from_range(low, high));
        if self.ranges.is_empty() {
            self.bits = Tnum::empty();
        }
    }
}

impl<const K: usize> Default for BitRange<K> {
    fn default() -> Self {
        Self::new(RangeSet::default(), Tnum::default())
    }
}

impl<const K: usize> Add for BitRange<K> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.ranges + other.ranges, self.bits + other.bits)
    }
}
