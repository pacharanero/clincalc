# Walkthrough

Four real calculators, end to end, in the order you'd actually meet them. Every command below is copy-pasteable; every example file is committed to the repo so you never need to invent your own input.

If you haven't installed `clincalc` yet, head to [Install](install.md) first - it takes one line.

!!! tip "How to read this page"
    Each section follows the same shape: **discover** the calculator, **ask** it for a template, **fill** the template, **compute**. Once you've done it once, you've done all 53 active calculators.

## Before you start

Every calculator is driven through the same four moves. There are no per-calculator flags to learn:

```bash
clincalc list                       # what's available
clincalc calc <name>                # print a fillable JSON template
clincalc calc <name> --schema       # the full JSON Schema (the formal contract)
clincalc calc <name> --input <json> # compute - file path, `-` for stdin, or inline
```

Computing always needs an explicit `--input`, so a bare `clincalc calc <name>` is pure discovery and will never block waiting on stdin. `clincalc <name>` remains supported as shorthand.

---

## 1. FeverPAIN - five yes/no criteria

FeverPAIN is a five-criterion score for acute sore throat that guides antibiotic prescribing. It's the gentlest possible introduction: every input is a boolean.

=== "Discover"

    ```console
    $ clincalc list | head -3
    feverpain     FeverPAIN Score
    asrs          ASRS-v1.1 Adult ADHD Screener
    phq9          PHQ-9 Depression Severity
    ```

=== "Ask for a template"

    ```console
    $ clincalc calc feverpain
    {
      "absence_of_cough": "<boolean> No cough or coryza",
      "attend_rapidly": "<boolean> Symptom onset within 3 days (≤ 3 days)",
      "fever": "<boolean> Fever in the last 24 hours",
      "inflamed_tonsils": "<boolean> Severely inflamed tonsils",
      "purulence": "<boolean> Purulence (pus on the tonsils)"
    }
    ```

    Each placeholder describes the value the calculator wants. The template's shape is the input's shape; fill in and pass it back.

=== "Compute"

    The repo ships a ready-made input:

    ```json title="examples/feverpain.json"
    --8<-- "examples/feverpain.json"
    ```

    Pipe it in:

    ```console
    $ clincalc calc feverpain --input examples/feverpain.json
    feverpain = 3

    A score of 3 is associated with 34–40% isolation of streptococcus. A delayed prescribing strategy is appropriate after discussion with the patient.

    Working:
      attend_rapidly_criterion: true
      fever_criterion: true
      inflamed_criterion: false
      level: delayed
      no_cough_criterion: false
      prescribing_recommendation: Delayed antibiotic prescribing
      purulence_criterion: true
      score: 3
      streptococcus_rate: 34–40%

    Reference: Little P, Stuart B, Hobbs FDR, et al. Lancet Infect Dis. 2014. Little P, Hobbs FR, Moore M, et al. Health Technol Assess. 2014;18(6):1-102.
    ```

That text block is the headline. It is a clean, paste-able clinical summary - the result, the interpretation, every intermediate value as `working`, and the primary citation. Drop it straight into a letter, a record, or a message.

!!! note "Soft interoperability"
    Copy-and-paste is often derided as a kludge, but it is what clinicians actually use. `clincalc` treats this textual summary as a **first-class output**, not an afterthought.

---

## 2. GAD-7 - a questionnaire as an array

GAD-7 (Generalised Anxiety Disorder, 7-item) is a questionnaire: seven items each scored 0-3 (not at all → nearly every day). Inputs that are *lists of equivalent things* arrive as JSON arrays.

=== "Inline JSON"

    The compact way - paste the whole input on the command line:

    ```bash
    clincalc calc gad7 --input '{"responses":[2,2,2,1,1,1,2]}'
    ```

=== "From the repo"

    Or use the committed example:

    ```json title="examples/gad7.json"
    --8<-- "examples/gad7.json"
    ```

    ```bash
    clincalc calc gad7 --input examples/gad7.json
    ```

Either way:

```console
gad7 = 11

Total score 11/21 indicates moderate anxiety symptoms. At or above the cut-point of 10 for likely generalised anxiety disorder; further assessment is warranted. GAD-7 supports severity grading; it is not a diagnosis.

Working:
  above_case_threshold: true
  answers: [2,2,2,1,1,1,2]
  severity: moderate
  total_score: 11

Reference: Spitzer RL, Kroenke K, Williams JBW, Löwe B. A brief measure for assessing generalized anxiety disorder: the GAD-7. Arch Intern Med. 2006;166(10):1092-1097. doi:10.1001/archinte.166.10.1092
```

The PHQ-9 (depression) and AUDIT (alcohol) calculators follow the identical pattern - `responses` as an array of 0-3 (or 0-4) integers.

---

