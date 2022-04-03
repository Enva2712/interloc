//! The `il` binary lets you run interloc from the command line

use clap::Parser;
use interloc::{Inter, Loc};
use serde_yaml::from_reader;
use std::fs::File;

/// Interloc verifies that changes to an interface are safe
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// The interface definition file we're coming from
    #[clap(short, long, value_name = "FILE")]
    from: String,

    /// The interface definition file we're updating to
    #[clap(short, long, value_name = "FILE")]
    to: String,

    /// A loc file of dependencies into the interface
    #[clap(short, value_name = "FILE", multiple_occurrences = true)]
    locs: Vec<String>,
}

fn main() -> Result<(), failure::Error> {
    let args = Args::parse();

    let from_interface: Inter = from_reader(File::open(args.from)?)?;
    let to_interface: Inter = from_reader(File::open(args.to)?)?;

    let loc = match args.locs.len() {
        0 => Loc::Tip,
        _ => {
            let mut combined = Loc::Empty;
            for loc_file in args.locs {
                combined.consume(from_reader(File::open(loc_file)?)?);
            }
            combined
        }
    };

    let from_interface = match loc.select_subset(&from_interface) {
        Some(i) => i,
        None => {
            panic!("The locs you passed didn't match the interface");
        }
    };

    let mut encountered_error = false;
    for err in from_interface.try_fit_within(&to_interface) {
        if !encountered_error {
            encountered_error = true;
        }
        println!("error: {}", err)
    }

    if encountered_error {
        println!("");
        println!("The interfaces aren't compatible ☹️");
        println!("See above for specific problems");
    } else {
        println!("The interfaces are compatible 🎉")
    }

    Ok(())
}
