# Barotrauma Compress

A simple CLI utility compress and decompress Barotrauma save files using fully cross-platform native code.

## Installation

**Manual:** download an artifact from the [latest release](https://github.com/zkxs/barotrauma-compress-rs/releases/latest)

**Cargo**: `cargo install barotrauma-compress`

## Usage

Grab an 

Whether to compress or decompress will be chosen automatically based on what `<INPUT>` is: files will be decompressed, and directories will be compressed.

```
Usage: barotrauma-compress <INPUT>

Arguments:
  <INPUT>  input file or directory.

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Building from Source

1. [Install Rust](https://www.rust-lang.org/tools/install)
2. Clone the project
3. `cargo build --release`

## License

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License.
See [LICENSE](LICENSE) for the full license text.
