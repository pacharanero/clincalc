// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Lund and Browder Chart - age-adjusted %TBSA (total body surface area)
//! burn estimate (Lund, Browder. Surg Gynecol Obstet. 1944).
//!
//! The chart divides the body into 19 named regions (symmetric regions
//! split left/right) and assigns each a maximum %TBSA that depends on age
//! for the head, thigh, and lower leg only - everything else is constant
//! across ages. This implementation takes each region as the **fraction of
//! that region actually burned** (0.0 none to 1.0 the whole region), not a
//! strict whole-region boolean: at least one widely used institutional
//! charting protocol explicitly defines each region by its *maximum*
//! percentage and expects the assessor to fill in a lower number for
//! partial involvement, so this is a documented clinical charting
//! convention, not an invented liberty. A fraction of exactly `0.0` or
//! `1.0` for every region reproduces the strict whole-region-only reading
//! some clinicians use at the bedside.
//!
//! The 19-region percentage table itself has no single clean primary
//! citation: the 1944 original paper has no stable online copy, and the
//! modern age-banded table is reproduced identically across independent
//! institutional burn-unit protocols (cross-checked here against the
//! University of South Alabama and Vanderbilt University Medical Center
//! paediatric burn-resuscitation protocols) rather than one citable
//! source. This is the same "widely reproduced clinical teaching table"
//! situation as several other instruments in this registry.
//!
//! **Limitation inherent to the chart itself, not this implementation**:
//! within one named region the chart cannot distinguish a circumferential
//! burn from a localised one of the same area, so two differently-shaped
//! burns of equal fraction report the same %TBSA.
//!
//! The result is designed to feed directly into [`crate::calculators::parkland`]'s
//! `tbsa_percent` input.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::calculator::{CalcError, Calculator};
use crate::license::CalculatorLicense;
use crate::response::CalculationResponse;

pub const NAME: &str = "lund_browder";

pub const REFERENCE: &str = "Lund CC, Browder NC. The estimation of areas of burns. Surg Gynecol Obstet. \
1944;79:352-358 (original chart and age-variable regional division; no stable online copy of the 1944 \
original exists). The modern age-banded percentage table is as widely reproduced in clinical practice and \
cross-checked here against independent institutional burn-protocol charts (University of South Alabama; \
Vanderbilt University Medical Center paediatric burn fluid-resuscitation protocol), consistent with the \
chart reproduced in Hettiaratchy S, Papini R. ABC of burns: initial management of a major burn: II - \
assessment and resuscitation. BMJ. 2004;329(7457):101-103.";

pub const LICENSE: CalculatorLicense = CalculatorLicense::new(
    "Public-domain method - implemented from the primary literature and cross-checked against independent institutional burn-protocol charts",
    "https://doi.org/10.1136/bmj.329.7457.101",
);

/// Lund-Browder inputs: age plus the fraction of each of the 19 named
/// regions that is burned (0.0 none to 1.0 the whole region).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LundBrowderInput {
    pub age_years: f64,
    pub head_fraction_burned: f64,
    pub neck_fraction_burned: f64,
    pub anterior_trunk_fraction_burned: f64,
    pub posterior_trunk_fraction_burned: f64,
    pub right_buttock_fraction_burned: f64,
    pub left_buttock_fraction_burned: f64,
    pub genitalia_fraction_burned: f64,
    pub right_upper_arm_fraction_burned: f64,
    pub left_upper_arm_fraction_burned: f64,
    pub right_lower_arm_fraction_burned: f64,
    pub left_lower_arm_fraction_burned: f64,
    pub right_hand_fraction_burned: f64,
    pub left_hand_fraction_burned: f64,
    pub right_thigh_fraction_burned: f64,
    pub left_thigh_fraction_burned: f64,
    pub right_lower_leg_fraction_burned: f64,
    pub left_lower_leg_fraction_burned: f64,
    pub right_foot_fraction_burned: f64,
    pub left_foot_fraction_burned: f64,
}

/// The six age bands the chart defines - discrete columns read off a wall
/// chart, not interpolated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgeBand {
    UnderOne,
    OneToFour,
    FiveToNine,
    TenToFourteen,
    Fifteen,
    Adult,
}

