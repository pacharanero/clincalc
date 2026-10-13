// SPDX-FileCopyrightText: 2017 ClinRisk Ltd.
// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! QDiabetes-2018 - 10-year risk of developing type 2 diabetes (Hippisley-Cox &
//! Coupland, BMJ 2017).
//!
//! NICE recommends a validated computer-based risk-assessment tool such as
//! QDiabetes to identify patients at high risk of type 2 diabetes from
//! routinely available primary-care data. Like QRISK3, it is a sex-stratified
//! Cox fractional-polynomial model, but it additionally offers three nested
//! model variants of increasing precision:
//! - Model A: demographics, BMI, and clinical history only.
//! - Model B: Model A plus a fasting blood glucose reading.
//! - Model C: Model A plus an HbA1c reading.
//!
//! Models B and C are mutually exclusive refinements of A, not combinable (the
//! original algorithm has no model using both a glucose and an HbA1c term).
//! This implementation selects the model automatically from which optional
//! reading, if any, is supplied.
//!
//! Coefficients are transcribed verbatim from ClinRisk's open-source reference
//! C implementation (Copyright 2017 ClinRisk Ltd., released under the AGPL v3+
//! to enable faithful reimplementation) and validated against an independent
//! from-source re-implementation (see the tests below).
//!
//! As with QRISK3, BMI is derived from height and weight rather than accepted
//! directly, ethnicity and smoking are modelled as enums carrying their own
//! coefficients and age interactions, and two fields (polycystic ovary
//! syndrome and gestational diabetes) apply to the female model only and are
//! ignored for male patients.
//!
//! `qdiabetes.org`'s current website terms forbid using *the website* to
//! develop or test software; that is a usage restriction on the hosted
//! service, separate from the AGPL licence ClinRisk granted over the
//! algorithm's reference source code specifically so it could be faithfully
//! reimplemented elsewhere (the same pattern already relied on for
//! [`crate::calculators::qrisk3`]). The reference source is no longer hosted
//! live; it is cited here from a Wayback Machine capture.
//!
//! Per ClinRisk's licence terms, the official disclaimer ([`DISCLAIMER`]) is
//! carried in every response: an inaccurate implementation could lead to wrong
//! treatment, so the score must be checked against the original at
//! qdiabetes.org.

// The coefficients below carry more decimal places than an f64 can represent;
// they are kept exactly as ClinRisk published them so the transcription is
// verifiable digit-for-digit against the source. Rust rounds each to the
// nearest f64 just as the original C compiler does, so the extra digits are
// faithful, not misleading - hence the allow rather than truncating the
// published values.
#![allow(clippy::excessive_precision)]

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

/// Machine name.
pub const NAME: &str = "qdiabetes";

/// Primary citation.
pub const REFERENCE: &str = "Hippisley-Cox J, Coupland C. Development and validation of QDiabetes-2018 \
risk prediction algorithm to estimate future risk of type 2 diabetes: cohort study. BMJ. \
2017;359:j5019. doi:10.1136/bmj.j5019.";

/// Distribution licence: ClinRisk Ltd. released the QDiabetes-2018 algorithm
/// source under the AGPL v3+ specifically to enable faithful reimplementation;
/// the coefficients here are transcribed verbatim from that source. The
/// original `qdiabetes.org/2018/src.php` is no longer hosted live; this cites
/// an Internet Archive capture of it instead.
pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "AGPL-3.0-or-later - QDiabetes-2018 algorithm Copyright 2017 ClinRisk Ltd.",
    "https://web.archive.org/web/20220526155433id_/https://qdiabetes.org/2018/src.php",
);

/// ClinRisk's required disclaimer, carried alongside every score per the
/// licence terms. Inaccurate implementations can lead to wrong treatment, so
/// the result must be checked against the original algorithm at qdiabetes.org.
pub const DISCLAIMER: &str = "The initial version of this file, to be found at http://qdiabetes.org, \
faithfully implements QDiabetes-2018. ClinRisk Ltd. have released this code under the GNU Affero \
General Public License to enable others to implement the algorithm faithfully. However, the nature \
of the GNU Affero General Public License is such that we cannot prevent, for example, someone \
accidentally altering the coefficients, getting the inputs wrong, or just poor programming. ClinRisk \
Ltd. stress, therefore, that it is the responsibility of the end user to check that the source that \
they receive produces the same results as the original code found at http://qdiabetes.org. Inaccurate \
implementations of risk scores can lead to wrong patients being given the wrong treatment.";

/// Biological sex, selecting the male or female equation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sex {
    Male,
    Female,
}

/// Self-reported ethnicity, in the nine QDiabetes categories (identical to
/// QRISK3's). Each carries its own coefficient; "white or not stated" is the
/// reference (coefficient 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ethnicity {
    /// White or not stated (reference category).
    WhiteOrNotStated,
    Indian,
    Pakistani,
    Bangladeshi,
    OtherAsian,
    BlackCaribbean,
    BlackAfrican,
    Chinese,
    OtherEthnicGroup,
}

impl Ethnicity {
    /// 1-based index into the model's `Iethrisk` array (matching the published
    /// source, where index 1 = white/not stated maps to coefficient 0).
    fn index(self) -> usize {
        match self {
            Ethnicity::WhiteOrNotStated => 1,
            Ethnicity::Indian => 2,
            Ethnicity::Pakistani => 3,
            Ethnicity::Bangladeshi => 4,
            Ethnicity::OtherAsian => 5,
            Ethnicity::BlackCaribbean => 6,
            Ethnicity::BlackAfrican => 7,
            Ethnicity::Chinese => 8,
            Ethnicity::OtherEthnicGroup => 9,
        }
    }
}

/// Smoking status, in the five QDiabetes categories (identical to QRISK3's).
/// Carries both a coefficient and age interactions; "non-smoker" is the
/// reference (coefficient 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Smoking {
    NonSmoker,
    ExSmoker,
    /// Light smoker (fewer than 10 a day).
    LightSmoker,
    /// Moderate smoker (10 to 19 a day).
    ModerateSmoker,
    /// Heavy smoker (20 or more a day).
    HeavySmoker,
}

impl Smoking {
    /// 0-based smoking category, matching the published source's `Ismoke`
    /// array (0 = non-smoker = reference).
    fn cat(self) -> usize {
        match self {
            Smoking::NonSmoker => 0,
            Smoking::ExSmoker => 1,
            Smoking::LightSmoker => 2,
            Smoking::ModerateSmoker => 3,
            Smoking::HeavySmoker => 4,
        }
    }
}

