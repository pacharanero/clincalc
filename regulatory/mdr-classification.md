# UK MDR + EU MDR Software-as-Medical-Device Classification - calc

> **Template Origin**: Community | **ArcKit Version**: arckit-uk-nhs 5.0.3 | **Command**: `/arckit:uk-mdr-classification`

## Document Control

| Field | Value |
|---|---|
| **Document ID** | `regulatory/mdr-classification.md` (repo-root placement; this repo is not ArcKit-scaffolded, so no `ARC-NNN-NHSMDR` ID is minted) |
| **Document Type** | SaMD/AIaMD Classification Assessment (UK MDR 2002 + EU MDR 2017/745) |
| **Project** | calc - open library of clinical calculators |
| **Classification** | PUBLIC (open-source project) |
| **Status** | DRAFT |
| **Version** | 0.1.0 |
| **Created Date** | 2026-07-04 |
| **Last Modified** | 2026-07-04 |
| **Review Cycle** | On material MHRA/EU change; and on any change of intended purpose |
| **Next Review Date** | 2026-10-04 |
| **Owner** | Marcus Baw, Maintainer / Product Owner (Baw Medical Ltd) |
| **Reviewed By** | [PENDING - qualified Regulatory Affairs Specialist] |
| **Approved By** | [PENDING - qualified Regulatory Affairs Specialist] |
| **Distribution** | Public (repository) |

## Revision History

| Version | Date | Author | Changes | Approved By | Approval Date |
|---|---|---|---|---|---|
| 0.1.0 | 2026-07-04 | ArcKit AI | Initial creation from `/arckit:uk-mdr-classification` command | PENDING | PENDING |
| 0.1.0 | 2026-07-04 | ArcKit AI | Route A depth (intended purpose, Art 5(5), SOUP); §11 FDA non-device CDS; §12 comparators (MDCalc / FeverPAIN / QRISK); Annex A draft wording | PENDING | PENDING |

---

## Statutory currency

This assessment is pinned to:

- UK MDR 2002 as amended through the Medical Devices (Amendment) (Great Britain) Regulations 2024, plus the Medical Devices (Post-market Surveillance Requirements) Regulations 2024 (in force from 16 June 2025 - verify), and the **draft Medical Devices (Amendment) Regulations 2026** now in consultation (MHRA call for evidence opened 11 May 2026, closing 19 June 2026)
- EU MDR 2017/745 (current text), Annex VIII Rule 11
- MHRA Software and AI as a Medical Device Programme - work packages published as of the assessment date below

**Assessment date**: 2026-07-04

> ⚠️ The GB framework is mid-reform and the CE-recognition position is unsettled: the MHRA ran a targeted consultation (16 Feb - 10 Apr 2026) on **indefinite** recognition of CE-marked devices in GB, not yet enacted. Re-run this assessment when the 2026 Amending Regulations and the CE-recognition outcome are published. Verify all dates against legislation.gov.uk and gov.uk/guidance/regulating-medical-devices-in-the-uk before reliance.

---

## Executive Summary

| Field | Value |
|---|---|
| **Is this product a medical device?** | **Borderline - tips to YES for any clinician-facing distribution** (see §1). The determinant is *intended purpose* and *how each surface is placed on the market*, not the code. |
| **UK MDR 2002 class (if yes)** | Class I under a strict legacy 2002 reading - **but not safe to rely on**; MHRA guidance and the 2026 reform point to **IIa-equivalent**. Plan for IIa. |
| **EU MDR 2017/745 class (if yes)** | **IIa** (Rule 11) for the decision-support calculators; **IIb** for a subset (vital-parameter monitoring / serious-deterioration decisions); per-calculator triage required |
| **Marking pathway** | GB: UKCA (CE currently recognised, transitional). NI: CE or UKNI. EU: CE. Recommended: **UKCA + CE-via-NI for UK-wide; CE for EU** |
| **Conformity-assessment route** | **Approved Body (UK) / Notified Body (EU)** if IIa+ - i.e. not self-declarable in the conservative reading |
| **Quality Management System expectation** | ISO 13485 (if IIa+) |
| **Standards alignment expected** | ISO 14971, IEC 62304 (safety class **C** absent segregation - see §7), IEC 62366-1, BS EN 82304-1, ISO/IEC 27001 (limited) |

