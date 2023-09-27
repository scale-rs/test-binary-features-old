use crate::output;
use std::ffi::OsStr;
use std::process::{Command, Output, Stdio};

#[derive(thiserror::Error, Debug)]
#[error("Err")]
struct Err {}

/// Return a finished [Child] and its [ExitStatus].
fn output(path: impl AsRef<OsStr>, arg: Option<impl AsRef<OsStr>>) -> Output {
    let mut command = Command::new(path);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    if let Some(arg) = arg {
        command.arg(arg);
    }
    let mut child = command.spawn().unwrap();
    let _ = child.wait().unwrap();
    child.wait_with_output().unwrap()
}

fn output_ok() -> Output {
    output("/usr/bin/echo", Some("hello"))
}

fn output_failed() -> Output {
    output("/usr/bin/cat", Some("/non/existing/file"))
}

#[test]
fn has_error() {
    let ok = output_ok();
    let ok_status = ok.status;
    assert!(!output::has_error(&Some((ok, "ok".to_owned())), &None));

    let failed = output_failed();
    let failed_status = failed.status;
    assert!(output::has_error(
        &Some((failed, "failed".to_owned())),
        &None
    ));

    let ok_outputs_but_failed_status = Some((
        Output {
            status: failed_status,
            stdout: vec![1u8],
            stderr: Vec::with_capacity(0),
        },
        "ok_outputs_but_failed_status".to_owned(),
    ));
    assert!(output::has_error(&ok_outputs_but_failed_status, &None));

    let failed_outputs_but_ok_status = Some((
        Output {
            status: ok_status,
            stdout: Vec::with_capacity(0),
            stderr: vec![1u8],
        },
        "failed_outputs_but_ok_status".to_owned(),
    ));
    assert!(output::has_error(&failed_outputs_but_ok_status, &None));

    assert!(output::has_error(&None, &Some(Box::new(Err {}))));
}
