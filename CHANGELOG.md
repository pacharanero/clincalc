# Changelog

All notable changes to `calc` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows [SemVer](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-10-17

### Added

- **calc**: Add Rockall Score for upper GI bleeding (CALC-070) ([#125](https://github.com/pacharanero/clincalc/issues/125)) ([0974779](https://github.com/pacharanero/clincalc/commit/0974779c4e66e7476864c14b3918c763914bfab3))

- **calc**: Add QDiabetes-2018 10-year type 2 diabetes risk (CALC-068) ([#124](https://github.com/pacharanero/clincalc/issues/124)) ([e6e2b61](https://github.com/pacharanero/clincalc/commit/e6e2b6105ad8add891c16ac1ee1b3106dd93fabb))

- **calc**: Add Durnin-Womersley 4-site skinfold body fat % (CALC-067) ([4679b17](https://github.com/pacharanero/clincalc/commit/4679b1702b4c82f0a460396d4ae7aa1f6ec09cec))

- Add CentileReference trait for LMS centile references (ENG-010.3) ([b8494d0](https://github.com/pacharanero/clincalc/commit/b8494d04aa2da8f0066c1979ec94db5cbab8e43e))

- **engine**: Add LMS linear and cubic Lagrange interpolation (ENG-010.2) ([b53b99e](https://github.com/pacharanero/clincalc/commit/b53b99e98bd5bead0a860ff908f2ff8a785e3e32))

- **engine**: Add LMS Box-Cox and normal CDF transforms (ENG-010.1) ([55aff37](https://github.com/pacharanero/clincalc/commit/55aff37025a0e5cc4ab22688f5c62ddb2205eb85))

- **locale**: Block incomplete translation bundles (ENG-001.4) ([2e1567f](https://github.com/pacharanero/clincalc/commit/2e1567f50fd303670ca56e8f5aaa319cbb32feb4))

- **cli**: Add clincalc audit-references command (ENG-008.2) ([a697636](https://github.com/pacharanero/clincalc/commit/a697636bd99d9022870ae25c3224e950a66f24c4))

- **surfaces**: Report licence reverification when recorded ([5b9a05c](https://github.com/pacharanero/clincalc/commit/5b9a05c6ecb1806bf725b99f19125807775180ac))

- **license**: Add last_verified and verification_url (ENG-008.1) ([a11fa6c](https://github.com/pacharanero/clincalc/commit/a11fa6c52d550d77b5e37561111b04e20fbd1a2a))

- **cli**: Complete clipboard provenance (ENG-006.6) ([9174f24](https://github.com/pacharanero/clincalc/commit/9174f24c1617d0974ca33cde5ea0ca3fc6abee22))

### Changed

- **cli**: Deduplicate inputs/working line rendering ([5539d8c](https://github.com/pacharanero/clincalc/commit/5539d8c6b5f7ab947ec26d0f612074c6c428f092))

### Dependencies

- **deps**: Bump rustls 0.23.45 -> 0.23.46 ([4238b87](https://github.com/pacharanero/clincalc/commit/4238b87eae4aeed0a7109ffca85ddb2235931a4e))

- **deps**: Bump the routine-minor-and-patch group ([89fc5f5](https://github.com/pacharanero/clincalc/commit/89fc5f58844f6c266907cf9ca09503d392f1cd3a))

- **deps**: Bump tauri-plugin-opener in /gui/src-tauri ([87745f7](https://github.com/pacharanero/clincalc/commit/87745f7a8ee47495786fc105f3aeb06fba235d17))

- **deps**: Bump tauri in /gui/src-tauri ([4a224e1](https://github.com/pacharanero/clincalc/commit/4a224e1fda0d5ce39ff246026f26aed77e45c44d))

- **deps**: Bump tokio in / ([59cf26c](https://github.com/pacharanero/clincalc/commit/59cf26c6a54717a11d6bd70fbb8516a477ae1da4))

- **deps**: Bump zensical in / ([3a852dc](https://github.com/pacharanero/clincalc/commit/3a852dcf4e9ea79b2ec72c8ca13695932dccfe19))

- **deps**: Bump dtolnay/rust-toolchain ([a566a87](https://github.com/pacharanero/clincalc/commit/a566a871a9e5f20755851c75194dd3f2816866af))

- **deps**: Bump taiki-e/install-action ([304531b](https://github.com/pacharanero/clincalc/commit/304531b39cdb9b0519645192fe8e30e4cc099d41))

- **deps**: Bump tauri-plugin-opener in /gui/src-tauri ([838a115](https://github.com/pacharanero/clincalc/commit/838a115ea6c627d74235ca5c79211e6c03e35a0f))

- **deps**: Bump the routine-minor-and-patch group ([f288e3d](https://github.com/pacharanero/clincalc/commit/f288e3dc28eb913689c26c1143df74d9798923c0))

- **deps**: Bump tauri in /gui/src-tauri ([8161789](https://github.com/pacharanero/clincalc/commit/8161789276278ad48bb6ef72145d269d33214f67))

- **deps**: Bump pyo3 in /python ([#113](https://github.com/pacharanero/clincalc/issues/113)) ([4830c06](https://github.com/pacharanero/clincalc/commit/4830c065b4b217573b43acb4f911d94afab39ea1))

- **deps**: Bump taiki-e/install-action ([#106](https://github.com/pacharanero/clincalc/issues/106)) ([c3da0a4](https://github.com/pacharanero/clincalc/commit/c3da0a41e37775447b81e65794f432e9b557814e))

- **deps**: Bump dtolnay/rust-toolchain ([#107](https://github.com/pacharanero/clincalc/issues/107)) ([845865e](https://github.com/pacharanero/clincalc/commit/845865edc5fbc4e3df92a65889d8698deff06840))

- **deps**: Bump zensical in / ([ebfff30](https://github.com/pacharanero/clincalc/commit/ebfff3004d03be1530ec96a71cf0b00951fa50f2))

- **deps**: Bump the routine-minor-and-patch group ([#104](https://github.com/pacharanero/clincalc/issues/104)) ([6ed6201](https://github.com/pacharanero/clincalc/commit/6ed6201b7321cca179053907aaa7baacb2659355))

- **deps**: Bump tauri in /gui/src-tauri ([#103](https://github.com/pacharanero/clincalc/issues/103)) ([82f572d](https://github.com/pacharanero/clincalc/commit/82f572da4067288d7342e67a337fb67a6c81dcba))

- **deps**: Bump zensical in / ([#102](https://github.com/pacharanero/clincalc/issues/102)) ([340592b](https://github.com/pacharanero/clincalc/commit/340592b09ba410445a8568f59d50e92b18b080b9))

- **deps**: Bump taiki-e/install-action ([#101](https://github.com/pacharanero/clincalc/issues/101)) ([a002f94](https://github.com/pacharanero/clincalc/commit/a002f94b573184372dcfb22d7d0dc11d67d2b0ef))

- **deps**: Bump taiki-e/install-action ([#92](https://github.com/pacharanero/clincalc/issues/92)) ([3229b91](https://github.com/pacharanero/clincalc/commit/3229b91417d7e2ec7a2246c1be077fdd6324bb48))

- **deps**: Bump zensical in / ([#94](https://github.com/pacharanero/clincalc/issues/94)) ([293a907](https://github.com/pacharanero/clincalc/commit/293a90769fbea5b810e872d746c5b9dc7e56a143))

- **deps**: Bump clap in / ([#95](https://github.com/pacharanero/clincalc/issues/95)) ([b4f934c](https://github.com/pacharanero/clincalc/commit/b4f934c3117fccd4e47719d0d5a254210477b82d))

- **deps**: Bump clap_complete in / ([#96](https://github.com/pacharanero/clincalc/issues/96)) ([0bf0e4a](https://github.com/pacharanero/clincalc/commit/0bf0e4a061a1470f2c9dbe4d3f074e46f504811e))

- **deps**: Bump the routine-minor-and-patch group ([#97](https://github.com/pacharanero/clincalc/issues/97)) ([a47d4b5](https://github.com/pacharanero/clincalc/commit/a47d4b52348882c7c96f5a0a1610544b0a29b5fe))

- **deps**: Bump dtolnay/rust-toolchain ([#93](https://github.com/pacharanero/clincalc/issues/93)) ([96c11d2](https://github.com/pacharanero/clincalc/commit/96c11d27597f5dc8e85277c54c6be0b127f40a39))

- **deps**: Bump the routine-minor-and-patch group ([16e4c59](https://github.com/pacharanero/clincalc/commit/16e4c59a55f7f6ba7d299bd1c5f87e80a403c3d1))

- **deps**: Bump zensical in / ([6d9160d](https://github.com/pacharanero/clincalc/commit/6d9160d0186cdc6d09fc23dbbed64ebea38c706c))

- **deps**: Bump the github-actions-minor-and-patch group with 2 updates ([33ddf91](https://github.com/pacharanero/clincalc/commit/33ddf91a64c31690a01eb2369964968c9cac891f))

### Documentation

- **case-studies**: Add "how long will these restrictions actually last?" ([d7a71bd](https://github.com/pacharanero/clincalc/commit/d7a71bddda505a763771fd4b8bef50473c9e5c70))

- Add case study on the ADHD scoring-tool licensing gap ([20f5f5d](https://github.com/pacharanero/clincalc/commit/20f5f5d92c3cbac73b5f9e4c8b740f4a76282df2))

- **roadmap**: Track Vanderbilt/SNAP-IV/WURS as rights-review ADHD candidates ([2763952](https://github.com/pacharanero/clincalc/commit/276395256a68253e7a440c49b199e7664d54add4))

- **roadmap**: Add 20 calculator gaps found via MDCalc cross-reference (CALC-070..089) ([24e531f](https://github.com/pacharanero/clincalc/commit/24e531f613f5c3f3d6424a8e33a18e380e2e0f80))

- **roadmap**: Add QDiabetes-2018 and QCancer candidates ([e5693f7](https://github.com/pacharanero/clincalc/commit/e5693f7ffc549fe614276e1334e8242ef9d95687))

- **roadmap**: Add ENG-011/ENG-012 from reviewing Oli Evans's ods ([74b0c9d](https://github.com/pacharanero/clincalc/commit/74b0c9d147cb17b31b99999f99baccfa16b2a0c2))

- **roadmap**: Remove DOC-002, superseded by completed ENG-001.7 ([fe7ec9b](https://github.com/pacharanero/clincalc/commit/fe7ec9bfcbd34a2639a849f981af59a4db2a8fa0))

### Fixed

- **calc**: Update hardcoded registry counts for the new calculator ([dee255f](https://github.com/pacharanero/clincalc/commit/dee255f64e98eb3df51142be24f2dfa2d142c565))

- **audit**: Resolve broken intra-doc link in module docs ([648b265](https://github.com/pacharanero/clincalc/commit/648b2656536e9b09b9bc5715f774d5f8eac39198))

- **release**: Dispatch environment-bound workflow ([0efac74](https://github.com/pacharanero/clincalc/commit/0efac744a73406182337c00524fa996dae799a1a))

- **release**: Pass secrets to reusable workflow ([cbd2e23](https://github.com/pacharanero/clincalc/commit/cbd2e23cf96013666380b40c5a145bb165a474ed))

## [0.3.4] - 2026-09-13

### Fixed

- **release**: Accept cargo-dist checksum spacing ([4246b40](https://github.com/pacharanero/clincalc/commit/4246b4090388c00ea2d15bdbc5766a31367be404))

## [0.3.3] - 2026-09-13

### Fixed

- **release**: Align validation with shipped artifacts ([90d6d92](https://github.com/pacharanero/clincalc/commit/90d6d92cf717032a17e5d2bc575d56539e928671))

## [0.3.2] - 2026-09-13

### Fixed

- **release**: Unblock distribution without MSI ([7fc1926](https://github.com/pacharanero/clincalc/commit/7fc19268aff94e13bc3da862db0febf722cab139))

## [0.3.1] - 2026-09-13

### Added

- **calc**: Add Jackson-Pollock 3-site skinfold body fat % (CALC-048) ([#77](https://github.com/pacharanero/clincalc/issues/77)) ([aee3d11](https://github.com/pacharanero/clincalc/commit/aee3d11eea74160a9dbd30aaefad17494fdb2f30))

- **calc**: Add resistance-training protein intake ([#76](https://github.com/pacharanero/clincalc/issues/76)) ([1fc8375](https://github.com/pacharanero/clincalc/commit/1fc83756db1d49071f987ef644bbb2dd7062aaad))

- **calc**: Add Skeletal Muscle Mass Index (CALC-052) ([#75](https://github.com/pacharanero/clincalc/issues/75)) ([b4dd457](https://github.com/pacharanero/clincalc/commit/b4dd4574c1bfcc29a80a951961c41856661fbdf9))

- **calc**: Add Fat-Free Mass Index (CALC-051) ([#73](https://github.com/pacharanero/clincalc/issues/73)) ([28730d2](https://github.com/pacharanero/clincalc/commit/28730d2775d407a2e4f5e7c8b13bf61146f475b5))

- **cli**: Add markdown output format for computed results ([#70](https://github.com/pacharanero/clincalc/issues/70)) ([5110e7d](https://github.com/pacharanero/clincalc/commit/5110e7d5a15db96cc01102ca4a17d051a02c9529))

- **calc**: Add Body Adiposity Index (CALC-049) ([#69](https://github.com/pacharanero/clincalc/issues/69)) ([e921917](https://github.com/pacharanero/clincalc/commit/e921917a1322a54960a0f22334753bb45e526b0e))

- **calc**: Add NYHA functional classification ([4c78971](https://github.com/pacharanero/clincalc/commit/4c78971e3c95492276c16a4dce7c6befcac3406d))

- **calc**: Add ORBIT bleeding score ([0624d96](https://github.com/pacharanero/clincalc/commit/0624d961764e5497457d65812a4a5c71b3945e85))

- **calc**: Register unavailable parity gaps ([0c42354](https://github.com/pacharanero/clincalc/commit/0c42354243da14d2dd6080d37162c7091a0e7cc8))

- **calc**: Add ARDSNet predicted body weight ([86c48b8](https://github.com/pacharanero/clincalc/commit/86c48b83145e381bbafdf39342d30169fb48c8c3))

- **calc**: Register NYHA as unavailable ([f9c4a6c](https://github.com/pacharanero/clincalc/commit/f9c4a6c592f7bbe7c1d86f0452b46b9826e2867c))

- **calc**: Add Duke-ISCVID criteria ([0dfcf93](https://github.com/pacharanero/clincalc/commit/0dfcf93dec3f864df384da62e6512537dae89bf9))

- **calc**: Add NIH Stroke Scale ([e4e7de2](https://github.com/pacharanero/clincalc/commit/e4e7de2488d4f42501fb1498354d6150605d2251))

- **calc**: Add PASI and Pitt Bacteraemia ([5d411f2](https://github.com/pacharanero/clincalc/commit/5d411f233501ae5ae5db745938d85ef0546878d1))

- **calc**: Add MELD 3.0 and PSA density ([0a6c1ab](https://github.com/pacharanero/clincalc/commit/0a6c1ab5a63fef57ceffceb90289a7576e5c951b))

- **calc**: Add CIWA-Ar and COWS ([b4eb6b8](https://github.com/pacharanero/clincalc/commit/b4eb6b8eb84cb0792f919d2b2914020673cebcb9))

- **calc**: Add free-water deficit and ISTH DIC ([58a7c55](https://github.com/pacharanero/clincalc/commit/58a7c5528fa438327cc3c06275aed13252eb5659))

- **calc**: Add 4Ts and Khorana calculators ([e72e0d4](https://github.com/pacharanero/clincalc/commit/e72e0d4a390be6b1846416c1a61632252de70112))

- **calc**: Add Apgar and Binet calculators ([70cc03a](https://github.com/pacharanero/clincalc/commit/70cc03a30852ca717312a5ac266b9d40a7c1879b))

- **calc**: Add BSA and HOMA-IR calculators ([0aaa094](https://github.com/pacharanero/clincalc/commit/0aaa094fdeadf8d0b8e74e2ad770c1d61453b8de))

- **calc**: Add LDL and non-HDL cholesterol (CALC-041) ([4079b6e](https://github.com/pacharanero/clincalc/commit/4079b6e22b6c04d11c010ac76ccc3ae81a6db3bc))

- **api**: Add REST locale negotiation (ENG-001.6) ([6dca687](https://github.com/pacharanero/clincalc/commit/6dca687387f6407da75fefa3fe3554cf5c997aef))

- **calc**: Add modified Hinchey classification (CALC-019) ([4353168](https://github.com/pacharanero/clincalc/commit/43531687f29d003b986d443c8579f63c310e7eb7))

- **calc**: Add Glasgow-Blatchford bleeding score (CALC-018) ([03c138d](https://github.com/pacharanero/clincalc/commit/03c138d42bd0c9e7716ce8df920384ea62e034ef))

- **calc**: Add RCRI perioperative cardiac risk score (CALC-033) ([51df1c2](https://github.com/pacharanero/clincalc/commit/51df1c211f230fb5af13798399c6901c9344afbb))

- **calc**: Add LRINEC score for necrotizing fasciitis risk (CALC-021) ([#63](https://github.com/pacharanero/clincalc/issues/63)) ([734f0ac](https://github.com/pacharanero/clincalc/commit/734f0acd712e5f45e901881be813c44c3888c418))

- **calc**: Add Wilks score for bodyweight-adjusted strength comparison (CALC-046) ([#62](https://github.com/pacharanero/clincalc/issues/62)) ([13ea365](https://github.com/pacharanero/clincalc/commit/13ea365fd76d86e7843d215a8898ce7ea7979532))

- **calc**: Add relative fat mass (RFM) (CALC-050) ([#61](https://github.com/pacharanero/clincalc/issues/61)) ([c4b1e91](https://github.com/pacharanero/clincalc/commit/c4b1e91cbadf70d178523e3e37b6602804005ce4))

- **calc**: Add waist-to-hip ratio (WHR) (CALC-047) ([#60](https://github.com/pacharanero/clincalc/issues/60)) ([00f90cb](https://github.com/pacharanero/clincalc/commit/00f90cb4dbe29a418238b972fb956ef274d5be60))

- **calc**: Add PERC rule for PE exclusion (CALC-029) ([#55](https://github.com/pacharanero/clincalc/issues/55)) ([694914f](https://github.com/pacharanero/clincalc/commit/694914f842e987bd69ee95487e4a3c01a46f61c7))

- **python**: Add keyword-only locale to Python bindings (ENG-001.6) ([#54](https://github.com/pacharanero/clincalc/issues/54)) ([36cd109](https://github.com/pacharanero/clincalc/commit/36cd10915a8e4e13ec2468392be28041eb767351))

- **calc**: Add hyperglycaemia-corrected sodium (CALC-020) ([#46](https://github.com/pacharanero/clincalc/issues/46)) ([83b8b41](https://github.com/pacharanero/clincalc/commit/83b8b4199c4401dd968eddfb425001c8fee4471a))

### Dependencies

- **deps**: Bump the routine-minor-and-patch group ([#71](https://github.com/pacharanero/clincalc/issues/71)) ([56a7582](https://github.com/pacharanero/clincalc/commit/56a7582d7a2da0f7b02ecbfefef709264a864aec))

- **deps**: Bump tauri-plugin-opener in /gui/src-tauri ([#72](https://github.com/pacharanero/clincalc/issues/72)) ([b34b6ad](https://github.com/pacharanero/clincalc/commit/b34b6addfeee91789b23754c1423382160d1469c))

- **deps**: Bump taiki-e/install-action from 2.86.3 to 2.86.7 in the github-actions-minor-and-patch group ([#67](https://github.com/pacharanero/clincalc/issues/67)) ([18e466a](https://github.com/pacharanero/clincalc/commit/18e466ad6c2d87d6483630109e57b4c8bd5b08be))

- **deps**: Bump the routine-minor-and-patch group ([#68](https://github.com/pacharanero/clincalc/issues/68)) ([0192664](https://github.com/pacharanero/clincalc/commit/0192664b55d1d714301c811b6025a6f5805b4a67))

- **deps**: Bump zensical from 0.0.55 to 0.0.57 ([#57](https://github.com/pacharanero/clincalc/issues/57)) ([e6c6b51](https://github.com/pacharanero/clincalc/commit/e6c6b51dea1e0d3d55354820651351fb7a6f712f))

- **deps**: Upgrade Mantine packages to 9.5.1 ([#50](https://github.com/pacharanero/clincalc/issues/50)) ([05ab1fe](https://github.com/pacharanero/clincalc/commit/05ab1fef9dadf0fff79ad4060eb81105e34c1525))

- **deps**: Bump the routine-minor-and-patch group ([#48](https://github.com/pacharanero/clincalc/issues/48)) ([d954f3b](https://github.com/pacharanero/clincalc/commit/d954f3b5c68cf909b4f2deb215875a58594d1b72))

- **deps**: Bump zensical in / ([#47](https://github.com/pacharanero/clincalc/issues/47)) ([760358a](https://github.com/pacharanero/clincalc/commit/760358a053d01d1e81659821811f0e340b3a0083))

### Documentation

- Clarify development-only positioning ([dde2d90](https://github.com/pacharanero/clincalc/commit/dde2d90902c49ccd720562c086df672242e414d5))

- **roadmap**: Track MedikQuantis parity ([0468eb0](https://github.com/pacharanero/clincalc/commit/0468eb008e4f428b2894c765e8e2e996a177bf63))

- Add contributing-a-translation guide ([#53](https://github.com/pacharanero/clincalc/issues/53)) ([7292305](https://github.com/pacharanero/clincalc/commit/7292305a0273c5527854f51923b7351be784a507))

### Fixed

- **calc**: Pin CHA2DS2-VASc S2 systemic-embolism boundary ([#74](https://github.com/pacharanero/clincalc/issues/74)) ([d24b5cb](https://github.com/pacharanero/clincalc/commit/d24b5cbdff601092571389e9e270edc0ad50a7e6))

- **api**: Correct locale negotiation semantics ([52749e9](https://github.com/pacharanero/clincalc/commit/52749e9a14089950d6279e60fa50956ddb231a01))

- **api**: Satisfy clippy::unnecessary_sort_by on CI's newer toolchain ([d1c12f4](https://github.com/pacharanero/clincalc/commit/d1c12f483d4be1d014c16ec009d9e40c0c205814))

## [0.3.0] - 2026-08-20

### Added

- **calc**: Converge CHA2DS2-VASc scoring with MedikQuantis (COLL-001) ([#30](https://github.com/pacharanero/clincalc/issues/30)) ([d2b6041](https://github.com/pacharanero/clincalc/commit/d2b604119b8a2b8e915b39ba44e643801d6e8ce4))

- **locale**: Add BCP 47 locale support and CURB-65 translations ([3a01ba9](https://github.com/pacharanero/clincalc/commit/3a01ba9475b685d570818a0c386511b98659e820))

- **calc**: Add CHA2DS2-VASc event rates ([238cb6c](https://github.com/pacharanero/clincalc/commit/238cb6c4804fb35e098a7fe1d96f70d856ee52ab))

- **calc**: Add GCS (Glasgow Coma Scale) ([#26](https://github.com/pacharanero/clincalc/issues/26)) ([a99f26a](https://github.com/pacharanero/clincalc/commit/a99f26a08805c08a02cc76717dcd385a9b33cbe1))

### Dependencies

- **deps**: Resolve GUI unsoundness audit ([c36feff](https://github.com/pacharanero/clincalc/commit/c36feff447046e5b497490e010df725d9b1cc80c))

- **deps**: Update vulnerable frontend transitive dependencies ([aad9fba](https://github.com/pacharanero/clincalc/commit/aad9fbad96268f498141dd60fe1bf878abf062ea))

- **deps**: Bump taiki-e/install-action from 2.85.10 to 2.85.13 ([#45](https://github.com/pacharanero/clincalc/issues/45)) ([5e5fd13](https://github.com/pacharanero/clincalc/commit/5e5fd13aebf3a58703d85856fc71bb08b6402cb4))

- **deps**: Bump dtolnay/rust-toolchain ([#38](https://github.com/pacharanero/clincalc/issues/38)) ([7f5aad8](https://github.com/pacharanero/clincalc/commit/7f5aad857e117d231e963de8e42a26f263e7ff84))

- **deps**: Bump Swatinem/rust-cache from 2.9.1 to 2.9.2 ([#40](https://github.com/pacharanero/clincalc/issues/40)) ([bb2e43d](https://github.com/pacharanero/clincalc/commit/bb2e43d990044759ef51a14efc09e8ac6e93ee23))

- **deps**: Bump clap_complete from 4.6.8 to 4.6.9 ([#41](https://github.com/pacharanero/clincalc/issues/41)) ([0d3ed2a](https://github.com/pacharanero/clincalc/commit/0d3ed2acdff91cfb5fadffa087a7afb06f76a73d))

- **deps**: Bump clap from 4.6.5 to 4.6.6 ([#42](https://github.com/pacharanero/clincalc/issues/42)) ([9a4213d](https://github.com/pacharanero/clincalc/commit/9a4213d0e53c270cb76977edc09de123e86979bf))

- **deps**: Bump taiki-e/install-action from 2.85.5 to 2.85.10 ([#43](https://github.com/pacharanero/clincalc/issues/43)) ([9833361](https://github.com/pacharanero/clincalc/commit/9833361f0112dd0e3dd806bba55edb8f222d8379))

- **deps**: Bump clap from 4.6.4 to 4.6.5 ([#37](https://github.com/pacharanero/clincalc/issues/37)) ([46db830](https://github.com/pacharanero/clincalc/commit/46db8305acd1093628ec929d3aa219ad4dcc845d))

- **deps**: Bump pypa/gh-action-pypi-publish from 1.14.1 to 1.14.2 ([#34](https://github.com/pacharanero/clincalc/issues/34)) ([ca94a08](https://github.com/pacharanero/clincalc/commit/ca94a08315f0ab9c5e73545416d5515f78a58122))

- **deps**: Bump taiki-e/install-action from 2.85.2 to 2.85.5 ([#35](https://github.com/pacharanero/clincalc/issues/35)) ([05c4f5a](https://github.com/pacharanero/clincalc/commit/05c4f5acc8d87aeb4a37a75835d02a26ae34a5b9))

- **deps**: Bump clap_complete from 4.6.7 to 4.6.8 ([#31](https://github.com/pacharanero/clincalc/issues/31)) ([ee1ae45](https://github.com/pacharanero/clincalc/commit/ee1ae45cbc14b61c5e1731ceab4fa5241a2319af))

- **deps**: Bump taiki-e/install-action from 2.83.4 to 2.85.2 ([#29](https://github.com/pacharanero/clincalc/issues/29)) ([238523e](https://github.com/pacharanero/clincalc/commit/238523e9c3f99524a7a0846ae01e356507293350))

- **deps**: Bump PyO3/maturin-action ([#28](https://github.com/pacharanero/clincalc/issues/28)) ([096f2b2](https://github.com/pacharanero/clincalc/commit/096f2b28fcc7cf4dd33aa1ecc42fd5dece9ea167))

- **deps**: Bump actions/checkout from 7.0.0 to 7.0.1 ([#27](https://github.com/pacharanero/clincalc/issues/27)) ([d9e5b16](https://github.com/pacharanero/clincalc/commit/d9e5b1692e24be94662e0573ddac90978f6afd2f))

- **deps**: Bump actions/setup-python from 6.3.0 to 7.0.0 ([#21](https://github.com/pacharanero/clincalc/issues/21)) ([89f10bb](https://github.com/pacharanero/clincalc/commit/89f10bb45358f6d487b73f8310a7585f36929a7d))

- **deps**: Bump dtolnay/rust-toolchain ([#16](https://github.com/pacharanero/clincalc/issues/16)) ([7eccbfd](https://github.com/pacharanero/clincalc/commit/7eccbfdb599ddc0c205afd222b4b5d90fdb46805))

- **deps**: Bump taiki-e/install-action from 2.82.7 to 2.83.4 ([#20](https://github.com/pacharanero/clincalc/issues/20)) ([6432942](https://github.com/pacharanero/clincalc/commit/643294212ca90c6439bfd0d9e34bc5a181112d01))

- **deps**: Bump clap from 4.6.3 to 4.6.4 ([#25](https://github.com/pacharanero/clincalc/issues/25)) ([387454c](https://github.com/pacharanero/clincalc/commit/387454c1800b16262c5a6a86422ed386505a85b3))

### Documentation

- **roadmap**: Plan centile lookup support ([0178e90](https://github.com/pacharanero/clincalc/commit/0178e900152dbec9cfe00ad597e1ee814ac95078))

- Update published install and Python guidance ([b7803cf](https://github.com/pacharanero/clincalc/commit/b7803cf69fc35249b08489b9763057b661197c5d))

### Fixed

- **licensing**: Preserve ClinRisk distribution terms ([f253129](https://github.com/pacharanero/clincalc/commit/f253129303cf252ced715185292c57edb08e07c0))

- **python**: Preserve structured batch inputs ([4ab8099](https://github.com/pacharanero/clincalc/commit/4ab80995ff85950cbb53693de24c080815e0f43d))

- **curb65**: Enforce adult reviewed content ([6268d4a](https://github.com/pacharanero/clincalc/commit/6268d4a00254950818a648e5d498149d6bd99e79))

- **asrs**: Qualify eligibility and scoring methods ([422794e](https://github.com/pacharanero/clincalc/commit/422794eec45842d5c58fe4ac7db346c2f4a37143))

- **inputs**: Enforce closed calculator schemas ([edc9a6c](https://github.com/pacharanero/clincalc/commit/edc9a6c38d35ebab0ac04b0c2376a55f7e672f78))

- **release**: Sync MSI package description ([ce2b205](https://github.com/pacharanero/clincalc/commit/ce2b205ae2e9b06669fc9d0334cacaebbdbb085e))

- **asrs**: Restore permitted six-question scorer ([9cd4c69](https://github.com/pacharanero/clincalc/commit/9cd4c6943a31b9a00ba5ef07fc880d346ac1feb2))

- **locale**: Keep unsupported representations in English ([f58a967](https://github.com/pacharanero/clincalc/commit/f58a96732e643e1c6e212eb9d3125b44ef767613))

- **api**: Return structured JSON request errors ([fd6651a](https://github.com/pacharanero/clincalc/commit/fd6651aa5bb1a894ddcb419be6d1a13244f3c6f8))

- **curb65**: Correct mortality and care guidance ([379a327](https://github.com/pacharanero/clincalc/commit/379a327f7bed94ea15290c3e3dabb17cdfd73000))

- **release**: Ship legal notices in distributions ([f4d0c25](https://github.com/pacharanero/clincalc/commit/f4d0c25bbddb33c416297975f392f18c0f1f3ed7))

- **licensing**: Remove restricted ASRS content ([cffd2fe](https://github.com/pacharanero/clincalc/commit/cffd2fe2154f886149de094475adb36480d77605))

## [0.2.2] - 2026-07-21

### Added

- **calc**: Add anthropometric and training calculators ([ea4969b](https://github.com/pacharanero/clincalc/commit/ea4969bbf62d51a69630c8bfcd884bdb487761c8))

- **python**: Ship PyO3 bindings, PyPI CI, and pandas batch helpers ([b2bb680](https://github.com/pacharanero/clincalc/commit/b2bb6805519eac9e19a5b79c292b9706fdb1e0bd))

- **api**: Add GET /openapi.json - OpenAPI 3.1 spec auto-generated from registry ([c26504e](https://github.com/pacharanero/clincalc/commit/c26504eae5251f3471b5f404ced9e6e7a8a1331e))

- **api**: Add REST API surface behind rest-api feature (on by default) ([7dbaa08](https://github.com/pacharanero/clincalc/commit/7dbaa08aaf25b6f8d6727f6b002333c749a129dd))

- **calc**: Add 10 calculators (CALC-009/010/011/012/013/014/015/016/039/040) ([eb35871](https://github.com/pacharanero/clincalc/commit/eb35871f723559105be9fd296cd3408627dfb15c))

- **calc**: Add Cockcroft-Gault creatinine clearance (CALC-047) ([a0f271b](https://github.com/pacharanero/clincalc/commit/a0f271bb6698db502d6b20d94ba4a8a662b8663b))

- **cli**: Add human input and profile conveniences ([7046b15](https://github.com/pacharanero/clincalc/commit/7046b15e1852e962ffd8d913294530d12ac7363d))

- **cli**: Add calculator aliases and energy presets ([77d4108](https://github.com/pacharanero/clincalc/commit/77d4108dc6f4883ca33b60d70429a139bc27a4e8))

- **cli**: Add command structure and completions surface ([628a4df](https://github.com/pacharanero/clincalc/commit/628a4df322f11eed2a93b8f5ab1509a8b1625d5c))

- **calc**: Add energy requirement equations ([5990395](https://github.com/pacharanero/clincalc/commit/5990395604bc17e3972ae598d3460e76354bd2fb))

- **gui**: Add priority calculators and simplify shell ([a2231d3](https://github.com/pacharanero/clincalc/commit/a2231d3b42b936a294433abf3c0874b64d0efba1))

### Documentation

- **roadmap**: Checkbox-format all ENG implementation steps ([02eb48f](https://github.com/pacharanero/clincalc/commit/02eb48fcf9637f42da6d9531d5bc2cb880ff2e35))

- **roadmap**: Checkbox-format COLL-001/002 implementation steps ([2d38465](https://github.com/pacharanero/clincalc/commit/2d3846551402b31076a6ea1954280303100fb6dd))

- **roadmap**: Add COLL-001/002 MedikQuantis collaboration plan ([853c0ce](https://github.com/pacharanero/clincalc/commit/853c0ce582e0309bf87b9f4091f7306d7ba55ee3))

- **roadmap**: Clear completed API/Python items and expand ENG backlog ([a1950b6](https://github.com/pacharanero/clincalc/commit/a1950b6db2ac567a4f9087280a3bffad7dce9f77))

- Mark API-001 done, close API-003 in favour of reverse proxy ([5a430b8](https://github.com/pacharanero/clincalc/commit/5a430b85777cd29683a56562bb1c5fd7e4d10ec5))

- Mark API-001 done, close API-003 in favour of reverse proxy ([99d1676](https://github.com/pacharanero/clincalc/commit/99d167683581a1a5544ce43c7ba90539b81a76fe))

- Document all planned surfaces (REST API, Python FFI, GUI, web) in README and roadmap ([b67623b](https://github.com/pacharanero/clincalc/commit/b67623bbcbec96252c631fb692c60cf6bd483e12))

- Complete calculator authoring docs items ([e652fbf](https://github.com/pacharanero/clincalc/commit/e652fbfbeb8317b06c6bd5cdd83822aaaddebabc))

- Add roadmap item identifiers ([4a6692e](https://github.com/pacharanero/clincalc/commit/4a6692e22620ddac1c3a3aba7087737c8eb40b3f))

- Check off completed roadmap items ([3f96e38](https://github.com/pacharanero/clincalc/commit/3f96e3879de1bd7ee1b46599c3cdccd69e09316d))

- **gui**: Document debugging and Linux diagnostics ([c5495d9](https://github.com/pacharanero/clincalc/commit/c5495d9006b38a35a378e38d48fef3297a412b0d))

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


