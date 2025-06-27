// Copyright 2023 Michael Ripley
// This file is part of barotrauma-compress.
// barotrauma-compress is licensed under the AGPL-3.0 license (see LICENSE file for details).

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::mem::MaybeUninit;
use std::path::PathBuf;
use std::process::ExitCode;
use std::{fs, io};

use clap::Parser as _;
use flate2::bufread::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;

use crate::cli_args::CliArgs;

mod cli_args;

const INITIAL_FILENAME_BUFFER_SIZE: usize = 256;

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

// source: https://docs.rs/debug_print/1.0.0/src/debug_print/lib.rs.html#49-52
// licensed under MIT OR Apache-2.0
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => (#[cfg(debug_assertions)] println!($($arg)*));
}

fn decompress(file_path: PathBuf) -> Result<(), String> {
    // open the save file
    let file = File::open(&file_path).map_err(|e| format!("Could not open save file: {}", e))?;
    let gzip_input = BufReader::new(file);
    // the filesystem buffering is handled by this BufReader, so we can make small reads from the underlying stream
    debug_println!("fs read buffer size = {}", gzip_input.capacity());
    let mut input = GzDecoder::new(gzip_input);

    // create the output directory
    let mut directory_path: PathBuf = file_path
        .parent()
        .ok_or("Could not get parent directory of save file")?
        .to_path_buf();
    directory_path.push(
        file_path
            .file_stem()
            .ok_or("Could not remove extension from save file")?,
    );
    debug_println!("directory_path = {}", directory_path.display());
    fs::create_dir(&directory_path).map_err(|e| {
        format!(
            "Could not create target directory \"{}\": {}",
            directory_path.display(),
            e
        )
    })?;

    let mut length_buffer: [u8; 4] = unsafe {
        #[allow(clippy::uninit_assumed_init, invalid_value)]
        // oh no~ there might be GARBAGE in my bytes, whatever will I do?
        MaybeUninit::uninit().assume_init()
    };

    // this buffer is specifically for holding u16-aligned filenames
    let mut filename_buffer = Vec::<u16>::with_capacity(INITIAL_FILENAME_BUFFER_SIZE);

    loop {
        // read the filename length
        if input.read_exact(&mut length_buffer).is_err() {
            // as the filename length prefix is the first token in a chunk, its absence isn't a problem:
            // it just means we have no chunks left and are therefore done decompressing
            break;
        }

        let filename_length: usize = u32::from_le_bytes(length_buffer) as usize;
        debug_println!("filename_size = {}", filename_length * 2);

        // grow the u16 filename buffer if required
        if filename_buffer.capacity() < filename_length {
            filename_buffer.reserve(filename_length - filename_buffer.capacity());
        }

        // size the underlying slice to the necessary size without zeroing
        unsafe {
            filename_buffer.set_len(filename_length);
        }

        // cast the filename buffer to a u8 slice for writing
        let filename_buffer_u8: &mut [u8] = unsafe {
            if let (&mut [], bytes, &mut []) = filename_buffer.align_to_mut() {
                bytes
            } else {
                unreachable!("Vec<u16> is always u8-aligned");
            }
        };

        // read the dang filename, finally
        input
            .read_exact(filename_buffer_u8)
            .map_err(|e| format!("Reached end of stream unexpectedly when reading filename: {}", e))?;
        let filename =
            String::from_utf16(&filename_buffer).map_err(|e| format!("Filename was not valid UTF-16: {}", e))?;
        debug_println!("Decoded filename: {}", filename);

        // get the file length
        input
            .read_exact(&mut length_buffer)
            .map_err(|e| format!("Reached end of stream unexpectedly when reading file length: {}", e))?;
        let file_length: u64 = u32::from_le_bytes(length_buffer) as u64;
        debug_println!("file_length = {}", file_length);

        // create the output file
        let mut output_file_path = directory_path.clone();
        output_file_path.push(&filename);
        let mut output_file =
            File::create(&output_file_path).map_err(|e| format!("Unable to create output file: {}", e))?;

        let mut output_file_reader = input.take(file_length);

        // I don't think we need to buffer the writes for this...
        let bytes_written = io::copy(&mut output_file_reader, &mut output_file)
            .map_err(|e| format!("Error writing decompressed file: {}", e))?;
        assert_eq!(bytes_written, file_length);
        input = output_file_reader.into_inner();

        debug_println!("wrote {}", output_file_path.display());
    }

    Ok(())
}

