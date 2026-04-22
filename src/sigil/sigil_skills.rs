use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};
use strum::{EnumCount, EnumIter, FromRepr};

use crate::{
    sigil::calc::{BaseSkillRanges, SigilCraft},
    ui::Language,
};

pub enum SpecificEffect {
    None,
    Named(&'static str),
    Points,
}

impl From<&'static str> for SpecificEffect {
    fn from(value: &'static str) -> Self {
        Self::Named(value)
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    FromRepr,
    EnumIter,
    EnumCount,
    Serialize,
    Deserialize,
)]
#[repr(u16)]
pub enum SigilSkill {
    Empty = 0,
    Attack = 1,
    Element = 2,
    Status = 3,
    Affinity = 4,
    Defense = 5,
    Stun = 6,
    FlyingWyvernSlayer = 7,
    BirdWyvernSlayer = 8,
    CarapaceonSlayer = 9,
    PiscineSlayer = 10,
    FangedBeastSlayer = 11,
    BruteWyvernSlayer = 12,
    LeviathanSlayer = 13,
    ElderDragonSlayer = 14,
    SmallMonsterSlayer = 15,
    ElementalCoatingCrystal = 16,
    HeatWave = 17,
    IntenseCold = 18,
    MorningSlayer = 19,
    NightSlayer = 20,
    BreedingSeasonSlayer = 21,
    WarmSeasonSlayer = 22,
    ColdSeasonSlayer = 23,
    MondaySlayer = 24,
    TuesdaySlayer = 25,
    WednesdaySlayer = 26,
    ThursdaySlayer = 27,
    FridaySlayer = 28,
    SaturdaySlayer = 29,
    SundaySlayer = 30,
    ManagerFriend = 31,
    BalloonFriend = 32,
    Dowsing = 33,
    HunterSoul = 34,
    SoulCollection = 35,
    GoocooFriend = 36,
    HalkFriend = 37,
    MoneyExpert = 38,
    ExperienceExpert = 39,
    Omnivorous = 40,
    CarvingDivision = 41,
    NoddingOff = 42,
    Decoy = 43,
    Tantrum = 44,
    Unity = 45,
    Support = 46,
    RapidFireAdd = 47,
    SuperRapidFireAdd = 48,
    RapidFireDisable = 49,
    HeatCannonAdd = 50,
    BallistaSaver = 51,
    RawBowBoost = 52,
    RawSnsBoost = 53,
    SonicBombRange = 54,
    TechBoostSnS = 55,
    TechBoostDb = 56,
    TechBoostGs = 57,
    TechBoostLs = 58,
    TechBoostLance = 59,
    TechBoostGunlance = 60,
    TechBoostHammer = 61,
    TechBoostHorn = 62,
    TechChangeSnS = 63,
    TechChangeDb = 64,
    TechChangeGs = 65,
    TechChangeLs = 66,
    TechChangeLance = 67,
    TechChangeGunlance = 68,
    TechChangeHammer = 69,
    TechChangeHorn = 70,
    TechChangeLbg = 71,
    TechChangeHbg = 72,
    TechChangeBow = 73,
    LengthUp = 74,
    ShellingChange = 75,
    NoteChange1 = 76,
    NoteChange2 = 77,
    NoteChange3 = 78,
    ArcShotChange = 79,
    RarityChange = 80,
    ShellingLvUp = 81,
    TechBoostBow = 82,
    CatBreeder = 83,
    ShotChange = 84,
    DrawStrengthGs = 85,
    DrawStrengthHammer = 86,
    DrawStrengthLance = 87,
    DrawStrengthSns = 88,
    DrawStrengthDb = 89,
    DrawStrengthLs = 90,
    DrawStrengthHorn = 91,
    DrawStrengthGunlance = 92,
    DrawStrengthTonfa = 93,
    TechBoostTonfa = 94,
    FangedWyvernSlayer = 95,
    UnsheathedMovementSpeed = 96,
    TechBoostSaf = 97,
    TechBoostHbg = 98,
    GsUp = 99,
    HbgUp = 100,
    HammerUp = 101,
    LanceUp = 102,
    SnsUp = 103,
    LbgUp = 104,
    DbUp = 105,
    LsUp = 106,
    HornUp = 107,
    GunlanceUp = 108,
    BowUp = 109,
    TonfaUp = 110,
    SafUp = 111,
    MsUp = 112,
    ZCooldown = 113,
    ZDuration = 114,
    ZFireRes = 115,
    ZWaterRes = 116,
    ZIceRes = 117,
    ZThunderRes = 118,
    ZDragonRes = 119,
    ZHealing = 120,
    ZHeroics = 121,
    ZAttack = 122,
    ZElement = 123,
    ZMovementSpeed = 124,
    RAttack = 125,
    RElement = 126,
    RAffinity = 127,
    RStatus = 128,
    RStun = 129,
    RHealing = 130,
    RAllRes = 131,
}

