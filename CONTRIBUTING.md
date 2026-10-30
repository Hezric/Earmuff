# Contributing

Please open an issue in [Hezric/Earmuff](https://github.com/Hezric/Earmuff) before a substantial change. Specifications use parent issues; implementation uses native sub-issues. The labels `ready-for-agent`, `in-progress`, `blocked`, and `done` record delivery state.

Use a `<type>/<short-desc>` branch and signed, scoped Conventional Commits, such as `fix(audio): handle a disconnected endpoint`. Public contributions are English; keep the Chinese README synchronized. [AGENTS.md](AGENTS.md) describes the repository workflow.

Run the checks in [CI](.github/workflows/ci.yml). Keep audio tests simulated. Launching Earmuff immediately enables protection and can lower the real default output's volume; get the user's authorization before a hardware check. `vp run dev` previews only the frontend and is safe.

Open one PR per coherent change with the behavior and test results. Required jobs are `test` and `policy`. Main requires signed, tested, linear history. Rebase stale branches locally and re-sign, then use `--force-with-lease` only on your branch. Maintainers squash-merge using the PR title and body and verify the resulting signature. Weekly grouped Dependabot updates use the same checks and squash policy.

Release only on the user's instruction. The reminder issue never authorizes publication. The release workflow consumes a tested, signed annotated version tag; protected tags are immutable. See the release procedure in AGENTS.md.

Contributions are licensed under MIT. Report security issues privately through the repository Security tab when available.
