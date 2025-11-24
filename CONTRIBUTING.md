## Build targets
- Rust 1.75+; Android NDK r26; Kotlin 1.9.
- Python 3.11 for bench scripts.

## Dev workflow
1. Fork + branch: `feature/...`, `fix/...`
2. Tests: `cargo test` + `pytest bench/tests`
3. Benchmarks: `bench/run_latency.py --runs 100 --distance 1.0`
4. PR checklist: tests green, docs updated, reproducible results attached.

## Style & reviews
- Evidence over opinions. Include measurements and configs.
- No vendor shaming. Compare features and data neutrally.

