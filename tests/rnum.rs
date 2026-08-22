use numeric_domains::Rnum;

const CONST_ONE: Rnum = Rnum::from_value(1);
const CONST_TWO: Rnum = Rnum::from_value(2);
const CONST_SUM: Rnum = CONST_ONE.add(CONST_TWO);
const CONST_DIFFERENCE: Rnum = CONST_ONE.subtract(CONST_TWO);
const CONST_NEGATION: Rnum = CONST_ONE.negate();
const CONST_NOT: Rnum = CONST_ONE.bit_not();
const CONST_PRODUCT: Rnum = CONST_TWO.multiply(CONST_TWO);
const CONST_DIVISION: Rnum = CONST_TWO.divide(CONST_ONE);
const CONST_REMAINDER: Rnum = CONST_TWO.remainder(CONST_ONE);

#[test]
fn operators_have_const_inherent_equivalents() {
    assert_eq!(CONST_SUM, CONST_ONE + CONST_TWO);
    assert_eq!(CONST_DIFFERENCE, CONST_ONE - CONST_TWO);
    assert_eq!(CONST_NEGATION, -CONST_ONE);
    assert_eq!(CONST_NOT, !CONST_ONE);
    assert_eq!(CONST_PRODUCT, CONST_TWO * CONST_TWO);
    assert_eq!(CONST_DIVISION, CONST_TWO / CONST_ONE);
    assert_eq!(CONST_REMAINDER, CONST_TWO % CONST_ONE);
}

#[test]
fn constructs_valid_ranges() {
    let range = Rnum::new(1, 10, -4, 7).expect("valid bounds");
    assert_eq!(range.unsigned_bounds(), (1, 10));
    assert_eq!(range.signed_bounds(), (-4, 7));
}

#[test]
fn rejects_reversed_bounds() {
    assert_eq!(Rnum::new(10, 1, -4, 7), None);
    assert_eq!(Rnum::new(1, 10, 7, -4), None);
}

#[test]
fn value_is_a_singleton_in_both_interpretations() {
    let range = Rnum::from_value(u64::MAX);
    assert_eq!(range.unsigned_bounds(), (u64::MAX, u64::MAX));
    assert_eq!(range.signed_bounds(), (-1, -1));
}

#[test]
fn common_domain_queries_handle_signed_and_unsigned_bounds() {
    let range = Rnum::new(0, u64::MAX, -1, 1).unwrap();
    assert!(range.contains_value(0));
    assert!(range.contains_value(1));
    assert!(range.contains_value(u64::MAX));
    assert_eq!(range.min_value(), Some(0));
    assert_eq!(range.max_value(), Some(u64::MAX));
    assert!(!range.is_const());

    let positive = Rnum::new(2, 4, 2, 4).unwrap();
    let negative = Rnum::new(u64::MAX, u64::MAX, -1, -1).unwrap();
    let union = positive.union(negative);
    assert!(union.contains(positive));
    assert!(union.contains(negative));
}

#[test]
fn default_contains_every_machine_value() {
    let range = Rnum::default();
    assert!(range.is_defined());
    assert!(range.contains_value(0));
    assert!(range.contains_value(u64::MAX));
}

#[test]
fn union_and_intersection_have_set_semantics() {
    let left = Rnum::new(1, 5, 1, 5).unwrap();
    let right = Rnum::new(4, 8, 4, 8).unwrap();
    let union = left.union(right);
    let intersection = left.intersection(right);

    for value in 0..10 {
        assert!(union.contains_value(value) || !left.contains_value(value));
        assert!(union.contains_value(value) || !right.contains_value(value));
        assert_eq!(
            intersection.contains_value(value),
            left.contains_value(value) && right.contains_value(value)
        );
    }

    let empty = Rnum::from_value(1).intersection(Rnum::from_value(2));
    assert!(!empty.has_value());
    assert!(!empty.contains_value(0));
    assert!(empty.union(left).contains(left));
}

#[test]
fn arithmetic_preserves_non_wrapping_bounds() {
    let left = Rnum::new(2, 4, 2, 4).unwrap();
    let right = Rnum::new(5, 7, 5, 7).unwrap();

    assert_eq!((left + right).unsigned_bounds(), (7, 11));
    assert_eq!((left + right).signed_bounds(), (7, 11));
    assert_eq!((right - left).unsigned_bounds(), (1, 5));
    assert_eq!((left * right).unsigned_bounds(), (10, 28));
    assert_eq!((right / left).unsigned_bounds(), (1, 3));
    assert_eq!((right % left).unsigned_bounds(), (0, 3));
    assert_eq!((!left).unsigned_bounds(), (!4, !2));
}

#[test]
fn wrapping_only_forgets_the_affected_interpretation() {
    let near_unsigned_max = Rnum::new(u64::MAX - 1, u64::MAX, -2, -1).unwrap();
    let one = Rnum::from_value(1);
    let result = near_unsigned_max + one;

    assert_eq!(result.unsigned_bounds(), (u64::MIN, u64::MAX));
    assert_eq!(result.signed_bounds(), (-1, 0));
    assert!(result.contains_value(u64::MAX));
    assert!(result.contains_value(0));
}

#[test]
fn division_ignores_zero_when_nonzero_divisors_are_available() {
    let dividend = Rnum::new(10, 20, 10, 20).unwrap();
    let divisor = Rnum::new(0, 4, 0, 4).unwrap();
    assert_eq!(
        dividend.checked_div(divisor).unwrap().unsigned_bounds(),
        (2, 20)
    );
    assert_eq!(dividend.checked_div(Rnum::from_value(0)), None);
}
