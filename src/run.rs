use crate::group::{self, FeaturesIter, GroupOfChildren};
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

pub fn consume_tasks<'a, S>(tasks: crate::group::GroupParallelTasksIter<'a, S>)
where
    S: Borrow<str> + 'a + ?Sized,
    &'a S: Borrow<str>,
{
}

pub fn run_sub_dirs<'a, S>(
    parent_dir: &'a S,
    sub_dirs_and_features: impl Iterator<Item = (&'a S /*sub_dir*/, FeaturesIter<'a, S>)> + 'a,
    _sub_dirs_and_features_dyn: &'a mut dyn Iterator<
        Item = (&'a S /*sub_dir*/, FeaturesIter<'a, S>),
    >,
    binary_crate: &'a BinaryCrateName<'a, S>,
    //binary_crate: BinaryCrateName<'a, S>,
    //features: impl Iterator<Item = &'a S> + Clone + 'a,
    until: &'a GroupEnd,
) -> DynErrResult<()>
where
    S: Borrow<str> + 'a + ?Sized,
    &'a S: Borrow<str>,
{
    let mut tasks = sub_dirs_and_features.map(|(sub_dir, features)| {
        (
            sub_dir,
            /*binary_crate*/ &BinaryCrateName::Main,
            features,
        )
    });

    //consume_tasks(&mut tasks);
    //consume_tasks(&mut tasks);

    let (mut children, mut mode_and_outputs) = group::parallel_tasks(parent_dir, &mut tasks, until);
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
