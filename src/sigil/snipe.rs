use std::fmt::Display;

use eframe::egui::Ui;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    rng::Lcg,
    sigil::{
        calc::{BaseSkillRanges, RedRoll, Sigil, SigilCraft, SigilType, SkillRoll},
        recipe::{rarity_recipe_name, zenith_heroics_recipe_name, zenith_resistance_recipe_name},
    },
    ui::Language,
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

pub struct RarityIterator(usize);

impl RarityIterator {
    pub fn new(advances: usize) -> Self {
        Self(advances)
    }
}

impl Iterator for RarityIterator {
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

impl AfterSnipeOffset {
    fn new(sigil: &Sigil, rng: &Lcg) -> Self {
        match sigil.red_roll {
            RedRoll::No => AfterSnipeOffset::Certain(rng.peek_low_byte(1)),
            RedRoll::Maybe => {
                AfterSnipeOffset::Uncertain((rng.peek_low_byte(1), rng.peek_low_byte(2)))
            }
            RedRoll::Yes => AfterSnipeOffset::Certain(rng.peek_low_byte(2)),
        }
    }

    pub fn ui(&self, ui: &mut Ui) -> Option<u8> {
        match self {
            AfterSnipeOffset::Certain(offset) => {
                if ui.button(offset.to_string()).clicked() {
                    return Some(*offset);
                }
            }
            AfterSnipeOffset::Uncertain((no_red, red)) => {
                if ui.button(format!("No red roll: {}", no_red)).clicked() {
                    return Some(*no_red);
                }
                if ui.button(format!("Red roll: {}", red)).clicked() {
                    return Some(*red);
                }
            }
        }
        None
    }
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
pub enum ResultSequence {
    Rarity(RaritySequence),
    ZenithRes(ZenithResistanceSequence),
    ZenithHeroics(ZenithHeroicsSequence),
}

impl ResultSequence {
    pub fn ui(&self, ui: &mut Ui, language: Language) {
        match self {
            ResultSequence::Rarity(rarity_sequence) => rarity_sequence.ui(ui, language),
            ResultSequence::ZenithRes(resistance_sequence) => resistance_sequence.ui(ui, language),
            ResultSequence::ZenithHeroics(heroics_sequence) => heroics_sequence.ui(ui, language),
        }
    }
}

#[derive(Debug)]
pub struct RaritySequence {
    pub full_batches: usize,
    pub end_sequence: Vec<usize>,
}

impl RaritySequence {
    fn calculate(advances: usize) -> Self {
        let mut full_batches = 0;
        let mut end_sequence = Vec::new();
        for crafts in RarityIterator::new(advances) {
            if crafts == 10 {
                full_batches += 1;
            } else {
                end_sequence.push(crafts);
            }
        }
        Self {
            full_batches,
            end_sequence,
        }
    }

