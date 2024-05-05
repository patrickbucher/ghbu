use clap::Parser;
use ghbu::{create_callbacks, LocalRepo, Scope};
use git2::{build::RepoBuilder, FetchOptions};
use std::{env, path::Path, path::PathBuf, process, time::Duration};

const TOKEN_ENVVAR: &str = "GITHUB_TOKEN";

/// Backs up the GitHub repositories of the given accounts.
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// User
    #[arg(short, long)]
    user: Option<String>,

    /// Organization
    #[arg(short, long)]
    org: Option<String>,

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
    let args = Args::parse();

    let scope: Scope = if args.user.is_some() && args.org.is_some() {
        eprintln!("provide either user or org, but not both");
        process::exit(1);
    } else if let Some(name) = args.user {
        Scope {
            name: name.clone(),
            endpoint: "user/repos".into(),
            query: ("affiliation".into(), "owner".into()),
        }
    } else if let Some(name) = args.org {
        Scope {
            name: name.clone(),
            endpoint: format!("orgs/{}/repos", name.clone()),
            query: ("type".into(), "all".into()),
        }
    } else {
        eprintln!("provide either user or org, but not neither");
        process::exit(1);
    };

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

    let to: PathBuf = [&args.to, &scope.name].iter().collect();
    let path: &Path = match ghbu::prepare_clone_dir(to.as_path()) {
        Ok(path) => *path,
        Err(err) => {
            eprintln!("prepare clone directory {}: {}", args.to, err);
            process::exit(1);
        }
    };

    let all_repos = ghbu::fetch_repo_ssh_urls_by_name(github_token, scope);
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
            Ok(d) => println!(
                "{}: fetched to {} in {}",
                repo.name(),
                repo.display_path(),
                format_secs(d)
            ),
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
            Ok(d) => println!(
                "{}: cloned to {} in {}",
                repo.name(),
                repo.display_path(),
                format_secs(d)
            ),
            Err(e) => eprintln!("{}: cloning to {}: {}", repo.name(), repo.display_path(), e),
        }
    }
}

fn format_secs(d: Duration) -> String {
    let millis: u128 = d.as_millis().into();
    let (seconds, millis) = (millis / 1000, millis % 1000);
    format!("{seconds}.{millis}s")
}
