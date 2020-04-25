ersatz
======
[![Latest version](https://img.shields.io/crates/v/ersatz.svg)](https://crates.io/crates/ersatz)
![Rust](https://img.shields.io/badge/rust-nightly-brightgreen.svg)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

Analysis, transformation and synthesis of entity-reaction systems.
Based on Ehrenfeucht and Rozenberg's theory of [Reaction
Systems](https://doi.org/10.1142/9789813148208_0001).

## Prerequisites

In principle, `ersatz` should build wherever `rustc` and `cargo` runs.
Its executables should run on any
[platform](https://forge.rust-lang.org/release/platform-support.html)
supporting Rust `std` library.

Be aware, though, that the project is very much WIP.  Currently, the
main toolchain used in development is nightly channel of Rust 1.44.

## Installation

Having [Rust](https://www.rust-lang.org/downloads.html) installed,
ensure its version is at least 1.44: check with `cargo version` and
run `rustup update` if needed.  Then

```bash
$ cargo install ersatz
```

will automatically download, build, and install the latest `ersatz`
release on [crates.io](https://crates.io/crates/ersatz).

## License

`ersatz` is licensed under the MIT license.  Please read the
[LICENSE-MIT](LICENSE-MIT) file in this repository for more
information.
