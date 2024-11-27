use super::stars::{stars, OsuDifficultyAttributes, OsuPerformanceAttributes};
use crate::{Beatmap, GameMods};

/// Calculator for pp on osu!standard maps.
///
/// # Example
///
/// ```
/// # use akatsuki_pp::Beatmap;
/// # use akatsuki_pp::osu_2019::OsuPP;
/// # /*
/// let map: Beatmap = ...
/// # */
/// # let map = Beatmap::default();
/// let attrs = OsuPP::from_map(&map)
///     .mods(8 + 64) // HDDT
///     .combo(1234)
///     .misses(1)
///     .accuracy(98.5) // should be set last
///     .calculate();
///
/// println!("PP: {} | Stars: {}", attrs.pp, attrs.difficulty.stars);
///
/// let next_result = OsuPP::from_attributes(attrs.difficulty) // reusing previous results for performance
///     .mods(8 + 64)      // has to be the same to reuse attributes
///     .accuracy(99.5)
///     .calculate();
///
/// println!("PP: {} | Stars: {}", next_result.pp, next_result.difficulty.stars);
/// ```
#[derive(Clone, Debug)]
pub struct OsuPP<'m> {
    map: Option<&'m Beatmap>,
    attributes: Option<OsuDifficultyAttributes>,
    mods: GameMods,
    combo: Option<u32>,
    acc: Option<f32>,

    n300: Option<u32>,
    n100: Option<u32>,
    n50: Option<u32>,
    n_misses: u32,

    passed_objects: Option<u32>,
}

impl<'m> OsuPP<'m> {
    /// Creates a new calculator for the given map.
    #[inline]
    pub fn from_map(map: &'m Beatmap) -> Self {
        Self {
            map: Some(map),
            attributes: None,
            mods: GameMods::default(),
            combo: None,
            acc: None,
            n300: None,
            n100: None,
            n50: None,
            n_misses: 0,
            passed_objects: None,
        }
    }

    /// Creates a new calculator for the given attributes.
    #[inline]
    pub fn from_attributes(attributes: OsuDifficultyAttributes) -> Self {
        Self {
            map: None,
            attributes: Some(attributes),
            mods: GameMods::default(),
            combo: None,
            acc: None,
            n300: None,
            n100: None,
            n50: None,
            n_misses: 0,
            passed_objects: None,
        }
    }

    /// Specify mods through their bit values.
    ///
    /// See [https://github.com/ppy/osu-api/wiki#mods](https://github.com/ppy/osu-api/wiki#mods)
    #[inline]
    pub fn mods(mut self, mods: impl Into<GameMods>) -> Self {
        self.mods = mods.into();

        self
    }

    /// Specify the max combo of the play.
    #[inline]
    pub fn combo(mut self, combo: u32) -> Self {
        self.combo = Some(combo);

        self
    }

    /// Specify the amount of 300s of a play.
    #[inline]
    pub fn n300(mut self, n300: u32) -> Self {
        self.n300 = Some(n300);

        self
    }

    /// Specify the amount of 100s of a play.
    #[inline]
    pub fn n100(mut self, n100: u32) -> Self {
        self.n100 = Some(n100);

        self
    }

    /// Specify the amount of 50s of a play.
    #[inline]
    pub fn n50(mut self, n50: u32) -> Self {
        self.n50 = Some(n50);

        self
    }

    /// Specify the amount of misses of a play.
    #[inline]
    pub fn misses(mut self, n_misses: u32) -> Self {
        self.n_misses = n_misses;

        self
    }

    #[inline]
    pub fn passed_objects(mut self, passed_objects: u32) -> Self {
        self.passed_objects = self.passed_objects.replace(passed_objects);

        self
    }

    /// Generate the hit results with respect to the given accuracy between `0` and `100`.
    ///
    /// Be sure to set `misses` beforehand!
    pub fn accuracy(mut self, acc: f32) -> Self {
        let n_objects = self.n_objects();

        let acc = acc / 100.0;

        if self.n100.or(self.n50).is_some() {
            let mut n100 = self.n100.unwrap_or(0);
            let mut n50 = self.n50.unwrap_or(0);

            let placed_points = 2 * n100 + n50 + self.n_misses;
            let missing_objects = n_objects - n100 - n50 - self.n_misses;
            let missing_points =
                ((6.0 * acc * n_objects as f32).round() as u32).saturating_sub(placed_points);

            let mut n300 = missing_objects.min(missing_points / 6);
            n50 += missing_objects - n300;

            if let Some(orig_n50) = self.n50.filter(|_| self.n100.is_none()) {
                // Only n50s were changed, try to load some off again onto n100s
                let difference = n50 - orig_n50;
                let n = n300.min(difference / 4);

                n300 -= n;
                n100 += 5 * n;
                n50 -= 4 * n;
            }

            self.n300.replace(n300);
            self.n100.replace(n100);
            self.n50.replace(n50);
        } else {
            let misses = self.n_misses.min(n_objects);
            let target_total = (acc * n_objects as f32 * 6.0).round() as u32;
            let delta = target_total - (n_objects - misses);

            let mut n300 = delta / 5;
            let mut n100 = delta % 5;
            let mut n50 = n_objects - n300 - n100 - misses;

            // Sacrifice n300s to transform n50s into n100s
            let n = n300.min(n50 / 4);
            n300 -= n;
            n100 += 5 * n;
            n50 -= 4 * n;

            self.n300.replace(n300);
            self.n100.replace(n100);
            self.n50.replace(n50);
        }

        let acc = (6 * self.n300.unwrap() + 2 * self.n100.unwrap() + self.n50.unwrap()) as f32
            / (6 * n_objects) as f32;

        self.acc.replace(acc);

        self
    }