/// Which of the three nested QDiabetes-2018 models was used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Model {
    /// Demographics, BMI, and clinical history only.
    A,
    /// Model A plus a fasting blood glucose reading.
    B,
    /// Model A plus an HbA1c reading.
    C,
}

/// QDiabetes-2018 inputs. BMI is derived from height and weight. Supplying
/// `fasting_glucose_mmol_l` selects model B; supplying `hba1c_mmol_mol`
/// selects model C; supplying neither uses model A. The two are mutually
/// exclusive, matching the original algorithm (there is no combined model).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QdiabetesInput {
    /// Age in years. QDiabetes-2018 is validated for ages 25-84.
    pub age: u8,
    pub sex: Sex,
    pub ethnicity: Ethnicity,
    pub smoking: Smoking,
    /// Height in centimetres (for BMI).
    pub height_cm: f64,
    /// Weight in kilograms (for BMI).
    pub weight_kg: f64,
    /// Townsend deprivation score (area-based; negative = less deprived).
    pub townsend: f64,
    /// Immediate family (mother, father, brother, or sister) with diabetes.
    pub family_history_diabetes: bool,
    /// Heart attack, angina, stroke, or TIA.
    pub cardiovascular_disease: bool,
    /// High blood pressure requiring treatment.
    pub treated_hypertension: bool,
    /// Learning disabilities.
    pub learning_disability: bool,
    /// Manic depression (bipolar disorder) or schizophrenia. Narrower than
    /// QRISK3's `severe_mental_illness`, which also includes severe
    /// depression; QDiabetes-2018 does not carry a depression term.
    pub bipolar_or_schizophrenia: bool,
    /// On regular steroid tablets.
    pub regular_steroids: bool,
    /// On statins.
    pub statin: bool,
    /// On atypical antipsychotic medication.
    pub atypical_antipsychotic: bool,
    /// Polycystic ovary syndrome (female model only; ignored for males).
    pub polycystic_ovary_syndrome: bool,
    /// Gestational diabetes, i.e. diabetes that arose during a pregnancy
    /// (female model only; ignored for males).
    pub gestational_diabetes: bool,
    /// Fasting blood glucose, mmol/l (2.0-6.99). Selects model B. Mutually
    /// exclusive with `hba1c_mmol_mol`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fasting_glucose_mmol_l: Option<f64>,
    /// HbA1c, mmol/mol (15.0-47.99). Selects model C. Mutually exclusive with
    /// `fasting_glucose_mmol_l`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hba1c_mmol_mol: Option<f64>,
}

/// The computed outcome.
#[derive(Debug, Clone, PartialEq)]
pub struct QdiabetesOutcome {
    /// 10-year type 2 diabetes risk, as a percentage rounded to one decimal
    /// place.
    pub risk_percent: f64,
    /// The derived BMI (kg/m^2) used in the model.
    pub bmi: f64,
    /// Which nested model (A, B, or C) was used.
    pub model: Model,
    pub interpretation: String,
}

fn b(flag: bool) -> f64 {
    if flag { 1.0 } else { 0.0 }
}

/// Model A, female. Coefficients transcribed verbatim from ClinRisk's
/// reference C source.
fn female_a_raw(i: &QdiabetesInput, bmi: f64) -> f64 {
    let survivor10 = 0.986227273941040_f64;

    let iethrisk = [
        0.0,
        0.0,
        1.0695857881565456000000000,
        1.3430172097414006000000000,
        1.8029022579794518000000000,
        1.1274654517708020000000000,
        0.4214631490239910100000000,
        0.2850919645908353000000000,
        0.8815108797589199500000000,
        0.3660573343168487300000000,
    ];
    let ismoke = [
        0.0,
        0.0656016901750590550000000,
        0.2845098867369837400000000,
        0.3567664381700702000000000,
        0.5359517110678775300000000,
    ];

    let dage = i.age as f64 / 10.0;
    let mut age_2 = dage.powf(3.0);
    let mut age_1 = dage.powf(0.5);
    let dbmi = bmi / 10.0;
    let mut bmi_1 = dbmi;
    let mut bmi_2 = dbmi.powf(3.0);

    age_1 -= 2.123332023620606;
    age_2 -= 91.644744873046875;
    bmi_1 -= 2.571253299713135;
    bmi_2 -= 16.999439239501953;
    let town = i.townsend - 0.391116052865982;

    let mut a = 0.0;
    a += iethrisk[i.ethnicity.index()];
    a += ismoke[i.smoking.cat()];

    a += age_1 * 4.3400852699139278000000000;
    a += age_2 * -0.0048771702696158879000000;
    a += bmi_1 * 2.9320361259524925000000000;
    a += bmi_2 * -0.0474002058748434900000000;
    a += town * 0.0373405696180491510000000;

    a += b(i.atypical_antipsychotic) * 0.5526764611098438100000000;
    a += b(i.regular_steroids) * 0.2679223368067459900000000;
    a += b(i.cardiovascular_disease) * 0.1779722905458669100000000;
    a += b(i.gestational_diabetes) * 1.5248871531467574000000000;
    a += b(i.learning_disability) * 0.2783514358717271700000000;
    a += b(i.bipolar_or_schizophrenia) * 0.2618085210917905900000000;
    a += b(i.polycystic_ovary_syndrome) * 0.3406173988206666100000000;
    a += b(i.statin) * 0.6590728773280821700000000;
    a += b(i.treated_hypertension) * 0.4394758285813711900000000;
    a += b(i.family_history_diabetes) * 0.5313359456558733900000000;

    a += age_1 * b(i.atypical_antipsychotic) * -0.8031518398316395100000000;
    a += age_1 * b(i.learning_disability) * -0.8641596002882057100000000;
    a += age_1 * b(i.statin) * -1.9757776696583935000000000;
    a += age_1 * bmi_1 * 0.6553138757562945200000000;
    a += age_1 * bmi_2 * -0.0362096572016301770000000;
    a += age_1 * b(i.family_history_diabetes) * -0.2641171450558896200000000;
    a += age_2 * b(i.atypical_antipsychotic) * 0.0004684041181021049800000;
    a += age_2 * b(i.learning_disability) * 0.0006724968808953360200000;
    a += age_2 * b(i.statin) * 0.0023750534194347966000000;
    a += age_2 * bmi_1 * -0.0044719662445263054000000;
    a += age_2 * bmi_2 * 0.0001185479967753342000000;
    a += age_2 * b(i.family_history_diabetes) * 0.0004161025828904768300000;

    100.0 * (1.0 - survivor10.powf(a.exp()))
}

