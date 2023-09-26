use crate::indicators::{BinaryCrateName, GroupEnd, SpawningMode};
use crate::output::{DynErrResult, OptOutput, OutputAndOrError};
use crate::task;
use core::borrow::Borrow;
use core::time::Duration;
use std::collections::HashMap;
use std::io::{self, Result as IoResult, Write};
use std::process::{Child, Output};
use std::thread;

/// How long to sleep before checking again whether any child process(es) finished.
const SLEEP_BETWEEN_CHECKING_CHILDREN: Duration = Duration::from_millis(10);

/// Result of [Child]'s `id()` method. NOT a (transparent) single item struct, because we don't use
/// [u32] for anything else here.
pub type ChildId = u32;

pub type ChildInfo = String;
pub type ChildAndInfo = (Child, ChildInfo);

/// Group of active (running) Child processes.
///
/// NOT [std::collections::HashSet], because that makes referencing the items less efficient.
///
/// Keys are results of [Child]'s `id()` method.
pub type GroupOfChildren = HashMap<ChildId, ChildAndInfo>;

pub type FeaturesIterDynBox<'a, S> //where S: ?Sized,
    = Box<dyn Iterator<Item = &'a S /* feature */> + 'a>;

pub type ParallelTasksIterDyn<'a, S> = dyn Iterator<
        Item = (
            &'a S, /* subdir */
            &'a BinaryCrateName<'a, S>,
            FeaturesIterDynBox<'a, S>,
            ChildInfo,
        ),
    > + 'a;
pub type ParallelTasksIterDynBox<'a, S> = Box<ParallelTasksIterDyn<'a, S>>;

#[cfg(test)]
/// Compile time-only check that result of [parallel_tasks_from_generic] is compatible with
/// [start_parallel_tasks].
fn _parallel_tasks_from_generic_result_is_compatible() {
    if true {
        panic!("A compile-time check only.");
    }
    let mut tasks = parallel_tasks_from_generic(
        vec![(
            "some_dir",
            &BinaryCrateName::Main,
            [].into_iter(),
            "Child description here".to_owned(),
        )]
        .into_iter(),
    );
    start_parallel_tasks(&mut tasks, "", &GroupEnd::ProcessAll);
}

/// This does return a generic (`impl`) iterator itself. Then you can store it, and pass a (mutable)
/// reference to it when calling [start_parallel_tasks].
pub fn parallel_tasks_from_generic<'a, S, FEATURES, TASKS>(
    tasks: TASKS,
) -> impl Iterator<
    Item = (
        &'a S, /* subdir */
        &'a BinaryCrateName<'a, S>,
        FeaturesIterDynBox<'a, S>,
        ChildInfo,
    ),
> + 'a
where
    S: Borrow<str> + 'a + ?Sized, // for FeatureSet
    &'a S: Borrow<str>,           // for BinaryCrateName
    FEATURES: Iterator<Item = &'a S> + 'a,
    TASKS: Iterator<
            Item = (
                &'a S, /* sub_dir */
                &'a BinaryCrateName<'a, S>,
                FEATURES,
                ChildInfo,
            ),
        > + 'a,
{
    tasks.map(|(sub_dir, binary_crate_name, features, child_info)| {
        // The following fails:
        //
        // (sub_dir, binary_crate_name, Box::new(features))
        let features = Box::new(features) as FeaturesIterDynBox<'a, S>;
        (sub_dir, binary_crate_name, features, child_info)
    })
}

pub(crate) type GroupExecution = (GroupOfChildren, SpawningMode);
pub(crate) type GroupExecutionAndOptOutput = (GroupExecution, OptOutput);
pub(crate) type GroupExecutionAndOutputs = (GroupExecution, Vec<OutputAndOrError>);

