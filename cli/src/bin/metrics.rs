extern crate metrics_lib;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use std::io::{self, Read};
use std::str::FromStr;
use structopt::StructOpt;

use metrics_lib::*;

#[derive(Debug)]
enum Format {
    TOML,
    YAML,
}

impl FromStr for Format {
    type Err = Box<std::error::Error>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "t" | "toml" | "TOML" => Ok(Format::TOML),
            "y" | "yaml" | "YAML" => Ok(Format::YAML),
            _ => Err("unrecognized format".into()),
        }
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "metrics", about = "A tool to describe metrics semantics")]
struct Opt {
    #[structopt(short = "f", long = "format", help = "Input format (YAML or TOML)", default_value = "TOML")]
    input: Format,

    #[structopt(short = "o", long = "output-format", help = "Output format (YAML or TOML)", default_value = "TOML")]
    output: Format,
}

fn main() {
    let opt = Opt::from_args();
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).unwrap();

    let metrics = match opt.input {
        Format::TOML => parse_toml(&buffer),
        Format::YAML => parse_yaml(&buffer),
    }.expect("Couldn't parse input");

    let output = match opt.output {
        Format::TOML => generate_toml(&metrics),
        Format::YAML => generate_yaml(&metrics),
    }.expect("Couldn't generate output");
    println!("{}", &output);
}
