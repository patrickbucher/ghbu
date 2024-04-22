use reqwest::blocking;
use std::{env, process};

const TOKEN_ENVVAR: &str = "GITHUB_TOKEN";

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
    println!("{github_token}");

    // do a request
    let url = "https://www.paedubucher.ch/index.html";
    match blocking::get(url) {
        Ok(body) => println!("{:?}", body),
        Err(err) => println!("{err}"),
    }
}
