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

pub type FeatureSetWithInto<'s, S, INTO> //where S: ?Sized,
    = dyn IntoIterator<Item = &'s S /* feature */, IntoIter = INTO>;

pub type GroupParallelTasksWithInto<'s, S, #[allow(non_camel_case_types)] FEATURE_SET, INTO> =
    dyn IntoIterator<
        Item = (
            &'s S, /* subdir */
            &'s BinaryCrateName<'s, S>,
            FEATURE_SET,
        ),
        IntoIter = INTO,
    >;
pub type ParallelTasksNoIntoButFeaturesetGeneric<
    's,
    S,
    #[allow(non_camel_case_types)] FEATURE_SET,
> = dyn Iterator<
    Item = (
        &'s S, /* subdir */
        &'s BinaryCrateName<'s, S>,
        FEATURE_SET,
    ),
>;
//-----

pub type FeaturesIterDyn<'a, S> //where S: ?Sized,
    = &'a mut dyn Iterator<Item = &'a S /* feature */>;

pub type GroupParallelTasksIterDyn<'a, S> = &'a mut dyn Iterator<
    Item = (
        &'a S, /* subdir */
        &'a BinaryCrateName<'a, S>,
        FeaturesIterDyn<'a, S>,
    ),
>;
//-----

struct T {}
impl<F> From<F> for T
where
    F: Iterator<Item = char>,
{
    fn from(value: F) -> Self {
        Self {}
    }
}

fn transform() {
    if true {
        let t: T = ['x'; 1].iter().cloned().into();

        //let t = (Box::new(['x'; 1].iter().cloned()) as Box<Into<T>>).into();
    }
}
//-----

#[repr(transparent)]
pub struct FeaturesGEN<'a, S, #[allow(non_camel_case_types)] FEATURES>
where
    S: Borrow<str> + 'a + ?Sized,
    FEATURES: Iterator<Item = &'a S>,
{
    features: FEATURES,
}
impl<'a, S, #[allow(non_camel_case_types)] FEATURES> From<FEATURES> for FeaturesGEN<'a, S, FEATURES>
where
    S: Borrow<str> + 'a + ?Sized,
    FEATURES: Iterator<Item = &'a S>,
{
    fn from(features: FEATURES) -> Self {
        Self { features }
    }
}

#[repr(transparent)]
pub struct Features<'a, S>
where
    S: Borrow<str> + 'a + ?Sized,
{
    features: Box<dyn Iterator<Item = &'a S> + 'a>,
}
impl<'a, S, FEATURES> From<FEATURES> for Features<'a, S>
where
    S: Borrow<str> + 'a + ?Sized,
    FEATURES: Iterator<Item = &'a S> + 'a,
{
    fn from(features: FEATURES) -> Self {
        Self {
            features: Box::new(features),
        }
    }
}

#[repr(transparent)]
pub struct GroupParallelTasks<'a, S>
where
    S: Borrow<str> + 'a + ?Sized, // for Features
    &'a S: Borrow<str>,           // for BinaryCrateName
{
    group_tasks: Box<
        dyn Iterator<
                Item = (
                    &'a S, /* sub_dir */
                    &'a BinaryCrateName<'a, S>,
                    Features<'a, S>,
                ),
            > + 'a,
    >,
}
impl<'a, S, TASKS> From<TASKS> for GroupParallelTasks<'a, S>
where
    S: Borrow<str> + 'a + ?Sized, // for FeatureSet
    &'a S: Borrow<str>,           // for BinaryCrateName
    TASKS: Iterator<
            Item = (
                &'a S, /* sub_dir */
                &'a BinaryCrateName<'a, S>,
                Features<'a, S>,
            ),
        > + 'a,
{
    fn from(group_tasks: TASKS) -> Self {
        Self {
            group_tasks: Box::new(group_tasks),
        }
    }
}
/*trait Tr<T> {}
struct St {}
impl Tr<u32> for St {}
impl Tr<u8> for St {}*/
#[cfg(NOT_POSSIBLE)]
impl<'a, S, FEATURES, TASKS> From<TASKS> for GroupParallelTasks<'a, S>
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
    fn from(group_tasks: TASKS) -> Self {
        let group_tasks = group_tasks.map(|(sub_dir, binary_crate_name, features)| {
            (sub_dir, binary_crate_name, features.into())
        });
        let group_tasks = Box::new(group_tasks);
        Self { group_tasks }
    }
}

