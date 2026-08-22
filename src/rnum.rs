use core::ops::{Add, Div, Mul, Neg, Not, Rem, Sub};

/// Range number.
///
/// This independent signed/unsigned-bounds representation follows the scalar
/// range information maintained alongside tracked numbers by the Linux eBPF
/// verifier:
/// <https://docs.kernel.org/bpf/verifier.html#register-value-tracking>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rnum {
    max: u64,
    min: u64,

    smax: i64,
    smin: i64,
    empty: bool,
}

impl Rnum {
    const fn unknown() -> Self {
        Self {
            min: u64::MIN,
            max: u64::MAX,
            smin: i64::MIN,
            smax: i64::MAX,
            empty: false,
        }
    }

    /// Construct independent unsigned and signed inclusive ranges.
    pub const fn new(min: u64, max: u64, smin: i64, smax: i64) -> Option<Self> {
        if min <= max && smin <= smax {
            Some(Self {
                min,
                max,
                smin,
                smax,
                empty: false,
            })
        } else {
            None
        }
    }

    pub const fn from_value(value: u64) -> Self {
        let signed = value as i64;
        Self {
            min: value,
            max: value,
            smin: signed,
            smax: signed,
            empty: false,
        }
    }

    pub const fn unsigned_bounds(&self) -> (u64, u64) {
        (self.min, self.max)
    }

    pub const fn signed_bounds(&self) -> (i64, i64) {
        (self.smin, self.smax)
    }

    /// Whether the domain contains exactly one machine value.
    pub const fn is_const(&self) -> bool {
        matches!((self.min_value(), self.max_value()), (Some(min), Some(max)) if min == max)
    }

    /// Return the sole contained value, if this domain is constant.
    pub const fn value(&self) -> Option<u64> {
        match self.extrema() {
            Some((min, max)) if min == max => Some(min),
            _ => None,
        }
    }

    /// Whether the domain contains at least one machine value.
    pub const fn is_defined(&self) -> bool {
        self.has_value()
    }

    /// Whether this domain includes `value` in both interpretations.
    pub const fn contains_value(&self, value: u64) -> bool {
        !self.empty
            && self.min <= value
            && value <= self.max
            && self.smin <= value as i64
            && (value as i64) <= self.smax
    }

    /// Return the least range containing both operands.
    pub const fn union(&self, other: Self) -> Self {
        if self.empty || !self.has_value() {
            return other;
        }
        if other.empty || !other.has_value() {
            return *self;
        }
        Self {
            min: if self.min < other.min {
                self.min
            } else {
                other.min
            },
            max: if self.max > other.max {
                self.max
            } else {
                other.max
            },
            smin: if self.smin < other.smin {
                self.smin
            } else {
                other.smin
            },
            smax: if self.smax > other.smax {
                self.smax
            } else {
                other.smax
            },
            empty: false,
        }
    }

    /// Return the values represented by both operands.
    pub const fn intersection(&self, other: Self) -> Self {
        if self.empty || other.empty {
            return Self::empty();
        }
        let result = Self {
            min: if self.min > other.min {
                self.min
            } else {
                other.min
            },
            max: if self.max < other.max {
                self.max
            } else {
                other.max
            },
            smin: if self.smin > other.smin {
                self.smin
            } else {
                other.smin
            },
            smax: if self.smax < other.smax {
                self.smax
            } else {
                other.smax
            },
            empty: false,
        };
        if result.min > result.max || result.smin > result.smax || !result.has_value() {
            Self::empty()
        } else {
            result
        }
    }

    /// Whether this abstract range includes all bounds represented by `other`.
    pub const fn contains(&self, other: Self) -> bool {
        !other.has_value()
            || (self.has_value()
                && self.min <= other.min
                && other.max <= self.max
                && self.smin <= other.smin
                && other.smax <= self.smax)
    }

    pub const fn has_value(&self) -> bool {
        self.extrema().is_some()
    }

    pub const fn min_value(&self) -> Option<u64> {
        match self.extrema() {
            Some((min, _)) => Some(min),
            None => None,
        }
    }

    pub const fn max_value(&self) -> Option<u64> {
        match self.extrema() {
            Some((_, max)) => Some(max),
            None => None,
        }
    }

    const fn extrema(&self) -> Option<(u64, u64)> {
        if self.empty {
            return None;
        }
        let mut result: Option<(u64, u64)> = None;

        if self.smax >= 0 {
            let low = (if self.smin > 0 { self.smin } else { 0 }) as u64;
            let low = if low > self.min { low } else { self.min };
            let high = self.smax as u64;
            let high = if high < self.max { high } else { self.max };
            if low <= high {
                result = Some((low, high));
            }
        }
        if self.smin < 0 {
            let low = self.smin as u64;
            let low = if low > self.min { low } else { self.min };
            let signed_high = if self.smax < -1 { self.smax } else { -1 };
            let high = signed_high as u64;
            let high = if high < self.max { high } else { self.max };
            if low <= high {
                result = Some(match result {
                    Some((min, max)) => (
                        if min < low { min } else { low },
                        if max > high { max } else { high },
                    ),
                    None => (low, high),
                });
            }
        }
        result
    }

    const fn empty() -> Self {
        Self {
            min: 0,
            max: 0,
            smin: 0,
            smax: 0,
            empty: true,
        }
    }

    const fn bounded(min: u64, max: u64, smin: i64, smax: i64) -> Self {
        let result = Self {
            min,
            max,
            smin,
            smax,
            empty: false,
        };
        // The bounds are computed independently.  A valid input always leaves
        // at least one concrete result in their intersection, but retaining
        // this guard makes the helper robust against future transfer functions.
        if result.has_value() {
            result
        } else {
            Self::unknown()
        }
    }

