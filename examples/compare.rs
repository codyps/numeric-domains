use numeric_domains::{BitRange, RangeSet, Tnum, WrappedInterval};

fn main() {
    let wrapping = WrappedInterval::new(u64::MAX - 2, u64::MAX) + WrappedInterval::from_value(2);
    println!(
        "wrapping add: {:?}, {} values (ordinary unsigned hull: 2^64 values)",
        wrapping.bounds(),
        wrapping.cardinality()
    );

    let branches = RangeSet::<2>::from_range(10, 15).union(RangeSet::from_range(20, 30));
    println!(
        "two branch ranges: {:?}, {} values (single hull: 21 values)",
        branches.ranges(),
        branches.cardinality()
    );

    let even = Tnum::from_parts(0, u64::MAX - 1);
    let product = BitRange::<2>::new(RangeSet::from_range(8, 11), even);
    let represented: Vec<_> = (8..=11)
        .filter(|&value| product.contains_value(value))
        .collect();
    println!(
        "range [8,11] x even-known-bit fact: {:?} (range alone: 4 values)",
        represented
    );
}
