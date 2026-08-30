// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Registry entries for calculators that deliberately return no score.
//!
//! Some clinical tools cannot be shipped openly because their algorithm is a
//! trade secret (e.g. FRAX) or their content is copyrighted and licence-locked
//! (e.g. the MMSE). This project refuses to ship a half-right reimplementation
//! or to quietly omit them. Separate entries also represent tools awaiting
//! rights review and tools excluded because returning a score would create a
//! foreseeable clinical-safety risk. Each is registered as a first-class
//! calculator whose computation returns its exact reason and alternatives.
//!
//! The point is transparency, not obstruction: a clinician searching for FRAX
//! finds out exactly why it is not here and where to turn, rather than silence.

use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Shared advice appended to every proprietary calculator's response.
pub const ADVOCACY: &str = "Clinical decision tools that public healthcare relies on should be open \
and free to use. If you agree, consider writing to your MP or elected representative to ask why \
tools essential to patient care are locked behind proprietary licences, and to support open \
clinical knowledge. Open alternatives are listed above where they exist.";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NoInput {}

/// A calculator that cannot be shipped because it is proprietary or
/// licence-locked. Computing it returns the reason, the owner, alternatives,
/// and advocacy advice rather than a score.
#[derive(Debug, Clone, Copy)]
pub struct ProprietaryCalculator {
    pub name: &'static str,
    pub title: &'static str,
    /// One-line description of what the tool does.
    pub purpose: &'static str,
    /// Who owns / controls the rights.
    pub owner: &'static str,
    /// Why it cannot be shipped (trade-secret algorithm, copyright, etc.).
    pub reason: &'static str,
    /// Open alternatives a clinician can use instead (machine names of
    /// calculators shipped here where one exists, or named external tools).
    pub alternatives: &'static [&'static str],
    /// A URL with more information (typically the owner's page).
    pub source_url: &'static str,
}

/// A named tool that is deliberately unavailable because exposing its score
/// would create a foreseeable clinical-safety risk.
#[derive(Debug, Clone, Copy)]
pub struct SafetyUnavailableCalculator {
    pub name: &'static str,
    pub title: &'static str,
    /// One-line description of what the tool does.
    pub purpose: &'static str,
    /// Why exposing the tool would be unsafe.
    pub reason: &'static str,
    /// Safer approaches clinicians should use instead.
    pub alternatives: &'static [&'static str],
    /// URLs supporting the safety decision.
    pub evidence_urls: &'static [&'static str],
    /// A URL supporting the safety decision.
    pub source_url: &'static str,
}

/// A named tool withheld while unrestricted implementation and redistribution
/// rights remain unresolved, without claiming that the algorithm is proprietary.
#[derive(Debug, Clone, Copy)]
pub struct RightsReviewUnavailableCalculator {
    pub name: &'static str,
    pub title: &'static str,
    pub purpose: &'static str,
    pub rights_holder_or_contact: &'static str,
    pub reason: &'static str,
    pub alternatives: &'static [&'static str],
    pub source_url: &'static str,
}

impl Calculator for ProprietaryCalculator {
    fn name(&self) -> &'static str {
        self.name
    }

    fn title(&self) -> &'static str {
        self.title
    }

    fn description(&self) -> &'static str {
        self.purpose
    }

    fn reference(&self) -> &'static str {
        self.source_url
    }

    fn license(&self) -> CalculatorLicense {
        CalculatorLicense {
            license: "Proprietary / licence-locked - not freely distributable",
            source_url: self.source_url,
        }
    }

    fn input_schema(&self) -> Value {
        // No inputs: computing it only ever returns the explanation.
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": self.title,
            "type": "object",
            "additionalProperties": false,
            "properties": {},
            "description": "Proprietary calculator: takes no inputs and returns an explanation of why it cannot be shipped."
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        if !input.is_object() {
            return Err(CalcError::InvalidInput("expected an object".into()));
        }
        let _: NoInput = serde_json::from_value(input.clone())
            .map_err(|error| CalcError::InvalidInput(error.to_string()))?;

        let mut working = Map::new();
        working.insert("status".into(), json!("unavailable-proprietary"));
        working.insert("owner".into(), json!(self.owner));
        working.insert("reason".into(), json!(self.reason));
        working.insert("alternatives".into(), json!(self.alternatives));
        working.insert("what_you_can_do".into(), json!(ADVOCACY));

        let interpretation = format!(
            "{title} is not available here because it is proprietary or licence-locked. \
Owner: {owner}. {reason} {advocacy}",
            title = self.title,
            owner = self.owner,
            reason = self.reason,
            advocacy = ADVOCACY
        );

        Ok(CalculationResponse {
            calculator: self.name.to_string(),
            result: json!("unavailable: proprietary"),
            interpretation,
            working,
            reference: self.source_url.to_string(),
        })
    }
}