    const fn signed_hull(min: u64, max: u64) -> (i64, i64) {
        if max <= i64::MAX as u64 || min >= (i64::MAX as u64) + 1 {
            (min as i64, max as i64)
        } else {
            (i64::MIN, i64::MAX)
        }
    }

    pub const fn add(self, other: Self) -> Self {
        if !self.has_value() || !other.has_value() {
            return Self::empty();
        }
        let (min, max) = if self.max <= u64::MAX - other.max {
            (self.min + other.min, self.max + other.max)
        } else {
            (u64::MIN, u64::MAX)
        };
        let signed_min = self.smin as i128 + other.smin as i128;
        let signed_max = self.smax as i128 + other.smax as i128;
        let (smin, smax) = if signed_min >= i64::MIN as i128 && signed_max <= i64::MAX as i128 {
            (signed_min as i64, signed_max as i64)
        } else {
            (i64::MIN, i64::MAX)
        };
        Self::bounded(min, max, smin, smax)
    }

    pub const fn subtract(self, other: Self) -> Self {
        if !self.has_value() || !other.has_value() {
            return Self::empty();
        }
        let (min, max) = if self.min >= other.max {
            (self.min - other.max, self.max - other.min)
        } else {
            (u64::MIN, u64::MAX)
        };
        let signed_min = self.smin as i128 - other.smax as i128;
        let signed_max = self.smax as i128 - other.smin as i128;
        let (smin, smax) = if signed_min >= i64::MIN as i128 && signed_max <= i64::MAX as i128 {
            (signed_min as i64, signed_max as i64)
        } else {
            (i64::MIN, i64::MAX)
        };
        Self::bounded(min, max, smin, smax)
    }

    pub const fn negate(self) -> Self {
        if !self.has_value() {
            return Self::empty();
        }
        let (min, max) = if self.min == 0 && self.max == 0 {
            (0, 0)
        } else if self.min > 0 {
            (self.max.wrapping_neg(), self.min.wrapping_neg())
        } else {
            (u64::MIN, u64::MAX)
        };
        let (smin, smax) = if self.smin != i64::MIN {
            (-self.smax, -self.smin)
        } else {
            (i64::MIN, i64::MAX)
        };
        Self::bounded(min, max, smin, smax)
    }

    pub const fn bit_not(self) -> Self {
        if !self.has_value() {
            return Self::empty();
        }
        Self::bounded(!self.max, !self.min, !self.smax, !self.smin)
    }

    pub const fn multiply(self, other: Self) -> Self {
        if !self.has_value() || !other.has_value() {
            return Self::empty();
        }
        let (min, max) = if self.max == 0 || other.max <= u64::MAX / self.max {
            (self.min * other.min, self.max * other.max)
        } else {
            (u64::MIN, u64::MAX)
        };
        let products = [
            self.smin as i128 * other.smin as i128,
            self.smin as i128 * other.smax as i128,
            self.smax as i128 * other.smin as i128,
            self.smax as i128 * other.smax as i128,
        ];
        let mut signed_min = products[0];
        let mut signed_max = products[0];
        let mut index = 1;
        while index < products.len() {
            if products[index] < signed_min {
                signed_min = products[index];
            }
            if products[index] > signed_max {
                signed_max = products[index];
            }
            index += 1;
        }
        let (smin, smax) = if signed_min >= i64::MIN as i128 && signed_max <= i64::MAX as i128 {
            (signed_min as i64, signed_max as i64)
        } else {
            (i64::MIN, i64::MAX)
        };
        Self::bounded(min, max, smin, smax)
    }

    pub const fn checked_div(self, other: Self) -> Option<Self> {
        if !self.has_value() || !other.has_value() {
            return Some(Self::empty());
        }
        if other.max == 0 {
            return None;
        }
        let least_divisor = if other.min == 0 { 1 } else { other.min };
        let min = self.min / other.max;
        let max = self.max / least_divisor;
        let (smin, smax) = Self::signed_hull(min, max);
        Some(Self::bounded(min, max, smin, smax))
    }

    pub const fn divide(self, other: Self) -> Self {
        match self.checked_div(other) {
            Some(result) => result,
            None => Self::empty(),
        }
    }

    pub const fn remainder(self, other: Self) -> Self {
        if !self.has_value() || !other.has_value() {
            return Self::empty();
        }
        if other.max == 0 {
            return Self::empty();
        }
        let max = if self.max < other.max - 1 {
            self.max
        } else {
            other.max - 1
        };
        let (smin, smax) = Self::signed_hull(0, max);
        Self::bounded(0, max, smin, smax)
    }
}

impl Default for Rnum {
    /// Default is the range containing every 64-bit machine value.
    fn default() -> Self {
        Self::unknown()
    }
}

impl Add for Rnum {
    type Output = Rnum;
    fn add(self, other: Self) -> Self {
        Self::add(self, other)
    }
}

impl Sub for Rnum {
    type Output = Rnum;
    fn sub(self, other: Self) -> Self {
        self.subtract(other)
    }
}

impl Neg for Rnum {
    type Output = Rnum;
    fn neg(self) -> Self {
        self.negate()
    }
}

impl Not for Rnum {
    type Output = Rnum;
    fn not(self) -> Self {
        self.bit_not()
    }
}

impl Mul for Rnum {
    type Output = Rnum;
    fn mul(self, other: Self) -> Self {
        self.multiply(other)
    }
}

impl Div for Rnum {
    type Output = Rnum;
    fn div(self, other: Self) -> Self {
        self.divide(other)
    }
}

impl Rem for Rnum {
    type Output = Rnum;
    fn rem(self, other: Self) -> Self {
        self.remainder(other)
    }
}
