use std::{
    ffi::OsStr,
    fs::File,
    path::{Path, PathBuf},
    str::FromStr,
};

use uuid::Uuid;
use walkdir::{DirEntry, WalkDir};

use std::io::Write;

pub fn save_dir_mapping(
    output_dir_path: PathBuf,
    dir_mapping: serde_json::Map<String, serde_json::Value>,
) {
    let mut output_json = File::create(Path::new(&output_dir_path)).unwrap();
    let serialized_json = serde_json::to_string_pretty(&dir_mapping).unwrap();

    let wrote_json = write!(output_json, "{}", serialized_json);
    match wrote_json {
        Err(error) => panic!(
            "Something wrong happened! Received the following error {:?}",
            error
        ),
        Ok(_) => (),
    }
}

pub fn is_dir(entry: &DirEntry) -> bool {
    entry.path().is_dir()
}

pub fn filter_dir_and_extension(entry: &DirEntry, desired_extension: &String) -> bool {
    // Include directories:
    if entry.path().is_dir() {
        return true;
    }

    // We only care about files with the extension that was passed in:
    let desired_ext = OsStr::new(&desired_extension);
    match entry.path().extension() {
        Some(s) if s == desired_ext => true,
        None | Some(_) => false,
    }
}

pub struct GetFilepathsResult {
    pub input_to_output_vec: Vec<(PathBuf, PathBuf)>,
    pub directory_mapping: serde_json::Map<String, serde_json::Value>,
}

pub fn get_filepaths(
    replaypack_input_dir: &PathBuf,
    replaypack_output_dir: &PathBuf,
    desired_extension: &String,
) -> GetFilepathsResult {
    let mut output = vec![];

    let entries = WalkDir::new(replaypack_input_dir)
        .into_iter()
        .filter_entry(|entry| filter_dir_and_extension(entry, desired_extension));

    let mut directory_mapping = serde_json::Map::new();
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
            .strip_prefix(replaypack_input_dir.to_str().unwrap())
            .unwrap();
        println!("{}", relative_entry_path.display());

        let src = entry.path();
        let mut dst = PathBuf::from(replaypack_output_dir);
        println!("{}", dst.display());

        let mut unique_id_filename = Uuid::new_v4().to_simple().to_string();
        println!("{}", unique_id_filename);

        let mut extension_w_dot = String::from(".");
        extension_w_dot.push_str(&desired_extension);
        unique_id_filename.push_str(&extension_w_dot);

        dst.push(src.file_name().unwrap());

        output.push((src.to_owned(), dst));
        directory_mapping.insert(
            unique_id_filename,
            serde_json::Value::String(
                String::from_str(relative_entry_path.to_str().unwrap()).unwrap(),
            ),
        );
    }

    return GetFilepathsResult {
        input_to_output_vec: output,
        directory_mapping,
    };
}

pub fn copy_files(files: &[(PathBuf, PathBuf)]) {
    for (input, output) in files {
        // Ensure the directory exists:
        if let Err(e) = std::fs::copy(input, output) {
            eprintln!("failed to copy files: {e}");
        }
    }
}
