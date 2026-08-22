use core::ops::Add;

/// A bounded union of at most `K` disjoint, inclusive unsigned intervals.
///
/// Operations first compute their exact interval pieces and, when there are
/// too many, fill the smallest gaps until the result fits. Thus `K` is a
/// compile-time precision/storage knob.
///
/// References:
/// - GCC's bounded multi-range (`irange`) design:
///   <https://gcc.gnu.org/pipermail/gcc/2020-September/233620.html>
/// - Bagnara, Hill, and Zaffanella, "Widening Operators for Powerset Domains":
///   <https://www.cs.unipr.it/~zaffanella/Papers/Abstracts/Q349>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RangeSet<const K: usize = 2> {
    ranges: [(u64, u64); K],
    len: usize,
}

impl<const K: usize> RangeSet<K> {
    pub const fn empty() -> Self {
        assert!(K > 0, "a range set needs positive capacity");
        Self {
            ranges: [(0, 0); K],
            len: 0,
        }
    }

    pub fn full() -> Self {
        Self::from_range(0, u64::MAX)
    }

    pub fn from_value(value: u64) -> Self {
        Self::from_range(value, value)
    }

    pub fn from_range(low: u64, high: u64) -> Self {
        assert!(low <= high, "linear range bounds must be ordered");
        let mut result = Self::empty();
        result.ranges[0] = (low, high);
        result.len = 1;
        result
    }

    pub fn ranges(&self) -> &[(u64, u64)] {
        &self.ranges[..self.len]
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn contains_value(&self, value: u64) -> bool {
        self.ranges()
            .iter()
            .any(|&(low, high)| low <= value && value <= high)
    }

    pub fn union(self, other: Self) -> Self {
        let mut pieces = Vec::with_capacity(self.len + other.len);
        pieces.extend_from_slice(self.ranges());
        pieces.extend_from_slice(other.ranges());
        Self::from_pieces(pieces)
    }

    pub fn intersection(self, other: Self) -> Self {
        let mut pieces = Vec::with_capacity(self.len * other.len);
        for &(a, b) in self.ranges() {
            for &(c, d) in other.ranges() {
                let low = a.max(c);
                let high = b.min(d);
                if low <= high {
                    pieces.push((low, high));
                }
            }
        }
        Self::from_pieces(pieces)
    }

    /// The number of represented values. `2^64` is representable in `u128`.
    pub fn cardinality(&self) -> u128 {
        self.ranges()
            .iter()
            .map(|&(low, high)| u128::from(high) - u128::from(low) + 1)
            .sum()
    }

    fn from_pieces(mut pieces: Vec<(u64, u64)>) -> Self {
        if pieces.is_empty() {
            return Self::empty();
        }
        pieces.sort_unstable();
        let mut merged: Vec<(u64, u64)> = Vec::with_capacity(pieces.len());
        for (low, high) in pieces {
            if let Some(last) = merged.last_mut() {
                if low <= last.1.saturating_add(1) {
                    last.1 = last.1.max(high);
                    continue;
                }
            }
            merged.push((low, high));
        }

        while merged.len() > K {
            let gap = (0..merged.len() - 1)
                .min_by_key(|&i| u128::from(merged[i + 1].0) - u128::from(merged[i].1) - 1)
                .expect("more than one interval");
            let high = merged[gap + 1].1;
            merged[gap].1 = high;
            merged.remove(gap + 1);
        }

        let mut result = Self::empty();
        result.len = merged.len();
        result.ranges[..result.len].copy_from_slice(&merged);
        result
    }

    fn add_linear(a: (u64, u64), b: (u64, u64), out: &mut Vec<(u64, u64)>) {
        let low = u128::from(a.0) + u128::from(b.0);
        let high = u128::from(a.1) + u128::from(b.1);
        let modulus = 1_u128 << 64;
        if high - low + 1 >= modulus {
            out.push((0, u64::MAX));
        } else if high < modulus {
            out.push((low as u64, high as u64));
        } else if low >= modulus {
            out.push(((low - modulus) as u64, (high - modulus) as u64));
        } else {
            out.push((low as u64, u64::MAX));
            out.push((0, (high - modulus) as u64));
        }
    }
}

impl<const K: usize> Default for RangeSet<K> {
    fn default() -> Self {
        Self::full()
    }
}

impl<const K: usize> Add for RangeSet<K> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        if self.is_empty() || other.is_empty() {
            return Self::empty();
        }
        let mut pieces = Vec::with_capacity(self.len * other.len * 2);
        for &left in self.ranges() {
            for &right in other.ranges() {
                Self::add_linear(left, right, &mut pieces);
            }
        }
        Self::from_pieces(pieces)
    }
}
