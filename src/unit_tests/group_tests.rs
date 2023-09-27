use crate::{
    group,
    indicators::{BinaryCrateName, GroupEnd},
};

/// Compile time-only check that result of [parallel_tasks_from_generic] is compatible with
/// [start_parallel_tasks].
fn _parallel_tasks_from_generic_result_is_compatible() {
    if true {
        panic!("A compile-time check only.");
    }
    let mut tasks = group::parallel_tasks_from_generic(
        vec![(
            "some_dir",
            &BinaryCrateName::Main,
            [].into_iter(),
            "Child description here".to_owned(),
        )]
        .into_iter(),
    );
    group::start_parallel_tasks(&mut tasks, "", &GroupEnd::ProcessAll);
}
