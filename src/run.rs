use crate::group::{self, GroupOfChildren};
use crate::indicators::{BinaryCrateName, ExitStatusWrapped, GroupEnd, SequenceEnd};
use crate::output::DynErrResult;
use crate::task;
use core::borrow::Borrow;
use core::time::Duration;
use std::io::{self, Write};
use std::thread;

/// How long to sleep before checking again whether any child process(es) finished.
const SLEEP_BETWEEN_CHECKING_CHILDREN: Duration = Duration::from_millis(10);

/// Run a group of parallel binary crate invocations. Each item (a tuple) of the group consists of
/// two fields:
/// - subdirectory, and
/// - crate feature name(s), if any.
///
/// All entries are run in parallel. It's an error if two or more entries have the same subdirectory
/// name.
pub fn parallel_single_tasks<'s, S, FEATURES, TASKS>(
    parent_dir: &S,
    tasks: TASKS,
    group_until: GroupEnd,
) where
    S: Borrow<str> + 's + ?Sized,
    FEATURES: IntoIterator<Item = S>,
    TASKS: IntoIterator<Item = (&'s S /*binary crate name*/, FEATURES)>,
{
}

/// Run multiple sequences, where each sequence step runs a group of task(s) in parallel.
///
/// Their output may be reordered, to have any non-empty `stderr` at the end.
pub fn parallel_sequences_of_parallel_tasks<
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

pub fn run_sub_dirs<'s, 'b, S, B>(
    parent_dir: &S,
    sub_dirs: impl IntoIterator<Item = &'s S>,
    binary_crate: BinaryCrateName<'b, B>,
    until: &GroupEnd,
) -> DynErrResult<()>
where
    S: Borrow<str> + 's + ?Sized,
    B: 'b + ?Sized,
    &'b B: Borrow<str>,
{
    let tasks = sub_dirs
        .into_iter()
        .map(|sub_dir| (sub_dir, &BinaryCrateName::Main, []));
    let (mut children, mut mode_and_outpus) = group::start(parent_dir, tasks, until);

    loop {
        let finished_result = group::try_finished_child(&mut children);
        match finished_result {
            Ok(Some(child_id)) => {
                let child = children.remove(&child_id).unwrap();
                let output = child.wait_with_output()?;

                // If we have both non-empty stdout and stderr, print stdout first and stderr
                // second. That way the developer is more likely to notice (and there is less
                // vertical distance to scroll up).
                {
                    let mut stdout = io::stdout().lock();
                    stdout.write_all(&output.stdout)?;
                }
                {
                    let mut stderr = io::stderr().lock();
                    stderr.write_all(&output.stderr)?;
                    if !output.stderr.is_empty() {
                        stderr.flush()?;
                    }
                }

                if output.status.success() && output.stderr.is_empty() {
                    continue;
                } else {
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
