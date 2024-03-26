#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Biome {
    Badlands,
    BambooJungle,
    BasaltDeltas,
    Beach,
    BirchForest,
    CherryGrove,
    ColdOcean,
    CrimsonForest,
    DarkForest,
    DeepColdOcean,
    DeepDark,
    DeepFrozenOcean,
    DeepLukewarmOcean,
    DeepOcean,
    Desert,
    DripstoneCaves,
    EndBarrens,
    EndHighlands,
    EndMidlands,
    ErodedBadlands,
    FlowerForest,
    Forest,
    FrozenOcean,
    FrozenPeaks,
    FrozenRiver,
    Grove,
    IceSpikes,
    JaggedPeaks,
    Jungle,
    LukewarmOcean,
    LushCaves,
    MangroveSwamp,
    Meadow,
    MushroomFields,
    NetherWastes,
    Ocean,
    OldGrowthBirchForest,
    OldGrowthPineTaiga,
    OldGrowthSpruceTaiga,
    Plains,
    River,
    Savanna,
    SavannaPlateau,
    SmallEndIslands,
    SnowyBeach,
    SnowyPlains,
    SnowySlopes,
    SnowyTaiga,
    SoulSandValley,
    SparseJungle,
    StonyPeaks,
    StonyShore,
    SunflowerPlains,
    Swamp,
    Taiga,
    TheEnd,
    TheVoid,
    WarmOcean,
    WarpedForest,
    WindsweptForest,
    WindsweptGravellyHills,
    WindsweptHills,
    WindsweptSavanna,
    WoodedBadlands,
}

#[derive(Copy, Clone, Debug)]
pub struct InvalidBiomeErr;

impl TryFrom<u64> for Biome {
    type Error = InvalidBiomeErr;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Biome::Badlands),
            1 => Ok(Biome::BambooJungle),
            2 => Ok(Biome::BasaltDeltas),
            3 => Ok(Biome::Beach),
            4 => Ok(Biome::BirchForest),
            5 => Ok(Biome::CherryGrove),
            6 => Ok(Biome::ColdOcean),
            7 => Ok(Biome::CrimsonForest),
            8 => Ok(Biome::DarkForest),
            9 => Ok(Biome::DeepColdOcean),
            10 => Ok(Biome::DeepDark),
            11 => Ok(Biome::DeepFrozenOcean),
            12 => Ok(Biome::DeepLukewarmOcean),
            13 => Ok(Biome::DeepOcean),
            14 => Ok(Biome::Desert),
            15 => Ok(Biome::DripstoneCaves),
            16 => Ok(Biome::EndBarrens),
            17 => Ok(Biome::EndHighlands),
            18 => Ok(Biome::EndMidlands),
            19 => Ok(Biome::ErodedBadlands),
            20 => Ok(Biome::FlowerForest),
            21 => Ok(Biome::Forest),
            22 => Ok(Biome::FrozenOcean),
            23 => Ok(Biome::FrozenPeaks),
            24 => Ok(Biome::FrozenRiver),
            25 => Ok(Biome::Grove),
            26 => Ok(Biome::IceSpikes),
            27 => Ok(Biome::JaggedPeaks),
            28 => Ok(Biome::Jungle),
            29 => Ok(Biome::LukewarmOcean),
            30 => Ok(Biome::LushCaves),
            31 => Ok(Biome::MangroveSwamp),
            32 => Ok(Biome::Meadow),
            33 => Ok(Biome::MushroomFields),
            34 => Ok(Biome::NetherWastes),
            35 => Ok(Biome::Ocean),
            36 => Ok(Biome::OldGrowthBirchForest),
            37 => Ok(Biome::OldGrowthPineTaiga),
            38 => Ok(Biome::OldGrowthSpruceTaiga),
            39 => Ok(Biome::Plains),
            40 => Ok(Biome::River),
            41 => Ok(Biome::Savanna),
            42 => Ok(Biome::SavannaPlateau),
            43 => Ok(Biome::SmallEndIslands),
            44 => Ok(Biome::SnowyBeach),
            45 => Ok(Biome::SnowyPlains),
            46 => Ok(Biome::SnowySlopes),
            47 => Ok(Biome::SnowyTaiga),
            48 => Ok(Biome::SoulSandValley),
            49 => Ok(Biome::SparseJungle),
            50 => Ok(Biome::StonyPeaks),
            51 => Ok(Biome::StonyShore),
            52 => Ok(Biome::SunflowerPlains),
            53 => Ok(Biome::Swamp),
            54 => Ok(Biome::Taiga),
            55 => Ok(Biome::TheEnd),
            56 => Ok(Biome::TheVoid),
            57 => Ok(Biome::WarmOcean),
            58 => Ok(Biome::WarpedForest),
            59 => Ok(Biome::WindsweptForest),
            60 => Ok(Biome::WindsweptGravellyHills),
            61 => Ok(Biome::WindsweptHills),
            62 => Ok(Biome::WindsweptSavanna),
            63 => Ok(Biome::WoodedBadlands),
            _ => Err(InvalidBiomeErr)
        }
    }
}

