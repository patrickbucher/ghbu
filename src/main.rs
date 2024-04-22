use clap::Parser;
use reqwest::blocking::Client;
use serde_json::Value;
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
    let mut ssh_urls: Vec<String> = Vec::new();
    let client = Client::new();
    let url = "https://api.github.com/user/repos";
    let mut page = 1;
    loop {
        let req = client
            .get(url)
            .bearer_auth(&github_token)
            .header("Accept", "application/json")
            .header("User-Agent", "reqwest")
            .query(&[
                ("affiliation", "owner"),
                ("per_page", "20"),
                ("page", &page.to_string()),
            ]);
        match req.send() {
            Ok(res) => {
                let payload = res.json::<serde_json::Value>();
                if let Ok(Value::Array(arr)) = payload {
                    if arr.len() < 1 {
                        break;
                    }
                    for e in arr {
                        if let Some(Value::String(ssh_url)) = e.get("ssh_url") {
                            ssh_urls.push(ssh_url.to_string());
                        }
                    }
                }
            }
            Err(err) => {
                println!("{err}");
                break;
            }
        }
        page += 1;
    }
    for ssh_url in ssh_urls {
        println!("{ssh_url}");
    }
}
