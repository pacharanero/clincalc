# Transfer to the Open Health Technology Foundation

Tracks [GOV-001](roadmap.md) in detail: moving `clincalc` from Marcus Baw's personal GitHub account (`pacharanero`) to the Open Health Technology Foundation (OHTF, <https://openhealthtechnology.org>), a UK community interest company (company number 17544421). Marcus remains the named maintainer throughout; this is a governance and continuity change, not a change of maintainership.

`clincalc` is already listed on the OHTF site as a **Candidate project** (<https://openhealthtechnology.org/projects/#clincalc>). Per OHTF's [project lifecycle](https://openhealthtechnology.org/governance/project-lifecycle/), the stages are Candidate -> Incubating -> Active -> Maintenance -> Archived. This plan takes `clincalc` from Candidate to Incubating. Reaching Active is a later milestone (DCO enforcement, full security-contact handover) and is not required to transfer the repository.

OHTF's own rule: "Projects keep their existing licences and copyright holders until they complete onboarding." The transfer does not change `clincalc`'s AGPL-3.0-or-later licence or its "Marcus Baw and Baw Medical Ltd" SPDX copyright headers unless a separate, explicit decision is made later.

## Status

**The repository has transferred.** `github.com/ohtf/clincalc` is now the canonical location; `pacharanero/clincalc` redirects to it. Phases 1-3 below are complete.

- [x] `clincalc` listed as a Candidate project on openhealthtechnology.org.
- [x] `ohtf/clincalc` GitHub team already exists, with Marcus (`pacharanero`) as a team maintainer.
- [x] `SECURITY.md` added at repo root.
- [x] All six decisions recorded below made (2026-11-10).
- [x] `ohtf/homebrew-tap` created and seeded (empty `Formula/`); `Cargo.toml`'s `tap` and `release.yml`'s `publish-homebrew-formula` job repointed to it.
- [x] Tauri bundle identifier changed to `io.github.ohtf.clincalc`.
- [x] **crates.io co-ownership**: `github:ohtf:clincalc` added as a second owner via the crates.io web UI (2026-11-10) - confirmed via `crates.io/api/v1/crates/clincalc/owners`.
- [x] **GitHub repository transferred** to `ohtf/clincalc` (2026-11-10, by Marcus via GitHub's UI). Local `origin` remote updated.
- [x] **All `pacharanero/clincalc` and `pacharanero.github.io/clincalc` references repointed** across Cargo.toml, docs, workflows, the GUI, and the Python package (commit `f542f452`, with two follow-up catches in `docs/index.md`/`docs/install.md`/`python/README.md`).
- [x] **Incidental fix**: a GitHub-side `ubuntu-latest` runner-image rollout broke the Python 3.9 CI job the same morning (unrelated to the transfer); pinned to `ubuntu-24.04`, then moot once 3.9/3.10 support was dropped entirely (see `CHANGELOG.md` / commit `dccde896`).
- [ ] **Blocked**: `HOMEBREW_TAP_TOKEN` repo secret still grants access to `pacharanero/homebrew-tap` only, not `ohtf/homebrew-tap`. The next release's `publish-homebrew-formula` job will fail (fail-closed, not silent) until this is replaced.
- [ ] **Blocked**: PyPI secondary maintainer invite - needs the project owner to add the OHTF PyPI account as a collaborator on `clincalc` via PyPI's web UI; no API access available to do this directly.
- [ ] **Not yet done**: crates.io Trusted Publishing and PyPI Trusted Publishing still trust `pacharanero/clincalc`, not `ohtf/clincalc` - see Phase 4. The next tagged release will fail the OIDC handshake on both until this is fixed.
- [ ] Phase 5: update the OHTF site's project entry from Candidate to Incubating.

## What is already in good shape

- Licence is AGPL-3.0-or-later, OSI-approved: satisfies OHTF's "open source, or willing to be" test and the Candidate -> Incubating open-source-commitment agreement without any licence change.
- Clinical safety position is already documented (`clinical-safety/SAFETY.md`, `SAFETY-CASE.md`, `HAZARD-LOG.md`), satisfying "honest about safety."
- `publish-crates.yml` and `publish-python.yml` already use crates.io and PyPI **Trusted Publishing** (GitHub OIDC) rather than long-lived tokens. No secret rotation is needed on transfer - only repointing the trust relationship to the new repository path (see below).
- Marcus is already a named maintainer with a security-relevant history of responsible dependency/CVE handling (see `spec/roadmap.md` ENG-008).

## Decisions (resolved 2026-11-10)

1. **Repository name**: keep `clincalc`. No action needed until Phase 2.
2. **Homebrew tap**: OHTF gets its own tap, not a shared repurposing of `pacharanero/homebrew-tap` (which stays as-is for `sct`/`dsc-rs`/`gitehr`, none of which are OHTF projects yet). Done: `ohtf/homebrew-tap` created, seeded with an empty `Formula/` directory and a README; `Cargo.toml`'s `tap` and `release.yml`'s `publish-homebrew-formula` job (checkout target and environment URL) repointed to it.
3. **GitHub Pages domain**: default `ohtf.github.io/clincalc/` for now, no custom domain. No action needed until Phase 2/3 - this is what happens automatically on transfer.
4. **crates.io co-ownership**: add `github:ohtf:clincalc` as a second owner. Done (2026-11-10) - added via the crates.io web UI after the scoped-token route (`cargo owner --add`) hit a 403 on a publish-only token.
5. **PyPI secondary maintainer**: OHTF already has a shared PyPI account (it publishes `nhs-number`). Add it as a second maintainer on the `clincalc` PyPI project rather than relying solely on Trusted Publishing. **Needs your action** - see below.
6. **Tauri bundle identifier**: changed to `io.github.ohtf.clincalc`. Done - safe, since no signed GUI build has ever shipped.

## Blocked - needs your action

1. **`HOMEBREW_TAP_TOKEN` secret**: currently a PAT scoped to `pacharanero/homebrew-tap` (repo secret, set 2026-07-05, now living on `ohtf/clincalc` since the transfer carried repo secrets over). It has no access to `ohtf/homebrew-tap`. Fix: create a new PAT (fine-grained, scoped to just `ohtf/homebrew-tap`, contents: read/write) and either update the existing secret (`gh secret set HOMEBREW_TAP_TOKEN --repo ohtf/clincalc`) or tell me the new token value and I will. Until this is done, `publish-homebrew-formula` will fail loudly on the next release (not publish silently to the wrong place).
2. **PyPI secondary maintainer invite**: no PyPI API access available to do this directly. Go to <https://pypi.org/manage/project/clincalc/collaboration/> and invite the OHTF PyPI account as a maintainer (or owner, matching however `nhs-number` is set up).
3. **crates.io Trusted Publishing repoint** (Phase 4.1 below): go to <https://crates.io/crates/clincalc/settings> and update the configured GitHub repository owner from `pacharanero` to `ohtf`. Needed before the next tagged release - `publish-crates.yml` will otherwise fail the OIDC handshake.
4. **PyPI Trusted Publishing repoint** (Phase 4.2 below): go to <https://pypi.org/manage/project/clincalc/settings/publishing/> and make the equivalent change.

## Execution order

### Phase 1 - done

1. [x] `SECURITY.md` added at repo root.
2. [x] `github:ohtf:clincalc` added as a crates.io co-owner (via the web UI, not `cargo owner --add`).
3. [x] `gui/src-tauri/tauri.conf.json`'s `identifier` updated to `io.github.ohtf.clincalc`.
4. [x] `ohtf/homebrew-tap` created; `Cargo.toml` and `release.yml` repointed to it. `HOMEBREW_TAP_TOKEN` secret still needs replacing - see "Blocked" above.
5. [ ] Agree OHTF's [open-source commitment](https://openhealthtechnology.org/governance/open-source-commitment/) applies to `clincalc` - the last Candidate -> Incubating gate, a statement not a code change. Still outstanding.

### Phase 2 - done: the transfer itself

Completed 2026-11-10 by Marcus via GitHub's "Transfer ownership" UI. `pacharanero/clincalc` now redirects to `ohtf/clincalc`; the local `origin` remote was updated to match. GitHub's redirect is not permanent, which is why Phase 3 happened immediately rather than "eventually."

### Phase 3 - done: repointed every stale reference

Commit `f542f452` (plus two follow-up catches in `docs/index.md`, `docs/install.md`, and `python/README.md`) replaced every `github.com/pacharanero/clincalc` and `pacharanero.github.io/clincalc` reference across `Cargo.toml`, `mkdocs.yml`, `cliff.toml`, `docs/*.md`, `README.md`/`examples/README.md`, the Python and GUI sub-crate manifests and third-party-notices, `src/api.rs`'s MCP contact URL, and `AGENTS.md`.

Deliberately left untouched: `CHANGELOG.md` (historical record), `clinical-safety/*.md`'s references to the unrelated `github.com/pacharanero/SAFETY.md` repo, and README.md's `(@pacharanero)` attribution for the "soft interoperability" term (prose, not a repo URL).

Verified at the time: `cargo fmt`/`clippy`/`test` (default + `mcp` features) clean, `cargo publish --dry-run` succeeded against the new repository metadata, GUI `npm run build` clean, REUSE lint clean, zensical docs build clean, and CI green on `ohtf/clincalc` (including an incidental same-morning fix for a GitHub runner-image regression in the Python 3.9 CI job, since superseded by dropping 3.9/3.10 support entirely).

### Phase 4 - still outstanding (manual, web UI, cannot be scripted)

1. [ ] **crates.io Trusted Publishing**: crates.io crate settings -> Trusted Publishing -> update the configured GitHub repository owner from `pacharanero` to `ohtf` (repo name and workflow path stay `clincalc` / `publish-crates.yml`). Do this before the next tagged release - `publish-crates.yml` will otherwise fail the OIDC handshake (fail-closed, not a silent wrong-identity publish).
2. [ ] **PyPI Trusted Publishing**: PyPI project settings -> Publishing -> same update, owner `pacharanero` -> `ohtf`.
3. [ ] **`HOMEBREW_TAP_TOKEN`**: replace with a PAT scoped to `ohtf/homebrew-tap`.
4. [ ] **GitHub Pages**: confirm the Pages deployment targets the right branch/path on `ohtf/clincalc` (should be automatic - verify `docs.yml` has actually run and deployed, not just built).
5. [ ] Verify a real dry-run: tag a patch release (or use the existing `s/check-release-version` / `s/check-github-release` scripts against a test tag) and confirm the full auto-tag -> cargo-dist -> Homebrew -> crates.io -> PyPI chain completes against the new repo path before relying on it for a real release.

### Phase 5 - not yet started: update the OHTF site

The OHTF website's project entry for `clincalc` already exists (candidate stage). Open a PR against `ohtf/openhealthtechnology.org-website` moving the `clincalc` entry from "Candidate projects" to "Foundation Projects" (Incubating), once Phase 1 and Phase 4 are complete.

## What this plan deliberately does not do yet

- Does not change the AGPL-3.0-or-later licence or the SPDX copyright headers.
- Does not pursue DCO-enforcement CI or Active-stage status - that is a follow-on, not a blocker for the transfer itself.
- Does not touch `pacharanero/homebrew-tap`, which stays exactly as-is for `sct`/`dsc-rs`/`gitehr`.
- Does not attempt the GitHub repository transfer itself, which only the current owner can initiate from GitHub's UI.