`calc` is a suite of recognised clinical scores that returns not just a number but a **clinical interpretation** (e.g. `curb65 = 4 → "High severity ... consider hospital admission and assessment for intensive care"`). Software that creates new medical information used to inform diagnostic or therapeutic decisions is, on the settled MHRA/MDCG reading, **medical device software** - so any surface put in front of clinicians as a usable clinical tool (the `calc` CLI installed by clinicians, the Tauri GUI) is very likely SaMD, landing at **EU MDR Rule 11 Class IIa** for most calculators and **IIb** for the highest-stakes ones. The single most consequential regulatory lever is **positioning**: whether `clincalc` as a library (`default-features = false`) is placed on the market as a *developer component/library* (pushing device obligations onto downstream integrators such as GitEHR) or whether `calc`'s own clinician-facing surfaces are placed as *finished devices* (making Baw Medical Ltd the manufacturer). The recommended next step is a **qualified Regulatory Affairs review of the intended-purpose statement and placing-on-market model**, plus an MHRA borderline pre-submission where the component route is pursued. Note in `calc`'s favour: it is **deterministic and rule-based, not AI/ML**, so the AIaMD-specific regime does not apply.

---

## 1. Scope determination - is this a medical device?

### 1.1 Intended-purpose statement (verbatim from upstream artefacts)

From the project's own descriptions (README, `spec/calculators.md`, `docs/how-it-works.md`, and the DCB0129 `clinical-safety/SAFETY-CASE.md`):

> "calc is an open, standalone library of clinical calculators: a pure Rust scoring engine (`clincalc`) and the surfaces it drives... Given anonymous inputs conforming to a calculator's published schema, it returns the computed score together with a human-readable clinical interpretation and the primary-source reference. It is intended as a **decision aid** for registered clinicians (and for clinical software acting on their behalf), used for each score's intended population and interpreted in clinical context... [it] never makes an autonomous clinical decision."

> "Clinicians need clinical digital tools to provide good care... This project makes them open source, free to use, evidence-based, and auditable." (README)

> Worked example (README): `calc curb65 --input '{...}'` → `curb65 = 4` / *"High severity ... consider hospital admission and assessment for intensive care."*

**Load-bearing observations.** The declared purpose is a *decision aid* returning a **clinical interpretation** (not a bare number), for use by clinicians in relation to individual patients. The words "decision aid", "clinician remains responsible", and "never makes an autonomous clinical decision" are helpful risk-lowering framing but, per MHRA guidance, **disclaimers do not override an intended purpose evidenced by actual functionality and marketing**. The interpretive output ("consider hospital admission") is information used to take a decision with a therapeutic purpose.

### 1.2 UK MDR 2002 regulation 2 definition test

| Test point | Applies? | Notes |
|---|---|---|
| Diagnosis / prevention / monitoring / treatment / alleviation of disease | **Yes (for many calculators)** | Diagnostic scores (Wells, Centor/FeverPAIN), monitoring/severity (NEWS2, CURB-65), prognostic/prediction (QRISK3, CHA2DS2-VASc) |
| Diagnosis / monitoring / treatment / alleviation / compensation for injury or handicap | Partly | e.g. CHALICE (paediatric head injury) informs imaging decisions |
| Investigation / replacement / modification of anatomy or physiological process | No | calc performs no action on the body |
| Control of conception | No | |
| Principal action NOT pharmacological / immunological / metabolic | **Yes** | Purely computational; qualifies as a device by *information*, not by chemical action |

### 1.3 MHRA stand-alone software decision tree

| Decision point | Outcome |
|---|---|
| Does the software perform an action on data different from storage, archival, communication, simple search? | **Yes** - it computes a score and generates an interpretation (new medical information). The MDCG 2019-11 "creation or modification of medical information" limb is met. |
| Is the action performed for the benefit of an individual patient? | **Yes** - the clinician runs a specific patient's data to inform that patient's care. |
| If both Yes → likely medical device | **Both Yes → qualifies as medical device software (for a clinician-facing distribution with a medical intended purpose).** |

### 1.4 Borderline rationale

The case is borderline **not on qualification of the calculators** (they qualify) but on **who is the manufacturer of what is placed on the market**, because `calc` is deliberately polymorphic:

- **Route A - `clincalc` (with `default-features = false`) as a software *component / library* for developers.** Distributed via crates.io / `cargo install` and consumed by downstream products (GitEHR and others). If the placed-on-market intended purpose is *"a software toolkit for developers to embed"* - with no clinical intended purpose claimed and no direct clinician-facing distribution - then `clincalc` may sit as a **component**, and the finished-device obligations fall on the downstream manufacturer who adds a clinical intended purpose. This is a recognised open-source SaMD strategy. It is **not** a loophole: MDR general safety requirements still reach components supplied for incorporation, and the moment the component is handed to clinicians as a usable tool the analysis flips to Route B.
- **Route B - `calc`'s own clinician-facing surfaces (the `calc` CLI installed and run by clinicians; the Tauri GUI).** Here a product *is* placed on the market with a clinical intended purpose, and **Baw Medical Ltd is the manufacturer**. "Placing on the market" does **not** require payment - free/open-source software distributed in the course of an activity can still be a device; intended purpose governs.

