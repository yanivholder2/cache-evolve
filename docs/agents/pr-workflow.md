# PR workflow

## Leaving feedback on PRs

Use **review thread comments** (Files changed tab), not plain conversation comments. Review thread comments create resolvable conversations tracked by a counter ("N of M conversations resolved") in the Files changed tab. Plain comments have no resolved/unresolved state.

Add a review thread comment via `gh api`:

```bash
gh api repos/<owner>/<repo>/pulls/<number>/comments -X POST \
  --field body="..." \
  --field commit_id="<head SHA>" \
  --field path="<file path>" \
  --field subject_type="file"
```

Get the head SHA from: `gh api repos/<owner>/<repo>/pulls/<number> --jq '.head.sha'`

## Blocking a PR

"Request changes" is not available on your own PRs. For self-authored PRs, use:
- A review thread comment (resolvable, visible in Files changed)

## Branch protection

"Require all conversations resolved before merging" is a branch protection rule — only available on public repos or paid GitHub plans. On a free private repo, resolution is honor-system only.