impl Biome {
    pub const COUNT: usize = 64;
    pub const BITS: usize = (u64::BITS - 63u64.leading_zeros()) as usize;

    pub fn id(&self) -> u64 {
        match self {
            Biome::Badlands => 0,
            Biome::BambooJungle => 1,
            Biome::BasaltDeltas => 2,
            Biome::Beach => 3,
            Biome::BirchForest => 4,
            Biome::CherryGrove => 5,
            Biome::ColdOcean => 6,
            Biome::CrimsonForest => 7,
            Biome::DarkForest => 8,
            Biome::DeepColdOcean => 9,
            Biome::DeepDark => 10,
            Biome::DeepFrozenOcean => 11,
            Biome::DeepLukewarmOcean => 12,
            Biome::DeepOcean => 13,
            Biome::Desert => 14,
            Biome::DripstoneCaves => 15,
            Biome::EndBarrens => 16,
            Biome::EndHighlands => 17,
            Biome::EndMidlands => 18,
            Biome::ErodedBadlands => 19,
            Biome::FlowerForest => 20,
            Biome::Forest => 21,
            Biome::FrozenOcean => 22,
            Biome::FrozenPeaks => 23,
            Biome::FrozenRiver => 24,
            Biome::Grove => 25,
            Biome::IceSpikes => 26,
            Biome::JaggedPeaks => 27,
            Biome::Jungle => 28,
            Biome::LukewarmOcean => 29,
            Biome::LushCaves => 30,
            Biome::MangroveSwamp => 31,
            Biome::Meadow => 32,
            Biome::MushroomFields => 33,
            Biome::NetherWastes => 34,
            Biome::Ocean => 35,
            Biome::OldGrowthBirchForest => 36,
            Biome::OldGrowthPineTaiga => 37,
            Biome::OldGrowthSpruceTaiga => 38,
            Biome::Plains => 39,
            Biome::River => 40,
            Biome::Savanna => 41,
            Biome::SavannaPlateau => 42,
            Biome::SmallEndIslands => 43,
            Biome::SnowyBeach => 44,
            Biome::SnowyPlains => 45,
            Biome::SnowySlopes => 46,
            Biome::SnowyTaiga => 47,
            Biome::SoulSandValley => 48,
            Biome::SparseJungle => 49,
            Biome::StonyPeaks => 50,
            Biome::StonyShore => 51,
            Biome::SunflowerPlains => 52,
            Biome::Swamp => 53,
            Biome::Taiga => 54,
            Biome::TheEnd => 55,
            Biome::TheVoid => 56,
            Biome::WarmOcean => 57,
            Biome::WarpedForest => 58,
            Biome::WindsweptForest => 59,
            Biome::WindsweptGravellyHills => 60,
            Biome::WindsweptHills => 61,
            Biome::WindsweptSavanna => 62,
            Biome::WoodedBadlands => 63,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Biome::Badlands => "badlands",
            Biome::BambooJungle => "bamboo_jungle",
            Biome::BasaltDeltas => "basalt_deltas",
            Biome::Beach => "beach",
            Biome::BirchForest => "birch_forest",
            Biome::CherryGrove => "cherry_grove",
            Biome::ColdOcean => "cold_ocean",
            Biome::CrimsonForest => "crimson_forest",
            Biome::DarkForest => "dark_forest",
            Biome::DeepColdOcean => "deep_cold_ocean",
            Biome::DeepDark => "deep_dark",
            Biome::DeepFrozenOcean => "deep_frozen_ocean",
            Biome::DeepLukewarmOcean => "deep_lukewarm_ocean",
            Biome::DeepOcean => "deep_ocean",
            Biome::Desert => "desert",
            Biome::DripstoneCaves => "dripstone_caves",
            Biome::EndBarrens => "end_barrens",
            Biome::EndHighlands => "end_highlands",
            Biome::EndMidlands => "end_midlands",
            Biome::ErodedBadlands => "eroded_badlands",
            Biome::FlowerForest => "flower_forest",
            Biome::Forest => "forest",
            Biome::FrozenOcean => "frozen_ocean",
            Biome::FrozenPeaks => "frozen_peaks",
            Biome::FrozenRiver => "frozen_river",
            Biome::Grove => "grove",
            Biome::IceSpikes => "ice_spikes",
            Biome::JaggedPeaks => "jagged_peaks",
            Biome::Jungle => "jungle",
            Biome::LukewarmOcean => "lukewarm_ocean",
            Biome::LushCaves => "lush_caves",
            Biome::MangroveSwamp => "mangrove_swamp",
            Biome::Meadow => "meadow",
            Biome::MushroomFields => "mushroom_fields",
            Biome::NetherWastes => "nether_wastes",
            Biome::Ocean => "ocean",
            Biome::OldGrowthBirchForest => "old_growth_birch_forest",
            Biome::OldGrowthPineTaiga => "old_growth_pine_taiga",
            Biome::OldGrowthSpruceTaiga => "old_growth_spruce_taiga",
            Biome::Plains => "plains",
            Biome::River => "river",
            Biome::Savanna => "savanna",
            Biome::SavannaPlateau => "savanna_plateau",
            Biome::SmallEndIslands => "small_end_islands",
            Biome::SnowyBeach => "snowy_beach",
            Biome::SnowyPlains => "snowy_plains",
            Biome::SnowySlopes => "snowy_slopes",
            Biome::SnowyTaiga => "snowy_taiga",
            Biome::SoulSandValley => "soul_sand_valley",
            Biome::SparseJungle => "sparse_jungle",
            Biome::StonyPeaks => "stony_peaks",
            Biome::StonyShore => "stony_shore",
            Biome::SunflowerPlains => "sunflower_plains",
            Biome::Swamp => "swamp",
            Biome::Taiga => "taiga",
            Biome::TheEnd => "the_end",
            Biome::TheVoid => "the_void",
            Biome::WarmOcean => "warm_ocean",
            Biome::WarpedForest => "warped_forest",
            Biome::WindsweptForest => "windswept_forest",
            Biome::WindsweptGravellyHills => "windswept_gravelly_hills",
            Biome::WindsweptHills => "windswept_hills",
            Biome::WindsweptSavanna => "windswept_savanna",
            Biome::WoodedBadlands => "wooded_badlands",
        }
    }
    
