# Case studies

Worked examples of a problem this project exists to name: clinical decision tools that public healthcare relies on being locked behind licences that were never justified by the underlying intellectual content. Each case study below is grounded in an actual audit performed while building clincalc's registry, with sources, not speculation.

## Why isn't there an open ADHD scoring tool?

clincalc ships exactly one ADHD instrument: `asrs`, the WHO-developed six-question adult screener, because NYU and Harvard (who hold the rights jointly) explicitly grant free clinical, non-clinical, and commercial use with attribution. That grant is the exception. Every other ADHD rating scale or structured interview in routine UK and US clinical use was checked against the same bar - an explicit, citable statement from the rights holder permitting unrestricted reproduction or reimplementation - and every one of them failed it, for reasons that have nothing to do with the clinical content and everything to do with how each instrument happened to get monetised once computers entered the room.

### What a UK ADHD assessment actually uses

NICE NG87 does not mandate a specific instrument; it asks for a comprehensive clinical and developmental assessment. In practice, both NHS services and Right to Choose providers converge on a small toolkit, documented in RCPsych's own good-practice guidance:

- **DIVA-5**, the structured diagnostic interview, for adults. It maps directly to DSM-5 criteria and is the instrument RCPsych names explicitly.
- **CAADID** (Conners' Adult ADHD Diagnostic Interview), the commercial alternative structured interview.
- Supporting rating scales: **CAARS** (Conners) and the **Barkley scales**, both used as corroborating evidence alongside the structured interview.
- **WURS**, sometimes used specifically to establish retrospective childhood symptom onset.

Every one of those, bar the already-shipped ASRS, is rights-managed:

| Instrument | What was found |
|---|---|
| Conners' Rating Scales / CAARS / CAADID | Definitively proprietary. MHS's own manuals state the assessment "cannot be adapted, including the creation of a customized version, adapted scoring algorithms, or deploying the assessment in an alternate digital platform or format without the express written permission of MHS." |
| DIVA-5 | Run by a non-profit (the DIVA Foundation) in obvious good faith, but not free: a per-clinician fee applies, and the instrument text is itself "reprinted with permission" from the DSM-5, adding a second rights dependency (the APA's) on top of the Foundation's own terms. |
| Vanderbilt ADHD Diagnostic Rating Scales | NICHQ offers only the 2002 first edition free, with an attribution request, not an unrestricted reproduction grant. It is co-authored with a pharmaceutical company, and the current (third) edition is sold commercially through the AAP Bookstore. |
| SNAP-IV | Distributor statements directly contradict each other: one states it is "printed with permission. All rights reserved," another secondary source claims it is public domain. The restrictive statement is the one consistent with how the instrument is actually gatekept. |
| Wender Utah Rating Scale (WURS) | No unrestricted-reproduction statement in the 1993 primary paper. It is tracked by MAPI Trust's ePROVIDE, a registry that generally exists because an instrument's use is being actively managed, not because it is free. |

Not one of these restrictions traces back to anything inventive in the instrument itself. A rating scale is a short list of questions and a summation rule. None of that is copyrightable subject matter under the expression/method distinction that underpins every other calculator in this registry (see [WIPO Copyright Treaty Article 2](https://www.wipo.int/wipolex/en/text/295166)): copyright protects the specific wording and typesetting of a questionnaire, not the diagnostic method it operationalises. What's actually being licensed is the printed form, the scoring sheet, and - increasingly - the software that administers and scores it.

### The pre-digital counterfactual

Compare this with Kurt Schneider's First Rank Symptoms of schizophrenia (Schneider K. *Klinische Psychopathologie*. 1946/1959; English translation *Clinical Psychopathology*). It is, structurally, exactly the same kind of object as a modern ADHD screener: a clinician reads a defined list of phenomena, checks which are present, and a defined rule turns the checklist into a diagnostic judgement. It was published in a book, cited by name in textbooks and papers for seventy years, taught to every psychiatry trainee, and reimplemented in every rating instrument and structured interview descended from it - all without anyone needing a licence, a per-clinician fee, or a portal account. Nobody tried to stop a hospital from printing it on a ward checklist, because in a pre-digital era there was no practical mechanism to do so, and because the idea that you would try was not really on the table: a diagnostic method was something you published and let the field use.

What changed is not the legal status of diagnostic rating scales - the expression/method distinction was just as true in 1959 - but the available *infrastructure* for metering access to them. A paper questionnaire photocopied on a ward is unenforceable. A questionnaire gated behind a vendor portal, a per-use licence fee, a "no derivative digital platforms" clause, and a scoring algorithm you're contractually forbidden from reimplementing is enforceable, because digitisation gave rights holders a point of control - the software - that a photocopier never offered. The content didn't become more ownable. The distribution channel became ownable, and the content rode along on top of it. That is the proprietorisation this project keeps running into: not a change in what the law protects, but a change in what's practically possible to lock down once a clinical tool has to pass through a screen rather than a printed page.

### What it would take to fix this

Nothing about any of these instruments is clinically irreplaceable - the underlying diagnostic logic in DSM-5 and ICD-11 is open, and a from-source reimplementation of e.g. SNAP-IV's or the Vanderbilt's item structure and scoring rule, done independently from the published validation papers rather than copying the rights holder's typeset questionnaire, may well be defensible under the same method/expression reasoning already used elsewhere in this registry. clincalc deliberately doesn't take that route without an explicit grant, because "probably defensible" is not the bar this project holds every other calculator to, and getting it wrong on a diagnostic instrument used to gate medication access is a worse failure mode than leaving the gap visible.

The three candidates audited here are tracked, not abandoned: [`CALC-090`](https://github.com/pacharanero/clincalc/blob/main/spec/calculator-roadmap.md) (Vanderbilt), `CALC-091` (SNAP-IV), and `CALC-092` (WURS) sit in the roadmap's rights-review queue, the same queue holding Norton and SCORE2. If any rights holder confirms unrestricted reimplementation terms - the way NYU and Harvard already have for the ASRS - the calculator ships the same week.
