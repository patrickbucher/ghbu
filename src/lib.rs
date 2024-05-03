use git2::build::RepoBuilder;
use git2::{Cred, Error, ErrorClass, ErrorCode, FetchOptions, RemoteCallbacks, Repository};
use reqwest::blocking::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::{fs, io, path::Display, path::Path};

/// Scope determines if the user's or an organization's repositories are backed up.
pub struct Scope {
    pub name: String,
    pub endpoint: String,
    pub query: (String, String),
}

/// LocalRepo is a Git repository with a local path.
pub struct LocalRepo {
    name: String,
    ssh_url: String,
    path: Box<Path>,
}

impl LocalRepo {
    /// Builds a LocalRepo with its target path within `base_dir`.
    pub fn new(name: String, ssh_url: String, base_dir: &Path) -> LocalRepo {
        let path = base_dir.join(&name).into_boxed_path();
        LocalRepo {
            name,
            ssh_url,
            path,
        }
    }

    /// Clones the repository from `ssh_url` to `path`.
    pub fn clone(&self, builder: &mut RepoBuilder) -> Result<Repository, Error> {
        builder.clone(&self.ssh_url, &self.path)
    }

    /// Fetches the repository's HEAD.
    pub fn fetch(&self, options: &mut FetchOptions) -> Result<(), Error> {
        let repo = self.open_bare()?;
        let mut origin = repo.find_remote("origin")?;
        let head = repo.head()?;
        if !head.is_branch() {
            return Err(Error::new(
                ErrorCode::NotFound,
                ErrorClass::Reference,
                "HEAD does not refer to a branch",
            ));
        }
        let branch = match head.shorthand() {
            Some(b) => b,
            None => {
                return Err(Error::new(
                    ErrorCode::NotFound,
                    ErrorClass::Reference,
                    "unable to get branch for HEAD",
                ))
            }
        };
        origin.fetch(&[branch], Some(options), None)
    }

    /// The repository's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The repository's path.
    pub fn display_path(&self) -> Display {
        self.path.display()
    }

    /// Checks whether or not the LocalRepo's path refers to an existing directory.
    pub fn existing_dir(&self) -> bool {
        self.path.exists() && self.path.is_dir()
    }

    /// Tries to open the LocalRepo as bare repository through its path.
    pub fn open_bare(&self) -> Result<Repository, Error> {
        Repository::open_bare(&self.path)
    }

    /// Deletes the LocalRepo's path recursively.
    pub fn annihilate(&self) -> io::Result<()> {
        fs::remove_dir_all(&self.path)
    }
}

/// Creates the callbacks using SSH credentials.
pub fn create_callbacks(keyfile: &Path) -> RemoteCallbacks {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|url, username, _| {
        match username {
            Some(u) => Cred::ssh_key(u, None, keyfile, None /* TODO: provide passphrase */),
            None => Err(Error::new(
                ErrorCode::User,
                ErrorClass::Invalid,
                format!("cannot determine username from URL {url}"),
            )),
        }
    });
    callbacks
}

/// Checks if dir exists and if it is a directory; creates the directory, if needed.
pub fn prepare_clone_dir(path: &Path) -> Result<Box<&Path>, String> {
    match path.exists() {
        true => match path.is_dir() {
            true => Ok(Box::new(path)),
            false => Err("path exists, but is not a directory".into()),
        },
        false => match fs::create_dir_all(path) {
            Ok(_) => Ok(Box::new(path)),
            Err(e) => Err(e.to_string()),
        },
    }
}

/// Calls the GitHub endpoint for the scope and returns a HashMap of SSH URLs by repository name.
pub fn fetch_repo_ssh_urls_by_name(github_token: String, scope: Scope) -> HashMap<String, String> {
    let mut ssh_urls: HashMap<String, String> = HashMap::new();
    let client = Client::new();
    let url = format!("https://api.github.com/{}", scope.endpoint);
    let mut page = 1;
    loop {
        let req = client
            .get(url.clone())
            .bearer_auth(&github_token)
            .header("Accept", "application/json")
            .header("User-Agent", "reqwest")
            .query(&[
                ("per_page", "20"),
                ("page", &page.to_string()),
                (&scope.query.0, &scope.query.1),
            ]);
        match req.send() {
            Ok(res) => {
                if let Ok(Value::Array(arr)) = res.json::<serde_json::Value>() {
                    if arr.is_empty() {
                        break;
                    }
                    for repo in arr {
                        match (repo.get("name"), repo.get("ssh_url")) {
                            (Some(Value::String(name)), Some(Value::String(ssh_url))) => {
                                ssh_urls.insert(name.into(), ssh_url.into());
                            }
                            _ => eprintln!("skipping repo (missing name/ssh_url)"),
                        }
                    }
                }
            }
            Err(e) => eprintln!("request failed: {:?}", e),
        }
        page += 1;
    }
    ssh_urls
}
