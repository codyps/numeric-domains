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

    pub const fn add(self, other: Self) -> Self {
        if !self.has_value() || !other.has_value() {
            return Self::empty();
        }
        match (self.value(), other.value()) {
            (Some(left), Some(right)) => Self::from_value(left.wrapping_add(right)),
            _ => Self::unknown(),
        }
    }

    pub const fn subtract(self, other: Self) -> Self {
        if !self.has_value() || !other.has_value() {
            return Self::empty();
        }
        match (self.value(), other.value()) {
            (Some(left), Some(right)) => Self::from_value(left.wrapping_sub(right)),
            _ => Self::unknown(),
        }
    }

    pub const fn negate(self) -> Self {
        if !self.has_value() {
            return Self::empty();
        }
        match self.value() {
            Some(value) => Self::from_value(value.wrapping_neg()),
            None => Self::unknown(),
        }
    }

    pub const fn bit_not(self) -> Self {
        if !self.has_value() {
            return Self::empty();
        }
        match self.value() {
            Some(value) => Self::from_value(!value),
            None => Self::unknown(),
        }
    }

    pub const fn multiply(self, other: Self) -> Self {
        if !self.has_value() || !other.has_value() {
            return Self::empty();
        }
        match (self.value(), other.value()) {
            (Some(left), Some(right)) => Self::from_value(left.wrapping_mul(right)),
            _ => Self::unknown(),
        }
    }

    pub const fn checked_div(self, other: Self) -> Option<Self> {
        if !self.has_value() || !other.has_value() {
            return Some(Self::empty());
        }
        match (self.value(), other.value()) {
            (_, Some(0)) => None,
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
        if !self.has_value() || !other.has_value() {
            return Self::empty();
        }
        match (self.value(), other.value()) {
            (_, Some(0)) => Self::empty(),
            (Some(dividend), Some(divisor)) => Self::from_value(dividend % divisor),
            _ => Self::unknown(),
        }
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
