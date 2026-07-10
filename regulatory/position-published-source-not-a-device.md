# Regulatory position statement - published source code is not a medical device

> **Nature of this document**: an argued *position*, drafted for the author to develop with a specialist medical-device regulatory litigator. It is **not** legal or regulatory advice and it is **not** a determination. It sets out the position the author intends to defend, together with the strongest counter-arguments and the answers to them, so that the position can be pressure-tested before it is relied on or asserted publicly. Where it states the law, verify against the primary sources cited before reliance.

## Document Control

| Field | Value |
|---|---|
| **Document ID** | `regulatory/position-published-source-not-a-device.md` |
| **Document Type** | Regulatory position statement (argument, not determination) |
| **Project** | clincalc - open library of clinical calculators |
| **Position-holder** | Marcus Baw / Baw Medical Ltd |
| **Classification** | PUBLIC |
| **Status** | DRAFT - for specialist legal review |
| **Version** | 0.1.0 |
| **Created / Last Modified** | 2026-07-04 |
| **Companion documents** | `regulatory/mdr-classification.md` (esp. §1.4a, §5, §11, §12, Annex A); `clinical-safety/SAFETY-CASE.md` |

---

## The position, in one paragraph

Published source code, standing alone, is not a medical device. It is a written specification of an algorithm - literature in a machine-readable format - which exhibits no behaviour and can cause no patient harm in its published state. A regulated medical device comes into being when a *finished, ready-to-use instance* is created and supplied with a medical intended purpose. The regulated obligations therefore attach to whoever compiles, integrates, deploys, or ships that ready-to-use instance - the manufacturer or the deploying institution - and not to the upstream author who published the source. This is not a claim that open-source status changes anything; it is a claim about the difference between *a written description of a mechanism* and *the working mechanism itself*.

---

## 1. The claim, precisely stated

1. **Published source code is not, in itself, a medical device.** As published, it is not a finished product, it is not "ready for use" by any final user, it has not been "put into service", and - where its author declares a non-medical intended purpose for it (component, reference implementation, research artefact) - it does not qualify as a device at all.
2. **The regulated device is the finished, ready-to-use instance placed on the market or put into service with a medical intended purpose.** That is the artefact the regulation is written to control.
3. **The manufacturer / deployer is whoever brings that ready-to-use instance into being** - by compiling, integrating, packaging, distributing, or deploying it for clinical use - not the person who published the source it was built from.

### A deliberate framing choice: "ready for use", not "currently executing"

The intuitive version of this argument is "it only becomes a device when it runs on a machine". That intuition is correct in spirit but is **not the line the law draws, and is more aggressive than the position needs**. The defensible, statute-anchored line is **"finished product, ready for use"**, not "the CPU is presently executing instructions". The reasons this matters:

- The statutory trigger for obligations is *placing on the market / putting into service* of a device, and "putting into service" is defined as being *"made available to the final user as being **ready for use**... for its intended purpose"* (EU MDR Article 2). "Ready for use" is in the text; "executing" is not.
- Arguing "execution" would force the untenable claim that a *finished, installable, double-clickable clinical application* is not a device until its CPU runs it. That contradicts settled practice and would discredit the strong part of the case.
- "Ready for use" cleanly separates the two things that matter: **published source that must be built** (not ready for use → not a device) from **a shipped, runnable product** (ready for use → a device). Source code fails the "ready for use" test; that is the whole argument, and it does not depend on the metaphysics of runtime.

**The position is therefore: source code that a person must still compile, configure and integrate is not a finished device "ready for use", and is not placed on the market as a device by the act of publishing it.**

---

## 2. Statutory anchors

The position is built on the regulation's own defined terms, not on an engineering ontology.

- **Intended purpose.** A device is defined by the manufacturer's *intended* medical purpose (UK MDR 2002 reg 2; EU MDR Article 2(1)). Publishing source code with an expressly non-medical intended purpose (a software component / reference implementation / research and educational artefact) is evidence against manufacturer status. The author controls this, and must keep the *whole presentation* consistent with it (see §4, CA5).
- **"Software" is a device category, but only when it meets the definition.** The definition lists "software" alongside instruments and apparatus. This does not make all software a device; it makes *software intended for a medical purpose and placed on the market as a finished product* a device. Source code that is neither finished nor placed on the market as a device is not caught merely because "software" appears in the list.
- **"Making available on the market" / "placing on the market".** Defined as supply of *a device* for use on the market, "whether in return for payment or free of charge" (EU MDR Article 2). Free supply is not a defence; but the object supplied must be *a device*. Publishing source that is not yet a finished, ready-to-use device is not making a device available.
- **"Putting into service" = "ready for use".** The pivotal term (above). Source that must be built is not "ready for use" by a final user.
- **"Manufacturer" = markets the device under its name or trademark.** The obligations fall on whoever markets the *finished device*. The upstream publisher of source is not marketing a finished device.
- **MDSW "finished product" concept.** MDSW qualification guidance (MDCG 2019-11) treats medical device software as a product in its own right. *Action for legal review: pin the exact "finished product" language and whether it supports the "source is not yet a finished product" reading.*
- **Source code as expression.** US courts have held source code is protected speech (*Bernstein v. United States*; *Junger v. Daley*). These are First-Amendment cases, not device-classification cases, so they are *analogical support* for "published source is a form of literature/expression", not binding authority on MDR classification. Cite them for the ontology, not the conclusion.

