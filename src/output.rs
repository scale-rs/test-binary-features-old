use crate::group::ChildInfo;
use std::error::Error;
use std::process::Output;

pub type DynErr = Box<dyn Error>;
pub type DynErrResult<T> = Result<T, DynErr>;

/// For disambiguation.
pub type ProcessOutput = Output;

/// [Output] part mey not be present, if [std::process::Child::wait_with_output] failed.
pub type ChildOutput<M> = (Option<Output>, ChildInfo, M);
pub type ChildOutputOption<M> = Option<ChildOutput<M>>;
pub type DynErrOption = Option<DynErr>;

/// Collected output and/or error.
pub type OutputAndOrError<M> = (ChildOutputOption<M>, DynErrOption);

/// Whether the given output and/or error [Option]s indicate an error. Instead of two parameters,
/// this could accept one parameter [OutputAndOrError]. But then it would consume it, which upsets
/// ergonomics.
pub fn has_error<M>(output_option: &ChildOutputOption<M>, error_option: &DynErrOption) -> bool {
    error_option.is_some()
        || {
            matches!(output_option, Some((Some(out), _, _)) if !out.status.success() || !out.stderr.is_empty())
        }
}

pub type OptOutput<M> = Option<OutputAndOrError<M>>;