/// Model B, female (adds fasting blood glucose).
fn female_b_raw(i: &QdiabetesInput, bmi: f64, fbs: f64) -> f64 {
    let survivor10 = 0.990905702114105_f64;

    let iethrisk = [
        0.0,
        0.0,
        0.9898906127239111000000000,
        1.2511504196326508000000000,
        1.4934757568196120000000000,
        0.9673887434565966400000000,
        0.4844644519593178100000000,
        0.4784214955360102700000000,
        0.7520946270805577400000000,
        0.4050880741541424400000000,
    ];
    let ismoke = [
        0.0,
        0.0374156307236963230000000,
        0.2252973672514482800000000,
        0.3099736428023662800000000,
        0.4361942139496417500000000,
    ];

    let dage = i.age as f64 / 10.0;
    let mut age_1 = dage.powf(0.5);
    let mut age_2 = dage.powf(3.0);
    let dbmi = bmi / 10.0;
    let mut bmi_2 = dbmi.powf(3.0);
    let mut bmi_1 = dbmi;
    let mut fbs_2 = fbs.powf(-1.0) * fbs.ln();
    let mut fbs_1 = fbs.powf(-1.0);

    age_1 -= 2.123332023620606;
    age_2 -= 91.644744873046875;
    bmi_1 -= 2.571253299713135;
    bmi_2 -= 16.999439239501953;
    fbs_1 -= 0.208309367299080;
    fbs_2 -= 0.326781362295151;
    let town = i.townsend - 0.391116052865982;

    let mut a = 0.0;
    a += iethrisk[i.ethnicity.index()];
    a += ismoke[i.smoking.cat()];

    a += age_1 * 3.7650129507517280000000000;
    a += age_2 * -0.0056043343436614941000000;
    a += bmi_1 * 2.4410935031672469000000000;
    a += bmi_2 * -0.0421526334799096420000000;
    a += fbs_1 * -2.1887891946337308000000000;
    a += fbs_2 * -69.9608419828660290000000000;
    a += town * 0.0358046297663126500000000;

    a += b(i.atypical_antipsychotic) * 0.4748378550253853400000000;
    a += b(i.regular_steroids) * 0.3767933443754728500000000;
    a += b(i.cardiovascular_disease) * 0.1967261568066525100000000;
    a += b(i.gestational_diabetes) * 1.0689325033692647000000000;
    a += b(i.learning_disability) * 0.4542293408951034700000000;
    a += b(i.bipolar_or_schizophrenia) * 0.1616171889084260500000000;
    a += b(i.polycystic_ovary_syndrome) * 0.3565365789576717100000000;
    a += b(i.statin) * 0.5809287382718667500000000;
    a += b(i.treated_hypertension) * 0.2836632020122907300000000;
    a += b(i.family_history_diabetes) * 0.4522149766206111600000000;

    a += age_1 * b(i.atypical_antipsychotic) * -0.7683591642786522500000000;
    a += age_1 * b(i.learning_disability) * -0.7983128124297588200000000;
    a += age_1 * b(i.statin) * -1.9033508839833257000000000;
    a += age_1 * bmi_1 * 0.4844747602404915200000000;
    a += age_1 * bmi_2 * -0.0319399883071813450000000;
    a += age_1 * fbs_1 * 2.2442903047404350000000000;
    a += age_1 * fbs_2 * 13.0068388699783030000000000;
    a += age_1 * b(i.family_history_diabetes) * -0.3040627374034501300000000;
    a += age_2 * b(i.atypical_antipsychotic) * 0.0005194455624413476200000;
    a += age_2 * b(i.learning_disability) * 0.0003028327567161890600000;
    a += age_2 * b(i.statin) * 0.0024397111406018711000000;
    a += age_2 * bmi_1 * -0.0041572976682154057000000;
    a += age_2 * bmi_2 * 0.0001126882194204252200000;
    a += age_2 * fbs_1 * 0.0199345308534312550000000;
    a += age_2 * fbs_2 * -0.0716677187529306680000000;
    a += age_2 * b(i.family_history_diabetes) * 0.0004523639671202325400000;

    100.0 * (1.0 - survivor10.powf(a.exp()))
}

/// Model C, female (adds HbA1c).
fn female_c_raw(i: &QdiabetesInput, bmi: f64, hba1c: f64) -> f64 {
    let survivor10 = 0.988788545131683_f64;

    let iethrisk = [
        0.0,
        0.0,
        0.5990951599291540800000000,
        0.7832030965635389300000000,
        1.1947351247960103000000000,
        0.7141744699168143300000000,
        0.1195328468388768800000000,
        0.0136688728784904270000000,
        0.5709226537693945500000000,
        0.1709107628106929200000000,
    ];
    let ismoke = [
        0.0,
        0.0658482585100006730000000,
        0.1458413689734224000000000,
        0.1525864247480118700000000,
        0.3078741679661397600000000,
    ];

    let dage = i.age as f64 / 10.0;
    let mut age_1 = dage.powf(0.5);
    let mut age_2 = dage.powf(3.0);
    let dbmi = bmi / 10.0;
    let mut bmi_2 = dbmi.powf(3.0);
    let mut bmi_1 = dbmi;
    let dhba1c = hba1c / 10.0;
    let mut hba1c_1 = dhba1c.powf(0.5);
    let mut hba1c_2 = dhba1c;

    age_1 -= 2.123332023620606;
    age_2 -= 91.644744873046875;
    bmi_1 -= 2.571253299713135;
    bmi_2 -= 16.999439239501953;
    hba1c_1 -= 1.886751174926758;
    hba1c_2 -= 3.559829950332642;
    let town = i.townsend - 0.391116052865982;

    let mut a = 0.0;
    a += iethrisk[i.ethnicity.index()];
    a += ismoke[i.smoking.cat()];

    a += age_1 * 3.5655214891947722000000000;
    a += age_2 * -0.0056158243572733135000000;
    a += bmi_1 * 2.5043028874544841000000000;
    a += bmi_2 * -0.0428758018926904610000000;
    a += hba1c_1 * 8.7368031307362184000000000;
    a += hba1c_2 * -0.0782313866699499700000000;
    a += town * 0.0358668220563482940000000;

    a += b(i.atypical_antipsychotic) * 0.5497633311042200400000000;
    a += b(i.regular_steroids) * 0.1687220550638970400000000;
    a += b(i.cardiovascular_disease) * 0.1644330036273934400000000;
    a += b(i.gestational_diabetes) * 1.1250098105171140000000000;
    a += b(i.learning_disability) * 0.2891205831073965800000000;
    a += b(i.bipolar_or_schizophrenia) * 0.3182512249068407700000000;
    a += b(i.polycystic_ovary_syndrome) * 0.3380644414098174500000000;
    a += b(i.statin) * 0.4559396847381116400000000;
    a += b(i.treated_hypertension) * 0.4040022295023758000000000;
    a += b(i.family_history_diabetes) * 0.4428015404826031700000000;

    a += age_1 * b(i.atypical_antipsychotic) * -0.8125434197162131300000000;
    a += age_1 * b(i.learning_disability) * -0.9084665765269808200000000;
    a += age_1 * b(i.statin) * -1.8557960585560658000000000;
    a += age_1 * bmi_1 * 0.6023218765235252000000000;
    a += age_1 * bmi_2 * -0.0344950383968044700000000;
    a += age_1 * b(i.family_history_diabetes) * -0.2727571351506187200000000;
    a += age_1 * hba1c_1 * 25.4412033227367150000000000;
    a += age_1 * hba1c_2 * -6.8076080421556107000000000;
    a += age_2 * b(i.atypical_antipsychotic) * 0.0004665611306005428000000;
    a += age_2 * b(i.learning_disability) * 0.0008518980139928006500000;
    a += age_2 * b(i.statin) * 0.0022627250963352537000000;
    a += age_2 * bmi_1 * -0.0043386645663133425000000;
    a += age_2 * bmi_2 * 0.0001162778561671208900000;
    a += age_2 * b(i.family_history_diabetes) * 0.0004354519795220774900000;
    a += age_2 * hba1c_1 * -0.0522541355885925220000000;
    a += age_2 * hba1c_2 * 0.0140548259061144530000000;

    100.0 * (1.0 - survivor10.powf(a.exp()))
}

