use std::error::Error;
use ersatz::cli::{App, Command, Describe};

fn main() -> Result<(), Box<dyn Error>> {
    let ref cli_spec_str = include_str!("ersatz.cli");

    let cli_spec = clap::YamlLoader::load_from_str(cli_spec_str)?;
    let cli_matches = clap::App::from_yaml(&cli_spec[0]);
    let app = App::from_clap(cli_matches);

    let result = match app.subcommand_name().unwrap_or("describe") {
        "describe" => Describe::run(&app),
        _ => unreachable!(),
    };

    if let Err(err) = result {
        eprintln!("[Error] {}.", err);
        std::process::exit(-1)
    } else {
        std::process::exit(0)
    }
}