---

## 3. The purposive argument

Medical-device regulation exists to manage the risk that a device's *behaviour* harms a patient. Behaviour is a property of a working mechanism. Static source code exhibits no behaviour: it computes nothing, displays nothing, and is incapable of interacting with a patient or a clinician until it is built and run. The risk the regulation is designed to control does not exist until a working instance exists. Regulating the published text as if it were the mechanism would regulate something that cannot, in its published state, produce the harm the regulation targets - and would do grave collateral damage to open-source health software, which depends on the free publication of exactly this kind of algorithmic literature.

**Honest limit of this argument.** The regulation *does* attach obligations to intended-use artefacts in their dormant state - an unpowered infusion pump in a warehouse is a regulated device though it can harm no one as it sits (see §4, CA3). So "it cannot harm anyone in its published state" is true but is not, on its own, decisive under the current framework. That is exactly why the position is anchored on **"finished product / ready for use"** rather than on "can it cause harm right now": the pump is finished and ready for use; published source is not.

---

## 4. Counter-arguments and answers

The position must survive these. Each is the argument a regulator's counsel would make.

**CA1 - "Software is a named device type; the definition never says 'executing software', so intended-for-medical-purpose software is a device regardless of runtime."**
*Answer.* Agreed that runtime is not the test - which is why this position is not argued on runtime. The test is whether a *device* has been placed on the market / put into service. "Software" in the definition means *software that is a finished product with a medical intended purpose*; published source that must still be compiled and integrated is not that finished product, and its author can declare a non-medical intended purpose for it. The category "software" does not collapse the distinction between a finished software product and its source specification.

**CA2 - "The regulated event is market-placement, not creation. Uploading an app to a store is placing on the market; that happens before any user runs it."**
*Answer.* Correct, and consistent with this position. Market-placement of a *ready-to-use product* is the trigger - so a finished, installable clinical app *is* placed on the market at distribution, and this position does not dispute that. But publishing *source that is not a ready-to-use product* is not the market-placement of a device. This is precisely why the line is "ready for use / finished product", and why the position concedes the shipped-binary case (see §5).

**CA3 - "The unplugged infusion pump in a warehouse cannot harm anyone either, yet it is a regulated device. So 'harmless in its static state' proves nothing."**
*Answer.* The pump is a *finished device, ready for use*; it needs only power. Source code is *not* finished and *not* ready for use; it needs compilation, configuration and integration by a competent party before any working instance exists. The analogy therefore supports the position: dormant *finished* devices are regulated; source code is not a dormant finished device, it is the specification from which one could be built.

**CA4 - "Source published with build instructions is a device supplied in kit form, intended to be assembled into a medical device."**
*Answer.* A kit doctrine requires that the supplied elements are *intended by the supplier* to be assembled into a device with a medical purpose. Where the author's declared intended purpose is a non-medical component / reference / research artefact, and the source requires genuine engineering work (not mere assembly) and independent intended-purpose decisions by the integrator, the kit analogy does not hold. *Action for legal review: confirm how the assembly/kit provisions are worded and whether "must be programmed / integrated" defeats them.*

**CA5 - "A 'not a medical device' label cannot override an intended purpose evidenced by how the thing is actually presented and marketed."**
*Answer.* Agreed, and this is the real constraint on the position, not a refutation of it. The non-medical intended purpose must be *genuine and consistent across the whole presentation* (README, docs, marketing). Where the same project also ships clinician-facing, ready-to-use surfaces and markets them for point-of-care use, those surfaces are a *different artefact* with a *different intended purpose*, and the position does not shield them (see §5). The remedy is to keep the published-source/library presentation genuinely non-medical and to handle the clinician-facing surfaces separately - not to rely on a disclaimer bolted onto an inconsistent whole.

