use crate::indicators::{GroupEnd, SpawningMode};
use std::error::Error;
use std::process::Output;

pub type DynErr = Box<dyn Error>;
pub type DynErrResult<T> = Result<T, DynErr>;

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