    fn ui(&self, ui: &mut Ui, language: Language) {
        let rarity_name = rarity_recipe_name(language);
        if self.full_batches > 0 {
            ui.label(format!("10x {} - {} times", rarity_name, self.full_batches));
        }
        let end_sequence: Vec<String> = self
            .end_sequence
            .iter()
            .map(|craft_count| format!("{}x {}", craft_count, rarity_name))
            .collect();
        if !end_sequence.is_empty() {
            let joined = end_sequence.join(" -> ");
            ui.label(joined);
        }
    }
}

struct HeroicsIterator(usize);

impl HeroicsIterator {
    fn new(advances: usize) -> Self {
        Self(advances)
    }
}

impl Iterator for HeroicsIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            0..5 => None,
            6..8 => None,
            9 => None,
            12 => None,
            5 | 8 | 10..12 | 13..33 => {
                let count = match self.0 % 3 {
                    0 => self.0 / 3 - 4,
                    1 => self.0 / 3 - 2,
                    2 => self.0 / 3,
                    _ => 0,
                };
                self.0 -= count * 3 + 2;
                Some(count)
            }
            33..45 => {
                self.0 -= 20;
                Some(6)
            }
            45.. => {
                self.0 -= 32;
                Some(10)
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct ZenithHeroicsSequence {
    pub full_batches: usize,
    pub end_sequence: Vec<usize>,
}

impl ZenithHeroicsSequence {
    fn calculate(advances: usize) -> Self {
        let mut full_batches = 0;
        let mut end_sequence = Vec::new();
        for count in HeroicsIterator::new(advances) {
            if count == 10 {
                full_batches += 1;
            } else {
                end_sequence.push(count);
            }
        }
        Self {
            full_batches,
            end_sequence,
        }
    }

    fn ui(&self, ui: &mut Ui, language: Language) {
        let heroics_name = zenith_heroics_recipe_name(language);
        if self.full_batches > 0 {
            ui.label(format!(
                "10x {} - {} times",
                heroics_name, self.full_batches
            ));
        }
        let end_sequence: Vec<String> = self
            .end_sequence
            .iter()
            .map(|craft_count| format!("{}x {}", craft_count, heroics_name))
            .collect();
        if !end_sequence.is_empty() {
            let joined = end_sequence.join(" -> ");
            ui.label(joined);
        }
    }
}

#[derive(Debug, Default)]
pub struct ZenithResistanceSequence {
    pub resistance_batches: usize,
    pub resistance_end: usize,
    pub resistance_extra: Vec<usize>,
    pub heroics_end: Vec<usize>,
}

impl ZenithResistanceSequence {
    fn calculate(mut advances: usize, seed: u32) -> Self {
        // Strategy: Spam 10 batch variable red roll resistance crafts until in one batch distance of 13-17 advances left.
        // Calculate a single batch that gets to 13-17 advances.
        // From here, check if it's possible to reach 0, 5, 8, 10 or 11 advances left with resistance batches.
        // Finish with an always red roll heroics sequence.

        fn resistance_roll(rng: &mut Lcg) -> bool {
            rng.advance();
            rng.advance();
            let resistance_roll = rng.advance() % 100;
            resistance_roll >= 92
        }

        if advances < 5 {
            return Self::default();
        }

        let mut rng = Lcg::new(seed);
        let mut resistance_batches = 0;
        while advances > 44 {
            advances -= 31;
            resistance_batches += 1;
            let mut red = false;
            for _ in 0..10 {
                red |= resistance_roll(&mut rng);
            }
            rng.advance();
            if red {
                advances -= 1;
                rng.advance();
            }
        }
        let resistance_end = advances.saturating_sub(15) / 3;
        if resistance_end > 0 {
            advances -= 3 * resistance_end + 1;
            let mut red = false;
            for _ in 0..resistance_end {
                red |= resistance_roll(&mut rng);
            }
            rng.advance();
            if red {
                advances -= 1;
                rng.advance();
            }
        }
        let (resistance_extra, advances_left) = find_zenith_extra_sequence(&rng, advances);
        advances = advances_left;

        let heroics_end = zenith_end(advances);

        Self {
            resistance_batches,
            resistance_end,
            resistance_extra,
            heroics_end,
        }
    }

    fn ui(&self, ui: &mut Ui, language: Language) {
        let res_name = zenith_resistance_recipe_name(language);
        if self.resistance_batches > 0 {
            ui.label(format!(
                "10x {} - {} times",
                res_name, self.resistance_batches
            ));
        }
        let mut sequence = Vec::new();
        if self.resistance_end > 0 {
            sequence.push(format!("{}x {}", self.resistance_end, res_name));
        }
        sequence.extend(
            self.resistance_extra
                .iter()
                .map(|craft_count| format!("{}x {}", craft_count, res_name)),
        );
        sequence.extend(self.heroics_end.iter().map(|craft_count| {
            format!("{}x {}", craft_count, zenith_heroics_recipe_name(language))
        }));
        if !sequence.is_empty() {
            let joined = sequence.join(" -> ");
            ui.label(joined);
        }
    }
}

#[derive(Debug)]
pub struct WrongResult {
    pub seed: u32,
    pub advances: usize,
    pub sigil: Sigil,
    pub offset: AfterSnipeOffset,
}

impl WrongResult {
    fn new(
        seed: u32,
        advances: usize,
        recipe: &SigilCraft,
        base_ranges: &[BaseSkillRanges],
    ) -> Self {
        let mut rng = Lcg::new(seed);
        rng.advance_n(advances);
        let sigil = recipe.roll(&mut rng, base_ranges);
        let offset = AfterSnipeOffset::new(&sigil, &rng);
        Self {
            seed,
            advances,
            sigil,
            offset,
        }
    }
}

#[derive(Debug)]
pub struct SnipeResult {
    pub seed: u32,
    pub advances: usize,
    pub offset: AfterSnipeOffset,
    pub sequence: ResultSequence,
    pub early_sigils: Vec<WrongResult>,
    pub late_sigils: Vec<WrongResult>,
}

impl SnipeResult {
    fn new(
        seed: u32,
        advances: usize,
        offset: AfterSnipeOffset,
        recipe: &SigilCraft,
        base_ranges: &[BaseSkillRanges],
        early_late_count: u32,
        zenith_strategy: ZenithStrategy,
    ) -> Self {
        let sequence = match recipe.sigil_type {
            SigilType::Normal => {
                let sequence = RaritySequence::calculate(advances);
                ResultSequence::Rarity(sequence)
            }
            SigilType::Premium => {
                panic!("Premium sigils not supported");
            }
            SigilType::Zenith => match zenith_strategy {
                ZenithStrategy::Resistance => {
                    let sequence = ZenithResistanceSequence::calculate(advances, seed);
                    ResultSequence::ZenithRes(sequence)
                }
                ZenithStrategy::Heroics => {
                    let sequence = ZenithHeroicsSequence::calculate(advances);
                    ResultSequence::ZenithHeroics(sequence)
                }
            },
        };
        let early_sigils = (seed - early_late_count..seed)
            .map(|seed| WrongResult::new(seed, advances, recipe, base_ranges))
            .collect();
        let late_sigils = (seed + 1..=seed + early_late_count)
            .map(|seed| WrongResult::new(seed, advances, recipe, base_ranges))
            .collect();
        Self {
            seed,
            advances,
            offset,
            sequence,
            early_sigils,
            late_sigils,
        }
    }
}

fn find_zenith_extra_sequence(rng: &Lcg, advances: usize) -> (Vec<usize>, usize) {
    let mut rng = *rng;

    match advances {
        13 => {
            let ten = { rng.advance_n(3) >= 92 };
            let seven = { rng.advance_n(3) >= 92 };
            let six = rng.advance() >= 92;
            let four = { rng.advance_n(2) >= 92 };
            let two = { rng.advance_n(2) >= 92 };
            let one = rng.advance() >= 92;

            let double = ten || seven;
            if double && two {
                return (vec![2, 1], 0);
            }
            if !ten && !seven && !four && !one {
                return (vec![4], 0);
            }
            if double {
                return (vec![2], 5);
            }
            if !ten && !six {
                return (vec![1, 1], 5);
            }
        }
        14 => {
            let eleven = { rng.advance_n(3) >= 92 };
            let eight = { rng.advance_n(3) >= 92 };
            let five = { rng.advance_n(3) >= 92 };
            let two = { rng.advance_n(3) >= 92 };
            let one = rng.advance() >= 92;

            if eleven || eight || five || two {
                return (vec![4], 0);
            }
            if !eleven && !eight && !five && !one {
                return (vec![3, 1], 0);
            }
        }
        15 => {
            let twelve = { rng.advance_n(3) >= 92 };
            let nine = { rng.advance_n(3) >= 92 };
            let seven = { rng.advance_n(2) >= 92 };
            let six = rng.advance() >= 92;
            let five = rng.advance() >= 92;
            let two = { rng.advance_n(3) >= 92 };
            let one = rng.advance() >= 92;

            if twelve && seven && two {
                return (vec![1, 1, 1], 0);
            }
            if !twelve && !nine && !five && !one {
                return (vec![2, 1, 1], 0);
            }
            if twelve && seven {
                return (vec![1, 1], 5);
            }
            if !twelve && !nine {
                if !six {
                    return (vec![3], 5);
                }
                return (vec![2], 8);
            }
            if twelve {
                return (vec![1], 10);
            } else {
                return (vec![1], 11);
            }
        }
        16 => {
            let thirteen = { rng.advance_n(3) >= 92 };
            let ten = { rng.advance_n(3) >= 92 };
            let nine = rng.advance() >= 92;
            let seven = { rng.advance_n(2) >= 92 };
            let six = rng.advance() >= 92;
            let four = { rng.advance_n(2) >= 92 };
            let two = { rng.advance_n(2) >= 92 };
            let one = rng.advance() >= 92;

            let triple = thirteen || ten || seven;
            if triple && two {
                return (vec![3, 1], 0);
            }
            if !thirteen && !ten {
                if !seven && !four && !one {
                    return (vec![5], 0);
                } else if !six {
                    return (vec![2, 1], 5);
                }
            }
            if triple {
                return (vec![3], 5);
            }
            if !thirteen && !nine {
                return (vec![1, 1], 8);
            }
        }
        17 => {
            let fourteen = { rng.advance_n(3) >= 92 };
            let eleven = { rng.advance_n(3) >= 92 };
            let ten = rng.advance() >= 92;
            let eight = { rng.advance_n(2) >= 92 };
            let six = { rng.advance_n(2) >= 92 };
            let five = rng.advance() >= 92;
            let two = { rng.advance_n(3) >= 92 };
            let one = rng.advance() >= 92;

            if fourteen || eleven || eight || five || two {
                return (vec![5], 0);
            }
            if !fourteen {
                if !eleven && !eight && !five && !one {
                    return (vec![4, 1], 0);
                }
                if !ten && !six {
                    return (vec![1, 1, 1], 5);
                }
                if !eleven {
                    return (vec![2], 10);
                }
                return (vec![1], 13);
            }
        }
        _ => {}
    }
    (Vec::new(), advances)
}

fn zenith_end(advances: usize) -> Vec<usize> {
    match advances {
        5 => vec![1],
        8 => vec![2],
        10 => vec![1, 1],
        11 => vec![3],
        13 => vec![2, 1],
        14 => vec![4],
        15 => vec![1, 1, 1],
        16 => vec![3, 1],
        17 => vec![5],
        _ => Vec::new(),
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
        Some(AfterSnipeOffset::new(&sigil, &rng))
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZenithStrategy {
    Resistance,
    Heroics,
}

pub fn snipe(
    recipe: &SigilCraft,
    base_ranges: &[BaseSkillRanges],
    starting_seed: u32,
    seeds_to_search: usize,
    depth_per_seed: usize,
    requirements: &Requirements,
    zenith_strategy: ZenithStrategy,
) -> Vec<SnipeResult> {
    let early_advances = match recipe.sigil_type {
        SigilType::Normal => vec![0, 3, 5, 6, 7, 8, 9, 10, 11, 12],
        SigilType::Premium => panic!("Premium sigil unsupported"),
        SigilType::Zenith => vec![0, 5, 8, 10, 11],
    };
    (0..seeds_to_search)
        .into_par_iter()
        .flat_map(|i| {
            let mut results = Vec::new();
            let mut advances = 0;
            let seed = starting_seed + i as u32;
            let mut rng = Lcg::new(seed);

            let start_depth = 13.min(depth_per_seed + 1);
            for i in 0..start_depth {
                if early_advances.contains(&i)
                    && let Some(offset) = roll(&rng, recipe, base_ranges, requirements)
                {
                    results.push(SnipeResult::new(
                        seed,
                        advances,
                        offset,
                        recipe,
                        base_ranges,
                        3,
                        zenith_strategy,
                    ));
                }
                rng.advance();
                advances += 1;
            }

            for _ in 13..depth_per_seed {
                if let Some(offset) = roll(&rng, recipe, base_ranges, requirements) {
                    results.push(SnipeResult::new(
                        seed,
                        advances,
                        offset,
                        recipe,
                        base_ranges,
                        3,
                        zenith_strategy,
                    ));
                }
                rng.advance();
                advances += 1;
            }
            results
        })
        .collect()
}
