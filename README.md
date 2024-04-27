# `ghbu`: GitHub Backup

## Design

Usage (backup the repositories owned by the user with the given `GITHUB_TOKEN`
to `~/github-backup`, access by the given SSH key): 

    $ GITHUB_TOKEN=0123abc ghbu --to ~/github-backup --keyfile ~/.ssh/id_ed25519

## Dependencies

- [reqwest](https://crates.io/crates/reqwest): request repositories from GitHub
  API
- [git2](https://docs.rs/git2/latest/git2/): clone and pull repositories
- [clap](https://crates.io/crates/clap): parse command line arguments

## Design

- If the repository does not exist yet in the `--to` folder, clone it with as a
  bare repository (to save space).
    - `git clone --bare [url] [name]`
- If the repository already exists in the `--to` folder, fetch it.
    - `git fetch origin master:master`
