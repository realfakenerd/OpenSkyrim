## Summary

<!-- Briefly describe the purpose of this PR and what problem or feature it addresses. -->

## Type of Change

<!-- Mark the relevant option with an [x] -->

- [ ] 🐛 Bug fix (non-breaking change fixing an issue)
- [ ] ✨ New feature (non-breaking change adding functionality)
- [ ] ⚡ Performance optimization
- [ ] 📦 Asset converter / parser (`crates/converter`)
- [ ] 🎮 Engine / Subsystem (`crates/engine`)
- [ ] 🖥️ Launcher / UI (`crates/launcher`)
- [ ] 📝 Documentation / Spec update (`docs/specs/`)
- [ ] 💥 Breaking change (fix or feature causing existing functionality to change)

## Related Issues & Specs

<!-- Link related issues or specifications below. Example: Fixes #123, Related to #456 -->
- Fixes #
- Related to #
- Spec reference: `docs/specs/`

## Details & Implementation

<!-- Describe what was modified, added, or refactored. -->
- 

## Verification & Testing

<!-- Describe how these changes were tested (OS, GPU/backend, test commands). -->

- **Environment**: OS: `Windows / Linux / macOS` | Backend: `Vulkan / DX12 / Metal`
- **Commands executed**:
  - [ ] `cargo check --workspace`
  - [ ] `cargo test --workspace`
  - [ ] `cargo fmt --all -- --check`
  - [ ] `cargo clippy --all-targets --all-features -- -D warnings`
  - [ ] Manual runtime testing (e.g., `cargo run -p launcher` or `cargo run -p engine`)

<!-- If applicable, attach screenshots, GIFs, or benchmark results below. -->
### Visual / Benchmark Proof (Optional)

## Checklist

- [ ] My code follows the project's code style and idioms.
- [ ] I have added / updated doc comments (`///`) and tests for new logic.
- [ ] I have updated any relevant specs in `docs/specs/`.
- [ ] **Clean-Room Verification**: I confirm that this PR contains no proprietary or copyrighted game assets (`.esm`, `.bsa`, `.dds`, `.nif`, audio, etc.).
- [ ] All CI checks, formatters, and clippy passes locally.
