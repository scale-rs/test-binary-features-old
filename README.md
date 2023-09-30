# Automate testing your crate features

This crate helps you with testing any combinations of your crate's features programmatically. All
with one `cargo test`.

You'll need to set up auxiliary crates. Also, this can't run unit tests (`#[test]`) as-is - you'll
need to refactor those functions and have them called from binary crate(s) within the same project.

Based on `test-binary`.