/// Model A, male.
fn male_a_raw(i: &QdiabetesInput, bmi: f64) -> f64 {
    let survivor10 = 0.978732228279114_f64;

    let iethrisk = [
        0.0,
        0.0,
        1.1000230829124793000000000,
        1.2903840126147210000000000,
        1.6740908848727458000000000,
        1.1400446789147816000000000,
        0.4682468169065580600000000,
        0.6990564996301544800000000,
        0.6894365712711156800000000,
        0.4172222846773820900000000,
    ];
    let ismoke = [
        0.0,
        0.1638740910548557300000000,
        0.3185144911395897900000000,
        0.3220726656778343200000000,
        0.4505243716340953100000000,
    ];

    let dage = i.age as f64 / 10.0;
    let mut age_2 = dage.powf(3.0);
    let mut age_1 = dage.ln();
    let dbmi = bmi / 10.0;
    let mut bmi_2 = dbmi.powf(3.0);
    let mut bmi_1 = dbmi.powf(2.0);

    age_1 -= 1.496392488479614;
    age_2 -= 89.048171997070313;
    bmi_1 -= 6.817805767059326;
    bmi_2 -= 17.801923751831055;
    let town = i.townsend - 0.515986680984497;

    let mut a = 0.0;
    a += iethrisk[i.ethnicity.index()];
    a += ismoke[i.smoking.cat()];

    a += age_1 * 4.4642324388691348000000000;
    a += age_2 * -0.0040750108019255568000000;
    a += bmi_1 * 0.9512902786712067500000000;
    a += bmi_2 * -0.1435248827788547500000000;
    a += town * 0.0259181820676787250000000;

    a += b(i.atypical_antipsychotic) * 0.4210109234600543600000000;
    a += b(i.regular_steroids) * 0.2218358093292538400000000;
    a += b(i.cardiovascular_disease) * 0.2026960575629002100000000;
    a += b(i.learning_disability) * 0.2331532140798696100000000;
    a += b(i.bipolar_or_schizophrenia) * 0.2277044952051772700000000;
    a += b(i.statin) * 0.5849007543114134200000000;
    a += b(i.treated_hypertension) * 0.3337939218350107800000000;
    a += b(i.family_history_diabetes) * 0.6479928489936953600000000;

    a += age_1 * b(i.atypical_antipsychotic) * -0.9463772226853415200000000;
    a += age_1 * b(i.learning_disability) * -0.9384237552649983300000000;
    a += age_1 * b(i.statin) * -1.7479070653003299000000000;
    a += age_1 * bmi_1 * 0.4514759924187976600000000;
    a += age_1 * bmi_2 * -0.1079548126277638100000000;
    a += age_1 * b(i.family_history_diabetes) * -0.6011853042930119800000000;
    a += age_2 * b(i.atypical_antipsychotic) * -0.0000519927442172335000000;
    a += age_2 * b(i.learning_disability) * 0.0007102643855968814100000;
    a += age_2 * b(i.statin) * 0.0013508364599531669000000;
    a += age_2 * bmi_1 * -0.0011797722394560309000000;
    a += age_2 * bmi_2 * 0.0002147150913931929100000;
    a += age_2 * b(i.family_history_diabetes) * 0.0004914185594087803400000;

    100.0 * (1.0 - survivor10.powf(a.exp()))
}

