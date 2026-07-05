# Clinical Safety Case Report - calc

> **Template Origin**: Community | **ArcKit Version**: arckit-uk-nhs 5.0.3 | **Command**: `/arckit:uk-nhs-dcb0129` | **Filename**: `SAFETY-CASE.md` (DCB0129 manufacturer case)

## Document Control

| Field | Value |
|---|---|
| **Document ID** | `SAFETY-CASE.md` (Marcus Baw SAFETY.md spec convention; no ARC- prefix) |
| **Document Type** | Clinical Safety Case Report (DCB0129 manufacturer) |
| **Project** | calc - open library of clinical calculators |
| **Classification** | PUBLIC (open-source project) |
| **Status** | DRAFT |
| **Version** | 0.1.0 |
| **Created Date** | 2026-07-03 |
| **Last Modified** | 2026-07-03 |
| **Review Cycle** | Quarterly (recommended); on every material product change |
| **Next Review Date** | 2026-10-03 |
| **Owner** | Marcus Baw, Maintainer / Product Owner (Baw Medical Ltd) |
| **Reviewed By** | [PENDING - CSO] |
| **Approved By** | [PENDING - CSO sign-off in §6 below] |
| **Distribution** | Public (repository) |

## Revision History

| Version | Date | Author | Changes | Approved By | Approval Date |
|---|---|---|---|---|---|
| 0.1.0 | 2026-07-03 | ArcKit AI | Initial creation from `/arckit:uk-nhs-dcb0129` command | PENDING | PENDING |

---

## Risk scoring scales (DCB0129 convention)

| Scale | 1 | 2 | 3 | 4 | 5 |
|---|---|---|---|---|---|
| **Severity** | Catastrophic | Major | Considerable | Significant | Minor |
| **Likelihood** | Very High | High | Medium | Low | Very Low |

**Risk levels**: `unacceptable` · `high` · `medium` · `low`
**Hazard status**: `open` · `mitigated` · `accepted` · `closed`

> ⚠️ **Numbering direction.** These severity/likelihood numbers follow the *SAFETY.md spec's* ordinal labels - **1 = most severe / most likely, 5 = least**. That is the reverse of a typical 5×5 risk register (including ArcKit's own Orange Book-based `/arckit:risk`, where higher = worse), so take care when cross-referencing. DCB0129 *itself* does **not** number these axes - it uses the word categories above and reserves 1-5 for the resulting **risk rating**, where 5 = unacceptable. Do not read a severity/likelihood number here as a DCB0129 risk-rating number.

---

## 1. Intended Use

`calc` is an open, standalone library of clinical calculators. Its purpose is to compute recognised clinical scores and calculator outputs (e.g. symptom scores, risk scores, severity indices) **from anonymous inputs supplied by the caller**, returning the numeric result together with a human-readable clinical interpretation and the primary-source reference. It is a single Rust crate (`clincalc`) - a pure scoring engine plus the `calc` CLI (the default `cli` feature) - driving multiple surfaces from one source of truth: the command line, an MCP host surface, and a desktop GUI (Tauri). GitEHR is one downstream consumer; the library is reusable by anyone with no knowledge of GitEHR.

### What this product is

A deterministic, stateless calculation engine. Given a set of inputs conforming to a calculator's published input schema, it returns the same result every time, with an interpretation string and a citation. It performs the arithmetic and banding of a clinical score exactly as specified in that score's primary publication, and makes the algorithm's provenance (licence + evidence URL) part of the shipped contract.

### What this product is not

- It is **not** an electronic health record, and it holds, stores, or transmits **no** patient-identifiable data. It has no patient identity, no record lookup, no persistence, no cache, no audit log, and no network access (the `clincalc` engine, with `default-features = false`, is a strict leaf depending only on `serde`/`serde_json`).
- It is **not** an autonomous clinical decision-maker. Every output is a decision *aid* to be interpreted and acted on by a responsible clinician.
- It does **not** capture the clinical data it scores, decide when a score is indicated, or record who acted on the result - those are responsibilities of the embedding host.
- It does **not** claim to be exhaustive or to replace local clinical guidelines; where a local pathway differs from a calculator's source, the local pathway governs.