    fn assert_hitresults(&mut self) {
        if self.acc.is_none() {
            let n_objects = self.n_objects();

            let remaining = n_objects
                .saturating_sub(self.n300.unwrap_or(0))
                .saturating_sub(self.n100.unwrap_or(0))
                .saturating_sub(self.n50.unwrap_or(0))
                .saturating_sub(self.n_misses);

            if remaining > 0 {
                if self.n300.is_none() {
                    self.n300.replace(remaining);
                    self.n100.get_or_insert(0);
                    self.n50.get_or_insert(0);
                } else if self.n100.is_none() {
                    self.n100.replace(remaining);
                    self.n50.get_or_insert(0);
                } else if self.n50.is_none() {
                    self.n50.replace(remaining);
                } else {
                    *self.n300.as_mut().unwrap() += remaining;
                }
            } else {
                self.n300.get_or_insert(0);
                self.n100.get_or_insert(0);
                self.n50.get_or_insert(0);
            }

            let numerator = self.n50.unwrap() + self.n100.unwrap() * 2 + self.n300.unwrap() * 6;
            self.acc.replace(numerator as f32 / n_objects as f32 / 6.0);
        }
    }

    /// Returns an object which contains the pp and [`DifficultyAttributes`](crate::osu::DifficultyAttributes)
    /// containing stars and other attributes.
    pub fn calculate(mut self) -> OsuPerformanceAttributes {
        if self.attributes.is_none() {
            let attributes = stars(self.map.unwrap(), self.mods.clone(), self.passed_objects);
            self.attributes.replace(attributes);
        }

        // Make sure the hitresults and accuracy are set
        self.assert_hitresults();

        let total_hits = self.total_hits() as f32;
        let mut multiplier = 1.2;

        let effective_miss_count = self.calculate_effective_miss_count();

        // SO penalty
        if self.mods.so() {
            multiplier *=
                1.0 - (self.attributes.as_ref().unwrap().n_spinners as f32 / total_hits).powf(0.85);
        }

        let aim_value = self.compute_aim_value(total_hits, effective_miss_count);
        let mut speed_value = self.compute_speed_value(total_hits, effective_miss_count);

        let difficulty = self.attributes.as_ref().unwrap();
        let streams_nerf =
            ((difficulty.aim_strain / difficulty.speed_strain) * 100.0).round() / 100.0;

        if streams_nerf < 1.09 {
            let factor = (streams_nerf as f32 - 0.29).max(0.0).powf(2.0);
            speed_value *= factor;
        }

        let pp = (aim_value.powf(1.1) + speed_value.powf(1.1)).powf(1.0 / 1.1) * multiplier;

        OsuPerformanceAttributes {
            difficulty: self.attributes.unwrap(),
            pp_aim: aim_value as f64,
            pp_speed: speed_value as f64,
            pp: pp as f64,
            effective_miss_count: effective_miss_count as f64,
        }
    }

    fn compute_aim_value(&self, total_hits: f32, effective_miss_count: f32) -> f32 {
        let attributes = self.attributes.as_ref().unwrap();

        // TD penalty
        let raw_aim = if self.mods.td() {
            attributes.aim_strain.powf(0.8) as f32
        } else {
            attributes.aim_strain as f32
        };

        let mut aim_value = (5.0 * (raw_aim / 0.0675).max(1.0) - 4.0).powi(3) / 100_000.0;

        // Longer maps are worth more
        let len_bonus = 0.95
            + 0.4 * (total_hits / 2000.0).min(1.0)
            + (total_hits > 2000.0) as u8 as f32 * 0.5 * (total_hits / 2000.0).log10();
        aim_value *= len_bonus;

        // Penalize misses
        if effective_miss_count > 0.0 {
            let miss_penalty = self.calculate_miss_penalty(
                effective_miss_count,
                attributes.aim_difficult_strain_count,
            );
            aim_value *= miss_penalty;
        }

        // HD bonus
        if self.mods.hd() {
            aim_value *= 1.0 + 0.04 * (12.0 - attributes.ar) as f32;
        }

        // FL bonus
        if self.mods.fl() {
            aim_value *= 1.0
                + 0.3 * (total_hits / 200.0).min(1.0)
                + (total_hits > 200.0) as u8 as f32
                    * 0.25
                    * ((total_hits - 200.0) / 300.0).min(1.0)
                + (total_hits > 500.0) as u8 as f32 * (total_hits - 500.0) / 1600.0;
        }

        // Scale with accuracy
        aim_value *= self.acc.unwrap();
        aim_value *= 0.98 + attributes.od as f32 * attributes.od as f32 / 2500.0;

        aim_value
    }

