# sway-alttab

[![crates.io](https://flat.badgen.net/crates/v/sway-alttab)](https://crates.io/crates/sway-alttab) [![crates.io](https://flat.badgen.net/crates/d/sway-alttab)](https://crates.io/crates/sway-alttab) [![Build Status](https://flat.badgen.net/github/checks/reisub0/sway-alttab?label=build)](https://github.com/reisub0/sway-alttab/actions?query=workflow%3Abuild) [![made-with-rust](https://flat.badgen.net/badge/made%20with%20♥/rust/dea584)](https://www.rust-lang.org/)

A simple daemon that keeps track of your last focused window and switches to it on receiving a SIGUSR1. Automatically binds Alt-Tab to the same action.

## Installation

### From Binaries

Binary releases can be found at the [Releases](https://github.com/reisub0/sway-alttab/releases/latest) page.

### From Crates.io

```bash
cargo install sway-alttab
```

### From Source

```bash
git clone https://gitlab.com/reisub0/sway-alttab
cargo install --path sway-alttab
```

## WIP
[x] Offer the ability to customise the keybindings

## License

sway-alttab is licensed under the [MIT License](https://choosealicense.com/licenses/mit/).
