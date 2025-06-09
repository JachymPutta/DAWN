use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug, Default)]
pub struct Args {
    // #[arg(short, long)]
    // pub program: PathBuf, // TODO: add expressions -- so far just programs
    //                       // #[arg(short, long, conflicts_with = "program")]
    //                       // pub expression: String,
}

impl Args {
    pub fn validate(&mut self) {

        // assert!(
        //     self.program.is_file() && self.program.extension() == Some(std::ffi::OsStr::new("nix")),
        //     "Expected a .nix file, but got: {:?}",
        //     self.program
        // );
    }
}
