# `ghbu`: GitHub Backup

## Design

Intended usage (backup the repositories of accounts `foo` and `bar` to
`~/github-backup/foo` and `~/github-backup/bar`, respectively):

    $ GITHUB_TOKEN=0123456789abcdef ghbu --to ~/github-backup foo bar

## Dependencies

- [reqwest](https://crates.io/crates/reqwest): request repositories from GitHub API
- [git2](https://docs.rs/git2/latest/git2/): clone and pull repositories
