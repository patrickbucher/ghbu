use clap::Parser;
use reqwest::blocking;
use std::{env, process};

const TOKEN_ENVVAR: &str = "GITHUB_TOKEN";

/// Backs up the GitHub repositories of the given accounts.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Output Directory
    #[arg(short, long)]
    to: String,

    /// Accounts to backup
    #[arg()]
    accounts: Vec<String>,
}

fn main() {
    // get environment variable
    let github_token = match env::vars()
        .filter(|(k, _)| k == TOKEN_ENVVAR)
        .map(|(_, v)| v)
        .next()
    {
        Some(v) => v,
        None => {
            eprintln!("missing environment variable {TOKEN_ENVVAR}");
            process::exit(1);
        }
    };

    // parse arguments
    let args = Args::parse();
    println!("backup accounts {:?}", args.accounts);

    println!("token: {}, backup to: {}", github_token, args.to);

    // do a request
    let url = "https://www.paedubucher.ch/index.html";
    match blocking::get(url) {
        Ok(body) => println!("{:?}", body),
        Err(err) => println!("{err}"),
    }
}
