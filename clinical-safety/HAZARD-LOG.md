---
hazards:
  - id: H001
    description: "Incorrect scoring logic - a calculator's compute() diverges from its cited primary-source algorithm and returns a wrong score"
    cause: "Transcription error implementing the published algorithm; mis-weighted predicate; incorrect banding boundary; algorithm reverse-engineered from a third-party implementation rather than the primary publication"
    effect: "A clinician acts on a numerically wrong score, potentially over- or under-estimating clinical risk (e.g. an incorrect stroke, bleeding, sepsis, or triage banding)"
    severity: 2
    likelihood: 3
    risk: high
    controls:
      - C001
      - C002
      - C003
    residual-severity: 2
    residual-likelihood: 5
    residual-risk: low
    status: open
    cso-reviewed: false
    date-raised: "2026-07-03"
    date-closed:
  - id: H002
    description: "Input unit or scale mismatch - a value is supplied in a different unit from the one the calculator expects and is scored on the raw number"
    cause: "Ambiguous units at the library boundary (e.g. creatinine µmol/L vs mg/dL, glucose mmol/L vs mg/dL, weight kg vs lb, age years vs months); caller passes a value in local units; schema unit not read"
    effect: "A physiologically valid but unit-wrong value produces a confident, wrong score with no error raised"
    severity: 2
    likelihood: 3
    risk: high
    controls:
      - C004
      - C005
      - C006
    residual-severity: 2
    residual-likelihood: 4
    residual-risk: medium
    status: open
    cso-reviewed: false
    date-raised: "2026-07-03"
    date-closed:
  - id: H003
    description: "Out-of-range, physiologically implausible, or out-of-population input accepted and scored"
    cause: "compute() does not range-check a numeric input or enforce a required administration context; an implausible value (negative age, systolic BP of 900, impossible lab value) or an ineligible patient/recall period is treated as valid"
    effect: "A nonsensical or out-of-population input yields a plausible-looking score instead of an explicit error, masking a data-entry or administration mistake upstream"
    severity: 3
    likelihood: 3
    risk: medium
    controls:
      - C005
      - C007
      - C008
    residual-severity: 3
    residual-likelihood: 5
    residual-risk: low
    status: open
    cso-reviewed: false
    date-raised: "2026-07-03"
    date-closed:
  - id: H004
    description: "Optional predicate silently defaulted - an omitted clinician-asserted input is treated as false/zero and changes the score"
    cause: "An optional boolean/enum predicate defaults on deserialization when the caller intended 'unknown', not 'absent'; the difference between 'not asserted' and 'asserted false' is collapsed"
    effect: "The score is computed as if a risk factor were confirmed absent when it was merely not entered, biasing the result low"
    severity: 2
    likelihood: 3
    risk: high
    controls:
      - C009
      - C010
    residual-severity: 2
    residual-likelihood: 4
    residual-risk: medium
    status: open
    cso-reviewed: false
    date-raised: "2026-07-03"
    date-closed:
  - id: H005
    description: "Wrong calculator selected - identity confusion between similarly-named or similarly-scoped scores"
    cause: "Two clinically distinct scores share a name stem or abbreviation (e.g. Wells DVT vs Wells PE, CURB-65 vs CRB-65, CHADS2 vs CHA2DS2-VASc); caller invokes the wrong machine name or picks the wrong entry in a list"
    effect: "A valid score for the wrong instrument is returned and mistaken for the intended one"
    severity: 2
    likelihood: 4
    risk: medium
    controls:
      - C011
      - C012
    residual-severity: 2
    residual-likelihood: 5
    residual-risk: low
    status: open
    cso-reviewed: false
    date-raised: "2026-07-03"
    date-closed:
  - id: H006
    description: "Naked result - a score is copied via the copy-paste / clipboard feature without its interpretation, provenance, or the inputs that produced it"
    cause: "The clipboard summary (to_summary_text) is pasted into a free-text field; interpretation and reference travel with it but the input values do not; a downstream reader sees a number without the clinical context or the data behind it"
    effect: "A result is trusted out of context - the reader cannot see which inputs produced it, whether it is current, or that it is a decision aid rather than a decision"
    severity: 3
    likelihood: 3
    risk: medium
    controls:
      - C013
      - C014
      - C015
    residual-severity: 3
    residual-likelihood: 4
    residual-risk: medium
    status: open
    cso-reviewed: false
    date-raised: "2026-07-03"
    date-closed:
  - id: H007
    description: "Guideline drift - the published guideline underlying a calculator is updated, retracted, or superseded and the engine still computes the old algorithm"
    cause: "A clinical body revises the score or its thresholds; the calculator is not updated; no review cadence links the code to the evolving source"
    effect: "The engine returns a score that no longer reflects current clinical guidance"
    severity: 2
    likelihood: 4
    risk: medium
    controls:
      - C016
      - C017
    residual-severity: 2
    residual-likelihood: 5
    residual-risk: low
    status: open
    cso-reviewed: false
    date-raised: "2026-07-03"
    date-closed:
  - id: H008
    description: "Rounding or boundary error at a clinical decision threshold"
    cause: "Floating-point rounding or an off-by-one at a banding cutoff flips the result across a decision boundary (e.g. a score that should be low-risk lands one band higher, or vice versa)"
    effect: "A patient is placed in the wrong risk band precisely at the threshold that changes clinical management"
    severity: 2
    likelihood: 4
    risk: medium
    controls:
      - C018
      - C019
    residual-severity: 2
    residual-likelihood: 5
    residual-risk: low
    status: open
    cso-reviewed: false
    date-raised: "2026-07-03"
    date-closed:
  - id: H009
    description: "Model estimate mistaken for a direct measurement despite poor individual agreement"
    cause: "A regression-derived anthropometric estimate is displayed as a percentage without prominent evidence limitations; cohort-level correlation is mistaken for individual accuracy"
    effect: "A clinician or patient treats a biased estimate as measured body composition and uses it to classify health status or guide treatment"
    severity: 3
    likelihood: 3
    risk: medium
    controls:
      - C013
      - C015
      - C020
    residual-severity: 3
    residual-likelihood: 4
    residual-risk: medium
    status: open
    cso-reviewed: false
    date-raised: "2026-09-01"
    date-closed:

