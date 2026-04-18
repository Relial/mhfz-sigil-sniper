use std::fmt::Display;

use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    prng::Lcg,
    sigil::calc::{BaseSkillRanges, RedRoll, SigilCraft, SkillRoll},
};

#[derive(Debug, Clone)]
pub struct Requirement {
    pub skill_roll: SkillRoll,
    count: u8,
}

impl Requirement {
    pub fn new(skill_roll: SkillRoll) -> Self {
        Self {
            skill_roll,
            count: 1,
        }
    }
}

impl PartialEq for Requirement {
    fn eq(&self, other: &Self) -> bool {
        self.skill_roll == other.skill_roll
    }
}

#[derive(Debug, Clone)]
pub struct Requirements {
    requirements: [Option<Requirement>; 3],
    count: usize,
    not_done: usize,
}

impl Requirements {
    pub fn from_skill_rolls(skill_rolls: &[SkillRoll]) -> Self {
        let mut new = Self {
            requirements: [const { None }; 3],
            count: 0,
            not_done: 0,
        };
        for roll in skill_rolls {
            new.add(Requirement::new(roll.clone()));
        }
        new
    }

    fn add(&mut self, requirement: Requirement) {
        if self.count < 3 {
            if let Some(existing) = self
                .requirements
                .iter_mut()
                .flatten()
                .find(|req| **req == requirement)
            {
                existing.count += 1;
                self.count += 1;
            } else {
                let missing = self
                    .requirements
                    .iter_mut()
                    .find(|opt| opt.is_none())
                    .unwrap();
                *missing = Some(requirement);
                self.not_done += 1;
                self.count += 1;
            }
        }
    }

    fn done(&self) -> bool {
        self.not_done == 0
    }

    fn fulfill(&mut self, skill_roll: &SkillRoll) {
        if let Some(req) = self
            .requirements
            .iter_mut()
            .flatten()
            .find(|req| req.count > 0 && req.skill_roll == *skill_roll)
        {
            req.count -= 1;
            if req.count == 0 {
                self.not_done -= 1;
            }
        }
    }
}

pub struct RaritySequence(usize);

impl RaritySequence {
    pub fn new(advances: usize) -> Self {
        Self(advances)
    }
}

impl Iterator for RaritySequence {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            0..3 => None,
            3 | 5..22 => {
                let curr = self.0;
                match self.0 % 2 {
                    0 => {
                        self.0 = 3;
                        Some((curr - 4) / 2)
                    }
                    1 => {
                        self.0 = 0;
                        Some((curr - 1) / 2)
                    }
                    _ => None,
                }
            }
            22..26 => {
                self.0 -= 17;
                Some(8)
            }
            26.. => {
                self.0 -= 21;
                Some(10)
            }
            4 => None,
        }
    }
}

#[derive(Debug)]
pub enum AfterSnipeOffset {
    Certain(u8),
    Uncertain((u8, u8)),
}

impl Display for AfterSnipeOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AfterSnipeOffset::Certain(offset) => offset.to_string(),
            AfterSnipeOffset::Uncertain((offset_1, offset_2)) => {
                format!("{} | {}", offset_1, offset_2)
            }
        };
        write!(f, "{s}")
    }
}

#[derive(Debug)]
pub struct SnipeResult {
    pub seed: u32,
    pub advances: usize,
    pub offset: AfterSnipeOffset,
    pub rarity_full_batches: usize,
    pub rarity_crafts: Vec<usize>,
}

impl SnipeResult {
    fn new(seed: u32, advances: usize, offset: AfterSnipeOffset) -> Self {
        let mut rarity_full_batches = 0;
        let mut rarity_crafts = Vec::new();
        for crafts in RaritySequence::new(advances) {
            if crafts == 10 {
                rarity_full_batches += 1;
            } else {
                rarity_crafts.push(crafts);
            }
        }
        Self {
            seed,
            advances,
            offset,
            rarity_full_batches,
            rarity_crafts,
        }
    }
}

fn roll(
    rng: &Lcg,
    recipe: &SigilCraft,
    base_ranges: &[BaseSkillRanges],
    requirements: &Requirements,
) -> Option<AfterSnipeOffset> {
    let mut requirements = requirements.clone();
    let mut rng = *rng;
    let sigil = recipe.roll(&mut rng, base_ranges);
    for roll in &sigil.rolls {
        requirements.fulfill(roll);
    }
    if requirements.done() {
        match sigil.red_roll {
            RedRoll::No => Some(AfterSnipeOffset::Certain(rng.peek_low_byte(1))),
            RedRoll::Maybe => Some(AfterSnipeOffset::Uncertain((
                rng.peek_low_byte(1),
                rng.peek_low_byte(2),
            ))),
            RedRoll::Yes => Some(AfterSnipeOffset::Certain(rng.peek_low_byte(2))),
        }
    } else {
        None
    }
}

pub fn snipe(
    recipe: &SigilCraft,
    base_ranges: &[BaseSkillRanges],
    starting_seed: u32,
    seeds_to_search: usize,
    depth_per_seed: usize,
    requirements: &Requirements,
) -> Vec<SnipeResult> {
    (0..seeds_to_search)
        .into_par_iter()
        .flat_map(|i| {
            let mut results = Vec::new();
            let mut advances = 0;
            let seed = starting_seed + i as u32;
            let mut rng = Lcg::new(seed);

            if let Some(offset) = roll(&rng, recipe, base_ranges, requirements) {
                results.push(SnipeResult::new(seed, advances, offset));
            }
            rng.advance();
            rng.advance();
            rng.advance();
            advances += 3;
            if let Some(offset) = roll(&rng, recipe, base_ranges, requirements) {
                results.push(SnipeResult::new(seed, advances, offset));
            }
            rng.advance();
            rng.advance();
            advances += 2;

            for _ in 5..depth_per_seed {
                if let Some(offset) = roll(&rng, recipe, base_ranges, requirements) {
                    results.push(SnipeResult::new(seed, advances, offset));
                }
                rng.advance();
                advances += 1;
            }
            results
        })
        .collect()
}
