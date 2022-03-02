use std::{ffi::OsString, io::ErrorKind, path::PathBuf};

use clap::Parser;
use walkdir::WalkDir;

use crate::utils;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Input directory that will be walked and searched for replaypacks
    #[clap(short, long, default_value = "processing/input", parse(from_os_str))]
    pub input_directory: PathBuf,

    /// Output directory where the files will be copied into a flat directory structure
    #[clap(short, long, default_value = "processing/output", parse(from_os_str))]
    pub output_directory: PathBuf,

    /// File extension which will be used to detect the files that ought to be copied to a new flat directory structure
    #[clap(short, long, default_value = "SC2Replay", parse(from_os_str))]
    pub file_extension: OsString,
}

pub fn directory_flattener(
    main_input_dir: PathBuf,
    main_output_dir: PathBuf,
    desired_extension: OsString,
) {
    // Iterate over the depth 1 from input dir.
    // This accesses directories (replaypacks) that are within the input directory:
    let intermediate_child_dirs = WalkDir::new(&main_input_dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_entry(utils::is_dir)
        .map(|dir_entry| dir_entry.unwrap().path().to_owned());

    // Iterating over all of the replaypacks that were found in the input directory:
    for input_replaypack in intermediate_child_dirs {
        // Output dir is composed of the input dirs last component and the root of the output
        let sub = &input_replaypack.strip_prefix(&main_input_dir).unwrap();
        let output_dir = main_output_dir.join(sub);

        // Remove the any old output directory in case it exists. Otherwise, every run would create
        // duplicates as output file names are randomly generated.
        match std::fs::remove_dir_all(&output_dir) {
            Ok(_) => {}
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => panic!("failed removing old output directory: {e}"),
        }

        let files_to_copy =
            utils::get_filepaths(&input_replaypack, &output_dir, &desired_extension);

        utils::copy_files(&files_to_copy.input_to_output_vec);

        utils::save_dir_mapping(&output_dir, &files_to_copy.directory_mapping);
    }
}
