pub mod directory_flattener;
pub mod utils;

use std::path::PathBuf;

use clap::Parser;

fn main() {
    let args = directory_flattener::Args::parse();

    let main_input_dir = PathBuf::from(&args.input_directory).canonicalize().unwrap();
    let main_output_dir = PathBuf::from(args.output_directory).canonicalize().unwrap();

    let desired_extension = args.file_extension;

    directory_flattener::directory_flattener(main_input_dir, main_output_dir, desired_extension);
}