impl SigilSkill {
    pub fn name(&self, language: Language) -> &'static str {
        match language {
            Language::English => self.name_en(),
            Language::Japanese => self.name_jp(),
        }
    }

    fn name_en(&self) -> &'static str {
        match self {
            SigilSkill::Empty => "?",
            SigilSkill::Attack => "Attack",
            SigilSkill::Element => "Elemental",
            SigilSkill::Status => "Status Attack",
            SigilSkill::Affinity => "Affinity",
            SigilSkill::Defense => "Defense",
            SigilSkill::Stun => "Slugger",
            SigilSkill::FlyingWyvernSlayer => "Flying Wyvern",
            SigilSkill::BirdWyvernSlayer => "Bird Wyvern",
            SigilSkill::CarapaceonSlayer => "Carapaceon",
            SigilSkill::PiscineSlayer => "Piscine Wyvern",
            SigilSkill::FangedBeastSlayer => "Fanged Beast",
            SigilSkill::BruteWyvernSlayer => "Brute Wyvern",
            SigilSkill::LeviathanSlayer => "Leviathan",
            SigilSkill::ElderDragonSlayer => "Elder Dragon",
            SigilSkill::SmallMonsterSlayer => "Small Monster",
            SigilSkill::ElementalCoatingCrystal => "?",
            SigilSkill::HeatWave => "Heat",
            SigilSkill::IntenseCold => "Cold",
            SigilSkill::MorningSlayer => "Morning",
            SigilSkill::NightSlayer => "Evening",
            SigilSkill::BreedingSeasonSlayer => "Spring",
            SigilSkill::WarmSeasonSlayer => "Summer",
            SigilSkill::ColdSeasonSlayer => "Winter",
            SigilSkill::MondaySlayer => "Monday",
            SigilSkill::TuesdaySlayer => "Tuesday",
            SigilSkill::WednesdaySlayer => "Wednesday",
            SigilSkill::ThursdaySlayer => "Thursday",
            SigilSkill::FridaySlayer => "Friday",
            SigilSkill::SaturdaySlayer => "Saturday",
            SigilSkill::SundaySlayer => "Sunday",
            SigilSkill::ManagerFriend => "Manager",
            SigilSkill::BalloonFriend => "Balloon",
            SigilSkill::Dowsing => "Dowsing",
            SigilSkill::HunterSoul => "Hunter Soul",
            SigilSkill::SoulCollection => "Soul Harvest",
            SigilSkill::GoocooFriend => "Goocoo Pal",
            SigilSkill::HalkFriend => "Halk Friend",
            SigilSkill::MoneyExpert => "Negotiator",
            SigilSkill::ExperienceExpert => "Adept",
            SigilSkill::Omnivorous => "Omnivorous",
            SigilSkill::CarvingDivision => "Scrapper",
            SigilSkill::NoddingOff => "Dozer",
            SigilSkill::Decoy => "Decoy",
            SigilSkill::Tantrum => "Spoiled",
            SigilSkill::Unity => "Unity",
            SigilSkill::Support => "Support",
            SigilSkill::RapidFireAdd => "Rapid Fire+",
            SigilSkill::SuperRapidFireAdd => "S.Rapid Fire+",
            SigilSkill::RapidFireDisable => "Rapid Fire-",
            SigilSkill::HeatCannonAdd => "Heat Beam+",
            SigilSkill::BallistaSaver => "Ballista Saver",
            SigilSkill::RawBowBoost => "Bow Boost",
            SigilSkill::RawSnsBoost => "S&S Boost",
            SigilSkill::SonicBombRange => "Sonic Range",
            SigilSkill::TechBoostSnS => "Sword Arts",
            SigilSkill::TechBoostDb => "D.Blades Arts",
            SigilSkill::TechBoostGs => "G.Sword Arts",
            SigilSkill::TechBoostLs => "L.Sword Arts",
            SigilSkill::TechBoostLance => "Lance Arts",
            SigilSkill::TechBoostGunlance => "Gunlance Arts",
            SigilSkill::TechBoostHammer => "Hammer Arts",
            SigilSkill::TechBoostHorn => "Horn Arts",
            SigilSkill::TechChangeSnS => "Sword Tech",
            SigilSkill::TechChangeDb => "D.Blades Tech",
            SigilSkill::TechChangeGs => "G.Sword Tech",
            SigilSkill::TechChangeLs => "L.Sword Tech",
            SigilSkill::TechChangeLance => "Lance Tech",
            SigilSkill::TechChangeGunlance => "Gunlance Tech",
            SigilSkill::TechChangeHammer => "Hammer Tech",
            SigilSkill::TechChangeHorn => "Horn Tech",
            SigilSkill::TechChangeLbg => "L.Bowgun Tech",
            SigilSkill::TechChangeHbg => "H.Bowgun Tech",
            SigilSkill::TechChangeBow => "Bow Tech",
            SigilSkill::LengthUp => "Length Up",
            SigilSkill::ShellingChange => "Shelling",
            SigilSkill::NoteChange1 => "Note Change 1",
            SigilSkill::NoteChange2 => "Note Change 2",
            SigilSkill::NoteChange3 => "Note Change 3",
            SigilSkill::ArcShotChange => "Arc Shot",
            SigilSkill::RarityChange => "Rarity",
            SigilSkill::ShellingLvUp => "Shelling+",
            SigilSkill::TechBoostBow => "Bow Arts",
            SigilSkill::CatBreeder => "Cat Person",
            SigilSkill::ShotChange => "Shots",
            SigilSkill::DrawStrengthGs => "Draw (GS)",
            SigilSkill::DrawStrengthHammer => "Draw (Hammer)",
            SigilSkill::DrawStrengthLance => "Draw (Lance)",
            SigilSkill::DrawStrengthSns => "Draw (S&S)",
            SigilSkill::DrawStrengthDb => "Draw (DS)",
            SigilSkill::DrawStrengthLs => "Draw (LS)",
            SigilSkill::DrawStrengthHorn => "Draw (HH)",
            SigilSkill::DrawStrengthGunlance => "Draw (GL)",
            SigilSkill::DrawStrengthTonfa => "Draw (Tonfa)",
            SigilSkill::TechBoostTonfa => "Tonfa Arts",
            SigilSkill::FangedWyvernSlayer => "Fanged Wyvern",
            SigilSkill::UnsheathedMovementSpeed => "Unsheathe",
            SigilSkill::TechBoostSaf => "S.Axe Arts",
            SigilSkill::TechBoostHbg => "H.Bowgun Arts",
            SigilSkill::GsUp => "Great Sword+",
            SigilSkill::HbgUp => "Heavy Bowgun+",
            SigilSkill::HammerUp => "Hammer+",
            SigilSkill::LanceUp => "Lance+",
            SigilSkill::SnsUp => "Sword+",
            SigilSkill::LbgUp => "Light Bowgun+",
            SigilSkill::DbUp => "Dual Blades+",
            SigilSkill::LsUp => "Long Sword+",
            SigilSkill::HornUp => "Hunting Horn+",
            SigilSkill::GunlanceUp => "Gunlance+",
            SigilSkill::BowUp => "Bow+",
            SigilSkill::TonfaUp => "Tonfa+",
            SigilSkill::SafUp => "Switch Axe+",
            SigilSkill::MsUp => "Magnet Spike+",
            SigilSkill::ZCooldown => "Cooldown",
            SigilSkill::ZDuration => "Duration",
            SigilSkill::ZFireRes => "Fire Res",
            SigilSkill::ZWaterRes => "Water Res",
            SigilSkill::ZIceRes => "Ice Res",
            SigilSkill::ZThunderRes => "Thunder Res",
            SigilSkill::ZDragonRes => "Dragon Res",
            SigilSkill::ZHealing => "Healing",
            SigilSkill::ZHeroics => "Heroics",
            SigilSkill::ZAttack => "Attack",
            SigilSkill::ZElement => "Element",
            SigilSkill::ZMovementSpeed => "Move Speed",
            SigilSkill::RAttack => "Attack",
            SigilSkill::RElement => "Element",
            SigilSkill::RAffinity => "Affinity",
            SigilSkill::RStatus => "Status",
            SigilSkill::RStun => "Stun",
            SigilSkill::RHealing => "Healing",
            SigilSkill::RAllRes => "All Res",
        }
    }

    fn name_jp(&self) -> &'static str {
        match self {
            SigilSkill::Empty => "?",
            SigilSkill::Attack => "攻撃力補正",
            SigilSkill::Element => "属性値",
            SigilSkill::Status => "状態異常値",
            SigilSkill::Affinity => "会心率",
            SigilSkill::Defense => "防御力",
            SigilSkill::Stun => "スタン値",
            SigilSkill::FlyingWyvernSlayer => "飛竜種特効",
            SigilSkill::BirdWyvernSlayer => "鳥竜種特効",
            SigilSkill::CarapaceonSlayer => "甲殻種特効",
            SigilSkill::PiscineSlayer => "魚竜種特効",
            SigilSkill::FangedBeastSlayer => "牙獣種特効",
            SigilSkill::BruteWyvernSlayer => "獣竜種特効",
            SigilSkill::LeviathanSlayer => "海竜種特効",
            SigilSkill::ElderDragonSlayer => "古龍種特効",
            SigilSkill::SmallMonsterSlayer => "小型特効",
            SigilSkill::ElementalCoatingCrystal => "?",
            SigilSkill::HeatWave => "猛暑",
            SigilSkill::IntenseCold => "厳寒",
            SigilSkill::MorningSlayer => "朝方特効",
            SigilSkill::NightSlayer => "深夜特効",
            SigilSkill::BreedingSeasonSlayer => "繁殖期特効",
            SigilSkill::WarmSeasonSlayer => "温暖期特効",
            SigilSkill::ColdSeasonSlayer => "寒冷期特効",
            SigilSkill::MondaySlayer => "月曜日特効",
            SigilSkill::TuesdaySlayer => "火曜日特効",
            SigilSkill::WednesdaySlayer => "水曜日特効",
            SigilSkill::ThursdaySlayer => "木曜日特効",
            SigilSkill::FridaySlayer => "金曜日特効",
            SigilSkill::SaturdaySlayer => "土曜日特効",
            SigilSkill::SundaySlayer => "日曜日特効",
            SigilSkill::ManagerFriend => "管理人の友",
            SigilSkill::BalloonFriend => "気球の友",
            SigilSkill::Dowsing => "ダウジング",
            SigilSkill::HunterSoul => "狩人魂",
            SigilSkill::SoulCollection => "入魂のススメ",
            SigilSkill::GoocooFriend => "グークの友",
            SigilSkill::HalkFriend => "ホルクの友",
            SigilSkill::MoneyExpert => "報酬上手",
            SigilSkill::ExperienceExpert => "経験上手",
            SigilSkill::Omnivorous => "雑食",
            SigilSkill::CarvingDivision => "解体屋",
            SigilSkill::NoddingOff => "居眠り",
            SigilSkill::Decoy => "おとり",
            SigilSkill::Tantrum => "だだコネ",
            SigilSkill::Unity => "結束",
            SigilSkill::Support => "援護",
            SigilSkill::RapidFireAdd => "速射追加",
            SigilSkill::SuperRapidFireAdd => "超速射追加",
            SigilSkill::RapidFireDisable => "速射無効",
            SigilSkill::HeatCannonAdd => "排熱噴射追加",
            SigilSkill::BallistaSaver => "バリスタ節約",
            SigilSkill::RawBowBoost => "無属弓強化",
            SigilSkill::RawSnsBoost => "無属片手強化",
            SigilSkill::SonicBombRange => "音爆範囲拡大",
            SigilSkill::TechBoostSnS => "技強化[片手]",
            SigilSkill::TechBoostDb => "技強化[双剣]",
            SigilSkill::TechBoostGs => "技強化[大剣]",
            SigilSkill::TechBoostLs => "技強化[太刀]",
            SigilSkill::TechBoostLance => "技強化[槍]",
            SigilSkill::TechBoostGunlance => "技強化[銃槍]",
            SigilSkill::TechBoostHammer => "技強化[鎚]",
            SigilSkill::TechBoostHorn => "技強化[笛]",
            SigilSkill::TechChangeSnS => "技変化[片手]",
            SigilSkill::TechChangeDb => "技変化[双剣]",
            SigilSkill::TechChangeGs => "技変化[大剣]",
            SigilSkill::TechChangeLs => "技変化[太刀]",
            SigilSkill::TechChangeLance => "技変化[槍]",
            SigilSkill::TechChangeGunlance => "技変化[銃槍]",
            SigilSkill::TechChangeHammer => "技変化[鎚]",
            SigilSkill::TechChangeHorn => "技変化[笛]",
            SigilSkill::TechChangeLbg => "技変化[軽銃]",
            SigilSkill::TechChangeHbg => "技変化[重銃]",
            SigilSkill::TechChangeBow => "技変化[弓]",
            SigilSkill::LengthUp => "リーチUP",
            SigilSkill::ShellingChange => "砲撃変化",
            SigilSkill::NoteChange1 => "旋律変化１",
            SigilSkill::NoteChange2 => "旋律変化２",
            SigilSkill::NoteChange3 => "旋律変化３",
            SigilSkill::ArcShotChange => "曲射変化",
            SigilSkill::RarityChange => "レア度変化",
            SigilSkill::ShellingLvUp => "砲撃強化",
            SigilSkill::TechBoostBow => "技強化[弓]",
            SigilSkill::CatBreeder => "ネコブリーダー",
            SigilSkill::ShotChange => "弾変化",
            SigilSkill::DrawStrengthGs => "抜刀強化[大剣]",
            SigilSkill::DrawStrengthHammer => "抜刀強化[鎚]",
            SigilSkill::DrawStrengthLance => "抜刀強化[槍]",
            SigilSkill::DrawStrengthSns => "抜刀強化[片手]",
            SigilSkill::DrawStrengthDb => "抜刀強化[双剣]",
            SigilSkill::DrawStrengthLs => "抜刀強化[太刀]",
            SigilSkill::DrawStrengthHorn => "抜刀強化[笛]",
            SigilSkill::DrawStrengthGunlance => "抜刀強化[銃槍]",
            SigilSkill::DrawStrengthTonfa => "抜刀強化[棍]",
            SigilSkill::TechBoostTonfa => "技強化[棍]",
            SigilSkill::FangedWyvernSlayer => "牙竜種特効",
            SigilSkill::UnsheathedMovementSpeed => "抜刀移動速度",
            SigilSkill::TechBoostSaf => "技強化[剣斧]",
            SigilSkill::TechBoostHbg => "技強化[重銃]",
            SigilSkill::GsUp => "大剣強化",
            SigilSkill::HbgUp => "重銃強化",
            SigilSkill::HammerUp => "鎚強化",
            SigilSkill::LanceUp => "槍強化",
            SigilSkill::SnsUp => "片手剣強化",
            SigilSkill::LbgUp => "軽銃強化",
            SigilSkill::DbUp => "双剣強化",
            SigilSkill::LsUp => "太刀強化",
            SigilSkill::HornUp => "狩猟笛強化",
            SigilSkill::GunlanceUp => "銃槍強化",
            SigilSkill::BowUp => "弓強化",
            SigilSkill::TonfaUp => "穿龍棍強化",
            SigilSkill::SafUp => "剣斧強化",
            SigilSkill::MsUp => "磁斬鎚強化",
            SigilSkill::ZCooldown => "再使用待機時間",
            SigilSkill::ZDuration => "効果時間",
            SigilSkill::ZFireRes => "[辿]火耐性",
            SigilSkill::ZWaterRes => "[辿]水耐性",
            SigilSkill::ZIceRes => "[辿]氷耐性",
            SigilSkill::ZThunderRes => "[辿]雷耐性",
            SigilSkill::ZDragonRes => "[辿]龍耐性",
            SigilSkill::ZHealing => "[辿]持続回復",
            SigilSkill::ZHeroics => "[辿]豪傑",
            SigilSkill::ZAttack => "[辿]攻撃力",
            SigilSkill::ZElement => "[辿]属性値",
            SigilSkill::ZMovementSpeed => "[辿]移動速度",
            SigilSkill::RAttack => "[範]攻撃力",
            SigilSkill::RElement => "[範]属性値",
            SigilSkill::RAffinity => "[範]会心率",
            SigilSkill::RStatus => "[範]状態異常",
            SigilSkill::RStun => "[範]スタン値",
            SigilSkill::RHealing => "[範]持続回復",
            SigilSkill::RAllRes => "[範]全耐性",
        }
    }

    pub fn points_text(&self, language: Language, points: i16) -> Option<SpecificEffect> {
        match self {
            SigilSkill::Empty => Some(SpecificEffect::None),
            SigilSkill::Status => Some(SpecificEffect::None),
            SigilSkill::Stun => Some(SpecificEffect::None),
            SigilSkill::ElementalCoatingCrystal => Some(SpecificEffect::None),
            SigilSkill::BalloonFriend => Some(SpecificEffect::None),
            SigilSkill::HunterSoul => Some(SpecificEffect::None),
            SigilSkill::SoulCollection => Some(SpecificEffect::None),
            SigilSkill::HalkFriend => Some(SpecificEffect::None),
            SigilSkill::Omnivorous => Some(SpecificEffect::None),
            SigilSkill::CarvingDivision => Some(SpecificEffect::None),
            SigilSkill::NoddingOff => Some(SpecificEffect::None),
            SigilSkill::Decoy => Some(SpecificEffect::None),
            SigilSkill::Tantrum => Some(SpecificEffect::None),
            SigilSkill::Unity => Some(SpecificEffect::None),
            SigilSkill::Support => Some(SpecificEffect::None),
            SigilSkill::RapidFireAdd => {
                points_named_effect(language, *self, points).map(|v| v.into())
            }
            SigilSkill::SuperRapidFireAdd => {
                points_named_effect(language, *self, points).map(|v| v.into())
            }
            SigilSkill::RapidFireDisable => {
                points_named_effect(language, *self, points).map(|v| v.into())
            }
            SigilSkill::HeatCannonAdd => Some(SpecificEffect::None),
            SigilSkill::BallistaSaver => Some(SpecificEffect::None),
            SigilSkill::RawBowBoost => Some(SpecificEffect::None),
            SigilSkill::RawSnsBoost => Some(SpecificEffect::None),
            SigilSkill::SonicBombRange => Some(SpecificEffect::None),
            SigilSkill::TechBoostSnS => {
                points_named_effect(language, *self, points).map(|v| v.into())
            }
            SigilSkill::TechBoostDb => {
                points_named_effect(language, *self, points).map(|v| v.into())
            }
            SigilSkill::TechBoostGs => {
                points_named_effect(language, *self, points).map(|v| v.into())
            }
            SigilSkill::TechBoostLs => {
                points_named_effect(language, *self, points).map(|v| v.into())
            }
            SigilSkill::TechBoostLance => {
                points_named_effect(language, *self, points).map(|v| v.into())
            }
            SigilSkill::TechBoostGunlance => {
                points_named_effect(language, *self, points).map(|v| v.into())
            }
            SigilSkill::TechBoostHammer => {
                points_named_effect(language, *self, points).map(|v| v.into())
            }
            SigilSkill::TechBoostHorn => {
                points_named_effect(language, *self, points).map(|v| v.into())
            }
            SigilSkill::TechChangeSnS => {
                points_named_effect(language, *self, points).map(|v| v.into())
            }
            SigilSkill::TechChangeDb => None,
            SigilSkill::TechChangeGs => None,
            SigilSkill::TechChangeLs => None,
            SigilSkill::TechChangeLance => {
                points_named_effect(language, *self, points).map(|v| v.into())
            }
            SigilSkill::TechChangeGunlance => None,
            SigilSkill::TechChangeHammer => None,
            SigilSkill::TechChangeHorn => None,
            SigilSkill::TechChangeLbg => {
                points_named_effect(language, *self, points).map(|v| v.into())
            }
            SigilSkill::TechChangeHbg => None,
            SigilSkill::TechChangeBow => {
                points_named_effect(language, *self, points).map(|v| v.into())
            }
            SigilSkill::LengthUp => points_named_effect(language, *self, points).map(|v| v.into()),
            SigilSkill::ShellingChange => {
                points_named_effect(language, *self, points).map(|v| v.into())
            }
            SigilSkill::NoteChange1 => {
                points_named_effect(language, *self, points).map(|v| v.into())
            }
            SigilSkill::NoteChange2 => {
                points_named_effect(language, *self, points).map(|v| v.into())
            }
            SigilSkill::NoteChange3 => {
                points_named_effect(language, *self, points).map(|v| v.into())
            }
            SigilSkill::ArcShotChange => {
                points_named_effect(language, *self, points).map(|v| v.into())
            }
            SigilSkill::RarityChange => {
                points_named_effect(language, *self, points).map(|v| v.into())
            }
            SigilSkill::ShellingLvUp => Some(SpecificEffect::None),
            SigilSkill::TechBoostBow => {
                points_named_effect(language, *self, points).map(|v| v.into())
            }
            SigilSkill::CatBreeder => Some(SpecificEffect::None),
            SigilSkill::ShotChange => {
                points_named_effect(language, *self, points).map(|v| v.into())
            }
            SigilSkill::DrawStrengthGs => None,
            SigilSkill::DrawStrengthHammer => None,
            SigilSkill::DrawStrengthLance => Some(SpecificEffect::None),
            SigilSkill::DrawStrengthSns => None,
            SigilSkill::DrawStrengthDb => None,
            SigilSkill::DrawStrengthLs => Some(SpecificEffect::None),
            SigilSkill::DrawStrengthHorn => None,
            SigilSkill::DrawStrengthGunlance => Some(SpecificEffect::None),
            SigilSkill::DrawStrengthTonfa => None,
            SigilSkill::TechBoostTonfa => {
                points_named_effect(language, *self, points).map(|v| v.into())
            }
            SigilSkill::UnsheathedMovementSpeed => Some(SpecificEffect::None),
            SigilSkill::TechBoostSaf => {
                points_named_effect(language, *self, points).map(|v| v.into())
            }
            SigilSkill::TechBoostHbg => {
                points_named_effect(language, *self, points).map(|v| v.into())
            }
            SigilSkill::ZHeroics => Some(SpecificEffect::None),
            SigilSkill::RStatus => Some(SpecificEffect::None),
            SigilSkill::RStun => Some(SpecificEffect::None),
            SigilSkill::RHealing => Some(SpecificEffect::None),
            _ => Some(SpecificEffect::Points),
        }
    }

    pub fn points_options(
        &self,
        recipe: &SigilCraft,
        base_ranges: &[BaseSkillRanges],
    ) -> Vec<RangeInclusive<i16>> {
        let mut points_options: Vec<RangeInclusive<i16>> = recipe
            .normal_rolls
            .iter()
            .filter_map(|craft_roll| {
                if craft_roll.skill_roll.skill == *self {
                    Some(craft_roll.skill_roll.points.clone())
                } else {
                    None
                }
            })
            .collect();
        if points_options.is_empty() {
            let ranges = &base_ranges[*self as usize];
            points_options.extend(
                ranges
                    .points_ranges
                    .iter()
                    .map(|range_budget| range_budget.points_range.clone()),
            );
        }
        points_options
    }
}

