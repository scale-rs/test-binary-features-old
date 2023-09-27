use crate::indicators::GroupEnd;
use core::borrow::Borrow;

/// Run a sequence of the same binary crate (under the same sub dir) invocation(s), but each
/// invocation with possibly different combinations of crate features.
///
/// The tasks are run in sequence, but their output may be reordered, to have any non-empty `stderr`
/// at the end.
pub fn sequence_single_tasks<
    's,
    S,
    #[allow(non_camel_case_types)] FEATURE_SET,
    #[allow(non_camel_case_types)] FEATURE_SETS,
>(
    parent_dir: &S,
    sub_dir: &S,
    feature_sets: FEATURE_SETS,
    group_until: GroupEnd,
) where
    S: Borrow<str> + 's + ?Sized,
    FEATURE_SET: IntoIterator<Item = &'s S>,
    FEATURE_SETS: IntoIterator<Item = FEATURE_SET>,
{
}
