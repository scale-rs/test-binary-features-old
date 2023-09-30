use crate::indicators::{BinaryCrateName, GroupEnd, SpawningMode};
use crate::output::{DynErr, DynErrResult, OptOutput, ProcessOutput};
use crate::task;
use core::borrow::Borrow;
use core::ops::{Deref, DerefMut};
use core::time::Duration;
use phantom_newtype::Id;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::io::{self, Result as IoResult, Write};
use std::process::Child;
use std::thread;

/// How long to sleep before checking again whether any child process(es) finished.
const SLEEP_BETWEEN_CHECKING_CHILDREN: Duration = Duration::from_millis(10);

/// For disambiguation.
pub type ChildProcess = Child;

/// Result of [Child]'s `id()` method, wrapped.
//#[repr(transparent)]
//pub sttrans ChildProcessId(pub u32);
pub type ChildProcessId = Id<ChildProcess, u32>;

pub type ChildInfo = String;

/// Having a struct with (one) named field (for example, `its`), instead of a (one field) tuple
/// struct, could make some code "nicer".
/// ```rust
/// #[repr(transparent)]
/// pub struct ChildInfoMeta<M> {
///     /* Not called `it`, so as not to confuse it with iterators.*/
///     pub its: ChildInfoMetaTuple<M>,
/// }
/// ```
/// However, that would be useful mostly for either
/// - accessing the (only) field, but that we can also dereference with asterisk (through [Deref]
///   and [DerefMut]). Or
/// - accessing the anonymous/positional field(s) of the underlying tuple. But for that we have our
///   named accessor methods, so we don't need to access it through/by specifying the wrapped tuple
///   itself. So we don't need any of that much.
///
/// Instead, the consumers benefit more from easily destructuring our wrappers struct into its
/// (only) positional/anonymous field, especially so they destructure the underlying tuple into its
/// fields (and give them names as local variables).
///
/// Why anonymous tuples (with nameless fields)? Brevity of positional constructor. And pattern matching.
pub struct ChildInfoMeta<M>(ChildProcess, ChildInfo, M);
impl<M> ChildInfoMeta<M> {
    /// Useful if we don't want to publish the wrapped field.
    /*pub fn new(process: ChildProcess, info: ChildInfo, meta: M) -> Self {
        Self((process, info, meta))
    }*/

    pub fn child(&self) -> &ChildProcess {
        &self.0
    }
    pub fn info(&self) -> &ChildInfo {
        &self.1
    }
    /// Only for non-[Copy], or if [Copy] but deemed large.
    ///
    /// If [Copy] and primitive, then this function doesn't return a reference, but the field value.
    pub fn meta(&self) -> &M {
        &self.2
    }

    // For [Copy] but deemed large.
    //
    // pub fn meta_copy(&self) -> M
    //
    // Suggest not to have `meta_ref(&self) ->&M for a reference to (deemed large) M. Why? Because
    // sharing & accessing by references is preferred to copying the data (wherever possible). So we
    // make this preferred method name a short one.

    pub fn meta_mut(&mut self) -> &mut M {
        &mut self.2
    }
}

/// Only for [Copy], and only if NOT primitive.
///
/// Rename to `meta_copy()` for non-primitive types, so that its cost would be evident everywhere
/// it's used.
impl<M> ChildInfoMeta<M>
where
    M: Copy,
{
    pub fn meta_copy(&self) -> M {
        self.2
    }
}

// @TODO From/Into

/// Implementing [Deref] and [DerefMut] doesn't improve ergonomics for accessing the tuple's field
/// by numeric index. For that we have to put the dereferencing asterisk and the struct instance
/// into parenthesis, e.g.
/// ```rust
/// let wrap: ChildInfoMetaWrap = ...;
/// meta = (*wrap).2;
/// ```
/// So we may just as well use
/// ```rust
/// let wrap: ChildInfoMetaWrap = ...;
/// meta = wrap.0.2;
/// ```
/// - a little easier to type and read. However, we really want to use our named accessor methods.
///
/// But dereferencing with asterisk is ergonomic when we need to cast the struct back to the tuple
/// type.

/// Group of active (running) Child processes.
///
/// NOT [std::collections::HashSet], because that makes referencing the items less efficient.
///
/// Keys are results of [ChildProcess]'s `id()` method.
pub type GroupOfChildren<M> = HashMap<ChildProcessId, ChildInfoMeta<M>>;

pub type Features<'a, S> //where S: ?Sized,
    = Vec<&'a S /* feature */>;

pub type ParallelTasks<'a, S, M> = Vec<(
    &'a S, /* subdir */
    &'a BinaryCrateName<'a, S>,
    Features<'a, S>,
    ChildInfo,
    M,
)>;

pub(crate) type GroupExecution<M> = (GroupOfChildren<M>, SpawningMode);
pub(crate) type GroupOfChildrenAndOptOutput<M> = (GroupOfChildren<M>, OptOutput<M>);
pub(crate) type GroupExecutionAndStartErrors<M> = (GroupExecution<M>, Vec<DynErr>);

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
pub fn start_parallel_tasks<'a, S, M>(
    tasks: ParallelTasks<'a, S, M>,
    parent_dir: &'a S,
    until: &'a GroupEnd,
) -> GroupExecutionAndStartErrors<M>
//@TODO change the output type to be only a Vec<DynErr>.
where
    S: Borrow<str> + 'a + ?Sized,
    &'a S: Borrow<str>,
{
    let mut children = GroupOfChildren::new();
    let mut spawning_mode = SpawningMode::default();
    let mut errors = Vec::with_capacity(0);

    for (sub_dir, binary_crate, features, child_info, meta) in tasks {
        let child_or_err = task::spawn(parent_dir, sub_dir, binary_crate, &features);

        match child_or_err {
            Ok(child) => {
                children.insert(child.id().into(), ChildInfoMeta(child, child_info, meta));
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
/// Beware: [Ok] of [Some] actually CAN contain [ExitStatus] _NOT_ being OK!
pub(crate) fn try_finished_child<M>(
    children: &mut GroupOfChildren<M>,
) -> DynErrResult<Option<ChildProcessId>> {
    for (child_id, ChildInfoMeta(child, _, _)) in children.iter_mut() {
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

pub(crate) fn print_output(output: &ProcessOutput) -> IoResult<()> {
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
pub fn collect_finished_child<M>(
    mut children: GroupOfChildren<M>,
) -> Option<GroupOfChildrenAndOptOutput<M>> {
    let finished_result = try_finished_child(&mut children);
    match finished_result {
        Ok(Some(child_id)) => {
            let ChildInfoMeta(child, child_info, meta) = children.remove(&child_id).unwrap();
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
                children,
                Some((Some((child_output, child_info, meta)), err)),
            ))
        }
        Ok(None) => {
            if children.is_empty() {
                None
            } else {
                Some((children, None))
            }
        }
        Err(err) => Some((children, Some((None, Some(err))))),
    }
}

#[must_use]
pub fn life_cycle_step<M>(
    (mut _children, mut _spawning_mode): GroupExecution<M>,
    _until: &GroupEnd,
) -> DynErrResult<()> {
    /*match collect_finished_child(children) {

    }*/
    thread::sleep(SLEEP_BETWEEN_CHECKING_CHILDREN);
    // @TODO kill
    panic!()
}

#[must_use]
pub fn life_cycle_loop<M>(
    (mut _children, mut _spawning_mode): GroupExecution<M>,
    _until: &GroupEnd,
) -> DynErrResult<()> {
    loop {
        panic!();
    }
}
