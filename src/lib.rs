#![allow(unused)]
#![cfg_attr(nightly, feature(implied_bounds))]
//#![cfg_attr(nightly, feature(lazy_type_alias))]
//#![cfg_attr(nightly, feature(min_specialization))]
//! Any `S` generic parameter is for [String]/[str] slice-like type, used for accepting names of
//! directories, files/binary crates, features...
//!
//! Any `B` generic parameter is for [BinaryCrateName]. That's separate from `S` because of
//! lifetimes and borrowing.

mod group;
mod group_of_sequences_of_groups;
mod indicators;
#[cfg(test)]
mod lib_test;
mod output;
#[cfg(test)]
mod output_test;
mod run;
mod sequence_of_groups;
mod task;

use core::borrow::Borrow;

trait Tr {}
struct St {}
impl Tr for St {}
fn f_dyn(t: &dyn Tr) {}

fn pass_box_as_dyn() {
    let s = St {};
    let b: Box<dyn Tr> = Box::new(s);

    f_dyn(&*b);
}

#[repr(transparent)]
pub struct IterSingleLevel<'a, S>
where
    S: Borrow<str> + 'a + ?Sized,
{
    iter_single_level: Box<dyn Iterator<Item = &'a S> + 'a>,
}
impl<'a, S, #[allow(non_camel_case_types)] ITER_SINGLE_LEVEL> From<ITER_SINGLE_LEVEL>
    for IterSingleLevel<'a, S>
where
    S: Borrow<str> + 'a + ?Sized,
    ITER_SINGLE_LEVEL: Iterator<Item = &'a S> + 'a,
{
    fn from(iter_single_level: ITER_SINGLE_LEVEL) -> Self {
        let iter_single_level = Box::new(iter_single_level);
        Self { iter_single_level }
    }
}

#[repr(transparent)]
pub struct IterTwoLevels<'a, S>
where
    S: Borrow<str> + 'a + ?Sized,
{
    iter_two_levels: Box<dyn Iterator<Item = (&'a S, IterSingleLevel<'a, S>)> + 'a>,
}
impl<'a, S, TASKS> From<TASKS> for IterTwoLevels<'a, S>
where
    S: Borrow<str> + 'a + ?Sized,
    TASKS: Iterator<Item = (&'a S, IterSingleLevel<'a, S>)> + 'a,
{
    fn from(iter_two_levels: TASKS) -> Self {
        let iter_two_levels = Box::new(iter_two_levels);
        Self { iter_two_levels }
    }
}
/*trait Tr<T> {}
struct St {}
impl Tr<u32> for St {}
impl Tr<u8> for St {}*/
#[cfg(NOT_POSSIBLE)]
impl<'a, S, ITER_SINGLE_LEVEL, ITER_TWO_LEVELS> From<ITER_TWO_LEVELS> for IterTwoLevels<'a, S>
where
    S: Borrow<str> + 'a + ?Sized, // for FeatureSet
    &'a S: Borrow<str>,           // for BinaryCrateName
    ITER_SINGLE_LEVEL: Iterator<Item = &'a S> + 'a,
    ITER_TWO_LEVELS: Iterator<Item = (&'a S, ITER_SINGLE_LEVEL)> + 'a,
{
    fn from(iter_two_levels: ITER_TWO_LEVELS) -> Self {
        let iter_two_levels = iter_two_levels.map(|(sub_dir, binary_crate_name, features)| {
            (sub_dir, binary_crate_name, features.into())
        });
        let iter_two_levels = Box::new(iter_two_levels);
        Self { iter_two_levels }
    }
}

fn from_iter_two_levels<'a, S, ITER_SINGLE_LEVEL, ITER_TWO_LEVELS>(
    iter_two_levels: ITER_TWO_LEVELS,
) -> IterTwoLevels<'a, S>
where
    S: Borrow<str> + 'a + ?Sized, // for FeatureSet
    &'a S: Borrow<str>,           // for BinaryCrateName
    ITER_SINGLE_LEVEL: Iterator<Item = &'a S> + 'a,
    ITER_TWO_LEVELS: Iterator<Item = (&'a S, ITER_SINGLE_LEVEL)> + 'a,
{
    let iter_two_levels = iter_two_levels.map(|(sub_dir, features)| (sub_dir, features.into()));
    let iter_two_levels = Box::new(iter_two_levels);
    IterTwoLevels { iter_two_levels }
}
