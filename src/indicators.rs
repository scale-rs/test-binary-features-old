use crate::output::{self, ChildOutputOption, DynErrOption};
use core::borrow::Borrow;

pub enum BinaryCrateName<'b, B>
where
    B: 'b + ?Sized,
    &'b B: Borrow<str>,
{
    /// The binary (executable) name is the same as `[package]` name in `Cargo.toml`. (That's the
    /// default binary crate, and its source code is (by
    /// default/[auto-discovery](https://doc.rust-lang.org/nightly/cargo/reference/cargo-targets.html#target-auto-discovery)))
    /// in `src/main.rs`.)
    Main,
    /// Non-default binary name, whose source code is (by default) under
    /// [`src/bin/`](https://doc.rust-lang.org/nightly/cargo/reference/cargo-targets.html#binaries).
    /// The binary (executable) name is (by default/
    /// [auto-discovery](https://doc.rust-lang.org/nightly/cargo/reference/cargo-targets.html#target-auto-discovery))
    /// the same as its source file name (excluding `.rs`; add `.exe` on Windows).
    Other(&'b B),
}
impl<'b, B> BinaryCrateName<'b, B>
where
    B: 'b + ?Sized,
    &'b B: Borrow<str>,
{
    pub fn borrow(&self) -> &str {
        match self {
            Self::Main => "main",
            Self::Other(o) => o.borrow(),
        }
    }
}

/// Indicate when to end an execution of parallel tasks in the same group, or a sequence of groups.
pub enum GroupEnd {
    /// Stop any and all active tasks on first failure. Stop them without reporting any output from
    /// them (except for the failed task). Don't start any subsequent task(s).
    OnFailureStopAll,
    /// On failure of any tasks that have already started, wait until all other parallel tasks
    /// finish, too. Report output from all of them. Potentially reorder their outputs, so that
    /// outputs of any failed task(s) will be at the end. Don't start any subsequent tasks.
    OnFailureFinishActive,
    /// Run all group(s) and all task(s) in each group. Wait for all of them, even if any of them
    /// fail.
    ProcessAll,
}

impl GroupEnd {
    pub fn mode_after_error_in_same_group(&self) -> SpawningMode {
        match self {
            Self::OnFailureStopAll => SpawningMode::StopAll,
            Self::OnFailureFinishActive => SpawningMode::FinishActive,
            Self::ProcessAll => SpawningMode::ProcessAll,
        }
    }
}

pub enum SequenceEnd {
    /// On success of this group continue the sequence (any successive groups in this sequence),
    /// even if any other parallel sequence(s) have failed.
    ContinueRegardlessOfOthers,
    /// If any other sequence fails, stop this one, too. (Then follow this sequence's [GroupEnd].)
    StopOnOthersFailure,
}

/// Mode of handling task life cycle.
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum SpawningMode {
    /// Default (until there is any error, or until we finish all tasks).
    ProcessAll,
    /// Finish active tasks, collect their output. Don't start any new ones.
    FinishActive,
    /// Stop any and all active tasks. Ignore their output (except for the task that has failed and
    /// that triggered this mode).
    StopAll,
}

impl SpawningMode {
    pub fn has_error(&self) -> bool {
        self != &Self::ProcessAll
    }
}
impl Default for SpawningMode {
    fn default() -> Self {
        Self::ProcessAll
    }
}
impl SpawningMode {
    pub fn after_output_and_or_error<M>(
        self,
        output: &ChildOutputOption<M>,
        error: &DynErrOption,
        group_until: &GroupEnd,
    ) -> Self {
        if self.has_error() {
            debug_assert_eq!(self, group_until.mode_after_error_in_same_group());
            self
        } else {
            if output::has_error(&output, &error) {
                group_until.mode_after_error_in_same_group()
            } else {
                debug_assert_eq!(self, SpawningMode::ProcessAll);
                self
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn binary_crate_name_borrow() {
        assert_eq!(BinaryCrateName::Main.borrow(), "main");
        assert_eq!(
            BinaryCrateName::Other("other_binary").borrow(),
            "other_binary"
        );
    }

    #[test]
    fn spawning_mode_has_error() {
        assert!(!SpawningMode::ProcessAll.has_error());
        assert!(SpawningMode::FinishActive.has_error());
        assert!(SpawningMode::StopAll.has_error());
    }
}