    pub fn has_precipitation(&self) -> bool {
        match self {
            Biome::Badlands => false,
            Biome::BambooJungle => true,
            Biome::BasaltDeltas => false,
            Biome::Beach => true,
            Biome::BirchForest => true,
            Biome::CherryGrove => true,
            Biome::ColdOcean => true,
            Biome::CrimsonForest => false,
            Biome::DarkForest => true,
            Biome::DeepColdOcean => true,
            Biome::DeepDark => true,
            Biome::DeepFrozenOcean => true,
            Biome::DeepLukewarmOcean => true,
            Biome::DeepOcean => true,
            Biome::Desert => false,
            Biome::DripstoneCaves => true,
            Biome::EndBarrens => false,
            Biome::EndHighlands => false,
            Biome::EndMidlands => false,
            Biome::ErodedBadlands => false,
            Biome::FlowerForest => true,
            Biome::Forest => true,
            Biome::FrozenOcean => true,
            Biome::FrozenPeaks => true,
            Biome::FrozenRiver => true,
            Biome::Grove => true,
            Biome::IceSpikes => true,
            Biome::JaggedPeaks => true,
            Biome::Jungle => true,
            Biome::LukewarmOcean => true,
            Biome::LushCaves => true,
            Biome::MangroveSwamp => true,
            Biome::Meadow => true,
            Biome::MushroomFields => true,
            Biome::NetherWastes => false,
            Biome::Ocean => true,
            Biome::OldGrowthBirchForest => true,
            Biome::OldGrowthPineTaiga => true,
            Biome::OldGrowthSpruceTaiga => true,
            Biome::Plains => true,
            Biome::River => true,
            Biome::Savanna => false,
            Biome::SavannaPlateau => false,
            Biome::SmallEndIslands => false,
            Biome::SnowyBeach => true,
            Biome::SnowyPlains => true,
            Biome::SnowySlopes => true,
            Biome::SnowyTaiga => true,
            Biome::SoulSandValley => false,
            Biome::SparseJungle => true,
            Biome::StonyPeaks => true,
            Biome::StonyShore => true,
            Biome::SunflowerPlains => true,
            Biome::Swamp => true,
            Biome::Taiga => true,
            Biome::TheEnd => false,
            Biome::TheVoid => false,
            Biome::WarmOcean => true,
            Biome::WarpedForest => false,
            Biome::WindsweptForest => true,
            Biome::WindsweptGravellyHills => true,
            Biome::WindsweptHills => true,
            Biome::WindsweptSavanna => false,
            Biome::WoodedBadlands => false,
        }
    }
    
