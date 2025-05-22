use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long)]
    pub program: String,
    // TODO: add expressions -- so far just programs
    // #[arg(short, long, conflicts_with = "program")]
    // pub expression: String,
}

impl Args {
    // FIXME maybe this should error out instead of mutating args
    pub fn validate(&mut self) {
        //TODO: check that the program is a valid path
    }
}