impl Calculator for SafetyUnavailableCalculator {
    fn name(&self) -> &'static str {
        self.name
    }

    fn title(&self) -> &'static str {
        self.title
    }

    fn description(&self) -> &'static str {
        self.purpose
    }

    fn reference(&self) -> &'static str {
        self.source_url
    }

    fn license(&self) -> CalculatorLicense {
        CalculatorLicense {
            license: "Unavailable - clinical-safety exclusion",
            source_url: self.source_url,
        }
    }

    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": self.title,
            "type": "object",
            "additionalProperties": false,
            "properties": {},
            "description": "Clinically unsafe scoring tool: takes no inputs and returns an explanation of why no score is provided."
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        if !input.is_object() {
            return Err(CalcError::InvalidInput("expected an object".into()));
        }
        let _: NoInput = serde_json::from_value(input.clone())
            .map_err(|error| CalcError::InvalidInput(error.to_string()))?;

        let mut working = Map::new();
        working.insert("status".into(), json!("unavailable-clinical-safety"));
        working.insert("reason".into(), json!(self.reason));
        working.insert("alternatives".into(), json!(self.alternatives));
        working.insert("evidence_urls".into(), json!(self.evidence_urls));

        let interpretation = format!(
            "{title} is unavailable for clinical-safety reasons and returns no score. {reason} \
Use {alternatives}. This entry must not be used to reduce observation, referral, treatment, or \
admission/discharge decisions.",
            title = self.title,
            reason = self.reason,
            alternatives = self.alternatives.join("; ")
        );

        Ok(CalculationResponse {
            calculator: self.name.to_string(),
            result: json!("unavailable: clinical-safety"),
            interpretation,
            working,
            reference: self.source_url.to_string(),
        })
    }
}

impl Calculator for RightsReviewUnavailableCalculator {
    fn name(&self) -> &'static str {
        self.name
    }

    fn title(&self) -> &'static str {
        self.title
    }

    fn description(&self) -> &'static str {
        self.purpose
    }

    fn reference(&self) -> &'static str {
        self.source_url
    }

    fn license(&self) -> CalculatorLicense {
        CalculatorLicense {
            license: "Unavailable - redistribution rights unresolved",
            source_url: self.source_url,
        }
    }

    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": self.title,
            "type": "object",
            "additionalProperties": false,
            "properties": {},
            "description": "Rights-review entry: takes no inputs and explains why no implementation is distributed."
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        if !input.is_object() {
            return Err(CalcError::InvalidInput("expected an object".into()));
        }
        let _: NoInput = serde_json::from_value(input.clone())
            .map_err(|error| CalcError::InvalidInput(error.to_string()))?;

        let mut working = Map::new();
        working.insert("status".into(), json!("unavailable-rights-review"));
        working.insert(
            "rights_holder_or_contact".into(),
            json!(self.rights_holder_or_contact),
        );
        working.insert("reason".into(), json!(self.reason));
        working.insert("alternatives".into(), json!(self.alternatives));

        Ok(CalculationResponse {
            calculator: self.name.to_string(),
            result: json!("unavailable: rights-review"),
            interpretation: format!(
                "{title} is not implemented while unrestricted software redistribution rights remain unresolved. Contact: {contact}. {reason} No score is returned.",
                title = self.title,
                contact = self.rights_holder_or_contact,
                reason = self.reason,
            ),
            working,
            reference: self.source_url.to_string(),
        })
    }
}

