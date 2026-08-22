use numeric_domains::Tnum;
use proptest::prelude::*;

proptest! {
    // NOTE: the domain of non-overflowing addition is smaller than the full range of the `u64`.
    // Minimally shrink things. This is really nasty, and there should be a simpler way to do
    // things.
    #[test]
    fn exact_addition(a: u64, b: u64) {
        let tr = Tnum::from_value(a) + Tnum::from_value(b);
        let r = a.wrapping_add(b);

        assert_eq!(Some(r), tr.value());
    }

}

#[test]
fn constant_bitwise_operations_are_exact() {
    let one = Tnum::from_value(1);
    let zero = Tnum::from_value(0);

    assert_eq!((one & one).value(), Some(1));
    assert_eq!((one ^ zero).value(), Some(1));
    assert_eq!((!one).value(), Some(!1));
}

#[test]
fn unknown_addition_does_not_collapse_to_a_constant() {
    let result = Tnum::default() + Tnum::from_value(0);
    assert!(!result.is_const());
    assert!(result.contains_value(0));
    assert!(result.contains_value(u64::MAX));
}

#[test]
fn shifts_use_modulo_64_counts() {
    let value = Tnum::from_value(3);
    assert_eq!((value << 64).value(), Some(3));
    assert_eq!((value >> 128).value(), Some(3));
}

#[test]
fn common_domain_queries_report_bounds_and_containment() {
    let low_bit_unknown = Tnum::from_parts(2, 1);
    assert!(low_bit_unknown.is_defined());
    assert!(low_bit_unknown.has_value());
    assert_eq!(low_bit_unknown.unsigned_bounds(), (2, 3));
    assert_eq!(low_bit_unknown.signed_bounds(), (2, 3));
    assert!(low_bit_unknown.contains(Tnum::from_value(2)));
    assert!(low_bit_unknown.contains(Tnum::from_value(3)));
    assert!(!Tnum::from_value(2).contains(low_bit_unknown));

    let sign_unknown = Tnum::from_parts(5, 1 << 63);
    assert_eq!(sign_unknown.signed_bounds(), (i64::MIN + 5, 5));
}

#[test]
fn union_and_intersection_have_set_semantics() {
    let left = Tnum::from_parts(0b0001, 0b0110);
    let right = Tnum::from_parts(0b0011, 0b0100);
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

    let empty = Tnum::from_value(1).intersection(Tnum::from_value(2));
    assert!(!empty.has_value());
    assert_eq!(empty.union(left), left);
}

#[test]
fn reduced_width_operations_contain_all_concrete_results() {
    const WIDTH: u32 = 4;
    const LIMIT: u64 = 1 << WIDTH;

    let domains: Vec<_> = (0..LIMIT)
        .flat_map(|mask| {
            (0..LIMIT)
                .filter(move |value| value & mask == 0)
                .map(move |value| Tnum::from_parts(value, mask))
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
                }
            }
        }
    }
}