pub fn points_named_effect(
    language: Language,
    skill: SigilSkill,
    points: i16,
) -> Option<&'static str> {
    match language {
        Language::English => points_named_effect_en(skill, points),
        Language::Japanese => points_named_effect_jp(skill, points),
    }
}

fn points_named_effect_en(skill: SigilSkill, points: i16) -> Option<&'static str> {
    match skill {
        SigilSkill::RapidFireAdd => match points {
            1 => Some("Normal 1 RF"),
            2 => Some("Normal 2 RF"),
            3 => Some("Pierce 1 RF"),
            4 => Some("Pellet 1 RF"),
            5 => Some("Crag 1 RF"),
            6 => Some("Flaming RF"),
            7 => Some("Water RF"),
            8 => Some("Thunder RF"),
            9 => Some("Freeze RF"),
            _ => None,
        },
        SigilSkill::SuperRapidFireAdd => match points {
            1 => Some("Normal 1 RF+"),
            2 => Some("Normal 2 RF+"),
            3 => Some("Pierce 1 RF+"),
            4 => Some("Pellet 1 RF+"),
            5 => Some("Crag 1 RF+"),
            6 => Some("Flaming RF+"),
            7 => Some("Water RF+"),
            8 => Some("Thunder RF+"),
            9 => Some("Freeze RF+"),
            _ => None,
        },
        SigilSkill::RapidFireDisable => match points {
            1 => Some("Normal 1 RF-"),
            2 => Some("Normal 2 RF-"),
            3 => Some("Pierce 1 RF-"),
            4 => Some("Pellet 1 RF-"),
            5 => Some("Crag 1 RF-"),
            6 => Some("Flaming RF-"),
            7 => Some("Water RF-"),
            8 => Some("Thunder RF-"),
            9 => Some("Freeze RF-"),
            _ => None,
        },
        SigilSkill::TechBoostSnS => match points {
            1 => Some("Sliding Slash"),
            2 => Some("Shield Bash"),
            3 => Some("Inf. Slash"),
            _ => None,
        },
        SigilSkill::TechBoostDb => match points {
            1 => Some("Front Flip"),
            2 => Some("Lunge Strike"),
            3 => Some("Clash Sharpen"),
            _ => None,
        },
        SigilSkill::TechBoostGs => match points {
            1 => Some("Upswing (GS)"),
            2 => Some("Rotation"),
            3 => Some("Guard Slash"),
            _ => None,
        },
        SigilSkill::TechBoostLs => match points {
            1 => Some("Piercing Stab"),
            2 => Some("Upswing (LS)"),
            3 => Some("Retreat"),
            4 => Some("Evade Slash"),
            _ => None,
        },
        SigilSkill::TechBoostLance => match points {
            1 => Some("Guard Charge"),
            2 => Some("Charge Finish"),
            _ => None,
        },
        SigilSkill::TechBoostGunlance => match points {
            1 => Some("Lunge Thrust"),
            2 => Some("Wide Sweep"),
            _ => None,
        },
        SigilSkill::TechBoostHammer => match points {
            1 => Some("Power Swing"),
            2 => Some("Upswing"),
            3 => Some("Hammer Charge"),
            _ => None,
        },
        SigilSkill::TechBoostHorn => match points {
            1 => Some("Performance"),
            2 => Some("Hilt Stab"),
            _ => None,
        },
        SigilSkill::TechChangeSnS => match points {
            1 => Some("Roundslash"),
            _ => None,
        },
        SigilSkill::TechChangeDb => match points {
            1 => Some("Clash Sharpen"),
            _ => None,
        },
        SigilSkill::TechChangeLance => match points {
            1 => Some("Drill Charge"),
            _ => None,
        },
        SigilSkill::TechChangeLbg => match points {
            1 => Some("Perfect Shot+"),
            _ => None,
        },
        SigilSkill::TechChangeBow => match points {
            1 => Some("Pierce"),
            _ => None,
        },
        SigilSkill::LengthUp => match points {
            1 => Some("Length (S&S)"),
            2 => Some("Length (L)"),
            3 => Some("Length (GS)"),
            4 => Some("Length (LS)"),
            5 => Some("Length (DB)"),
            6 => Some("Length (H)"),
            _ => None,
        },
        SigilSkill::ShellingChange => match points {
            1 => Some("Normal"),
            2 => Some("Wide"),
            3 => Some("Long"),
            _ => None,
        },
        SigilSkill::NoteChange1 => match points {
            1 => Some("White Note 1"),
            2 => Some("Purple Note 1"),
            3 => Some("Blue Note 1"),
            4 => Some("Yellow Note 1"),
            5 => Some("Red Note 1"),
            6 => Some("Green Note 1"),
            7 => Some("Sky Note 1"),
            _ => None,
        },
        SigilSkill::NoteChange2 => match points {
            1 => Some("White Note 2"),
            2 => Some("Purple Note 2"),
            3 => Some("Blue Note 2"),
            4 => Some("Yellow Note 2"),
            5 => Some("Red Note 2"),
            6 => Some("Green Note 2"),
            7 => Some("Sky Note 2"),
            _ => None,
        },
        SigilSkill::NoteChange3 => match points {
            1 => Some("White Note 3"),
            2 => Some("Purple Note 3"),
            3 => Some("Blue Note 3"),
            4 => Some("Yellow Note 3"),
            5 => Some("Red Note 3"),
            6 => Some("Green Note 3"),
            7 => Some("Sky Note 3"),
            _ => None,
        },
        SigilSkill::ArcShotChange => match points {
            1 => Some("Wide Arc"),
            2 => Some("Focus Arc"),
            3 => Some("Blast Arc"),
            4 => Some("Slice Arc"),
            _ => None,
        },
        SigilSkill::RarityChange => match points {
            1 => Some("Rarity 1"),
            2 => Some("Rarity 2"),
            3 => Some("Rarity 3"),
            4 => Some("Rarity 4"),
            5 => Some("Rarity 5"),
            6 => Some("Rarity 6"),
            7 => Some("Rarity 7"),
            8 => Some("Rarity 8"),
            9 => Some("Rarity 9"),
            10 => Some("Rarity 10"),
            11 => Some("Rarity 11"),
            12 => Some("Rarity 12"),
            _ => None,
        },
        SigilSkill::TechBoostBow => match points {
            1 => Some("Uppercut"),
            _ => None,
        },
        SigilSkill::ShotChange => match points {
            1 => Some("Cluster+"),
            2 => Some("Impact Shot"),
            _ => None,
        },
        SigilSkill::TechBoostTonfa => match points {
            1 => Some("EX Gauge"),
            2 => Some("Combo Timer"),
            _ => None,
        },
        SigilSkill::TechBoostSaf => match points {
            1 => Some("Stun (SA)"),
            2 => Some("S.Axe Guard"),
            3 => Some("S.Axe Charge"),
            _ => None,
        },
        SigilSkill::TechBoostHbg => match points {
            1 => Some("Heat Gauge+"),
            _ => None,
        },
        _ => None,
    }
}