impl AgeBand {
    fn from_age(age_years: f64) -> Self {
        if age_years < 1.0 {
            Self::UnderOne
        } else if age_years < 5.0 {
            Self::OneToFour
        } else if age_years < 10.0 {
            Self::FiveToNine
        } else if age_years < 15.0 {
            Self::TenToFourteen
        } else if age_years < 16.0 {
            Self::Fifteen
        } else {
            Self::Adult
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Self::UnderOne => "under_1_year",
            Self::OneToFour => "1_to_4_years",
            Self::FiveToNine => "5_to_9_years",
            Self::TenToFourteen => "10_to_14_years",
            Self::Fifteen => "15_years",
            Self::Adult => "adult_16_plus",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::UnderOne => "under 1 year",
            Self::OneToFour => "1-4 years",
            Self::FiveToNine => "5-9 years",
            Self::TenToFourteen => "10-14 years",
            Self::Fifteen => "15 years",
            Self::Adult => "adult (16 and over)",
        }
    }

    /// Maximum %TBSA per region for this age band. Head, thigh, and lower
    /// leg vary with age; every other region is constant across all bands.
    fn region_percentages(self) -> RegionPercentages {
        let (head, thigh, lower_leg) = match self {
            Self::UnderOne => (19.0, 5.5, 5.0),
            Self::OneToFour => (17.0, 6.5, 5.0),
            Self::FiveToNine => (13.0, 8.0, 5.5),
            Self::TenToFourteen => (11.0, 8.5, 6.0),
            Self::Fifteen => (9.0, 9.0, 6.5),
            Self::Adult => (7.0, 9.5, 7.0),
        };
        RegionPercentages {
            head,
            neck: 2.0,
            anterior_trunk: 13.0,
            posterior_trunk: 13.0,
            right_buttock: 2.5,
            left_buttock: 2.5,
            genitalia: 1.0,
            right_upper_arm: 4.0,
            left_upper_arm: 4.0,
            right_lower_arm: 3.0,
            left_lower_arm: 3.0,
            right_hand: 2.5,
            left_hand: 2.5,
            right_thigh: thigh,
            left_thigh: thigh,
            right_lower_leg: lower_leg,
            left_lower_leg: lower_leg,
            right_foot: 3.5,
            left_foot: 3.5,
        }
    }
}

/// Each region's maximum %TBSA for one age band. Always sums to 100.0.
struct RegionPercentages {
    head: f64,
    neck: f64,
    anterior_trunk: f64,
    posterior_trunk: f64,
    right_buttock: f64,
    left_buttock: f64,
    genitalia: f64,
    right_upper_arm: f64,
    left_upper_arm: f64,
    right_lower_arm: f64,
    left_lower_arm: f64,
    right_hand: f64,
    left_hand: f64,
    right_thigh: f64,
    left_thigh: f64,
    right_lower_leg: f64,
    left_lower_leg: f64,
    right_foot: f64,
    left_foot: f64,
}