    pub fn temperature(&self) -> f64 {
        match self {
            Biome::Badlands => 2.0,
            Biome::BambooJungle => 0.95,
            Biome::BasaltDeltas => 2.0,
            Biome::Beach => 0.8,
            Biome::BirchForest => 0.6,
            Biome::CherryGrove => 0.5,
            Biome::ColdOcean => 0.5,
            Biome::CrimsonForest => 2.0,
            Biome::DarkForest => 0.7,
            Biome::DeepColdOcean => 0.5,
            Biome::DeepDark => 0.8,
            Biome::DeepFrozenOcean => 0.5,
            Biome::DeepLukewarmOcean => 0.5,
            Biome::DeepOcean => 0.5,
            Biome::Desert => 2.0,
            Biome::DripstoneCaves => 0.8,
            Biome::EndBarrens => 0.5,
            Biome::EndHighlands => 0.5,
            Biome::EndMidlands => 0.5,
            Biome::ErodedBadlands => 2.0,
            Biome::FlowerForest => 0.7,
            Biome::Forest => 0.7,
            Biome::FrozenOcean => 0.0,
            Biome::FrozenPeaks => -0.7,
            Biome::FrozenRiver => 0.0,
            Biome::Grove => -0.2,
            Biome::IceSpikes => 0.0,
            Biome::JaggedPeaks => -0.7,
            Biome::Jungle => 0.95,
            Biome::LukewarmOcean => 0.5,
            Biome::LushCaves => 0.5,
            Biome::MangroveSwamp => 0.8,
            Biome::Meadow => 0.5,
            Biome::MushroomFields => 0.9,
            Biome::NetherWastes => 2.0,
            Biome::Ocean => 0.5,
            Biome::OldGrowthBirchForest => 0.6,
            Biome::OldGrowthPineTaiga => 0.3,
            Biome::OldGrowthSpruceTaiga => 0.25,
            Biome::Plains => 0.8,
            Biome::River => 0.5,
            Biome::Savanna => 2.0,
            Biome::SavannaPlateau => 2.0,
            Biome::SmallEndIslands => 0.5,
            Biome::SnowyBeach => 0.05,
            Biome::SnowyPlains => 0.0,
            Biome::SnowySlopes => -0.3,
            Biome::SnowyTaiga => -0.5,
            Biome::SoulSandValley => 2.0,
            Biome::SparseJungle => 0.95,
            Biome::StonyPeaks => 1.0,
            Biome::StonyShore => 0.2,
            Biome::SunflowerPlains => 0.8,
            Biome::Swamp => 0.8,
            Biome::Taiga => 0.25,
            Biome::TheEnd => 0.5,
            Biome::TheVoid => 0.5,
            Biome::WarmOcean => 0.5,
            Biome::WarpedForest => 2.0,
            Biome::WindsweptForest => 0.2,
            Biome::WindsweptGravellyHills => 0.2,
            Biome::WindsweptHills => 0.2,
            Biome::WindsweptSavanna => 2.0,
            Biome::WoodedBadlands => 2.0,
        }
    }

    pub fn fog_color(&self) -> u32 {
        match self {
            Biome::Badlands => 12638463,
            Biome::BambooJungle => 12638463,
            Biome::BasaltDeltas => 6840176,
            Biome::Beach => 12638463,
            Biome::BirchForest => 12638463,
            Biome::CherryGrove => 12638463,
            Biome::ColdOcean => 12638463,
            Biome::CrimsonForest => 3343107,
            Biome::DarkForest => 12638463,
            Biome::DeepColdOcean => 12638463,
            Biome::DeepDark => 12638463,
            Biome::DeepFrozenOcean => 12638463,
            Biome::DeepLukewarmOcean => 12638463,
            Biome::DeepOcean => 12638463,
            Biome::Desert => 12638463,
            Biome::DripstoneCaves => 12638463,
            Biome::EndBarrens => 10518688,
            Biome::EndHighlands => 10518688,
            Biome::EndMidlands => 10518688,
            Biome::ErodedBadlands => 12638463,
            Biome::FlowerForest => 12638463,
            Biome::Forest => 12638463,
            Biome::FrozenOcean => 12638463,
            Biome::FrozenPeaks => 12638463,
            Biome::FrozenRiver => 12638463,
            Biome::Grove => 12638463,
            Biome::IceSpikes => 12638463,
            Biome::JaggedPeaks => 12638463,
            Biome::Jungle => 12638463,
            Biome::LukewarmOcean => 12638463,
            Biome::LushCaves => 12638463,
            Biome::MangroveSwamp => 12638463,
            Biome::Meadow => 12638463,
            Biome::MushroomFields => 12638463,
            Biome::NetherWastes => 3344392,
            Biome::Ocean => 12638463,
            Biome::OldGrowthBirchForest => 12638463,
            Biome::OldGrowthPineTaiga => 12638463,
            Biome::OldGrowthSpruceTaiga => 12638463,
            Biome::Plains => 12638463,
            Biome::River => 12638463,
            Biome::Savanna => 12638463,
            Biome::SavannaPlateau => 12638463,
            Biome::SmallEndIslands => 10518688,
            Biome::SnowyBeach => 12638463,
            Biome::SnowyPlains => 12638463,
            Biome::SnowySlopes => 12638463,
            Biome::SnowyTaiga => 12638463,
            Biome::SoulSandValley => 1787717,
            Biome::SparseJungle => 12638463,
            Biome::StonyPeaks => 12638463,
            Biome::StonyShore => 12638463,
            Biome::SunflowerPlains => 12638463,
            Biome::Swamp => 12638463,
            Biome::Taiga => 12638463,
            Biome::TheEnd => 10518688,
            Biome::TheVoid => 12638463,
            Biome::WarmOcean => 12638463,
            Biome::WarpedForest => 1705242,
            Biome::WindsweptForest => 12638463,
            Biome::WindsweptGravellyHills => 12638463,
            Biome::WindsweptHills => 12638463,
            Biome::WindsweptSavanna => 12638463,
            Biome::WoodedBadlands => 12638463,
        }
    }
    