/// Start a group of parallel child process(es) - tasks, all under the same `parent_dir`.
///
/// This does NOT have a [crate::indicators::SpawningMode] parameter - we behave as if under
/// [crate::indicators::SpawningMode::ProcessAll].
///
/// This does NOT check for exit status/stderr of any spawn child processes. It only checks if the
/// actual spawning itself (system call) was successful. If all spawn successfully, then the
/// [crate::indicators::SpawningMode] of the result tuple is [SpawningMode::ProcessAll]. Otherwise
/// the [crate::indicators::SpawningMode] part of the result tuple is either
/// [crate::indicators::SpawningMode::FinishActive] or [crate::indicators::SpawningMode::StopAll],
/// depending on the given `until` ([GroupEnd]).
pub(crate) fn start_parallel_tasks<'a, S>(
    mut tasks: &mut ParallelTasksIterDyn<'a, S>,
    parent_dir: &'a S,
    until: &'a GroupEnd,
) -> GroupExecutionAndOutputs
where
    S: Borrow<str> + 'a + ?Sized,
    &'a S: Borrow<str>,
{
    let mut children = GroupOfChildren::new();
    let mut spawning_mode = SpawningMode::default();
    let mut outputs = vec![];

    for (sub_dir, binary_crate, features, child_info) in &mut tasks {
        let child_or_err = task::spawn(parent_dir, sub_dir, binary_crate, features);

        match child_or_err {
            Ok(child) => {
                children.insert(child.id(), (child, child_info));
            }
            Err(err) => {
                if true {
                    panic!("Don't override the vector, but cumulate:");
                }
                spawning_mode = until.same_group_after_output_and_or_error(&None, &Some(err));
            }
        };
    }
    ((children, spawning_mode), outputs)
}

/// Iterate over the given children max. once. Take the first finished child (if any), and return
/// its process ID and exit status.
///
/// The [ChildId] is child process ID of the finished process.
///
/// Beware: [Ok] of [Some] CAN contain [ExitStatus] _NOT_ being OK!
pub(crate) fn try_finished_child(children: &mut GroupOfChildren) -> DynErrResult<Option<ChildId>> {
    for (child_id, (child, _)) in children.iter_mut() {
        let opt_status_or_err = child.try_wait();

        match opt_status_or_err {
            Ok(Some(_exit_status)) => {
                return Ok(Some(*child_id));
            }
            Ok(None) => {}
            Err(err) => return Err(Box::new(err)),
        }
    }
    Ok(None)
}

pub(crate) fn print_output(output: &Output) -> IoResult<()> {
    // If we have both non-empty stdout and stderr, print stdout first, and stderr second. That way
    // the developer is more likely to notice (and there is less vertical distance to scroll up).
    {
        let mut stdout = io::stdout().lock();
        // @TODO print process "name"
        write!(stdout, "Exit status: {}", output.status)?;
        stdout.write_all(&output.stdout)?;
    }
    {
        let mut stderr = io::stderr().lock();
        stderr.write_all(&output.stderr)?;
        if !output.stderr.is_empty() {
            stderr.flush()?;
        }
    }
    Ok(())
}

/// Return [Ok] of [None] when all children have finished (and their output has been handled
/// previously).
#[must_use]
pub fn life_cycle_step(
    (mut children, mut spawning_mode): GroupExecution,
    until: &GroupEnd,
) -> DynErrResult<Option<GroupExecutionAndOptOutput>> {
    let finished_result = try_finished_child(&mut children);
    match finished_result {
        Ok(Some(child_id)) => {
            let (child, child_info) = children.remove(&child_id).unwrap();
            let child_output = child.wait_with_output()?;
            Ok(Some((
                (children, spawning_mode),
                Some((Some((child_output, child_info)), None)),
            )))
        }
        Ok(None) => {
            if children.is_empty() {
                Ok(None)
            } else {
                Ok(Some(((children, spawning_mode), None)))
            }
        }
        Err(err) => Ok(Some(((children, spawning_mode), Some((None, Some(err)))))),
    }
}

#[must_use]
pub fn life_cycle_loop(
    (mut children, mut spawning_mode): GroupExecution,
    until: &GroupEnd,
) -> DynErrResult<()> {
    thread::sleep(SLEEP_BETWEEN_CHECKING_CHILDREN);
    panic!()
}
