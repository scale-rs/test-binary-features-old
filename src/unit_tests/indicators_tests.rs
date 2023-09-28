use crate::indicators::{BinaryCrateName, SpawningMode};
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
