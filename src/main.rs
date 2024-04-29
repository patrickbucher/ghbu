use clap::Parser;
use ghbu::LocalRepo;
use git2::{Cred, RemoteCallbacks};
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
            Ok(_) => eprintln!("{}: removed broken repo at {}", r.name(), r.path()),
            Err(_) => eprintln!("{}: unable to remove broken repo at {}", r.name(), r.path()),
        });
    }

    let (to_fetch, to_clone): (Vec<_>, Vec<_>) = local_repos
        .iter()
        .partition(|r| r.existing_dir() && r.open_bare().is_ok());

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_, username, _| {
        Cred::ssh_key(
            username.unwrap(), // FIXME
            None,
            Path::new(&args.keyfile),
            None, // TODO: provide passphrase as optional command line argument
        )
    });

    let mut options = git2::FetchOptions::new();
    options.remote_callbacks(callbacks);

    for repo in to_fetch {
        match repo.fetch(&mut options) {
            Ok(_) => eprintln!("{}: fetched to {}", repo.name(), repo.path()),
            Err(_) => eprintln!("{}: fetching to {}: failed", repo.name(), repo.path()),
        }
    }

    let mut builder = git2::build::RepoBuilder::new();
    builder.bare(true);
    builder.fetch_options(options);

    for repo in to_clone {
        match repo.clone(&mut builder) {
            Ok(_) => eprintln!("{}: cloned to {}", repo.name(), repo.path()),
            Err(_) => eprintln!("{}: cloning to {}: failed", repo.name(), repo.path()),
        }
    }
}
