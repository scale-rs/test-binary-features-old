# Reason for this submodule, and naming conventions

This module exists, so that unit test modules are separate from the source - to help with file
browsing/navigation.

Having the actual test modules not called `tests`, but with each name based on module they test and
with a postfix `_tests.rs`, helps with navigation across editor tabs.

The name `unit_tests`, and having `_tests` at the end of each test module, mean duplication in the
import paths. But we don't import those modules anyway (other than once from `lib.rs`, and one per
test from `unit_tests.rs`).