    pub fn water_color(&self) -> u32 {
        match self {
            Biome::Badlands => 4159204,
            Biome::BambooJungle => 4159204,
            Biome::BasaltDeltas => 4159204,
            Biome::Beach => 4159204,
            Biome::BirchForest => 4159204,
            Biome::CherryGrove => 6141935,
            Biome::ColdOcean => 4020182,
            Biome::CrimsonForest => 4159204,
            Biome::DarkForest => 4159204,
            Biome::DeepColdOcean => 4020182,
            Biome::DeepDark => 4159204,
            Biome::DeepFrozenOcean => 3750089,
            Biome::DeepLukewarmOcean => 4566514,
            Biome::DeepOcean => 4159204,
            Biome::Desert => 4159204,
            Biome::DripstoneCaves => 4159204,
            Biome::EndBarrens => 4159204,
            Biome::EndHighlands => 4159204,
            Biome::EndMidlands => 4159204,
            Biome::ErodedBadlands => 4159204,
            Biome::FlowerForest => 4159204,
            Biome::Forest => 4159204,
            Biome::FrozenOcean => 3750089,
            Biome::FrozenPeaks => 4159204,
            Biome::FrozenRiver => 3750089,
            Biome::Grove => 4159204,
            Biome::IceSpikes => 4159204,
            Biome::JaggedPeaks => 4159204,
            Biome::Jungle => 4159204,
            Biome::LukewarmOcean => 4566514,
            Biome::LushCaves => 4159204,
            Biome::MangroveSwamp => 3832426,
            Biome::Meadow => 937679,
            Biome::MushroomFields => 4159204,
            Biome::NetherWastes => 4159204,
            Biome::Ocean => 4159204,
            Biome::OldGrowthBirchForest => 4159204,
            Biome::OldGrowthPineTaiga => 4159204,
            Biome::OldGrowthSpruceTaiga => 4159204,
            Biome::Plains => 4159204,
            Biome::River => 4159204,
            Biome::Savanna => 4159204,
            Biome::SavannaPlateau => 4159204,
            Biome::SmallEndIslands => 4159204,
            Biome::SnowyBeach => 4020182,
            Biome::SnowyPlains => 4159204,
            Biome::SnowySlopes => 4159204,
            Biome::SnowyTaiga => 4020182,
            Biome::SoulSandValley => 4159204,
            Biome::SparseJungle => 4159204,
            Biome::StonyPeaks => 4159204,
            Biome::StonyShore => 4159204,
            Biome::SunflowerPlains => 4159204,
            Biome::Swamp => 6388580,
            Biome::Taiga => 4159204,
            Biome::TheEnd => 4159204,
            Biome::TheVoid => 4159204,
            Biome::WarmOcean => 4445678,
            Biome::WarpedForest => 4159204,
            Biome::WindsweptForest => 4159204,
            Biome::WindsweptGravellyHills => 4159204,
            Biome::WindsweptHills => 4159204,
            Biome::WindsweptSavanna => 4159204,
            Biome::WoodedBadlands => 4159204,
        }
    }
    
