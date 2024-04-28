use clap::Parser;
use ghbu::LocalRepo;
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
            .filter(|r| r.existing_dir() && !r.open_bare().is_ok());
        broken_repos.for_each(|r| match r.annihilate() {
            Ok(_) => println!("{}: removed broken repo at {}", r.name(), r.path()),
            Err(_) => println!("{}: unable to remove broken repo at {}", r.name(), r.path()),
        });
    }

    // TODO: refactor code from here to use `local_repos` instead of `all_repos`
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
