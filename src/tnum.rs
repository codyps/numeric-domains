use core::ops::{Add, BitAnd, BitOr, BitXor, Not, Shl, Shr};

/// Tracking number
///
/// Tracks on a bit-by-bit level whether we know the value of a bit & what that value is (if
/// known).
///
/// References:
///  - Linux's production implementation: <https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/bpf/tnum.c>
///  - Linux verifier documentation: <https://docs.kernel.org/bpf/verifier.html#register-value-tracking>
///  - <https://bitmath.blogspot.com/2013/08/addition-in-bitfield-domain.html>
///  - <https://bitmath.blogspot.com/2014/02/addition-in-bitfield-domain-alternative.html>
///  - "Abstract Domains for Bit-Level Machine Integer and Floating-point Operations"
///    ([paper](https://www-apr.lip6.fr/~mine/publi/article-mine-wing12.pdf))
///  - <https://www.omnimaga.org/other-computer-languages-help/addition-in-the-bitfield-domain/>
///
// bits in mask: 1 = unknown, 0 = known
// bits in value, if known: 1 = 1, 0 = 0
// bits in value, if unknown = 0 (iow: 1 is forbidden if bit is unknown)
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Tnum {
    value: u64,
    mask: u64,
    empty: bool,
}

impl Tnum {
    /// The empty set.
    pub const fn empty() -> Self {
        Self {
            value: 0,
            mask: 0,
            empty: true,
        }
    }

    /// Construct a tracking number from its known value and unknown-bit mask.
    ///
    /// Value bits covered by the mask are cleared to maintain the canonical
    /// `value & mask == 0` representation.
    pub fn from_parts(value: u64, mask: u64) -> Self {
        Self {
            value: value & !mask,
            mask,
            empty: false,
        }
    }

    pub fn from_value(value: u64) -> Self {
        Self {
            value,
            mask: 0,
            empty: false,
        }
    }

    /// Return the canonical `(known_value, unknown_mask)` representation.
    pub fn parts(&self) -> Option<(u64, u64)> {
        (!self.empty).then_some((self.value, self.mask))
    }

    pub fn is_const(&self) -> bool {
        !self.empty && self.mask == 0
    }

    pub fn value(&self) -> Option<u64> {
        if self.is_const() {
            Some(self.value)
        } else {
            None
        }
    }

    /// Whether this abstract value includes `value`.
    pub fn contains_value(&self, value: u64) -> bool {
        !self.empty && value & !self.mask == self.value
    }

    /// Return the least tracking number containing both operands.
    pub fn union(self, other: Self) -> Self {
        if self.empty {
            return other;
        }
        if other.empty {
            return self;
        }
        let differing = self.value ^ other.value;
        let mask = self.mask | other.mask | differing;
        Self::from_parts(self.value, mask)
    }

    /// Return the values represented by both operands.
    pub fn intersection(self, other: Self) -> Self {
        if self.empty || other.empty {
            return Self::empty();
        }
        let conflicting = (self.value ^ other.value) & !(self.mask | other.mask);
        if conflicting != 0 {
            Self::empty()
        } else {
            Self::from_parts(self.value | other.value, self.mask & other.mask)
        }
    }

    pub fn is_defined(&self) -> bool {
        !self.empty
    }

    /// Whether this tracking number includes every value in `other`.
    pub fn contains(&self, other: Self) -> bool {
        other.empty
            || (!self.empty
                && other.mask & !self.mask == 0
                && other.value & !self.mask == self.value)
    }

    pub fn has_value(&self) -> bool {
        !self.empty
    }

    pub fn min_value(&self) -> Option<u64> {
        (!self.empty).then_some(self.value)
    }

    pub fn max_value(&self) -> Option<u64> {
        (!self.empty).then_some(self.value | self.mask)
    }

    pub fn unsigned_bounds(&self) -> (u64, u64) {
        (self.value, self.value | self.mask)
    }