#[repr(transparent)]
pub struct GroupParallelTasksGEN<'a, S, FEATURES, TASKS>
where
    S: Borrow<str> + 'a + ?Sized, // for Features
    &'a S: Borrow<str>,           // for BinaryCrateName
    FEATURES: Iterator<Item = &'a S>,
    TASKS: Iterator<
        Item = (
            &'a S, /* sub_dir */
            &'a BinaryCrateName<'a, S>,
            FeaturesGEN<'a, S, FEATURES>,
        ),
    >,
{
    group_tasks: TASKS,
}
impl<'a, S, FEATURES, TASKS> From<TASKS> for GroupParallelTasksGEN<'a, S, FEATURES, TASKS>
where
    S: Borrow<str> + 'a + ?Sized, // for FeatureSet
    &'a S: Borrow<str>,           // for BinaryCrateName
    FEATURES: Iterator<Item = &'a S>,
    TASKS: Iterator<
        Item = (
            &'a S, /* sub_dir */
            &'a BinaryCrateName<'a, S>,
            FeaturesGEN<'a, S, FEATURES>,
        ),
    >,
{
    fn from(group_tasks: TASKS) -> Self {
        Self { group_tasks }
    }
}

impl<'a, S, FEATURES, TASKS> GroupParallelTasksGEN<'a, S, FEATURES, TASKS>
where
    S: Borrow<str> + 'a + ?Sized, // for FeatureSet
    &'a S: Borrow<str>,           // for BinaryCrateName
    FEATURES: Iterator<Item = &'a S>,
    TASKS: Iterator<
        Item = (
            &'a S, /* sub_dir */
            &'a BinaryCrateName<'a, S>,
            FeaturesGEN<'a, S, FEATURES>,
        ),
    >,
{
    pub fn start(
        mut self,
        parent_dir: &'a S,
        until: &'a GroupEnd,
    ) -> (GroupOfChildren, SpawningModeAndOutputs) {
        let mut children = GroupOfChildren::new();
        let mut mode_and_outputs = SpawningModeAndOutputs::default();

        for (sub_dir, binary_crate, features) in self.group_tasks {
            let child_or_err = task::spawn(parent_dir, sub_dir, binary_crate, features.features);

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
}

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
pub fn parallel_tasks<'a, S>(
    parent_dir: &'a S,
    tasks: GroupParallelTasksIterDyn<'a, S>,
    until: &'a GroupEnd,
) -> (GroupOfChildren, SpawningModeAndOutputs)
where
    S: Borrow<str> + 'a + ?Sized,
    &'a S: Borrow<str>,
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

pub fn parallel_tasks_OLD<
    'a,
    S,
    #[allow(non_camel_case_types)] FEATURE_SET,
    #[allow(non_camel_case_types)] PARALLEL_TASKS,
>(
    parent_dir: &'a S,
    tasks: PARALLEL_TASKS,
    until: &GroupEnd,
) -> (GroupOfChildren, SpawningModeAndOutputs)
where
    S: Borrow<str> + 'a + ?Sized, // for FeatureSet
    &'a S: Borrow<str>,           // for BinaryCrateName
    FEATURE_SET: IntoIterator<Item = &'a S /* feature */>,
    PARALLEL_TASKS: IntoIterator<
        Item = (
            &'a S, /* sub_dir */
            &'a BinaryCrateName<'a, S>,
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
