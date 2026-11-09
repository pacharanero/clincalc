# Security Policy

## Reporting a vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

If you believe you have found a security vulnerability in `clincalc`, report it privately using one of the following:

- GitHub's private vulnerability reporting: go to this repo's **Security** tab and click **Report a vulnerability**.
- Email: marcus@bawmedical.co.uk

Please include:

- A description of the issue and its impact
- Steps to reproduce, or a proof-of-concept if possible
- The affected version(s) or commit hash
- Any suggested mitigation

You can expect an initial acknowledgement within **3 working days** and a more substantive response within **10 working days**.

## Scope

This policy covers the code in this repository: the `clincalc` Rust engine and CLI, the Python bindings, the MCP server, and the desktop GUI. Vulnerabilities in upstream dependencies should be reported to the upstream maintainers; if they affect `clincalc`, open a `cargo audit`-style report here once upstream has released a patch, and we will track the bump (see `spec/roadmap.md` ENG-008).

## Clinical-safety note

`clincalc` computes clinical scores. A vulnerability that could cause a wrong result to be returned silently, or that could let an attacker alter scoring logic, input validation, or licence/schema metadata, has clinical-safety implications as well as security ones - flag this explicitly in your report. See `clinical-safety/SAFETY.md` for how `clincalc` documents its clinical safety position; a safety-relevant security report will be cross-linked from the hazard log once triaged.

## Disclosure

We follow a coordinated disclosure process. Once a fix is available, we will:

1. Publish a security advisory via GitHub.
2. Credit the reporter (unless they prefer to remain anonymous).
3. Update `CHANGELOG.md` under a `Security` heading.

## Note for Open Health Technology Foundation (OHTF) context

`clincalc` is a Candidate project for adoption by the Open Health Technology Foundation (<https://openhealthtechnology.org>). Once onboarding completes and the repository moves to `github.com/ohtf`, OHTF's foundation-wide security policy (<https://openhealthtechnology.org/community/security/>) also applies; this file continues to take precedence for anything specific to `clincalc`, per OHTF's own stated policy precedence. See `spec/ohtf-transfer.md` for the transfer plan.
