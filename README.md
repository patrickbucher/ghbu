# `ghbu`: GitHub Backup

## Usage

Backup the repositories owned by user `joe`:

    $ GITHUB_TOKEN=0123abc ghbu --to ~/github-backup --keyfile ~/.ssh/id_ed25519 --user joe

Backup the repositories owned by organization `acme`:

    $ GITHUB_TOKEN=0123abc ghbu --to ~/github-backup --keyfile ~/.ssh/id_ed25519 --org acme

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
    - [ ] consider supporting HTTPS credentials
    - [ ] consider supporting SSH Key from Agent
    - [ ] add support for SSH Passphrase (as environment variable)
    - [ ] consider measuring timing and/or displaying progress
