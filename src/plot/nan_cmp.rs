//! NaN-safe float ordering for sort comparators.

/// A total order on `f64` (and `f32`) for sort comparators: numbers compare as usual
/// (`-0.0 == 0.0`, unlike `f64::total_cmp`), NaN sorts above every number,
/// and NaNs are equal to each other.
///
/// `partial_cmp(..).unwrap_or(Equal)` is not a total order once a key is NaN
/// (NaN "equals" both 1 and 2), and the std sorts may panic on such a
/// comparator; `partial_cmp(..).unwrap()` panics outright. Use `a.nan_cmp(&b)`
/// instead, and `b.nan_cmp(&a)` for descending.
pub(crate) trait NanCmp {
    fn nan_cmp(&self, other: &Self) -> std::cmp::Ordering;
}

macro_rules! impl_nan_cmp {
    ($($t:ty),*) => {$(
        impl NanCmp for $t {
            #[inline]
            fn nan_cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.partial_cmp(other)
                    .unwrap_or_else(|| self.is_nan().cmp(&other.is_nan()))
            }
        }
    )*};
}
impl_nan_cmp!(f64, f32);
