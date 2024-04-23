use reqwest::blocking::Client;
use serde_json::Value;
use std::collections::HashMap;

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
                            (Some(name), Some(ssh_url)) => {
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
