//#![cfg_attr(feature = "nightly", feature(can_vector, read_buf, write_all_vectored))]
//! Any `S` generic parameter is for [String]/[str] slice-like type, used for accepting names of
//! directories, files/binary crates, features...
//!
//! Any `B` generic parameter is for [BinaryCrateName]. That's separate from `S` because of
//! lifetimes and borrowing.

use core::borrow::Borrow;
use core::time::Duration;
use indicators::{BinaryCrateName, ExitStatusWrapped, GroupEnd, SequenceEnd, SpawningMode};
use std::collections::HashMap;
use std::error::Error;
use std::io::{self, Write};
use std::process::{Child, Output};
use std::thread;

mod group;
mod indicators;
#[cfg(test)]
mod lib_test;
mod task;

const INTERMEDIARY_DIR: &'static str = "testbins";

/// Based on
/// https://www.baeldung.com/linux/pipe-buffer-capacity#:~:text=In%20Linux%2C%20pipe%20buffer%20capacity,page%20size%20of%204%2C096%20bytes)
/// and https://unix.stackexchange.com/questions/11946/how-big-is-the-pipe-buffer.
const BUFFER_SIZE: usize = 16 * 4096;

/// How long to sleep before checking again whether any child process(es) finished.
const SLEEP_BETWEEN_CHECKING_CHILDREN: Duration = Duration::from_millis(50);

/// Result of [Child]'s `id()` method. NOT a (transparent) single item struct, because we don't use
/// [u32] for anything else here.
type ChildId = u32;

/// NOT [std::collections::HashSet], because that doesn't allow mutable access to items (otherwise
/// their equality and hash code could change, and HashSet's invariants wouldn't hold true anymore).
///
/// Keys are results of [Child]'s `id()` method.
///
/// We could use [Vec], but child processes get removed incrementally => O(n^2).
type GroupOfChildren = HashMap<ChildId, Child>;

type DynErr = Box<dyn Error>;
type DynErrResult<T> = Result<T, DynErr>;

pub type OutputOption = Option<Output>;
pub type DynErrOption = Option<DynErr>;

/// Collected output and/or error.
pub type OutputAndOrError = (OutputOption, DynErrOption);

/// Whether the given output and/or error [Option]s indicate an error. Instead of two parameters,
/// this could accept one parameter [OutputAndOrError]. But then it would consume it, which upsets
/// ergonomics.
pub fn has_error(output: &OutputOption, error: &DynErrOption) -> bool {
    error.is_some() || {
        matches!(output, Some(out) if !out.status.success() || !out.stderr.is_empty())
    }
}

pub struct SpawningModeAndOutputs {
    pub mode: SpawningMode,
    pub outputs: Vec<OutputAndOrError>,
}
impl Default for SpawningModeAndOutputs {
    fn default() -> Self {
        Self {
            mode: SpawningMode::ProcessAll,
            outputs: Vec::with_capacity(0),
        }
    }
}
impl SpawningModeAndOutputs {
    pub fn group_after_output_and_or_error(
        mut self,
        output: OutputOption,
        error: DynErrOption,
        group_until: &GroupEnd,
    ) -> Self {
        let has_new_error = has_error(&output, &error);
        self.outputs.push((output, error));

        let mode = if self.mode.has_error() {
            debug_assert_eq!(self.mode, group_until.mode_after_error_in_same_group());
            self.mode
        } else {
            if has_new_error {
                group_until.mode_after_error_in_same_group()
            } else {
                debug_assert_eq!(self.mode, SpawningMode::ProcessAll);
                self.mode
            }
        };
        Self {
            mode,
            outputs: self.outputs,
        }
    }
}