---

## 5. Scope of the position - airtight vs. contested

| Artefact | Position | Confidence |
|---|---|---|
| **Published source that must be compiled / integrated** (e.g. `clincalc` (with `default-features = false`) as a crate/library) | Not a device: not a finished product, not "ready for use", not put into service; non-medical intended purpose declarable. | **Strong** - built on the "ready for use" statutory term; converges with how the law is actually enforced. |
| **A finished, ready-to-run product the author ships** (e.g. a Tauri GUI installer; a `calc` binary handed to clinicians ready to run, marketed for point-of-care use) | The position does **not** shield this. Distributing a ready-to-use software product with a medical intended purpose is placing a device on the market; the author is the manufacturer; the clinician who runs it is a user, not the manufacturer. | **This is the boundary.** Arguing otherwise (i.e. "not a device until the clinician executes it") is the losing framing and should not be asserted. |

**Consequence for how you distribute.** If the intention is that the *runner / integrator* carries the regulatory responsibility, the mechanism cannot be "it is not a device until run". It has to be one of: (a) distribute source / components that are not "ready for use", not finished clinician-facing products; (b) rely on the health-institution in-house exemption (EU MDR Article 5(5)) for deployments within a single legal entity; or (c) genuinely constitute the integrator/deployer as the manufacturer of their finished product - which shipping a finished product yourself does not achieve.

---

## 6. Application to calc

- **`clincalc` as published source (`default-features = false` for the pure engine)**: sit within the strong zone, *provided* the repository's stated intended purpose is a developer component / reference implementation / research artefact (see `regulatory/mdr-classification.md`, Annex A) and the presentation is kept consistent with that.
- **The clinician-facing surfaces** (a distributed Tauri GUI; a CLI promoted to clinicians as a bedside tool): sit on the contested boundary. These are the artefacts to either (i) keep clearly demarcated as evaluation / research / developer tools, (ii) deploy only under Article 5(5) in-house arrangements, or (iii) accept as devices with their own conformity route. This is a per-surface decision.
- The two positions are consistent: the conservative classification in `mdr-classification.md` ("borderline, tips to device for a clinician-facing distribution") and this statement (published source is not a device) describe *different artefacts*. The published library is not the placed-on-market clinician product.

---

## 7. Status and what is needed

- **Not settled law.** No decided case on this exact fact pattern (published open-source clinical-algorithm library) was found. That is why the position is worth establishing - and why it must not be asserted as settled.
- **Specialist legal review required** on: the exact scope of "ready for use" and "put into service"; the MDSW "finished product" concept in MDCG 2019-11; the assembly/kit provisions; and whether a declared non-medical intended purpose for a published library is respected in practice by the MHRA and EU competent authorities.
- **Precedent to study**: QRISK / ClinRisk (see `mdr-classification.md` §12) - the open algorithm was published freely, yet the *productised, runnable engine offered for clinical use* was registered as a Class I device. This supports the position's *dividing line* (source vs. placed-on-market engine) while cautioning that a *runnable engine offered for clinical use* is likely a device.
- **Policy dimension**: the position aligns with the interest of open-source health software and is likely to attract support from the regulatory-affairs and open-source clinical communities. Consider engaging the MHRA directly (borderline pre-submission / policy engagement) before, or instead of, litigation.

---

## External references

| Ref | Title | Source | Used in |
|---|---|---|---|
| EU-MDR-ART-2 | EU MDR 2017/745 Article 2 - definitions ("making available", "putting into service", "manufacturer", "intended purpose") | EUR-Lex | §1, §2 |
| UK-MDR-2002 | Medical Devices Regulations 2002 (as amended) reg 2 - definition | legislation.gov.uk | §2 |
| EU-MDR-ART-5-5 | EU MDR Article 5(5) health-institution exemption; MDCG 2023-1 | European Commission MDCG | §5 |
| MDCG-2019-11 | Qualification and classification of software under MDR/IVDR | European Commission MDCG | §2, §7 |
| BERNSTEIN | Bernstein v. United States - source code as protected speech | US 9th Cir. | §2 |
| JUNGER | Junger v. Daley - source code as protected speech | US 6th Cir. | §2 |
| CALC-MDR | calc SaMD/AIaMD classification assessment | This repository - `regulatory/mdr-classification.md` | Throughout |

---

## Important

This is an argued position for legal development, not advice and not a determination. Do not assert it to a regulator, a court, a customer, or the public as settled law. Have it reviewed and shaped by a qualified medical-device regulatory litigator, and reconcile it with the conservative classification in `regulatory/mdr-classification.md` before relying on either.
