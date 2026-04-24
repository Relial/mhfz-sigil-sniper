use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};
use strum::{FromRepr, IntoEnumIterator};

use crate::{
    rng::Lcg,
    sigil::{
        recipe::{MOSTLY_RELIABLE, RARITY, RELIABLE, UNSUPPORTED},
        sigil_skills::SigilSkill,
    },
};

const NO_DUPE: [SigilSkill; 44] = [
    SigilSkill::RapidFireAdd,
    SigilSkill::SuperRapidFireAdd,
    SigilSkill::RapidFireDisable,
    SigilSkill::RawBowBoost,
    SigilSkill::RawSnsBoost,
    SigilSkill::TechBoostSnS,
    SigilSkill::TechBoostDb,
    SigilSkill::TechBoostGs,
    SigilSkill::TechBoostLs,
    SigilSkill::TechBoostLance,
    SigilSkill::TechBoostGunlance,
    SigilSkill::TechBoostHammer,
    SigilSkill::TechBoostHorn,
    SigilSkill::TechChangeSnS,
    SigilSkill::TechChangeDb,
    SigilSkill::TechChangeGs,
    SigilSkill::TechChangeLs,
    SigilSkill::TechChangeLance,
    SigilSkill::TechChangeGunlance,
    SigilSkill::TechChangeHammer,
    SigilSkill::TechChangeHorn,
    SigilSkill::TechChangeLbg,
    SigilSkill::TechChangeHbg,
    SigilSkill::TechChangeBow,
    SigilSkill::LengthUp,
    SigilSkill::ShellingChange,
    SigilSkill::NoteChange1,
    SigilSkill::NoteChange2,
    SigilSkill::NoteChange3,
    SigilSkill::ArcShotChange,
    SigilSkill::GsUp,
    SigilSkill::HbgUp,
    SigilSkill::HammerUp,
    SigilSkill::LanceUp,
    SigilSkill::SnsUp,
    SigilSkill::LbgUp,
    SigilSkill::DbUp,
    SigilSkill::LsUp,
    SigilSkill::HornUp,
    SigilSkill::GunlanceUp,
    SigilSkill::BowUp,
    SigilSkill::TonfaUp,
    SigilSkill::SafUp,
    SigilSkill::MsUp,
];

