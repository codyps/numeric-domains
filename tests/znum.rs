use numeric_domains::Znum;
use proptest::prelude::*;

proptest! {
    #[test]
    fn const_value_roundtrip(x: u64) {
        prop_assert_eq!(Znum::from_value(x).value(), Some(x));
    }

    #[test]
    fn const_contains(x: u64) {
        prop_assert!(Znum::from_value(x).contains_value(x));
    }

    #[test]
    fn const_is_defined(x: u64) {
        prop_assert!(Znum::from_value(x).is_defined());
    }

    #[test]
    fn const_bitor_is_value(x: u64, y: u64) {
        let result = Znum::from_value(x) | Znum::from_value(y);
        prop_assert_eq!(result.value(), Some(x | y));
    }

    #[test]
    fn const_bitand_is_value(x: u64, y: u64) {
        let result = Znum::from_value(x) & Znum::from_value(y);
        prop_assert_eq!(result.value(), Some(x & y));
    }

    #[test]
    fn const_xor_is_value(x: u64, y: u64) {
        let result = Znum::from_value(x) ^ Znum::from_value(y);
        prop_assert_eq!(result.value(), Some(x ^ y));
    }

    #[test]
    fn const_shl_is_value(x: u64, shift: u8) {
        let effective_shift = u32::from(shift).rem_euclid(64);
        let result = Znum::from_value(x) << shift;
        prop_assert_eq!(result.value(), Some(x.wrapping_shl(effective_shift)));
    }

    #[test]
    fn const_shr_is_value(x: u64, shift: u8) {
        let effective_shift = u32::from(shift).rem_euclid(64);
        let result = Znum::from_value(x) >> shift;
        prop_assert_eq!(result.value(), Some(x.wrapping_shr(effective_shift)));
    }

    #[test]
    fn const_not_is_value(x: u64) {
        prop_assert_eq!((!Znum::from_value(x)).value(), Some(!x));
    }

    #[test]
    fn const_add_is_wrapping_value(x: u64, y: u64) {
        let result = Znum::from_value(x) + Znum::from_value(y);
        prop_assert_eq!(result.value(), Some(x.wrapping_add(y)));
    }

    #[test]
    fn const_sub_is_wrapping_value(x: u64, y: u64) {
        let result = Znum::from_value(x) - Znum::from_value(y);
        prop_assert_eq!(result.value(), Some(x.wrapping_sub(y)));
    }

    #[test]
    fn const_neg_is_wrapping_value(x: u64) {
        prop_assert_eq!((-Znum::from_value(x)).value(), Some(x.wrapping_neg()));
    }

    #[test]
    fn const_mul_is_wrapping_value(x: u64, y: u64) {
        let result = Znum::from_value(x) * Znum::from_value(y);
        prop_assert_eq!(result.value(), Some(x.wrapping_mul(y)));
    }

    #[test]
    fn const_div_is_value(x: u64, y in 1_u64..) {
        prop_assert_eq!((Znum::from_value(x) / Znum::from_value(y)).value(), Some(x / y));
        prop_assert_eq!(
            Znum::from_value(x).checked_div(Znum::from_value(y)),
            Some(Znum::from_value(x / y)),
        );
    }

    #[test]
    fn const_checked_div_by_zero_is_none(x: u64) {
        prop_assert_eq!(Znum::from_value(x).checked_div(Znum::from_value(0)), None);
    }

    #[test]
    fn const_rem_is_value(x: u64, y in 1_u64..) {
        prop_assert_eq!((Znum::from_value(x) % Znum::from_value(y)).value(), Some(x % y));
    }

    #[test]
    fn union_of_constants_contains_both(x: u64, y: u64) {
        let left = Znum::from_value(x);
        let right = Znum::from_value(y);
        let union = left.union(right);

        prop_assert_eq!(union, right.union(left));
        prop_assert!(union.contains_value(x));
        prop_assert!(union.contains_value(y));
    }

    #[test]
    fn union_contains_pairwise_unions(x: u64, y: u64, z: u64) {
        let x = Znum::from_value(x);
        let y = Znum::from_value(y);
        let z = Znum::from_value(z);
        let union = x.union(y).union(z);

        prop_assert!(union.contains(x.union(y)));
        prop_assert!(union.contains(y.union(z)));
        prop_assert!(union.contains(z.union(x)));
    }

    #[test]
    fn constant_intersection_is_empty_or_equal(x: u64, y: u64) {
        let intersection = Znum::from_value(x).intersection(Znum::from_value(y));
        prop_assert!(!intersection.is_defined() || x == y);
    }

    #[test]
    fn min_value_is_valid(ones: u64, zeros: u64) {
        let value = Znum::from_parts(ones, zeros);
        if let Some(minimum) = value.min_value() {
            prop_assert!(value.contains_value(minimum));
        }
    }
}

#[test]
fn instance_is_defined() {
    assert!(!Znum::from_parts(0, 0).is_defined());
    assert!(!Znum::from_parts(0, 1).is_defined());
    assert!(!Znum::from_parts(1, 1).is_defined());
    assert!(!Znum::from_parts(1, 0).is_defined());
    assert!(!Znum::from_parts(0xfffffffffffffffe, 0xfffffffffffffffe).is_defined());
    assert!(Znum::from_parts(0xfffffffffffffffe, 0xffffffffffffffff).is_defined());
    assert!(Znum::from_parts(0xffffffffffffffff, 0xfffffffffffffffe).is_defined());
    assert!(Znum::from_parts(0xffffffffffffffff, 0xffffffffffffffff).is_defined());
}

