# calc desktop GUI

Tauri 2 + React 19 + Mantine 8 + Vite 7, matching the GitEHR house-style frontend stack so the two apps read as one product family (same fonts, same teal primary).

Status: **MVP**. One calculator (FeverPAIN) is hand-crafted end-to-end. The other 51 calculators in the registry are listed in the sidebar and show a "GUI coming soon" placeholder when selected - the scoring already works via the CLI today.

## Architecture

```
gui/
├── package.json           React / Mantine / Tauri JS deps
├── vite.config.ts         Tauri-aware Vite config (port 5173, no auto-open)
├── tsconfig.json
├── index.html             Loads IBM Plex Sans + Space Grotesk
├── public/
│   └── logo.svg           function-variant icon, currentColor for CSS theming
├── src/
│   ├── main.tsx           Mantine theme (teal, IBM Plex Sans / Space Grotesk)
│   ├── App.tsx            AppShell + sidebar nav + filter
│   ├── App.css            Thin overrides on top of Mantine
│   ├── api/calc.ts        Typed wrappers around Tauri invoke
│   └── calculators/
│       └── FeverPain.tsx  Hand-crafted UI for the MVP calculator
└── src-tauri/
    ├── Cargo.toml         Excluded from the calc workspace (own Cargo.lock)
    ├── tauri.conf.json
    ├── capabilities/      Window permissions
    ├── icons/             32/128/256/.ico/.icns (.icns is currently a renamed PNG; needs real iconutil on macOS)
    └── src/lib.rs         Tauri commands: list_calculators, calculate
```

The Tauri backend is a thin wrapper: `list_calculators` enumerates `clincalc::all()` into a `CalcSummary` shape the frontend can render; `calculate` takes a name + JSON input and returns the same `CalculationResponse` shape every surface produces. **No scoring logic lives in the GUI** - everything defers to `clincalc`.

## Adding a calculator UI

1. Implement the typed React component under `src/calculators/<Name>.tsx`. Use Mantine primitives and follow the FeverPAIN layout as a template:
   - Form on the left, result on the right.
   - Recompute on every change via `useEffect`, no Calculate button.
   - Prominent **paste-ready summary** card (the "soft interoperability" headline) with an editable Textarea and Copy button.
2. Register the component in `App.tsx`'s `IMPLEMENTED` map.
3. (If it should appear in the "Featured" section at the top of the sidebar, add its name to the `FEATURED` array.)

Do not try to auto-generate the form from `input_schema()`. The schema is sufficient for the CLI's generic template, but clinician-facing forms need hand-tuned hints, ordering, and units per calculator. Madness lies that way.

## Local dev

One-off setup:

```bash
cd gui
npm install
# also install the Tauri prerequisites for your OS:
# https://tauri.app/start/prerequisites/
```

Then from anywhere in the repo:

```bash
s/gui-dev
```

That runs the Tauri CLI, which boots Vite (port 5173) and the Rust backend together with hot reload on both sides.

## Production build

```bash
cd gui
npm install
npm run tauri build
```

Outputs:

- **Windows**: NSIS `.exe` installer + MSI in `src-tauri/target/release/bundle/`.
- **macOS**: `.app` + `.dmg`. The bundled `.icns` is currently a renamed PNG; for production macOS builds, regenerate it on a Mac with `iconutil -c icns icon.iconset`.
- **Linux**: `.deb`, `.rpm`, `.AppImage`.

## Windows code signing

Not configured yet. When the EV cert from SSL.com / Sectigo arrives, point Tauri at it via `bundle.windows.certificateThumbprint` (or the env vars `TAURI_SIGNING_CERTIFICATE_THUMBPRINT` etc.). Without a cert, Windows SmartScreen shows the "Windows protected your PC" dialog on first run and users must click "More info → Run anyway".

## Workspace placement

`gui/src-tauri/` is **excluded** from the calc Cargo workspace (root `Cargo.toml` `exclude = ["gui/src-tauri"]`). This is the same arrangement GitEHR uses: Tauri pulls in a very different dependency graph (tao, wry, webkit bindings) that would slow down every `cargo build` for the engine + CLI. The GUI has its own `Cargo.lock`.
