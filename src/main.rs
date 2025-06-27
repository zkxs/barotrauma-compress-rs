// Copyright 2025 Michael Ripley
// This file is part of barotrauma-compress.
// barotrauma-compress is licensed under the AGPL-3.0 license (see LICENSE file for details).

use std::process::ExitCode;

use crate::cli_args::CliArgs;
use barotrauma_compress::{compress, decompress};
use clap::Parser as _;

mod cli_args;

// source: https://docs.rs/debug_print/1.0.0/src/debug_print/lib.rs.html#49-52
// licensed under MIT OR Apache-2.0
macro_rules! debug_println {
    ($($arg:tt)*) => (#[cfg(debug_assertions)] println!($($arg)*));
}

fn main() -> ExitCode {
    // a silly little wrapper because I don't like how Result prints when used as a return value from main
    if let Err(e) = handle_args() {
        eprintln!("{}", e);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn handle_args() -> Result<(), String> {
    let args: CliArgs = CliArgs::parse();
    debug_println!("Input: {}", args.input.display());

    if args.input.is_file() {
        decompress(args.input).map_err(|e| format!("Error performing decompress operation: {e}"))
    } else if args.input.is_dir() {
        compress(args.input).map_err(|e| format!("Error performing compress operation: {e}"))
    } else {
        Err("Could not open input as a file or directory. Does it exist?".to_string())
    }
}
