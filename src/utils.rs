use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    fs::{self, File},
    path::{Path, PathBuf},
};

use uuid::Uuid;
use walkdir::{DirEntry, WalkDir};

pub fn save_dir_mapping(output_dir_path: &Path, dir_mapping: &BTreeMap<PathBuf, PathBuf>) {
    let mut output_json = File::create(output_dir_path.join("mapping.json")).unwrap();
    let wrote_json = serde_json::to_writer_pretty(&mut output_json, &dir_mapping);

    if let Err(error) = wrote_json {
        panic!(
            "Something wrong happened! Received the following error {:?}",
            error
        )
    }
}

pub fn is_dir(entry: &DirEntry) -> bool {
    entry.path().is_dir()
}

pub fn filter_dir_and_extension(entry: &DirEntry, desired_extension: &OsStr) -> bool {
    // Include directories:
    if entry.path().is_dir() {
        return true;
    }

    // We only care about files with the extension that was passed in:
    entry
        .path()
        .extension()
        .map(|ext| ext == desired_extension)
        .unwrap_or_default()
}

pub struct GetFilepathsResult {
    pub input_to_output_vec: Vec<(PathBuf, PathBuf)>,
    pub directory_mapping: BTreeMap<PathBuf, PathBuf>,
}

pub fn get_filepaths(
    replaypack_input_dir: &Path,
    replaypack_output_dir: &Path,
    desired_extension: &OsStr,
) -> GetFilepathsResult {
    let mut output = vec![];

    let entries = WalkDir::new(replaypack_input_dir)
        .into_iter()
        .filter_entry(|entry| filter_dir_and_extension(entry, desired_extension));

    let mut directory_mapping = BTreeMap::new();
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                // log the error and continue
                eprintln!("{e}");
                continue;
            }
        };

        // We don't really care about the directories them selves
        // as they are part of the file path already
        if entry.path().is_dir() {
            continue;
        }

        // Full parent entry path is required to find the relative path for the mapping:
        let full_parent_entry_path = entry.path();
        println!("{}", full_parent_entry_path.display());

        // This will be used in the mapping.
        // {"new_filename.extension": "old/relative/path/old_filename.extension"}
        let relative_entry_path = full_parent_entry_path
            .strip_prefix(replaypack_input_dir)
            .unwrap();
        println!("{}", relative_entry_path.display());

        let mut unique_id_filename = OsString::from(Uuid::new_v4().to_simple().to_string());
        unique_id_filename.push(".");
        unique_id_filename.push(desired_extension);
        let unique_id_filename = PathBuf::from(unique_id_filename);
        println!("{}", unique_id_filename.display());

        output.push((
            entry.path().to_owned(),
            replaypack_output_dir.join(&unique_id_filename),
        ));
        directory_mapping.insert(unique_id_filename, relative_entry_path.to_owned());
    }

    GetFilepathsResult {
        input_to_output_vec: output,
        directory_mapping,
    }
}

pub fn copy_files(files: &[(PathBuf, PathBuf)]) {
    for (input, output) in files {
        // Ensure the directory exists:
        if let Some(parent) = output.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("failed to create parent directory: {e}");
                return;
            }
        }

        if let Err(e) = fs::copy(input, output) {
            eprintln!("failed to copy files: {e}");
        }
    }
}
