use crate::indicators::{BinaryCrateName, GroupEnd};
use crate::output::{DynErrResult, SpawningModeAndOutputs};
use crate::task;
use core::borrow::Borrow;
use std::collections::HashMap;
use std::process::Child;

/// Result of [Child]'s `id()` method. NOT a (transparent) single item struct, because we don't use
/// [u32] for anything else here.
pub type ChildId = u32;

/// Group of active (running) Child processes.
///
/// NOT [std::collections::HashSet], because that makes referencing the items less efficient.
///
/// Keys are results of [Child]'s `id()` method.
pub type GroupOfChildren = HashMap<ChildId, Child>;

// @TODO consider generic types:
// TASKS<S> = ... &dyn Iterator<...>
//
// --- if references, then we do NOT need Box<&dyn ...>, just &dyn ...
/// S: Borrow<str> + 's + ?Sized
//pub type FeatureSet<'s, S> = IntoIterator<Item = /*&(dyn S + 's)*/&'s dyn S>;

pub type FeatureSet<'s, S, INTO> //where S: ?Sized,
    = dyn IntoIterator<Item = &'s S /* feature */, IntoIter = INTO>;

pub type ParallelTasks<'s, 'b, S, B, #[allow(non_camel_case_types)] FEATURE_SET, INTO> =
    dyn IntoIterator<
        Item = (
            &'s S, /* subdir */
            &'b BinaryCrateName<'b, B>,
            FEATURE_SET,
        ),
        IntoIter = INTO,
    >;

#[repr(transparent)]
pub struct WrapperWithStringyBound<'s, T, S>
where
    S: Borrow<str> + 's + ?Sized,
{
    wrapped: T,
    _phantom: std::marker::PhantomData<&'s S>,
}

pub fn parallel_tasks<
    's,
    'b,
    S,
    B,
    #[allow(non_camel_case_types)] FEATURE_SET_INTO,
    #[allow(non_camel_case_types)] PARALLEL_TASKS_INTO,
>(
    parent_dir: &S,
    tasks: &ParallelTasks<'s, 'b, S, B, FeatureSet<'s, S, FEATURE_SET_INTO>, PARALLEL_TASKS_INTO>,
    until: GroupEnd,
) where
    S: Borrow<str> + 's + ?Sized,
    B: 'b + ?Sized,
    &'b B: Borrow<str>,
{
}

/// Start a group of parallel child process(es) - tasks, all under the same `parent_dir`.
///
/// This does NOT have a [SpawningMode] parameter - we behave as if under
/// [SpawningMode::ProcessAll].
///
/// This does NOT check for exit status/stderr of any spawn child processes. It only checks if the
/// actual spawning itself (system call) was successful. If all spawn successfully, then the
/// [SpawningMode] of the result tuple is [SpawningMode::ProcessAll]. Otherwise the [SpawningMode]
/// part of the result tuple is either [SpawningMode::FinishActive] or [SpawningMode::StopAll],
/// depending on the given `until` ([GroupEnd]).
pub fn start<
    's,
    'b,
    S,
    B,
    #[allow(non_camel_case_types)] FEATURE_SET,
    #[allow(non_camel_case_types)] PARALLEL_TASKS,
>(
    parent_dir: &S,
    tasks: PARALLEL_TASKS,
    until: &GroupEnd,
) -> (GroupOfChildren, SpawningModeAndOutputs)
where
    S: Borrow<str> + 's + ?Sized,
    B: 'b + ?Sized,
    &'b B: Borrow<str>,
    FEATURE_SET: IntoIterator<Item = &'s S /* feature */>,
    PARALLEL_TASKS: IntoIterator<
        Item = (
            &'s S, /* sub_dir */
            &'b BinaryCrateName<'b, B>,
            FEATURE_SET,
        ),
    >,
{
    let mut children = GroupOfChildren::new();
    let mut mode_and_outputs = SpawningModeAndOutputs::default();

    for (sub_dir, binary_crate, features) in tasks {
        let child_or_err = task::spawn(parent_dir, sub_dir, binary_crate, features);

        match child_or_err {
            Ok(child) => {
                children.insert(child.id(), child);
            }
            Err(err) => {
                mode_and_outputs = until.same_group_after_output_and_or_error(None, Some(err));
            }
        };
    }
    (children, mode_and_outputs)
}

/// Iterate over the given children max. once. Take the first finished child (if any), and return
/// its process ID and exit status.
///
/// The [ChildId] is child process ID of the finished process.
///
/// Beware: [Ok] of [Some] CAN contain [ExitStatus] _NOT_ being OK!
pub fn try_finished_child(children: &mut GroupOfChildren) -> DynErrResult<Option<ChildId>> {
    for (child_id, child) in children.iter_mut() {
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

pub fn life_cycle_step(
    group: GroupOfChildren,
    mode_and_outputs: SpawningModeAndOutputs,
    until: GroupEnd,
) -> (GroupOfChildren, SpawningModeAndOutputs) {
    panic!()
}