impl SigilSkill {
    fn invalid_duplicate(&self, ranges: &BaseSkillRanges, current_rolls: &[SigilSkill]) -> bool {
        if *self == SigilSkill::ShellingLvUp {
            return false;
        }
        let Some(first_range) = ranges.points_ranges.first() else {
            return false;
        };
        if *first_range.points_range.start() == 1
            && *first_range.points_range.end() == 1
            && current_rolls.contains(self)
        {
            return true;
        }
        if *self as u8 > 112 {
            return false;
        }
        current_rolls.contains(self) && NO_DUPE.contains(self)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub enum RedRoll {
    No,
    Maybe,
    Yes,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct SkillRoll {
    pub skill: SigilSkill,
    pub points: RangeInclusive<i16>,
}

impl SkillRoll {
    pub fn new(skill: SigilSkill, points: RangeInclusive<i16>) -> Self {
        Self { skill, points }
    }

    pub fn red_roll(&self, ranges: &BaseSkillRanges) -> RedRoll {
        if let Some(red_req) = &ranges.red_roll {
            match red_req {
                RedRollRequirement::Threshold(threshold) => {
                    if self.points.start() >= threshold {
                        RedRoll::Yes
                    } else if self.points.end() < threshold {
                        RedRoll::No
                    } else {
                        RedRoll::Maybe
                    }
                }
                RedRollRequirement::Specific(red_points) => {
                    let own_points_that_are_be_red = self
                        .points
                        .clone()
                        .filter(|own_point| {
                            red_points.iter().any(|red_point| red_point == own_point)
                        })
                        .count();
                    if own_points_that_are_be_red == self.points.len() {
                        RedRoll::Yes
                    } else if own_points_that_are_be_red > 0 {
                        RedRoll::Maybe
                    } else {
                        RedRoll::No
                    }
                }
            }
        } else {
            RedRoll::No
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Sigil {
    pub rolls: Vec<SkillRoll>,
    pub red_roll: RedRoll,
}

impl Sigil {
    pub fn from_skill_rolls(skill_rolls: Vec<SkillRoll>, base_ranges: &[BaseSkillRanges]) -> Self {
        let mut red_roll = RedRoll::No;
        for roll in &skill_rolls {
            let red = roll.red_roll(&base_ranges[roll.skill as usize]);
            if red > red_roll {
                red_roll = red;
            }
        }
        Self {
            rolls: skill_rolls,
            red_roll,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRepr, Serialize, Deserialize)]
pub enum SigilType {
    Normal,
    Premium,
    Zenith,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RecipeSupport {
    Full,
    Mostly,
    Low,
    ZAoe,
    Rarity,
    Unsupported,
}

impl RecipeSupport {
    pub fn from_idx(recipe_idx: usize) -> Self {
        if RELIABLE.contains(&recipe_idx) {
            Self::Full
        } else if MOSTLY_RELIABLE.contains(&recipe_idx) {
            Self::Mostly
        } else if recipe_idx == 381 {
            Self::ZAoe
        } else if RARITY.contains(&recipe_idx) {
            Self::Rarity
        } else if UNSUPPORTED.contains(&recipe_idx) {
            Self::Unsupported
        } else {
            Self::Low
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SigilCraft {
    pub sigil_type: SigilType,
    pub normal_rolls: Vec<SigilCraftRoll>,
    pub not_blacklisted: Vec<SigilSkill>,
    pub budget: RangeInclusive<u8>,
    pub support: RecipeSupport,
    pub recipe_idx: usize,
}

impl SigilCraft {
    pub fn from_bytes(
        rolls_bytes: &[u8],
        blacklist_bytes: &[u8],
        recipe_bytes: &[u8],
        recipe_idx: usize,
    ) -> Self {
        assert!(rolls_bytes.len() == 64);
        assert!(recipe_bytes.len() == 40);
        let mut normal_rolls = Vec::new();
        let mut not_blacklisted = Vec::new();
        for i in 0..8 {
            let start = i * 8;
            let roll = SigilCraftRoll::from_bytes(&rolls_bytes[start..start + 8]);
            if roll.skill_roll.skill == SigilSkill::Empty {
                break;
            }
            if roll.probability != 0 || roll.guaranteed {
                normal_rolls.push(roll);
            }
        }
        let blacklist: Vec<SigilSkill> = blacklist_bytes
            .chunks_exact(2)
            .map(|skill_bytes| {
                let id = u16::from_le_bytes(skill_bytes.try_into().unwrap());
                SigilSkill::from_repr(id).unwrap()
            })
            .collect();

        let sigil_type = SigilType::from_repr(recipe_bytes[0]).unwrap();
        if sigil_type == SigilType::Normal || sigil_type == SigilType::Premium {
            for skill in SigilSkill::iter().skip(1).take(112) {
                if !normal_rolls
                    .iter()
                    .any(|roll| roll.skill_roll.skill == skill)
                    && !blacklist.contains(&skill)
                {
                    not_blacklisted.push(skill);
                }
            }
        }
        let budget = recipe_bytes[1]..=recipe_bytes[2];
        let support = RecipeSupport::from_idx(recipe_idx);
        Self {
            sigil_type,
            normal_rolls,
            not_blacklisted,
            budget,
            support,
            recipe_idx,
        }
    }

    pub fn roll(&self, rng: &mut Lcg, base_ranges: &[BaseSkillRanges]) -> Sigil {
        match self.sigil_type {
            SigilType::Normal => self.normal_roll(rng, base_ranges),
            SigilType::Premium => self.normal_roll(rng, base_ranges),
            SigilType::Zenith => self.zenith_roll(rng, base_ranges),
        }
    }

    fn normal_roll(&self, rng: &mut Lcg, base_ranges: &[BaseSkillRanges]) -> Sigil {
        let mut current_skills = Vec::new();
        let mut current_points_ranges = Vec::new();
        let mut remaining_budget = *self.budget.end();

        for _ in 0..3 {
            let mut total_chance = 0;
            let budget_rng = rng.advance();
            let budget_value = (budget_rng % remaining_budget as u32) as u8;
            let roll_rng = rng.advance();
            let roll_value = (roll_rng % 100) as u8;
            if !self.normal_rolls.iter().any(|roll| {
                total_chance += roll.probability;
                if (roll_value < total_chance || roll.guaranteed)
                    && !roll.skill_roll.skill.invalid_duplicate(
                        &base_ranges[roll.skill_roll.skill as usize],
                        &current_skills,
                    )
                {
                    current_skills.push(roll.skill_roll.skill);
                    current_points_ranges.push(roll.skill_roll.points.clone());
                    let budget = base_ranges[roll.skill_roll.skill as usize]
                        .points_ranges
                        .iter()
                        .find(|points_range| {
                            // Arbitrarily assume highest point roll and hope it works out.
                            points_range
                                .points_range
                                .contains(roll.skill_roll.points.end())
                        })
                        .unwrap()
                        .budget;
                    remaining_budget = remaining_budget.saturating_sub(budget);
                    true
                } else {
                    false
                }
            }) && let Some((skill, range_budget)) =
                self.handle_blacklist_roll(rng, base_ranges, budget_value, &current_skills)
            {
                current_skills.push(skill);
                current_points_ranges.push(range_budget.points_range);
                remaining_budget = remaining_budget.saturating_sub(range_budget.budget);
            }
            if remaining_budget == 0 {
                break;
            }
        }
        let skill_rolls = current_skills
            .into_iter()
            .zip(current_points_ranges)
            .map(|(skill, points)| SkillRoll::new(skill, points))
            .collect();
        Sigil::from_skill_rolls(skill_rolls, base_ranges)
    }

    fn handle_blacklist_roll(
        &self,
        rng: &mut Lcg,
        base_ranges: &[BaseSkillRanges],
        budget: u8,
        current_rolls: &[SigilSkill],
    ) -> Option<(SigilSkill, SkillPointsRangeBudget)> {
        // Can't actually know for sure what our remaining budget is
        // which means the random budget used here is also often wrong.
        // This roll is built on hopes and prayers
        let skill_choices_len = self.not_blacklisted.len();
        for _ in 0..skill_choices_len {
            let skill_choice_rng = rng.advance();
            let choice_idx = skill_choice_rng as usize % skill_choices_len;
            let skill = &self.not_blacklisted[choice_idx];
            if skill.invalid_duplicate(&base_ranges[*skill as usize], current_rolls) {
                continue;
            }
            let skill_ranges = &base_ranges[*skill as usize];
            let available_ranges: Vec<_> = skill_ranges
                .points_ranges
                .iter()
                .filter(|range_budget| range_budget.budget != 0 && range_budget.budget < budget)
                .collect();
            if !available_ranges.is_empty() {
                let range_choise_rng = rng.advance();
                let chosen_range_idx = range_choise_rng as usize % available_ranges.len();
                let chosen_range = available_ranges[chosen_range_idx];
                return Some((*skill, chosen_range.clone()));
            }
        }
        None
    }

    fn zenith_roll(&self, rng: &mut Lcg, base_ranges: &[BaseSkillRanges]) -> Sigil {
        if self.support == RecipeSupport::ZAoe {
            return self.zenith_aoe_roll(rng, base_ranges);
        }
        let cooldown_rng = rng.advance();
        let mut cooldown_value = (cooldown_rng % 100) as u8;
        let cooldown = self
            .normal_rolls
            .iter()
            .rev()
            .filter(|roll| roll.skill_roll.skill == SigilSkill::ZCooldown)
            .find(|roll| {
                let found = cooldown_value < roll.probability;
                cooldown_value = cooldown_value.saturating_sub(roll.probability);
                found
            })
            .map(|roll| roll.skill_roll.clone())
            .unwrap();

        let duration_rng = rng.advance();
        let mut duration_value = (duration_rng % 100) as u8;
        let duration = self
            .normal_rolls
            .iter()
            .rev()
            .filter(|roll| roll.skill_roll.skill == SigilSkill::ZDuration)
            .find(|roll| {
                let found = duration_value < roll.probability;
                duration_value = duration_value.saturating_sub(roll.probability);
                found
            })
            .map(|roll| roll.skill_roll.clone())
            .unwrap();

        let skill_rng = rng.advance();
        let mut skill_value = (skill_rng % 100) as u8;
        let skill = self
            .normal_rolls
            .iter()
            .rev()
            .filter(|roll| {
                ![SigilSkill::ZCooldown, SigilSkill::ZDuration].contains(&roll.skill_roll.skill)
            })
            .find(|roll| {
                let found = skill_value < roll.probability;
                skill_value = skill_value.saturating_sub(roll.probability);
                found
            })
            .map(|roll| roll.skill_roll.clone())
            .unwrap();

        Sigil::from_skill_rolls(vec![cooldown, duration, skill], base_ranges)
    }

    fn zenith_aoe_roll(&self, rng: &mut Lcg, base_ranges: &[BaseSkillRanges]) -> Sigil {
        // Skill choice can't be predicted so this only exists as a reference for what point ranges you can expect
        let rolls = (0..3)
            .map(|_| {
                let range_rng = rng.advance();
                let mut range_value = (range_rng % 100) as u8;
                self.normal_rolls
                    .iter()
                    .rev()
                    .find(|roll| {
                        let found = range_value < roll.probability;
                        range_value = range_value.saturating_sub(roll.probability);
                        found
                    })
                    .map(|roll| roll.skill_roll.clone())
                    .unwrap()
            })
            .collect();
        let mut sigil = Sigil::from_skill_rolls(rolls, base_ranges);
        if sigil.red_roll == RedRoll::No {
            // AoE sigil skills without points are always red, so the no case can't actually be predicted
            sigil.red_roll = RedRoll::Maybe;
        }
        sigil
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SigilCraftRoll {
    pub skill_roll: SkillRoll,
    pub probability: u8,
    pub guaranteed: bool,
}

impl SigilCraftRoll {
    fn from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() == 8);
        let skill_id = u16::from_le_bytes([bytes[0], bytes[1]]);
        assert!(skill_id < 132);
        let points_start = i16::from_le_bytes([bytes[4], bytes[5]]);
        let points_end = i16::from_le_bytes([bytes[6], bytes[7]]);
        let skill_roll = SkillRoll::new(
            SigilSkill::from_repr(skill_id).unwrap(),
            points_start..=points_end,
        );
        Self {
            skill_roll,
            probability: bytes[2],
            guaranteed: bytes[3] != 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RedRollRequirement {
    Threshold(i16),
    Specific(Vec<i16>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BaseSkillRanges {
    pub points_ranges: Vec<SkillPointsRangeBudget>,
    pub red_roll: Option<RedRollRequirement>,
}

impl BaseSkillRanges {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() == 0x58);
        let points_ranges = (0..8)
            .filter_map(|i| {
                let start = i * 8;
                let range_budget = SkillPointsRangeBudget::from_bytes(&bytes[start..start + 8]);
                if *range_budget.points_range.start() == 0 && *range_budget.points_range.end() == 0
                {
                    None
                } else {
                    Some(range_budget)
                }
            })
            .collect();
        let red_roll = if bytes[0x44] == 0 {
            None
        } else {
            if bytes[0x48] == 0 {
                Some(RedRollRequirement::Threshold(i16::from_le_bytes([
                    bytes[0x46],
                    bytes[0x47],
                ])))
            } else {
                Some(RedRollRequirement::Specific(
                    (0..7)
                        .map_while(|i| {
                            let base = 0x48 + i * 2;
                            if bytes[base] == 0 {
                                None
                            } else {
                                Some(i16::from_le_bytes([bytes[base], bytes[base + 1]]))
                            }
                        })
                        .collect(),
                ))
            }
        };
        Self {
            points_ranges,
            red_roll,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPointsRangeBudget {
    pub points_range: RangeInclusive<i16>,
    pub budget: u8,
}

impl SkillPointsRangeBudget {
    fn from_bytes(bytes: &[u8]) -> Self {
        assert!(bytes.len() == 8);
        let points_start = i16::from_le_bytes([bytes[0], bytes[1]]);
        let points_end = i16::from_le_bytes([bytes[2], bytes[3]]);
        Self {
            points_range: points_start..=points_end,
            budget: bytes[4],
        }
    }
}
