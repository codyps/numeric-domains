use core::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Rem, Shl, Shr, Sub};

/// Tracks which bits "may be 1s" (o) and "may be 0s" (z)
///
/// Compared to other bit domains, the Z domain requires minimal storage, which is not scaled with
/// the number of operations, but as a result the accuracy of the domain is somewhat limited.
///
///  - ["Abstract Domains for Bit-Level Machine Integer and Floating-point Operations"](https://www-apr.lip6.fr/~mine/publi/article-mine-wing12.pdf)
///  - Published proceedings entry and DOI: <https://doi.org/10.29007/b63g>
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Znum {
    z: u64,
    o: u64,
}

impl Znum {
    pub fn from_parts(ones: u64, zeros: u64) -> Self {
        Znum { o: ones, z: zeros }
    }

    /// From a value, generate a Znum
    ///
    /// The resulting Znum only contains the provided value `v`, and no other values. It is
    /// considered a "constant"
    pub fn from_value(v: u64) -> Self {
        Znum { o: v, z: !v }
    }

    /// Is there only a single contained value?
    pub fn is_const(&self) -> bool {
        // all const bits (differing)
        let a = self.z ^ self.o;
        // ensure all are set
        !a == 0
    }

    /// If this is a constant (only a single contained value), return that value. Otherwise, return
    /// None.
    pub fn value(&self) -> Option<u64> {
        if self.is_const() {
            Some(self.o)
        } else {
            None
        }
    }

    /// Is any value contained in this?
    ///
    /// In other words, are there _no_ undefined bits?
    pub fn is_defined(&self) -> bool {
        // all _defined_ bits
        let a = self.z | self.o;
        // ensure all are set
        !a == 0
    }

    /// Is a specific value contained in this?
    pub fn contains_value(&self, v: u64) -> bool {
        // bits provided by `ones`
        let po = self.o & v;
        // bits provided by `zeros`
        let pz = self.z & !v;
        // ensure all bits are provided
        !(po | pz) == 0
    }

    /// Return the least Z-domain containing every value in either operand.
    pub fn union(&self, other: Self) -> Self {
        if !self.has_value() {
            return other;
        }
        if !other.has_value() {
            return *self;
        }
        Znum {
            o: self.o | other.o,
            z: self.z | other.z,
        }
    }

    /// `self` includes all possible elements in `other`
    pub fn contains(&self, other: Self) -> bool {
        other.o & !self.o == 0 && other.z & !self.z == 0
    }

    /// Return the values represented by both operands.
    pub fn intersection(&self, other: Self) -> Self {
        Znum {
            o: self.o & other.o,
            z: self.z & other.z,
        }
    }

    pub fn has_value(&self) -> bool {
        !(self.o | self.z) == 0
    }

    pub fn max_value(&self) -> Option<u64> {
        if self.has_value() {
            Some(self.o)
        } else {
            None
        }
    }

    pub fn min_value(&self) -> Option<u64> {
        if self.has_value() {
            Some(self.o & !(self.z))
        } else {
            None
        }
    }

    pub fn unsigned_bounds(&self) -> Option<(u64, u64)> {
        Some((self.min_value()?, self.max_value()?))
    }

    pub fn signed_bounds(&self) -> Option<(i64, i64)> {
        const SIGN: u64 = 1 << 63;
        if !self.has_value() {
            return None;
        }
        match (self.z & SIGN != 0, self.o & SIGN != 0) {
            (true, true) => Some((((self.o & !self.z) | SIGN) as i64, (self.o & !SIGN) as i64)),
            (true, false) => Some(((self.o & !self.z) as i64, self.o as i64)),
            (false, true) => Some(((self.o & !self.z) as i64, self.o as i64)),
            (false, false) => None,
        }
    }

    /// Divide by `other`, returning `None` when it can only be zero.
    ///
    /// If `other` contains both zero and nonzero values, the result describes
    /// the divisions by its nonzero values.
    pub fn checked_div(self, other: Self) -> Option<Self> {
        if other.max_value() == Some(0) {
            return None;
        }

        if !self.has_value() || !other.has_value() {
            return Some(Self { z: 0, o: 0 });
        }

        match (self.value(), other.value()) {
            (Some(dividend), Some(divisor)) => {
                Some(Self::from_value(dividend.checked_div(divisor)?))
            }
            _ => Some(Self::default()),
        }
    }

    /*
    /// All elements in `other` are also elements in `self`
    pub fn is_subset(&self, other: Self) -> bool {
        todo!()
    }
    */

    /*
    pub fn from_range(low: u64, high: u64) -> Self {
        todo!()
    }
    */
}

impl Default for Znum {
    /// Default is a completely unknown value.
    fn default() -> Self {
        Self {
            z: u64::MAX,
            o: u64::MAX,
        }
    }
}

impl BitOr for Znum {
    type Output = Znum;
    fn bitor(self, other: Self) -> Self {
        Self {
            z: self.z & other.z,
            o: self.o | other.o,
        }
    }
}

impl BitAnd for Znum {
    type Output = Znum;
    fn bitand(self, other: Self) -> Self {
        Self {
            z: self.z | other.z,
            o: self.o & other.o,
        }
    }
}

impl BitXor for Znum {
    type Output = Znum;
    fn bitxor(self, other: Self) -> Self {
        Self {
            z: (self.z & other.z) | (self.o & other.o),
            o: (self.z & other.o) | (self.o & other.z),
        }
    }
}

impl Not for Znum {
    type Output = Znum;
    fn not(self) -> Self {
        Self {
            z: self.o,
            o: self.z,
        }
    }
}

impl Add for Znum {
    type Output = Znum;

