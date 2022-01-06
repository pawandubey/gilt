use std::{path::PathBuf, fmt::Display};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt()]
struct Options {
    #[structopt(short, long)]
    colorize: bool,

    #[structopt(short, long, parse(from_os_str), env = "HOME")]
    location: PathBuf,

    #[structopt(short, long, parse(from_str))]
    exec: String,

    #[structopt(short, long, default_value = "stdin", help = "json, stdin")]
    output: OutputType,
}

#[derive(Debug)]
enum OutputType {
    Stdin,
    Json,
}

impl std::str::FromStr for OutputType {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(OutputType::Json),
            "stdin" => Ok(OutputType::Stdin),
            _ => Err("Unsupported output type."),
        }
    }
}

// TODO: Maybe cleanup
// impl Display for OutputType {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match &self {
//             OutputType::Json => write!(f, "json"),
//             OutputType::Stdin => write!(f, "stdin"),
//         }
//     }
// }

fn main() {
    let options = Options::from_args();
    println!("{:?}", options)
}
