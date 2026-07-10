# Changelog

All notable changes to `calc` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows [SemVer](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2026-07-10

### Added

- **mcp**: Add registry-backed MCP server ([68645ce](https://github.com/pacharanero/clincalc/commit/68645ce4ab6763d407a2b75f80c7e1affb5113d4))

## [0.2.0] - 2026-07-05

### Added

- **cli**: Shell completions ([a01f17d](https://github.com/pacharanero/clincalc/commit/a01f17db5aea7fa83860577c7f0adacdd3821a69))

### Build

- **deps**: Bump Swatinem/rust-cache ([#1](https://github.com/pacharanero/clincalc/issues/1)) ([be56905](https://github.com/pacharanero/clincalc/commit/be569058078ab284981d2130f2c9d7ce043de98c))

### Changed

- Single `clincalc` crate + cargo-dist release pipeline **[breaking]** ([5ee7c63](https://github.com/pacharanero/clincalc/commit/5ee7c6351fe412a659d4fc1e2aa48541ec6ed388))

### Dependencies

- **deps**: Bump actions/setup-python from 6.2.0 to 6.3.0 ([#5](https://github.com/pacharanero/clincalc/issues/5)) ([f108b0b](https://github.com/pacharanero/clincalc/commit/f108b0ba27bfc45b1faa1707e400f7576f41090f))

### Documentation

- **safety**: DCB0129 clinical safety case, MDR classification, and position statement ([03b234f](https://github.com/pacharanero/clincalc/commit/03b234f36ee5b1a51b7e5f6ee58c31525479bf67))

### Fixed

- **gui**: Drop version pin on the clincalc path dependency ([6afb5ae](https://github.com/pacharanero/clincalc/commit/6afb5ae685f20337ea84831c9b6dddcd276ffb3a))

## [0.1.0] - 2026-06-29

### Added

- **release**: Publish calc-core to crates.io via release-plz; top-level ROADMAP.md ([53f3145](https://github.com/pacharanero/clincalc/commit/53f314565bf1b6ab51f011b03a2796eaf6d6260b))

- **gui**: Tauri 2 desktop app with FeverPAIN as the MVP calculator ([467e018](https://github.com/pacharanero/clincalc/commit/467e01844b0e7bd84e91ba72f3805c569d786fae))

- **calc**: Tag taxonomy + multilingual design + MedikQuantis candidates ([8cd019d](https://github.com/pacharanero/clincalc/commit/8cd019d6d1d47ec48950a40c5353e31bba799590))

- Re-establish tools as the home of the GitEHR calculators ([38efb7c](https://github.com/pacharanero/clincalc/commit/38efb7c1bd297d4443f63cb04c3211301f103d7a))

- **calc**: Require a distribution licence + evidence URL per calculator ([654e23c](https://github.com/pacharanero/clincalc/commit/654e23c5fa6be83efedaf52349ab74a857bb5afe))

- **calc**: Replace per-calculator flags with one JSON template surface ([597794b](https://github.com/pacharanero/clincalc/commit/597794b85af43aba2812ec17a25e694d32dc491b))

- **calc**: Add QRISK3 and QFracture (LGPL, validated against ClinRisk) ([922d6a2](https://github.com/pacharanero/clincalc/commit/922d6a27927f308f9eeeee24201013d4df980877))

- **calc**: Add Gleason, NPI, CHALICE, KDIGO CKD-risk, GRACE, EuroSCORE II ([94007e4](https://github.com/pacharanero/clincalc/commit/94007e40203c8385376f5b8debd5f2cf0edf303e))

- **calc**: Add Padua, UKELD, NHFS, BODE, ABPI, Waterlow; CFS + LANSS stubs ([862b83b](https://github.com/pacharanero/clincalc/commit/862b83bb646dd7afea17c56f23013bc4f5590a53))

- **calc**: Add DAS28, uACR, SOFA, HEART, TIMI, Child-Pugh, MELD; MUST stub ([45fec91](https://github.com/pacharanero/clincalc/commit/45fec91beab3a3b51119bc14576f098142bb835c))

- **calc**: Add NEWS2, CURB-65, Wells DVT/PE, HAS-BLED, ABCD2, qSOFA, 4AT ([72e99c4](https://github.com/pacharanero/clincalc/commit/72e99c486dd4f3a1e235e4789af29c011ddcd1e7))

- **calc**: Add AUDIT-C, AUDIT, EPDS, IPSS, AMTS, MRC Dyspnoea; CAT stub ([896d901](https://github.com/pacharanero/clincalc/commit/896d901dd63548e7bcd427329e895c65f79ec836))

- **calc**: Protest stubs for proprietary / licence-locked calculators ([5ef856a](https://github.com/pacharanero/clincalc/commit/5ef856a3f020a5920ba353ae9b1a13dd92b7b79e))

- **calc**: Add CHA2DS2-VASc with the full input-definition treatment ([3b1e28f](https://github.com/pacharanero/clincalc/commit/3b1e28fce15a076a202de79d0c5aef8a8c387b3e))

- **calc**: Require a distribution licence + evidence URL per calculator ([4a81765](https://github.com/pacharanero/clincalc/commit/4a817656f74720e91b30b2debf14a55a53233b77))

- **calc**: Add eGFR (CKD-EPI 2021) and FIB-4 calculators ([da3a403](https://github.com/pacharanero/clincalc/commit/da3a4030eadf8c12a3047c9720bc6ac2f5958746))

- **calc**: Add PHQ-9 and GAD-7; design input-definition system; calc docs ([7b9188a](https://github.com/pacharanero/clincalc/commit/7b9188a4e66bcd38b1907adf3f3d4395cf682296))

- **calc**: Replace per-calculator flags with one JSON template surface ([4992c7e](https://github.com/pacharanero/clincalc/commit/4992c7e1f9273161f831da946905cc21bd1fa1aa))

### Deprecate

- Clinical calculators moved into the GitEHR repo ([fb95616](https://github.com/pacharanero/clincalc/commit/fb956164987fd2f1c52cb840848902596fd92ec1))

### Documentation

- **roadmap**: Add StatinMD risk calculator to wishlist ([605645a](https://github.com/pacharanero/clincalc/commit/605645abe2db4d77cd7653932057dc8658137432))

- **brand**: Function-variant logo + teal palette (off NHS Blue) ([32fcbd8](https://github.com/pacharanero/clincalc/commit/32fcbd8864d34db71db0f98cf31508dc4e832852))

- Flatten roadmap by completion; catalogue as one table ([8c29500](https://github.com/pacharanero/clincalc/commit/8c295000c2478a337b5306bdf8ad4c0b6c724f02))

- **spec**: Reformat roadmap as checklists; add engineering section ([22d12f8](https://github.com/pacharanero/clincalc/commit/22d12f82461d0beca37503aaa248598498768c46))

- Add Zensical site, AGENTS.md, refresh stale spec ([d9cc9d7](https://github.com/pacharanero/clincalc/commit/d9cc9d77ece4de46c70026372b0c3e81b1fb8467))

- **calc-core**: Correct stale --print-schema reference to --schema ([849e69f](https://github.com/pacharanero/clincalc/commit/849e69f744a63ca4bdb4b462ba2c8285a010080c))

### Fixed

- **release-plz**: Drop unsupported filter_commits field ([5ddeea9](https://github.com/pacharanero/clincalc/commit/5ddeea922e854cb4b6fef94bf30899315a4ab10f))

- **gui**: Capture checkbox value before async setState updater ([171a169](https://github.com/pacharanero/clincalc/commit/171a1693c3f9dde4ccecea1b8413feef31db15ae))

- **gui**: Surface render errors via ErrorBoundary; harden response reads ([0842cf8](https://github.com/pacharanero/clincalc/commit/0842cf84f4c6fd51366f56b0c264b1cbcdec32ba))

- **calc-cli**: Show real response for inputless calculators ([f6cfb3a](https://github.com/pacharanero/clincalc/commit/f6cfb3aab8b263a95428c805ba58f78abb8b1cae))

- **calc-core,calc-cli**: Templates round-trip for every calculator ([53120ce](https://github.com/pacharanero/clincalc/commit/53120ce86b14e6cf095f51adced7b7bbc37155df))


