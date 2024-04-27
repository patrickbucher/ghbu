use clap::Parser;
use git2::{Cred, RemoteCallbacks, Repository};
use std::collections::HashMap;
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

    if let Err(err) = ghbu::prepare_clone_dir(&args.to) {
        eprintln!("prepare clone directory {}: {}", args.to, err);
        process::exit(1);
    }

    let all_repos = ghbu::fetch_repo_ssh_urls_by_name(github_token);
    let (old_repos, new_repos): (HashMap<_, _>, HashMap<_, _>) =
        all_repos.iter().partition(|(name, _)| {
            let path = Path::new(&args.to).join(name);
            match Repository::open_bare(path) {
                Ok(_) => true,
                Err(_) => false, // TODO: remove path? (broken repo)
            }
        });

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

    for (name, _) in old_repos {
        let abs_path = Path::new(&args.to).join(name);
        if let Ok(repo) = Repository::open_bare(&abs_path) {
            let head = repo.head().unwrap(); // FIXME
            if head.is_branch() {
                let branch_name = head.shorthand().unwrap(); // FIXME
                let mut origin = repo.find_remote("origin").unwrap(); // FIXME
                match origin.fetch(&[branch_name], Some(&mut options), None) {
                    Ok(_) => eprintln!("{name}: fetched {branch_name}"),
                    Err(err) => eprintln!("{}: fetching {}: {}", name, branch_name, err),
                }
            } else {
                eprintln!("{}: head is not a branch", name);
            }
        } else {
            eprintln!(
                "{}: repo at path {} cannot be opened as bare repo",
                name,
                abs_path.display()
            );
        }
    }

    let mut builder = git2::build::RepoBuilder::new();
    builder.bare(true);
    builder.fetch_options(options);

    for (name, ssh_url) in new_repos {
        let abs_path = Path::new(&args.to).join(name);
        match builder.clone(&ssh_url, &abs_path) {
            Ok(_) => eprintln!("{}: cloned to {}", name, abs_path.display()),
            Err(err) => eprintln!("{}: cloning to {}: {}", name, abs_path.display(), err),
        }
    }
}