> **One-crate caveat (default `cli` feature).** `calc` ships as a **single crate**, `clincalc`, whose CLI is behind a `cli` feature that is **on by default**. So `cargo install clincalc` (and the crate's default build) *is* the clinician-facing `calc` binary - the crate's default published form is the Route B finished tool, not the bare Route A component. This slightly weakens a clean Route A "pure component" posture: a regulator could observe that the default artefact is the runnable clinician CLI. If the component positioning is pursued, consider making `cli` **not** a default feature (so `cargo add clincalc` and the default library surface is the pure `serde`-only engine, and the CLI is an explicit opt-in), and lead the crate's presentation with the library/component intended purpose. This is a Regulatory-Affairs-informed engineering decision.

Closest MHRA Borderline Manual analogues: clinical calculators / clinical scoring tools that provide patient-specific interpretive output are generally treated as devices; the "simple calculator a clinician could do by hand and which provides no interpretation" carve-out does **not** fit calc, because calc supplies interpretation and covers high-stakes scores.

**Recommendation:** obtain an **MHRA borderline pre-submission** decision if Route A (component) is pursued, and a Regulatory Affairs opinion on the intended-purpose wording either way.

### 1.4a The line the law actually draws - and the three mechanisms that support Route A

No medical-device framework (EU MDR, UK MDR 2002, or US FDA) draws the line at "running vs static code". The regulated boundary is set by three concepts, none of which is execution:

- **Intended purpose** - what the manufacturer intends, evidenced by presentation and function.
- **Placing on the market / making available / putting into service** - EU MDR Article 2 defines *making available* as "any supply... in the course of a commercial activity, **whether in return for payment or free of charge**", and *putting into service* as being "made available to the final user as being **ready for use**... for its intended purpose". (Free is not exempt; but raw source that a third party must build and integrate is arguably not a finished device "ready for use".)
- **Manufacturer** - the person who "markets that device **under its name or trademark**".

Three mechanisms give Route A real support when expressed in these terms:

1. **Intended purpose is controllable.** Publishing source code is not, in itself, asserting a medical intended purpose. A repository presented as a *software component / reference implementation / research artefact*, explicitly stating it is *not* intended for the diagnosis/treatment/etc. of individual patients and *not* placed on the market as a finished device, is evidence against manufacturer status (see Annex A for draft wording). The lever only holds if the **whole presentation** (README, docs, marketing) is consistent - a disclaimer cannot override a clinical intended purpose evidenced elsewhere.
2. **Component / SOUP means the integrator is the manufacturer.** Open-source software incorporated as a library becomes "software of unknown provenance" inside the integrator's device; the finished-device obligations attach to whoever integrates it and places the finished product on the market, not to the upstream project.
3. **Health-institution in-house exemption (EU MDR Article 5(5)).** A device manufactured and used *within the same legal entity* (e.g. an NHS trust building calc for its own use), on a non-industrial scale, where no equivalent CE-marked device meets the need, is **exempt from most of MDR**; the (reduced) obligations sit with the deploying institution, not the upstream author (MDCG 2023-1). This is the closest legislated analogue to "responsibility of the runner".

**Where Route A fails:** the moment `calc` itself distributes a *finished, runnable, clinician-facing product* (a downloadable Tauri GUI installer marketed for point-of-care use; `cargo install clincalc` promoted to clinicians as a bedside tool), Baw Medical Ltd has made a device available / put it into service and is the manufacturer, regardless of who presses run. The component and in-house arguments cover the library and third-party deployments, **not** a finished product the project ships. See §12 for how QRISK/ClinRisk navigated exactly this split in practice.

> The fuller argued position - that *published source code is not itself a medical device* because it is not a finished product "ready for use", framed in the statute's own terms with the counter-arguments and answers - is developed in `regulatory/position-published-source-not-a-device.md`. That statement and this conservative assessment are consistent: they describe *different artefacts* (the published library vs. a placed-on-market clinician product).

### 1.5 Determination

> **Determination**: `calc` **IS BORDERLINE**, and for any clinician-facing distribution (Route B) it **IS** medical device software under UK MDR 2002 and EU MDR 2017/745. This assessment proceeds on the conservative assumption that a clinician-facing `calc` distribution **is a device**. If Regulatory Affairs elects and can defend the developer-component positioning (Route A), §§2-9 transfer to the downstream integrator's device and calc closes out under §5 for the component itself.

---

## 2. UK MDR 2002 classification

### 2.1 Classification rules applied

UK MDR 2002 as it currently stands transposes the pre-MDD Annex IX rules and, unlike EU MDR, has **no dedicated standalone-software rule**. Under a strict legacy reading, standalone software that does not administer/exchange energy or drive an active therapeutic device has commonly been placed at **Class I**. However: (a) MHRA guidance already treats clinical decision-support software as carrying higher risk than the legacy letter implies; and (b) the **draft Medical Devices (Amendment) Regulations 2026** introduce more granular, risk-based software classification aligned toward EU MDR Rule 11, expected to reclassify much Class I SaMD upward. Relying on the Class I legacy reading for a decision-support tool is therefore **not prudent**.

### 2.2 Determination

> **UK MDR 2002 Class**: Class I on a strict legacy reading; **treat as IIa-equivalent for planning** pending the 2026 reform. Do not self-certify a decision-support release on the Class I reading without Regulatory Affairs sign-off.

### 2.3 Subclass flags

| Flag | Applicable? | Rationale |
|---|---|---|
| Sterile | N/A for SaMD | No physical product |
| Measuring function | **No** | calc computes clinical scores from clinician-entered values; it does not itself perform a metrological measurement |
| Reusable surgical instrument | N/A for SaMD | |

### 2.4 Self-certification eligibility

| Eligibility | Status | Notes |
|---|---|---|
| Self-certification permitted (Class I, non-sterile, non-measuring) | **Only under the legacy Class I reading - not recommended** | The conservative IIa reading removes self-certification |
| If self-certified, MHRA registration (DORS) required | Yes (if the Class I route is taken) | Registration is required even for Class I |

---

## 3. EU MDR 2017/745 classification (for NI placement and EU market access)

### 3.1 Rule 11 application (software)

Walking Rule 11 for calc's decision-support calculators:

1. **Does the software provide information used to take decisions with diagnostic or therapeutic purposes?** - **Yes.** e.g. CURB-65 informing admission/ICU assessment; CHA2DS2-VASc/HAS-BLED informing anticoagulation; QRISK3 informing statin decisions; CHALICE informing paediatric head-CT decisions. → **at least Class IIa.**
2. **Could those decisions have an impact that may cause death or irreversible deterioration?** - **Case-by-case.** For most calculators, with a clinician in the loop and the tool positioned as an *aid*, the realistic answer is no at the tool level → stays below Class III. A minority could be argued into **Class III** in a worst-case, sole-reliance framing; this needs per-calculator review, but the aid-not-autonomous intended purpose weighs against III.
3. **Could those decisions cause serious deterioration or necessitate surgical intervention?** - **For some, yes** → **Class IIb** (e.g. scores whose error could contribute to a seriously wrong escalation/withholding decision).
4. **Monitoring limb** - software intended to monitor physiological processes is IIa, **except monitoring of vital physiological parameters where the nature of variation could cause immediate danger → IIb.** NEWS2 (aggregate track-and-trigger over vital signs) is a candidate **IIb** on this limb; requires review.

### 3.2 Other rules considered

No other Annex VIII rule dominates: calc does not drive/influence an active therapeutic device (it outputs information to a human), contains no substances, and contacts no body tissue. Rule 11 governs.

### 3.3 Determination

> **EU MDR 2017/745 Class**: **IIa** for the bulk of the decision-support library; **IIb** for a subset (vital-parameter monitoring e.g. NEWS2; scores informing decisions that could cause serious deterioration). **A per-calculator classification triage is required** - the single-engine library spans heterogeneous risk, and the product inherits the **highest** class of the calculators it ships.

### 3.4 Divergence from UK MDR

| | UK MDR 2002 | EU MDR 2017/745 |
|---|---|---|
| Class | I (legacy) → IIa-equivalent (reform/prudent) | IIa (subset IIb) |
| Conformity route | Self-declaration (legacy) → Approved Body (prudent) | Notified Body |
| Marking | UKCA | CE (or UKNI for NI) |

The EU reading is **more conservative** and is the safer planning basis. The GB/EU gap is exactly the "Class I under legacy UK → IIa under EU MDR" migration the reform is designed to close; do not anchor on the legacy Class I position.

---

## 4. Marking pathway

| Pathway | Required? | Conditions |
|---|---|---|
| **UKCA marking** (Great Britain) | Yes, for a GB device release | MHRA DORS registration; UK Approved Body if IIa+ |
| **UKNI marking** (NI placement of GB-manufactured) | Optional route for NI | Notified Body involved; UKNI alone **not** valid in the rest of the EU |
| **CE marking** (NI under Windsor Framework + EU access) | Yes for NI/EU | Notified Body; EUDAMED registration |
| **Recognition of CE marking in GB** (transitional) | **Currently applicable** | EU MDR-compliant devices placeable in GB during transition (currently to 30 June 2030; MDD-route earlier); MHRA consulted Feb-Apr 2026 on **indefinite** CE recognition - outcome pending. **Verify at each release.** |

### 4.1 Recommended routing

For an open, UK-focused clinician tool with likely NI/EU interest: **UKCA for GB + CE (via a Notified Body) for NI and the EU.** Because CE is currently recognised in GB, a CE-marked device can serve GB + NI + EU during the transition - potentially the lower-friction single route until the GB reform settles. Confirm with Regulatory Affairs against the live recognition position.

### 4.2 Registration obligations

| Obligation | Status | Notes |
|---|---|---|
| MHRA Device Online Registration System (DORS) | PENDING | Required for GB placement (all classes, incl. Class I) |
| EUDAMED registration (if CE-marked) | PENDING | |
| UK Responsible Person | N/A | Manufacturer (Baw Medical Ltd) is UK-based |
| EU Authorised Representative (if CE without EU establishment) | PENDING / conditional | Required to place on the EU market from GB |

---

## 5. Conformity-assessment route

| Class | UK route | EU route |
|---|---|---|
| Class I (non-sterile, non-measuring) | Self-declaration | Self-declaration |
| Class IIa / IIb / III | Approved Body | Notified Body |

| Conformity item | Status | Notes |
|---|---|---|
| Technical documentation prepared | PENDING | Annex II / III equivalents; the calc architecture, per-calculator provenance (`license()` + evidence URL), and literature-vector tests are strong raw material |
| Quality Management System (ISO 13485) | PENDING | Required for IIa+; signposted, not generated here |
| Clinical evaluation | PENDING | **Literature route is well-suited** - every calculator reproduces a published, peer-reviewed, validated instrument; the clinical evidence is the instrument's own evidence base, cited per calculator via `reference()`/`license()` |
| Declaration of Conformity | PENDING | |

### Not-a-medical-device closure (only relevant if Route A / component positioning is adopted and defended)

If Regulatory Affairs adopts and can defend Route A, the closure statement would read: *`clincalc` is placed on the market as a software component/library for developers, with no clinical intended purpose claimed and no direct clinician-facing distribution; it is not represented as a medical device in its documentation or marketing; the finished-device obligations attach to downstream manufacturers who incorporate it with a clinical intended purpose.* This closure must be signed off by the responsible person, and the marketing/README wording must be scrubbed of clinician-facing intended-purpose claims to remain consistent with it - a genuine tension with calc's current "clinicians need clinical digital tools" framing, which reads as a clinical intended purpose. **This is the crux for Regulatory Affairs to resolve.**

Two supports and one caution for this closure: (a) third parties who deploy calc *within a single health institution* can rely on the **Article 5(5) in-house exemption** (§1.4a), carrying the reduced obligations themselves; (b) integrators who build calc into a finished product carry the **SOUP / component** obligations for it. **Caution:** the QRISK precedent (§12) shows the MHRA has treated an open, licensed clinical-risk-algorithm *engine* as a registrable **Class I device** in its own right, so even a well-drawn Route A may still land clincalc as a *low-class* device rather than fully outside scope. Do not read "component" as "unregulated"; read it as "the lowest-burden route, with obligations shared with integrators".

---

## 6. MHRA SaMD / AIaMD Programme considerations

`calc` is **deterministic, rule-based software** (`clincalc` is a pure leaf crate; scores are fixed published algorithms; no machine learning, no adaptivity, no training data). The AI-specific regime therefore **does not apply** - a material simplification versus an AIaMD product.

| MHRA AIaMD Programme Work Package | Applicable? | Status / commitment |
|---|---|---|
| WP1 Software | **Yes** | Core SaMD guidance applies |
| WP2 Risk classification | **Yes** | Directly relevant (this assessment) |
| WP3 Innovative devices | N/A | Not an innovative/novel device type |
| WP4 / WP9 Cyber Security | **Yes (limited)** | calc stores/transmits no patient data, but supply-chain/build integrity (crates.io, reproducible builds, dependency pinning) and the host-embedding surface matter |
| WP6 AIaMD | **N/A** | No AI/ML |
| WP11 Best Practice for Manufacturers | **Yes** | Applies to the manufacturing QMS |

---

## 7. Standards alignment

| Standard | Applicable? | Status | Notes |
|---|---|---|---|
| ISO 14971 (risk management) | Yes | PENDING | Cross-reference the DCB0129 hazard log at `clinical-safety/HAZARD-LOG.md`; note ISO 14971 and DCB0129 are complementary but distinct - a device calc needs both, with cross-referenced hazard content |
| IEC 62304 (software lifecycle) | Yes | PENDING | **Software safety class C (conservative)** - see below |
| ISO 13485 (QMS) | If IIa+ | PENDING | Signposted only |
| IEC 62366-1 (usability engineering) | **Yes** | PENDING | The Tauri GUI (and the copy-paste "soft interoperability" headline) is safety-relevant UI - H005 (wrong calculator) and H006 (naked result on paste) are usability hazards |
| ISO/IEC 27001 (information security) | Yes (limited) | PENDING | calc holds no patient data; scope is build/supply-chain security, not data protection |
| BS EN 82304-1 (health software products) | **Recommended** | PENDING | Fits a standalone health-software product well |
| ISO/IEC TR 24028 (AI trustworthiness) | **N/A** | - | No AI/ML |

**IEC 62304 safety class - architectural note.** The class is assigned at the software-system level. Because `clincalc` is a **single engine hosting calculators of heterogeneous stakes** - from low-risk reference scores to high-stakes tools (CHALICE, NEWS2, CHA2DS2-VASc) where a wrong output could contribute to death or serious injury - the undivided engine inherits the **highest** applicable class: **Class C**. To claim a lower class (B) for the bulk, the architecture would need explicit **segregation** of high-stakes calculators (IEC 62304 §4.3/§5.1.1 segregation), which the current leaf-crate design does not provide. This is a concrete design decision for Regulatory Affairs + engineering: accept Class C engine-wide, or segregate.

---

## 8. Post-market obligations

### 8.1 Post-market surveillance (PMS) plan

Outline: monitor (a) issue tracker / user reports of scoring discrepancies, (b) **published-guideline changes** for every implemented score (the calc-specific driver - maps to DCB0129 hazard **H007** guideline drift), (c) downstream host incident feedback. Cadence: continuous triage + a periodic (e.g. annual) guideline re-verification sweep. The UK Post-market Surveillance Requirements Regulations 2024 (in force from 16 June 2025 - verify) apply to GB devices across all classes.

### 8.2 Vigilance reporting

| Event | Reporting timeline | Recipient |
|---|---|---|
| Serious incident (UK) | Within statutory timelines (verify current MHRA timelines) | MHRA |
| Serious incident (EU) | 15 days; 10 for serious public-health threat; 2 for death/serious deterioration | Competent Authority via EUDAMED |
| Field Safety Corrective Action | Without undue delay | MHRA / Competent Authority |

For calc, the most plausible serious-incident vector is a **latent scoring error** shipped to many hosts at once - the FSCA/FSN path (e.g. yank a version, notify downstream crates consumers) should be designed around the fact that a fix propagates only when downstreams update their dependency.

### 8.3 Periodic Safety Update Report (PSUR)

| Class | Cadence |
|---|---|
| Class III | Annual |
| Class IIb | Biennial (EU MDR Art. 86) - verify |
| Class IIa | Biennial / on request - verify |
| Class I | PMS report (EU MDR Art. 85), not a PSUR |

### 8.4 Trend reporting

Track statistically significant increases in non-serious scoring-discrepancy reports; a cluster against a single calculator triggers a hazard-log review and possible FSN.

### 8.5 AIaMD substantial-change handling

**N/A** - no adaptive/ML behaviour; there is no "model drift" or "expected adaptation" line to manage. This is a genuine advantage of the deterministic design.

---

## 9. Substantial change triggers

| Trigger | Action | Notes |
|---|---|---|
| Adding a higher-risk calculator | Reassess classification (engine inherits highest class) | e.g. adding a Class IIb/III-candidate score raises the whole engine unless segregated |
| Change of intended purpose / positioning | Reassess (Route A ↔ Route B) | The most consequential lever - see §1.4/§5 |
| Change of interpretation text that alters clinical meaning | Reassess clinical evaluation + hazard log | Interpretation strings are load-bearing (H006) |
| Change of intended-user or patient population | Reassess usability + clinical evaluation | e.g. clinician-only → patient-facing would materially raise risk |
| Change of a score's algorithm on guideline update | Reassess clinical evaluation; version + FSN as needed | Maps to H007 |
| Change of operating principle (rule-based → ML) | Reassess software safety class **and pull in the AIaMD regime** | Would fundamentally change this assessment |

---

## 10. Open regulatory risks

| Risk | Status | Mitigation |
|---|---|---|
| GB framework mid-reform (draft 2026 Amending Regs; call for evidence to 19 Jun 2026) - likely reclassifies decision-support SaMD upward | Active | Plan for IIa-equivalent now; re-run this assessment on publication |
| CE-recognition-in-GB position unsettled (MHRA indefinite-recognition consultation Feb-Apr 2026, not yet enacted) | Active | Verify the live recognition position at every release; keep CE route open |
| **Route A vs Route B positioning unresolved** | Active | The single biggest open question - needs a Regulatory Affairs decision and consistent README/marketing wording |
| Open-source/free ≠ out-of-scope | Active | Do not treat "free/open" as removing device status; intended purpose governs |
| Per-calculator class heterogeneity in one engine | Active | Run the per-calculator classification triage; decide segregate-vs-accept-Class-C |
| Windsor Framework arrangements may change | Monitor | Check the NI route at each release |

---

## 11. Other-jurisdiction note - US FDA (non-device Clinical Decision Support)

You asked how this looks "anywhere", so: the US has a mechanism the EU/UK lack. The 21st Century Cures Act (section 3060) amended the FD&C Act (section 520(o)(1)(E)) to carve certain Clinical Decision Support **out of the device definition entirely** where it meets four criteria - the decisive ones being that the software (a) does not analyse a medical image or signal, (b) displays/analyses medical information and guidelines, and (c) **enables the clinician to independently review the basis of the recommendation** so they do not rely primarily on it. calc's transparency (open source, pure auditable scoring, cited primary literature, visible working) is well-suited to the "independent review" prong. There is **no equivalent CDS carve-out in EU/UK law**, so a calculator that is *non-device CDS* in the US may still be *Class IIa MDSW* under EU MDR Rule 11. FDA's January 2026 final CDS guidance is the current reference; note that time-critical / high-acuity use can defeat the carve-out.

---

## 12. Regulatory precedents - how comparable calculators handle this

| Product | Model | What it teaches calc |
|---|---|---|
| **MDCalc** (US) | Point-of-care CDS platform; states the service and its output are **"not certified as a medical device"**; heavy disclaimers ("does not offer medical advice"; "trust your clinical judgment"; "not a replacement for experienced clinical judgment"). | The **US non-device-CDS + disclaimer** model (§11) in action - a hugely popular clinician calculator operating *without* a device registration by keeping the clinician in independent control. This travels far less well in the EU/UK, where disclaimers do not remove device status. |
| **FeverPAIN** (Southampton; NICE NG84) | The **score itself is a published clinical prediction rule** (knowledge / guideline), not a device; NICE embeds it; the researchers' FeverPAIN *app* is embedded in GP consultation systems. | Separates the **clinical rule** (not a device, like calc's clinical content under CC-BY-SA) from the **software implementation** supplied for clinical use (which may be). calc's own split of AGPL engine vs CC-BY-SA content mirrors this. (Note: FeverPAIN is University of Southampton, not Oxford.) |
| **QRISK2/3** (ClinRisk / Endeavour Predict) | Core algorithm **open-sourced under LGPL v3**; ClinRisk produces **both open and closed source** implementations; the **EP QRISK3 Engine is registered as a Class I device with the MHRA** and is **licensed to other manufacturers as a component** so they build their own front-ends. MHRA has stated QRISK3 algorithms are medical devices requiring registration. | **The closest live precedent to calc's architecture**: open, auditable algorithm plus an engine licensed to integrators who ship finished products. Two lessons: (1) the component-licensed-to-integrators model is *proven and commercial*; (2) MHRA still treated the productised **engine** as a registrable **Class I device**, even though the *source* is openly published. Expect the same: publish the source freely, but a *runnable calc engine offered for clinical use* may itself be a low-class device. |

