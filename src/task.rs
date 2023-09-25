use crate::indicators::BinaryCrateName;
use crate::output::DynErrResult;
use core::borrow::Borrow;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use test_binary::TestBinary;

fn manifest_path_for_subdir<S>(parent_dir: &S, sub_dir: &S) -> PathBuf
where
    S: Borrow<str> + ?Sized,
{
    PathBuf::from_iter([parent_dir.borrow(), sub_dir.borrow(), "Cargo.toml"])
}

pub fn spawnNoImplIntoIter<'s, 'b, S, B, #[allow(non_camel_case_types)] FEATURES_INTO>(
    parent_dir: &S,
    sub_dir: &S,
    binary_crate: &BinaryCrateName<'b, B>,
    features: &crate::group::FeatureSetWithInto<'s, S, FEATURES_INTO>,
) where
    S: Borrow<str> + 's + ?Sized,
    B: 'b + ?Sized,
    &'b B: Borrow<str>,
{
}

pub fn spawnNoImplNoInto<'s, 'b, S, B>(
    parent_dir: &S,
    sub_dir: &S,
    binary_crate: &BinaryCrateName<'b, B>,
    features: &mut crate::group::FeaturesIterDyn<'s, S>,
) where
    S: Borrow<str> + 's + ?Sized,
    B: 'b + ?Sized,
    &'b B: Borrow<str>,
{
    for feature in features {}
}

pub fn spawn<'s, 'b, S, B>(
    parent_dir: &S,
    sub_dir: &S,
    binary_crate: &BinaryCrateName<'b, B>,
    features: impl IntoIterator<Item = &'s S>,
) -> DynErrResult<Child>
where
    S: Borrow<str> + 's + ?Sized,
    B: 'b + ?Sized,
    &'b B: Borrow<str>,
{
    let manifest_path = manifest_path_for_subdir(parent_dir, sub_dir);
    let binary_crate = binary_crate.borrow();
    let mut binary = TestBinary::relative_to_parent(binary_crate, &manifest_path);
    binary.with_profile("dev");
    for feature in features {
        binary.with_feature(feature.borrow());
    }
    // @TODO DOC if we don't paralellize the tested feature combinations fully, then apply
    // .with_feature(...) once per feature; re-build in the same folder (per the same
    // channel/sequence of run, but stop on the first error (or warning), unless configured
    // otherwise.
    match binary.build() {
        Ok(path) => {
            let mut command = Command::new(path);
            command.stdout(Stdio::piped());
            command.stderr(Stdio::piped());
            //command.env("RUST_TEST_TIME_INTEGRATION", "3600000");
            println!(
                "Starting a process under {}/ binary crate {}.",
                sub_dir.borrow(),
                binary_crate
            );
            return Ok(command.spawn()?);
        }
        Err(e) => Err(Box::new(e)),
    }
}
