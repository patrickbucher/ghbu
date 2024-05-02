use clap::Parser;
use ghbu::{create_callbacks, LocalRepo};
use git2::{build::RepoBuilder, FetchOptions};
use std::{env, path::Path, process};

const TOKEN_ENVVAR: &str = "GITHUB_TOKEN";

/// Backs up the GitHub repositories of the given accounts.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Output Directory
    #[arg(short, long)]
    to: String,

    /// SSH Key File
    #[arg(short, long)]
    keyfile: String,

    /// Cleanup Broken Repositories
    #[arg(short, long, default_value_t = false)]
    cleanup: bool,
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

    let path: &Path = match ghbu::prepare_clone_dir(&args.to) {
        Ok(path) => *path,
        Err(err) => {
            eprintln!("prepare clone directory {}: {}", args.to, err);
            process::exit(1);
        }
    };

    let all_repos = ghbu::fetch_repo_ssh_urls_by_name(github_token);
    let local_repos: Vec<LocalRepo> = all_repos
        .iter()
        .map(|(name, ssh_url)| LocalRepo::new(name.clone(), ssh_url.clone(), path))
        .collect();

    if args.cleanup {
        let broken_repos = local_repos
            .iter()
            .filter(|r| r.existing_dir() && r.open_bare().is_err());
        broken_repos.for_each(|r| match r.annihilate() {
            Ok(_) => println!("{}: removed broken repo at {}", r.name(), r.display_path()),
            Err(e) => eprintln!(
                "{}: unable to remove broken repo at {}: {}",
                r.name(),
                r.display_path(),
                e
            ),
        });
    }

    let (to_fetch, to_clone): (Vec<_>, Vec<_>) = local_repos
        .iter()
        .partition(|r| r.existing_dir() && r.open_bare().is_ok());

    let callbacks = create_callbacks(Path::new(&args.keyfile));
    let mut options = FetchOptions::new();
    options.remote_callbacks(callbacks);

    for repo in to_fetch {
        match repo.fetch(&mut options) {
            Ok(_) => println!("{}: fetched to {}", repo.name(), repo.display_path()),
            Err(e) => eprintln!(
                "{}: fetching to {}: {}",
                repo.name(),
                repo.display_path(),
                e
            ),
        }
    }

    let mut builder = RepoBuilder::new();
    builder.bare(true);
    builder.fetch_options(options);

    for repo in to_clone {
        match repo.clone(&mut builder) {
            Ok(_) => println!("{}: cloned to {}", repo.name(), repo.display_path()),
            Err(e) => eprintln!("{}: cloning to {}: {}", repo.name(), repo.display_path(), e),
        }
    }
}