controls:
  - id: C001
    description: "Scoring is verified against the primary source and implemented from the cited publication, never reverse-engineered from a competitor's implementation (AGENTS.md mandate). Adding a calculator requires the algorithm to be traced to its publication."
  - id: C002
    description: "Every calculator carries literature-vector unit tests: published worked examples run through compute() and asserted against the source's own stated result. CI (cargo test) must be green before merge."
  - id: C003
    description: "license() is a required trait method returning a licence with an http(s) evidence URL; a registry test rejects any calculator that omits it. The provenance of every shipped algorithm is always on record and re-checkable from the cited source."
  - id: C004
    description: "input_schema() (JSON Schema) declares each input's type, units, and permitted values, exposed via `clincalc calc <name> --schema` and to any MCP/GUI host, so the expected unit is machine-discoverable rather than assumed."
  - id: C005
    description: "Strongly-typed Input structs (serde::Deserialize with deny_unknown_fields) reject wrong-shape and unknown input at the boundary; malformed, misspelled, or wrong-type input returns CalcError::InvalidInput rather than being silently ignored or coerced. A registry test enforces this for every closed schema."
  - id: C006
    description: "Per-calculator documentation states the expected unit for every numeric input. The governed input-definition system (spec/calculator-input-definitions.md) is the planned single source of truth for what each input means and the unit it carries."
  - id: C007
    description: "Range, eligibility, and administration-context validation at typed deserialization and compute() boundaries returns CalcError::InvalidInput for physiologically implausible or out-of-population inputs rather than scoring them. Each calculator's valid domain is derived from its primary source; for example, ASRS requires confirmation of adult age and its six-month recall period, CURB-65 rejects ages below 18, PERC requires confirmation of clinician gestalt below 15%, and NYHA requires attestation of defined or presumed cardiac disease."
  - id: C008
    description: "Enumerated and boolean predicates constrain the input domain by construction; free numeric inputs are bounded by schema minimum/maximum so out-of-range values are rejectable at the schema layer."
  - id: C009
    description: "The governed input-definition system centralises the meaning of each clinician-asserted predicate, so 'not asserted' and 'asserted false' are defined explicitly and not collapsed by accident."
  - id: C010
    description: "Required inputs are marked required in the schema and in the typed Input; a missing required input fails deserialization (CalcError::InvalidInput) rather than defaulting to a scored value."
  - id: C011
    description: "Every calculator has a unique, stable machine name(), a human title(), and a one-line description(); the central tag taxonomy (src/tags.rs) and `clincalc list --tag` support unambiguous selection."
  - id: C012
    description: "The docs catalogue (docs/calculators.md) and per-calculator reference disambiguate similarly-named scores, each stating its distinct indication and primary source."
  - id: C013
    description: "CalculationResponse returns a human-readable interpretation alongside every numeric result, plus a working map giving the step-by-step breakdown - the number never travels alone within the engine's own output."
  - id: C014
    description: "reference (the primary citation) is carried in every CalculationResponse and included in the clipboard summary (to_summary_text), so a pasted result names the guideline it came from."
  - id: C015
    description: "Documentation (README, docs/how-it-works.md) states that calc outputs are decision aids, not autonomous clinical decisions; the responsible clinician remains accountable for interpretation and action."
  - id: C016
    description: "Each calculator's reference() cites its primary source/guideline and license() evidences it with a URL, so the exact algorithm version in force is identifiable and re-checkable against the current publication."
  - id: C017
    description: "CHANGELOG.md, SemVer, and a single-sourced workspace version track changes; a review cadence (owned by the CSO) periodically re-checks each calculator against its current published guideline."
  - id: C018
    description: "Literature-vector tests include boundary cases that exercise each banding cutoff, so a threshold flip is caught by a failing test."
  - id: C019
    description: "Integer arithmetic is used for integer scores; explicit banding/threshold tests assert the correct band on both sides of each cutoff."
  - id: C020
    description: "Regression-derived estimates with poor individual agreement are labelled as estimates rather than measurements, retain later validation evidence in every interpretation, exclude unsupported diagnostic cut-points, and constrain inputs to a source-observed validation envelope."