    fn compute_speed_value(&self, total_hits: f32, effective_miss_count: f32) -> f32 {
        let attributes = self.attributes.as_ref().unwrap();

        let mut speed_value =
            (5.0 * (attributes.speed_strain as f32 / 0.0675).max(1.0) - 4.0).powi(3) / 100_000.0;

        // Longer maps are worth more
        let len_bonus = 0.95
            + 0.4 * (total_hits / 2000.0).min(1.0)
            + (total_hits > 2000.0) as u8 as f32 * 0.5 * (total_hits / 2000.0).log10();
        speed_value *= len_bonus;

        // Penalize misses
        if effective_miss_count > 0.0 {
            let miss_penalty = self.calculate_miss_penalty(
                effective_miss_count,
                attributes.speed_difficult_strain_count,
            );
            speed_value *= miss_penalty;
        }

        // HD bonus
        if self.mods.hd() {
            speed_value *= 1.0 + 0.04 * (12.0 - attributes.ar) as f32;
        }

        // Scaling the speed value with accuracy and OD
        let n300 = self.n300.unwrap();
        let n100 = self.n100.unwrap();
        let n50 = self.n50.unwrap();

        let relevant_total_diff = total_hits - attributes.speed_note_count;
        let relevant_n300 = (n300 as f32 - relevant_total_diff).max(0.0);
        let relevant_n100 = (n100 as f32 - (relevant_total_diff - n300 as f32).max(0.0)).max(0.0);
        let relevant_n50 =
            (n50 as f32 - (relevant_total_diff - (n300 + n100) as f32).max(0.0)).max(0.0);

        let relevant_acc = if attributes.speed_note_count == 0.0 {
            0.0
        } else {
            (relevant_n300 * 6.0 + relevant_n100 * 2.0 + relevant_n50)
                / (attributes.speed_note_count * 6.0)
        };

        speed_value *= (0.95 + attributes.od as f32 * attributes.od as f32 / 750.0)
            * ((self.acc.unwrap() + relevant_acc) / 2.0).powf((14.5 - attributes.od as f32) / 2.0);

        speed_value *= 0.99_f32.powf(match (n50 as f32) < total_hits / 500.0 {
            true => 0.0,
            false => self.n50.unwrap() as f32 - total_hits / 500.0,
        });

        speed_value
    }

    #[inline]
    fn total_hits(&self) -> u32 {
        let n_objects = self.n_objects();

        (self.n300.unwrap_or(0) + self.n100.unwrap_or(0) + self.n50.unwrap_or(0) + self.n_misses)
            .min(n_objects)
    }

    #[inline]
    fn calculate_miss_penalty(
        &self,
        effective_miss_count: f32,
        difficult_strain_count: f32,
    ) -> f32 {
        0.96 / ((effective_miss_count / (4.0 * difficult_strain_count.ln().powf(0.94))) + 1.0)
    }

    #[inline]
    fn calculate_effective_miss_count(&self) -> f32 {
        let mut combo_based_miss_count = 0.0;

        let attributes = self.attributes.as_ref().unwrap();
        let combo = self.combo.unwrap_or(attributes.max_combo as u32) as f32;
        let n100 = self.n100.unwrap_or(0) as f32;
        let n50 = self.n50.unwrap_or(0) as f32;

        if attributes.n_sliders > 0 {
            let fc_threshold = attributes.max_combo as f32 - (0.1 * attributes.n_sliders as f32);
            if combo < fc_threshold {
                combo_based_miss_count = fc_threshold / combo.max(1.0);
            }
        }

        combo_based_miss_count = combo_based_miss_count.min(n100 + n50 + self.n_misses as f32);
        combo_based_miss_count.max(self.n_misses as f32)
    }

    #[inline]
    fn n_objects(&self) -> u32 {
        if let Some(passed_objects) = self.passed_objects {
            return passed_objects;
        }

        match self.attributes.as_ref() {
            Some(attributes) => {
                (attributes.n_circles + attributes.n_sliders + attributes.n_spinners) as u32
            }
            None => self.map.unwrap().hit_objects.len() as u32,
        }
    }
}

/// Provides attributes for an osu! beatmap.
pub trait OsuAttributeProvider {
    /// Returns the attributes of the map.
    fn attributes(self) -> Option<OsuDifficultyAttributes>;
}

impl OsuAttributeProvider for OsuDifficultyAttributes {
    #[inline]
    fn attributes(self) -> Option<OsuDifficultyAttributes> {
        Some(self)
    }
}

impl OsuAttributeProvider for OsuPerformanceAttributes {
    #[inline]
    fn attributes(self) -> Option<OsuDifficultyAttributes> {
        Some(self.difficulty)
    }
}
