use core::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Not, Rem, Shl, Shr, Sub};

/// Tracking number
///
/// Tracks on a bit-by-bit level whether we know the value of a bit & what that value is (if
/// known).
///
/// References:
///  - http://bitmath.blogspot.com/2013/08/addition-in-bitfield-domain.html
///  - http://bitmath.blogspot.com/2014/02/addition-in-bitfield-domain-alternative.html
///  - "Abstract Domains for Bit-Level Machine Integer and Floating-point Operations"
///    https://www-apr.lip6.fr/~mine/publi/article-mine-wing12.pdf
///  - https://www.omnimaga.org/other-computer-languages-help/addition-in-the-bitfield-domain/
///
// bits in mask: 1 = unknown, 0 = known
// bits in value, if known: 1 = 1, 0 = 0
// bits in value, if unknown = 0 (iow: 1 is forbidden if bit is unknown)
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Tnum {
    value: u64,
    mask: u64,
}

impl Tnum {
    pub fn from_value(value: u64) -> Self {
        Self { value, mask: 0 }
    }

    pub fn is_const(&self) -> bool {
        self.mask == 0
    }

    pub fn value(&self) -> Option<u64> {
        if self.mask == 0 {
            Some(self.value)
        } else {
            None
        }
    }
}

impl Default for Tnum {
    /// Default is a completely unknown value
    fn default() -> Self {
        Self { value: 0, mask: !0 }
    }
}

impl Not for Tnum {
    type Output = Tnum;
    fn not(self) -> Self {
        Self {
            value: !self.value,
            mask: self.mask,
        }
    }
}

impl BitOr for Tnum {
    type Output = Tnum;
    fn bitor(self, other: Self) -> Self {
        // algorithm from https://www.omnimaga.org/computer-programming/addition-in-the-bitfield-domain/
        // (m1, v1) | (m2, v2) = ((m1 & m2) | v1 | v2, v1 | v2)   // both known or one of them is 1
        let v1 = self.value | other.value;
        let m1 = self.mask | other.mask;
        // bit-wise saturation
        let m2 = m1 & !v1;

        Self {
            value: v1,
            mask: m2,
        }
    }
}

impl BitAnd for Tnum {
    type Output = Tnum;
    fn bitand(self, other: Self) -> Self {
        // algorithm from https://www.omnimaga.org/computer-programming/addition-in-the-bitfield-domain/
        // (m1, v1) & (m2, v2) = ((m1 & m2) | (m1 ^ v1) | (m2 ^ v2), v1 & v2) // both known or one of them is 0
        Self {
            mask: (self.mask & other.mask) | (self.mask ^ self.value) | (other.mask ^ other.value),
            value: self.value ^ other.value,
        }
    }
}

impl BitXor for Tnum {
    type Output = Tnum;
    fn bitxor(self, other: Self) -> Self {
        // algorithm from https://www.omnimaga.org/computer-programming/addition-in-the-bitfield-domain/
        // (m1, v1) ^ (m2, v2) = (m1 & m2, (v1 ^ v2) & m1 & m2)
        Self {
            mask: self.mask & other.mask,
            value: (self.value ^ other.value) & self.mask & other.mask,
        }
    }
}

impl Shl<u8> for Tnum {
    type Output = Tnum;
    fn shl(self, shift: u8) -> Self {
        Self {
            value: self.value << shift,
            mask: self.mask << shift,
        }
    }
}

impl Shr<u8> for Tnum {
    type Output = Tnum;
    fn shr(self, shift: u8) -> Self {
        Self {
            value: self.value >> shift,
            mask: self.mask >> shift,
        }
    }
}

impl Add for Tnum {
    // TODO: this will panic on overflow. It isn't immediately clear that the math here works in
    // the face of overflow, so we don't handle it. Likely should make this return a `Option<Tnum>`
    // to get checked overflows.
    type Output = Tnum;
    fn add(self, other: Self) -> Self::Output {
        // algorithm from https://www.omnimaga.org/computer-programming/addition-in-the-bitfield-domain/
        let an = !self.value & self.mask;
        let bn = !other.value & other.mask;
        let g0 = an & bn;
        let g1 = self.value & other.value;
        let p0 = an ^ bn;
        let pg1 = self.value | other.value;
        let g0l = !g0 & ((g0 << 1) | 1);
        let g1f = g1 & !(g1 << 1);
        let nc = (p0 + g0l & !p0) - g0l | g0;
        let m1 = !pg1 | (pg1 & g1f);
        // overflow here
        let c = (pg1 + g1f & m1) - g1f;
        let cm = (c | nc) << 1 | 1;
        let m = cm & self.mask & self.value;
        let v = self.value + other.value;

        Self { mask: m, value: v }
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
