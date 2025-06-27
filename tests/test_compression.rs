use barotrauma_compress::{compress, decompress};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Test decompression against a real save file
#[test]
fn test_decompress() {
    // set up the test directory
    let mut repo_test_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    repo_test_file_path.push("test-resources");
    repo_test_file_path.push("barotrauma_nerds_000.save");
    let original_len = repo_test_file_path.metadata().unwrap().len();
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_dir_path = temp_dir.path();
    let test_file_path = temp_dir_path.join(repo_test_file_path.file_name().unwrap());
    std::fs::copy(repo_test_file_path, &test_file_path).unwrap();

    // make sure decompression works
    decompress(&test_file_path).unwrap();

    // copy decompressed dir to a new name so I can compress it again
    let test_dir_1_path = test_file_path.parent().unwrap().join("barotrauma_nerds_000");
    let test_dir_2_path = test_dir_1_path.parent().unwrap().join("test");
    copy_dir_all(test_dir_1_path, &test_dir_2_path).unwrap();

    // re-compress
    compress(&test_dir_2_path).unwrap();

    let recompressed_file_path = test_dir_2_path.with_extension("save");

    let recompressed_len = std::fs::metadata(recompressed_file_path).unwrap().len();

    // The vanilla Barotrauma and barotrauma-compress gzip files differ.
    // This is due to different gzip implementations, but is fine because Barotrauma uses a standard decoder.
    let ratio = recompressed_len as f64 / original_len as f64;
    println!("original_len={original_len}\nrecompressed_len={recompressed_len}\nratio={ratio:.3}");
}

/// Test compression against a real decompressed save
#[test]
fn test_compress() {
    // set up the test directory
    let mut repo_test_dir_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    repo_test_dir_path.push("test-resources");
    repo_test_dir_path.push("test");
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_dir_path = temp_dir.path();
    let test_dir_path = temp_dir_path.join(repo_test_dir_path.file_name().unwrap());
    copy_dir_all(repo_test_dir_path, &test_dir_path).unwrap();

    // make sure compression works
    compress(&test_dir_path).unwrap();

    // rename the compressed file so we can decompress it into a new dir next to the original dir
    let test_file_path_1 = test_dir_path.parent().unwrap().join("test.save");
    let test_file_path_2 = test_dir_path.parent().unwrap().join("test2.save");
    std::fs::rename(test_file_path_1, &test_file_path_2).unwrap();

    // decompress the save we just compressed
    decompress(test_file_path_2).unwrap();
    let test_dir_path_2 = test_dir_path.parent().unwrap().join("test2");

    // ensure the input files equal the output files
    assert_dirs_equal(test_dir_path, test_dir_path_2);
}

/// Copy a directory recursively
fn copy_dir_all(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> std::io::Result<()> {
    for source_entry in WalkDir::new(&source) {
        let source_entry = source_entry?;
        let relative_path = source_entry
            .path()
            .strip_prefix(&source)
            .expect("Expected a child DirEntry to start with the parent path");
        let destination_path = destination.as_ref().join(relative_path);
        if source_entry.file_type().is_dir() {
            std::fs::create_dir(destination_path)?;
        } else if source_entry.file_type().is_file() {
            std::fs::copy(source_entry.path(), destination_path)?;
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, UnknownEntryType))?;
        }
    }
    Ok(())
}

/// Error returned if a DirEntry is neither a directory nor a regular file.
#[derive(Debug)]
struct UnknownEntryType;

impl Display for UnknownEntryType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Foo")
    }
}

impl Error for UnknownEntryType {}

/// Assert two directory trees are byte-for-byte equal in every file.
fn assert_dirs_equal(dir_a: impl AsRef<Path>, dir_b: impl AsRef<Path>) {
    let dir_a_owned = dir_a.as_ref().to_owned();
    let walkdir_a = WalkDir::new(&dir_a).sort_by(move |a, b| {
        let a_stripped = a.path().strip_prefix(&dir_a_owned).unwrap();
        let b_stripped = b.path().strip_prefix(&dir_a_owned).unwrap();
        a_stripped.cmp(b_stripped)
    });
    let dir_b_owned = dir_b.as_ref().to_owned();
    let walkdir_b = WalkDir::new(&dir_b).sort_by(move |a, b| {
        let a_stripped = a.path().strip_prefix(&dir_b_owned).unwrap();
        let b_stripped = b.path().strip_prefix(&dir_b_owned).unwrap();
        a_stripped.cmp(b_stripped)
    });
    for (entry_a, entry_b) in walkdir_a.into_iter().zip(walkdir_b) {
        let entry_a = entry_a.unwrap();
        let entry_b = entry_b.unwrap();
        let a_stripped = entry_a.path().strip_prefix(&dir_a).unwrap();
        let b_stripped = entry_b.path().strip_prefix(&dir_b).unwrap();
        assert_eq!(a_stripped, b_stripped, "expected relative file paths to be the same");
        assert_eq!(
            entry_a.file_type(),
            entry_b.file_type(),
            "expected file types to be the same"
        );
        if entry_a.file_type().is_file() {
            let mut file_a = File::open(entry_a.path()).unwrap();
            let mut file_b = File::open(entry_b.path()).unwrap();

            assert_eq!(
                file_a.metadata().unwrap().len(),
                file_b.metadata().unwrap().len(),
                "file length differs"
            );

            const BUFFER_SIZE: usize = 4096;
            let mut buf_a: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
            let mut buf_b: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
            loop {
                let len_a = file_a.read(&mut buf_a).unwrap();
                let len_b = file_b.read(&mut buf_b).unwrap();
                assert_eq!(len_a, len_b, "read lengths differ");

                if len_a == 0 {
                    break;
                }

                let slice_a = &buf_a[..len_a];
                let slice_b = &buf_b[..len_b];
                assert_eq!(slice_a, slice_b, "read contents differ");
            }
        }
        println!("{} == {}", entry_a.path().display(), entry_b.path().display());
    }
}