### Intended users

Registered clinicians (and clinical software embedding the engine on their behalf) who are competent to select the appropriate score, enter valid inputs, and interpret the result within the patient's clinical context. Calculators are not intended for use by patients as a sole basis for self-management decisions.

### Intended patient population

Population is **calculator-specific** - each score defines its own indicated population in its primary source (e.g. an age band, a presenting condition, a care setting). The engine itself is population-agnostic; the intended population for any given result is that of the calculator invoked.

### Intended clinical context

As a point-of-care or point-of-review decision aid across settings (primary, secondary, community, and self-hosted/offline use), wherever a recognised clinical score is used to support - not replace - a clinician's judgement.

---

## 2. Scope

### In scope for this safety case

- The correctness of each calculator's scoring logic against its cited primary source.
- The safe handling of inputs at the engine boundary (units, ranges, missing/optional predicates, wrong-shape data).
- The safe presentation of results by the engine's own output contract (`CalculationResponse`: result, interpretation, working, reference) and its clipboard summary (`to_summary_text`).
- Unambiguous identification and selection of the correct calculator.
- Traceability of every calculator to a current, cited clinical source (guideline-drift management).

### Out of scope for this safety case

- **Deployment-specific arrangements** - how any particular host (GitEHR, the Tauri GUI, an MCP host, a third-party integration) captures inputs, displays results, stores them in a record, transmits them, or authenticates users. These are covered by that host's **DCB0160 deployer** safety case.
- **Patient identity, record storage, audit, access control, and data transmission** - the engine performs none of these; they arise only at the host boundary (see the "Hazards deliberately assessed as not applicable" section of the Hazard Log).
- **Off-label clinical use** - using a calculator outside the population or indication defined by its primary source.
- **The clinical decision itself** - the engine informs, it does not decide.

### Deployment assumptions

These assumptions are the manufacturer's; each becomes an **obligation on the deploying organisation's DCB0160 case**:

1. The host presents the result **together with** its interpretation and primary-source reference, and does not strip them (the engine supplies all three; the host must not discard them).
2. The host records, where clinically relevant, the **inputs** that produced a result, since the engine's clipboard summary carries the result, interpretation, and reference but **not** the input values (see H006).
3. The host applies the correct **units** to numeric inputs as declared by each calculator's `input_schema()` and does not pass locally-scaled values without conversion (see H002).
4. The host distinguishes **"not asserted" from "asserted false"** for optional predicates and does not silently default unknowns (see H004).
5. The host and its users treat outputs as **decision aids**, retaining clinical accountability for interpretation and action.
6. The host is responsible for **audit, identity, access control, and secure storage/transmission** of any result it retains.

---

## 3. Safety Argument

Top-level claim:

> **G1: calc is acceptably safe for use by registered clinicians (and clinical software acting on their behalf) as a clinical-scoring decision aid, when each calculator is used for its intended population and the result is interpreted in clinical context.**

This claim is supported by the sub-claims below. Each references hazards (H-ID) from the [Hazard Log](./HAZARD-LOG.md) and the controls (C-ID) that mitigate them.

### G1.1 - Every calculator computes its score correctly against a cited primary source

Evidence:

- Scoring is verified against primary sources and implemented from the cited publication, never reverse-engineered - a non-negotiable rule in `AGENTS.md` (**C001**, addressing **H001**).
- Every calculator ships literature-vector unit tests: published worked examples asserted against the source's own stated result; `cargo test` must be green in CI before merge (**C002**, **H001**).
- Integer scores use integer arithmetic and carry explicit banding tests on both sides of each cutoff, so rounding/boundary flips fail a test (**C018, C019**, addressing **H008**).

### G1.2 - Every shipped algorithm has recorded, re-checkable provenance

Evidence:

- `license()` is a required trait method returning a licence with an `http(s)` evidence URL; a **registry test rejects any calculator that omits it** - a calculator with no evidenced provenance does not ship (**C003**, **H001**).
- Each calculator's `reference()` cites its primary source, so the exact algorithm in force is identifiable and re-checkable against the current publication (**C016**, addressing **H007**).

