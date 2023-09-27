//#![allow(unused)]
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
mod output;
mod run;
mod sequence_of_groups;
mod task;
#[cfg(test)]
mod unit_tests;
