use clap::Parser;
use git2::{Cred, RemoteCallbacks, Repository};
use std::{env, path::Path, process};

const TOKEN_ENVVAR: &str = "GITHUB_TOKEN";

/// Backs up the GitHub repositories of the given accounts.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Output Directory
    #[arg(short, long)]
    to: String,
}

// TODO: username and SSH key file as command line arguments?

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
    if let Err(e) = ghbu::prepare_clone_dir(&args.to) {
        eprintln!("{e}");
        process::exit(1);
    }

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_, username, _| {
        Cred::ssh_key(
            username.unwrap(),
            None,
            Path::new("/home/patrick/.ssh/id_ed25519"),
            None,
        )
    });
    let mut options = git2::FetchOptions::new();
    options.remote_callbacks(callbacks);
    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(options);

    for (name, url) in ghbu::fetch_repo_ssh_urls_by_name(github_token) {
        let repo_path = Path::new(&args.to).join(name.clone());
        let repo_path_str = repo_path.clone();
        let repo_path_str = repo_path_str.display();

        let mut cbs = RemoteCallbacks::new();
        cbs.credentials(|_, username, _| {
            Cred::ssh_key(
                username.unwrap(),
                None,
                Path::new("/home/patrick/.ssh/id_ed25519"),
                None,
            )
        });
        let mut opts = git2::FetchOptions::new();
        opts.remote_callbacks(cbs);

        match repo_path.exists() {
            true => match Repository::open(repo_path) {
                Ok(repo) => match repo.find_remote("origin") {
                    Ok(mut origin) => match origin.fetch(&["master"], Some(&mut opts), None) {
                        Ok(_) => {
                            println!("fetched origin/master for {}", name);
                        }
                        Err(e) => {
                            eprintln!("fetch origin/mater for repo {}: {}", name, e);
                        }
                    },
                    Err(e) => {
                        eprintln!("repo {} has no origin: {}", name, e);
                    }
                },
                Err(e) => {
                    eprintln!("repo {} in {} is broken: {}", name, repo_path_str, e);
                }
            },
            false => match builder.clone(&url, &repo_path) {
                Ok(r) => println!(
                    "cloned {url} to {}: {}",
                    repo_path_str,
                    r.workdir().unwrap().display()
                ),
                Err(e) => eprintln!("cloning {url} to {}: {}", repo_path_str, e),
            },
        }
    }
}
