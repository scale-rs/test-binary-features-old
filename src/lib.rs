//#![cfg_attr(feature = "nightly", feature(can_vector, read_buf, write_all_vectored))]
//! Any `S` generic parameter is for [String]/[str] slice-like type, used for accepting names of
//! directories, files/binary crates, features...
//!
//! Any `B` generic parameter is for [BinaryCrateName]. That's separate from `S` because of
//! lifetimes and borrowing.

mod group;
mod indicators;
#[cfg(test)]
mod lib_test;
mod output;
#[cfg(test)]
mod output_test;
mod run;
mod task;