/// The proprietary / licence-locked tools, surfaced so clinicians learn why
/// they are absent and where to turn.
pub const PROPRIETARY: &[ProprietaryCalculator] = &[
    ProprietaryCalculator {
        name: "frax",
        title: "FRAX (10-year fracture risk)",
        purpose: "10-year probability of osteoporotic and hip fracture (NICE CG146).",
        owner: "University of Sheffield (Centre for Metabolic Bone Diseases)",
        reason: "The FRAX algorithm and its country-specific coefficients are a trade secret and \
have never been published, so it cannot be reimplemented from primary literature.",
        alternatives: &[
            "qfracture (open UK fracture-risk algorithm)",
            "Garvan Fracture Risk Calculator",
        ],
        source_url: "https://frax.shef.ac.uk/",
    },
    ProprietaryCalculator {
        name: "mmse",
        title: "MMSE (Mini-Mental State Examination)",
        purpose: "Cognitive screening / dementia monitoring (NICE NG97).",
        owner: "Psychological Assessment Resources, Inc. (PAR)",
        reason: "The MMSE has been copyright-controlled by PAR since 2001; reproducing or \
distributing the instrument requires a paid licence.",
        alternatives: &[
            "amts (Abbreviated Mental Test Score - public domain, shipped here)",
            "MoCA (Montreal Cognitive Assessment)",
            "6CIT, GPCOG (free cognitive screens)",
        ],
        source_url: "https://www.parinc.com/products/MMSE",
    },
    ProprietaryCalculator {
        name: "elf",
        title: "ELF (Enhanced Liver Fibrosis test)",
        purpose: "Second-line serum biomarker test for liver fibrosis (NICE NG49).",
        owner: "Siemens Healthineers",
        reason: "The commercial ELF score uses a proprietary, recalibrated algorithm over its \
serum biomarkers; the shipped score cannot be reproduced openly.",
        alternatives: &[
            "fib4 (FIB-4 index - first-line, shipped here)",
            "NAFLD Fibrosis Score",
            "Transient elastography (FibroScan)",
        ],
        source_url: "https://www.siemens-healthineers.com/laboratory-diagnostics/assays-by-diseases-conditions/liver-disease/elf-test",
    },
    ProprietaryCalculator {
        name: "cfs",
        title: "CFS (Clinical Frailty Scale)",
        purpose: "9-point judgement-based frailty grading in older adults (1 Very Fit to 9 Terminally Ill).",
        owner: "Dalhousie University (Geriatric Medicine Research; Kenneth Rockwood et al.)",
        reason: "The CFS is copyrighted by Dalhousie University. Non-commercial use is free but \
requires a signed permission agreement that forbids changing or commercialising the scale, and an \
EMR vendor incorporating it into its offering needs a licence. A no-modification, signed-permission \
agreement is incompatible with shipping the content under this project's open AGPL licence.",
        alternatives: &[
            "Electronic Frailty Index (eFI) - open, derived from routine primary-care EHR data",
            "PRISMA-7 (7-item frailty screen)",
            "Edmonton Frail Scale",
        ],
        source_url: "https://www.dal.ca/sites/gmr/our-tools/clinical-frailty-scale.html",
    },
    ProprietaryCalculator {
        name: "lanss",
        title: "LANSS (Leeds Assessment of Neuropathic Symptoms and Signs)",
        purpose: "Screening for pain of predominantly neuropathic origin (7 items, 0-24; >=12 likely neuropathic).",
        owner: "Michael I. Bennett; published in Elsevier's journal Pain (2001), all rights reserved",
        reason: "The LANSS instrument is copyrighted and is reproduced in the literature only with \
the permission of M. Bennett. There is no public-domain or free-reuse grant, so embedding the \
scored instrument in software requires permission from the author/publisher.",
        alternatives: &[
            "DN4 (Douleur Neuropathique 4 - check licensing)",
            "painDETECT (check licensing)",
            "Clinical neuropathic pain assessment per NICE CG173",
        ],
        source_url: "https://doi.org/10.1016/S0304-3959(00)00482-6",
    },
    ProprietaryCalculator {
        name: "must",
        title: "MUST (Malnutrition Universal Screening Tool)",
        purpose: "Malnutrition risk screening (NICE CG32).",
        owner: "BAPEN (British Association for Parenteral and Enteral Nutrition)",
        reason: "BAPEN holds copyright in the MUST tool. The download is free only to individual \
professionals using it; reproducing MUST (which shipping an implementation does) requires \
applying to BAPEN for permission under a renewable, audited licence agreement, so it cannot be \
shipped under an open licence.",
        alternatives: &[
            "Clinical assessment per NICE CG32 (BMI, unplanned weight loss, acute disease effect)",
            "MNA (Mini Nutritional Assessment - check licensing)",
        ],
        source_url: "https://www.bapen.org.uk/must-and-self-screening/reproducing-must-application-form/",
    },
    ProprietaryCalculator {
        name: "cat",
        title: "CAT (COPD Assessment Test)",
        purpose: "Symptom-burden / health-status measure in COPD (8 items, 0-40; GOLD/NICE NG115).",
        owner: "GSK group of companies (CAT Governance Board: GSK / GOLD / COPD Foundation)",
        reason: "The CAT is trademarked and copyrighted by GSK. Free permissions cover only \
reproduction of the published instrument by researchers and explicitly forbid incorporating it \
into any other electronic system or means of data capture; embedding it in software requires a \
signed licence agreement with GSK.",
        alternatives: &[
            "mrc_dyspnoea (MRC dyspnoea scale - breathlessness grading, shipped here)",
            "Clinical COPD assessment per GOLD / NICE NG115",
        ],
        source_url: "https://www.catestonline.org/hcp-homepage/legal-notices.html",
    },
    ProprietaryCalculator {
        name: "acq",
        title: "ACQ (Asthma Control Questionnaire)",
        purpose: "Asthma control monitoring (BTS/NICE/SIGN asthma).",
        owner: "Elizabeth Juniper / QOL Technologies Ltd",
        reason: "The ACQ is copyrighted and its use and reproduction require a licence from the \
copyright holder.",
        alternatives: &[
            "Asthma Control Test (ACT)",
            "RCP three questions / clinical asthma control assessment",
        ],
        source_url: "https://www.qoltech.co.uk/acq.html",
    },
    ProprietaryCalculator {
        name: "ohs",
        title: "Oxford Hip Score (OHS)",
        purpose: "Patient-reported outcome after hip replacement (NHS England PROMs).",
        owner: "Oxford University Innovation",
        reason: "The Oxford Hip Score is copyrighted and its use in software requires a licence \
from Oxford University Innovation.",
        alternatives: &["EQ-5D (generic PROM)", "HOOS / HOOS-12 (check licensing)"],
        source_url: "https://innovation.ox.ac.uk/outcome-measures/oxford-hip-score-ohs/",
    },
    ProprietaryCalculator {
        name: "oks",
        title: "Oxford Knee Score (OKS)",
        purpose: "Patient-reported outcome after knee replacement (NHS England PROMs).",
        owner: "Oxford University Innovation",
        reason: "The Oxford Knee Score is copyrighted and its use in software requires a licence \
from Oxford University Innovation.",
        alternatives: &["EQ-5D (generic PROM)", "KOOS / KOOS-12 (check licensing)"],
        source_url: "https://innovation.ox.ac.uk/outcome-measures/oxford-knee-score-oks/",
    },
    ProprietaryCalculator {
        name: "scorad",
        title: "SCORAD",
        purpose: "Generic atopic-dermatitis extent and severity assessment.",
        owner: "European Task Force on Atopic Dermatitis; rights managed through Mapi Research Trust and Pierre Fabre Eczema Foundation",
        reason: "ePROVIDE identifies all rights reserved, and the rights materials reserve \
adaptation and software-integration rights; no unrestricted grant has been identified.",
        alternatives: &[
            "Use SCORAD under a local licence",
            "Clinician-directed eczema assessment (no equivalent unrestricted scored alternative identified)",
        ],
        source_url: "https://www.pierrefabreeczemafoundation.org/en/legal-notice-po-scorad",
    },
];

