use core::ops::Add;

use crate::RangeSet;

/// One inclusive interval on the circle of integers modulo `2^64`.
///
/// References:
/// - Gange et al., "Interval Analysis and Machine Arithmetic: Why Signedness
///   Ignorance Is Bliss": <https://doi.org/10.1145/2693264>
/// - LLVM's production `ConstantRange` implementation:
///   <https://llvm.org/doxygen/classllvm_1_1ConstantRange.html>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WrappedInterval {
    low: u64,
    high: u64,
    empty: bool,
    full: bool,
}

impl WrappedInterval {
    pub const fn empty() -> Self {
        Self {
            low: 0,
            high: 0,
            empty: true,
            full: false,
        }
    }

    pub const fn full() -> Self {
        Self {
            low: 0,
            high: u64::MAX,
            empty: false,
            full: true,
        }
    }

    /// Construct the clockwise arc from `low` through `high`, inclusive.
    /// `low > high` deliberately denotes a wrapping interval.
    pub const fn new(low: u64, high: u64) -> Self {
        Self {
            low,
            high,
            empty: false,
            full: false,
        }
    }

    pub const fn from_value(value: u64) -> Self {
        Self::new(value, value)
    }

    pub const fn bounds(&self) -> Option<(u64, u64)> {
        if self.empty {
            None
        } else {
            Some((self.low, self.high))
        }
    }

    pub fn is_wrapping(&self) -> bool {
        !self.empty && !self.full && self.low > self.high
    }

    pub fn contains_value(&self, value: u64) -> bool {
        if self.empty {
            false
        } else if self.full {
            true
        } else if self.low <= self.high {
            self.low <= value && value <= self.high
        } else {
            value >= self.low || value <= self.high
        }
    }

    pub fn cardinality(&self) -> u128 {
        if self.empty {
            0
        } else if self.full {
            1_u128 << 64
        } else {
            u128::from(self.high.wrapping_sub(self.low)) + 1
        }
    }

    /// Split at the unsigned zero point. The conversion is exact.
    pub fn as_range_set(&self) -> RangeSet<2> {
        if self.empty {
            RangeSet::empty()
        } else if self.full {
            RangeSet::full()
        } else if self.low <= self.high {
            RangeSet::from_range(self.low, self.high)
        } else {
            RangeSet::from_range(self.low, u64::MAX).union(RangeSet::from_range(0, self.high))
        }
    }
}

impl Default for WrappedInterval {
    fn default() -> Self {
        Self::full()
    }
}

impl Add for WrappedInterval {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        if self.empty || other.empty {
            return Self::empty();
        }
        if self.full || other.full || self.cardinality() + other.cardinality() > (1_u128 << 64) {
            return Self::full();
        }
        Self::new(
            self.low.wrapping_add(other.low),
            self.high.wrapping_add(other.high),
        )
    }
}
