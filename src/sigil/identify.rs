use std::ops::RangeInclusive;

use crate::{
    rng::Lcg,
    sigil::{
        calc::{BaseSkillRanges, RedRollRequirement, Sigil, SigilCraft},
        sigil_skills::SigilSkill,
    },
};

#[derive(Debug, Clone, Copy)]
pub struct OffsetSigilRoll {
    pub skill: SigilSkill,
    pub points: i16,
}

impl OffsetSigilRoll {
    pub fn new(skill: SigilSkill) -> Self {
        Self { skill, points: 1 }
    }

    fn red_roll(&self, ranges: &BaseSkillRanges) -> bool {
        if let Some(red_req) = &ranges.red_roll {
            match red_req {
                RedRollRequirement::Threshold(threshold) => self.points >= *threshold,
                RedRollRequirement::Specific(red_points) => red_points.contains(&self.points),
            }
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OffsetSigil {
    pub rolls: [Option<OffsetSigilRoll>; 3],
}

impl OffsetSigil {
    pub fn new() -> Self {
        Self { rolls: [None; 3] }
    }
}

struct ReferenceRoll {
    skill: SigilSkill,
    possible_points_ranges: Vec<RangeInclusive<i16>>,
}

impl ReferenceRoll {
    fn new(skill: SigilSkill, points_ranges: Vec<RangeInclusive<i16>>) -> Self {
        Self {
            skill,
            possible_points_ranges: points_ranges,
        }
    }
}

pub struct ReferenceSigil {
    rolls: Vec<ReferenceRoll>,
    red_roll: bool,
}

impl ReferenceSigil {
    pub fn from_crafted_sigil(
        sigil: &OffsetSigil,
        recipe: &SigilCraft,
        base_ranges: &[BaseSkillRanges],
    ) -> Self {
        let mut red_roll = false;
        let rolls: Vec<ReferenceRoll> = sigil
            .rolls
            .iter()
            .flatten()
            .map(|roll| {
                let mut possible_ranges: Vec<RangeInclusive<i16>> = recipe
                    .normal_rolls
                    .iter()
                    .filter_map(|craft_skill| {
                        if craft_skill.skill_roll.skill != roll.skill {
                            return None;
                        }
                        if craft_skill.skill_roll.points.contains(&roll.points) {
                            Some(craft_skill.skill_roll.points.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                let other_ranges = recipe
                    .not_blacklisted
                    .iter()
                    .filter_map(|non_standard_skill| {
                        if *non_standard_skill != roll.skill {
                            return None;
                        }
                        let ranges = &base_ranges[*non_standard_skill as usize];
                        Some(ranges.points_ranges.iter().filter_map(|range| {
                            if range.points_range.contains(&roll.points) {
                                Some(range.points_range.clone())
                            } else {
                                None
                            }
                        }))
                    })
                    .flatten();
                possible_ranges.extend(other_ranges);

                if roll.red_roll(&base_ranges[roll.skill as usize]) {
                    red_roll = true;
                }
                ReferenceRoll::new(roll.skill, possible_ranges)
            })
            .collect();
        Self { rolls, red_roll }
    }
}

impl PartialEq<Sigil> for ReferenceSigil {
    fn eq(&self, other: &Sigil) -> bool {
        self.rolls.len() == other.rolls.len()
            && other.rolls.iter().enumerate().all(|(i, skill_roll)| {
                let reference_roll = &self.rolls[i];
                reference_roll.skill == skill_roll.skill
                    && reference_roll
                        .possible_points_ranges
                        .contains(&skill_roll.points)
            })
    }
}

pub enum IdentifyResult {
    NotFound,
    OneResult((u32, u8)),
    MultipleResults(Vec<(u32, u8)>),
}

pub fn identify_seed_offset(
    sigil_sequence: &[ReferenceSigil],
    starting_seed: u32,
    recipe: &SigilCraft,
    base_ranges: &[BaseSkillRanges],
) -> IdentifyResult {
    let sequence_length = sigil_sequence.len();
    let mut results = Vec::new();
    // Account for timestamp provided by user being slightly wrong
    for seed in starting_seed - 10..starting_seed + 266 {
        let mut rng = Lcg::new(seed);
        if (0..sequence_length).all(|sigil_i| {
            let sigil = recipe.roll(&mut rng, base_ranges);
            let reference = &sigil_sequence[sigil_i];
            rng.advance();
            if reference.red_roll {
                rng.advance();
            }
            reference == &sigil
        }) {
            results.push((seed, rng.low_byte()));
        }
    }
    match results.len() {
        0 => IdentifyResult::NotFound,
        1 => IdentifyResult::OneResult(results[0]),
        _ => IdentifyResult::MultipleResults(results),
    }
}
