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
    pub const fn from_parts(ones: u64, zeros: u64) -> Self {
        if ones | zeros == u64::MAX {
            Znum { o: ones, z: zeros }
        } else {
            Self::empty()
        }
    }

    const fn empty() -> Self {
        Self { z: 0, o: 0 }
    }

    const fn is_empty(&self) -> bool {
        self.o | self.z == 0
    }

    const fn validity_mask(&self) -> u64 {
        self.o | self.z
    }

    const fn unknown() -> Self {
        Self {
            z: u64::MAX,
            o: u64::MAX,
        }
    }

    /// From a value, generate a Znum
    ///
    /// The resulting Znum only contains the provided value `v`, and no other values. It is
    /// considered a "constant"
    pub const fn from_value(v: u64) -> Self {
        Znum { o: v, z: !v }
    }

    /// Is there only a single contained value?
    pub const fn is_const(&self) -> bool {
        // all const bits (differing)
        let a = self.z ^ self.o;
        // ensure all are set
        !a == 0
    }

    /// If this is a constant (only a single contained value), return that value. Otherwise, return
    /// None.
    pub const fn value(&self) -> Option<u64> {
        if self.is_const() {
            Some(self.o)
        } else {
            None
        }
    }

    /// Is any value contained in this?
    ///
    /// In other words, are there _no_ undefined bits?
    pub const fn is_defined(&self) -> bool {
        !self.is_empty()
    }

    /// Is a specific value contained in this?
    pub const fn contains_value(&self, v: u64) -> bool {
        // bits provided by `ones`
        let po = self.o & v;
        // bits provided by `zeros`
        let pz = self.z & !v;
        // ensure all bits are provided
        !(po | pz) == 0
    }

    /// Return the least Z-domain containing every value in either operand.
    pub const fn union(&self, other: Self) -> Self {
        if self.is_empty() {
            return other;
        }
        if other.is_empty() {
            return *self;
        }
        Znum {
            o: self.o | other.o,
            z: self.z | other.z,
        }
    }

    /// `self` includes all possible elements in `other`
    pub const fn contains(&self, other: Self) -> bool {
        other.is_empty() || (!self.is_empty() && other.o & !self.o == 0 && other.z & !self.z == 0)
    }

    /// Return the values represented by both operands.
    pub const fn intersection(&self, other: Self) -> Self {
        Self::from_parts(self.o & other.o, self.z & other.z)
    }

    pub const fn has_value(&self) -> bool {
        !self.is_empty()
    }

    pub const fn max_value(&self) -> Option<u64> {
        if self.has_value() {
            Some(self.o)
        } else {
            None
        }
    }

    pub const fn min_value(&self) -> Option<u64> {
        if self.has_value() {
            Some(self.o & !(self.z))
        } else {
            None
        }
    }

    pub const fn unsigned_bounds(&self) -> Option<(u64, u64)> {
        if self.is_empty() {
            None
        } else {
            Some((self.o & !self.z, self.o))
        }
    }

    pub const fn signed_bounds(&self) -> Option<(i64, i64)> {
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

    pub const fn bit_or(self, other: Self) -> Self {
        let valid = self.validity_mask() & other.validity_mask();
        Self {
            z: (self.z & other.z) & valid,
            o: (self.o | other.o) & valid,
        }
    }

    pub const fn bit_and(self, other: Self) -> Self {
        let valid = self.validity_mask() & other.validity_mask();
        Self {
            z: (self.z | other.z) & valid,
            o: (self.o & other.o) & valid,
        }
    }

    pub const fn bit_xor(self, other: Self) -> Self {
        let valid = self.validity_mask() & other.validity_mask();
        Self {
            z: ((self.z & other.z) | (self.o & other.o)) & valid,
            o: ((self.z & other.o) | (self.o & other.z)) & valid,
        }
    }

    pub const fn bit_not(self) -> Self {
        Self {
            z: self.o,
            o: self.z,
        }
    }

    pub const fn add(self, other: Self) -> Self {
        if self.is_empty() || other.is_empty() {
            return Self::empty();
        }
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

    pub const fn subtract(self, other: Self) -> Self {
        if self.is_empty() || other.is_empty() {
            return Self::empty();
        }
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

    pub const fn shift_left(self, shift: u8) -> Self {
        if self.is_empty() {
            return Self::empty();
        }
        let shift = (shift as u32) % 64;
        Self {
            z: self.z.wrapping_shl(shift) | (1_u64.wrapping_shl(shift) - 1),
            o: self.o.wrapping_shl(shift),
        }
    }

    pub const fn shift_right(self, shift: u8) -> Self {
        if self.is_empty() {
            return Self::empty();
        }
        let shift = (shift as u32) % 64;
        let new_zeros = if shift == 0 {
            0
        } else {
            u64::MAX.wrapping_shl(64 - shift)
        };
        Self {
            z: self.z.wrapping_shr(shift) | new_zeros,
            o: self.o.wrapping_shr(shift),
        }
    }

    pub const fn negate(self) -> Self {
        Self::from_value(0).subtract(self)
    }

    pub const fn multiply(self, other: Self) -> Self {
        if self.is_empty() || other.is_empty() {
            return Self::empty();
        }
        let mut product = Self::from_value(0);
        let mut bit = 0_u8;
        while bit < 64 {
            let mask = 1_u64 << bit;
            if self.o & mask != 0 {
                let with_bit = product.add(other.shift_left(bit));
                product = if self.z & mask != 0 {
                    product.union(with_bit)
                } else {
                    with_bit
                };
            }
            bit += 1;
        }
        product
    }

    /// Divide by `other`, returning `None` when it can only be zero.
    ///
    /// If `other` contains both zero and nonzero values, the result describes
    /// the divisions by its nonzero values.
    pub const fn checked_div(self, other: Self) -> Option<Self> {
        if let Some(0) = other.max_value() {
            return None;
        }

        if self.is_empty() || other.is_empty() {
            return Some(Self::empty());
        }

        match (self.value(), other.value()) {
            (Some(_), Some(0)) => None,
            (Some(dividend), Some(divisor)) => Some(Self::from_value(dividend / divisor)),
            _ => Some(Self::unknown()),
        }
    }

    pub const fn divide(self, other: Self) -> Self {
        match self.checked_div(other) {
            Some(result) => result,
            None => Self::empty(),
        }
    }

    pub const fn remainder(self, other: Self) -> Self {
        if self.is_empty() || other.is_empty() {
            return Self::empty();
        }
        match (self.value(), other.value()) {
            (_, Some(0)) => Self::empty(),
            (Some(dividend), Some(divisor)) => Self::from_value(dividend % divisor),
            _ => Self::unknown(),
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
        Self::unknown()
    }
}

impl BitOr for Znum {
    type Output = Znum;
    fn bitor(self, other: Self) -> Self {
        Self::bit_or(self, other)
    }
}

impl BitAnd for Znum {
    type Output = Znum;
    fn bitand(self, other: Self) -> Self {
        Self::bit_and(self, other)
    }
}

impl BitXor for Znum {
    type Output = Znum;
    fn bitxor(self, other: Self) -> Self {
        Self::bit_xor(self, other)
    }
}

impl Not for Znum {
    type Output = Znum;
    fn not(self) -> Self {
        self.bit_not()
    }
}

impl Add for Znum {
    type Output = Znum;

    fn add(self, other: Self) -> Self {
        Self::add(self, other)
    }
}

impl Sub for Znum {
    type Output = Znum;

    fn sub(self, other: Self) -> Self {
        self.subtract(other)
    }
}

impl Shl<u8> for Znum {
    type Output = Znum;
    fn shl(self, shift: u8) -> Self {
        self.shift_left(shift)
    }
}

impl Shr<u8> for Znum {
    type Output = Znum;
    fn shr(self, shift: u8) -> Self {
        self.shift_right(shift)
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
        self.negate()
    }
}

impl Mul for Znum {
    type Output = Znum;

    fn mul(self, other: Self) -> Self {
        self.multiply(other)
    }
}

impl Div for Znum {
    type Output = Znum;

    fn div(self, other: Self) -> Self {
        self.divide(other)
    }
}

impl Rem for Znum {
    type Output = Znum;

    fn rem(self, other: Self) -> Self {
        self.remainder(other)
    }
}