    pub fn water_fog_color(&self) -> u32 {
        match self {
            Biome::Badlands => 329011,
            Biome::BambooJungle => 329011,
            Biome::BasaltDeltas => 329011,
            Biome::Beach => 329011,
            Biome::BirchForest => 329011,
            Biome::CherryGrove => 6141935,
            Biome::ColdOcean => 329011,
            Biome::CrimsonForest => 329011,
            Biome::DarkForest => 329011,
            Biome::DeepColdOcean => 329011,
            Biome::DeepDark => 329011,
            Biome::DeepFrozenOcean => 329011,
            Biome::DeepLukewarmOcean => 267827,
            Biome::DeepOcean => 329011,
            Biome::Desert => 329011,
            Biome::DripstoneCaves => 329011,
            Biome::EndBarrens => 329011,
            Biome::EndHighlands => 329011,
            Biome::EndMidlands => 329011,
            Biome::ErodedBadlands => 329011,
            Biome::FlowerForest => 329011,
            Biome::Forest => 329011,
            Biome::FrozenOcean => 329011,
            Biome::FrozenPeaks => 329011,
            Biome::FrozenRiver => 329011,
            Biome::Grove => 329011,
            Biome::IceSpikes => 329011,
            Biome::JaggedPeaks => 329011,
            Biome::Jungle => 329011,
            Biome::LukewarmOcean => 267827,
            Biome::LushCaves => 329011,
            Biome::MangroveSwamp => 5077600,
            Biome::Meadow => 329011,
            Biome::MushroomFields => 329011,
            Biome::NetherWastes => 329011,
            Biome::Ocean => 329011,
            Biome::OldGrowthBirchForest => 329011,
            Biome::OldGrowthPineTaiga => 329011,
            Biome::OldGrowthSpruceTaiga => 329011,
            Biome::Plains => 329011,
            Biome::River => 329011,
            Biome::Savanna => 329011,
            Biome::SavannaPlateau => 329011,
            Biome::SmallEndIslands => 329011,
            Biome::SnowyBeach => 329011,
            Biome::SnowyPlains => 329011,
            Biome::SnowySlopes => 329011,
            Biome::SnowyTaiga => 329011,
            Biome::SoulSandValley => 329011,
            Biome::SparseJungle => 329011,
            Biome::StonyPeaks => 329011,
            Biome::StonyShore => 329011,
            Biome::SunflowerPlains => 329011,
            Biome::Swamp => 2302743,
            Biome::Taiga => 329011,
            Biome::TheEnd => 329011,
            Biome::TheVoid => 329011,
            Biome::WarmOcean => 270131,
            Biome::WarpedForest => 329011,
            Biome::WindsweptForest => 329011,
            Biome::WindsweptGravellyHills => 329011,
            Biome::WindsweptHills => 329011,
            Biome::WindsweptSavanna => 329011,
            Biome::WoodedBadlands => 329011,
        }
    }
    
    pub fn sky_color(&self) -> u32 {
        match self {
            Biome::Badlands => 7254527,
            Biome::BambooJungle => 7842047,
            Biome::BasaltDeltas => 7254527,
            Biome::Beach => 7907327,
            Biome::BirchForest => 8037887,
            Biome::CherryGrove => 8103167,
            Biome::ColdOcean => 8103167,
            Biome::CrimsonForest => 7254527,
            Biome::DarkForest => 7972607,
            Biome::DeepColdOcean => 8103167,
            Biome::DeepDark => 7907327,
            Biome::DeepFrozenOcean => 8103167,
            Biome::DeepLukewarmOcean => 8103167,
            Biome::DeepOcean => 8103167,
            Biome::Desert => 7254527,
            Biome::DripstoneCaves => 7907327,
            Biome::EndBarrens => 0,
            Biome::EndHighlands => 0,
            Biome::EndMidlands => 0,
            Biome::ErodedBadlands => 7254527,
            Biome::FlowerForest => 7972607,
            Biome::Forest => 7972607,
            Biome::FrozenOcean => 8364543,
            Biome::FrozenPeaks => 8756735,
            Biome::FrozenRiver => 8364543,
            Biome::Grove => 8495359,
            Biome::IceSpikes => 8364543,
            Biome::JaggedPeaks => 8756735,
            Biome::Jungle => 7842047,
            Biome::LukewarmOcean => 8103167,
            Biome::LushCaves => 8103167,
            Biome::MangroveSwamp => 7907327,
            Biome::Meadow => 8103167,
            Biome::MushroomFields => 7842047,
            Biome::NetherWastes => 7254527,
            Biome::Ocean => 8103167,
            Biome::OldGrowthBirchForest => 8037887,
            Biome::OldGrowthPineTaiga => 8168447,
            Biome::OldGrowthSpruceTaiga => 8233983,
            Biome::Plains => 7907327,
            Biome::River => 8103167,
            Biome::Savanna => 7254527,
            Biome::SavannaPlateau => 7254527,
            Biome::SmallEndIslands => 0,
            Biome::SnowyBeach => 8364543,
            Biome::SnowyPlains => 8364543,
            Biome::SnowySlopes => 8560639,
            Biome::SnowyTaiga => 8625919,
            Biome::SoulSandValley => 7254527,
            Biome::SparseJungle => 7842047,
            Biome::StonyPeaks => 7776511,
            Biome::StonyShore => 8233727,
            Biome::SunflowerPlains => 7907327,
            Biome::Swamp => 7907327,
            Biome::Taiga => 8233983,
            Biome::TheEnd => 0,
            Biome::TheVoid => 8103167,
            Biome::WarmOcean => 8103167,
            Biome::WarpedForest => 7254527,
            Biome::WindsweptForest => 8233727,
            Biome::WindsweptGravellyHills => 8233727,
            Biome::WindsweptHills => 8233727,
            Biome::WindsweptSavanna => 7254527,
            Biome::WoodedBadlands => 7254527,
        }
    }
    