#[test]
fn empty_domain_is_absorbing_for_addition() {
    let empty = Znum::from_parts(0, 0);
    let result = empty + Znum::from_value(1);
    assert!(!result.has_value());
}

#[test]
fn empty_domain_is_absorbing_for_subtraction() {
    let empty = Znum::from_parts(0, 0);
    assert!(!(empty - Znum::from_value(1)).has_value());
    assert!(!(Znum::from_value(1) - empty).has_value());
}

#[test]
fn every_empty_encoding_is_the_union_identity() {
    let empty = Znum::from_parts(1, 0);
    let zero = Znum::from_value(0);
    assert!(!empty.has_value());
    assert_eq!(empty.union(zero), zero);
    assert_eq!(zero.union(empty), zero);
}

#[test]
fn shifts_preserve_empty_domains() {
    let missing_sign_bit = Znum::from_parts(0, u64::MAX >> 1);
    let missing_low_bit = Znum::from_parts(0, u64::MAX << 1);
    assert!(!(missing_sign_bit << 1).has_value());
    assert!(!(missing_low_bit >> 1).has_value());
}

#[test]
fn empty_domain_and_zero_divisors_have_no_arithmetic_results() {
    let empty = Znum::from_parts(0, 0);
    let one = Znum::from_value(1);
    let zero = Znum::from_value(0);
    assert!(!(-empty).has_value());
    assert!(!(empty * one).has_value());
    assert!(!(one * empty).has_value());
    assert!(!(empty / one).has_value());
    assert!(!(one / empty).has_value());
    assert!(!(empty % one).has_value());
    assert!(!(one % empty).has_value());
    assert!(!(one / zero).has_value());
    assert!(!(one % zero).has_value());
}

#[test]
fn contains_requires_all_other_possibilities() {
    let zero = Znum::from_value(0);
    let low_bit_unknown = Znum::from_parts(1, u64::MAX);
    assert!(!zero.contains(low_bit_unknown));
    assert!(low_bit_unknown.contains(zero));
}

#[test]
fn shifts_use_modulo_64_counts() {
    let value = Znum::from_value(3);
    assert_eq!((value << 64).value(), Some(3));
    assert_eq!((value >> 128).value(), Some(3));
}

#[test]
fn default_and_bounds_match_the_common_domain_api() {
    let unknown = Znum::default();
    assert_eq!(unknown.unsigned_bounds(), Some((u64::MIN, u64::MAX)));
    assert_eq!(unknown.signed_bounds(), Some((i64::MIN, i64::MAX)));

    let empty = Znum::from_parts(0, 0);
    assert_eq!(empty.unsigned_bounds(), None);
    assert_eq!(empty.signed_bounds(), None);
}

#[test]
fn union_and_intersection_have_set_semantics() {
    let left = Znum::from_parts(0b0111, !0b0001);
    let right = Znum::from_parts(0b0111, !0b0011);
    let union = left.union(right);
    let intersection = left.intersection(right);

    for value in 0..16 {
        assert!(union.contains_value(value) || !left.contains_value(value));
        assert!(union.contains_value(value) || !right.contains_value(value));
        assert_eq!(
            intersection.contains_value(value),
            left.contains_value(value) && right.contains_value(value)
        );
    }

    let empty = Znum::from_value(1).intersection(Znum::from_value(2));
    assert!(!empty.has_value());
    assert_eq!(empty.union(left), left);
}

#[test]
fn reduced_width_operations_contain_all_concrete_results() {
    const WIDTH: u32 = 4;
    const LIMIT: u64 = 1 << WIDTH;

    let domains: Vec<_> = (0..LIMIT)
        .flat_map(|unknown| {
            (0..LIMIT)
                .filter(move |ones| ones & unknown == 0)
                .map(move |ones| {
                    let may_be_one = ones | unknown;
                    let may_be_zero = !ones | unknown;
                    Znum::from_parts(may_be_one, may_be_zero)
                })
        })
        .collect();

    for &left in &domains {
        for a in (0..LIMIT).filter(|&value| left.contains_value(value)) {
            assert!((!left).contains_value(!a));

            for raw_shift in u8::MIN..=u8::MAX {
                let shift = u32::from(raw_shift).rem_euclid(64);
                assert!((left << raw_shift).contains_value(a.wrapping_shl(shift)));
                assert!((left >> raw_shift).contains_value(a.wrapping_shr(shift)));
            }
        }

        for &right in &domains {
            for a in (0..LIMIT).filter(|&value| left.contains_value(value)) {
                for b in (0..LIMIT).filter(|&value| right.contains_value(value)) {
                    assert!((left & right).contains_value(a & b));
                    assert!((left | right).contains_value(a | b));
                    assert!((left ^ right).contains_value(a ^ b));
                    assert!((left + right).contains_value(a.wrapping_add(b)));
                    assert!((left - right).contains_value(a.wrapping_sub(b)));
                    assert!((left * right).contains_value(a.wrapping_mul(b)));
                    if let Some(quotient) = a.checked_div(b) {
                        assert!((left / right).contains_value(quotient));
                        assert!((left % right).contains_value(a % b));
                    }
                }
            }
        }
    }
}
