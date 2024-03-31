use clap::{ArgGroup, Parser};

#[derive(Parser)]
#[clap(version, about, long_about = None)]
#[clap(group(
    ArgGroup::new("ports")
        .required(true)
        .multiple(true)
        .args(&["expose", "forward"]),
))]
pub struct Options {
    #[arg(short = 'n', long = "name", default_value = "forwarder")]
    pub service_name: String,

    #[arg(short = 'e', long = "expose", value_delimiter = ',')]
    pub expose: Option<Vec<u16>>,

    #[arg(short = 'f', long = "forward", value_delimiter = ',')]
    pub forward: Option<Vec<String>>,
}

pub fn parse() -> Options {
    Options::parse()
}