/// Run a group of parallel binary crate invocations. Each item (a tuple) of the group consists of
/// two fields:
/// - subdirectory, and
/// - crate feature name(s), if any.
///
/// All entries are run in parallel. It's an error if two or more entries have the same subdirectory
/// name.
pub fn run_parallel_single_tasks<'s, S, FEATURES, TASKS>(
    parent_dir: &S,
    tasks: TASKS,
    group_until: GroupEnd,
) where
    S: Borrow<str> + 's + ?Sized,
    FEATURES: IntoIterator<Item = S>,
    TASKS: IntoIterator<Item = (&'s S /*binary crate name*/, FEATURES)>,
{
}

/// Run a sequence of the same binary crate (under the same sub dir) invocation(s), but each
/// invocation with possibly different combinations of crate features.
///
/// The tasks are run in sequence, but their output may be reordered, to have any non-empty `stderr`
/// at the end.
pub fn run_sequence_single_tasks<
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

/// Run multiple sequences, where each sequence step runs a group of task(s) in parallel.
///
/// Their output may be reordered, to have any non-empty `stderr` at the end.
pub fn run_parallel_sequences_of_parallel_tasks<
    's,
    S,
    #[allow(non_camel_case_types)] FEATURE_SET,
    #[allow(non_camel_case_types)] PARALLEL_TASKS,
    #[allow(non_camel_case_types)] SEQUENCE_TASKS,
    SEQUENCES,
>(
    parent_dir: &S,
    sequences: SEQUENCES,
) where
    S: Borrow<str> + 's + ?Sized,
    FEATURE_SET: IntoIterator<Item = &'s S /* feature*/>,
    PARALLEL_TASKS: IntoIterator<Item = (&'s S /* binary crate name*/, FEATURE_SET)>,
    SEQUENCE_TASKS: IntoIterator<Item = PARALLEL_TASKS>,
    SEQUENCES: IntoIterator<Item = (GroupEnd, SequenceEnd, SEQUENCE_TASKS)>,
{
}

fn run_sub_dirs<'s, 'b, S, B>(
    parent_dir: &S,
    sub_dirs: impl IntoIterator<Item = &'s S>,
    binary_crate: BinaryCrateName<'b, B>,
) -> DynErrResult<()>
where
    S: Borrow<str> + 's + ?Sized,
    B: 'b + ?Sized,
    &'b B: Borrow<str>,
{
    let mut children = GroupOfChildren::new();
    for sub_dir in sub_dirs {
        let child_or_err = task::spawn(parent_dir, &sub_dir, &binary_crate, []);

        match child_or_err {
            Ok(child) => children.insert(child.id(), child),
            Err(err) => {
                for (_, mut other_child) in children {
                    let _ = other_child.kill();
                }
                return Err(err);
            }
        };
    }

    loop {
        let finished_result = group::finished_child(&mut children);
        match finished_result {
            Ok(Some(child_id)) => {
                let child = children.remove(&child_id).unwrap();
                let output = child.wait_with_output()?;

                // If we have both non-empty stdout and stderr, print stdout first and stderr
                // second. That way the developer is more likely to notice (and there is less
                // vertical distance to scroll up).
                let mut stdout = io::stdout().lock();
                let mut stderr = io::stderr().lock();
                stdout.write_all(&output.stdout)?;
                stderr.write_all(&output.stderr)?;
                if !output.stderr.is_empty() {
                    stderr.flush()?;
                }

                if output.status.success() && output.stderr.is_empty() {
                    continue;
                } else {
                    stderr.flush()?;
                    break Err(Box::new(ExitStatusWrapped::new(output.status)));
                }
            }
            Ok(None) => {
                if children.is_empty() {
                    break Ok(());
                } else {
                    thread::sleep(SLEEP_BETWEEN_CHECKING_CHILDREN);
                    continue;
                }
            }
            Err(err) => {
                break Err(err);
            }
        }
    }
}

#[test]
pub fn run_all_mock_combinations() -> DynErrResult<()> {
    if false {
        run_sub_dirs(
            INTERMEDIARY_DIR,
            vec!["fs_mock_entry_mock", "fs_mock_entry_real"],
            BinaryCrateName::Main,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