    fn add(self, other: Self) -> Self {
        // An undefined bit makes the represented Cartesian product empty.
        if !self.has_value() || !other.has_value() {
            return Self { z: 0, o: 0 };
        }

        // Convert may-zero/may-one bits to the equivalent known-value and
        // unknown-mask representation, propagate carries, then convert back.
        let left_value = self.o & !self.z;
        let left_mask = self.o & self.z;
        let right_value = other.o & !other.z;
        let right_mask = other.o & other.z;

        let mask_sum = left_mask.wrapping_add(right_mask);
        let value_sum = left_value.wrapping_add(right_value);
        let sigma = mask_sum.wrapping_add(value_sum);
        let carry_changes = sigma ^ value_sum;
        let mask = carry_changes | left_mask | right_mask;
        let value = value_sum & !mask;

        Self {
            o: value | mask,
            z: !value | mask,
        }
    }
}

impl Sub for Znum {
    type Output = Znum;

    fn sub(self, other: Self) -> Self {
        if !self.has_value() || !other.has_value() {
            return Self { z: 0, o: 0 };
        }

        // Subtraction in the equivalent known-value/unknown-mask domain. The
        // transfer function is Linux's `tnum_sub`, translated back to the
        // may-zero/may-one representation:
        // https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/tree/kernel/bpf/tnum.c
        // `alpha` and `beta` expose every borrow that may change when the
        // left or right unknown bits vary.
        let left_value = self.o & !self.z;
        let left_mask = self.o & self.z;
        let right_value = other.o & !other.z;
        let right_mask = other.o & other.z;

        let value_difference = left_value.wrapping_sub(right_value);
        let alpha = value_difference.wrapping_add(left_mask);
        let beta = value_difference.wrapping_sub(right_mask);
        let borrow_changes = alpha ^ beta;
        let mask = borrow_changes | left_mask | right_mask;
        let value = value_difference & !mask;

        Self {
            o: value | mask,
            z: !value | mask,
        }
    }
}

impl Shl<u8> for Znum {
    type Output = Znum;
    fn shl(self, shift: u8) -> Self {
        if !self.has_value() {
            return Self { z: 0, o: 0 };
        }
        let shift = u32::from(shift).rem_euclid(64);
        // ones move up, zeros move up, empty space filled by zeros
        Self {
            z: self.z.wrapping_shl(shift) | (1_u64.wrapping_shl(shift) - 1),
            o: self.o.wrapping_shl(shift),
        }
    }
}

impl Shr<u8> for Znum {
    type Output = Znum;
    fn shr(self, shift: u8) -> Self {
        if !self.has_value() {
            return Self { z: 0, o: 0 };
        }
        let shift = u32::from(shift).rem_euclid(64);
        // ones move down, zeros move down, empty space filled by zeros
        //
        // note: this special case with zero prevents us from shifting by 64bits (and getting a
        // shift overflow).
        let nz = if shift == 0 {
            0
        } else {
            (u64::MAX).wrapping_shl(64 - shift)
        };

        Self {
            z: self.z.wrapping_shr(shift) | nz,
            o: self.o.wrapping_shr(shift),
        }
    }
}

/*
impl Add for Znum {
    type Output = Znum;
    fn add(self, other: Self) -> Self {
        /*
         * 1 bit addition truth table:
         *
         * o1z1o2z2O Z
         * 0 0 0 0 0 0
         * 0 0 0 1 0 0
         * 0 0 1 0 0 0
         * 0 0 1 1 0 0
         * 0 1 0 0 0 0
         * 0 1 0 1 0 1
         * 0 1 1 0 1 0
         * 0 1 1 1 1 1
         * 1 0 0 0 0 0
         * 1 0 0 1 1 0
         * 1 0 1 0 0 1
         * 1 0 1 1 1 1
         * 1 1 0 0 0 0
         * 1 1 0 1 1 1
         * 1 1 1 0 1 1
         * 1 1 1 1 1 1
         */

        /*
         * +1:
         *   o: self.o + 1 | ((self.o ^ self.z) & 1)
         *   z: self.z
         *
         *  if self.o & 1 == 1
         *
         * +2:
         *   o
         */

        /*
        Self {
            o: self.o + other.o,
        }
        */

    }
}
*/

impl Neg for Znum {
    type Output = Znum;

    fn neg(self) -> Self {
        Self::from_value(0) - self
    }
}

impl Mul for Znum {
    type Output = Znum;

    fn mul(self, other: Self) -> Self {
        if !self.has_value() || !other.has_value() {
            return Self { z: 0, o: 0 };
        }

        // Abstract shift-and-add multiplication. At an unknown multiplier
        // bit, union the products obtained by omitting and adding the term.
        let mut product = Self::from_value(0);
        for bit in 0_u8..64 {
            let mask = 1_u64 << bit;
            if self.o & mask != 0 {
                let with_bit = product + (other << bit);
                product = if self.z & mask != 0 {
                    product.union(with_bit)
                } else {
                    with_bit
                };
            }
        }
        product
    }
}

impl Div for Znum {
    type Output = Znum;

    fn div(self, other: Self) -> Self {
        self.checked_div(other).unwrap_or(Self { z: 0, o: 0 })
    }
}

impl Rem for Znum {
    type Output = Znum;

    fn rem(self, other: Self) -> Self {
        if !self.has_value() || !other.has_value() || other.max_value() == Some(0) {
            return Self { z: 0, o: 0 };
        }

        match (self.value(), other.value()) {
            (Some(dividend), Some(divisor)) if divisor != 0 => Self::from_value(dividend % divisor),
            _ => Self::default(),
        }
    }
}
