// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Corrected 2023 Duke-ISCVID Criteria for Infective Endocarditis.
//!
//! This is an observation-derived implementation of the research case
//! definition. Pathology, microbiology, imaging, surgery, clinical findings,
//! and explicit rejection evidence are entered; criterion domains and the
//! final classification are derived rather than supplied by the caller.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "duke_iscvid";
pub const REFERENCE: &str = "Fowler VG Jr, Durack DT, Selton-Suty C, et al. The 2023 Duke-International Society for Cardiovascular Infectious Diseases Criteria for Infective Endocarditis: Updating the Modified Duke Criteria. Clin Infect Dis. 2023;77(4):518-526. doi:10.1093/cid/ciad271. PMCID:PMC10681650. Correction: Clin Infect Dis. 2023;77(8):1222. doi:10.1093/cid/ciad510. PMCID:PMC10893910.";
pub const LICENSE: CalculatorLicense = CalculatorLicense {
    license: "Uncopyrightable method under 17 U.S.C. Section 102(b) - factual criteria independently expressed from an all-rights-reserved source publication; source prose and tables are not redistributed",
    source_url: "https://uscode.house.gov/view.xhtml?req=granuleid:USC-prelim-title17-section102&num=0&edition=prelim",
};

const VERSION: &str = "corrected_2023_duke_iscvid_ciad271_with_ciad510_correction";
const LIMITATIONS: &str = "This is a research case-definition classification that supplements but never replaces clinical judgement. It is not a screening tool, diagnosis substitute, exclusion rule, mortality estimate, indication for surgery, antibiotic choice, or duration rule. Advanced molecular tests, cardiac CT, and FDG PET/CT may be unavailable, particularly in resource-limited settings, and the criteria have recognised sensitivity and specificity limitations.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentContext {
    #[serde(
        rename = "clinician_classification_of_suspected_ie_using_corrected_2023_duke_iscvid_criteria"
    )]
    ClinicianClassificationOfSuspectedIeUsingCorrected2023DukeIscvidCriteria,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PathologicEvidence {
    None,
    QualifyingMicroorganismIdentifiedWithClinicalSignsOfActiveIe,
    ActiveEndocarditisIdentified,
    UnsupportedSingleSkinBacteriumPcrOnValveOrWire,
}