/// Tools withheld pending permission or legal review, without asserting that
/// independently implementing their published methods is necessarily restricted.
pub const RIGHTS_REVIEW_UNAVAILABLE: &[RightsReviewUnavailableCalculator] = &[
    RightsReviewUnavailableCalculator {
        name: "nyha",
        title: "NYHA Functional Classification",
        purpose: "Functional classification of activity limitation associated with heart disease.",
        rights_holder_or_contact: "American Heart Association / New York Heart Association",
        reason: "The American Heart Association states that its copyrighted materials may not be reproduced without prior written permission, but that general policy does not establish that independent implementation of the classification is prohibited. No unrestricted grant for embedding and redistributing the class descriptors in open software has been identified, so implementation awaits permission or legal review.",
        alternatives: &[
            "mrc_dyspnoea (open breathlessness-related disability scale, shipped here; not equivalent to NYHA)",
            "Clinician assessment using NYHA materials licensed by the local organisation",
        ],
        source_url: "https://www.heart.org/en/about-us/statements-and-policies/copyright",
    },
    RightsReviewUnavailableCalculator {
        name: "norton",
        title: "Norton Scale",
        purpose: "Generic pressure-ulcer risk screening.",
        rights_holder_or_contact: "Centre for Policy on Ageing",
        reason: "LOINC identifies copyright 1962 CPA and says it reproduced the scale with permission, but that does not establish that independent implementation is prohibited. No unrestricted redistribution grant has been identified, so implementation awaits permission or legal review.",
        alternatives: &[
            "braden (shipped; not equivalent)",
            "waterlow (shipped; not equivalent)",
        ],
        source_url: "https://loinc.org/75243-6/panel",
    },
    RightsReviewUnavailableCalculator {
        name: "orbit",
        title: "ORBIT Bleeding Risk Score",
        purpose: "Generic major-bleeding risk stratification in anticoagulated atrial fibrillation.",
        rights_holder_or_contact: "Original authors / Oxford University Press",
        reason: "The original 2015 article is CC BY-NC 4.0. That licence restricts reuse of the article but does not by itself establish that an independent implementation of the published method is prohibited; no explicit unrestricted software-redistribution grant has been identified, so implementation awaits permission or legal review.",
        alternatives: &[
            "hasbled (shipped; not equivalent)",
            "Bleeding-risk scores do not determine whether anticoagulation should be withheld",
        ],
        source_url: "https://pmc.ncbi.nlm.nih.gov/articles/PMC4670965/",
    },
    RightsReviewUnavailableCalculator {
        name: "score2",
        title: "SCORE2 / SCORE2-OP",
        purpose: "Generic European 10-year fatal and non-fatal cardiovascular risk prediction.",
        rights_holder_or_contact: "SCORE2 working groups / European Society of Cardiology",
        reason: "The publications and official calculator access do not provide explicit unrestricted terms for redistributing a complete calibrated software implementation. SCORE2 describes its Stata code as available on request, while SCORE2-OP publishes coefficients; neither fact alone establishes that independent implementation is prohibited, so implementation awaits permission or legal review.",
        alternatives: &[
            "Official ESC HeartScore",
            "qrisk3 (shipped for its validated UK population; not equivalent)",
        ],
        source_url: "https://www.escardio.org/guidelines/practice-tools/cvd-prevention-toolbox/score-risk-charts/",
    },
];