fn compress(directory_path: PathBuf) -> Result<(), String> {
    let file_path: PathBuf = directory_path.with_extension("save");

    // enumerate files in the input directory
    let mut input_file_paths = Vec::with_capacity(2);
    for entry in fs::read_dir(directory_path).map_err(|e| format!("Unable to enumerate input directory: {}", e))? {
        let entry =
            entry.map_err(|e| format!("Unable to read an entry while enumerating the input directory: {}", e))?;
        let path = entry.path();
        if !path.is_file() {
            // the directory must be flat... I think? If baro supports directory structure then color me surprised.
            return Err(format!("Unable to compress nested directories: \"{}\"", path.display()));
        }
        input_file_paths.push(path);
    }

    // ensure the output file doesn't already exist, as I don't want users to accidentally clobber their saves
    if file_path.exists() {
        return Err(format!("Target file \"{}\" already exists", file_path.display()));
    }

    // create the output file
    let output_file = File::create(file_path).map_err(|e| format!("Unable to create output file: {}", e))?;
    // I do three small writes in a row for file metadata, so we buffer the writer here
    let gzip_output = BufWriter::new(output_file);
    debug_println!("fs write buffer size = {}", gzip_output.capacity());
    // default compression is *probably* fine
    let mut output = GzEncoder::new(gzip_output, Compression::default());

    // add each file to the gzip
    for input_file_path in input_file_paths {
        debug_println!("processing: {}", input_file_path.display());
        let mut input_file = File::open(&input_file_path).map_err(|e| format!("Unable to open input file: {}", e))?;

        // write the filename length prefix
        let input_filename = input_file_path
            .file_name()
            .ok_or("Unable to extract filename of input file")?
            .to_str()
            .ok_or("Unable to convert input filename to unicode")?;
        let input_filename_length = input_filename.len() as u32;
        let mut input_filename_utf16: Vec<u16> = input_filename.encode_utf16().collect();
        let input_filename_buffer_u8: &[u8] = unsafe {
            if let (&mut [], bytes, &mut []) = input_filename_utf16.align_to_mut() {
                bytes
            } else {
                unreachable!("Vec<u16> is always u8-aligned");
            }
        };

        // write the filename length prefix
        let input_filename_length_prefix = input_filename_length.to_le_bytes();
        output
            .write_all(&input_filename_length_prefix)
            .map_err(|e| format!("Unable to write filename length prefix to save: {}", e))?;

        // write the filename
        output
            .write_all(input_filename_buffer_u8)
            .map_err(|e| format!("Unable to write filename to save: {}", e))?;

        // write the file size prefix
        let file_size = input_file
            .metadata()
            .map_err(|e| format!("Unable to read metadata for input file: {}", e))?
            .len();
        let file_size_prefix: u32 = file_size.try_into().map_err(|e| {
            format!(
                "Input file too long (blame the Baro devs for their 4GB filesize limit): {}",
                e
            )
        })?;
        let file_size_prefix = file_size_prefix.to_le_bytes();
        output
            .write_all(&file_size_prefix)
            .map_err(|e| format!("Unable to write filesize prefix to save: {}", e))?;

        // write the file contents
        io::copy(&mut input_file, &mut output).map_err(|e| format!("Error writing input file to save: {}", e))?;
    }

    // because we're using a BufWriter we should explicitly flush to disk
    output
        .flush()
        .map_err(|e| format!("Error flushing save to disk: {}", e))?;
    Ok(())
}
