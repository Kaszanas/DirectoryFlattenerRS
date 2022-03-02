pub mod directory_flattener;
pub mod utils;

use clap::Parser;

fn main() {
    let args = directory_flattener::Args::parse();

    let main_input_dir = args.input_directory.canonicalize().unwrap();
    let main_output_dir = args.output_directory.canonicalize().unwrap();

    let desired_extension = args.file_extension;

    directory_flattener::directory_flattener(main_input_dir, main_output_dir, desired_extension);
}
