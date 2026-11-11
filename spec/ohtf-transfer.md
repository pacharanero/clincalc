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
- [x] **Homebrew push credential fixed properly**: replaced the `HOMEBREW_TAP_TOKEN` PAT (scoped to the wrong repo anyway) with `HOMEBREW_TAP_DEPLOY_KEY`, an SSH deploy key added directly to `ohtf/homebrew-tap` with write access - not a personal-account credential at all, repo-scoped rather than account-scoped. `release.yml`'s checkout step now uses `ssh-key:` instead of `token:`. The old `HOMEBREW_TAP_TOKEN` secret was deleted from `ohtf/clincalc` as dead weight.
- [x] **PyPI ownership transferred**: Marcus transferred the `clincalc` PyPI project to OHTF directly (not just a secondary-maintainer invite - full ownership). Confirmed by the user; PyPI's public API does not expose ownership/collaborator data, so this could not be independently verified via API.
- [x] **crates.io and PyPI Trusted Publishing repointed** to `ohtf/clincalc` (2026-11-10, via each registry's web UI).
- [x] **Publishing environments locked down**: `crates-io` and `pypi` restricted to `v*` tags only; `homebrew` and `github-release` restricted to `main` branch + `v*` tags (matching their actual dispatch paths - `auto-tag.yml` always dispatches at the tag ref, manual recovery runs from `main` with the tag as an input); `github-pages` already restricted to `main`. All five had been checked against `s/README.md`'s documented intent - `crates-io`/`pypi`/`homebrew`/`github-release` had drifted to `deployment_branch_policy: null` (fully unrestricted) at some point and were not actually enforcing this.
- [x] **GitHub Pages confirmed working** on the new location: `https://ohtf.github.io/clincalc/` returns 200; `docs.yml` deploys via the artifact-based method (no `gh-pages` branch).
- [x] **README.md and AGENTS.md updated** to state `clincalc` is managed by OHTF, link to this plan, and record the open-source-commitment confirmation and the major-announcements-go-to-the-website convention.
- [~] **Phase 4.5 (the dry-run) in progress**: `v0.5.0` cut via `s/version++ minor` (2026-11-10). Tag created, binaries built for 4 of 5 targets, but `build-local-artifacts (aarch64-apple-darwin)` is being cancelled instantly (zero build time) on every attempt (3 so far), even after an unrelated GitHub-wide incident ("Git Operations and Pull Requests", 16:10-~21:00 UTC) fully resolved. This looks like an `ohtf`-org-level entitlement/billing issue specific to the `macos-14` runner label (Apple Silicon), not a code or workflow problem - `x86_64-apple-darwin` (`macos-15-intel`) and all three other targets built successfully in the same run. See "Blocked" below.
- [ ] Phase 5: update the OHTF site's project entry from Candidate to Incubating.

## What is already in good shape

- Licence is AGPL-3.0-or-later, OSI-approved: satisfies OHTF's "open source, or willing to be" test and the Candidate -> Incubating open-source-commitment agreement without any licence change.
- Clinical safety position is already documented (`clinical-safety/SAFETY.md`, `SAFETY-CASE.md`, `HAZARD-LOG.md`), satisfying "honest about safety."
- `publish-crates.yml` and `publish-python.yml` already use crates.io and PyPI **Trusted Publishing** (GitHub OIDC) rather than long-lived tokens - now repointed to `ohtf/clincalc` (Phase 4).
- Marcus is already a named maintainer with a security-relevant history of responsible dependency/CVE handling (see `spec/roadmap.md` ENG-008).

## Decisions (resolved 2026-11-10)

1. **Repository name**: keep `clincalc`. No action needed until Phase 2.
2. **Homebrew tap**: OHTF gets its own tap, not a shared repurposing of `pacharanero/homebrew-tap` (which stays as-is for `sct`/`dsc-rs`/`gitehr`, none of which are OHTF projects yet). Done: `ohtf/homebrew-tap` created, seeded with an empty `Formula/` directory and a README; `Cargo.toml`'s `tap` and `release.yml`'s `publish-homebrew-formula` job (checkout target and environment URL) repointed to it.
3. **GitHub Pages domain**: default `ohtf.github.io/clincalc/` for now, no custom domain. No action needed until Phase 2/3 - this is what happens automatically on transfer.
4. **crates.io co-ownership**: add `github:ohtf:clincalc` as a second owner. Done (2026-11-10) - added via the crates.io web UI after the scoped-token route (`cargo owner --add`) hit a 403 on a publish-only token.
5. **PyPI ownership**: OHTF already has a shared PyPI account (it publishes `nhs-number`). Rather than just adding it as a second maintainer, Marcus transferred full project ownership on `clincalc` to it. Done.
6. **Tauri bundle identifier**: changed to `io.github.ohtf.clincalc`. Done - safe, since no signed GUI build has ever shipped.

## Blocked - needs your action

1. ~~`macos-14` runner not available to `ohtf`~~ - **root cause found and fixed, 2026-11-11.** Not a billing/entitlement issue: GitHub fully retired the `macos-14` label on 2026-11-02 (announced 2026-10-01), and cargo-dist 0.31.0's own default runner mapping for `aarch64-apple-darwin` still points at it. Fixed by adding `[workspace.metadata.dist.github-custom-runners]` with `aarch64-apple-darwin = "macos-15"` to `Cargo.toml` - confirmed with the exact pinned `cargo-dist` 0.31.0 locally (`dist plan --output-format=json` now resolves that target to `macos-15`). No `release.yml` regeneration needed: the runner matrix is computed dynamically by the `plan` job at CI runtime from this config, not baked into the generated workflow file, and `allow-dirty = ["ci"]` already tells dist not to flag the file as stale against config changes. Retrying the `v0.5.0` release should now get past the step that was failing.

## Execution order

### Phase 1 - done

1. [x] `SECURITY.md` added at repo root.
2. [x] `github:ohtf:clincalc` added as a crates.io co-owner (via the web UI, not `cargo owner --add`).
3. [x] `gui/src-tauri/tauri.conf.json`'s `identifier` updated to `io.github.ohtf.clincalc`.
4. [x] `ohtf/homebrew-tap` created; `Cargo.toml` and `release.yml` repointed to it, pushing via `HOMEBREW_TAP_DEPLOY_KEY` (an SSH deploy key on the tap repo, not a personal PAT).
5. [x] Agree OHTF's [open-source commitment](https://openhealthtechnology.org/governance/open-source-commitment/) applies to `clincalc` - the last Candidate -> Incubating gate. Confirmed 2026-11-10 by Marcus, as owner of both `clincalc` and OHTF, with one stated understanding: `clincalc` will remain mindful of any upstream IP licence it must comply with, and if a proposed calculator's primary-source licensing would conflict with the open-source commitment, it is simply omitted from the registry rather than ever compromising that commitment. This matches `clincalc`'s existing practice for proprietary/rights-review calculators (see `docs/calculators.md`'s "Unavailable on principle" section) exactly - no change of practice, just confirmation it continues under OHTF. Recorded in `AGENTS.md`'s new Governance section.

