# Dev Loop Queue

## Status

- [x] Implement formal version save so the `Find -> Create -> Evaluate -> Version` loop is complete.
- [ ] Implement restore + diff flow for formal versions, including snapshot handling.

## Completed In This Round

- [x] Implement formal version save end-to-end (Rust command + snapshot copy + UI wiring).
- [x] Reconcile the product spec and implementation around `skill-create` versus `Claude CLI` draft generation.
- [x] Make workspace a first-class runtime context so bootstrap, create, eval, search, and recent/open flows use the current workspace instead of a fixed default sample.