impl PathologicEvidence {
    fn is_definite(self) -> bool {
        matches!(
            self,
            Self::QualifyingMicroorganismIdentifiedWithClinicalSignsOfActiveIe
                | Self::ActiveEndocarditisIdentified
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BloodCultureOrganism {
    None,
    StaphylococcusAureus,
    StaphylococcusLugdunensis,
    EnterococcusFaecalis,
    StreptococcusOtherThanPneumoniaeOrPyogenes,
    Granulicatella,
    Abiotrophia,
    Gemella,
    Hacek,
    CoagulaseNegativeStaphylococcusOtherThanLugdunensis,
    CorynebacteriumStriatumOrJeikeium,
    SerratiaMarcescens,
    PseudomonasAeruginosa,
    CutibacteriumAcnes,
    NontuberculousMycobacterium,
    Candida,
    StreptococcusPneumoniaeOrPyogenes,
    NonFaecalisEnterococcus,
    ClinicianClassifiedOtherOrganismAsOccasionallyCausingIeAndNotACommonContaminant,
    ClinicianClassifiedOtherOrganismAsRarelyCausingIeOrACommonContaminant,
    ClinicianClassifiedOrganismAsNotConsistentWithIe,
}

impl BloodCultureOrganism {
    fn group(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::StaphylococcusAureus
            | Self::StaphylococcusLugdunensis
            | Self::EnterococcusFaecalis
            | Self::StreptococcusOtherThanPneumoniaeOrPyogenes
            | Self::Granulicatella
            | Self::Abiotrophia
            | Self::Gemella
            | Self::Hacek => "universal_typical",
            Self::CoagulaseNegativeStaphylococcusOtherThanLugdunensis
            | Self::CorynebacteriumStriatumOrJeikeium
            | Self::SerratiaMarcescens
            | Self::PseudomonasAeruginosa
            | Self::CutibacteriumAcnes
            | Self::NontuberculousMycobacterium
            | Self::Candida => "prosthetic_material_only_typical",
            Self::ClinicianClassifiedOtherOrganismAsOccasionallyCausingIeAndNotACommonContaminant => {
                "occasional_noncontaminant_nontypical"
            }
            Self::StreptococcusPneumoniaeOrPyogenes
            | Self::NonFaecalisEnterococcus
            | Self::ClinicianClassifiedOtherOrganismAsRarelyCausingIeOrACommonContaminant => {
                "rare_or_common_contaminant_nontypical"
            }
            Self::ClinicianClassifiedOrganismAsNotConsistentWithIe => "not_consistent_with_ie",
        }
    }

    fn is_consistent_with_ie(self) -> bool {
        !matches!(
            self,
            Self::None | Self::ClinicianClassifiedOrganismAsNotConsistentWithIe
        )
    }

    fn is_typical_in_context(self, prosthetic_material: bool) -> bool {
        self.group() == "universal_typical"
            || (prosthetic_material && self.group() == "prosthetic_material_only_typical")
    }

    fn is_blood_major(self, sets: u8, prosthetic_material: bool) -> bool {
        match self.group() {
            "universal_typical" => sets >= 2,
            "prosthetic_material_only_typical" if prosthetic_material => sets >= 2,
            "prosthetic_material_only_typical"
            | "occasional_noncontaminant_nontypical"
            | "rare_or_common_contaminant_nontypical" => sets >= 3,
            _ => false,
        }
    }

    fn is_blood_minor(self, sets: u8, prosthetic_material: bool) -> bool {
        // Table 2 footnote r distinguishes occasional noncontaminants, where
        // one set is minor, from rare/common contaminants, where it is not.
        // The prosthetic-only named group follows its separate context rule.
        match self.group() {
            "universal_typical" => sets == 1,
            "prosthetic_material_only_typical" if prosthetic_material => false,
            "prosthetic_material_only_typical" | "rare_or_common_contaminant_nontypical" => {
                sets == 2
            }
            "occasional_noncontaminant_nontypical" => matches!(sets, 1 | 2),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MajorLaboratoryEvidence {
    None,
    BloodNucleicAcidCoxiellaBartonellaOrTropheryma,
    #[serde(rename = "coxiella_phase_i_igg_greater_than_1_800")]
    CoxiellaPhaseIIggGreaterThan1_800,
    CoxiellaSinglePositiveBloodCulture,
    #[serde(rename = "bartonella_henselae_or_quintana_igg_at_least_1_800")]
    BartonellaHenselaeOrQuintanaIggAtLeast1_800,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnatomicImagingEvidence {
    NoneOrNonqualifying,
    VegetationPerforationAneurysmAbscessPseudoaneurysmOrFistula,
    SignificantNewValvularRegurgitationOnEchocardiographyComparedWithPreviousImaging,
    NewPartialProstheticValveDehiscence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntracardiacProstheticMaterial {
    ProstheticValve,
    ValveRepairMaterial,
    EndovascularCied,
    OtherIntracardiacProstheticMaterial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PetCtEvidence {
    NoneOrNonqualifying,
    QualifyingNativeValveAbnormalUptake,
    QualifyingImplantedIntracardiacMaterialUptakeAtLeastThreeMonths,
    QualifyingImplantedIntracardiacMaterialUptakeLessThanThreeMonths,
    QualifyingAscendingAorticGraftUptakeWithConcomitantValveInvolvementAtLeastThreeMonths,
    QualifyingAscendingAorticGraftUptakeWithConcomitantValveInvolvementLessThanThreeMonths,
    IsolatedGeneratorPocketUptake,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurgicalInspectionEvidence {
    NoneOrNonqualifying,
    DirectEvidenceWithoutMajorImagingOrSubsequentPathologicConfirmation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Predisposition {
    PreviousIe,
    PreviousValveRepairWithoutCurrentMaterial,
    CongenitalHeartDisease,
    MoreThanMildValvularRegurgitationOrStenosis,
    HypertrophicObstructiveCardiomyopathy,
    InjectionDrugUse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VascularPhenomenon {
    ArterialEmbolus,
    SepticPulmonaryInfarct,
    CerebralAbscess,
    SplenicAbscess,
    MycoticAneurysm,
    IntracranialHaemorrhage,
    ConjunctivalHaemorrhage,
    JanewayLesion,
    PurulentPurpura,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImmunologicPhenomenon {
    PositiveRheumatoidFactor,
    OslerNodes,
    RothSpots,
    SourceDefinedImmuneComplexGlomerulonephritis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OtherMinorMicrobiology {
    None,
    IeConsistentOrganismFromOtherSterileSite,
    SerumSequencingOtherThanCoxiellaBartonellaTropheryma,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuscultationEvidence {
    NoneOrNonqualifying,
    NewValvularRegurgitationWithEchocardiographyUnavailable,
    NewRegurgitationButEchocardiographyAvailable,
    ChangedPreexistingMurmur,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RejectionEvidence {
    FirmAlternateMicrobiologicDiagnosisAllThreeConditions,
    FirmAlternateNonmicrobiologicDiagnosisBothConditions,
    NoRecurrenceAfterLessThanFourDaysAntibiotics,
    NoPathologicOrMacroscopicIeAtSurgeryOrAutopsyWithLessThanFourDaysAntibiotics,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DukeIscvidInput {
    pub assessment_context: AssessmentContext,
    pub pathologic_evidence: PathologicEvidence,
    pub intracardiac_prosthetic_material: Vec<IntracardiacProstheticMaterial>,
    pub blood_culture_organism: BloodCultureOrganism,
    pub positive_blood_culture_sets: u8,
    pub major_laboratory_evidence: MajorLaboratoryEvidence,
    pub anatomic_imaging_evidence: AnatomicImagingEvidence,
    pub pet_ct_evidence: PetCtEvidence,
    pub surgical_inspection_evidence: SurgicalInspectionEvidence,
    pub predisposition: Vec<Predisposition>,
    pub maximum_documented_temperature_c: Option<f64>,
    pub vascular_phenomena: Vec<VascularPhenomenon>,
    pub immunologic_phenomena: Vec<ImmunologicPhenomenon>,
    pub other_minor_microbiology: OtherMinorMicrobiology,
    pub auscultation_evidence: AuscultationEvidence,
    pub rejection_evidence: Vec<RejectionEvidence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct MajorDomains {
    pub microbiologic: bool,
    pub imaging: bool,
    pub surgical: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct MinorDomains {
    pub predisposition: bool,
    pub fever: bool,
    pub vascular: bool,
    pub immunologic: bool,
    pub microbiologic: bool,
    pub imaging: bool,
    pub physical_exam: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DukeIscvidOutcome {
    pub result: &'static str,
    pub classification_basis: &'static str,
    pub pathologic_definite: bool,
    pub major_domains: MajorDomains,
    pub minor_domains: MinorDomains,
    pub major_count: u8,
    pub minor_count: u8,
    pub blood_culture_group: &'static str,
    pub blood_culture_major: bool,
    pub blood_culture_minor_threshold_met: bool,
    pub matched_evidence: Vec<&'static str>,
    pub interpretation: String,
}

fn has_duplicates<T: PartialEq>(values: &[T]) -> bool {
    values
        .iter()
        .enumerate()
        .any(|(index, value)| values[index + 1..].contains(value))
}

pub fn compute(input: &DukeIscvidInput) -> Result<DukeIscvidOutcome, CalcError> {
    if has_duplicates(&input.intracardiac_prosthetic_material)
        || has_duplicates(&input.predisposition)
        || has_duplicates(&input.vascular_phenomena)
        || has_duplicates(&input.immunologic_phenomena)
        || has_duplicates(&input.rejection_evidence)
    {
        return Err(CalcError::InvalidInput(
            "intracardiac_prosthetic_material, predisposition, vascular_phenomena, immunologic_phenomena, and rejection_evidence must contain unique entries"
                .into(),
        ));
    }
    if input
        .predisposition
        .contains(&Predisposition::PreviousValveRepairWithoutCurrentMaterial)
        && input
            .intracardiac_prosthetic_material
            .contains(&IntracardiacProstheticMaterial::ValveRepairMaterial)
    {
        return Err(CalcError::InvalidInput(
            "previous_valve_repair_without_current_material contradicts current valve_repair_material"
                .into(),
        ));
    }
    if input
        .maximum_documented_temperature_c
        .is_some_and(|value| !value.is_finite())
    {
        return Err(CalcError::InvalidInput(
            "maximum_documented_temperature_c must be finite when supplied".into(),
        ));
    }
    match (
        input.blood_culture_organism,
        input.positive_blood_culture_sets,
    ) {
        (BloodCultureOrganism::None, 0) => {}
        (BloodCultureOrganism::None, _) => {
            return Err(CalcError::InvalidInput(
                "blood_culture_organism=none requires positive_blood_culture_sets=0".into(),
            ));
        }
        (_, 0) => {
            return Err(CalcError::InvalidInput(
                "a non-none blood_culture_organism requires at least one positive blood culture set"
                    .into(),
            ));
        }
        _ => {}
    }

    let has_intracardiac_prosthetic_material = !input.intracardiac_prosthetic_material.is_empty();
    if input.anatomic_imaging_evidence
        == AnatomicImagingEvidence::NewPartialProstheticValveDehiscence
        && !input
            .intracardiac_prosthetic_material
            .contains(&IntracardiacProstheticMaterial::ProstheticValve)
    {
        return Err(CalcError::InvalidInput(
            "new_partial_prosthetic_valve_dehiscence requires prosthetic_valve in intracardiac_prosthetic_material"
                .into(),
        ));
    }
    if matches!(
        input.pet_ct_evidence,
        PetCtEvidence::QualifyingImplantedIntracardiacMaterialUptakeAtLeastThreeMonths
            | PetCtEvidence::QualifyingImplantedIntracardiacMaterialUptakeLessThanThreeMonths
    ) && !has_intracardiac_prosthetic_material
    {
        return Err(CalcError::InvalidInput(
            "qualifying implanted intracardiac material PET/CT uptake requires at least one intracardiac_prosthetic_material entry"
                .into(),
        ));
    }
    if input.pet_ct_evidence == PetCtEvidence::IsolatedGeneratorPocketUptake
        && !input
            .intracardiac_prosthetic_material
            .contains(&IntracardiacProstheticMaterial::EndovascularCied)
    {
        return Err(CalcError::InvalidInput(
            "isolated_generator_pocket_uptake requires endovascular_cied in intracardiac_prosthetic_material"
                .into(),
        ));
    }

    let pathologic_definite = input.pathologic_evidence.is_definite();
    let anatomic_imaging_major =
        input.anatomic_imaging_evidence != AnatomicImagingEvidence::NoneOrNonqualifying;
    let pet_imaging_major = matches!(
        input.pet_ct_evidence,
        PetCtEvidence::QualifyingNativeValveAbnormalUptake
            | PetCtEvidence::QualifyingImplantedIntracardiacMaterialUptakeAtLeastThreeMonths
            | PetCtEvidence::QualifyingAscendingAorticGraftUptakeWithConcomitantValveInvolvementAtLeastThreeMonths
    );
    let imaging_major = anatomic_imaging_major || pet_imaging_major;
    let surgical_selected = input.surgical_inspection_evidence
        == SurgicalInspectionEvidence::DirectEvidenceWithoutMajorImagingOrSubsequentPathologicConfirmation;
    if surgical_selected && imaging_major {
        return Err(CalcError::InvalidInput(
            "direct surgical evidence is a major criterion only when no major imaging criterion is present"
                .into(),
        ));
    }
    if surgical_selected && pathologic_definite {
        return Err(CalcError::InvalidInput(
            "direct surgical evidence without subsequent pathologic confirmation contradicts pathologic definite evidence"
                .into(),
        ));
    }

    let firm_alternate_microbiology = input
        .rejection_evidence
        .contains(&RejectionEvidence::FirmAlternateMicrobiologicDiagnosisAllThreeConditions);
    let firm_alternate_nonmicrobiology = input
        .rejection_evidence
        .contains(&RejectionEvidence::FirmAlternateNonmicrobiologicDiagnosisBothConditions);
    let no_ie_at_surgery = input.rejection_evidence.contains(
        &RejectionEvidence::NoPathologicOrMacroscopicIeAtSurgeryOrAutopsyWithLessThanFourDaysAntibiotics,
    );
    if pathologic_definite
        && (firm_alternate_microbiology || firm_alternate_nonmicrobiology || no_ie_at_surgery)
    {
        return Err(CalcError::InvalidInput(
            "pathologic definite IE contradicts a firm alternate diagnosis or explicit absence of pathologic/macroscopic IE"
                .into(),
        ));
    }
    let any_qualifying_cardiac_imaging = imaging_major
        || matches!(
            input.pet_ct_evidence,
            PetCtEvidence::QualifyingImplantedIntracardiacMaterialUptakeLessThanThreeMonths
                | PetCtEvidence::QualifyingAscendingAorticGraftUptakeWithConcomitantValveInvolvementLessThanThreeMonths
        );
    if firm_alternate_microbiology
        && (input.positive_blood_culture_sets == 0
            || input
                .blood_culture_organism
                .is_typical_in_context(has_intracardiac_prosthetic_material))
    {
        return Err(CalcError::InvalidInput(
            "firm alternate microbiologic diagnosis requires at least one positive set for an organism that is not typical in the patient's intracardiac-prosthetic-material context"
                .into(),
        ));
    }
    if firm_alternate_microbiology && any_qualifying_cardiac_imaging {
        return Err(CalcError::InvalidInput(
            "firm alternate microbiologic diagnosis requires absence of IE evidence on cardiac imaging"
                .into(),
        ));
    }
    let any_microbiologic_evidence = input.pathologic_evidence
        == PathologicEvidence::QualifyingMicroorganismIdentifiedWithClinicalSignsOfActiveIe
        || input.pathologic_evidence
            == PathologicEvidence::UnsupportedSingleSkinBacteriumPcrOnValveOrWire
        || input.blood_culture_organism.is_consistent_with_ie()
        || input.major_laboratory_evidence != MajorLaboratoryEvidence::None
        || input.other_minor_microbiology != OtherMinorMicrobiology::None;
    if firm_alternate_nonmicrobiology && !firm_alternate_microbiology && any_microbiologic_evidence
    {
        return Err(CalcError::InvalidInput(
            "firm alternate nonmicrobiologic diagnosis requires absence of microbiologic evidence for IE"
                .into(),
        ));
    }
    if no_ie_at_surgery && surgical_selected {
        return Err(CalcError::InvalidInput(
            "direct surgical evidence of IE contradicts no pathologic or macroscopic IE at surgery or autopsy"
                .into(),
        ));
    }

    let blood_culture_major = input.blood_culture_organism.is_blood_major(
        input.positive_blood_culture_sets,
        has_intracardiac_prosthetic_material,
    );
    let laboratory_major = input.major_laboratory_evidence != MajorLaboratoryEvidence::None;
    let microbiologic_major = blood_culture_major || laboratory_major;
    let pet_imaging_minor = matches!(
        input.pet_ct_evidence,
        PetCtEvidence::QualifyingImplantedIntracardiacMaterialUptakeLessThanThreeMonths
            | PetCtEvidence::QualifyingAscendingAorticGraftUptakeWithConcomitantValveInvolvementLessThanThreeMonths
    );
    let below_major_blood_cultures = input.blood_culture_organism.is_blood_minor(
        input.positive_blood_culture_sets,
        has_intracardiac_prosthetic_material,
    );
    let other_microbiology_minor = input.other_minor_microbiology != OtherMinorMicrobiology::None
        || input.pathologic_evidence
            == PathologicEvidence::UnsupportedSingleSkinBacteriumPcrOnValveOrWire
        || below_major_blood_cultures;
    let microbiologic_minor = !microbiologic_major && other_microbiology_minor;

    let major_domains = MajorDomains {
        microbiologic: microbiologic_major,
        imaging: imaging_major,
        surgical: surgical_selected,
    };
    let minor_domains = MinorDomains {
        predisposition: !input.predisposition.is_empty()
            || input
                .intracardiac_prosthetic_material
                .iter()
                .any(|material| {
                    matches!(
                        material,
                        IntracardiacProstheticMaterial::ProstheticValve
                            | IntracardiacProstheticMaterial::ValveRepairMaterial
                            | IntracardiacProstheticMaterial::EndovascularCied
                    )
                }),
        fever: input
            .maximum_documented_temperature_c
            .is_some_and(|value| value > 38.0),
        vascular: !input.vascular_phenomena.is_empty(),
        immunologic: !input.immunologic_phenomena.is_empty(),
        microbiologic: microbiologic_minor,
        imaging: pet_imaging_minor,
        physical_exam: input.auscultation_evidence
            == AuscultationEvidence::NewValvularRegurgitationWithEchocardiographyUnavailable,
    };
    let major_count = [
        major_domains.microbiologic,
        major_domains.imaging,
        major_domains.surgical,
    ]
    .into_iter()
    .map(u8::from)
    .sum();
    let minor_count = [
        minor_domains.predisposition,
        minor_domains.fever,
        minor_domains.vascular,
        minor_domains.immunologic,
        minor_domains.microbiologic,
        minor_domains.imaging,
        minor_domains.physical_exam,
    ]
    .into_iter()
    .map(u8::from)
    .sum();

    let explicit_rejection = !input.rejection_evidence.is_empty();
    let (result, classification_basis) = if pathologic_definite {
        ("definite", "pathologic_criteria")
    } else if explicit_rejection {
        ("rejected", "explicit_rejection_criteria")
    } else if major_count >= 2 {
        ("definite", "two_major_criteria")
    } else if major_count >= 1 && minor_count >= 3 {
        ("definite", "one_major_and_three_minor_criteria")
    } else if minor_count >= 5 {
        ("definite", "five_minor_criteria")
    } else if major_count >= 1 && minor_count >= 1 {
        ("possible", "one_major_and_one_minor_criterion")
    } else if minor_count >= 3 {
        ("possible", "three_minor_criteria")
    } else {
        ("rejected", "below_possible_clinical_criteria")
    };

    let mut matched_evidence = Vec::new();
    if pathologic_definite {
        matched_evidence.push("pathologic_definite");
    }
    if blood_culture_major {
        matched_evidence.push("major_microbiology_blood_cultures");
    }
    if laboratory_major {
        matched_evidence.push("major_microbiology_laboratory");
    }
    if imaging_major {
        matched_evidence.push("major_imaging");
    }
    if surgical_selected {
        matched_evidence.push("major_surgical");
    }
    if minor_domains.predisposition {
        matched_evidence.push("minor_predisposition");
    }
    if minor_domains.fever {
        matched_evidence.push("minor_fever");
    }
    if minor_domains.vascular {
        matched_evidence.push("minor_vascular");
    }
    if minor_domains.immunologic {
        matched_evidence.push("minor_immunologic");
    }
    if minor_domains.microbiologic {
        matched_evidence.push("minor_microbiology");
    }
    if minor_domains.imaging {
        matched_evidence.push("minor_imaging");
    }
    if minor_domains.physical_exam {
        matched_evidence.push("minor_physical_exam");
    }
    if explicit_rejection {
        matched_evidence.push("explicit_rejection_evidence");
    }

    let rejection_warning = if result == "rejected" {
        " A rejected or below-possible classification does not independently exclude infective endocarditis."
    } else {
        ""
    };
    let interpretation = format!(
        "Corrected 2023 Duke-ISCVID classification: {result} ({classification_basis}; {major_count} major and {minor_count} minor clinical criterion domains).{rejection_warning} {LIMITATIONS}"
    );

    Ok(DukeIscvidOutcome {
        result,
        classification_basis,
        pathologic_definite,
        major_domains,
        minor_domains,
        major_count,
        minor_count,
        blood_culture_group: input.blood_culture_organism.group(),
        blood_culture_major,
        blood_culture_minor_threshold_met: below_major_blood_cultures,
        matched_evidence,
        interpretation,
    })
}

pub fn build_response(input: &DukeIscvidInput) -> Result<CalculationResponse, CalcError> {
    let outcome = compute(input)?;
    let mut working = Map::new();
    working.insert(
        "classification_basis".into(),
        json!(outcome.classification_basis),
    );
    working.insert("criteria_version".into(), json!(VERSION));
    working.insert("assessment_context".into(), json!(input.assessment_context));
    working.insert(
        "pathologic_definite".into(),
        json!(outcome.pathologic_definite),
    );
    working.insert(
        "pathologic_evidence".into(),
        json!(input.pathologic_evidence),
    );
    working.insert("major_domains".into(), json!(outcome.major_domains));
    working.insert("major_count".into(), json!(outcome.major_count));
    working.insert("minor_domains".into(), json!(outcome.minor_domains));
    working.insert("minor_count".into(), json!(outcome.minor_count));
    working.insert("matched_evidence".into(), json!(outcome.matched_evidence));
    working.insert(
        "intracardiac_prosthetic_material".into(),
        json!(input.intracardiac_prosthetic_material),
    );
    working.insert(
        "expanded_typical_organism_context".into(),
        json!(!input.intracardiac_prosthetic_material.is_empty()),
    );
    working.insert(
        "blood_culture_organism".into(),
        json!(input.blood_culture_organism),
    );
    working.insert(
        "blood_culture_derived_group".into(),
        json!(outcome.blood_culture_group),
    );
    working.insert(
        "positive_blood_culture_sets".into(),
        json!(input.positive_blood_culture_sets),
    );
    working.insert(
        "blood_culture_major".into(),
        json!(outcome.blood_culture_major),
    );
    working.insert(
        "blood_culture_minor_threshold_met".into(),
        json!(outcome.blood_culture_minor_threshold_met),
    );
    working.insert(
        "major_laboratory_evidence".into(),
        json!(input.major_laboratory_evidence),
    );
    working.insert(
        "anatomic_imaging_evidence".into(),
        json!(input.anatomic_imaging_evidence),
    );
    working.insert("pet_ct_evidence".into(), json!(input.pet_ct_evidence));
    working.insert(
        "surgical_inspection_evidence".into(),
        json!(input.surgical_inspection_evidence),
    );
    working.insert("predisposition".into(), json!(input.predisposition));
    working.insert(
        "maximum_documented_temperature_c".into(),
        json!(input.maximum_documented_temperature_c),
    );
    working.insert("vascular_phenomena".into(), json!(input.vascular_phenomena));
    working.insert(
        "immunologic_phenomena".into(),
        json!(input.immunologic_phenomena),
    );
    working.insert(
        "other_minor_microbiology".into(),
        json!(input.other_minor_microbiology),
    );
    working.insert(
        "auscultation_evidence".into(),
        json!(input.auscultation_evidence),
    );
    working.insert("rejection_evidence".into(), json!(input.rejection_evidence));
    working.insert("limitations".into(), json!(LIMITATIONS));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(outcome.result),
        interpretation: outcome.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn definition(concept: &str, statement: &str, caveats: &str, source: &Value) -> Value {
    json!({
        "concept": concept,
        "statement": statement,
        "excludes": ["Caller-supplied major or minor counts", "Caller-supplied points", "Unverified inference from incomplete records"],
        "caveats": caveats,
        "source": source,
        "status": "draft"
    })
}

fn enum_property(
    concept: &str,
    description: &str,
    values: &[&str],
    caveats: &str,
    source: &Value,
) -> Value {
    json!({
        "type": "string",
        "enum": values,
        "description": description,
        "definition": definition(concept, description, caveats, source)
    })
}

fn array_property(
    concept: &str,
    description: &str,
    values: &[&str],
    caveats: &str,
    source: &Value,
) -> Value {
    json!({
        "type": "array",
        "uniqueItems": true,
        "items": { "type": "string", "enum": values },
        "description": description,
        "definition": definition(concept, description, caveats, source)
    })
}

fn input_schema() -> Value {
    let source = json!({
        "citation": "Fowler VG Jr et al. Clin Infect Dis. 2023;77(4):518-526, corrected Clin Infect Dis. 2023;77(8):1222.",
        "url": "https://pmc.ncbi.nlm.nih.gov/articles/PMC10681650/",
        "correction_url": "https://pmc.ncbi.nlm.nih.gov/articles/PMC10893910/"
    });
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "DukeIscvidInput",
        "description": "Observation-derived corrected 2023 Duke-ISCVID research case definition for clinician classification of suspected infective endocarditis. Enter evidence, not points or criterion counts. The streptococcal exclusions use the corrected publication: Streptococcus pneumoniae and Streptococcus pyogenes. The all-rights-reserved source publication is cited for provenance; its prose and tables are not redistributed, and these factual rules are independently expressed.",
        "type": "object",
        "additionalProperties": false,
        "required": [
            "assessment_context", "pathologic_evidence", "intracardiac_prosthetic_material",
            "blood_culture_organism", "positive_blood_culture_sets", "major_laboratory_evidence",
            "anatomic_imaging_evidence", "pet_ct_evidence", "surgical_inspection_evidence",
            "predisposition", "maximum_documented_temperature_c", "vascular_phenomena",
            "immunologic_phenomena", "other_minor_microbiology", "auscultation_evidence",
            "rejection_evidence"
        ],
        "properties": {
            "assessment_context": {
                "type": "string",
                "const": "clinician_classification_of_suspected_ie_using_corrected_2023_duke_iscvid_criteria",
                "description": "A clinician is classifying suspected infective endocarditis using the corrected 2023 Duke-ISCVID research case definition.",
                "definition": definition("Duke-ISCVID assessment context", "Confirm clinician classification of suspected IE using the corrected 2023 Duke-ISCVID criteria.", "This is not patient self-assessment, screening, or a substitute for diagnosis and clinical judgement.", &source)
            },
            "pathologic_evidence": enum_property(
                "Pathologic evidence of definite IE",
                "Direct pathologic evidence. A qualifying microorganism must be identified by culture, staining, immunologic or nucleic-acid methods in a vegetation, cardiac tissue, explanted prosthetic valve or sewing ring, ascending aortic graft with valve involvement, endovascular CIED, or arterial embolus, with clinical signs and histological context of active IE. Active endocarditis may be acute or subacute/chronic. A lone skin-bacterium PCR result on a valve or wire without support is minor microbiology, not pathologic definite.",
                &["none", "qualifying_microorganism_identified_with_clinical_signs_of_active_ie", "active_endocarditis_identified", "unsupported_single_skin_bacterium_pcr_on_valve_or_wire"],
                "Molecular and staining results can persist after treated IE and can be false positive; interpret with clinical and histological evidence. An ascending aortic graft qualifies only with concomitant valve involvement.", &source
            ),
            "intracardiac_prosthetic_material": array_property(
                "Intracardiac prosthetic material",
                "Current intracardiac prosthetic material. Any entry activates the expanded typical-organism context. Prosthetic valve, valve-repair material, and endovascular CIED also establish the source-defined predisposition minor domain; other intracardiac prosthetic material alone does not invent a predisposition criterion.",
                &["prosthetic_valve", "valve_repair_material", "endovascular_cied", "other_intracardiac_prosthetic_material"],
                "Use each current material once. This is the single observation source for both organism-context and material-based predisposition derivation.", &source
            ),
            "blood_culture_organism": enum_property(
                "Blood-culture organism group",
                "Organism isolated from the positive blood-culture sets. Use a named variant whenever available. The three clinician_classified catchalls require a clinician or laboratory specialist to classify the identified organism under the source definitions; they must not be guessed by a patient, LLM, or non-specialist. The corrected streptococcal exclusions are S. pneumoniae and S. pyogenes.",
                &["none", "staphylococcus_aureus", "staphylococcus_lugdunensis", "enterococcus_faecalis", "streptococcus_other_than_pneumoniae_or_pyogenes", "granulicatella", "abiotrophia", "gemella", "hacek", "coagulase_negative_staphylococcus_other_than_lugdunensis", "corynebacterium_striatum_or_jeikeium", "serratia_marcescens", "pseudomonas_aeruginosa", "cutibacterium_acnes", "nontuberculous_mycobacterium", "candida", "streptococcus_pneumoniae_or_pyogenes", "non_faecalis_enterococcus", "clinician_classified_other_organism_as_occasionally_causing_ie_and_not_a_common_contaminant", "clinician_classified_other_organism_as_rarely_causing_ie_or_a_common_contaminant", "clinician_classified_organism_as_not_consistent_with_ie"],
                "Universal typical organisms: 1 positive set is minor and at least 2 are major. Prosthetic-material-only typical organisms: with intracardiac prosthetic material, 1 set is not minor and at least 2 are major; without material, 1 set is no criterion, 2 are minor, and at least 3 are major. Occasional noncontaminant nontypical organisms: 1 or 2 sets are minor and at least 3 are major. Rare or common-contaminant nontypical organisms, conservatively including S. pneumoniae/S. pyogenes and non-faecalis Enterococcus: 1 set is no criterion, 2 are minor, and at least 3 are major. Catchall classification is a clinician/laboratory attestation under the source definitions, not a patient, LLM, or non-specialist inference; use a named variant when available.", &source
            ),
            "positive_blood_culture_sets": {
                "type": "integer", "minimum": 0, "maximum": 255,
                "description": "Number of positive blood-culture sets. One set is a simultaneously drawn aerobic/anaerobic bottle pair and is positive if either bottle grows the organism.",
                "definition": definition("Positive blood-culture sets", "Count positive sets for the selected organism; none requires zero and every named organism requires at least one.", "The corrected 2023 criteria impose no timing or separate-venipuncture requirement, although separate venipunctures remain recommended when possible.", &source)
            },
            "major_laboratory_evidence": enum_property(
                "Non-routine microbiologic major evidence",
                "Qualifying blood nucleic-acid detection of Coxiella burnetii, Bartonella species, or Tropheryma whipplei; Coxiella phase I IgG >1:800; a single positive Coxiella blood culture; or Bartonella henselae/quintana IgG >=1:800.",
                &["none", "blood_nucleic_acid_coxiella_bartonella_or_tropheryma", "coxiella_phase_i_igg_greater_than_1_800", "coxiella_single_positive_blood_culture", "bartonella_henselae_or_quintana_igg_at_least_1_800"],
                "The Coxiella antibody threshold is strictly greater than 1:800; the Bartonella threshold is greater than or equal to 1:800 (or equivalent titres with other methodologies).", &source
            ),
            "anatomic_imaging_evidence": enum_property(
                "Anatomic imaging major evidence",
                "Structural lesions (vegetation, perforation, aneurysm, abscess, pseudoaneurysm, or intracardiac fistula) may be shown by echocardiography or cardiac CT. Significant new valvular regurgitation must be shown on echocardiography compared with previous imaging. New partial prosthetic-valve dehiscence compared with previous imaging also qualifies.",
                &["none_or_nonqualifying", "vegetation_perforation_aneurysm_abscess_pseudoaneurysm_or_fistula", "significant_new_valvular_regurgitation_on_echocardiography_compared_with_previous_imaging", "new_partial_prosthetic_valve_dehiscence"],
                "Cardiac CT does not establish the new-regurgitation route. Worsening or changing pre-existing regurgitation alone is not sufficient. Vegetation means an oscillating intracardiac mass on valve, cardiac tissue, endovascular CIED, or implanted material without another anatomic explanation.", &source
            ),
            "pet_ct_evidence": enum_property(
                "FDG PET/CT evidence",
                "Qualifying visually abnormal FDG uptake on a native valve, implanted intracardiac material, or an ascending aortic graft with concomitant valve involvement. Intracardiac-material and qualifying graft uptake at least 3 months after implantation is major; uptake before 3 months is minor.",
                &["none_or_nonqualifying", "qualifying_native_valve_abnormal_uptake", "qualifying_implanted_intracardiac_material_uptake_at_least_three_months", "qualifying_implanted_intracardiac_material_uptake_less_than_three_months", "qualifying_ascending_aortic_graft_uptake_with_concomitant_valve_involvement_at_least_three_months", "qualifying_ascending_aortic_graft_uptake_with_concomitant_valve_involvement_less_than_three_months", "isolated_generator_pocket_uptake"],
                "Implanted-intracardiac-material variants require at least one intracardiac_prosthetic_material entry. Graft variants require concomitant valve involvement but do not imply intracardiac prosthetic material or predisposition. Isolated generator-pocket uptake requires endovascular_cied and is neither major nor minor. Use intense focal, multifocal, or heterogeneous uptake for prosthetic-valve IE; some prostheses have intrinsic non-pathological uptake.", &source
            ),
            "surgical_inspection_evidence": enum_property(
                "Surgical major evidence",
                "Direct operative inspection evidence of IE, such as vegetation, abscess, valvular destruction, or prosthetic-valve dehiscence/loosening, only when major imaging and subsequent histologic or microbiologic confirmation are absent.",
                &["none_or_nonqualifying", "direct_evidence_without_major_imaging_or_subsequent_pathologic_confirmation"],
                "This criterion does not justify omitting appropriate histopathologic and microbiologic sampling. Contradictory major imaging or pathologic evidence is rejected rather than double counted.", &source
            ),
            "predisposition": array_property(
                "Predisposition minor domain",
                "Observed IE predispositions; one or more entries produce one minor domain.",
                &["previous_ie", "previous_valve_repair_without_current_material", "congenital_heart_disease", "more_than_mild_valvular_regurgitation_or_stenosis", "hypertrophic_obstructive_cardiomyopathy", "injection_drug_use"],
                "Current prosthetic valve, valve-repair material, and endovascular CIED are entered only in intracardiac_prosthetic_material and derive this same single domain there. Use previous_valve_repair_without_current_material only when no current repair material remains. Congenital heart disease includes repaired or unrepaired congenital anomalies.", &source
            ),
            "maximum_documented_temperature_c": {
                "type": ["number", "null"], "unit": "Cel",
                "description": "Maximum documented temperature in degrees Celsius, or null when no valid measurement is documented. Fever minor requires >38.0 C; exactly 38.0 C does not qualify.",
                "definition": definition("Documented fever minor domain", "Enter the maximum documented temperature; the calculator derives fever only when it is strictly greater than 38.0 C.", "Do not infer fever from symptoms or round 38.0 C upward.", &source)
            },
            "vascular_phenomena": array_property(
                "Vascular phenomena minor domain",
                "Clinical or radiological vascular phenomena; one or more entries produce one minor domain.",
                &["arterial_embolus", "septic_pulmonary_infarct", "cerebral_abscess", "splenic_abscess", "mycotic_aneurysm", "intracranial_haemorrhage", "conjunctival_haemorrhage", "janeway_lesion", "purulent_purpura"],
                "Multiple findings remain one vascular minor criterion.", &source
            ),
            "immunologic_phenomena": array_property(
                "Immunologic phenomena minor domain",
                "Observed immunologic phenomena; one or more entries produce one minor domain.",
                &["positive_rheumatoid_factor", "osler_nodes", "roth_spots", "source_defined_immune_complex_glomerulonephritis"],
                "Source-defined immune-complex glomerulonephritis requires either renal biopsy consistent with immune-complex-mediated renal disease, or unexplained AKI/acute-on-chronic kidney injury plus at least 2 of haematuria, proteinuria, cellular urinary casts, or serologic perturbation (hypocomplementaemia, cryoglobulinaemia, and/or circulating immune complexes). AKI is a new unexplained eGFR <60 mL/min/1.73 m2; acute-on-chronic injury is a reduction of at least one ordinal eGFR category: >=60, 30-59, 15-29, or <15 mL/min/1.73 m2.", &source
            ),
            "other_minor_microbiology": enum_property(
                "Other microbiology minor evidence",
                "Other IE-consistent microbiology: a positive culture or nucleic-acid test from a normally sterile site other than cardiac tissue, cardiac prosthesis, or arterial embolus; or positive serum amplicon/metagenomic sequencing for an organism other than Coxiella, Bartonella, or Tropheryma.",
                &["none", "ie_consistent_organism_from_other_sterile_site", "serum_sequencing_other_than_coxiella_bartonella_tropheryma"],
                "This contributes only when the microbiologic major domain is absent. Blood cultures meeting the exact organism-specific minor set threshold and an unsupported single skin-bacterium PCR on a valve/wire are derived separately into the same single minor domain.", &source
            ),
            "auscultation_evidence": enum_property(
                "Physical-examination minor evidence",
                "New valvular regurgitation identified by auscultation qualifies only when echocardiography is unavailable.",
                &["none_or_nonqualifying", "new_valvular_regurgitation_with_echocardiography_unavailable", "new_regurgitation_but_echocardiography_available", "changed_preexisting_murmur"],
                "A changed pre-existing murmur is insufficient. New auscultatory regurgitation does not qualify when echocardiography is available.", &source
            ),
            "rejection_evidence": array_property(
                "Explicit rejection evidence",
                "Source-defined explicit rejection routes. The microbiologic alternate route asserts all three conditions: identifiable non-typical-pathogen bloodstream source, rapid bloodstream-infection resolution, and no IE evidence on cardiac imaging. The nonmicrobiologic alternate route asserts both a non-IE cause for imaging findings and no microbiologic IE evidence.",
                &["firm_alternate_microbiologic_diagnosis_all_three_conditions", "firm_alternate_nonmicrobiologic_diagnosis_both_conditions", "no_recurrence_after_less_than_four_days_antibiotics", "no_pathologic_or_macroscopic_ie_at_surgery_or_autopsy_with_less_than_four_days_antibiotics"],
                "Any explicit route precedes count-based clinical classification unless pathologic definite evidence is present. Pathologic definite IE may coexist with the no-recurrence-after-short-antibiotics observation and remains definite. It is internally contradictory with either firm alternate diagnosis or explicit absence of pathologic/macroscopic IE. Less than four days is strict, not four days or fewer.", &source
            )
        },
        "allOf": [
            {
                "if": {
                    "properties": { "predisposition": { "contains": { "const": "previous_valve_repair_without_current_material" } } },
                    "required": ["predisposition"]
                },
                "then": {
                    "properties": { "intracardiac_prosthetic_material": { "not": { "contains": { "const": "valve_repair_material" } } } }
                }
            },
            {
                "if": { "properties": { "blood_culture_organism": { "const": "none" } }, "required": ["blood_culture_organism"] },
                "then": { "properties": { "positive_blood_culture_sets": { "const": 0 } } },
                "else": { "properties": { "positive_blood_culture_sets": { "minimum": 1 } } }
            },
            {
                "if": {
                    "properties": { "surgical_inspection_evidence": { "const": "direct_evidence_without_major_imaging_or_subsequent_pathologic_confirmation" } },
                    "required": ["surgical_inspection_evidence"]
                },
                "then": {
                    "properties": {
                        "pathologic_evidence": { "enum": ["none", "unsupported_single_skin_bacterium_pcr_on_valve_or_wire"] },
                        "anatomic_imaging_evidence": { "const": "none_or_nonqualifying" },
                        "pet_ct_evidence": { "enum": ["none_or_nonqualifying", "qualifying_implanted_intracardiac_material_uptake_less_than_three_months", "qualifying_ascending_aortic_graft_uptake_with_concomitant_valve_involvement_less_than_three_months", "isolated_generator_pocket_uptake"] }
                    }
                }
            },
            {
                "if": {
                    "properties": { "pathologic_evidence": { "enum": ["qualifying_microorganism_identified_with_clinical_signs_of_active_ie", "active_endocarditis_identified"] } },
                    "required": ["pathologic_evidence"]
                },
                "then": {
                    "properties": {
                        "rejection_evidence": {
                            "not": {
                                "contains": {
                                    "enum": ["firm_alternate_microbiologic_diagnosis_all_three_conditions", "firm_alternate_nonmicrobiologic_diagnosis_both_conditions", "no_pathologic_or_macroscopic_ie_at_surgery_or_autopsy_with_less_than_four_days_antibiotics"]
                                }
                            }
                        }
                    }
                }
            },
            {
                "if": {
                    "properties": { "rejection_evidence": { "contains": { "const": "firm_alternate_microbiologic_diagnosis_all_three_conditions" } } },
                    "required": ["rejection_evidence"]
                },
                "then": {
                    "properties": {
                        "positive_blood_culture_sets": { "minimum": 1 },
                        "blood_culture_organism": { "enum": ["coagulase_negative_staphylococcus_other_than_lugdunensis", "corynebacterium_striatum_or_jeikeium", "serratia_marcescens", "pseudomonas_aeruginosa", "cutibacterium_acnes", "nontuberculous_mycobacterium", "candida", "streptococcus_pneumoniae_or_pyogenes", "non_faecalis_enterococcus", "clinician_classified_other_organism_as_occasionally_causing_ie_and_not_a_common_contaminant", "clinician_classified_other_organism_as_rarely_causing_ie_or_a_common_contaminant", "clinician_classified_organism_as_not_consistent_with_ie"] },
                        "anatomic_imaging_evidence": { "const": "none_or_nonqualifying" },
                        "pet_ct_evidence": { "enum": ["none_or_nonqualifying", "isolated_generator_pocket_uptake"] }
                    },
                    "allOf": [
                        {
                            "if": { "properties": { "intracardiac_prosthetic_material": { "minItems": 1 } }, "required": ["intracardiac_prosthetic_material"] },
                            "then": { "properties": { "blood_culture_organism": { "enum": ["streptococcus_pneumoniae_or_pyogenes", "non_faecalis_enterococcus", "clinician_classified_other_organism_as_occasionally_causing_ie_and_not_a_common_contaminant", "clinician_classified_other_organism_as_rarely_causing_ie_or_a_common_contaminant", "clinician_classified_organism_as_not_consistent_with_ie"] } } }
                        }
                    ]
                }
            },
            {
                "if": {
                    "allOf": [
                        { "properties": { "rejection_evidence": { "contains": { "const": "firm_alternate_nonmicrobiologic_diagnosis_both_conditions" } } } },
                        { "not": { "properties": { "rejection_evidence": { "contains": { "const": "firm_alternate_microbiologic_diagnosis_all_three_conditions" } } } } }
                    ]
                },
                "then": {
                    "properties": {
                        "pathologic_evidence": { "const": "none" },
                        "blood_culture_organism": { "enum": ["none", "clinician_classified_organism_as_not_consistent_with_ie"] },
                        "major_laboratory_evidence": { "const": "none" },
                        "other_minor_microbiology": { "const": "none" }
                    }
                }
            },
            {
                "if": {
                    "properties": { "rejection_evidence": { "contains": { "const": "no_pathologic_or_macroscopic_ie_at_surgery_or_autopsy_with_less_than_four_days_antibiotics" } } },
                    "required": ["rejection_evidence"]
                },
                "then": {
                    "properties": {
                        "pathologic_evidence": { "enum": ["none", "unsupported_single_skin_bacterium_pcr_on_valve_or_wire"] },
                        "surgical_inspection_evidence": { "const": "none_or_nonqualifying" }
                    }
                }
            },
            {
                "if": {
                    "properties": { "anatomic_imaging_evidence": { "const": "new_partial_prosthetic_valve_dehiscence" } },
                    "required": ["anatomic_imaging_evidence"]
                },
                "then": { "properties": { "intracardiac_prosthetic_material": { "contains": { "const": "prosthetic_valve" } } } }
            },
            {
                "if": {
                    "properties": { "pet_ct_evidence": { "enum": ["qualifying_implanted_intracardiac_material_uptake_at_least_three_months", "qualifying_implanted_intracardiac_material_uptake_less_than_three_months"] } },
                    "required": ["pet_ct_evidence"]
                },
                "then": { "properties": { "intracardiac_prosthetic_material": { "minItems": 1 } } }
            },
            {
                "if": {
                    "properties": { "pet_ct_evidence": { "const": "isolated_generator_pocket_uptake" } },
                    "required": ["pet_ct_evidence"]
                },
                "then": { "properties": { "intracardiac_prosthetic_material": { "contains": { "const": "endovascular_cied" } } } }
            }
        ]
    })
}

pub struct DukeIscvid;

impl Calculator for DukeIscvid {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "2023 Duke-ISCVID Criteria for Infective Endocarditis"
    }

    fn description(&self) -> &'static str {
        "Corrected 2023 observation-derived research case-definition classification of suspected infective endocarditis as definite, possible, or rejected."
    }

    fn reference(&self) -> &'static str {
        REFERENCE
    }

    fn license(&self) -> CalculatorLicense {
        LICENSE
    }

    fn input_schema(&self) -> Value {
        input_schema()
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: DukeIscvidInput = serde_json::from_value(input.clone())
            .map_err(|error| CalcError::InvalidInput(error.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty() -> DukeIscvidInput {
        DukeIscvidInput {
            assessment_context: AssessmentContext::ClinicianClassificationOfSuspectedIeUsingCorrected2023DukeIscvidCriteria,
            pathologic_evidence: PathologicEvidence::None,
            intracardiac_prosthetic_material: vec![],
            blood_culture_organism: BloodCultureOrganism::None,
            positive_blood_culture_sets: 0,
            major_laboratory_evidence: MajorLaboratoryEvidence::None,
            anatomic_imaging_evidence: AnatomicImagingEvidence::NoneOrNonqualifying,
            pet_ct_evidence: PetCtEvidence::NoneOrNonqualifying,
            surgical_inspection_evidence: SurgicalInspectionEvidence::NoneOrNonqualifying,
            predisposition: vec![],
            maximum_documented_temperature_c: None,
            vascular_phenomena: vec![],
            immunologic_phenomena: vec![],
            other_minor_microbiology: OtherMinorMicrobiology::None,
            auscultation_evidence: AuscultationEvidence::NoneOrNonqualifying,
            rejection_evidence: vec![],
        }
    }

    fn one_major(input: &mut DukeIscvidInput) {
        input.major_laboratory_evidence =
            MajorLaboratoryEvidence::BloodNucleicAcidCoxiellaBartonellaOrTropheryma;
    }

    fn three_minor(input: &mut DukeIscvidInput) {
        input.predisposition = vec![Predisposition::PreviousIe];
        input.maximum_documented_temperature_c = Some(38.1);
        input.vascular_phenomena = vec![VascularPhenomenon::ArterialEmbolus];
    }

    #[test]
    fn table_one_and_two_rule_conformance_covers_every_classification_route() {
        let mut pathologic = empty();
        pathologic.pathologic_evidence = PathologicEvidence::ActiveEndocarditisIdentified;
        assert_eq!(
            compute(&pathologic).unwrap().classification_basis,
            "pathologic_criteria"
        );

        let mut two_major = empty();
        one_major(&mut two_major);
        two_major.intracardiac_prosthetic_material =
            vec![IntracardiacProstheticMaterial::ProstheticValve];
        two_major.anatomic_imaging_evidence =
            AnatomicImagingEvidence::NewPartialProstheticValveDehiscence;
        assert_eq!(
            compute(&two_major).unwrap().classification_basis,
            "two_major_criteria"
        );

        let mut major_three_minor = empty();
        one_major(&mut major_three_minor);
        three_minor(&mut major_three_minor);
        assert_eq!(
            compute(&major_three_minor).unwrap().classification_basis,
            "one_major_and_three_minor_criteria"
        );

        let mut five_minor = empty();
        three_minor(&mut five_minor);
        five_minor.immunologic_phenomena = vec![ImmunologicPhenomenon::OslerNodes];
        five_minor.other_minor_microbiology =
            OtherMinorMicrobiology::IeConsistentOrganismFromOtherSterileSite;
        assert_eq!(
            compute(&five_minor).unwrap().classification_basis,
            "five_minor_criteria"
        );

        let mut major_minor = empty();
        one_major(&mut major_minor);
        major_minor.predisposition = vec![Predisposition::PreviousIe];
        assert_eq!(
            compute(&major_minor).unwrap().classification_basis,
            "one_major_and_one_minor_criterion"
        );

        let mut minor_three = empty();
        three_minor(&mut minor_three);
        assert_eq!(
            compute(&minor_three).unwrap().classification_basis,
            "three_minor_criteria"
        );
        assert_eq!(
            compute(&empty()).unwrap().classification_basis,
            "below_possible_clinical_criteria"
        );
    }

    #[test]
    fn table_one_rule_conformance_explicit_rejection_precedes_counts() {
        let mut input = empty();
        one_major(&mut input);
        three_minor(&mut input);
        input.rejection_evidence =
            vec![RejectionEvidence::NoRecurrenceAfterLessThanFourDaysAntibiotics];
        let outcome = compute(&input).unwrap();
        assert_eq!(outcome.result, "rejected");
        assert_eq!(outcome.classification_basis, "explicit_rejection_criteria");
    }

    #[test]
    fn table_one_rule_conformance_covers_both_pathologic_and_all_rejection_routes() {
        for pathologic_evidence in [
            PathologicEvidence::QualifyingMicroorganismIdentifiedWithClinicalSignsOfActiveIe,
            PathologicEvidence::ActiveEndocarditisIdentified,
        ] {
            let no_recurrence = DukeIscvidInput {
                pathologic_evidence,
                rejection_evidence: vec![
                    RejectionEvidence::NoRecurrenceAfterLessThanFourDaysAntibiotics,
                ],
                ..empty()
            };
            let outcome = compute(&no_recurrence).unwrap();
            assert_eq!(outcome.result, "definite");
            assert_eq!(outcome.classification_basis, "pathologic_criteria");
            assert!(
                outcome
                    .matched_evidence
                    .contains(&"explicit_rejection_evidence")
            );
            let response = build_response(&no_recurrence).unwrap();
            assert_eq!(
                response.working["rejection_evidence"],
                json!(["no_recurrence_after_less_than_four_days_antibiotics"])
            );

            for rejection_evidence in [
                RejectionEvidence::FirmAlternateMicrobiologicDiagnosisAllThreeConditions,
                RejectionEvidence::FirmAlternateNonmicrobiologicDiagnosisBothConditions,
                RejectionEvidence::NoPathologicOrMacroscopicIeAtSurgeryOrAutopsyWithLessThanFourDaysAntibiotics,
            ] {
                let input = DukeIscvidInput {
                    pathologic_evidence,
                    rejection_evidence: vec![rejection_evidence],
                    ..empty()
                };
                assert!(compute(&input).is_err());
            }
        }

        let firm_alternate_microbiologic = DukeIscvidInput {
            blood_culture_organism:
                BloodCultureOrganism::ClinicianClassifiedOrganismAsNotConsistentWithIe,
            positive_blood_culture_sets: 1,
            rejection_evidence: vec![
                RejectionEvidence::FirmAlternateMicrobiologicDiagnosisAllThreeConditions,
            ],
            ..empty()
        };
        assert_eq!(
            compute(&firm_alternate_microbiologic)
                .unwrap()
                .classification_basis,
            "explicit_rejection_criteria"
        );

        for rejection_evidence in [
            RejectionEvidence::FirmAlternateNonmicrobiologicDiagnosisBothConditions,
            RejectionEvidence::NoRecurrenceAfterLessThanFourDaysAntibiotics,
            RejectionEvidence::NoPathologicOrMacroscopicIeAtSurgeryOrAutopsyWithLessThanFourDaysAntibiotics,
        ] {
            let input = DukeIscvidInput {
                rejection_evidence: vec![rejection_evidence],
                ..empty()
            };
            assert_eq!(
                compute(&input).unwrap().classification_basis,
                "explicit_rejection_criteria"
            );
        }
    }

    #[test]
    fn table_two_rule_conformance_covers_each_laboratory_and_anatomic_major_source() {
        for major_laboratory_evidence in [
            MajorLaboratoryEvidence::BloodNucleicAcidCoxiellaBartonellaOrTropheryma,
            MajorLaboratoryEvidence::CoxiellaPhaseIIggGreaterThan1_800,
            MajorLaboratoryEvidence::CoxiellaSinglePositiveBloodCulture,
            MajorLaboratoryEvidence::BartonellaHenselaeOrQuintanaIggAtLeast1_800,
        ] {
            let input = DukeIscvidInput {
                major_laboratory_evidence,
                ..empty()
            };
            assert!(compute(&input).unwrap().major_domains.microbiologic);
        }

        for anatomic_imaging_evidence in [
            AnatomicImagingEvidence::VegetationPerforationAneurysmAbscessPseudoaneurysmOrFistula,
            AnatomicImagingEvidence::SignificantNewValvularRegurgitationOnEchocardiographyComparedWithPreviousImaging,
        ] {
            let input = DukeIscvidInput {
                anatomic_imaging_evidence,
                ..empty()
            };
            assert!(compute(&input).unwrap().major_domains.imaging);
        }
        let dehiscence = DukeIscvidInput {
            intracardiac_prosthetic_material: vec![IntracardiacProstheticMaterial::ProstheticValve],
            anatomic_imaging_evidence: AnatomicImagingEvidence::NewPartialProstheticValveDehiscence,
            ..empty()
        };
        assert!(compute(&dehiscence).unwrap().major_domains.imaging);
        let native_pet = DukeIscvidInput {
            pet_ct_evidence: PetCtEvidence::QualifyingNativeValveAbnormalUptake,
            ..empty()
        };
        assert!(compute(&native_pet).unwrap().major_domains.imaging);
        assert_eq!(
            serde_json::to_value(
                AnatomicImagingEvidence::SignificantNewValvularRegurgitationOnEchocardiographyComparedWithPreviousImaging
            )
            .unwrap(),
            json!("significant_new_valvular_regurgitation_on_echocardiography_compared_with_previous_imaging")
        );
    }

    #[test]
    fn table_two_rule_conformance_derives_all_major_and_minor_domains() {
        let mut input = empty();
        input.blood_culture_organism = BloodCultureOrganism::StaphylococcusAureus;
        input.positive_blood_culture_sets = 2;
        input.anatomic_imaging_evidence =
            AnatomicImagingEvidence::VegetationPerforationAneurysmAbscessPseudoaneurysmOrFistula;
        input.predisposition = vec![Predisposition::CongenitalHeartDisease];
        input.maximum_documented_temperature_c = Some(39.0);
        input.vascular_phenomena = vec![VascularPhenomenon::SplenicAbscess];
        input.immunologic_phenomena =
            vec![ImmunologicPhenomenon::SourceDefinedImmuneComplexGlomerulonephritis];
        input.intracardiac_prosthetic_material =
            vec![IntracardiacProstheticMaterial::ProstheticValve];
        input.pet_ct_evidence =
            PetCtEvidence::QualifyingImplantedIntracardiacMaterialUptakeLessThanThreeMonths;
        input.auscultation_evidence =
            AuscultationEvidence::NewValvularRegurgitationWithEchocardiographyUnavailable;
        let outcome = compute(&input).unwrap();
        assert_eq!(outcome.major_count, 2);
        assert_eq!(outcome.minor_count, 6);
        assert!(outcome.major_domains.microbiologic && outcome.major_domains.imaging);
        assert!(!outcome.minor_domains.microbiologic);
        assert!(outcome.minor_domains.predisposition && outcome.minor_domains.fever);
        assert!(outcome.minor_domains.vascular && outcome.minor_domains.immunologic);
        assert!(outcome.minor_domains.imaging && outcome.minor_domains.physical_exam);

        let mut surgical = empty();
        surgical.surgical_inspection_evidence = SurgicalInspectionEvidence::DirectEvidenceWithoutMajorImagingOrSubsequentPathologicConfirmation;
        assert!(compute(&surgical).unwrap().major_domains.surgical);
    }

    #[test]
    fn prosthetic_material_is_the_single_source_for_context_and_predisposition() {
        for material in [
            IntracardiacProstheticMaterial::ProstheticValve,
            IntracardiacProstheticMaterial::ValveRepairMaterial,
            IntracardiacProstheticMaterial::EndovascularCied,
        ] {
            let input = DukeIscvidInput {
                intracardiac_prosthetic_material: vec![material],
                ..empty()
            };
            assert!(compute(&input).unwrap().minor_domains.predisposition);
        }

        let mut other_material = DukeIscvidInput {
            intracardiac_prosthetic_material: vec![
                IntracardiacProstheticMaterial::OtherIntracardiacProstheticMaterial,
            ],
            blood_culture_organism: BloodCultureOrganism::CutibacteriumAcnes,
            positive_blood_culture_sets: 2,
            ..empty()
        };
        let outcome = compute(&other_material).unwrap();
        assert!(outcome.blood_culture_major);
        assert!(!outcome.minor_domains.predisposition);
        let response = build_response(&other_material).unwrap();
        assert_eq!(
            response.working["intracardiac_prosthetic_material"],
            json!(["other_intracardiac_prosthetic_material"])
        );
        assert_eq!(
            response.working["expanded_typical_organism_context"],
            json!(true)
        );

        other_material.intracardiac_prosthetic_material.clear();
        let outcome = compute(&other_material).unwrap();
        assert!(!outcome.blood_culture_major);
        assert!(outcome.minor_domains.microbiologic);
    }

    #[test]
    fn table_two_rule_conformance_blood_culture_thresholds_and_corrected_groups() {
        for organism in [
            BloodCultureOrganism::StaphylococcusAureus,
            BloodCultureOrganism::StaphylococcusLugdunensis,
            BloodCultureOrganism::EnterococcusFaecalis,
            BloodCultureOrganism::StreptococcusOtherThanPneumoniaeOrPyogenes,
            BloodCultureOrganism::Granulicatella,
            BloodCultureOrganism::Abiotrophia,
            BloodCultureOrganism::Gemella,
            BloodCultureOrganism::Hacek,
        ] {
            let mut input = empty();
            input.blood_culture_organism = organism;
            input.positive_blood_culture_sets = 1;
            let outcome = compute(&input).unwrap();
            assert!(!outcome.blood_culture_major);
            assert!(outcome.minor_domains.microbiologic);
            assert_eq!(outcome.blood_culture_group, "universal_typical");
            input.positive_blood_culture_sets = 2;
            assert!(compute(&input).unwrap().blood_culture_major);
        }
        for organism in [
            BloodCultureOrganism::CoagulaseNegativeStaphylococcusOtherThanLugdunensis,
            BloodCultureOrganism::CorynebacteriumStriatumOrJeikeium,
            BloodCultureOrganism::SerratiaMarcescens,
            BloodCultureOrganism::PseudomonasAeruginosa,
            BloodCultureOrganism::CutibacteriumAcnes,
            BloodCultureOrganism::NontuberculousMycobacterium,
            BloodCultureOrganism::Candida,
        ] {
            let mut input = empty();
            input.blood_culture_organism = organism;
            input.positive_blood_culture_sets = 1;
            let outcome = compute(&input).unwrap();
            assert!(!outcome.blood_culture_major);
            assert!(!outcome.minor_domains.microbiologic);
            assert_eq!(
                outcome.blood_culture_group,
                "prosthetic_material_only_typical"
            );
            input.positive_blood_culture_sets = 2;
            let outcome = compute(&input).unwrap();
            assert!(!outcome.blood_culture_major);
            assert!(outcome.minor_domains.microbiologic);
            input.positive_blood_culture_sets = 3;
            assert!(compute(&input).unwrap().blood_culture_major);

            input.intracardiac_prosthetic_material =
                vec![IntracardiacProstheticMaterial::ProstheticValve];
            input.positive_blood_culture_sets = 1;
            let outcome = compute(&input).unwrap();
            assert!(!outcome.blood_culture_major);
            assert!(!outcome.minor_domains.microbiologic);
            input.positive_blood_culture_sets = 2;
            assert!(compute(&input).unwrap().blood_culture_major);
        }

        let mut occasional = empty();
        occasional.blood_culture_organism = BloodCultureOrganism::ClinicianClassifiedOtherOrganismAsOccasionallyCausingIeAndNotACommonContaminant;
        for sets in [1, 2] {
            occasional.positive_blood_culture_sets = sets;
            let outcome = compute(&occasional).unwrap();
            assert!(!outcome.blood_culture_major);
            assert!(outcome.minor_domains.microbiologic);
            assert_eq!(
                outcome.blood_culture_group,
                "occasional_noncontaminant_nontypical"
            );
        }
        occasional.positive_blood_culture_sets = 3;
        assert!(compute(&occasional).unwrap().blood_culture_major);

        // Table 2 footnote r excludes a single set for rare organisms and
        // common contaminants, unlike occasional noncontaminant organisms.
        for organism in [
            BloodCultureOrganism::StreptococcusPneumoniaeOrPyogenes,
            BloodCultureOrganism::NonFaecalisEnterococcus,
            BloodCultureOrganism::ClinicianClassifiedOtherOrganismAsRarelyCausingIeOrACommonContaminant,
        ] {
            let mut input = empty();
            input.blood_culture_organism = organism;
            input.positive_blood_culture_sets = 1;
            let outcome = compute(&input).unwrap();
            assert!(!outcome.blood_culture_major);
            assert!(!outcome.minor_domains.microbiologic);
            assert_eq!(
                outcome.blood_culture_group,
                "rare_or_common_contaminant_nontypical"
            );
            input.positive_blood_culture_sets = 2;
            let outcome = compute(&input).unwrap();
            assert!(!outcome.blood_culture_major);
            assert!(outcome.minor_domains.microbiologic);
            input.positive_blood_culture_sets = 3;
            assert!(compute(&input).unwrap().blood_culture_major);
        }

        let mut not_consistent = empty();
        not_consistent.blood_culture_organism =
            BloodCultureOrganism::ClinicianClassifiedOrganismAsNotConsistentWithIe;
        for sets in [1, 2, 3, 10] {
            not_consistent.positive_blood_culture_sets = sets;
            let outcome = compute(&not_consistent).unwrap();
            assert!(!outcome.blood_culture_major);
            assert!(!outcome.minor_domains.microbiologic);
            assert_eq!(outcome.blood_culture_group, "not_consistent_with_ie");
        }
    }

    #[test]
    fn table_two_rule_conformance_microbiology_major_suppresses_minor() {
        let mut input = empty();
        input.blood_culture_organism = BloodCultureOrganism::StaphylococcusAureus;
        input.positive_blood_culture_sets = 1;
        assert!(compute(&input).unwrap().minor_domains.microbiologic);
        input.major_laboratory_evidence =
            MajorLaboratoryEvidence::CoxiellaPhaseIIggGreaterThan1_800;
        let outcome = compute(&input).unwrap();
        assert!(outcome.major_domains.microbiologic);
        assert!(!outcome.minor_domains.microbiologic);
    }

    #[test]
    fn table_two_rule_conformance_temperature_and_pet_boundaries_are_exact() {
        let mut input = empty();
        input.maximum_documented_temperature_c = Some(38.0);
        assert!(!compute(&input).unwrap().minor_domains.fever);
        input.maximum_documented_temperature_c = Some(38.0001);
        assert!(compute(&input).unwrap().minor_domains.fever);
        input.intracardiac_prosthetic_material =
            vec![IntracardiacProstheticMaterial::ProstheticValve];
        input.pet_ct_evidence =
            PetCtEvidence::QualifyingImplantedIntracardiacMaterialUptakeLessThanThreeMonths;
        assert!(compute(&input).unwrap().minor_domains.imaging);
        assert!(!compute(&input).unwrap().major_domains.imaging);
        input.pet_ct_evidence =
            PetCtEvidence::QualifyingImplantedIntracardiacMaterialUptakeAtLeastThreeMonths;
        assert!(compute(&input).unwrap().major_domains.imaging);
        assert!(!compute(&input).unwrap().minor_domains.imaging);
        input.pet_ct_evidence = PetCtEvidence::IsolatedGeneratorPocketUptake;
        input.intracardiac_prosthetic_material =
            vec![IntracardiacProstheticMaterial::EndovascularCied];
        let outcome = compute(&input).unwrap();
        assert!(!outcome.major_domains.imaging && !outcome.minor_domains.imaging);
    }

    #[test]
    fn pet_targets_require_their_asserted_material_without_inventing_graft_context() {
        for pet_ct_evidence in [
            PetCtEvidence::QualifyingImplantedIntracardiacMaterialUptakeAtLeastThreeMonths,
            PetCtEvidence::QualifyingImplantedIntracardiacMaterialUptakeLessThanThreeMonths,
        ] {
            assert!(
                compute(&DukeIscvidInput {
                    pet_ct_evidence,
                    ..empty()
                })
                .is_err()
            );
        }

        assert!(
            compute(&DukeIscvidInput {
                pet_ct_evidence: PetCtEvidence::IsolatedGeneratorPocketUptake,
                ..empty()
            })
            .is_err()
        );
        let pocket = DukeIscvidInput {
            intracardiac_prosthetic_material: vec![
                IntracardiacProstheticMaterial::EndovascularCied,
            ],
            pet_ct_evidence: PetCtEvidence::IsolatedGeneratorPocketUptake,
            ..empty()
        };
        let pocket_outcome = compute(&pocket).unwrap();
        assert!(!pocket_outcome.major_domains.imaging);
        assert!(!pocket_outcome.minor_domains.imaging);

        for (pet_ct_evidence, expected_major, expected_minor) in [
            (
                PetCtEvidence::QualifyingAscendingAorticGraftUptakeWithConcomitantValveInvolvementAtLeastThreeMonths,
                true,
                false,
            ),
            (
                PetCtEvidence::QualifyingAscendingAorticGraftUptakeWithConcomitantValveInvolvementLessThanThreeMonths,
                false,
                true,
            ),
        ] {
            let input = DukeIscvidInput {
                pet_ct_evidence,
                ..empty()
            };
            let outcome = compute(&input).unwrap();
            assert_eq!(outcome.major_domains.imaging, expected_major);
            assert_eq!(outcome.minor_domains.imaging, expected_minor);
            assert!(!outcome.minor_domains.predisposition);
            assert_eq!(
                build_response(&input).unwrap().working["expanded_typical_organism_context"],
                json!(false)
            );
        }
    }

    #[test]
    fn prosthetic_valve_dehiscence_requires_a_prosthetic_valve() {
        let without_valve = DukeIscvidInput {
            anatomic_imaging_evidence: AnatomicImagingEvidence::NewPartialProstheticValveDehiscence,
            ..empty()
        };
        assert!(compute(&without_valve).is_err());

        let with_valve = DukeIscvidInput {
            intracardiac_prosthetic_material: vec![IntracardiacProstheticMaterial::ProstheticValve],
            ..without_valve
        };
        assert!(compute(&with_valve).unwrap().major_domains.imaging);
    }

    #[test]
    fn firm_alternate_microbiology_requires_positive_contextually_nontypical_cultures() {
        let rejection_evidence =
            vec![RejectionEvidence::FirmAlternateMicrobiologicDiagnosisAllThreeConditions];
        assert!(
            compute(&DukeIscvidInput {
                rejection_evidence: rejection_evidence.clone(),
                ..empty()
            })
            .is_err()
        );
        assert!(
            compute(&DukeIscvidInput {
                blood_culture_organism: BloodCultureOrganism::StaphylococcusAureus,
                positive_blood_culture_sets: 1,
                rejection_evidence: rejection_evidence.clone(),
                ..empty()
            })
            .is_err()
        );

        let prosthetic_only_typical = DukeIscvidInput {
            blood_culture_organism: BloodCultureOrganism::CutibacteriumAcnes,
            positive_blood_culture_sets: 1,
            rejection_evidence: rejection_evidence.clone(),
            ..empty()
        };
        assert_eq!(
            compute(&prosthetic_only_typical).unwrap().result,
            "rejected"
        );
        assert!(
            compute(&DukeIscvidInput {
                intracardiac_prosthetic_material: vec![
                    IntracardiacProstheticMaterial::OtherIntracardiacProstheticMaterial,
                ],
                ..prosthetic_only_typical
            })
            .is_err()
        );

        let simultaneous = DukeIscvidInput {
            blood_culture_organism: BloodCultureOrganism::ClinicianClassifiedOtherOrganismAsOccasionallyCausingIeAndNotACommonContaminant,
            positive_blood_culture_sets: 1,
            rejection_evidence: vec![
                RejectionEvidence::FirmAlternateMicrobiologicDiagnosisAllThreeConditions,
                RejectionEvidence::FirmAlternateNonmicrobiologicDiagnosisBothConditions,
            ],
            ..empty()
        };
        assert_eq!(compute(&simultaneous).unwrap().result, "rejected");
        assert!(
            DukeIscvid
                .calculate(&serde_json::to_value(simultaneous).unwrap())
                .is_ok()
        );
    }

    #[test]
    fn table_two_rule_conformance_unsupported_skin_pcr_is_minor_only() {
        let mut input = empty();
        input.pathologic_evidence =
            PathologicEvidence::UnsupportedSingleSkinBacteriumPcrOnValveOrWire;
        let outcome = compute(&input).unwrap();
        assert!(!outcome.pathologic_definite);
        assert!(outcome.minor_domains.microbiologic);
    }

    #[test]
    fn table_two_rule_conformance_nonqualifying_evidence_does_not_create_domains() {
        let mut input = empty();
        input.blood_culture_organism =
            BloodCultureOrganism::ClinicianClassifiedOrganismAsNotConsistentWithIe;
        input.positive_blood_culture_sets = 5;
        input.auscultation_evidence =
            AuscultationEvidence::NewRegurgitationButEchocardiographyAvailable;
        let outcome = compute(&input).unwrap();
        assert!(!outcome.major_domains.microbiologic);
        assert!(!outcome.minor_domains.microbiologic);
        assert!(!outcome.minor_domains.physical_exam);

        input.auscultation_evidence = AuscultationEvidence::ChangedPreexistingMurmur;
        assert!(!compute(&input).unwrap().minor_domains.physical_exam);
        input.other_minor_microbiology =
            OtherMinorMicrobiology::SerumSequencingOtherThanCoxiellaBartonellaTropheryma;
        assert!(compute(&input).unwrap().minor_domains.microbiologic);
    }

    #[test]
    fn contradictory_surgical_pathologic_and_rejection_assertions_are_rejected() {
        let mut surgical_imaging = empty();
        surgical_imaging.surgical_inspection_evidence = SurgicalInspectionEvidence::DirectEvidenceWithoutMajorImagingOrSubsequentPathologicConfirmation;
        surgical_imaging.intracardiac_prosthetic_material =
            vec![IntracardiacProstheticMaterial::ProstheticValve];
        surgical_imaging.anatomic_imaging_evidence =
            AnatomicImagingEvidence::NewPartialProstheticValveDehiscence;
        assert!(compute(&surgical_imaging).is_err());

        let mut surgical_pathology = empty();
        surgical_pathology.surgical_inspection_evidence = SurgicalInspectionEvidence::DirectEvidenceWithoutMajorImagingOrSubsequentPathologicConfirmation;
        surgical_pathology.pathologic_evidence = PathologicEvidence::ActiveEndocarditisIdentified;
        assert!(compute(&surgical_pathology).is_err());

        let mut alternate_with_imaging = empty();
        alternate_with_imaging.rejection_evidence =
            vec![RejectionEvidence::FirmAlternateMicrobiologicDiagnosisAllThreeConditions];
        alternate_with_imaging.blood_culture_organism =
            BloodCultureOrganism::ClinicianClassifiedOrganismAsNotConsistentWithIe;
        alternate_with_imaging.positive_blood_culture_sets = 1;
        alternate_with_imaging.intracardiac_prosthetic_material =
            vec![IntracardiacProstheticMaterial::ProstheticValve];
        alternate_with_imaging.anatomic_imaging_evidence =
            AnatomicImagingEvidence::NewPartialProstheticValveDehiscence;
        assert!(compute(&alternate_with_imaging).is_err());

        let mut alternate_with_early_pet = empty();
        alternate_with_early_pet.rejection_evidence =
            vec![RejectionEvidence::FirmAlternateMicrobiologicDiagnosisAllThreeConditions];
        alternate_with_early_pet.blood_culture_organism =
            BloodCultureOrganism::ClinicianClassifiedOrganismAsNotConsistentWithIe;
        alternate_with_early_pet.positive_blood_culture_sets = 1;
        alternate_with_early_pet.intracardiac_prosthetic_material =
            vec![IntracardiacProstheticMaterial::ProstheticValve];
        alternate_with_early_pet.pet_ct_evidence =
            PetCtEvidence::QualifyingImplantedIntracardiacMaterialUptakeLessThanThreeMonths;
        assert!(compute(&alternate_with_early_pet).is_err());
    }

    #[test]
    fn invalid_counts_nonfinite_temperature_duplicates_and_unknown_fields_are_rejected() {
        let mut contradiction = empty();
        contradiction.positive_blood_culture_sets = 1;
        assert!(compute(&contradiction).is_err());
        contradiction.blood_culture_organism = BloodCultureOrganism::StaphylococcusAureus;
        contradiction.positive_blood_culture_sets = 0;
        assert!(compute(&contradiction).is_err());

        let mut temperature = empty();
        temperature.maximum_documented_temperature_c = Some(f64::NAN);
        assert!(compute(&temperature).is_err());
        temperature.maximum_documented_temperature_c = Some(46.0);
        assert!(compute(&temperature).is_ok());

        let mut duplicate = empty();
        duplicate.intracardiac_prosthetic_material = vec![
            IntracardiacProstheticMaterial::ProstheticValve,
            IntracardiacProstheticMaterial::ProstheticValve,
        ];
        assert!(compute(&duplicate).is_err());
        let mut duplicate = empty();
        duplicate.predisposition = vec![Predisposition::PreviousIe, Predisposition::PreviousIe];
        assert!(compute(&duplicate).is_err());
        let mut duplicate = empty();
        duplicate.vascular_phenomena = vec![
            VascularPhenomenon::JanewayLesion,
            VascularPhenomenon::JanewayLesion,
        ];
        assert!(compute(&duplicate).is_err());
        let mut duplicate = empty();
        duplicate.immunologic_phenomena = vec![
            ImmunologicPhenomenon::RothSpots,
            ImmunologicPhenomenon::RothSpots,
        ];
        assert!(compute(&duplicate).is_err());
        let mut duplicate = empty();
        duplicate.rejection_evidence = vec![
            RejectionEvidence::NoRecurrenceAfterLessThanFourDaysAntibiotics,
            RejectionEvidence::NoRecurrenceAfterLessThanFourDaysAntibiotics,
        ];
        assert!(compute(&duplicate).is_err());

        let mut value = serde_json::to_value(empty()).unwrap();
        value["major_count"] = json!(2);
        assert!(DukeIscvid.calculate(&value).is_err());
    }

    #[test]
    fn dynamic_surface_rejects_every_cross_field_contradiction() {
        fn assert_dynamic_invalid(input: DukeIscvidInput) {
            assert!(
                DukeIscvid
                    .calculate(&serde_json::to_value(input).unwrap())
                    .is_err()
            );
        }

        assert_dynamic_invalid(DukeIscvidInput {
            positive_blood_culture_sets: 1,
            ..empty()
        });
        assert_dynamic_invalid(DukeIscvidInput {
            blood_culture_organism: BloodCultureOrganism::StaphylococcusAureus,
            positive_blood_culture_sets: 0,
            ..empty()
        });
        assert_dynamic_invalid(DukeIscvidInput {
            intracardiac_prosthetic_material: vec![
                IntracardiacProstheticMaterial::ValveRepairMaterial,
            ],
            predisposition: vec![Predisposition::PreviousValveRepairWithoutCurrentMaterial],
            ..empty()
        });
        assert_dynamic_invalid(DukeIscvidInput {
            surgical_inspection_evidence: SurgicalInspectionEvidence::DirectEvidenceWithoutMajorImagingOrSubsequentPathologicConfirmation,
            pathologic_evidence: PathologicEvidence::ActiveEndocarditisIdentified,
            ..empty()
        });
        assert_dynamic_invalid(DukeIscvidInput {
            surgical_inspection_evidence: SurgicalInspectionEvidence::DirectEvidenceWithoutMajorImagingOrSubsequentPathologicConfirmation,
            pet_ct_evidence: PetCtEvidence::QualifyingNativeValveAbnormalUptake,
            ..empty()
        });
        let pathologic_with_no_recurrence = DukeIscvidInput {
            pathologic_evidence: PathologicEvidence::ActiveEndocarditisIdentified,
            rejection_evidence: vec![
                RejectionEvidence::NoRecurrenceAfterLessThanFourDaysAntibiotics,
            ],
            ..empty()
        };
        assert_eq!(
            DukeIscvid
                .calculate(&serde_json::to_value(pathologic_with_no_recurrence).unwrap())
                .unwrap()
                .result,
            json!("definite")
        );
        assert_dynamic_invalid(DukeIscvidInput {
            intracardiac_prosthetic_material: vec![IntracardiacProstheticMaterial::ProstheticValve],
            pet_ct_evidence:
                PetCtEvidence::QualifyingImplantedIntracardiacMaterialUptakeLessThanThreeMonths,
            blood_culture_organism:
                BloodCultureOrganism::ClinicianClassifiedOrganismAsNotConsistentWithIe,
            positive_blood_culture_sets: 1,
            rejection_evidence: vec![
                RejectionEvidence::FirmAlternateMicrobiologicDiagnosisAllThreeConditions,
            ],
            ..empty()
        });
        assert_dynamic_invalid(DukeIscvidInput {
            anatomic_imaging_evidence: AnatomicImagingEvidence::NewPartialProstheticValveDehiscence,
            ..empty()
        });
        assert_dynamic_invalid(DukeIscvidInput {
            pet_ct_evidence:
                PetCtEvidence::QualifyingImplantedIntracardiacMaterialUptakeAtLeastThreeMonths,
            ..empty()
        });
        assert_dynamic_invalid(DukeIscvidInput {
            pet_ct_evidence: PetCtEvidence::IsolatedGeneratorPocketUptake,
            ..empty()
        });
        assert_dynamic_invalid(DukeIscvidInput {
            blood_culture_organism: BloodCultureOrganism::StaphylococcusAureus,
            positive_blood_culture_sets: 1,
            rejection_evidence: vec![
                RejectionEvidence::FirmAlternateNonmicrobiologicDiagnosisBothConditions,
            ],
            ..empty()
        });
        assert_dynamic_invalid(DukeIscvidInput {
            surgical_inspection_evidence: SurgicalInspectionEvidence::DirectEvidenceWithoutMajorImagingOrSubsequentPathologicConfirmation,
            rejection_evidence: vec![RejectionEvidence::NoPathologicOrMacroscopicIeAtSurgeryOrAutopsyWithLessThanFourDaysAntibiotics],
            ..empty()
        });
        assert_dynamic_invalid(DukeIscvidInput {
            pathologic_evidence: PathologicEvidence::ActiveEndocarditisIdentified,
            rejection_evidence: vec![RejectionEvidence::NoPathologicOrMacroscopicIeAtSurgeryOrAutopsyWithLessThanFourDaysAntibiotics],
            ..empty()
        });
    }

    #[test]
    fn schema_is_closed_complete_and_preserves_corrected_source_definitions() {
        let schema = DukeIscvid.input_schema();
        assert_eq!(schema["additionalProperties"], json!(false));
        assert_eq!(schema["required"].as_array().unwrap().len(), 16);
        assert_eq!(schema["properties"].as_object().unwrap().len(), 16);
        for property in schema["properties"].as_object().unwrap().values() {
            assert!(property.get("definition").is_some());
        }
        for name in [
            "intracardiac_prosthetic_material",
            "predisposition",
            "vascular_phenomena",
            "immunologic_phenomena",
            "rejection_evidence",
        ] {
            assert_eq!(schema["properties"][name]["uniqueItems"], json!(true));
        }
        assert!(
            schema["description"]
                .as_str()
                .unwrap()
                .contains("corrected publication")
        );
        assert!(
            schema["properties"]["immunologic_phenomena"]["definition"]["caveats"]
                .as_str()
                .unwrap()
                .contains("at least 2")
        );
        assert!(
            schema["properties"]["major_laboratory_evidence"]["definition"]["caveats"]
                .as_str()
                .unwrap()
                .contains("strictly greater")
        );
        assert!(
            schema["properties"]["pet_ct_evidence"]["definition"]["caveats"]
                .as_str()
                .unwrap()
                .contains("Isolated generator-pocket")
        );
        assert_eq!(schema["allOf"].as_array().unwrap().len(), 10);
        let blood_organisms = schema["properties"]["blood_culture_organism"]["enum"]
            .as_array()
            .unwrap();
        for category in [
            "clinician_classified_other_organism_as_occasionally_causing_ie_and_not_a_common_contaminant",
            "clinician_classified_other_organism_as_rarely_causing_ie_or_a_common_contaminant",
            "clinician_classified_organism_as_not_consistent_with_ie",
        ] {
            assert!(blood_organisms.contains(&json!(category)));
        }
        let pet_values = schema["properties"]["pet_ct_evidence"]["enum"]
            .as_array()
            .unwrap();
        for pet_value in [
            "qualifying_implanted_intracardiac_material_uptake_at_least_three_months",
            "qualifying_implanted_intracardiac_material_uptake_less_than_three_months",
            "qualifying_ascending_aortic_graft_uptake_with_concomitant_valve_involvement_at_least_three_months",
            "qualifying_ascending_aortic_graft_uptake_with_concomitant_valve_involvement_less_than_three_months",
        ] {
            assert!(pet_values.contains(&json!(pet_value)));
        }
        let organism_definition =
            serde_json::to_string(&schema["properties"]["blood_culture_organism"]["definition"])
                .unwrap();
        for required_attestation_term in [
            "clinician/laboratory attestation",
            "patient",
            "LLM",
            "non-specialist",
        ] {
            assert!(organism_definition.contains(required_attestation_term));
        }
        let pathologic_rejection_exclusions = schema["allOf"][3]["then"]["properties"]
            ["rejection_evidence"]["not"]["contains"]["enum"]
            .as_array()
            .unwrap();
        assert_eq!(pathologic_rejection_exclusions.len(), 3);
        assert!(!pathologic_rejection_exclusions.contains(&json!(
            "no_recurrence_after_less_than_four_days_antibiotics"
        )));
        assert_eq!(
            schema["allOf"][4]["then"]["properties"]["anatomic_imaging_evidence"]["const"],
            json!("none_or_nonqualifying")
        );
        assert_eq!(
            schema["allOf"][4]["then"]["properties"]["pet_ct_evidence"]["enum"],
            json!(["none_or_nonqualifying", "isolated_generator_pocket_uptake"])
        );
        assert_eq!(
            schema["allOf"][5]["then"]["properties"]["major_laboratory_evidence"]["const"],
            json!("none")
        );
        assert_eq!(
            schema["allOf"][6]["then"]["properties"]["surgical_inspection_evidence"]["const"],
            json!("none_or_nonqualifying")
        );
        assert!(
            schema["properties"]["maximum_documented_temperature_c"]
                .get("minimum")
                .is_none()
        );
        assert!(
            schema["properties"]["maximum_documented_temperature_c"]
                .get("maximum")
                .is_none()
        );
        assert!(
            schema["properties"]["anatomic_imaging_evidence"]["enum"]
                .as_array()
                .unwrap()
                .contains(&json!("significant_new_valvular_regurgitation_on_echocardiography_compared_with_previous_imaging"))
        );
        let conditions = serde_json::to_string(&schema["allOf"]).unwrap();
        for required_condition in [
            "previous_valve_repair_without_current_material",
            "valve_repair_material",
            "direct_evidence_without_major_imaging_or_subsequent_pathologic_confirmation",
            "qualifying_microorganism_identified_with_clinical_signs_of_active_ie",
            "firm_alternate_microbiologic_diagnosis_all_three_conditions",
            "qualifying_implanted_intracardiac_material_uptake_less_than_three_months",
            "qualifying_implanted_intracardiac_material_uptake_at_least_three_months",
            "new_partial_prosthetic_valve_dehiscence",
            "prosthetic_valve",
            "isolated_generator_pocket_uptake",
            "endovascular_cied",
            "firm_alternate_nonmicrobiologic_diagnosis_both_conditions",
            "clinician_classified_organism_as_not_consistent_with_ie",
            "no_pathologic_or_macroscopic_ie_at_surgery_or_autopsy_with_less_than_four_days_antibiotics",
        ] {
            assert!(conditions.contains(required_condition));
        }
        assert!(LICENSE.license.contains("all-rights-reserved"));
        assert!(LICENSE.license.contains("not redistributed"));
    }

    #[test]
    fn response_exposes_provenance_domains_limitations_and_no_treatment_advice() {
        let response = build_response(&empty()).unwrap();
        assert_eq!(response.result, json!("rejected"));
        assert_eq!(response.working["criteria_version"], json!(VERSION));
        assert!(response.working["major_domains"].is_object());
        assert!(response.working["minor_domains"].is_object());
        assert!(response.reference.contains("ciad510"));
        assert!(
            response
                .interpretation
                .contains("does not independently exclude")
        );
        assert!(response.interpretation.contains("research case-definition"));
        assert!(response.interpretation.contains("Advanced molecular tests"));
        assert!(
            !response
                .interpretation
                .to_lowercase()
                .contains("start antibiotics")
        );
        assert!(
            !response
                .interpretation
                .to_lowercase()
                .contains("treat with")
        );
    }

    #[test]
    fn committed_example_deserializes_and_classifies_as_possible() {
        let input: DukeIscvidInput =
            serde_json::from_str(include_str!("../../examples/duke-iscvid.json")).unwrap();
        let response = build_response(&input).unwrap();
        assert_eq!(response.result, json!("possible"));
        assert_eq!(response.working["major_count"], json!(1));
        assert_eq!(response.working["minor_count"], json!(1));
    }
}
