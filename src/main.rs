use clap::Parser;
use std::{env, process};

const TOKEN_ENVVAR: &str = "GITHUB_TOKEN";

/// Backs up the GitHub repositories of the given accounts.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Output Directory
    #[arg(short, long)]
    to: String,
}

fn main() {
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
    let args = Args::parse();
    println!("clone to: {}", args.to);
    if let Err(e) = ghbu::prepare_clone_dir(args.to) {
        eprintln!("{e}");
        process::exit(1);
    }

    for (name, url) in ghbu::fetch_repo_ssh_urls_by_name(github_token) {
        println!("{name:40} {url}");
    }
}
