use super::OsuPerformance;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct OsuDifficultyAttributes {
    pub aim_strain: f64,
    pub speed_strain: f64,
    pub ar: f64,
    pub od: f64,
    pub hp: f64,
    pub cs: f64,
    pub n_circles: u32,
    pub n_sliders: u32,
    pub n_spinners: u32,
    pub stars: f64,
    pub max_combo: u32,
    pub aim_difficult_strain_count: f64,
    pub speed_difficult_strain_count: f64,
    pub beatmap_id: i32,
    pub beatmap_creator: String,
    pub n_slider_ticks: u32,
}

impl OsuDifficultyAttributes {
    /// Return the maximum combo.
    pub const fn max_combo(&self) -> u32 {
        self.max_combo
    }

    /// Return the amount of hitobjects.
    pub const fn n_objects(&self) -> u32 {
        self.n_circles + self.n_sliders + self.n_spinners
    }

    /// Returns a builder for performance calculation.
    pub fn performance<'a>(self) -> OsuPerformance<'a> {
        self.into()
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct OsuPerformanceAttributes {
    pub difficulty: OsuDifficultyAttributes,
    pub pp: f64,
    pub pp_acc: f64,
    pub pp_aim: f64,
    pub pp_speed: f64,
    pub effective_miss_count: f64,
}

impl OsuPerformanceAttributes {
    /// Return the star value.
    pub const fn stars(&self) -> f64 {
        self.difficulty.stars
    }

    /// Return the performance point value.
    pub const fn pp(&self) -> f64 {
        self.pp
    }

    /// Return the maximum combo of the map.
    pub const fn max_combo(&self) -> u32 {
        self.difficulty.max_combo
    }
    /// Return the amount of hitobjects.
    pub const fn n_objects(&self) -> u32 {
        self.difficulty.n_objects()
    }

    /// Returns a builder for performance calculation.
    pub fn performance<'a>(self) -> OsuPerformance<'a> {
        self.difficulty.into()
    }
}

impl From<OsuPerformanceAttributes> for OsuDifficultyAttributes {
    fn from(attributes: OsuPerformanceAttributes) -> Self {
        attributes.difficulty
    }
}