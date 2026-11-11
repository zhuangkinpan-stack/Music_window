# Issue Tracker: GitHub

Issues and specs for this repository live in GitHub Issues.

## Repository

`zhuangkinpan-stack/Music_window`

Use the `gh` CLI for issue operations:

- Create: `gh issue create --title "..." --body "..."`
- Read: `gh issue view <number> --json number,title,body,labels,comments`
- List: `gh issue list --state open --json number,title,body,labels,comments`
- Comment: `gh issue comment <number> --body "..."`
- Edit labels: `gh issue edit <number> --add-label "..."`
- Close: `gh issue close <number>`

Before creating issues, verify the target repository:

```powershell
gh repo view --json nameWithOwner
```

## Pull requests as a triage surface

**No.** External pull requests are not treated as feature requests.

