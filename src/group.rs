use crate::indicators::{BinaryCrateName, GroupEnd, SpawningMode};
use crate::output::{DynErr, DynErrResult, OptOutput, OutputAndOrError};
use crate::task;
use core::borrow::Borrow;
use core::time::Duration;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::io::{self, Result as IoResult, Write};
use std::process::{Child, Output};
use std::thread;

/// How long to sleep before checking again whether any child process(es) finished.
const SLEEP_BETWEEN_CHECKING_CHILDREN: Duration = Duration::from_millis(10);

/// Result of [Child]'s `id()` method. NOT a (transparent) single item struct, because we don't use
/// [u32] for anything else here.
pub type ChildId = u32;

pub type ChildInfo = String;
pub type ChildInfoMeta = (Child, ChildInfo);

/// Group of active (running) Child processes.
///
/// NOT [std::collections::HashSet], because that makes referencing the items less efficient.
///
/// Keys are results of [Child]'s `id()` method.
pub type GroupOfChildren = HashMap<ChildId, ChildInfoMeta>;

pub type Features<'a, S> //where S: ?Sized,
    = Vec<&'a S /* feature */>;

pub type ParallelTasks<'a, S> = Vec<(
    &'a S, /* subdir */
    &'a BinaryCrateName<'a, S>,
    Features<'a, S>,
    ChildInfo,
)>;

pub(crate) type GroupExecution = (GroupOfChildren, SpawningMode);
pub(crate) type GroupExecutionAndOptOutput = (GroupExecution, OptOutput);
pub(crate) type GroupExecutionAndStartErrors = (GroupExecution, Vec<DynErr>);

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
pub fn start_parallel_tasks<'a, S>(
    mut tasks: ParallelTasks<'a, S>,
    parent_dir: &'a S,
    until: &'a GroupEnd,
) -> GroupExecutionAndStartErrors
//@TODO change the output type to be only a Vec<DynErr>.
where
    S: Borrow<str> + 'a + ?Sized,
    &'a S: Borrow<str>,
{
    let mut children = GroupOfChildren::new();
    let mut spawning_mode = SpawningMode::default();
    let mut errors = Vec::with_capacity(0);

    for (sub_dir, binary_crate, features, child_info) in tasks {
        let child_or_err = task::spawn(parent_dir, sub_dir, binary_crate, &features);

        match child_or_err {
            Ok(child) => {
                children.insert(child.id(), (child, child_info));
            }
            Err(err) => {
                spawning_mode = until.mode_after_error_in_same_group();
                errors.push(err);
            }
        };
    }
    ((children, spawning_mode), errors)
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

/// Return [Some] if any child has finished; return [None] when all children have finished. This
/// does NOT modify [SpawningMode] part of the result [GroupExecutionAndOptOutput].
#[must_use]
pub fn collect_finished_child(
    (mut children, spawning_mode): GroupExecution,
) -> Option<GroupExecutionAndOptOutput> {
    let finished_result = try_finished_child(&mut children);
    match finished_result {
        Ok(Some(child_id)) => {
            let (child, child_info) = children.remove(&child_id).unwrap();
            let (child_output, err) = match child.wait_with_output() {
                Ok(child_output) => (Some(child_output), None),
                Err(err) => (
                    None,
                    Some({
                        let err: Box<dyn StdError> = Box::new(err);
                        err
                    }),
                ),
            };
            Some((
                (children, spawning_mode),
                Some((Some((child_output, child_info)), err)),
            ))
        }
        Ok(None) => {
            if children.is_empty() {
                None
            } else {
                Some(((children, spawning_mode), None))
            }
        }
        Err(err) => Some(((children, spawning_mode), Some((None, Some(err))))),
    }
}

#[must_use]
pub fn life_cycle_step(
    (mut _children, mut _spawning_mode): GroupExecution,
    _until: &GroupEnd,
) -> DynErrResult<()> {
    thread::sleep(SLEEP_BETWEEN_CHECKING_CHILDREN);
    // @TODO kill
    panic!()
}
