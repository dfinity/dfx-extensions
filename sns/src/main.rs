use clap::Parser;

mod commands;
mod lib;

#[derive(Parser)]
#[clap(name("dfx"), version = "0.1.0", author = "")]
struct CliOptions {
    // #[clap(subcommand)]
    #[clap(subcommand)]
    command: commands::SubCommand,
}

fn main() {
    let options = CliOptions::parse();
    let cmd = options.command;
    commands::exec(cmd);
    println!("Hello, world!");
}
