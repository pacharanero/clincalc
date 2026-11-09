# Transfer to the Open Health Technology Foundation

Tracks [GOV-001](roadmap.md) in detail: moving `clincalc` from Marcus Baw's personal GitHub account (`pacharanero`) to the Open Health Technology Foundation (OHTF, <https://openhealthtechnology.org>), a UK community interest company (company number 17544421). Marcus remains the named maintainer throughout; this is a governance and continuity change, not a change of maintainership.

`clincalc` is already listed on the OHTF site as a **Candidate project** (<https://openhealthtechnology.org/projects/#clincalc>). Per OHTF's [project lifecycle](https://openhealthtechnology.org/governance/project-lifecycle/), the stages are Candidate -> Incubating -> Active -> Maintenance -> Archived. This plan takes `clincalc` from Candidate to Incubating. Reaching Active is a later milestone (DCO enforcement, full security-contact handover) and is not required to transfer the repository.

OHTF's own rule: "Projects keep their existing licences and copyright holders until they complete onboarding." The transfer does not change `clincalc`'s AGPL-3.0-or-later licence or its "Marcus Baw and Baw Medical Ltd" SPDX copyright headers unless a separate, explicit decision is made later.

## Status

- [x] `clincalc` listed as a Candidate project on openhealthtechnology.org.
- [x] `ohtf/clincalc` GitHub team already exists, with Marcus (`pacharanero`) as a team maintainer - this is the team crates.io ownership will reference.
- [x] `SECURITY.md` added at repo root (this change).
- [ ] Everything below.

## What is already in good shape

- Licence is AGPL-3.0-or-later, OSI-approved: satisfies OHTF's "open source, or willing to be" test and the Candidate -> Incubating open-source-commitment agreement without any licence change.
- Clinical safety position is already documented (`clinical-safety/SAFETY.md`, `SAFETY-CASE.md`, `HAZARD-LOG.md`), satisfying "honest about safety."
- `publish-crates.yml` and `publish-python.yml` already use crates.io and PyPI **Trusted Publishing** (GitHub OIDC) rather than long-lived tokens. No secret rotation is needed on transfer - only repointing the trust relationship to the new repository path (see below).
- Marcus is already a named maintainer with a security-relevant history of responsible dependency/CVE handling (see `spec/roadmap.md` ENG-008).

## Decisions needed before executing (ask, don't assume)

1. **Repository name.** No name clash exists under `github.com/ohtf` (unlike `nhs-number`, which became `nhs-number-py` to disambiguate from the Rust port). Recommend keeping the name `clincalc`. Confirm.
2. **Homebrew tap.** `pacharanero/homebrew-tap` is shared across `clincalc`, `sct`, `dsc-rs`, and `gitehr` - it is not `clincalc`-specific. Moving it would affect three other projects that have nothing to do with OHTF yet. Recommend **leaving the tap where it is**; the `clincalc.rb` formula inside it just needs its `url`/`homepage` fields updated to point at the new repo path. Confirm, or decide OHTF should have its own tap instead.
3. **GitHub Pages domain.** No CNAME is currently set; docs are served at `pacharanero.github.io/clincalc/`. After transfer this becomes `ohtf.github.io/clincalc/` by default. OHTF's own policy says it can "hold domains... where that protects the project's independence." Decide: default `ohtf.github.io` URL, or a subdomain of `openhealthtechnology.org` (e.g. `clincalc.openhealthtechnology.org`)? The latter is nicer but is a new piece of DNS/Pages config to set up and maintain.
4. **crates.io co-ownership.** `cargo owner --add github:ohtf:clincalc` can be run **today**, independent of the GitHub repo transfer - it only requires the acting crates.io account (`pacharanero`) to be a member of the `ohtf/clincalc` GitHub team, which is already true. This adds the team as a second owner alongside `pacharanero`, so publish/yank rights do not depend solely on one personal account. It is additive and reversible (`cargo owner --remove` later). Recommend doing this now. Confirm before it is run, since it changes who can act on the published crate.
5. **PyPI secondary maintainer.** PyPI has no GitHub-team-owner equivalent to crates.io's. Options: (a) rely solely on Trusted Publishing once repointed to `ohtf/clincalc` (no human PyPI account has standing access at all - arguably the most secure option, but means only a repo admin via CI can ever publish, never a human from a laptop), or (b) add a shared OHTF PyPI account as a second maintainer if one exists or is created. Decide which; (a) is recommended unless OHTF already has a shared PyPI account for other projects.
6. **Tauri desktop bundle identifier.** `gui/src-tauri/tauri.conf.json` currently has `"identifier": "io.github.pacharanero.calc"`. The GUI has not shipped any signed/distributed build yet (still pre-release per `spec/gui.md`), so this is safe to change now, before it is baked into any installed-app identity, update manifest, or code-signing certificate. Recommend `io.github.ohtf.clincalc`. Confirm the exact string.

## Execution order

### Phase 1 - safe to do now, before any transfer (reversible, no external side effects beyond crates.io ownership)

1. Add `SECURITY.md` at repo root (this change) - house style requires one on every public repo (`~/code/house-style/security.md`); also satisfies part of OHTF's Incubating gate.
2. Once decision 4 above is confirmed: `cargo owner --add github:ohtf:clincalc`. Verify with `cargo owner --list`.
3. Once decision 6 above is confirmed: update `gui/src-tauri/tauri.conf.json`'s `identifier` and re-test the GUI still launches (`s/gui-dev`).
4. Agree OHTF's [open-source commitment](https://openhealthtechnology.org/governance/open-source-commitment/) applies to `clincalc` - this is the last Candidate -> Incubating gate, and is a statement, not a code change. Record the agreement date here once done.

### Phase 2 - the transfer itself (manual, by Marcus, on github.com)

This step cannot be done through the GitHub CLI/API safely as an automated action - a repository transfer is initiated by the current owner and accepted by an org owner, and GitHub prompts for confirmation precisely because it is consequential (it changes the canonical URL, every clone/fork relationship, and webhook/app installations).

1. GitHub repo settings -> "Transfer ownership" -> `ohtf`.
2. GitHub creates an automatic redirect from `pacharanero/clincalc` to `ohtf/clincalc` for git operations and most URLs, but redirects are not forever and should not be relied on long-term - hence Phase 3 happens immediately after, not "eventually."
3. Confirm the `ohtf/clincalc` team (already existing) is attached to the transferred repo with maintain/admin access, and that Marcus's access is unchanged.

### Phase 3 - immediately after the transfer (one coordinated commit)

Every file below contains a `pacharanero/clincalc` or `pacharanero.github.io/clincalc` reference that becomes stale the moment the transfer completes. Run as a single find-and-replace pass, review the diff, then run the full gate (`cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `npm run build` in `gui/`) before committing:

```bash
# Run from the repo root, immediately after the GitHub transfer completes.
files=(
  Cargo.toml
  python/Cargo.toml
  python/pyproject.toml
  python/README.md
  gui/src-tauri/Cargo.toml
  gui/src/App.tsx
  mkdocs.yml
  cliff.toml
  src/api.rs
  third-party-notices.md
  docs/calculators.md
  docs/case-studies.md
  docs/index.md
  docs/install.md
  docs/mcp.md
  docs/python.md
  docs/translating.md
  examples/README.md
  spec/calculators.md
  .github/workflows/docs.yml
  AGENTS.md
)
sed -i 's#github\.com/pacharanero/clincalc#github.com/ohtf/clincalc#g; s#pacharanero\.github\.io/clincalc#ohtf.github.io/clincalc#g' "${files[@]}"
```

Do **not** touch `CHANGELOG.md` - it is a historical record of what was true at each past release and should not be rewritten.

Leave `.github/workflows/release.yml`'s `pacharanero/homebrew-tap` references alone unless decision 2 above changes (the tap itself is not moving).

After the sed pass, hand-check (sed will not catch these):

- `Cargo.toml`'s `repository`/`homepage` fields - confirm they parse and `cargo publish --dry-run` still succeeds.
- `README.md` - check for any badge URLs (shields.io, GitHub Actions status badges) that embed the old owner path; these commonly use a different URL shape than the plain repo links the sed pass catches.
- The Homebrew formula `clincalc.rb` inside `pacharanero/homebrew-tap` - its `url`/`homepage` need the new path even though the tap repo itself stays put.

### Phase 4 - reconfigure external publishing (manual, web UI, cannot be scripted)

1. **crates.io Trusted Publishing**: crates.io crate settings -> Trusted Publishing -> update the configured GitHub repository owner from `pacharanero` to `ohtf` (repo name and workflow path stay `clincalc` / `publish-crates.yml`). Until this is done, the next tagged release's `publish-crates.yml` run will fail at the OIDC handshake, not silently publish from the wrong identity - fail-closed, but do this before the next release.
2. **PyPI Trusted Publishing**: PyPI project settings -> Publishing -> same update, owner `pacharanero` -> `ohtf`.
3. **GitHub Pages**: confirm the Pages deployment still targets the right branch/path post-transfer (should be automatic, but verify `docs.yml` ran clean on the new repo).
4. Verify a real dry-run: tag a patch release (or use the existing `s/check-release-version` / `s/check-github-release` scripts against a test tag) and confirm the full auto-tag -> cargo-dist -> Homebrew -> crates.io -> PyPI chain completes against the new repo path before relying on it for a real release.

### Phase 5 - update the OHTF site

The OHTF website's project entry for `clincalc` already exists (candidate stage). Open a PR against `ohtf/openhealthtechnology.org-website` moving the `clincalc` entry from "Candidate projects" to "Foundation Projects" (Incubating), once Phase 1-4 are complete.

## What this plan deliberately does not do yet

- Does not change the AGPL-3.0-or-later licence or the SPDX copyright headers.
- Does not pursue DCO-enforcement CI or Active-stage status - that is a follow-on, not a blocker for the transfer itself.
- Does not touch the shared `pacharanero/homebrew-tap` repository's ownership, only the one formula file inside it.
- Does not attempt the GitHub repository transfer itself, which only the current owner can initiate from GitHub's UI.
