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

pub type FeaturesIterDynBox<'a, S> //where S: ?Sized,
    = Box<dyn Iterator<Item = &'a S /* feature */> + 'a>;

pub type ParallelTasksIterDyn<'a, S> = dyn Iterator<
        Item = (
            &'a S, /* subdir */
            &'a BinaryCrateName<'a, S>,
            FeaturesIterDynBox<'a, S>,
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
        vec![("some_dir", &BinaryCrateName::Main, [].into_iter())].into_iter(),
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
            ),
        > + 'a,
{
    tasks.map(|(sub_dir, binary_crate_name, features)| {
        // The following fails:
        //
        // (sub_dir, binary_crate_name, Box::new(features))
        let features = Box::new(features) as FeaturesIterDynBox<'a, S>;
        (sub_dir, binary_crate_name, features)
    })
}

pub(crate) type GroupExecutionAndOutputs = (GroupOfChildren, SpawningModeAndOutputs);
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
    let mut mode_and_outputs = SpawningModeAndOutputs::default();

    for (sub_dir, binary_crate, features) in &mut tasks {
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
pub(crate) fn try_finished_child(children: &mut GroupOfChildren) -> DynErrResult<Option<ChildId>> {
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

pub(crate) fn life_cycle_step(
    (mut children, mut mode_and_outputs): GroupExecutionAndOutputs,
    until: &GroupEnd,
) -> GroupExecutionAndOutputs {
    panic!()
}
