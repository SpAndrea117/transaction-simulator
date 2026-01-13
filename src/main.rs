use std::path::PathBuf;

use structopt::StructOpt;

use crate::{business_logic::apply_transaction, shared::errors::Error};

mod business_logic;
mod shared;

#[derive(Debug, StructOpt)]
struct Args {
    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

fn main() -> Result<(), Error> {
    let args = Args::from_args();

    apply_transaction(args.input, std::io::stdout())
}