### G1.3 - Inputs are handled safely at the engine boundary

Evidence:

- `input_schema()` (JSON Schema) declares each input's type, units, and permitted values, exposed via `calc <name> --schema` and to MCP/GUI hosts, so expected units are machine-discoverable (**C004**, addressing **H002**).
- Strongly-typed `Input` structs reject wrong-shape input and return `CalcError::InvalidInput` rather than silently coercing; range/plausibility checks reject implausible values instead of scoring them (**C005, C007, C008**, addressing **H002, H003**).
- Required inputs fail deserialization if missing rather than defaulting to a scored value; the governed input-definition system defines each clinician-asserted predicate so "not asserted" is not collapsed into "asserted false" (**C009, C010**, addressing **H004**).

### G1.4 - Results are never presented as a naked number by the engine

Evidence:

- `CalculationResponse` returns a human-readable `interpretation` and a `working` breakdown alongside every numeric `result`; the `reference` (primary citation) travels with the result and is included in the clipboard summary (**C013, C014**, addressing **H006**).
- Documentation states outputs are decision aids, not autonomous decisions, with the clinician remaining accountable (**C015**, **H006**).
- **Known residual gap:** the clipboard summary (`to_summary_text`) carries result, interpretation, and reference but **not** the input values - see §5 and H006.

### G1.5 - The correct calculator is unambiguously identifiable

Evidence:

- Every calculator has a unique stable machine `name()`, a human `title()`, and a `description()`; the central tag taxonomy (`src/tags.rs`) and `calc list --tag` support unambiguous selection (**C011**, addressing **H005**).
- The docs catalogue (`docs/calculators.md`) and per-calculator reference disambiguate similarly-named scores, each stating its distinct indication (**C012**, **H005**).

### G1.6 - Calculators stay aligned with current clinical guidance

Evidence:

- Per-calculator `reference()`/`license()` pin each algorithm to a cited, re-checkable source (**C016**, addressing **H007**).
- `CHANGELOG.md`, SemVer, and a single-sourced workspace version track change; a **CSO-owned review cadence** periodically re-checks each calculator against its current published guideline (**C017**, **H007**). *This cadence must be established by the appointed CSO - it is not yet operating.*

---

## 4. Evidence

### Testing strategy

- **Unit / literature-vector tests** per calculator: published worked examples run through `compute()` and asserted against the source's stated result (C002).
- **Registry tests** enforcing cross-cutting invariants - notably the mandatory, evidence-URL-bearing `license()` (C003).
- **Boundary tests** exercising banding cutoffs (C018/C019).
- CI gates on `cargo fmt --check`, `cargo clippy -D warnings`, and `cargo test` - the project does not merge red (`AGENTS.md`).

### Clinical validation

Clinical validity rests on **faithful implementation of each score's primary publication** rather than on an independent clinical investigation of the engine - appropriate for a calculator that reproduces published, peer-reviewed instruments. The level of evidence for any single result is therefore the level of evidence of the underlying published score. The CSO should, per calculator, confirm that: (a) the cited source is the current authoritative one; (b) the implementation matches it; and (c) the intended population and caveats are correctly represented in the `interpretation`.

### Usability evidence (IEC 62366-1 alignment, if applicable)

Not yet performed. The Tauri desktop GUI (with paste-ready clipboard summary as its headline "soft-interoperability" feature) is the primary human-facing surface and is the natural subject of formative and summative usability evaluation - in particular around H005 (calculator selection) and H006 (result presentation and copy-paste). This is a recommended pre-deployment activity for the CSO to scope.

### Real-world performance monitoring

No post-market surveillance process is yet defined. Recommended: a lightweight issue-triage path for reported scoring discrepancies, and a periodic guideline-drift review (C017) that re-checks each calculator against its current source. Discrepancy reports should feed back into the Hazard Log.

---

## 5. Residual Risk

### Accepted residual risks