---

# Hazard Log - clincalc

> **Template Origin**: Community | **ArcKit Version**: arckit-uk-nhs 5.0.3 | **Command**: `/arckit:uk-nhs-dcb0129` | **Filename**: `HAZARD-LOG.md` (DCB0129 manufacturer)

## Document Control

| Field | Value |
|---|---|
| **Document ID** | `HAZARD-LOG.md` (Marcus Baw SAFETY.md spec convention; no ARC- prefix) |
| **Document Type** | Hazard Log (DCB0129 manufacturer) |
| **Project** | clincalc - open library of clinical calculators |
| **Classification** | PUBLIC (open-source project) |
| **Status** | DRAFT |
| **Version** | 0.1.2 |
| **Created Date** | 2026-07-03 |
| **Last Modified** | 2026-09-01 |
| **Review Cycle** | Monthly (hazard logs are *living* documents) |
| **Next Review Date** | 2026-09-30 |
| **Owner** | Marcus Baw, Maintainer / Product Owner (Baw Medical Ltd) |
| **Reviewed By** | [PENDING - CSO] |
| **Approved By** | [PENDING - CSO] |
| **Distribution** | Public (repository) |

## Risk scoring scales (DCB0129 convention)

- **Severity**: `1` Catastrophic | `2` Major | `3` Considerable | `4` Significant | `5` Minor
- **Likelihood**: `1` Very High | `2` High | `3` Medium | `4` Low | `5` Very Low
- **Risk level**: `unacceptable` | `high` | `medium` | `low`
- **Status**: `open` | `mitigated` | `accepted` | `closed`

> ⚠️ **Numbering direction.** These severity/likelihood numbers follow the *SAFETY.md spec's* ordinal labels (`severity:1` = Catastrophic … `severity:5` = Minor; `likelihood:1` = Very High … `likelihood:5` = Very Low) - so here **1 = most severe / most likely, 5 = least**. That is the reverse of a typical 5×5 risk register (including ArcKit's own Orange Book-based `/arckit:risk`, where higher = worse), so take care when cross-referencing. Note that DCB0129 *itself* does **not** number these axes - the standard uses the word categories above and reserves 1-5 for the resulting **risk rating**, where 5 = unacceptable. Do not read a severity/likelihood number here as a DCB0129 risk-rating number.

---

## Hazards

*Rendered from the structured YAML at the top of this file. Keep in sync - edit YAML first, then re-render this table.*

| ID | Description | Sev | Like | Risk | Controls | Residual Risk | Status |
|---|---|---|---|---|---|---|---|
| H001 | Incorrect scoring logic vs cited primary source | 2 | 3 | HIGH | C001, C002, C003 | LOW | Open |
| H002 | Input unit / scale mismatch scored on raw number | 2 | 3 | HIGH | C004, C005, C006 | MEDIUM | Open |
| H003 | Out-of-range / implausible / out-of-population input accepted and scored | 3 | 3 | MEDIUM | C005, C007, C008 | LOW | Open |
| H004 | Optional predicate silently defaulted, biasing score | 2 | 3 | HIGH | C009, C010 | MEDIUM | Open |
| H005 | Wrong calculator selected (identity confusion) | 2 | 4 | MEDIUM | C011, C012 | LOW | Open |
| H006 | Naked result copied without interpretation / inputs | 3 | 3 | MEDIUM | C013, C014, C015 | MEDIUM | Open |
| H007 | Guideline drift - engine computes superseded algorithm | 2 | 4 | MEDIUM | C016, C017 | LOW | Open |
| H008 | Rounding / boundary error at a decision threshold | 2 | 4 | MEDIUM | C018, C019 | LOW | Open |
| H009 | Model estimate mistaken for a direct measurement despite poor individual agreement | 3 | 3 | MEDIUM | C013, C015, C020 | MEDIUM | Open |

## Controls

