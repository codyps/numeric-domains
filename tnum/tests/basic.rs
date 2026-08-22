use proptest::prelude::*;
use tnum::Tnum;

proptest! {
    // NOTE: the domain of non-overflowing addition is smaller than the full range of the `u64`.
    // Minimally shrink things. This is really nasty, and there should be a simpler way to do
    // things.
    #[test]
    fn exact_addition(mut a: u64, mut b: u64) {
        /*
         while a.checked_add(b).is_none() {
            a /= 2;
        }
        */

        let tr = Tnum::from_value(a) + Tnum::from_value(b);
        let r = a.wrapping_add(b);

        assert_eq!(Some(r), tr.value());
    }

}