/// Model B, male (adds fasting blood glucose).
fn male_b_raw(i: &QdiabetesInput, bmi: f64, fbs: f64) -> f64 {
    let survivor10 = 0.985019445419312_f64;

    let iethrisk = [
        0.0,
        0.0,
        1.0081475800686235000000000,
        1.3359138425778705000000000,
        1.4815419524892652000000000,
        1.0384996851820663000000000,
        0.5202348070887524700000000,
        0.8579673418258558800000000,
        0.6413108960765615500000000,
        0.4838340220821504800000000,
    ];
    let ismoke = [
        0.0,
        0.1119475792364162500000000,
        0.3110132095412204700000000,
        0.3328898469326042100000000,
        0.4257069026941993100000000,
    ];

    let dage = i.age as f64 / 10.0;
    let mut age_1 = dage.ln();
    let mut age_2 = dage.powf(3.0);
    let dbmi = bmi / 10.0;
    let mut bmi_1 = dbmi.powf(2.0);
    let mut bmi_2 = dbmi.powf(3.0);
    let mut fbs_1 = fbs.powf(-0.5);
    let mut fbs_2 = fbs.powf(-0.5) * fbs.ln();

    age_1 -= 1.496392488479614;
    age_2 -= 89.048171997070313;
    bmi_1 -= 6.817805767059326;
    bmi_2 -= 17.801923751831055;
    fbs_1 -= 0.448028832674026;
    fbs_2 -= 0.719442605972290;
    let town = i.townsend - 0.515986680984497;

    let mut a = 0.0;
    a += iethrisk[i.ethnicity.index()];
    a += ismoke[i.smoking.cat()];

    a += age_1 * 4.1149143302364717000000000;
    a += age_2 * -0.0047593576668505362000000;
    a += bmi_1 * 0.8169361587644297100000000;
    a += bmi_2 * -0.1250237740343336200000000;
    a += fbs_1 * -54.8417881280971070000000000;
    a += fbs_2 * -53.1120784984813600000000000;
    a += town * 0.0253741755198943560000000;

    a += b(i.atypical_antipsychotic) * 0.4417934088889577400000000;
    a += b(i.regular_steroids) * 0.3413547348339454100000000;
    a += b(i.cardiovascular_disease) * 0.2158977454372756600000000;
    a += b(i.learning_disability) * 0.4012885027585300100000000;
    a += b(i.bipolar_or_schizophrenia) * 0.2181769391399779300000000;
    a += b(i.statin) * 0.5147657600111734700000000;
    a += b(i.treated_hypertension) * 0.2467209287407037300000000;
    a += b(i.family_history_diabetes) * 0.5749437333987512700000000;

    a += age_1 * b(i.atypical_antipsychotic) * -0.9502224313823126600000000;
    a += age_1 * b(i.learning_disability) * -0.8358370163090045300000000;
    a += age_1 * b(i.statin) * -1.8141786919269460000000000;
    a += age_1 * bmi_1 * 0.3748482092078384600000000;
    a += age_1 * bmi_2 * -0.0909836579562487420000000;
    a += age_1 * fbs_1 * 21.0117301217643340000000000;
    a += age_1 * fbs_2 * 23.8244600447469740000000000;
    a += age_1 * b(i.family_history_diabetes) * -0.6780647705291665800000000;
    a += age_2 * b(i.atypical_antipsychotic) * 0.0001472972077162874300000;
    a += age_2 * b(i.learning_disability) * 0.0006012919264966409100000;
    a += age_2 * b(i.statin) * 0.0016393484911405418000000;
    a += age_2 * bmi_1 * -0.0010774782221531713000000;
    a += age_2 * bmi_2 * 0.0001911048730458310100000;
    a += age_2 * fbs_1 * -0.0390046079223835270000000;
    a += age_2 * fbs_2 * -0.0411277198058959470000000;
    a += age_2 * b(i.family_history_diabetes) * 0.0006257588248859499300000;

    100.0 * (1.0 - survivor10.powf(a.exp()))
}

/// Model C, male (adds HbA1c).
fn male_c_raw(i: &QdiabetesInput, bmi: f64, hba1c: f64) -> f64 {
    let survivor10 = 0.981181740760803_f64;

    let iethrisk = [
        0.0,
        0.0,
        0.6757120705498780300000000,
        0.8314732504966345600000000,
        1.0969133802228563000000000,
        0.7682244636456048200000000,
        0.2089752925910850200000000,
        0.3809159378197057900000000,
        0.3423583679661269500000000,
        0.2204647785343308300000000,
    ];
    let ismoke = [
        0.0,
        0.1159289120687865100000000,
        0.1462418263763327100000000,
        0.1078142411249314200000000,
        0.1984862916366847400000000,
    ];

    let dage = i.age as f64 / 10.0;
    let mut age_1 = dage.ln();
    let mut age_2 = dage.powf(3.0);
    let dbmi = bmi / 10.0;
    let mut bmi_1 = dbmi.powf(2.0);
    let mut bmi_2 = dbmi.powf(3.0);
    let dhba1c = hba1c / 10.0;
    let mut hba1c_1 = dhba1c.powf(0.5);
    let mut hba1c_2 = dhba1c;

    age_1 -= 1.496392488479614;
    age_2 -= 89.048171997070313;
    bmi_1 -= 6.817805767059326;
    bmi_2 -= 17.801923751831055;
    hba1c_1 -= 1.900265336036682;
    hba1c_2 -= 3.611008167266846;
    let town = i.townsend - 0.515986680984497;

    let mut a = 0.0;
    a += iethrisk[i.ethnicity.index()];
    a += ismoke[i.smoking.cat()];

    a += age_1 * 4.0193435623978031000000000;
    a += age_2 * -0.0048396442306278238000000;
    a += bmi_1 * 0.8182916890534932500000000;
    a += bmi_2 * -0.1255880870135964200000000;
    a += hba1c_1 * 8.0511642238857934000000000;
    a += hba1c_2 * -0.1465234689391449500000000;
    a += town * 0.0252299651849007270000000;

    a += b(i.atypical_antipsychotic) * 0.4554152522017330100000000;
    a += b(i.regular_steroids) * 0.1381618768682392200000000;
    a += b(i.cardiovascular_disease) * 0.1454698889623951800000000;
    a += b(i.learning_disability) * 0.2596046658040857000000000;
    a += b(i.bipolar_or_schizophrenia) * 0.2852378849058589400000000;
    a += b(i.statin) * 0.4255195190118552500000000;
    a += b(i.treated_hypertension) * 0.3316943000645931100000000;
    a += b(i.family_history_diabetes) * 0.5661232594368061900000000;

    a += age_1 * b(i.atypical_antipsychotic) * -1.0013331909079835000000000;
    a += age_1 * b(i.learning_disability) * -0.8916465737221592700000000;
    a += age_1 * b(i.statin) * -1.7074561167819817000000000;
    a += age_1 * bmi_1 * 0.4507452747267244300000000;
    a += age_1 * bmi_2 * -0.1085185980916560100000000;
    a += age_1 * b(i.family_history_diabetes) * -0.6141009388709716100000000;
    a += age_1 * hba1c_1 * 27.6705938271465650000000000;
    a += age_1 * hba1c_2 * -7.4006134846785434000000000;
    a += age_2 * b(i.atypical_antipsychotic) * 0.0002245597398574240700000;
    a += age_2 * b(i.learning_disability) * 0.0006604436076569648200000;
    a += age_2 * b(i.statin) * 0.0013873509357389619000000;
    a += age_2 * bmi_1 * -0.0012224736160287865000000;
    a += age_2 * bmi_2 * 0.0002266731010346126000000;
    a += age_2 * b(i.family_history_diabetes) * 0.0005060258289477209100000;
    a += age_2 * hba1c_1 * -0.0592014581247543300000000;
    a += age_2 * hba1c_2 * 0.0155920894851499880000000;

    100.0 * (1.0 - survivor10.powf(a.exp()))
}