## 3. AUDIT-C - mixed types via stdin

AUDIT-C is a three-item alcohol screen. Its input mixes a numeric array with an enum (`sex`), because the threshold for a positive screen differs by sex (4 for men, 3 for women). This is also a chance to see `clincalc` reading from **stdin** (`--input -`), which is the shape every Unix pipeline expects.

```json title="examples/auditc.json"
--8<-- "examples/auditc.json"
```

Pipe the file in:

```console
$ cat examples/auditc.json | clincalc calc auditc --input -
auditc = 7

Total score 7/12 indicates higher risk (male). At or above the validated cut-point of 4 for male patients; the screen is positive for hazardous drinking or a possible alcohol use disorder, and warrants further assessment. Also at or above the higher-specificity unisex cut-point of 5 used by some services. AUDIT-C is a screen for consumption-related risk; it is not a diagnosis.

Working:
  above_higher_specificity_threshold: true
  answers: [3,2,2]
  risk_band: higher risk
  screen_positive: true
  sex: male
  threshold: 4
  total_score: 7

Reference: Bush K, Kivlahan DR, McDonell MB, Fihn SD, Bradley KA. The AUDIT alcohol consumption questions (AUDIT-C): an effective brief screening test for problem drinking. Arch Intern Med. 1998;158(16):1789-1795. doi:10.1001/archinte.158.16.1789
```

!!! tip "Stdin in the wild"
    Anywhere an upstream tool already produces JSON - `jq`, an LLM, another program - pipe it straight in with `--input -`. No temporary files.

---

## 4. NEWS2 - vitals, enums, and JSON output

NEWS2 (the National Early Warning Score) is what acute-care staff in the NHS use at every set of observations. Its input is the busiest you'll meet: a mix of numbers, an enum (`spo2_scale`), a boolean, and another enum for consciousness.

This is also a good moment to switch to **`--format json`** - the same `CalculationResponse` shape every surface uses. It is what an LLM, a script, or another tool will consume.

```json title="examples/news2.json"
--8<-- "examples/news2.json"
```

```console
$ clincalc calc news2 --input examples/news2.json --format json
{
  "calculator": "news2",
  "result": 7,
  "interpretation": "NEWS2 7 (high). Emergency response: immediate assessment by a critical-care competent team, usually transfer to a higher level of care; continuous monitoring.",
  "working": {
    "air_or_oxygen_score": 2,
    "band": "high",
    "consciousness_score": 0,
    "pulse_score": 1,
    "respiratory_rate_score": 2,
    "single_parameter_3": false,
    "spo2_score": 1,
    "systolic_bp_score": 0,
    "temperature_score": 1,
    "total_score": 7
  },
  "reference": "Royal College of Physicians. National Early Warning Score (NEWS) 2: Standardising the assessment of acute-illness severity in the NHS. Updated report of a working party. London: RCP, 2017."
}
```

Drop the `--format json` and you get the same clinician-facing text block as the earlier calculators.

If you ever need the formal contract - exact field names, types, enumerations, units - ask for the JSON Schema:

```bash
clincalc calc news2 --schema
```

That schema also drives the MCP tool definition when `clincalc` is embedded in an LLM host, so an agent and a human are working from the **same contract**.

---

## Where the licence comes from

Every calculator records the terms its algorithm is distributed under, with a reverifiable URL:

```console
$ clincalc calc feverpain --license
{
  "license": "Public-domain method - implemented from the primary literature (NIHR HTA, open access)",
  "source_url": "https://www.ncbi.nlm.nih.gov/books/NBK261544/"
}
```

The same data is in `clincalc list --format json` for the whole catalogue, so an auditor can grep the basis on which every score in the library is shipped.

---

## Tools we name but cannot ship

A handful of widely-used clinical tools (FRAX, MMSE, MUST, CAT, ACQ, ELF, CFS, LANSS, OHS, OKS) are licence-locked or proprietary. `clincalc` lists them, but invoking them returns a structured explanation rather than a score:

```console
$ clincalc calc frax --input '{}'
frax = unavailable: proprietary

FRAX (10-year fracture risk) is not available here because it is proprietary or licence-locked. Owner: University of Sheffield (Centre for Metabolic Bone Diseases). The FRAX algorithm and its country-specific coefficients are a trade secret and have never been published, so it cannot be reimplemented from primary literature. ...
```

The point is to make the *gap* a first-class object. Where an open alternative exists (QFracture for FRAX, AMTS for MMSE), it is named in the response.

---

## Next steps

- Browse the full [Calculator catalogue](calculators.md) - 53 active calculators plus 10 named-but-unavailable.
- Read the [CLI reference](cli-reference.md) for every mode, flag, and exit code in one place.
- See [How it works](how-it-works.md) for the one-core-many-surfaces design and embedding `clincalc` in your own host.