    pub fn foliage_color(&self) -> Option<u32> {
        match self {
            Biome::Badlands => Some(10387789),
            Biome::BambooJungle => None,
            Biome::BasaltDeltas => None,
            Biome::Beach => None,
            Biome::BirchForest => None,
            Biome::CherryGrove => Some(11983713),
            Biome::ColdOcean => None,
            Biome::CrimsonForest => None,
            Biome::DarkForest => None,
            Biome::DeepColdOcean => None,
            Biome::DeepDark => None,
            Biome::DeepFrozenOcean => None,
            Biome::DeepLukewarmOcean => None,
            Biome::DeepOcean => None,
            Biome::Desert => None,
            Biome::DripstoneCaves => None,
            Biome::EndBarrens => None,
            Biome::EndHighlands => None,
            Biome::EndMidlands => None,
            Biome::ErodedBadlands => Some(10387789),
            Biome::FlowerForest => None,
            Biome::Forest => None,
            Biome::FrozenOcean => None,
            Biome::FrozenPeaks => None,
            Biome::FrozenRiver => None,
            Biome::Grove => None,
            Biome::IceSpikes => None,
            Biome::JaggedPeaks => None,
            Biome::Jungle => None,
            Biome::LukewarmOcean => None,
            Biome::LushCaves => None,
            Biome::MangroveSwamp => Some(9285927),
            Biome::Meadow => None,
            Biome::MushroomFields => None,
            Biome::NetherWastes => None,
            Biome::Ocean => None,
            Biome::OldGrowthBirchForest => None,
            Biome::OldGrowthPineTaiga => None,
            Biome::OldGrowthSpruceTaiga => None,
            Biome::Plains => None,
            Biome::River => None,
            Biome::Savanna => None,
            Biome::SavannaPlateau => None,
            Biome::SmallEndIslands => None,
            Biome::SnowyBeach => None,
            Biome::SnowyPlains => None,
            Biome::SnowySlopes => None,
            Biome::SnowyTaiga => None,
            Biome::SoulSandValley => None,
            Biome::SparseJungle => None,
            Biome::StonyPeaks => None,
            Biome::StonyShore => None,
            Biome::SunflowerPlains => None,
            Biome::Swamp => Some(6975545),
            Biome::Taiga => None,
            Biome::TheEnd => None,
            Biome::TheVoid => None,
            Biome::WarmOcean => None,
            Biome::WarpedForest => None,
            Biome::WindsweptForest => None,
            Biome::WindsweptGravellyHills => None,
            Biome::WindsweptHills => None,
            Biome::WindsweptSavanna => None,
            Biome::WoodedBadlands => Some(10387789),
        }
    }
    
    pub fn grass_color(&self) -> Option<u32> {
        match self {
            Biome::Badlands => Some(9470285),
            Biome::BambooJungle => None,
            Biome::BasaltDeltas => None,
            Biome::Beach => None,
            Biome::BirchForest => None,
            Biome::CherryGrove => Some(11983713),
            Biome::ColdOcean => None,
            Biome::CrimsonForest => None,
            Biome::DarkForest => None,
            Biome::DeepColdOcean => None,
            Biome::DeepDark => None,
            Biome::DeepFrozenOcean => None,
            Biome::DeepLukewarmOcean => None,
            Biome::DeepOcean => None,
            Biome::Desert => None,
            Biome::DripstoneCaves => None,
            Biome::EndBarrens => None,
            Biome::EndHighlands => None,
            Biome::EndMidlands => None,
            Biome::ErodedBadlands => Some(9470285),
            Biome::FlowerForest => None,
            Biome::Forest => None,
            Biome::FrozenOcean => None,
            Biome::FrozenPeaks => None,
            Biome::FrozenRiver => None,
            Biome::Grove => None,
            Biome::IceSpikes => None,
            Biome::JaggedPeaks => None,
            Biome::Jungle => None,
            Biome::LukewarmOcean => None,
            Biome::LushCaves => None,
            Biome::MangroveSwamp => None,
            Biome::Meadow => None,
            Biome::MushroomFields => None,
            Biome::NetherWastes => None,
            Biome::Ocean => None,
            Biome::OldGrowthBirchForest => None,
            Biome::OldGrowthPineTaiga => None,
            Biome::OldGrowthSpruceTaiga => None,
            Biome::Plains => None,
            Biome::River => None,
            Biome::Savanna => None,
            Biome::SavannaPlateau => None,
            Biome::SmallEndIslands => None,
            Biome::SnowyBeach => None,
            Biome::SnowyPlains => None,
            Biome::SnowySlopes => None,
            Biome::SnowyTaiga => None,
            Biome::SoulSandValley => None,
            Biome::SparseJungle => None,
            Biome::StonyPeaks => None,
            Biome::StonyShore => None,
            Biome::SunflowerPlains => None,
            Biome::Swamp => None,
            Biome::Taiga => None,
            Biome::TheEnd => None,
            Biome::TheVoid => None,
            Biome::WarmOcean => None,
            Biome::WarpedForest => None,
            Biome::WindsweptForest => None,
            Biome::WindsweptGravellyHills => None,
            Biome::WindsweptHills => None,
            Biome::WindsweptSavanna => None,
            Biome::WoodedBadlands => Some(9470285),
        }
    }

