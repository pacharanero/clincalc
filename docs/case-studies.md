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

See the next case study for how long that queue is actually likely to stay blocked.

## How long will these restrictions actually last?

The usual assumption about an over-aggressively licensed old work is that time fixes it: copyright is a fixed term, so eventually even a locked-down clinical scale lapses into the public domain the way a Dickens novel or a Beethoven score already has. The table below checks that assumption against every instrument clincalc's registry currently flags as proprietary or rights-review, plus the three ADHD candidates above, using each instrument's first publication date and, where it is publicly known, its named author's date of death.

This is not legal advice, and it is not a legal opinion on any specific instrument's current rights-holder or copyright status - several of the "owner" fields in clincalc's own registry are publishers or institutions, not the original named academic authors, and whether a given instrument's copyright was personally retained, assigned, or created as a work-for-hire changes which rule applies and was not independently confirmed for every entry below. It uses two settled statutory rules as illustrative yardsticks: the UK's life of the author plus 70 years from the end of that calendar year ([Copyright, Designs and Patents Act 1988, s.12](https://www.legislation.gov.uk/ukpga/1988/48/section/12)), and the US's equivalent life-plus-70 for individually-authored works created after 1 January 1978, or a fixed 95 years from publication for US works published before that date, or for corporate/work-for-hire authorship, 95 years from publication or 120 years from creation, whichever ends earlier ([US Copyright Office Circular 15A](https://www.copyright.gov/circs/circ15a.pdf)).

| Instrument | First published | Named author(s) | Author status | Illustrative earliest public-domain year |
|---|---|---|---|---|
| Norton Scale | 1962 | Doreen Norton | Died 30 Dec 2007 | End of 2077 (UK, life+70) |
| Conners' Teacher Rating Scale (original) | 1969 | C. Keith Conners | Died 5 Jul 2017 | 2064 (US, fixed 95y from publication - pre-1978 works use a fixed term regardless of when the author later died) |
| Mini-Mental State Examination (MMSE) | 1975 | Folstein, Folstein, McHugh | Not confirmed | 2070 (US, fixed 95y from publication) |
| Serum liver-fibrosis marker panel underlying `elf` | 1996-2004 (Rosenberg et al, *Gastroenterology* 2004) | Rosenberg et al | Not confirmed | Categorically different regime - see below |
| Oxford Hip Score | 1996 | Dawson, Fitzpatrick, Carr, Murray | Not confirmed | Not calculable without a death date; life+70 almost certainly runs past 2070 |
| Oxford Knee Score | 1998 | Dawson et al | Not confirmed | Same as above |
| SCORAD | 1990/1993 | European Task Force on Atopic Dermatitis (Stalder, Taieb, et al) | Not confirmed | At least 2063 (70y-from-publication floor for organisational/unknown authorship); longer if a named author's life governs |
| Wender Utah Rating Scale (WURS) | 1993 | Ward, Wender, Reimherr | Wender died 16 Jul 2016 | ~2086-2088 (life+70, or ~95y-from-publication if held as a journal-assigned corporate copyright) |
| LANSS Pain Scale | 2001 | Michael I. Bennett | Living (retired from Leeds, 2022) | Not yet calculable; will not begin running until after his death |
| Conners' Adult ADHD Rating Scales (CAARS) | 1999 | C. Keith Conners | Died 5 Jul 2017 | 2087 (life+70) or 2094 (95y-from-publication if corporate/work-for-hire) |
| Asthma Control Questionnaire (ACQ) | 1999 | Elizabeth Juniper | Presumed living (Professor Emeritus, McMaster; company still trades under her name) | Not yet calculable |
| Vanderbilt ADHD Diagnostic Rating Scales | 2002 | Wolraich et al; NICHQ, AAP, McNeil | Not confirmed | Not yet calculable; life+70 of the last surviving named author |
| Malnutrition Universal Screening Tool (MUST) | 2003 | Malnutrition Advisory Group (BAPEN) | Committee/organisational | At least 2073 (70y-from-publication floor) |
| COPD Assessment Test (CAT) | 2009 | Jones et al; GSK governance board | Not confirmed | ~2104 (95y-from-publication, if corporate-held) |
| Clinical Frailty Scale (CFS) | 2005 | Kenneth Rockwood et al | Living (active professor, Dalhousie) | Not yet calculable |
| DIVA 2.0 / DIVA-5 | 2010 / 2019 | Kooij, Francken | Presumed living | Far too recent to approach any regime's expiry |
| SCORE2 / SCORE2-OP | 2021 | ESC/SCORE2 working group (dozens of co-authors) | Multiple living co-authors | Far too recent |
| FRAX | Never published | Kanis et al, University of Sheffield | N/A | No term applies at all - see below |

Three things stand out.

**The oldest instrument on clincalc's blocked list, Norton's 1962 pressure-sore scale, does not clear UK copyright until the end of 2077** - 115 years after it was published, because Doreen Norton lived another 45 years after writing it. That is the *best-case*, most favourable example on this list: a scale old enough to predate the moon landing, written by someone who has been dead for nearly 20 years, still has over 50 years left to run. Everything written more recently, or by someone who lived longer or is still alive, runs later than that, often considerably later. Kenneth Rockwood, who published the Clinical Frailty Scale in 2005, is an active professor today; if he lives an ordinary lifespan, that scale's copyright will not even start its 70-year countdown until some point in the 2050s or 2060s, and won't lapse until well into the 22nd century.

**Two instruments on this list aren't under a fixed-term regime at all.** FRAX has never been published; the University of Sheffield keeps its coefficients as a trade secret specifically so they can't be reimplemented. A trade secret has no statutory term - it lasts exactly as long as the secret is kept, which in principle is forever. The algorithm behind `elf` sits in similar territory: the three underlying serum markers were published openly in 2004, but the specific scoring weights behind the commercial ELF assay are guarded as a combination of patent claims (which do expire, typically ~20 years from filing - sooner than any copyright on this list) and undisclosed proprietary calibration, which doesn't. Trade secrecy is not a weaker lock than copyright. It's a stronger one, because it has no clock running against it at all.

**Several of these instruments are also actively re-issued in updated commercial editions** - the Vanderbilt scales are on their third edition, Conners' instruments have been through multiple revisions (Conners 3, Conners 4, CAARS 2), MUST has had several "reviewed and reprinted" cycles. Each new edition is itself a fresh copyrightable work with its own publication date, meaning the clock conceptually resets on the version actually being distributed and sold even once the original edition's term would eventually lapse. For an actively-maintained commercial instrument, "eventually becomes generic" may not be a real deadline at all - just a receding horizon.

This is a choice, not a law of nature for validated clinical scores. ClinRisk, the company behind QRISK3, QFracture, and QDiabetes-2018 - all three shipped in this registry - releases its algorithm source under LGPL-3.0 or AGPL-3.0 specifically so others can implement it faithfully, rather than keeping it as a trade secret the way Sheffield keeps FRAX, or behind a vendor portal the way MHS keeps Conners. Nothing about cardiovascular or fracture risk prediction made that choice easier than diabetes-risk, ADHD, or frailty prediction did. ClinRisk just made it, and the other rights holders on this list, so far, haven't.
