# Example inputs

Ready-to-run JSON inputs for the `clincalc` CLI, one per file. They exist so you never have to invent your own test data: clone the repo and pipe a file straight in.

```bash
clincalc feverpain --input examples/feverpain.json   # from the file
cat examples/auditc.json | clincalc auditc --input -  # or over a pipe
```

| File | Calculator | What it shows |
|---|---|---|
| `feverpain.json` | `feverpain` | Five yes/no criteria (booleans) - a sore-throat case scoring 3 (delayed antibiotic). |
| `gad7.json` | `gad7` | A questionnaire: seven 0-3 responses summing to 11 (moderate anxiety). |
| `auditc.json` | `auditc` | An array plus an enum (`sex`) - an alcohol screen scoring 7 (positive). |
| `news2.json` | `news2` | Mixed vitals with enums - an acutely unwell patient scoring 7 (high). |

These are also the worked examples in the [Walkthrough](https://pacharanero.github.io/clincalc/walkthrough/). Each calculator's full input contract is `clinclinclincalc <name> --schema`; a fillable template is `clinclincalc <name>`.