/// Pure scoring: the QDiabetes-2018 model. BMI is derived from height and
/// weight; the model (A, B, or C) is selected from which optional reading, if
/// any, is supplied.
pub fn compute(input: &QdiabetesInput) -> Result<QdiabetesOutcome, CalcError> {
    if input.age < 25 || input.age > 84 {
        return Err(CalcError::InvalidInput(
            "QDiabetes-2018 is validated for ages 25-84".into(),
        ));
    }
    if !(input.height_cm.is_finite() && (140.0..=210.0).contains(&input.height_cm)) {
        return Err(CalcError::InvalidInput(
            "height_cm must be between 140 and 210".into(),
        ));
    }
    if !(input.weight_kg.is_finite() && (40.0..=180.0).contains(&input.weight_kg)) {
        return Err(CalcError::InvalidInput(
            "weight_kg must be between 40 and 180".into(),
        ));
    }
    if !input.townsend.is_finite() {
        return Err(CalcError::InvalidInput(
            "townsend must be a finite number".into(),
        ));
    }
    if input.fasting_glucose_mmol_l.is_some() && input.hba1c_mmol_mol.is_some() {
        return Err(CalcError::InvalidInput(
            "provide at most one of fasting_glucose_mmol_l / hba1c_mmol_mol; QDiabetes-2018 has no \
model using both"
                .into(),
        ));
    }
    if let Some(fbs) = input.fasting_glucose_mmol_l
        && !(fbs.is_finite() && (2.0..=6.99).contains(&fbs))
    {
        return Err(CalcError::InvalidInput(
            "fasting_glucose_mmol_l must be between 2.0 and 6.99".into(),
        ));
    }
    if let Some(hba1c) = input.hba1c_mmol_mol
        && !(hba1c.is_finite() && (15.0..=47.99).contains(&hba1c))
    {
        return Err(CalcError::InvalidInput(
            "hba1c_mmol_mol must be between 15.0 and 47.99".into(),
        ));
    }

    let height_m = input.height_cm / 100.0;
    let bmi = input.weight_kg / (height_m * height_m);

    let (model, raw) = match (
        input.sex,
        input.hba1c_mmol_mol,
        input.fasting_glucose_mmol_l,
    ) {
        (Sex::Female, Some(hba1c), _) => (Model::C, female_c_raw(input, bmi, hba1c)),
        (Sex::Female, None, Some(fbs)) => (Model::B, female_b_raw(input, bmi, fbs)),
        (Sex::Female, None, None) => (Model::A, female_a_raw(input, bmi)),
        (Sex::Male, Some(hba1c), _) => (Model::C, male_c_raw(input, bmi, hba1c)),
        (Sex::Male, None, Some(fbs)) => (Model::B, male_b_raw(input, bmi, fbs)),
        (Sex::Male, None, None) => (Model::A, male_a_raw(input, bmi)),
    };

    let risk_percent = (raw * 10.0).round() / 10.0;

    let model_label = match model {
        Model::A => "A (demographics and clinical history)",
        Model::B => "B (adds fasting blood glucose)",
        Model::C => "C (adds HbA1c)",
    };

    let interpretation = format!(
        "10-year risk of developing type 2 diabetes: {risk_percent}%, QDiabetes-2018 model \
{model_label}. NICE recommends a validated computer-based risk-assessment tool such as this to \
identify patients who may be at high risk of type 2 diabetes from routinely available primary-care \
data; a raised score should prompt confirmatory testing (fasting glucose or HbA1c) rather than a \
diagnosis on the score alone. BMI used: {bmi:.1} kg/m2. {DISCLAIMER}"
    );

    Ok(QdiabetesOutcome {
        risk_percent,
        bmi,
        model,
        interpretation,
    })
}