| ID | Description |
|---|---|
| C001 | Scoring verified against primary source; implemented from the cited publication, never reverse-engineered (AGENTS.md mandate) |
| C002 | Literature-vector unit tests per calculator; CI (`cargo test`) green before merge |
| C003 | `license()` mandatory with http(s) evidence URL; registry test rejects calculators lacking it |
| C004 | `input_schema()` declares type, units, permitted values; exposed via `--schema` and to MCP/GUI hosts |
| C005 | Typed `Input` structs reject wrong-shape and unknown input; a registry test enforces closed schemas; `CalcError::InvalidInput` surfaced, no silent ignore/coercion |
| C006 | Per-calculator documentation of expected units; governed input-definition system as planned single source of truth |
| C007 | Range, eligibility, and administration-context validation at typed deserialization and `compute()` boundaries rejects implausible or out-of-population inputs, including explicit assessment-context attestations for PERC and NYHA |
| C008 | Enum/boolean predicates constrain domain; numeric inputs bounded by schema `minimum`/`maximum` |
| C009 | Governed input-definition system defines each clinician-asserted predicate; 'not asserted' ≠ 'asserted false' |
| C010 | Required schema fields; missing required input fails deserialization rather than defaulting |
| C011 | Unique stable `name()`/`title()`/`description()`; central tag taxonomy (`tags.rs`); `clincalc list --tag` |
| C012 | Docs catalogue (`docs/calculators.md`) + per-calculator reference disambiguate similar scores |
| C013 | `CalculationResponse.interpretation` + `working` returned with every numeric `result` |
| C014 | `reference` (primary citation) carried in every response and in the clipboard summary (`to_summary_text`) |
| C015 | Documentation states outputs are decision aids, not autonomous decisions; clinician remains responsible |
| C016 | Per-calculator `reference()` cites primary source; `license()` evidence URL identifies the algorithm version |
| C017 | CHANGELOG + SemVer + single-sourced version; CSO-owned review cadence against current guidelines |
| C018 | Boundary-case literature vectors exercise banding cutoffs; a threshold flip fails a test |
| C019 | Integer arithmetic for integer scores; explicit banding tests on both sides of each cutoff |
| C020 | Poor-agreement regression outputs are labelled as estimates, carry later validation limitations, omit unsupported diagnostic cut-points, and accept only a source-observed validation envelope |

---

## Hazards deliberately assessed as not applicable

The `clincalc` engine (with `default-features = false`) is a **strict leaf** - pure, stateless, no I/O, no clocks, no randomness, no global state, no persistence, no network. Several hazards common to NHS digital-health products therefore do **not** arise in the manufacturer scope of this engine and are recorded here so their absence is a documented judgement, not an omission:

- **Wrong-patient / identity matching** - the engine holds no patient identity and performs no record lookup; it computes from anonymous inputs supplied by the caller.
- **Stale clinical data / cache-sync failure** - the engine has no cache, no persistence, and no data of its own; each call is a pure function of its inputs.
- **Authorisation bypass / confidentiality breach** - the engine stores and transmits no patient data and has no access-control surface.
- **Missing audit trail** - `to_summary_text()` is deliberately timestamp-free and the engine keeps no log; audit is the responsibility of the recording host (e.g. GitEHR).
- **Data-integrity loss on write** - the engine performs no writes.

These hazards **re-enter scope at the deployment / host boundary** - i.e. whichever application embeds the engine (GitEHR, the Tauri GUI, an MCP host) and stores or transmits the result. They belong in the **DCB0160 deployer** safety case for that host, not in this manufacturer log. See `SAFETY-CASE.md` §2 (Scope) and the deployment assumptions.

---

## How to extend this hazard log

- Add new hazards to the `hazards:` array in the YAML frontmatter
- Add new controls to the `controls:` array
- Re-render the Markdown tables below the frontmatter
- Have the CSO review each new hazard before flipping `cso-reviewed: true`
- Move hazards through statuses (`open` → `mitigated` → `closed`) as controls are evidenced
- For accepted residual risks above `low`, capture the acceptance rationale in `SAFETY-CASE.md` §5

---

## Important

These eight hazards are a **starter set** adapted to a stateless clinical-calculator engine. They are not a substitute for project-specific hazard identification by a qualified CSO and clinical SMEs, and they should be revisited per calculator - a high-stakes score (e.g. one that drives anticoagulation, sepsis escalation, or triage) may warrant its own hazards beyond the engine-wide set here. **A short hazard log is more often a sign of insufficient analysis than of a safe product.**

---

**Generated by**: ArcKit `/arckit:uk-nhs-dcb0129` command
**Generated on**: 2026-07-03
**ArcKit Version**: arckit-uk-nhs 5.0.3
**Project**: calc
**Model**: Claude Opus 4.8
**Spec lineage**: [Marcus Baw SAFETY.md v2.0.0-draft](https://github.com/pacharanero/SAFETY.md)
