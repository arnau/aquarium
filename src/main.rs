use clap::Parser;
use std::io::Write;

use aquarium::cli;

#[derive(Debug, Parser)]
enum Subcommand {
    #[clap(alias = "b")]
    Build(cli::build::Cmd),
    Clean(cli::clean::Cmd),
}

#[derive(Debug, Parser)]
#[clap(name = "wb", version)]
struct Cli {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

fn main() {
    env_logger::builder()
        .format(|buf, record| {
            let level_style = buf.default_styled_level(record.level());

            writeln!(buf, "{}: {}", level_style, record.args())
        })
        .init();

    let cli: Cli = Cli::parse();

    match cli.subcommand {
        Subcommand::Build(cmd) => match cmd.run() {
            Ok(_msg) => {
                // println!("{}", msg);
            }
            Err(err) => {
                eprintln!("{:?}", err);
            }
        },
        Subcommand::Clean(cmd) => match cmd.run() {
            Ok(_msg) => {
                // println!("{}", msg);
            }
            Err(err) => {
                eprintln!("{:?}", err);
            }
        },
    }
}
