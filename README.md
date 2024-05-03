# `ghbu`: GitHub Backup

## Design

Usage (backup the repositories owned by the user with the given `GITHUB_TOKEN`
to `~/github-backup`, access by the given SSH key): 

    $ GITHUB_TOKEN=0123abc ghbu --to ~/github-backup --keyfile ~/.ssh/id_ed25519

## Dependencies

- [`git2`](https://docs.rs/git2/latest/git2/): clone and fetch Git repositories
- [`reqwest`](https://crates.io/crates/reqwest): request repositories from GitHub API
- [`serde_json`](https://crates.io/crates/serde_json): unmarshal GitHub API payloads
- [`clap`](https://crates.io/crates/clap): parse command line arguments

## TODO

- [ ] Concurrency
    - [ ] interleave API calls with cloning/fetching of repositories
- [ ] Extension
    - [ ] implement for GitLab API
    - [ ] implement for Gitea API
    - [ ] Backup an Organization's Repositories
        - `GET /orgs/{org}/repos`
        - if `--org ORG` is present, backup organization to `TO/ORG`
        - otherwise, backup private repos to `TO/USERNAME`
            - need to pre-fetch username by token?
            - or indicate with `--user USER` explicitly?
            - `--user` and `--org` as mutually exclusive CLI paramters