/// Tools withheld because presenting their score would be clinically unsafe.
pub const SAFETY_UNAVAILABLE: &[SafetyUnavailableCalculator] = &[SafetyUnavailableCalculator {
    name: "sad_persons",
    title: "SAD PERSONS",
    purpose: "Generic legacy suicide-risk checklist.",
    reason: "For people presenting after self-harm, NICE NG225 says not to use risk tools or scales to predict future suicide or self-harm or to determine treatment or discharge. Separately, published systematic-review and validation evidence reports poor sensitivity and clinically important misclassification for SAD PERSONS, so exposing a score creates foreseeable disposition harm.",
    alternatives: &[
        "comprehensive clinician psychosocial assessment and risk formulation per NICE NG225",
        "immediate safety assessment and urgent specialist support where indicated",
    ],
    evidence_urls: &[
        "https://www.nice.org.uk/guidance/ng225/chapter/Recommendations#risk-assessment-tools-and-scales",
        "https://doi.org/10.1371/journal.pone.0180292",
        "https://doi.org/10.1002/da.22632",
    ],
    source_url: "https://www.nice.org.uk/guidance/ng225/chapter/Recommendations#risk-assessment-tools-and-scales",
}];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frax_returns_protest_not_a_score() {
        let frax = PROPRIETARY.iter().find(|p| p.name == "frax").unwrap();
        let r = frax.calculate(&json!({})).unwrap();
        assert_eq!(r.result, json!("unavailable: proprietary"));
        assert_eq!(r.working["status"], json!("unavailable-proprietary"));
        assert!(r.working["owner"].as_str().unwrap().contains("Sheffield"));
        assert!(r.interpretation.contains("MP"));
    }

    #[test]
    fn every_proprietary_names_an_alternative_and_owner() {
        for p in PROPRIETARY {
            assert!(!p.owner.is_empty(), "{}: owner required", p.name);
            assert!(
                !p.alternatives.is_empty(),
                "{}: alternatives required",
                p.name
            );
            assert!(p.source_url.starts_with("http"), "{}: source_url", p.name);
        }
    }

    #[test]
    fn proprietary_stubs_enforce_their_empty_object_schema() {
        let frax = PROPRIETARY.iter().find(|p| p.name == "frax").unwrap();

        assert!(frax.calculate(&json!({})).is_ok());
        for invalid in [json!({ "unexpected": true }), json!([]), json!(null)] {
            let error = frax.calculate(&invalid).unwrap_err();
            assert!(matches!(error, CalcError::InvalidInput(_)));
        }
    }

    #[test]
    fn nyha_explains_the_rights_block_without_reproducing_the_classification() {
        let nyha = RIGHTS_REVIEW_UNAVAILABLE
            .iter()
            .find(|calculator| calculator.name == "nyha")
            .unwrap();
        let response = nyha.calculate(&json!({})).unwrap();

        assert_eq!(response.result, json!("unavailable: rights-review"));
        assert!(response.interpretation.contains("prior written permission"));
        assert!(response.interpretation.contains("class descriptors"));
    }

    #[test]
    fn new_rights_locked_names_exist_and_return_no_score() {
        let calculator = PROPRIETARY
            .iter()
            .find(|calculator| calculator.name == "scorad")
            .unwrap();
        let response = calculator.calculate(&json!({})).unwrap();

        assert_eq!(response.result, json!("unavailable: proprietary"));
        assert_eq!(response.working["status"], json!("unavailable-proprietary"));
    }

    #[test]
    fn rights_review_entries_do_not_claim_proprietary_status() {
        for name in ["nyha", "norton", "orbit", "score2"] {
            let calculator = RIGHTS_REVIEW_UNAVAILABLE
                .iter()
                .find(|calculator| calculator.name == name)
                .unwrap();
            let response = calculator.calculate(&json!({})).unwrap();

            assert_eq!(response.result, json!("unavailable: rights-review"));
            assert_eq!(
                response.working["status"],
                json!("unavailable-rights-review")
            );
            assert!(!response.interpretation.contains("is proprietary"));
        }
    }

    #[test]
    fn safety_unavailable_uses_a_distinct_non_proprietary_response() {
        let sad_persons = SAFETY_UNAVAILABLE
            .iter()
            .find(|calculator| calculator.name == "sad_persons")
            .unwrap();
        let response = sad_persons.calculate(&json!({})).unwrap();

        assert_eq!(response.result, json!("unavailable: clinical-safety"));
        assert_eq!(
            response.working["status"],
            json!("unavailable-clinical-safety")
        );
        assert!(response.working.get("owner").is_none());
        assert!(response.interpretation.contains("returns no score"));
        assert!(
            response
                .interpretation
                .contains("must not be used to reduce")
        );
        assert!(response.interpretation.contains("admission/discharge"));
        assert!(!response.interpretation.contains("proprietary"));
    }

    #[test]
    fn every_safety_unavailable_entry_names_alternatives_and_source() {
        for calculator in SAFETY_UNAVAILABLE {
            assert!(
                !calculator.alternatives.is_empty(),
                "{}: alternatives required",
                calculator.name
            );
            assert!(
                calculator.source_url.starts_with("http"),
                "{}: source_url",
                calculator.name
            );
            assert!(!calculator.evidence_urls.is_empty());
            assert!(
                calculator
                    .evidence_urls
                    .iter()
                    .all(|url| url.starts_with("http"))
            );
        }
    }

    #[test]
    fn safety_stubs_enforce_their_empty_object_schema() {
        let sad_persons = &SAFETY_UNAVAILABLE[0];

        assert!(sad_persons.calculate(&json!({})).is_ok());
        for invalid in [json!({ "unexpected": true }), json!([]), json!(null)] {
            let error = sad_persons.calculate(&invalid).unwrap_err();
            assert!(matches!(error, CalcError::InvalidInput(_)));
        }
    }
}