    pub fn grass_color_modifier(&self) -> GrassColorModifier {
        match self {
            Biome::Badlands => GrassColorModifier::None,
            Biome::BambooJungle => GrassColorModifier::None,
            Biome::BasaltDeltas => GrassColorModifier::None,
            Biome::Beach => GrassColorModifier::None,
            Biome::BirchForest => GrassColorModifier::None,
            Biome::CherryGrove => GrassColorModifier::None,
            Biome::ColdOcean => GrassColorModifier::None,
            Biome::CrimsonForest => GrassColorModifier::None,
            Biome::DarkForest => GrassColorModifier::DarkForest,
            Biome::DeepColdOcean => GrassColorModifier::None,
            Biome::DeepDark => GrassColorModifier::None,
            Biome::DeepFrozenOcean => GrassColorModifier::None,
            Biome::DeepLukewarmOcean => GrassColorModifier::None,
            Biome::DeepOcean => GrassColorModifier::None,
            Biome::Desert => GrassColorModifier::None,
            Biome::DripstoneCaves => GrassColorModifier::None,
            Biome::EndBarrens => GrassColorModifier::None,
            Biome::EndHighlands => GrassColorModifier::None,
            Biome::EndMidlands => GrassColorModifier::None,
            Biome::ErodedBadlands => GrassColorModifier::None,
            Biome::FlowerForest => GrassColorModifier::None,
            Biome::Forest => GrassColorModifier::None,
            Biome::FrozenOcean => GrassColorModifier::None,
            Biome::FrozenPeaks => GrassColorModifier::None,
            Biome::FrozenRiver => GrassColorModifier::None,
            Biome::Grove => GrassColorModifier::None,
            Biome::IceSpikes => GrassColorModifier::None,
            Biome::JaggedPeaks => GrassColorModifier::None,
            Biome::Jungle => GrassColorModifier::None,
            Biome::LukewarmOcean => GrassColorModifier::None,
            Biome::LushCaves => GrassColorModifier::None,
            Biome::MangroveSwamp => GrassColorModifier::Swamp,
            Biome::Meadow => GrassColorModifier::None,
            Biome::MushroomFields => GrassColorModifier::None,
            Biome::NetherWastes => GrassColorModifier::None,
            Biome::Ocean => GrassColorModifier::None,
            Biome::OldGrowthBirchForest => GrassColorModifier::None,
            Biome::OldGrowthPineTaiga => GrassColorModifier::None,
            Biome::OldGrowthSpruceTaiga => GrassColorModifier::None,
            Biome::Plains => GrassColorModifier::None,
            Biome::River => GrassColorModifier::None,
            Biome::Savanna => GrassColorModifier::None,
            Biome::SavannaPlateau => GrassColorModifier::None,
            Biome::SmallEndIslands => GrassColorModifier::None,
            Biome::SnowyBeach => GrassColorModifier::None,
            Biome::SnowyPlains => GrassColorModifier::None,
            Biome::SnowySlopes => GrassColorModifier::None,
            Biome::SnowyTaiga => GrassColorModifier::None,
            Biome::SoulSandValley => GrassColorModifier::None,
            Biome::SparseJungle => GrassColorModifier::None,
            Biome::StonyPeaks => GrassColorModifier::None,
            Biome::StonyShore => GrassColorModifier::None,
            Biome::SunflowerPlains => GrassColorModifier::None,
            Biome::Swamp => GrassColorModifier::Swamp,
            Biome::Taiga => GrassColorModifier::None,
            Biome::TheEnd => GrassColorModifier::None,
            Biome::TheVoid => GrassColorModifier::None,
            Biome::WarmOcean => GrassColorModifier::None,
            Biome::WarpedForest => GrassColorModifier::None,
            Biome::WindsweptForest => GrassColorModifier::None,
            Biome::WindsweptGravellyHills => GrassColorModifier::None,
            Biome::WindsweptHills => GrassColorModifier::None,
            Biome::WindsweptSavanna => GrassColorModifier::None,
            Biome::WoodedBadlands => GrassColorModifier::None,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GrassColorModifier {
    None,
    DarkForest,
    Swamp,
}