fn points_named_effect_jp(skill: SigilSkill, points: i16) -> Option<&'static str> {
    match skill {
        SigilSkill::RapidFireAdd => match points {
            1 => Some("速射 通常弾LV1"),
            2 => Some("速射 通常弾LV2"),
            3 => Some("速射 貫通弾LV1"),
            4 => Some("速射 散弾LV1"),
            5 => Some("速射 徹甲LV1"),
            6 => Some("速射 火炎弾"),
            7 => Some("速射 水冷弾"),
            8 => Some("速射 電撃弾"),
            9 => Some("速射 氷結弾"),
            _ => None,
        },
        SigilSkill::SuperRapidFireAdd => match points {
            1 => Some("超速 通常弾LV1"),
            2 => Some("超速 通常弾LV2"),
            3 => Some("超速 貫通弾LV1"),
            4 => Some("超速 散弾LV1"),
            5 => Some("超速 徹甲LV1"),
            6 => Some("超速 火炎弾"),
            7 => Some("超速 水冷弾"),
            8 => Some("超速 電撃弾"),
            9 => Some("超速 氷結弾"),
            _ => None,
        },
        SigilSkill::RapidFireDisable => match points {
            1 => Some("非速 通常弾LV1"),
            2 => Some("非速 通常弾LV2"),
            3 => Some("非速 貫通弾LV1"),
            4 => Some("非速 散弾LV1"),
            5 => Some("非速 徹甲LV1"),
            6 => Some("非速 火炎弾"),
            7 => Some("非速 水冷弾"),
            8 => Some("非速 電撃弾"),
            9 => Some("非速 氷結弾"),
            _ => None,
        },
        SigilSkill::TechBoostSnS => match points {
            1 => Some("[片手]滑込斬"),
            2 => Some("[片手]盾攻撃"),
            3 => Some("[片手]無限連斬"),
            _ => None,
        },
        SigilSkill::TechBoostDb => match points {
            1 => Some("[双剣]縦回転斬"),
            2 => Some("[双剣]斬り払い"),
            3 => Some("[双剣]刃打術"),
            _ => None,
        },
        SigilSkill::TechBoostGs => match points {
            1 => Some("[大剣]斬り上げ"),
            2 => Some("[大剣]薙ぎ払い"),
            3 => Some("[大剣]ガード斬"),
            _ => None,
        },
        SigilSkill::TechBoostLs => match points {
            1 => Some("[太刀]貫刺し"),
            2 => Some("[太刀]斬り上げ"),
            3 => Some("[太刀]後退斬"),
            4 => Some("[太刀]瞬斬"),
            _ => None,
        },
        SigilSkill::TechBoostLance => match points {
            1 => Some("[槍]盾攻撃"),
            2 => Some("[槍]突進止撃"),
            _ => None,
        },
        SigilSkill::TechBoostGunlance => match points {
            1 => Some("[銃槍]踏込斬上"),
            2 => Some("[銃槍]踏込薙払"),
            _ => None,
        },
        SigilSkill::TechBoostHammer => match points {
            1 => Some("[鎚]振り抜き"),
            2 => Some("[鎚]振り上げ"),
            3 => Some("[鎚]溜移動術"),
            _ => None,
        },
        SigilSkill::TechBoostHorn => match points {
            1 => Some("[笛]演奏攻撃"),
            2 => Some("[笛]つか殴り"),
            _ => None,
        },
        SigilSkill::TechChangeSnS => match points {
            1 => Some("[片手]真空回転斬"),
            _ => None,
        },
        SigilSkill::TechChangeDb => match points {
            1 => Some("Clash Sharpen"),
            _ => None,
        },
        SigilSkill::TechChangeLance => match points {
            1 => Some("[槍]ドリル突進"),
            _ => None,
        },
        SigilSkill::TechChangeLbg => match points {
            1 => Some("[軽銃]Jゲージ"),
            _ => None,
        },
        SigilSkill::TechChangeBow => match points {
            1 => Some("[弓]貫薙弓"),
            _ => None,
        },
        SigilSkill::LengthUp => match points {
            1 => Some("[片手]蒼輝剣"),
            2 => Some("[槍]蒼輝槍"),
            3 => Some("[大剣]蒼輝大剣"),
            4 => Some("[太刀]蒼輝刀"),
            5 => Some("[双剣]蒼輝双剣"),
            6 => Some("[鎚]蒼輝鎚"),
            _ => None,
        },
        SigilSkill::ShellingChange => match points {
            1 => Some("砲撃[通常型]"),
            2 => Some("砲撃[拡散型]"),
            3 => Some("砲撃[放射型]"),
            _ => None,
        },
        SigilSkill::NoteChange1 => match points {
            1 => Some("旋律１[白]"),
            2 => Some("旋律１[紫]"),
            3 => Some("旋律１[青]"),
            4 => Some("旋律１[黄]"),
            5 => Some("旋律１[赤]"),
            6 => Some("旋律１[緑]"),
            7 => Some("旋律１[空]"),
            _ => None,
        },
        SigilSkill::NoteChange2 => match points {
            1 => Some("旋律２[白]"),
            2 => Some("旋律２[紫]"),
            3 => Some("旋律２[青]"),
            4 => Some("旋律２[黄]"),
            5 => Some("旋律２[赤]"),
            6 => Some("旋律２[緑]"),
            7 => Some("旋律２[空]"),
            _ => None,
        },
        SigilSkill::NoteChange3 => match points {
            1 => Some("旋律３[白]"),
            2 => Some("旋律３[紫]"),
            3 => Some("旋律３[青]"),
            4 => Some("旋律３[黄]"),
            5 => Some("旋律３[赤]"),
            6 => Some("旋律３[緑]"),
            7 => Some("旋律３[空]"),
            _ => None,
        },
        SigilSkill::ArcShotChange => match points {
            1 => Some("曲射[放散型]"),
            2 => Some("曲射[集中型]"),
            3 => Some("曲射[爆裂型]"),
            4 => Some("曲射[切断型]"),
            _ => None,
        },
        SigilSkill::RarityChange => match points {
            1 => Some("レア度１"),
            2 => Some("レア度２"),
            3 => Some("レア度３"),
            4 => Some("レア度４"),
            5 => Some("レア度５"),
            6 => Some("レア度６"),
            7 => Some("レア度７"),
            8 => Some("レア度８"),
            9 => Some("レア度９"),
            10 => Some("レア度１０"),
            11 => Some("レア度１１"),
            12 => Some("レア度１２"),
            _ => None,
        },
        SigilSkill::TechBoostBow => match points {
            1 => Some("[弓]昇龍弓"),
            _ => None,
        },
        SigilSkill::ShotChange => match points {
            1 => Some("[拡散弾]大爆発"),
            2 => Some("[散弾]打撃弾"),
            _ => None,
        },
        SigilSkill::TechBoostTonfa => match points {
            1 => Some("[棍]ＥＸゲージ"),
            2 => Some("[棍]Ｃタイマー"),
            _ => None,
        },
        SigilSkill::TechBoostSaf => match points {
            1 => Some("[剣斧]爆発気絶"),
            2 => Some("[剣斧]ガード"),
            3 => Some("[剣斧]溜移動術"),
            _ => None,
        },
        SigilSkill::TechBoostHbg => match points {
            1 => Some("[重銃]砲熱Ｇ"),
            _ => None,
        },
        _ => None,
    }
}
