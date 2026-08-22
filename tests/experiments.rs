use numeric_domains::{BitRange, RangeSet, Tnum, WrappedInterval};
use proptest::prelude::*;

#[test]
fn wrapped_interval_keeps_overflow_result_compact() {
    let near_max = WrappedInterval::new(u64::MAX - 2, u64::MAX);
    let result = near_max + WrappedInterval::from_value(2);

    assert_eq!(result.bounds(), Some((u64::MAX, 1)));
    assert!(result.is_wrapping());
    assert_eq!(result.cardinality(), 3);
    assert!(result.contains_value(u64::MAX));
    assert!(result.contains_value(0));
    assert!(result.contains_value(1));
    assert!(!result.contains_value(2));
}

#[test]
fn two_ranges_preserve_a_branch_disjunction() {
    let values = RangeSet::<2>::from_range(10, 15).union(RangeSet::from_range(20, 30));

    assert_eq!(values.ranges(), &[(10, 15), (20, 30)]);
    assert!(!values.contains_value(17));
    assert_eq!(values.cardinality(), 17);
}

#[test]
fn capacity_fills_the_smallest_gap_first() {
    let values = RangeSet::<2>::from_value(0)
        .union(RangeSet::from_value(10))
        .union(RangeSet::from_value(12));

    assert_eq!(values.ranges(), &[(0, 0), (10, 12)]);
    assert!(!values.contains_value(5));
}

#[test]
fn range_set_addition_preserves_wrapping_as_two_pieces() {
    let result = RangeSet::<2>::from_range(u64::MAX - 2, u64::MAX) + RangeSet::from_value(2);

    assert_eq!(result.ranges(), &[(0, 1), (u64::MAX, u64::MAX)]);
    assert_eq!(result.cardinality(), 3);
}

#[test]
fn reduced_product_is_stricter_than_either_component() {
    let value = BitRange::<2>::new(
        RangeSet::from_range(8, 11),
        Tnum::from_parts(0, u64::MAX - 1),
    );

    assert!(value.contains_value(8));
    assert!(!value.contains_value(9));
    assert!(value.contains_value(10));
    assert!(!value.contains_value(11));
}

#[test]
fn reduction_moves_common_prefix_bits_out_of_ranges() {
    let value = BitRange::<2>::from_ranges(RangeSet::from_range(0x1200, 0x12ff));
    let (known, unknown) = value.bits().parts().unwrap();

    assert_eq!(known, 0x1200);
    assert_eq!(unknown, 0xff);
}

#[test]
fn incompatible_components_reduce_to_empty() {
    let value = BitRange::<2>::new(RangeSet::from_range(0, 7), Tnum::from_parts(8, 0));
    assert!(value.is_empty());
}

proptest! {
    #[test]
    fn wrapped_add_contains_concrete_results(
        a_low: u64,
        a_len in 0_u16..1024,
        b_low: u64,
        b_len in 0_u16..1024,
        ai in 0_u16..1024,
        bi in 0_u16..1024,
    ) {
        let a_len = u64::from(a_len);
        let b_len = u64::from(b_len);
        let left = WrappedInterval::new(a_low, a_low.wrapping_add(a_len));
        let right = WrappedInterval::new(b_low, b_low.wrapping_add(b_len));
        let a = a_low.wrapping_add(u64::from(ai) % (a_len + 1));
        let b = b_low.wrapping_add(u64::from(bi) % (b_len + 1));

        prop_assert!((left + right).contains_value(a.wrapping_add(b)));
    }

    #[test]
    fn range_set_add_contains_concrete_results(
        a_low: u64,
        a_len in 0_u8..64,
        b_low: u64,
        b_len in 0_u8..64,
        ai in 0_u8..64,
        bi in 0_u8..64,
    ) {
        let a_high = a_low.saturating_add(u64::from(a_len));
        let b_high = b_low.saturating_add(u64::from(b_len));
        let left = RangeSet::<2>::from_range(a_low, a_high);
        let right = RangeSet::<2>::from_range(b_low, b_high);
        let a = a_low + u64::from(ai) % (a_high - a_low + 1);
        let b = b_low + u64::from(bi) % (b_high - b_low + 1);

        prop_assert!((left + right).contains_value(a.wrapping_add(b)));
    }
}