**Net:** publishing the open source (as QRISK does) is not what gets registered; the **productised, runnable engine offered for clinical use** is. That distinction - open algorithm vs placed-on-market engine - is the practical shape of your "static vs running" intuition, and QRISK shows it working in the real UK market.

---

## Annex A - Draft "Regulatory status and intended purpose" statement

*Proposed wording for the repository README and the `clincalc` crate-level docs, to make the Route A (component / non-device) positioning defensible. This is draft text for Regulatory Affairs review, not a determination.*

> ### Regulatory status and intended purpose
>
> `clincalc` and the `calc` command-line tool are published as **open-source software components and reference implementations** for developers, researchers, and integrators building health-software systems.
>
> **Intended purpose (as published in this repository):** to provide auditable, literature-referenced implementations of published clinical scores as a software library for incorporation into other systems, and for research, education, and evaluation. `calc` produces the numerical output of a published clinical score together with that score's own published interpretation text; it does not itself make clinical decisions.
>
> **This repository is not placed on the market as a finished medical device.** As published here, `calc` is *not* intended by its author to be used, on its own, for the diagnosis, prevention, monitoring, prediction, prognosis, treatment, or alleviation of disease in an individual patient.
>
> **If you incorporate `calc` into a product, or deploy it for clinical use, you are responsible for the regulatory status of that product or deployment.** Depending on the intended purpose you give it and how you place it on the market or put it into service, it may be medical device software (e.g. Class IIa under EU MDR Rule 11) requiring conformity assessment and UKCA / CE marking, or, within a single health institution, it may rely on the in-house exemption (EU MDR Article 5(5)). Those obligations attach to you as the manufacturer or deploying institution, not to this upstream project.
>
> `calc` is deterministic, rule-based software (no machine learning). Nothing here is regulatory advice.