    pub fn signed_bounds(&self) -> (i64, i64) {
        const SIGN: u64 = 1 << 63;
        if self.mask & SIGN != 0 {
            (
                (self.value | SIGN) as i64,
                ((self.value | self.mask) & !SIGN) as i64,
            )
        } else {
            (self.value as i64, (self.value | self.mask) as i64)
        }
    }
}

impl Default for Tnum {
    /// Default is a completely unknown value
    fn default() -> Self {
        Self {
            value: 0,
            mask: !0,
            empty: false,
        }
    }
}

impl Not for Tnum {
    type Output = Tnum;
    fn not(self) -> Self {
        if self.empty {
            return self;
        }
        Self {
            value: !self.value & !self.mask,
            mask: self.mask,
            empty: false,
        }
    }
}

impl BitOr for Tnum {
    type Output = Tnum;
    fn bitor(self, other: Self) -> Self {
        if self.empty || other.empty {
            return Self::empty();
        }
        // algorithm from https://www.omnimaga.org/computer-programming/addition-in-the-bitfield-domain/
        // (m1, v1) | (m2, v2) = ((m1 & m2) | v1 | v2, v1 | v2)   // both known or one of them is 1
        let v1 = self.value | other.value;
        let m1 = self.mask | other.mask;
        // bit-wise saturation
        let m2 = m1 & !v1;

        Self {
            value: v1,
            mask: m2,
            empty: false,
        }
    }
}

impl BitAnd for Tnum {
    type Output = Tnum;
    fn bitand(self, other: Self) -> Self {
        if self.empty || other.empty {
            return Self::empty();
        }
        let value = self.value & other.value;
        let may_be_one = (self.value | self.mask) & (other.value | other.mask);
        Self::from_parts(value, may_be_one & !value)
    }
}

impl BitXor for Tnum {
    type Output = Tnum;
    fn bitxor(self, other: Self) -> Self {
        if self.empty || other.empty {
            return Self::empty();
        }
        let mask = self.mask | other.mask;
        Self::from_parts(self.value ^ other.value, mask)
    }
}

impl Shl<u8> for Tnum {
    type Output = Tnum;
    fn shl(self, shift: u8) -> Self {
        if self.empty {
            return self;
        }
        let shift = u32::from(shift).rem_euclid(64);
        Self {
            value: self.value.wrapping_shl(shift),
            mask: self.mask.wrapping_shl(shift),
            empty: false,
        }
    }
}

impl Shr<u8> for Tnum {
    type Output = Tnum;
    fn shr(self, shift: u8) -> Self {
        if self.empty {
            return self;
        }
        let shift = u32::from(shift).rem_euclid(64);
        Self {
            value: self.value.wrapping_shr(shift),
            mask: self.mask.wrapping_shr(shift),
            empty: false,
        }
    }
}

impl Add for Tnum {
    type Output = Tnum;
    fn add(self, other: Self) -> Self::Output {
        if self.empty || other.empty {
            return Self::empty();
        }
        // Carry propagation can make every bit at and above an unknown input
        // bit unknown. This is the algorithm used by Linux's `tnum_add`:
        // https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/bpf/tnum.c
        // All additions intentionally use machine wrapping.
        let mask_sum = self.mask.wrapping_add(other.mask);
        let value_sum = self.value.wrapping_add(other.value);
        let sigma = mask_sum.wrapping_add(value_sum);
        let carry_changes = sigma ^ value_sum;
        let mask = carry_changes | self.mask | other.mask;
        Self::from_parts(value_sum, mask)
    }
}

/*
impl Sub for Tnum {
    type Output = Tnum;
    fn sub(self, other: Self) -> Self {
        unimplemented!()
    }
}

impl Mul for Tnum {
    type Output = Tnum;
    fn mul(self, other: Self) -> Self {
        unimplemented!()
    }
}

impl Div for Tnum {
    type Output = Tnum;
    fn div(self, other: Self) -> Self {
        unimplemented!()
    }
}

impl Rem for Tnum {
    type Output = Tnum;
    fn rem(self, other: Self) -> Self {
        unimplemented!()
    }
}

impl Neg for Tnum {
    type Output = Tnum;
    fn neg(self) -> Self {
        unimplemented!()
    }
}

*/