*The residual risks below remain above `low` after current controls and require explicit CSO consideration and sign-off before deployment.*

| H-ID | Residual Severity | Residual Likelihood | Residual Risk | Justification for acceptance (CSO to confirm) |
|---|---|---|---|---|
| H002 | 2 | 4 | medium | Units are declared in each `input_schema()`, but the engine cannot force a caller to supply the declared unit - it is a boundary the embedding host controls. Residual risk is mitigated by host obligation (deployment assumption 3) and is proportionate given the benefit of a reusable engine. |
| H004 | 2 | 4 | medium | Required inputs fail closed, but optional predicates can by design default; distinguishing "unknown" from "false" ultimately depends on the host's data capture (deployment assumption 4). |
| H006 | 3 | 4 | medium | The engine never emits a naked number in its own output, but the clipboard summary omits the **input values**, so a pasted result cannot be reconstructed from the text alone. Accepted for now with a host obligation to record inputs where clinically relevant (deployment assumption 2); **recommended improvement**: include the inputs (and a version stamp) in `to_summary_text()`. |

*The remaining hazards (H001, H003, H005, H007, H008) reduce to `low` residual risk after controls and are recorded as such in the Hazard Log.*

### Overall residual risk position

*[PENDING - CSO judgement.]* On the evidence above, the engine's principal residual risks are concentrated at the **host boundary** (units, optional-predicate semantics, and result provenance on copy-paste) rather than in the scoring itself, which is strongly controlled by primary-source verification, literature-vector testing, and mandatory evidenced provenance. Three of these boundary risks are best closed by (a) the recommended `to_summary_text()` provenance improvement and (b) the corresponding DCB0160 deployer obligations. The appointed CSO must make and record the overall judgement that the product, with these controls, is acceptably safe for its intended use - and should give particular attention, per calculator, to any high-stakes score whose worst-case severity exceeds the engine-wide assessment used here.

---

## 6. CSO Sign-off

| Field | Value |
|---|---|
| **CSO Name** | [PENDING] |
| **Registration** | [GMC / NMC / HCPC / GPhC number - PENDING] |
| **Date** | [PENDING] |
| **Statement** | I have reviewed this Clinical Safety Case Report and the associated Hazard Log. I am satisfied that the clinical hazards have been systematically identified and that residual risks are at an acceptable level given the intended clinical benefit of the product. I approve this safety case. **- [PENDING]** |
| **Signature** | [PENDING] |

---

## External References

| Doc ID | Title | Source | Used in |
|---|---|---|---|
| DCB0129 | Clinical Risk Management: its Application in the Manufacture of Health IT Systems | NHS England | This document, throughout |
| DCB0160 | Clinical Risk Management: its Application in the Deployment and Use of Health IT Systems | NHS England | §2 (scope boundary, deployment assumptions) |
| SAFETY-MD-SPEC | SAFETY.md spec v2.0.0-draft | Marcus Baw / pacharanero - <https://github.com/pacharanero/SAFETY.md> | Document structure, hazard-log YAML format |
| ISO-14971 | Application of risk management to medical devices | BSI | §1, §3, §5 (cross-reference if calculators are classified as devices) |
| IEC-62304 | Medical-device software lifecycle processes | BSI | §1, §4 (cross-reference if classified as a device) |
| IEC-62366-1 | Application of usability engineering to medical devices | BSI | §3 G1.4/G1.5, §4 |
| MHRA-SaMD | Software and AI as a Medical Device - MHRA guidance | MHRA | §1, §2 (classification of clinical calculators - run `/arckit:uk-mdr-classification`) |
| calc-AGENTS | calc AGENTS.md - engineering and verification mandates | This repository | §3, §4 (C001, C002 provenance) |

---

**Generated by**: ArcKit `/arckit:uk-nhs-dcb0129` command
**Generated on**: 2026-07-03
**ArcKit Version**: arckit-uk-nhs 5.0.3
**Project**: calc
**Model**: Claude Opus 4.8
**Spec lineage**: [Marcus Baw SAFETY.md v2.0.0-draft](https://github.com/pacharanero/SAFETY.md)