### README phrases that currently undercut this positioning

To keep the whole presentation consistent with Annex A (a disclaimer cannot override a clinical intended purpose evidenced elsewhere), review these existing phrases - each currently reads as a *clinical* intended purpose aimed at clinicians rather than a *developer component*:

- **"Clinicians need clinical digital tools to provide good care..."** (README, *Why*) - frames the project's purpose as delivering care tools to clinicians.
- **"This project makes them open source, free to use, evidence-based, and auditable"** - "them" = clinical calculators for clinicians.
- **"Soft interoperability... empowers clinicians to use the tools they want"** - clinician-facing product framing.
- The **CURB-65 worked example** emitting *"consider hospital admission and assessment for intensive care"* - interpretive therapeutic output shown as a clinician-facing feature.
- The clinician-facing surfaces themselves (the `calc` CLI promoted to clinicians; the Tauri GUI as a point-of-care tool) - these are the Route B surfaces; if Annex A is adopted they must be **demarcated** (e.g. "for evaluation / research", or handled as separate finished products with their own regulatory status), not presented as ready-to-use clinical tools under the same non-device banner.

**The unavoidable decision:** calc genuinely *is* both a developer library *and* a set of clinician-facing surfaces. Route A can cleanly cover `clincalc` (with `default-features = false`) as a library; it cannot simultaneously cover a Tauri GUI shipped to clinicians as a bedside tool. Decide, per surface, which are non-device components and which are (accepted) devices.

