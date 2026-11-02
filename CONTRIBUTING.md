# Contributing

Thanks for helping. Bug reports, fixes and small improvements are all welcome. For a large change,
please open an issue first so we can agree on the approach.

## Getting started

```sh
cargo build --release
cargo test --release --test selftest
```

Run the self-test before opening a pull request. CI runs it on Windows, Linux and macOS. The
[README](README.md#how-it-is-built) explains how the code is laid out.

`IDLEWALKER_TEST=1` is useful while developing: it leaves your sign-in entry alone and skips the
single-instance check.

## Things to know

- **Respect the layers.** Each crate in `crates/` may depend only on the layers inside it (see
  [DEVELOPMENT.md](docs/DEVELOPMENT.md#how-it-is-built)). Do not add an outward dependency to make
  something compile; declare a port in the inner crate and implement it in `crates/platform/`.
- **Translated code is the source now.** Edit `crates/domain/` and `crates/rendering/` directly.
  Do not rerun `tools/cs2rs`, because it overwrites them.
- **Keep OS code in `crates/platform/`.** Each backend exposes the same functions, so a new function
  needs adding to `windows.rs`, `macos.rs` and `linux.rs`, and reaches the rest of the program
  through a port.
- **Keep the cast on the UI thread.** The object model in `crates/domain/src/rt.rs` uses `unsafe`
  handles, and the compiler will not catch a handle used from the wrong thread.
- **Do not edit the version by hand.** The release tag sets it.
- **New characters should be original.** We can't accept new likenesses of characters owned by
  someone else.

## Pull requests

- Keep each pull request to one change, and say how you tested it and on which OS.
- If a feature behaves differently across OSes, update the table in the README.
- Add a line to the `Unreleased` section of [CHANGELOG.md](CHANGELOG.md) for anything users notice.

By contributing, you agree that your contribution is licensed under the [MIT License](LICENSE).
Please follow the [Code of Conduct](CODE_OF_CONDUCT.md).
