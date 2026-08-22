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
    /// Construct independent unsigned and signed inclusive ranges.
    pub fn new(min: u64, max: u64, smin: i64, smax: i64) -> Option<Self> {
        (min <= max && smin <= smax).then_some(Self {
            min,
            max,
            smin,
            smax,
            empty: false,
        })
    }

    pub fn from_value(value: u64) -> Self {
        let signed = value as i64;
        Self {
            min: value,
            max: value,
            smin: signed,
            smax: signed,
            empty: false,
        }
    }

    pub fn unsigned_bounds(&self) -> (u64, u64) {
        (self.min, self.max)
    }

    pub fn signed_bounds(&self) -> (i64, i64) {
        (self.smin, self.smax)
    }

    /// Whether the domain contains exactly one machine value.
    pub fn is_const(&self) -> bool {
        matches!((self.min_value(), self.max_value()), (Some(min), Some(max)) if min == max)
    }

    /// Return the sole contained value, if this domain is constant.
    pub fn value(&self) -> Option<u64> {
        let min = self.min_value()?;
        (self.max_value() == Some(min)).then_some(min)
    }

    /// Whether the domain contains at least one machine value.
    pub fn is_defined(&self) -> bool {
        self.has_value()
    }

    /// Whether this domain includes `value` in both interpretations.
    pub fn contains_value(&self, value: u64) -> bool {
        self.min <= value
            && value <= self.max
            && self.smin <= value as i64
            && (value as i64) <= self.smax
    }

    /// Return the least range containing both operands.
    pub fn union(&self, other: Self) -> Self {
        if self.empty || !self.has_value() {
            return other;
        }
        if other.empty || !other.has_value() {
            return *self;
        }
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
            smin: self.smin.min(other.smin),
            smax: self.smax.max(other.smax),
            empty: false,
        }
    }

    /// Return the values represented by both operands.
    pub fn intersection(&self, other: Self) -> Self {
        if self.empty || other.empty {
            return Self::empty();
        }
        let result = Self {
            min: self.min.max(other.min),
            max: self.max.min(other.max),
            smin: self.smin.max(other.smin),
            smax: self.smax.min(other.smax),
            empty: false,
        };
        if result.min > result.max || result.smin > result.smax || !result.has_value() {
            Self::empty()
        } else {
            result
        }
    }

    /// Whether this abstract range includes all bounds represented by `other`.
    pub fn contains(&self, other: Self) -> bool {
        !other.has_value()
            || (self.has_value()
                && self.min <= other.min
                && other.max <= self.max
                && self.smin <= other.smin
                && other.smax <= self.smax)
    }

    pub fn has_value(&self) -> bool {
        self.extrema().is_some()
    }

    pub fn min_value(&self) -> Option<u64> {
        self.extrema().map(|(min, _)| min)
    }

    pub fn max_value(&self) -> Option<u64> {
        self.extrema().map(|(_, max)| max)
    }

    fn extrema(&self) -> Option<(u64, u64)> {
        if self.empty {
            return None;
        }
        let mut result: Option<(u64, u64)> = None;
        let mut include = |low: u64, high: u64| {
            let low = low.max(self.min);
            let high = high.min(self.max);
            if low <= high {
                result = Some(match result {
                    Some((min, max)) => (min.min(low), max.max(high)),
                    None => (low, high),
                });
            }
        };

        if self.smax >= 0 {
            include(self.smin.max(0) as u64, self.smax as u64);
        }
        if self.smin < 0 {
            include(self.smin as u64, self.smax.min(-1) as u64);
        }
        result
    }

    fn empty() -> Self {
        Self {
            min: 0,
            max: 0,
            smin: 0,
            smax: 0,
            empty: true,
        }
    }
}

impl Default for Rnum {
    /// Default is the range containing every 64-bit machine value.
    fn default() -> Self {
        Self {
            min: u64::MIN,
            max: u64::MAX,
            smin: i64::MIN,
            smax: i64::MAX,
            empty: false,
        }
    }
}

/*
impl Add for Rnum {
    type Output = Rnum;
    fn add(self, other: Self) -> Self {
        Self {
            max: self.max + other.max,
            min: self.min + other.min,
        }
    }
}

impl Sub for Rnum {
    type Output = Rnum;
    fn sub(self, other: Self) -> Self {
        unimplemented!()
    }
}

impl Neg for Rnum {
    type Output = Rnum;
    fn neg(self) -> Self {
        unimplemented!()
    }
}

impl Not for Rnum {
    type Output = Rnum;
    fn not(self) -> Self {
        unimplemented!()
    }
}

impl Mul for Rnum {
    type Output = Rnum;
    fn mul(self, other: Self) -> Self {
        unimplemented!()
    }
}

impl Div for Rnum {
    type Output = Rnum;
    fn div(self, other: Self) -> Self {
        unimplemented!()
    }
}

impl Rem for Rnum {
    type Output = Rnum;
    fn rem(self, other: Self) -> Self {
        unimplemented!()
    }
}
*/