---

## External References

| Doc ID | Title | Source | Used in |
|---|---|---|---|
| UK-MDR-2002 | Medical Devices Regulations 2002 (as amended) | legislation.gov.uk - <https://www.legislation.gov.uk/uksi/2002/618> | Throughout |
| UK-MDR-AMEND-2026 | Draft Medical Devices (Amendment) Regulations 2026 (call for evidence, opened 11 May 2026) | MHRA / gov.uk | §§2, 10 |
| UK-PMS-2024 | Medical Devices (Post-market Surveillance Requirements) Regulations 2024 | legislation.gov.uk | §8 |
| EU-MDR-2017-745 | EU MDR 2017/745, Annex VIII Rule 11 | EUR-Lex - <https://eur-lex.europa.eu/eli/reg/2017/745> | §3 |
| MHRA-SAMD | Guidance: Medical device stand-alone software including apps | MHRA - <https://www.gov.uk/government/publications/medical-devices-software-applications-apps> | §1 |
| MHRA-AIAMD | Software and AI as a Medical Device Programme | MHRA - <https://www.gov.uk/government/publications/software-and-artificial-intelligence-ai-as-a-medical-device> | §6 |
| MHRA-REG-GUIDE | Regulating medical devices in the UK | MHRA - <https://www.gov.uk/guidance/regulating-medical-devices-in-the-uk> | §§4, 10 |
| MHRA-BORDERLINE | MHRA Borderline Manual | MHRA | §1.4 |
| MDCG-2019-11 | Guidance on qualification and classification of software under MDR/IVDR | European Commission MDCG | §1 |
| MDR-ART-5-5 | Article 5(5) health-institution exemption; MDCG 2023-1 guidance | European Commission MDCG | §1.4a, §5 |
| FDA-CDS | Clinical Decision Support Software (final guidance, Jan 2026); FD&C Act s.520(o)(1)(E) | US FDA | §11 |
| QRISK-CLINRISK | QRISK3 open-source algorithm (LGPL v3); EP QRISK3 Engine (Class I, MHRA); ClinRisk / Endeavour Predict | ClinRisk / Endeavour Health | §12 |
| MDCALC | MDCalc disclaimer and terms ("not certified as a medical device") | mdcalc.com | §12 |
| FEVERPAIN-NG84 | FeverPAIN clinical prediction rule; NICE NG84 (Sore throat, acute) | NICE / Univ. Southampton | §12 |
| ISO-14971 | Application of risk management to medical devices | BSI | §7 |
| IEC-62304 | Medical-device software lifecycle | BSI | §7 |
| ISO-13485 | Medical-device QMS | BSI | §7 |
| IEC-62366-1 | Usability engineering for medical devices | BSI | §7 |
| BS-EN-82304-1 | Health software - general requirements for product safety | BSI | §7 |
| DCB0129-HL | calc Hazard Log | This repository - `clinical-safety/HAZARD-LOG.md` | §7, §8 |

---

## Important

This classification assessment is **not** regulatory advice. The output MUST be reviewed and signed off by a qualified Regulatory Affairs Specialist or notified-body / approved-body advisor before being used to make product, procurement, or market-access decisions. Misclassification has material legal, commercial, and patient-safety consequences. The Route A / Route B positioning decision in particular is a legal-strategic judgement that only a qualified adviser should settle.

---

**Generated by**: ArcKit `/arckit:uk-mdr-classification` command
**Generated on**: 2026-07-04
**ArcKit Version**: arckit-uk-nhs 5.0.3
**Project**: calc
**Model**: Claude Opus 4.8