/// Build the dispatchable [`CalculationResponse`] from typed inputs.
pub fn build_response(input: &QdiabetesInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;

    let mut working = Map::new();
    working.insert("risk_percent".into(), json!(o.risk_percent));
    working.insert("bmi".into(), json!((o.bmi * 10.0).round() / 10.0));
    working.insert("model".into(), json!(o.model));
    working.insert("sex".into(), json!(input.sex));
    working.insert("disclaimer".into(), json!(DISCLAIMER));

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(o.risk_percent),
        interpretation: o.interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

/// Unit struct implementing the dynamic [`Calculator`] surface.
pub struct Qdiabetes;

impl Calculator for Qdiabetes {
    fn name(&self) -> &'static str {
        NAME
    }

    fn title(&self) -> &'static str {
        "QDiabetes-2018 (10-year type 2 diabetes risk)"
    }

    fn description(&self) -> &'static str {
        "10-year risk of developing type 2 diabetes (QDiabetes-2018), recommended by NICE for \
identifying patients at high risk from routinely available primary-care data."
    }

    fn reference(&self) -> &'static str {
        REFERENCE
    }

    fn license(&self) -> CalculatorLicense {
        LICENSE
    }

    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "QdiabetesInput",
            "type": "object",
            "additionalProperties": false,
            "required": [
                "age", "sex", "ethnicity", "smoking", "height_cm", "weight_kg", "townsend",
                "family_history_diabetes", "cardiovascular_disease", "treated_hypertension",
                "learning_disability", "bipolar_or_schizophrenia", "regular_steroids", "statin",
                "atypical_antipsychotic", "polycystic_ovary_syndrome", "gestational_diabetes"
            ],
            "properties": {
                "age": {
                    "type": "integer",
                    "minimum": 25,
                    "maximum": 84,
                    "description": "Age in years (QDiabetes-2018 is validated for 25-84)"
                },
                "sex": {
                    "type": "string",
                    "enum": ["male", "female"],
                    "description": "Biological sex, selecting the male or female equation"
                },
                "ethnicity": {
                    "type": "string",
                    "enum": [
                        "white_or_not_stated", "indian", "pakistani", "bangladeshi", "other_asian",
                        "black_caribbean", "black_african", "chinese", "other_ethnic_group"
                    ],
                    "description": "Self-reported ethnicity in the nine QDiabetes categories"
                },
                "smoking": {
                    "type": "string",
                    "enum": ["non_smoker", "ex_smoker", "light_smoker", "moderate_smoker", "heavy_smoker"],
                    "description": "Smoking status in the five QDiabetes categories"
                },
                "height_cm": {
                    "type": "number",
                    "minimum": 140,
                    "maximum": 210,
                    "description": "Height in centimetres (used to derive BMI)"
                },
                "weight_kg": {
                    "type": "number",
                    "minimum": 40,
                    "maximum": 180,
                    "description": "Weight in kilograms (used to derive BMI)"
                },
                "townsend": {
                    "type": "number",
                    "description": "Townsend deprivation score (area-based; negative = less deprived, 0 if unknown)"
                },
                "family_history_diabetes": {
                    "type": "boolean",
                    "description": "Immediate family (mother, father, brother, or sister) with diabetes"
                },
                "cardiovascular_disease": { "type": "boolean", "description": "Heart attack, angina, stroke, or TIA" },
                "treated_hypertension": { "type": "boolean", "description": "High blood pressure requiring treatment" },
                "learning_disability": { "type": "boolean", "description": "Learning disabilities" },
                "bipolar_or_schizophrenia": { "type": "boolean", "description": "Manic depression (bipolar disorder) or schizophrenia" },
                "regular_steroids": { "type": "boolean", "description": "On regular steroid tablets" },
                "statin": { "type": "boolean", "description": "On statins" },
                "atypical_antipsychotic": { "type": "boolean", "description": "On atypical antipsychotic medication" },
                "polycystic_ovary_syndrome": {
                    "type": "boolean",
                    "description": "Polycystic ovary syndrome (female model only; ignored for males)"
                },
                "gestational_diabetes": {
                    "type": "boolean",
                    "description": "Gestational diabetes, i.e. diabetes that arose during a pregnancy (female model only; ignored for males)"
                },
                "fasting_glucose_mmol_l": {
                    "type": "number",
                    "minimum": 2.0,
                    "maximum": 6.99,
                    "description": "Fasting blood glucose, mmol/l (2.0-6.99). Selects model B. Leave unset if unknown; mutually exclusive with hba1c_mmol_mol."
                },
                "hba1c_mmol_mol": {
                    "type": "number",
                    "minimum": 15.0,
                    "maximum": 47.99,
                    "description": "HbA1c, mmol/mol (15.0-47.99). Selects model C. Leave unset if unknown; mutually exclusive with fasting_glucose_mmol_l."
                }
            }
        })
    }

    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: QdiabetesInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A neutral baseline: every boolean false, mean-ish continuous values.
    /// Tests override fields as needed.
    fn base(age: u8, sex: Sex, height_cm: f64, weight_kg: f64) -> QdiabetesInput {
        QdiabetesInput {
            age,
            sex,
            ethnicity: Ethnicity::WhiteOrNotStated,
            smoking: Smoking::NonSmoker,
            height_cm,
            weight_kg,
            townsend: 0.0,
            family_history_diabetes: false,
            cardiovascular_disease: false,
            treated_hypertension: false,
            learning_disability: false,
            bipolar_or_schizophrenia: false,
            regular_steroids: false,
            statin: false,
            atypical_antipsychotic: false,
            polycystic_ovary_syndrome: false,
            gestational_diabetes: false,
            fasting_glucose_mmol_l: None,
            hba1c_mmol_mol: None,
        }
    }

    /// Round to one decimal place, as QDiabetes-2018 reports.
    fn r1(x: f64) -> f64 {
        (x * 10.0).round() / 10.0
    }

    // --- Validation against an independent from-source re-implementation ----
    //
    // The reference scores below come from a standalone Python transcription
    // of ClinRisk's published C source (see the archived src.php cited in
    // [`LICENSE`]), written and computed independently of this Rust module.
    // Matching these to one decimal place demonstrates a faithful port; BMI is
    // derived from the same height/weight given to `compute`.

    /// 55F, white, 165cm/68kg (BMI ~25.0), no risk factors, model A -> 2.1%.
    #[test]
    fn validation_female_model_a_baseline() {
        let i = base(55, Sex::Female, 165.0, 68.0);
        let o = compute(&i).unwrap();
        assert_eq!(r1(o.risk_percent), 2.1);
        assert_eq!(o.model, Model::A);
    }

    /// As above but Black African, with family history and treated hypertension -> 7.1%.
    #[test]
    fn validation_female_model_a_risk_factors() {
        let mut i = base(55, Sex::Female, 165.0, 68.0);
        i.ethnicity = Ethnicity::BlackAfrican;
        i.family_history_diabetes = true;
        i.treated_hypertension = true;
        assert_eq!(r1(compute(&i).unwrap().risk_percent), 7.1);
    }

    /// 60F, white, 165cm/68kg, fasting glucose 5.5 -> model B, 4.6%.
    #[test]
    fn validation_female_model_b() {
        let mut i = base(60, Sex::Female, 165.0, 68.0);
        i.fasting_glucose_mmol_l = Some(5.5);
        let o = compute(&i).unwrap();
        assert_eq!(r1(o.risk_percent), 4.6);
        assert_eq!(o.model, Model::B);
    }

    /// 60F, white, 165cm/68kg, HbA1c 42 -> model C, 6.0%.
    #[test]
    fn validation_female_model_c() {
        let mut i = base(60, Sex::Female, 165.0, 68.0);
        i.hba1c_mmol_mol = Some(42.0);
        let o = compute(&i).unwrap();
        assert_eq!(r1(o.risk_percent), 6.0);
        assert_eq!(o.model, Model::C);
    }

    /// 55M, white, 178cm/85kg (BMI ~26.8), no risk factors, model A -> 4.4%.
    #[test]
    fn validation_male_model_a_baseline() {
        let i = base(55, Sex::Male, 178.0, 85.0);
        let o = compute(&i).unwrap();
        assert_eq!(r1(o.risk_percent), 4.4);
        assert_eq!(o.model, Model::A);
    }

    /// As above but Pakistani, on statin and treated hypertension, heavy smoker -> 39.1%.
    #[test]
    fn validation_male_model_a_risk_factors() {
        let mut i = base(55, Sex::Male, 178.0, 85.0);
        i.ethnicity = Ethnicity::Pakistani;
        i.statin = true;
        i.treated_hypertension = true;
        i.smoking = Smoking::HeavySmoker;
        assert_eq!(r1(compute(&i).unwrap().risk_percent), 39.1);
    }

    /// 60M, white, 178cm/85kg, fasting glucose 5.8 -> model B, 9.8%.
    #[test]
    fn validation_male_model_b() {
        let mut i = base(60, Sex::Male, 178.0, 85.0);
        i.fasting_glucose_mmol_l = Some(5.8);
        let o = compute(&i).unwrap();
        assert_eq!(r1(o.risk_percent), 9.8);
        assert_eq!(o.model, Model::B);
    }

    /// 60M, white, 178cm/85kg, HbA1c 40 -> model C, 7.4%.
    #[test]
    fn validation_male_model_c() {
        let mut i = base(60, Sex::Male, 178.0, 85.0);
        i.hba1c_mmol_mol = Some(40.0);
        let o = compute(&i).unwrap();
        assert_eq!(r1(o.risk_percent), 7.4);
        assert_eq!(o.model, Model::C);
    }

    // --- Validation / boundary tests -----------------------------------

    #[test]
    fn rejects_age_outside_25_to_84() {
        let mut i = base(24, Sex::Female, 165.0, 68.0);
        assert!(compute(&i).is_err());
        i.age = 85;
        assert!(compute(&i).is_err());
    }

    #[test]
    fn accepts_age_boundaries() {
        assert!(compute(&base(25, Sex::Female, 165.0, 68.0)).is_ok());
        assert!(compute(&base(84, Sex::Female, 165.0, 68.0)).is_ok());
    }

    #[test]
    fn rejects_height_outside_140_to_210() {
        assert!(compute(&base(55, Sex::Female, 139.9, 68.0)).is_err());
        assert!(compute(&base(55, Sex::Female, 210.1, 68.0)).is_err());
    }

    #[test]
    fn rejects_weight_outside_40_to_180() {
        assert!(compute(&base(55, Sex::Female, 165.0, 39.9)).is_err());
        assert!(compute(&base(55, Sex::Female, 165.0, 180.1)).is_err());
    }

    #[test]
    fn rejects_fasting_glucose_outside_2_to_6_99() {
        let mut i = base(55, Sex::Female, 165.0, 68.0);
        i.fasting_glucose_mmol_l = Some(1.9);
        assert!(compute(&i).is_err());
        i.fasting_glucose_mmol_l = Some(7.0);
        assert!(compute(&i).is_err());
    }

    #[test]
    fn rejects_hba1c_outside_15_to_47_99() {
        let mut i = base(55, Sex::Female, 165.0, 68.0);
        i.hba1c_mmol_mol = Some(14.9);
        assert!(compute(&i).is_err());
        i.hba1c_mmol_mol = Some(48.0);
        assert!(compute(&i).is_err());
    }

    #[test]
    fn rejects_both_glucose_and_hba1c_together() {
        let mut i = base(55, Sex::Female, 165.0, 68.0);
        i.fasting_glucose_mmol_l = Some(5.0);
        i.hba1c_mmol_mol = Some(40.0);
        assert!(compute(&i).is_err());
    }

    #[test]
    fn hba1c_takes_precedence_if_somehow_both_are_set_via_dynamic_json() {
        // The typed path rejects both being set (see test above); this just
        // documents that the dynamic `calculate` path goes through the same
        // `compute` validation and also rejects it.
        let calc = Qdiabetes;
        let input = json!({
            "age": 55, "sex": "female", "ethnicity": "white_or_not_stated",
            "smoking": "non_smoker", "height_cm": 165.0, "weight_kg": 68.0, "townsend": 0.0,
            "family_history_diabetes": false, "cardiovascular_disease": false,
            "treated_hypertension": false, "learning_disability": false,
            "bipolar_or_schizophrenia": false, "regular_steroids": false, "statin": false,
            "atypical_antipsychotic": false, "polycystic_ovary_syndrome": false,
            "gestational_diabetes": false, "fasting_glucose_mmol_l": 5.0, "hba1c_mmol_mol": 40.0
        });
        assert!(calc.calculate(&input).is_err());
    }

    #[test]
    fn higher_bmi_increases_risk() {
        let lower = base(55, Sex::Female, 165.0, 60.0);
        let higher = base(55, Sex::Female, 165.0, 90.0);
        assert!(compute(&higher).unwrap().risk_percent > compute(&lower).unwrap().risk_percent);
    }

    #[test]
    fn pcos_and_gestational_diabetes_increase_female_risk() {
        let baseline = base(35, Sex::Female, 165.0, 68.0);
        let mut with_pcos = baseline;
        with_pcos.polycystic_ovary_syndrome = true;
        let mut with_gestdiab = baseline;
        with_gestdiab.gestational_diabetes = true;
        assert!(
            compute(&with_pcos).unwrap().risk_percent > compute(&baseline).unwrap().risk_percent
        );
        assert!(
            compute(&with_gestdiab).unwrap().risk_percent
                > compute(&baseline).unwrap().risk_percent
        );
    }

    #[test]
    fn dynamic_calculate_matches_typed_build_response() {
        let calc = Qdiabetes;
        let input = json!({
            "age": 55, "sex": "male", "ethnicity": "white_or_not_stated",
            "smoking": "non_smoker", "height_cm": 178.0, "weight_kg": 85.0, "townsend": 0.0,
            "family_history_diabetes": false, "cardiovascular_disease": false,
            "treated_hypertension": false, "learning_disability": false,
            "bipolar_or_schizophrenia": false, "regular_steroids": false, "statin": false,
            "atypical_antipsychotic": false, "polycystic_ovary_syndrome": false,
            "gestational_diabetes": false
        });
        let dynamic = calc.calculate(&input).unwrap();

        let typed = base(55, Sex::Male, 178.0, 85.0);
        let typed_response = build_response(&typed).unwrap();

        assert_eq!(dynamic.result, typed_response.result);
    }

    #[test]
    fn schema_is_valid_json_and_lists_all_required_fields() {
        let calc = Qdiabetes;
        let schema = calc.input_schema();
        let required = schema["required"].as_array().unwrap();
        for field in [
            "age",
            "sex",
            "ethnicity",
            "smoking",
            "height_cm",
            "weight_kg",
            "townsend",
            "family_history_diabetes",
            "cardiovascular_disease",
            "treated_hypertension",
            "learning_disability",
            "bipolar_or_schizophrenia",
            "regular_steroids",
            "statin",
            "atypical_antipsychotic",
            "polycystic_ovary_syndrome",
            "gestational_diabetes",
        ] {
            assert!(
                required.iter().any(|v| v == field),
                "missing required field: {field}"
            );
        }
    }

    #[test]
    fn license_is_agpl_with_archived_source_evidence() {
        let calc = Qdiabetes;
        let license = calc.license();
        assert!(license.license.contains("AGPL"));
        assert!(license.source_url.contains("web.archive.org"));
    }
}
