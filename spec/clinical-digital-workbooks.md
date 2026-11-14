<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Clinical Digital Workbooks (concept)

**Status: speculative, future, not committed.** This document records an idea for a possible future product that would consume `clincalc` as a component, not a `clincalc` feature itself. Nothing here implies a decision to build it. It is written down now because the idea is good enough to lose if it only lives in conversation, and because `clincalc`'s own roadmap (`ENG-003` `clincalc-web`, `ENG-004` FHIR export, the calculator input-definitions system) already brushes up against the same territory from the engine side.

## The idea

A **Clinical Digital Workbook** is an interactive, condition-specific webpage that walks a clinician through best-practice management of a patient's presentation, step by step, combining:

1. **`clincalc` scoring** at each decision point (a calculator call, with its typed input, result, and citation).
2. **Textbook narrative and advice** - the surrounding clinical reasoning, thresholds, and actions a standard reference or guideline would give at that point in the pathway.
3. **A structured, auditable deviation mechanism** - a first-class way for the treating clinician to say "I am doing something other than the textbook default here, and here is why", without the workbook pretending that didn't happen or silently overriding the record.
4. **Export** to HTML/PDF, structured enough to import into an EPR (electronic patient record) as a durable document of what was assessed, calculated, advised, decided, and why.

Think "a Marimo or Jupyter notebook, but the cells are clinical decision points instead of code cells, and the narrative between them is a textbook chapter instead of prose about the code." `clincalc` already solves the scoring-cell problem: typed input, pure computation, a citation, and a paste-ready result. What a workbook adds is everything around that cell - the guidance before it, the decision after it, and the record of what actually happened.

## Why this is hard, and why it is not a `clincalc` feature

`clincalc`'s entire design is a deliberate narrowing: pure functions, no I/O, no clock, no opinions about sequence, no prose beyond a citation and a one-line interpretation. That narrowing is what makes it auditable and embeddable, and it is exactly what a workbook needs to *not* be. A workbook has state (where is this patient in the pathway today), sequence (this step follows that one, conditionally), narrative at a textbook's depth (not a one-line interpretation), and a human decision layer (did the clinician follow the default, and if not, why). None of that belongs in the leaf engine; all of it belongs in a host that calls the leaf engine. This is the same relationship GitEHR already has to `clincalc` - a consumer, not a feature - just a more ambitious consumer.

## A worked example: major burns

This is the example that motivated the idea, and it is worth tracing in detail because it shows both how well the shape fits and where it breaks down.

1. **Primary survey and %TBSA estimation** - Lund and Browder chart (age-adjusted body-surface-area percentages) or the Rule of Nines, to estimate total body surface area burned. **`clincalc` does not yet implement either** - a concrete, near-term gap this idea surfaces, independent of whether a workbook is ever built. (Current clincalc calculators have no chart-based body-surface-burn estimator; this would be a new `CALC-*` candidate in its own right, scoped and reviewed exactly as any other calculator.)
2. **Parkland formula** - first-24-hour crystalloid volume (4 mL x kg x %TBSA, half in the first 8 hours from the time of burn). **Already shipped** (`parkland`, CALC-084) - the workbook step here is a direct `clincalc` call, feeding %TBSA from step 1 straight in.
3. **Escharotomy decision** - this is the step that shows the limit of "it's all calculators". There is a numeric threshold buried in it (compartment pressure above roughly 30 mmHg, or a Doppler-absent distal pulse), but the actual decision is a clinical-judgement synthesis: circumferential full-thickness burn, deteriorating distal perfusion on serial exam, rising compartment pressure, worsening pain or paraesthesia out of proportion to analgesia. A workbook needs to host this as **guided narrative with an embedded decision point**, not force it into a `Calculator`'s typed-input/single-result shape the way `canadian_ct_head` or `ottawa_ankle` can for a cleaner decision rule. Some of this content could still *use* a small decision-rule calculator as one input into the narrative; not all of it reduces to one.
4. **Ongoing resuscitation monitoring** - urine output targets (e.g. roughly 0.5 mL/kg/hr in adults, higher if myoglobinuria or electrical injury is suspected), fluid-rate titration against that target, recognition of over- and under-resuscitation. Mostly narrative and judgement, with small embedded calculations.
5. **Inhalation injury assessment, secondary survey, referral criteria** to a burns unit (e.g. American Burn Association-style criteria by %TBSA, body region, age, mechanism, and comorbidity) - narrative and criteria-checklist territory again, not a single score.

The pattern across this pathway: **some steps are a clean `clincalc` call, some are narrative with an embedded small decision rule, and some are almost entirely clinical judgement with a textbook's worth of caveats.** A workbook has to host all three without flattening the judgement-heavy steps into false precision, and without treating the calculator steps as mere arithmetic when they are the one part of the pathway that can and should be machine-verified against primary sources.

## Core design problems

### 1. Content governance ("tightly standards-controlled")

The narrative content is the harder half of this, and it needs the same discipline `clincalc` already applies to scoring: every piece of advice traceable to a primary source (a guideline, a textbook edition, a trial), versioned, dated, and re-verifiable - the same shape as `CalculatorLicense`'s evidence URL and `last_verified` tracking (`ENG-008`), and the same "statement / includes / excludes / source / status" shape already proposed for calculator **input definitions** (`spec/calculator-input-definitions.md`). A workbook's narrative "cell" is not so different from an input definition's `statement` and `caveats` fields, just longer-form and positioned between decision points rather than attached to one input. Reusing that governed-definition model, rather than inventing a second one, is the most direct inheritance from `clincalc`'s existing design.

