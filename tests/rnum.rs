use numeric_domains::Rnum;

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
    assert!(empty.union(left).contains(left));
}
