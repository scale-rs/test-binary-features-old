//#![cfg_attr(nightly, feature(lazy_type_alias))]
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
