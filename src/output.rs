use crate::group::ChildInfo;
use std::error::Error;
use std::process::Output;

pub type DynErr = Box<dyn Error>;
pub type DynErrResult<T> = Result<T, DynErr>;

/// For disambiguation.
pub type ProcessOutput = Output;
pub type ChildOutput = (Output, ChildInfo);
pub type ChildOutputOption = Option<ChildOutput>;
pub type DynErrOption = Option<DynErr>;

/// Collected output and/or error.
pub type OutputAndOrError = (ChildOutputOption, DynErrOption);

/// Whether the given output and/or error [Option]s indicate an error. Instead of two parameters,
/// this could accept one parameter [OutputAndOrError]. But then it would consume it, which upsets
/// ergonomics.
pub fn has_error(output_option: &ChildOutputOption, error_option: &DynErrOption) -> bool {
    error_option.is_some() || {
        matches!(output_option, Some((out, _)) if !out.status.success() || !out.stderr.is_empty())
    }
}

pub type OptOutput = Option<OutputAndOrError>;
