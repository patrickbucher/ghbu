# `ghbu`: GitHub Backup

## Design

Intended usage (backup the repositories of accounts `foo` and `bar` to
`~/github-backup/foo` and `~/github-backup/bar`, respectively):

    $ GITHUB_TOKEN=0123456789abcdef ghbu --to ~/github-backup foo bar

## Dependencies

- [reqwest](https://crates.io/crates/reqwest): request repositories from GitHub API
- [git2](https://docs.rs/git2/latest/git2/): clone and pull repositories
- [clap](https://crates.io/crates/clap): parse command line arguments

## Open Points

- How does the `git2` library deal with the SSH key?
- What if a repository is backed up, deleted, re-created under the same name,
  and therefore, cannot be pulled again? (Report it to the user, so that he can
  delete the old backed up repository manually? Provide a `--force` flag to get
  rid of the old version?)
- Shall there be a `--dry` run option, especially for testing the API access in
  the beginning?
- Shall the blocking reqwest client be used in the beginning for the sake of
  simplicity?