### Phase 2 - done: the transfer itself

Completed 2026-11-10 by Marcus via GitHub's "Transfer ownership" UI. `pacharanero/clincalc` now redirects to `ohtf/clincalc`; the local `origin` remote was updated to match. GitHub's redirect is not permanent, which is why Phase 3 happened immediately rather than "eventually."

### Phase 3 - done: repointed every stale reference

Commit `f542f452` (plus two follow-up catches in `docs/index.md`, `docs/install.md`, and `python/README.md`) replaced every `github.com/pacharanero/clincalc` and `pacharanero.github.io/clincalc` reference across `Cargo.toml`, `mkdocs.yml`, `cliff.toml`, `docs/*.md`, `README.md`/`examples/README.md`, the Python and GUI sub-crate manifests and third-party-notices, `src/api.rs`'s MCP contact URL, and `AGENTS.md`.

Deliberately left untouched: `CHANGELOG.md` (historical record), `clinical-safety/*.md`'s references to the unrelated `github.com/pacharanero/SAFETY.md` repo, and README.md's `(@pacharanero)` attribution for the "soft interoperability" term (prose, not a repo URL).

Verified at the time: `cargo fmt`/`clippy`/`test` (default + `mcp` features) clean, `cargo publish --dry-run` succeeded against the new repository metadata, GUI `npm run build` clean, REUSE lint clean, zensical docs build clean, and CI green on `ohtf/clincalc` (including an incidental same-morning fix for a GitHub runner-image regression in the Python 3.9 CI job, since superseded by dropping 3.9/3.10 support entirely).

### Phase 4 - done except the live-fire test

1. [x] **crates.io Trusted Publishing**: repointed to owner `ohtf`, workflow `publish-crates.yml`, environment `crates-io`.
2. [x] **PyPI Trusted Publishing**: repointed to owner `ohtf`, workflow `publish-python.yml`, environment `pypi`.
3. [x] **Homebrew push credential**: done via deploy key (see Phase 1.4), not a PAT.
4. [x] **GitHub Pages**: confirmed live and deploying correctly.
5. [ ] **Still to do**: a real dry-run. Nothing above has actually been exercised end to end yet - the Trusted Publisher configs, the deploy key, and the environment policies are all new as of today and have not been through a real `auto-tag -> cargo-dist -> Homebrew -> crates.io -> PyPI` cycle. The lowest-risk way to prove it: the next genuine `s/version++` release *is* the dry-run, since every step fails closed (wrong OIDC identity, wrong deploy key scope, wrong environment) rather than silently publishing somewhere wrong. Whether to force a throwaway patch release now purely to test the pipeline, versus trusting the fail-closed design and verifying at the next real release, is a judgement call for Marcus.

### Phase 5 - not yet started: update the OHTF site

The OHTF website's project entry for `clincalc` already exists (candidate stage). Open a PR against `ohtf/openhealthtechnology.org-website` moving the `clincalc` entry from "Candidate projects" to "Foundation Projects" (Incubating), once Phase 1 and Phase 4 are complete.

## What this plan deliberately does not do yet

- Does not change the AGPL-3.0-or-later licence or the SPDX copyright headers.
- Does not pursue DCO-enforcement CI or Active-stage status - that is a follow-on, not a blocker for the transfer itself.
- Does not touch `pacharanero/homebrew-tap`, which stays exactly as-is for `sct`/`dsc-rs`/`gitehr`.
- Does not attempt the GitHub repository transfer itself, which only the current owner can initiate from GitHub's UI.
