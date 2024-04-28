use git2::{Error, Repository};
use reqwest::blocking::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::{fs, io, path::Path};

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

    /// The repository's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The repository's SSH URL.
    pub fn ssh_url(&self) -> &str {
        &self.ssh_url
    }

    /// The repository's path.
    pub fn path(&self) -> &str {
        &self.path.to_str().unwrap() // FIXME
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

/// Checks if dir exists and if it is a directory; creates the directory, if needed.
pub fn prepare_clone_dir(dir: &str) -> Result<Box<&Path>, String> {
    let path = Path::new(dir);
    match path.exists() {
        true => match path.is_dir() {
            true => Ok(Box::new(path)),
            false => Err("path exists, but is not a directory".to_string()),
        },
        false => match fs::create_dir(path) {
            Ok(_) => Ok(Box::new(path)),
            Err(e) => Err(e.to_string()),
        },
    }
}

/// Calls the GitHub /user/repos endpoint and returns a HashMap of SSH URLs by repository name.
pub fn fetch_repo_ssh_urls_by_name(github_token: String) -> HashMap<String, String> {
    let mut ssh_urls: HashMap<String, String> = HashMap::new();
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
                if let Ok(Value::Array(arr)) = res.json::<serde_json::Value>() {
                    if arr.len() < 1 {
                        break;
                    }
                    for repo in arr {
                        match (repo.get("name"), repo.get("ssh_url")) {
                            (Some(Value::String(name)), Some(Value::String(ssh_url))) => {
                                ssh_urls.insert(name.to_string(), ssh_url.to_string());
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
