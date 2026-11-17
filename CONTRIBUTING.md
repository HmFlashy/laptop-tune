# Contributing to laptop-tune

Contributions are welcome!

## Development

1. Fork the repository and create a feature branch.
2. Make sure all unit tests pass and formatting/lints are clean:
   ```bash
   cargo fmt --check
   cargo clippy --all-targets
   cargo test
   ```
3. Keep sysfs operations testable by accepting a root `&Path` parameter (see `src/profile.rs` and `src/gpu.rs` for examples) so tests run without root privileges.
4. Open a pull request describing your changes and hardware tested (CPU/GPU/distro).
