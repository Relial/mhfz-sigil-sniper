use std::{
    env::current_exe,
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use strum::EnumCount as _;

use crate::sigil::{
    calc::{BaseSkillRanges, SigilCraft},
    sigil_skills::SigilSkill,
};

#[derive(Serialize, Deserialize)]
pub struct SigilData {
    pub recipes: Vec<SigilCraft>,
    pub skill_ranges: Vec<BaseSkillRanges>,
}

impl SigilData {
    pub fn load() -> Result<Self> {
        let dat_path = if cfg!(debug_assertions) {
            PathBuf::from("./dat/dat.bin")
        } else {
            let exe = current_exe()?;
            exe.parent()
                .ok_or(anyhow!("Failed to get exe parent dir"))?
                .join("dat/dat.bin")
        };
        let file = File::open(dat_path)?;
        let mut buf = [0; 2048];
        let (data, _) = postcard::from_io::<SigilData, _>((file, &mut buf))?;
        Ok(data)
    }

    pub fn save(&self) -> Result<()> {
        let dat_path = PathBuf::from("./dat/dat.bin");
        let file = File::create(dat_path)?;
        postcard::to_io(self, file)?;
        Ok(())
    }

    pub fn from_mhfdat(mhfdat_path: &Path) -> Result<Self> {
        let (recipes, skill_ranges): (Vec<SigilCraft>, Vec<BaseSkillRanges>) = {
            let mhfdat_path = PathBuf::from("./mhfdat.bin");
            let mhfdat = Mhfdat::load(&mhfdat_path)?;
            let recipes = (0..)
                .map_while(|i| {
                    let recipe_bytes = mhfdat.recipe_bytes(i);
                    if recipe_bytes[0..4] == [0x00, 0x00, 0x00, 0x00] {
                        return None;
                    }
                    let rolls_bytes = mhfdat.rolls_bytes(i);
                    let blacklist_bytes = mhfdat.blacklist_bytes(i);
                    let recipe =
                        SigilCraft::from_bytes(rolls_bytes, blacklist_bytes, recipe_bytes, i);
                    Some(recipe)
                })
                .collect();
            let ranges = mhfdat
                .base_rolls_budget_bytes()
                .iter()
                .map(|range_bytes| BaseSkillRanges::from_bytes(range_bytes))
                .collect();
            (recipes, ranges)
        };
        Ok(SigilData {
            recipes,
            skill_ranges,
        })
    }
}

const RECIPES_PTR: usize = 0x620;
const SKILL_ROLLS_PTR: usize = 0x624;
const BLACKLISTS_PTR: usize = 0x628;
const BASE_ROLLS_BUDGETS_PTR: usize = 0x62C;

pub struct Mhfdat(Vec<u8>);

impl Mhfdat {
    pub fn load(path: &Path) -> Result<Self> {
        let bytes = std::fs::read(path)?;
        Ok(Self(bytes))
    }

    pub fn recipe_bytes(&self, recipe_idx: usize) -> &[u8] {
        let recipes_offset =
            u32::from_le_bytes(self.0[RECIPES_PTR..RECIPES_PTR + 4].try_into().unwrap()) as usize;
        let specific = recipes_offset + recipe_idx * 40;
        &self.0[specific..specific + 40]
    }

    pub fn rolls_bytes(&self, recipe_idx: usize) -> &[u8] {
        let rolls_offset = u32::from_le_bytes(
            self.0[SKILL_ROLLS_PTR..SKILL_ROLLS_PTR + 4]
                .try_into()
                .unwrap(),
        ) as usize;
        let specific = rolls_offset + recipe_idx * 64;
        &self.0[specific..specific + 64]
    }

    pub fn blacklist_bytes(&self, recipe_idx: usize) -> &[u8] {
        let blacklists_offset = u32::from_le_bytes(
            self.0[BLACKLISTS_PTR..BLACKLISTS_PTR + 4]
                .try_into()
                .unwrap(),
        ) as usize;
        let specific_offset = blacklists_offset + recipe_idx * 4;
        let specific = u32::from_le_bytes(
            self.0[specific_offset..specific_offset + 4]
                .try_into()
                .unwrap(),
        ) as usize;
        let end = self.0[specific..]
            .chunks_exact(2)
            .position(|bytes| bytes == [0xFF, 0xFF])
            .unwrap();
        &self.0[specific..specific + end * 2]
    }

    pub fn base_rolls_budget_bytes(&self) -> Vec<&[u8]> {
        let base_rolls_offset = u32::from_le_bytes(
            self.0[BASE_ROLLS_BUDGETS_PTR..BASE_ROLLS_BUDGETS_PTR + 4]
                .try_into()
                .unwrap(),
        ) as usize;
        let skill_count = SigilSkill::COUNT;
        self.0[base_rolls_offset..]
            .chunks_exact(88)
            .take(skill_count)
            .collect()
    }
}