impl RegionPercentages {
    #[cfg(test)]
    fn total(&self) -> f64 {
        self.head
            + self.neck
            + self.anterior_trunk
            + self.posterior_trunk
            + self.right_buttock
            + self.left_buttock
            + self.genitalia
            + self.right_upper_arm
            + self.left_upper_arm
            + self.right_lower_arm
            + self.left_lower_arm
            + self.right_hand
            + self.left_hand
            + self.right_thigh
            + self.left_thigh
            + self.right_lower_leg
            + self.left_lower_leg
            + self.right_foot
            + self.left_foot
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LundBrowderOutcome {
    pub age_band: AgeBand,
    pub total_tbsa_percent: f64,
    /// (region name, %TBSA contributed) for each of the 19 regions, in
    /// chart order.
    pub contributions: [(&'static str, f64); 19],
}

pub fn compute(input: &LundBrowderInput) -> Result<LundBrowderOutcome, CalcError> {
    if !input.age_years.is_finite() || !(0.0..=130.0).contains(&input.age_years) {
        return Err(CalcError::InvalidInput(
            "age_years must be between 0 and 130".into(),
        ));
    }

    let fractions = [
        ("head_fraction_burned", input.head_fraction_burned),
        ("neck_fraction_burned", input.neck_fraction_burned),
        (
            "anterior_trunk_fraction_burned",
            input.anterior_trunk_fraction_burned,
        ),
        (
            "posterior_trunk_fraction_burned",
            input.posterior_trunk_fraction_burned,
        ),
        (
            "right_buttock_fraction_burned",
            input.right_buttock_fraction_burned,
        ),
        (
            "left_buttock_fraction_burned",
            input.left_buttock_fraction_burned,
        ),
        ("genitalia_fraction_burned", input.genitalia_fraction_burned),
        (
            "right_upper_arm_fraction_burned",
            input.right_upper_arm_fraction_burned,
        ),
        (
            "left_upper_arm_fraction_burned",
            input.left_upper_arm_fraction_burned,
        ),
        (
            "right_lower_arm_fraction_burned",
            input.right_lower_arm_fraction_burned,
        ),
        (
            "left_lower_arm_fraction_burned",
            input.left_lower_arm_fraction_burned,
        ),
        (
            "right_hand_fraction_burned",
            input.right_hand_fraction_burned,
        ),
        ("left_hand_fraction_burned", input.left_hand_fraction_burned),
        (
            "right_thigh_fraction_burned",
            input.right_thigh_fraction_burned,
        ),
        (
            "left_thigh_fraction_burned",
            input.left_thigh_fraction_burned,
        ),
        (
            "right_lower_leg_fraction_burned",
            input.right_lower_leg_fraction_burned,
        ),
        (
            "left_lower_leg_fraction_burned",
            input.left_lower_leg_fraction_burned,
        ),
        (
            "right_foot_fraction_burned",
            input.right_foot_fraction_burned,
        ),
        ("left_foot_fraction_burned", input.left_foot_fraction_burned),
    ];
    for (name, value) in fractions {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(CalcError::InvalidInput(format!(
                "{name} must be between 0.0 and 1.0"
            )));
        }
    }

    let age_band = AgeBand::from_age(input.age_years);
    let pct = age_band.region_percentages();

    let contributions: [(&'static str, f64); 19] = [
        ("head", pct.head * input.head_fraction_burned),
        ("neck", pct.neck * input.neck_fraction_burned),
        (
            "anterior_trunk",
            pct.anterior_trunk * input.anterior_trunk_fraction_burned,
        ),
        (
            "posterior_trunk",
            pct.posterior_trunk * input.posterior_trunk_fraction_burned,
        ),
        (
            "right_buttock",
            pct.right_buttock * input.right_buttock_fraction_burned,
        ),
        (
            "left_buttock",
            pct.left_buttock * input.left_buttock_fraction_burned,
        ),
        ("genitalia", pct.genitalia * input.genitalia_fraction_burned),
        (
            "right_upper_arm",
            pct.right_upper_arm * input.right_upper_arm_fraction_burned,
        ),
        (
            "left_upper_arm",
            pct.left_upper_arm * input.left_upper_arm_fraction_burned,
        ),
        (
            "right_lower_arm",
            pct.right_lower_arm * input.right_lower_arm_fraction_burned,
        ),
        (
            "left_lower_arm",
            pct.left_lower_arm * input.left_lower_arm_fraction_burned,
        ),
        (
            "right_hand",
            pct.right_hand * input.right_hand_fraction_burned,
        ),
        ("left_hand", pct.left_hand * input.left_hand_fraction_burned),
        (
            "right_thigh",
            pct.right_thigh * input.right_thigh_fraction_burned,
        ),
        (
            "left_thigh",
            pct.left_thigh * input.left_thigh_fraction_burned,
        ),
        (
            "right_lower_leg",
            pct.right_lower_leg * input.right_lower_leg_fraction_burned,
        ),
        (
            "left_lower_leg",
            pct.left_lower_leg * input.left_lower_leg_fraction_burned,
        ),
        (
            "right_foot",
            pct.right_foot * input.right_foot_fraction_burned,
        ),
        ("left_foot", pct.left_foot * input.left_foot_fraction_burned),
    ];

    let total_tbsa_percent: f64 = contributions.iter().map(|(_, v)| v).sum();

    Ok(LundBrowderOutcome {
        age_band,
        total_tbsa_percent,
        contributions,
    })
}

pub fn build_response(input: &LundBrowderInput) -> Result<CalculationResponse, CalcError> {
    let o = compute(input)?;
    let mut working = Map::new();
    working.insert("age_band".into(), json!(o.age_band.slug()));
    working.insert(
        "total_tbsa_percent".into(),
        json!(round1(o.total_tbsa_percent)),
    );
    for (name, value) in o.contributions {
        working.insert(format!("{name}_contribution_percent"), json!(round1(value)));
    }

    let interpretation = format!(
        "Estimated {total:.1}% TBSA burned (Lund-Browder chart, {band} age band). Feeds directly into the \
Parkland formula's tbsa_percent input for first-24-hour fluid-resuscitation volume. Each region is scored \
as the fraction of that region's own allotted %TBSA actually burned (0.0 none to 1.0 the whole region); \
within one region the chart cannot distinguish a circumferential burn from a localised one of the same \
area, so clinical judgement remains essential alongside this estimate.",
        total = o.total_tbsa_percent,
        band = o.age_band.description()
    );

    Ok(CalculationResponse {
        calculator: NAME.to_string(),
        result: json!(round1(o.total_tbsa_percent)),
        interpretation,
        working,
        reference: REFERENCE.to_string(),
    })
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

pub struct LundBrowder;
impl Calculator for LundBrowder {
    fn name(&self) -> &'static str {
        NAME
    }
    fn title(&self) -> &'static str {
        "Lund and Browder Chart (%TBSA Burn Estimate)"
    }
    fn description(&self) -> &'static str {
        "Age-adjusted total body surface area (%TBSA) burn estimate from 19 named body regions, feeding directly into the Parkland formula."
    }
    fn reference(&self) -> &'static str {
        REFERENCE
    }
    fn license(&self) -> CalculatorLicense {
        LICENSE
    }
    fn input_schema(&self) -> Value {
        json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema", "title": "LundBrowderInput", "type": "object",
            "additionalProperties": false,
            "required": [
                "age_years", "head_fraction_burned", "neck_fraction_burned", "anterior_trunk_fraction_burned",
                "posterior_trunk_fraction_burned", "right_buttock_fraction_burned", "left_buttock_fraction_burned",
                "genitalia_fraction_burned", "right_upper_arm_fraction_burned", "left_upper_arm_fraction_burned",
                "right_lower_arm_fraction_burned", "left_lower_arm_fraction_burned", "right_hand_fraction_burned",
                "left_hand_fraction_burned", "right_thigh_fraction_burned", "left_thigh_fraction_burned",
                "right_lower_leg_fraction_burned", "left_lower_leg_fraction_burned", "right_foot_fraction_burned",
                "left_foot_fraction_burned"
            ],
            "properties": {
                "age_years": { "type": "number", "minimum": 0, "maximum": 130, "description": "Age in years; selects the chart's age band (under 1, 1-4, 5-9, 10-14, 15, or adult 16+)" },
                "head_fraction_burned": { "type": "number", "minimum": 0, "maximum": 1, "description": "Fraction of the head (face/scalp) burned, 0.0 none to 1.0 all" },
                "neck_fraction_burned": { "type": "number", "minimum": 0, "maximum": 1, "description": "Fraction of the neck burned, 0.0 none to 1.0 all" },
                "anterior_trunk_fraction_burned": { "type": "number", "minimum": 0, "maximum": 1, "description": "Fraction of the anterior trunk (chest/abdomen) burned, 0.0 none to 1.0 all" },
                "posterior_trunk_fraction_burned": { "type": "number", "minimum": 0, "maximum": 1, "description": "Fraction of the posterior trunk (upper/lower back) burned, 0.0 none to 1.0 all" },
                "right_buttock_fraction_burned": { "type": "number", "minimum": 0, "maximum": 1, "description": "Fraction of the right buttock burned, 0.0 none to 1.0 all" },
                "left_buttock_fraction_burned": { "type": "number", "minimum": 0, "maximum": 1, "description": "Fraction of the left buttock burned, 0.0 none to 1.0 all" },
                "genitalia_fraction_burned": { "type": "number", "minimum": 0, "maximum": 1, "description": "Fraction of the genitalia burned, 0.0 none to 1.0 all" },
                "right_upper_arm_fraction_burned": { "type": "number", "minimum": 0, "maximum": 1, "description": "Fraction of the right upper arm burned, 0.0 none to 1.0 all" },
                "left_upper_arm_fraction_burned": { "type": "number", "minimum": 0, "maximum": 1, "description": "Fraction of the left upper arm burned, 0.0 none to 1.0 all" },
                "right_lower_arm_fraction_burned": { "type": "number", "minimum": 0, "maximum": 1, "description": "Fraction of the right lower arm (forearm) burned, 0.0 none to 1.0 all" },
                "left_lower_arm_fraction_burned": { "type": "number", "minimum": 0, "maximum": 1, "description": "Fraction of the left lower arm (forearm) burned, 0.0 none to 1.0 all" },
                "right_hand_fraction_burned": { "type": "number", "minimum": 0, "maximum": 1, "description": "Fraction of the right hand burned, 0.0 none to 1.0 all" },
                "left_hand_fraction_burned": { "type": "number", "minimum": 0, "maximum": 1, "description": "Fraction of the left hand burned, 0.0 none to 1.0 all" },
                "right_thigh_fraction_burned": { "type": "number", "minimum": 0, "maximum": 1, "description": "Fraction of the right thigh burned, 0.0 none to 1.0 all" },
                "left_thigh_fraction_burned": { "type": "number", "minimum": 0, "maximum": 1, "description": "Fraction of the left thigh burned, 0.0 none to 1.0 all" },
                "right_lower_leg_fraction_burned": { "type": "number", "minimum": 0, "maximum": 1, "description": "Fraction of the right lower leg (calf/shin) burned, 0.0 none to 1.0 all" },
                "left_lower_leg_fraction_burned": { "type": "number", "minimum": 0, "maximum": 1, "description": "Fraction of the left lower leg (calf/shin) burned, 0.0 none to 1.0 all" },
                "right_foot_fraction_burned": { "type": "number", "minimum": 0, "maximum": 1, "description": "Fraction of the right foot burned, 0.0 none to 1.0 all" },
                "left_foot_fraction_burned": { "type": "number", "minimum": 0, "maximum": 1, "description": "Fraction of the left foot burned, 0.0 none to 1.0 all" }
            }
        })
    }
    fn calculate(&self, input: &Value) -> Result<CalculationResponse, CalcError> {
        let parsed: LundBrowderInput = serde_json::from_value(input.clone())
            .map_err(|e| CalcError::InvalidInput(e.to_string()))?;
        build_response(&parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none(age_years: f64) -> LundBrowderInput {
        LundBrowderInput {
            age_years,
            head_fraction_burned: 0.0,
            neck_fraction_burned: 0.0,
            anterior_trunk_fraction_burned: 0.0,
            posterior_trunk_fraction_burned: 0.0,
            right_buttock_fraction_burned: 0.0,
            left_buttock_fraction_burned: 0.0,
            genitalia_fraction_burned: 0.0,
            right_upper_arm_fraction_burned: 0.0,
            left_upper_arm_fraction_burned: 0.0,
            right_lower_arm_fraction_burned: 0.0,
            left_lower_arm_fraction_burned: 0.0,
            right_hand_fraction_burned: 0.0,
            left_hand_fraction_burned: 0.0,
            right_thigh_fraction_burned: 0.0,
            left_thigh_fraction_burned: 0.0,
            right_lower_leg_fraction_burned: 0.0,
            left_lower_leg_fraction_burned: 0.0,
            right_foot_fraction_burned: 0.0,
            left_foot_fraction_burned: 0.0,
        }
    }

    fn all_burned(age_years: f64) -> LundBrowderInput {
        LundBrowderInput {
            head_fraction_burned: 1.0,
            neck_fraction_burned: 1.0,
            anterior_trunk_fraction_burned: 1.0,
            posterior_trunk_fraction_burned: 1.0,
            right_buttock_fraction_burned: 1.0,
            left_buttock_fraction_burned: 1.0,
            genitalia_fraction_burned: 1.0,
            right_upper_arm_fraction_burned: 1.0,
            left_upper_arm_fraction_burned: 1.0,
            right_lower_arm_fraction_burned: 1.0,
            left_lower_arm_fraction_burned: 1.0,
            right_hand_fraction_burned: 1.0,
            left_hand_fraction_burned: 1.0,
            right_thigh_fraction_burned: 1.0,
            left_thigh_fraction_burned: 1.0,
            right_lower_leg_fraction_burned: 1.0,
            left_lower_leg_fraction_burned: 1.0,
            right_foot_fraction_burned: 1.0,
            left_foot_fraction_burned: 1.0,
            ..none(age_years)
        }
    }

    #[test]
    fn no_burns_scores_zero() {
        let o = compute(&none(30.0)).unwrap();
        assert_eq!(o.total_tbsa_percent, 0.0);
    }

    #[test]
    fn every_age_band_sums_to_100_percent_when_fully_burned() {
        // Literature-vector check: the chart is defined so that burning
        // every region completely always totals exactly 100% TBSA,
        // regardless of age band.
        for age in [0.5, 3.0, 7.0, 12.0, 15.0, 40.0] {
            let o = compute(&all_burned(age)).unwrap();
            assert_eq!(o.total_tbsa_percent, 100.0, "age {age} did not sum to 100%");
        }
    }

    #[test]
    fn region_percentages_table_sums_to_100_for_every_band() {
        for band in [
            AgeBand::UnderOne,
            AgeBand::OneToFour,
            AgeBand::FiveToNine,
            AgeBand::TenToFourteen,
            AgeBand::Fifteen,
            AgeBand::Adult,
        ] {
            assert_eq!(band.region_percentages().total(), 100.0, "{band:?}");
        }
    }

    #[test]
    fn age_band_boundaries_select_the_right_column() {
        assert_eq!(AgeBand::from_age(0.0), AgeBand::UnderOne);
        assert_eq!(AgeBand::from_age(0.99), AgeBand::UnderOne);
        assert_eq!(AgeBand::from_age(1.0), AgeBand::OneToFour);
        assert_eq!(AgeBand::from_age(4.99), AgeBand::OneToFour);
        assert_eq!(AgeBand::from_age(5.0), AgeBand::FiveToNine);
        assert_eq!(AgeBand::from_age(9.99), AgeBand::FiveToNine);
        assert_eq!(AgeBand::from_age(10.0), AgeBand::TenToFourteen);
        assert_eq!(AgeBand::from_age(14.99), AgeBand::TenToFourteen);
        assert_eq!(AgeBand::from_age(15.0), AgeBand::Fifteen);
        assert_eq!(AgeBand::from_age(15.99), AgeBand::Fifteen);
        assert_eq!(AgeBand::from_age(16.0), AgeBand::Adult);
        assert_eq!(AgeBand::from_age(70.0), AgeBand::Adult);
    }

    #[test]
    fn head_thigh_and_lower_leg_shrink_or_grow_with_age() {
        let infant = AgeBand::UnderOne.region_percentages();
        let adult = AgeBand::Adult.region_percentages();
        assert!(infant.head > adult.head, "infant head % should be larger");
        assert!(
            infant.right_thigh < adult.right_thigh,
            "infant thigh % should be smaller"
        );
        assert!(
            infant.right_lower_leg < adult.right_lower_leg,
            "infant lower leg % should be smaller"
        );
        // Everything else is constant across ages.
        assert_eq!(infant.anterior_trunk, adult.anterior_trunk);
        assert_eq!(infant.right_hand, adult.right_hand);
    }

    #[test]
    fn half_of_one_region_contributes_half_its_maximum() {
        let mut i = none(30.0); // adult band: right_lower_arm max is 3.0
        i.right_lower_arm_fraction_burned = 0.5;
        let o = compute(&i).unwrap();
        assert_eq!(o.total_tbsa_percent, 1.5);
    }

    #[test]
    fn asymmetric_burns_are_not_doubled() {
        let mut i = none(30.0); // adult band: right_upper_arm max is 4.0
        i.right_upper_arm_fraction_burned = 1.0;
        let o = compute(&i).unwrap();
        assert_eq!(o.total_tbsa_percent, 4.0);
        assert_eq!(i.left_upper_arm_fraction_burned, 0.0);
    }

    #[test]
    fn rejects_fraction_out_of_range() {
        let mut i = none(30.0);
        i.head_fraction_burned = 1.5;
        assert!(compute(&i).is_err());
        let mut i = none(30.0);
        i.head_fraction_burned = -0.1;
        assert!(compute(&i).is_err());
    }

    #[test]
    fn rejects_age_out_of_range() {
        let i = none(-1.0);
        assert!(compute(&i).is_err());
        let i = none(131.0);
        assert!(compute(&i).is_err());
    }

    #[test]
    fn dynamic_calculate_matches_typed() {
        let mut typed = none(8.0); // 5-9 years band
        typed.anterior_trunk_fraction_burned = 1.0;
        typed.right_thigh_fraction_burned = 0.5;
        let value = serde_json::to_value(typed).unwrap();
        let dynamic = LundBrowder.calculate(&value).unwrap();
        assert_eq!(dynamic, build_response(&typed).unwrap());
        // 5-9 years: anterior_trunk 13.0 + half of right_thigh 8.0 = 17.0
        assert_eq!(dynamic.result, json!(17.0));
        assert_eq!(dynamic.working["age_band"], json!("5_to_9_years"));
    }

    #[test]
    fn rejects_missing_field() {
        assert!(LundBrowder.calculate(&json!({"age_years": 30.0})).is_err());
    }
}
