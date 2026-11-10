# Transfer to the Open Health Technology Foundation

Tracks [GOV-001](roadmap.md) in detail: moving `clincalc` from Marcus Baw's personal GitHub account (`pacharanero`) to the Open Health Technology Foundation (OHTF, <https://openhealthtechnology.org>), a UK community interest company (company number 17544421). Marcus remains the named maintainer throughout; this is a governance and continuity change, not a change of maintainership.

`clincalc` is already listed on the OHTF site as a **Candidate project** (<https://openhealthtechnology.org/projects/#clincalc>). Per OHTF's [project lifecycle](https://openhealthtechnology.org/governance/project-lifecycle/), the stages are Candidate -> Incubating -> Active -> Maintenance -> Archived. This plan takes `clincalc` from Candidate to Incubating. Reaching Active is a later milestone (DCO enforcement, full security-contact handover) and is not required to transfer the repository.

OHTF's own rule: "Projects keep their existing licences and copyright holders until they complete onboarding." The transfer does not change `clincalc`'s AGPL-3.0-or-later licence or its "Marcus Baw and Baw Medical Ltd" SPDX copyright headers unless a separate, explicit decision is made later.

## Status

- [x] `clincalc` listed as a Candidate project on openhealthtechnology.org.
- [x] `ohtf/clincalc` GitHub team already exists, with Marcus (`pacharanero`) as a team maintainer - this is the team crates.io ownership was meant to reference.
- [x] `SECURITY.md` added at repo root.
- [x] All six decisions below made (2026-11-10).
- [x] `ohtf/homebrew-tap` created and seeded (empty `Formula/`); `Cargo.toml`'s `tap` and `release.yml`'s `publish-homebrew-formula` job repointed to it.
- [x] Tauri bundle identifier changed to `io.github.ohtf.clincalc`.
- [ ] **Blocked**: `cargo owner --add github:ohtf:clincalc` failed - the stored crates.io token lacks the `change-owners` scope (crates.io API tokens are scope-limited; this one was issued for publish-only use). See "Blocked - needs your action" below.
- [ ] **Blocked**: `HOMEBREW_TAP_TOKEN` repo secret still grants access to `pacharanero/homebrew-tap` only, not `ohtf/homebrew-tap`. The next release's `publish-homebrew-formula` job will fail (fail-closed, not silent) until this is replaced.
- [ ] **Blocked**: PyPI secondary maintainer invite - needs the project owner to add the OHTF PyPI account as a collaborator on `clincalc` via PyPI's web UI; no API access available to do this directly.
- [ ] The GitHub repository transfer itself (Phase 2) and everything downstream of it (Phases 3-5).

## What is already in good shape

- Licence is AGPL-3.0-or-later, OSI-approved: satisfies OHTF's "open source, or willing to be" test and the Candidate -> Incubating open-source-commitment agreement without any licence change.
- Clinical safety position is already documented (`clinical-safety/SAFETY.md`, `SAFETY-CASE.md`, `HAZARD-LOG.md`), satisfying "honest about safety."
- `publish-crates.yml` and `publish-python.yml` already use crates.io and PyPI **Trusted Publishing** (GitHub OIDC) rather than long-lived tokens. No secret rotation is needed on transfer - only repointing the trust relationship to the new repository path (see below).
- Marcus is already a named maintainer with a security-relevant history of responsible dependency/CVE handling (see `spec/roadmap.md` ENG-008).

## Decisions (resolved 2026-11-10)

1. **Repository name**: keep `clincalc`. No action needed until Phase 2.
2. **Homebrew tap**: OHTF gets its own tap, not a shared repurposing of `pacharanero/homebrew-tap` (which stays as-is for `sct`/`dsc-rs`/`gitehr`, none of which are OHTF projects yet). Done: `ohtf/homebrew-tap` created, seeded with an empty `Formula/` directory and a README; `Cargo.toml`'s `tap` and `release.yml`'s `publish-homebrew-formula` job (checkout target and environment URL) repointed to it.
3. **GitHub Pages domain**: default `ohtf.github.io/clincalc/` for now, no custom domain. No action needed until Phase 2/3 - this is what happens automatically on transfer.
4. **crates.io co-ownership**: add `github:ohtf:clincalc` as a second owner. **Attempted, blocked** - see "Blocked - needs your action" below.
5. **PyPI secondary maintainer**: OHTF already has a shared PyPI account (it publishes `nhs-number`). Add it as a second maintainer on the `clincalc` PyPI project rather than relying solely on Trusted Publishing. **Needs your action** - see below.
6. **Tauri bundle identifier**: changed to `io.github.ohtf.clincalc`. Done - safe, since no signed GUI build has ever shipped.

## Blocked - needs your action

1. **crates.io owner-add**: `cargo owner --add github:ohtf:clincalc` failed with `403 Forbidden: this token does not have the required permissions to perform this action`. The token in `~/.cargo/credentials.toml` is scope-limited (likely publish-only, not `change-owners`). Fix: go to <https://crates.io/settings/tokens>, create a new token with the "Change owners" scope (or a legacy full-access token), `cargo login` with it, then run `cargo owner --add github:ohtf:clincalc` (I can run the command once the token is in place - just say so). Alternatively, add the owner directly via <https://crates.io/crates/clincalc/settings> in the browser, no token needed there.
2. **`HOMEBREW_TAP_TOKEN` secret**: currently a PAT scoped to `pacharanero/homebrew-tap` (repo secret on `pacharanero/clincalc`, set 2026-07-05). It has no access to the new `ohtf/homebrew-tap`. Fix: create a new PAT (fine-grained, scoped to just `ohtf/homebrew-tap`, contents: read/write) and either update the existing secret (`gh secret set HOMEBREW_TAP_TOKEN --repo pacharanero/clincalc`) or tell me the new token value and I will. Until this is done, `publish-homebrew-formula` will fail loudly on the next release (not publish silently to the wrong place).
3. **PyPI secondary maintainer invite**: no PyPI API access available to do this directly. Go to <https://pypi.org/manage/project/clincalc/collaboration/> and invite the OHTF PyPI account as a maintainer (or owner, matching however `nhs-number` is set up).

## Execution order

### Phase 1 - safe to do now, before any transfer (reversible, no external side effects beyond crates.io/Homebrew/PyPI ownership)

1. [x] Add `SECURITY.md` at repo root - house style requires one on every public repo (`~/code/house-style/security.md`); also satisfies part of OHTF's Incubating gate.
2. [ ] `cargo owner --add github:ohtf:clincalc` - blocked on token scope, see above.
3. [x] `gui/src-tauri/tauri.conf.json`'s `identifier` updated to `io.github.ohtf.clincalc`.
4. [x] `ohtf/homebrew-tap` created; `Cargo.toml` and `release.yml` repointed to it. `HOMEBREW_TAP_TOKEN` secret still needs replacing, see above.
5. [ ] Agree OHTF's [open-source commitment](https://openhealthtechnology.org/governance/open-source-commitment/) applies to `clincalc` - this is the last Candidate -> Incubating gate, and is a statement, not a code change. Record the agreement date here once done.

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

`Cargo.toml`'s `tap` and `release.yml`'s `publish-homebrew-formula` already point at `ohtf/homebrew-tap` (done ahead of the transfer, since the tap is a separate repo from the decision of where `clincalc`'s own source lives).

After the sed pass, hand-check (sed will not catch these):

- `Cargo.toml`'s `repository`/`homepage` fields - confirm they parse and `cargo publish --dry-run` still succeeds.
- `README.md` - check for any badge URLs (shields.io, GitHub Actions status badges) that embed the old owner path; these commonly use a different URL shape than the plain repo links the sed pass catches.

### Phase 4 - reconfigure external publishing (manual, web UI, cannot be scripted)

1. **crates.io Trusted Publishing**: crates.io crate settings -> Trusted Publishing -> update the configured GitHub repository owner from `pacharanero` to `ohtf` (repo name and workflow path stay `clincalc` / `publish-crates.yml`). Until this is done, the next tagged release's `publish-crates.yml` run will fail at the OIDC handshake, not silently publish from the wrong identity - fail-closed, but do this before the next release.
2. **PyPI Trusted Publishing**: PyPI project settings -> Publishing -> same update, owner `pacharanero` -> `ohtf`.
3. **`HOMEBREW_TAP_TOKEN`**: replace with a PAT scoped to `ohtf/homebrew-tap` if not already done in Phase 1.
4. **GitHub Pages**: confirm the Pages deployment still targets the right branch/path post-transfer (should be automatic, but verify `docs.yml` ran clean on the new repo).
5. Verify a real dry-run: tag a patch release (or use the existing `s/check-release-version` / `s/check-github-release` scripts against a test tag) and confirm the full auto-tag -> cargo-dist -> Homebrew -> crates.io -> PyPI chain completes against the new repo path before relying on it for a real release.

### Phase 5 - update the OHTF site

The OHTF website's project entry for `clincalc` already exists (candidate stage). Open a PR against `ohtf/openhealthtechnology.org-website` moving the `clincalc` entry from "Candidate projects" to "Foundation Projects" (Incubating), once Phase 1-4 are complete.

## What this plan deliberately does not do yet

- Does not change the AGPL-3.0-or-later licence or the SPDX copyright headers.
- Does not pursue DCO-enforcement CI or Active-stage status - that is a follow-on, not a blocker for the transfer itself.
- Does not touch `pacharanero/homebrew-tap`, which stays exactly as-is for `sct`/`dsc-rs`/`gitehr`.
- Does not attempt the GitHub repository transfer itself, which only the current owner can initiate from GitHub's UI.
