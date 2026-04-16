# gh PR target — always specify the fork

When creating pull requests with `gh pr create`, always pass
`--repo barrie-cork/lemmy`. Without it, `gh` defaults to the upstream
`LemmyNet/lemmy` because this repo is a fork.

```bash
# Correct
gh pr create --repo barrie-cork/lemmy --title "..." --body "..."

# Wrong — targets upstream LemmyNet/lemmy
gh pr create --title "..." --body "..."
```

This applies to all `gh pr` subcommands that accept `--repo`:
`create`, `view`, `list`, `merge`, `close`, etc.
