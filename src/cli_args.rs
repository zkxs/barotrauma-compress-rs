// Copyright 2023 Michael Ripley
// This file is part of barotrauma-compress.
// barotrauma-compress is licensed under the AGPL-3.0 license (see LICENSE file for details).

use std::path::PathBuf;
use clap::Parser;

/// A utility to compress and decompress Barotrauma saves. Whether to compress or decompress will be
/// chosen automatically based on what <INPUT> is: files will be decompressed, and directories will
/// be compressed.
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// input file or directory
    pub input: PathBuf,
}