Guideline content also goes stale and diverges by jurisdiction in exactly the way `clincalc` already has to handle per-calculator (NICE vs US guidance on ABCD2, CRB-65 vs CURB-65, UK-WHO vs CDC growth references). A workbook multiplies this problem across narrative paragraphs instead of input thresholds, so it needs its own version of `ENG-008`'s reference-currency tracking, scaled up.

### 2. The deviation mechanism

This is the problem that makes or breaks the idea. A workbook that cannot represent "the clinician deviated, with reason" is a rigid checklist that clinicians will route around (and rightly so - patient-specific needs, local resource constraints, drug shortages, comorbidity contraindications, and plain clinical judgement are all legitimate reasons to deviate from a textbook default). A workbook that represents deviation *without structure* is just a free-text box that erodes the standard it was built to protect, and produces an export nobody can audit.

The shape that is likely needed, not yet decided:

- Every decision point has a **default** (the textbook/guideline recommendation, cited).
- The clinician can **accept** the default, or **deviate**, and a deviation requires a **structured reason** (a small, reviewed taxonomy - patient preference, comorbidity/contraindication, local resource or formulary constraint, evolving evidence not yet in the cited guideline, clinical judgement on the individual presentation) plus **free text** for the specific justification.
- The exported record shows **both**: what the guideline/default said, and what was actually done and why. Nothing is silently overridden; nothing is hidden.
- Deviations are not failures to be flagged for audit by default - they are the expected, legitimate texture of real clinical practice, and the record should read that way rather than as a compliance checklist.

This is a clinical-safety-and-governance problem as much as a product one, and should be designed alongside whatever safety-case discipline the project adopts (see the house `clinical-safety.md` standard: hazard logs, DCB0129-style documentation), not bolted on afterward.

### 3. Export and EPR integration

HTML/PDF export for pasting or filing into an EPR is the easy half technically (the "soft interoperability" pattern the `clincalc` desktop GUI already uses for a single result generalises directly). The harder question is whether the export should *also* carry structured data - a FHIR `Bundle` of `Observation`s for each calculator step (building directly on the already-proposed `ENG-004` FHIR export) plus some structured representation of the narrative/deviation record - so a receiving EPR could file it as data, not just as a flattened document. That is a second, independent design problem from the document itself and should not block a first HTML/PDF-only version.

### 4. Scope discipline

The single biggest risk to this idea is scope: "encode all of medicine" is not a project, it is a career. The path that keeps this buildable is almost certainly: **pick one pathway, build it end to end, including the deviation mechanism and the export, before generalising anything into a "workbook engine."** Burns is a strong first candidate precisely because it is example-rich, mostly ICU/emergency practice (a bounded, protocol-driven specialty), has a `clincalc` calculator already shipped (`parkland`) and at least one clear gap to fill (Lund and Browder), and has a genuinely hard judgement step (escharotomy) that will immediately test whether the content model can host judgement, not just arithmetic. Sepsis bundles and DKA management are plausible second candidates for the same reasons once the first pathway proves the shape.

## Relationship to the existing ecosystem

- **`clincalc`** supplies every calculator step, unchanged, through the same `Calculator` trait and registry every other surface uses. A workbook is a new *consumer*, exactly like the CLI, the GUI, or an MCP host - it should call `clincalc` the same way they do, not fork or reimplement scoring.
- **`clincalc`'s calculator input-definitions system** (`spec/calculator-input-definitions.md`) is the most direct design precedent for governed narrative content, and should be the starting point for a workbook's content schema rather than a fresh design.
- **GitEHR** (the sibling EPR project) is the natural first integration target for the export, and plausibly the natural *host* for a workbook product, given it already embeds `clincalc` via the reusable `cli` module pattern documented in `docs/how-it-works.md`.
- **OHTF governance** - if this is ever built, it inherits the same open-source commitment and primary-source discipline `clincalc` already operates under; a workbook's clinical-safety stakes are at least as high as a calculator's, arguably higher, since it is giving management advice rather than a single number.

## What this is not, yet

This document does not propose a repository, an architecture, a tech stack, or a timeline. It does not add anything to `clincalc`'s own roadmap beyond the two concrete, independently-justifiable gaps it happened to surface (a Lund-and-Browder-style %TBSA calculator; FHIR export, already tracked as `ENG-004`). Those two items can be picked up on their own merits regardless of whether a workbook is ever built.

## Open questions for comment

- Where would this live - a new standalone repo, a GitEHR module, or something else entirely? (Given the scope, a new repo seems more likely than a `clincalc` feature or submodule, but this is undecided.)
- Is the right authoring format for workbook content something structured (JSON/YAML with a governed schema, machine-checkable like calculator input definitions) or closer to a documentation format (Markdown with embedded calculator-call directives, more like a notebook)? The Marimo comparison suggests the latter is more approachable to author; the governance requirement pulls toward the former.
- How much of the deviation-reason taxonomy can be shared across conditions/pathways versus needing to be pathway-specific?
- Should a workbook be able to *branch* (different pathways depending on an early finding, e.g. inhalation injury present vs absent) as a first-class structure, or is that better left to narrative judgement ("if X, see the inhalation injury section")?
- Who authors and clinically reviews pathway content at the rigor this needs, and how does that reviewer capacity get built - the same open question `clincalc`'s own translation and multilingual work already has for a smaller problem (native-speaker clinical review bandwidth)?
