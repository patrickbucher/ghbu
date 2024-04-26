# `ghbu`: GitHub Backup

## Design

Usage (backup the repositories owned by the user with the given `GITHUB_TOKEN` to `~/github-backup`): 

    $ GITHUB_TOKEN=0123456789abcdef ghbu --to ~/github-backup

## Dependencies

- [reqwest](https://crates.io/crates/reqwest): request repositories from GitHub API
- [git2](https://docs.rs/git2/latest/git2/): clone and pull repositories
- [clap](https://crates.io/crates/clap): parse command line arguments

## Design

- If the repository does not exist yet in the `--to` folder, clone it with as a
  bare repository (to save space).
    - `git clone --bare [url] [name]`
- If the repository already exists in the `--to` folder, fetch it.
    - `git fetch origin master:master`
    - figure out the branch name (here: master) using the following API
        - [get current head](https://docs.rs/git2/latest/git2/struct.Repository.html#method.head)
        - [make sure it is a branch](https://docs.rs/git2/latest/git2/struct.Reference.html#method.is_branch)
        - [get the branch name](https://docs.rs/git2/latest/git2/struct.Reference.html#method.name)

