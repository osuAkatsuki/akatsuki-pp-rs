use std::cmp;

use crate::{any::{HitResultPriority, IntoModePerformance, IntoPerformance}, osu::OsuScoreState, util::map_or_attrs::MapOrAttrs, Difficulty, GameMods, Performance};
use super::{OsuDifficultyAttributes, OsuPerformanceAttributes, OsuRelax};

pub mod gradual;

/// Performance calculator on osu!standard (relax) maps.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct OsuPerformance<'map> {
    pub(crate) map_or_attrs: MapOrAttrs<'map, OsuRelax>,
    pub(crate) difficulty: Difficulty,
    pub(crate) acc: Option<f64>,
    pub(crate) combo: Option<u32>,
    pub(crate) slider_tick_hits: Option<u32>,
    pub(crate) slider_end_hits: Option<u32>,
    pub(crate) n300: Option<u32>,
    pub(crate) n100: Option<u32>,
    pub(crate) n50: Option<u32>,
    pub(crate) misses: Option<u32>,
    pub(crate) hitresult_priority: HitResultPriority,
    pub(crate) lazer: Option<bool>,
}

impl<'map> OsuPerformance<'map> {
    /// Create a new performance calculator for osu! maps.
    ///
    /// The argument `map_or_attrs` must be either
    /// - previously calculated attributes ([`OsuDifficultyAttributes`]
    ///   or [`OsuPerformanceAttributes`])
    /// - a beatmap ([`OsuRelaxBeatmap<'map>`])
    ///
    /// If a map is given, difficulty attributes will need to be calculated
    /// internally which is a costly operation. Hence, passing attributes
    /// should be prefered.
    ///
    /// However, when passing previously calculated attributes, make sure they
    /// have been calculated for the same map and [`Difficulty`] settings.
    /// Otherwise, the final attributes will be incorrect.
    ///
    /// [`OsuBeatmap<'map>`]: crate::osu::OsuBeatmap
    pub fn new(map_or_attrs: impl IntoModePerformance<'map, OsuRelax>) -> Self {
        map_or_attrs.into_performance()
    }

    /// Try to create a new performance calculator for osu! maps.
    ///
    /// Returns `None` if `map_or_attrs` does not belong to osu! e.g.
    /// a [`Converted`], [`DifficultyAttributes`], or [`PerformanceAttributes`]
    /// of a different mode.
    ///
    /// See [`OsuPerformance::new`] for more information.
    ///
    /// [`Converted`]: crate::model::beatmap::Converted
    /// [`DifficultyAttributes`]: crate::any::DifficultyAttributes
    /// [`PerformanceAttributes`]: crate::any::PerformanceAttributes
    pub fn try_new(map_or_attrs: impl IntoPerformance<'map>) -> Option<Self> {
        if let Performance::OsuRelax(calc) = map_or_attrs.into_performance() {
            Some(calc)
        } else {
            None
        }
    }

    /// Specify mods.
    ///
    /// Accepted types are
    /// - `u32`
    /// - [`rosu_mods::GameModsLegacy`]
    /// - [`rosu_mods::GameMods`]
    /// - [`rosu_mods::GameModsIntermode`]
    /// - [`&rosu_mods::GameModsIntermode`](rosu_mods::GameModsIntermode)
    ///
    /// See <https://github.com/ppy/osu-api/wiki#mods>
    pub fn mods(mut self, mods: impl Into<GameMods>) -> Self {
        self.difficulty = self.difficulty.mods(mods);

        self
    }

    /// Specify the max combo of the play.
    pub const fn combo(mut self, combo: u32) -> Self {
        self.combo = Some(combo);

        self
    }

    /// Specify how hitresults should be generated.
    ///
    /// Defauls to [`HitResultPriority::BestCase`].
    pub const fn hitresult_priority(mut self, priority: HitResultPriority) -> Self {
        self.hitresult_priority = priority;

        self
    }

    /// Whether the calculated attributes belong to an osu!lazer or osu!stable
    /// score.
    ///
    /// Defaults to lazer.
    ///
    /// This affects internal accuracy calculation because lazer considers
    /// slider heads for accuracy whereas stable does not.
    pub const fn lazer(mut self, lazer: bool) -> Self {
        self.lazer = Some(lazer);

        self
    }

    /// Specify the amount of hit slider ticks.
    ///
    /// Only relevant for osu!lazer.
    pub const fn n_slider_ticks(mut self, n_slider_ticks: u32) -> Self {
        self.slider_tick_hits = Some(n_slider_ticks);

        self
    }

    /// Specify the amount of hit slider ends.
    ///
    /// Only relevant for osu!lazer.
    pub const fn n_slider_ends(mut self, n_slider_ends: u32) -> Self {
        self.slider_end_hits = Some(n_slider_ends);

        self
    }

    /// Specify the amount of 300s of a play.
    pub const fn n300(mut self, n300: u32) -> Self {
        self.n300 = Some(n300);

        self
    }

    /// Specify the amount of 100s of a play.
    pub const fn n100(mut self, n100: u32) -> Self {
        self.n100 = Some(n100);

        self
    }

    /// Specify the amount of 50s of a play.
    pub const fn n50(mut self, n50: u32) -> Self {
        self.n50 = Some(n50);

        self
    }

    /// Specify the amount of misses of a play.
    pub const fn misses(mut self, n_misses: u32) -> Self {
        self.misses = Some(n_misses);

        self
    }

    /// Use the specified settings of the given [`Difficulty`].
    pub fn difficulty(mut self, difficulty: Difficulty) -> Self {
        self.difficulty = difficulty;

        self
    }

    /// Amount of passed objects for partial plays, e.g. a fail.
    ///
    /// If you want to calculate the performance after every few objects,
    /// instead of using [`OsuPerformance`] multiple times with different
    /// `passed_objects`, you should use [`OsuGradualPerformance`].
    ///
    /// [`OsuGradualPerformance`]: crate::osu::OsuGradualPerformance
    pub fn passed_objects(mut self, passed_objects: u32) -> Self {
        self.difficulty = self.difficulty.passed_objects(passed_objects);

        self
    }

    /// Adjust the clock rate used in the calculation.
    ///
    /// If none is specified, it will take the clock rate based on the mods
    /// i.e. 1.5 for DT, 0.75 for HT and 1.0 otherwise.
    ///
    /// | Minimum | Maximum |
    /// | :-----: | :-----: |
    /// | 0.01    | 100     |
    pub fn clock_rate(mut self, clock_rate: f64) -> Self {
        self.difficulty = self.difficulty.clock_rate(clock_rate);

        self
    }

    /// Override a beatmap's set AR.
    ///
    /// `with_mods` determines if the given value should be used before
    /// or after accounting for mods, e.g. on `true` the value will be
    /// used as is and on `false` it will be modified based on the mods.
    ///
    /// | Minimum | Maximum |
    /// | :-----: | :-----: |
    /// | -20     | 20      |
    pub fn ar(mut self, ar: f32, with_mods: bool) -> Self {
        self.difficulty = self.difficulty.ar(ar, with_mods);

        self
    }

    /// Override a beatmap's set CS.
    ///
    /// `with_mods` determines if the given value should be used before
    /// or after accounting for mods, e.g. on `true` the value will be
    /// used as is and on `false` it will be modified based on the mods.
    ///
    /// | Minimum | Maximum |
    /// | :-----: | :-----: |
    /// | -20     | 20      |
    pub fn cs(mut self, cs: f32, with_mods: bool) -> Self {
        self.difficulty = self.difficulty.cs(cs, with_mods);

        self
    }

    /// Override a beatmap's set HP.
    ///
    /// `with_mods` determines if the given value should be used before
    /// or after accounting for mods, e.g. on `true` the value will be
    /// used as is and on `false` it will be modified based on the mods.
    ///
    /// | Minimum | Maximum |
    /// | :-----: | :-----: |
    /// | -20     | 20      |
    pub fn hp(mut self, hp: f32, with_mods: bool) -> Self {
        self.difficulty = self.difficulty.hp(hp, with_mods);

        self
    }

    /// Override a beatmap's set OD.
    ///
    /// `with_mods` determines if the given value should be used before
    /// or after accounting for mods, e.g. on `true` the value will be
    /// used as is and on `false` it will be modified based on the mods.
    ///
    /// | Minimum | Maximum |
    /// | :-----: | :-----: |
    /// | -20     | 20      |
    pub fn od(mut self, od: f32, with_mods: bool) -> Self {
        self.difficulty = self.difficulty.od(od, with_mods);

        self
    }

    /// Provide parameters through an [`OsuScoreState`].
    #[allow(clippy::needless_pass_by_value)]
    pub const fn state(mut self, state: OsuScoreState) -> Self {
        let OsuScoreState {
            max_combo,
            slider_tick_hits,
            slider_end_hits,
            n300,
            n100,
            n50,
            misses,
        } = state;

        self.combo = Some(max_combo);
        self.slider_tick_hits = Some(slider_tick_hits);
        self.slider_end_hits = Some(slider_end_hits);
        self.n300 = Some(n300);
        self.n100 = Some(n100);
        self.n50 = Some(n50);
        self.misses = Some(misses);

        self
    }

    /// Specify the accuracy of a play between `0.0` and `100.0`.
    /// This will be used to generate matching hitresults.
    pub fn accuracy(mut self, acc: f64) -> Self {
        self.acc = Some(acc.clamp(0.0, 100.0) / 100.0);

        self
    }

    /// Create the [`OsuScoreState`] that will be used for performance calculation.
    #[allow(clippy::too_many_lines)]
    pub fn generate_state(&mut self) -> OsuScoreState {
        let attrs = match self.map_or_attrs {
            MapOrAttrs::Map(ref map) => {
                let attrs = self.difficulty.with_mode().calculate(map);

                self.map_or_attrs.insert_attrs(attrs)
            }
            MapOrAttrs::Attrs(ref attrs) => attrs,
        };

        let max_combo = attrs.max_combo;
        let n_objects = cmp::min(
            self.difficulty.get_passed_objects() as u32,
            attrs.n_objects(),
        );
        let priority = self.hitresult_priority;

        let misses = self.misses.map_or(0, |n| cmp::min(n, n_objects));
        let n_remaining = n_objects - misses;

        let mut n300 = self.n300.map_or(0, |n| cmp::min(n, n_remaining));
        let mut n100 = self.n100.map_or(0, |n| cmp::min(n, n_remaining));
        let mut n50 = self.n50.map_or(0, |n| cmp::min(n, n_remaining));

        let lazer = self.lazer.unwrap_or(true);

        let (n_slider_ends, n_slider_ticks, max_slider_ends, max_slider_ticks) = if lazer {
            let n_slider_ends = self
                .slider_end_hits
                .map_or(attrs.n_sliders, |n| cmp::min(n, attrs.n_sliders));
            let n_slider_ticks = self
                .slider_tick_hits
                .map_or(attrs.n_slider_ticks, |n| cmp::min(n, attrs.n_slider_ticks));

            (
                n_slider_ends,
                n_slider_ticks,
                attrs.n_sliders,
                attrs.n_slider_ticks,
            )
        } else {
            (0, 0, 0, 0)
        };

        if let Some(acc) = self.acc {
            let target_total =
                acc * f64::from(30 * n_objects + 15 * max_slider_ends + 3 * max_slider_ticks);

            match (self.n300, self.n100, self.n50) {
                (Some(_), Some(_), Some(_)) => {
                    let remaining = n_objects.saturating_sub(n300 + n100 + n50 + misses);

                    match priority {
                        HitResultPriority::BestCase => n300 += remaining,
                        HitResultPriority::WorstCase => n50 += remaining,
                    }
                }
                (Some(_), Some(_), None) => n50 = n_objects.saturating_sub(n300 + n100 + misses),
                (Some(_), None, Some(_)) => n100 = n_objects.saturating_sub(n300 + n50 + misses),
                (None, Some(_), Some(_)) => n300 = n_objects.saturating_sub(n100 + n50 + misses),
                (Some(_), None, None) => {
                    let mut best_dist = f64::MAX;

                    n300 = cmp::min(n300, n_remaining);
                    let n_remaining = n_remaining - n300;

                    let raw_n100 = (target_total
                        - f64::from(
                            5 * n_remaining + 30 * n300 + 15 * n_slider_ends + 3 * n_slider_ticks,
                        ))
                        / 5.0;
                    let min_n100 = cmp::min(n_remaining, raw_n100.floor() as u32);
                    let max_n100 = cmp::min(n_remaining, raw_n100.ceil() as u32);

                    for new100 in min_n100..=max_n100 {
                        let new50 = n_remaining - new100;
                        let dist = (acc
                            - accuracy(
                                n_slider_ticks,
                                n_slider_ends,
                                n300,
                                new100,
                                new50,
                                misses,
                                max_slider_ticks,
                                max_slider_ends,
                            ))
                        .abs();

                        if dist < best_dist {
                            best_dist = dist;
                            n100 = new100;
                            n50 = new50;
                        }
                    }
                }
                (None, Some(_), None) => {
                    let mut best_dist = f64::MAX;

                    n100 = cmp::min(n100, n_remaining);
                    let n_remaining = n_remaining - n100;

                    let raw_n300 = (target_total
                        - f64::from(
                            5 * n_remaining + 10 * n100 + 15 * n_slider_ends + 3 * n_slider_ticks,
                        ))
                        / 25.0;
                    let min_n300 = cmp::min(n_remaining, raw_n300.floor() as u32);
                    let max_n300 = cmp::min(n_remaining, raw_n300.ceil() as u32);

                    for new300 in min_n300..=max_n300 {
                        let new50 = n_remaining - new300;
                        let curr_dist = (acc
                            - accuracy(
                                n_slider_ticks,
                                n_slider_ends,
                                new300,
                                n100,
                                new50,
                                misses,
                                max_slider_ticks,
                                max_slider_ends,
                            ))
                        .abs();

                        if curr_dist < best_dist {
                            best_dist = curr_dist;
                            n300 = new300;
                            n50 = new50;
                        }
                    }
                }
                (None, None, Some(_)) => {
                    let mut best_dist = f64::MAX;

                    n50 = cmp::min(n50, n_remaining);
                    let n_remaining = n_remaining - n50;

                    let raw_n300 = (target_total + f64::from(10 * misses + 5 * n50)
                        - f64::from(10 * n_objects + 15 * n_slider_ends + 3 * n_slider_ticks))
                        / 20.0;

                    let min_n300 = cmp::min(n_remaining, raw_n300.floor() as u32);
                    let max_n300 = cmp::min(n_remaining, raw_n300.ceil() as u32);

                    for new300 in min_n300..=max_n300 {
                        let new100 = n_remaining - new300;
                        let curr_dist = (acc
                            - accuracy(
                                n_slider_ticks,
                                n_slider_ends,
                                new300,
                                new100,
                                n50,
                                misses,
                                max_slider_ticks,
                                max_slider_ends,
                            ))
                        .abs();

                        if curr_dist < best_dist {
                            best_dist = curr_dist;
                            n300 = new300;
                            n100 = new100;
                        }
                    }
                }
                (None, None, None) => {
                    let mut best_dist = f64::MAX;

                    let raw_n300 = (target_total
                        - f64::from(5 * n_remaining + 15 * n_slider_ends + 3 * n_slider_ticks))
                        / 25.0;
                    let min_n300 = cmp::min(n_remaining, raw_n300.floor() as u32);
                    let max_n300 = cmp::min(n_remaining, raw_n300.ceil() as u32);

                    for new300 in min_n300..=max_n300 {
                        let raw_n100 = (target_total
                            - f64::from(
                                5 * n_remaining
                                    + 25 * new300
                                    + 15 * n_slider_ends
                                    + 3 * n_slider_ticks,
                            ))
                            / 5.0;
                        let min_n100 = cmp::min(raw_n100.floor() as u32, n_remaining - new300);
                        let max_n100 = cmp::min(raw_n100.ceil() as u32, n_remaining - new300);

                        for new100 in min_n100..=max_n100 {
                            let new50 = n_remaining - new300 - new100;
                            let curr_dist = (acc
                                - accuracy(
                                    n_slider_ticks,
                                    n_slider_ends,
                                    new300,
                                    new100,
                                    new50,
                                    misses,
                                    max_slider_ticks,
                                    max_slider_ends,
                                ))
                            .abs();

                            if curr_dist < best_dist {
                                best_dist = curr_dist;
                                n300 = new300;
                                n100 = new100;
                                n50 = new50;
                            }
                        }
                    }

                    match priority {
                        HitResultPriority::BestCase => {
                            // Shift n50 to n100 by sacrificing n300
                            let n = cmp::min(n300, n50 / 4);
                            n300 -= n;
                            n100 += 5 * n;
                            n50 -= 4 * n;
                        }
                        HitResultPriority::WorstCase => {
                            // Shift n100 to n50 by gaining n300
                            let n = n100 / 5;
                            n300 += n;
                            n100 -= 5 * n;
                            n50 += 4 * n;
                        }
                    }
                }
            }
        } else {
            let remaining = n_objects.saturating_sub(n300 + n100 + n50 + misses);

            match priority {
                HitResultPriority::BestCase => match (self.n300, self.n100, self.n50) {
                    (None, ..) => n300 = remaining,
                    (_, None, _) => n100 = remaining,
                    (.., None) => n50 = remaining,
                    _ => n300 += remaining,
                },
                HitResultPriority::WorstCase => match (self.n50, self.n100, self.n300) {
                    (None, ..) => n50 = remaining,
                    (_, None, _) => n100 = remaining,
                    (.., None) => n300 = remaining,
                    _ => n50 += remaining,
                },
            }
        }

        let max_possible_combo = max_combo.saturating_sub(misses);

        let max_combo = self.combo.map_or(max_possible_combo, |combo| {
            cmp::min(combo, max_possible_combo)
        });

        self.combo = Some(max_combo);
        self.slider_end_hits = Some(n_slider_ends);
        self.slider_tick_hits = Some(n_slider_ticks);
        self.n300 = Some(n300);
        self.n100 = Some(n100);
        self.n50 = Some(n50);
        self.misses = Some(misses);

        OsuScoreState {
            max_combo,
            slider_tick_hits: n_slider_ticks,
            slider_end_hits: n_slider_ends,
            n300,
            n100,
            n50,
            misses,
        }
    }

    /// Calculate all performance related values, including pp and stars.
    pub fn calculate(mut self) -> OsuPerformanceAttributes {
        let state = self.generate_state();

        let attrs = match self.map_or_attrs {
            MapOrAttrs::Map(ref map) => self.difficulty.with_mode().calculate(map),
            MapOrAttrs::Attrs(attrs) => attrs,
        };

        let effective_miss_count = calculate_effective_misses(&attrs, &state);

        let lazer = self.lazer.unwrap_or(true);

        let (n_slider_ends, n_slider_ticks) = if lazer {
            (attrs.n_sliders, attrs.n_slider_ticks)
        } else {
            (0, 0)
        };

        let inner = OsuPerformanceInner {
            attrs,
            mods: self.difficulty.get_mods(),
            acc: state.accuracy(n_slider_ticks, n_slider_ends),
            state,
            effective_miss_count,
            lazer,
        };

        inner.calculate()
    }

    pub(crate) const fn from_map_or_attrs(map_or_attrs: MapOrAttrs<'map, OsuRelax>) -> Self {
        Self {
            map_or_attrs,
            difficulty: Difficulty::new(),
            acc: None,
            combo: None,
            slider_tick_hits: None,
            slider_end_hits: None,
            n300: None,
            n100: None,
            n50: None,
            misses: None,
            hitresult_priority: HitResultPriority::DEFAULT,
            lazer: None,
        }
    }
}

impl<'map, T: IntoModePerformance<'map, OsuRelax>> From<T> for OsuPerformance<'map> {
    fn from(into: T) -> Self {
        into.into_performance()
    }
}

// * This is being adjusted to keep the final pp value scaled around what it used to be when changing things.
pub const PERFORMANCE_BASE_MULTIPLIER: f64 = 1.09;

struct OsuPerformanceInner<'mods> {
    attrs: OsuDifficultyAttributes,
    mods: &'mods GameMods,
    acc: f64,
    state: OsuScoreState,
    effective_miss_count: f64,
    lazer: bool,
}

impl OsuPerformanceInner<'_> {
    fn calculate(self) -> OsuPerformanceAttributes {
        let total_hits = self.state.total_hits();

        if total_hits == 0 {
            return OsuPerformanceAttributes {
                difficulty: self.attrs,
                ..Default::default()
            };
        }

        let total_hits = f64::from(total_hits);

        let mut multiplier = PERFORMANCE_BASE_MULTIPLIER;

        // SO penalty
        if self.mods.so() {
            multiplier *= 1.0 - (self.attrs.n_spinners as f64 / total_hits).powf(0.85);
        }

        let mut aim_value = self.compute_aim_value(total_hits);
        let speed_value = self.compute_speed_value(total_hits);
        let acc_value = self.compute_accuracy_value(total_hits);

        let mut acc_depression = 1.0;

        let streams_nerf =
            ((self.attrs.aim_strain / self.attrs.speed_strain) * 100.0).round() / 100.0;

        if streams_nerf < 1.09 {
            let acc_factor = (1.0 - self.acc).abs();
            acc_depression = (0.86 - acc_factor).max(0.5);

            if acc_depression > 0.0 {
                aim_value *= acc_depression;
            }
        }

        let nodt_bonus = match !(self.mods.dt() || self.mods.nc() || self.mods.ht()) {
            true => 1.02,
            false => 1.0,
        };

        let mut pp = (aim_value.powf(1.185 * nodt_bonus)
            + speed_value.powf(0.83 * acc_depression)
            + acc_value.powf(1.14 * nodt_bonus))
        .powf(1.0 / 1.1)
            * multiplier;

        if self.mods.dt() && self.mods.hr() {
            pp *= 1.025;
        }

        if self.attrs.beatmap_creator == "gwb" || self.attrs.beatmap_creator == "Plasma" {
            pp *= 0.9;
        }

        pp *= match self.attrs.beatmap_id {
            // Louder than steel [ok this is epic]
            1808605 => 0.85,

            // over the top [Above the stars]
            1821147 => 0.70,

            // Just press F [Parkour's ok this is epic]
            1844776 => 0.64,

            // Hardware Store [skyapple mode]
            1777768 => 0.90,

            // Akatsuki compilation [ok this is akatsuki]
            1962833 => {
                pp *= 0.885;

                if self.mods.dt() {
                    0.83
                } else {
                    1.0
                }
            }

            // Songs Compilation [Marathon]
            2403677 => 0.85,

            // Songs Compilation [Remembrance]
            2174272 => 0.85,

            // Apocalypse 1992 [Universal Annihilation]
            2382377 => 0.85,

            _ => 1.0,
        };

        OsuPerformanceAttributes {
            difficulty: self.attrs,
            pp_aim: aim_value,
            pp_speed: speed_value,
            pp_acc: acc_value,
            pp: pp,
            effective_miss_count: self.effective_miss_count,
        }
    }

    fn compute_aim_value(&self, total_hits: f64) -> f64 {
        // TD penalty
        let raw_aim = if self.mods.td() {
            self.attrs.aim_strain.powf(0.8)
        } else {
            self.attrs.aim_strain
        };

        let mut aim_value = (5.0 * (raw_aim / 0.0675).max(1.0) - 4.0).powi(3) / 100_000.0;

        // Longer maps are worth more
        let len_bonus = 0.88
            + 0.4 * (total_hits / 2000.0).min(1.0)
            + (total_hits > 2000.0) as u8 as f64 * 0.5 * (total_hits / 2000.0).log10();
        aim_value *= len_bonus;

        // Penalize misses
        if self.effective_miss_count > 0.0 {
            let miss_penalty = self.calculate_miss_penalty(self.effective_miss_count, total_hits);
            aim_value *= miss_penalty;
        }

        // AR bonus
        let mut ar_factor = if self.attrs.ar > 10.33 {
            0.3 * (self.attrs.ar - 10.33)
        } else {
            0.0
        };

        if self.attrs.ar < 8.0 {
            ar_factor = 0.025 * (8.0 - self.attrs.ar);
        }

        aim_value *= 1.0 + ar_factor * len_bonus;

        // HD bonus
        if self.mods.hd() {
            aim_value *= 1.0 + 0.05 * (11.0 - self.attrs.ar) as f64;
        }

        // FL bonus
        if self.mods.fl() {
            aim_value *= 1.0
                + 0.3 * (total_hits / 200.0).min(1.0)
                + (total_hits > 200.0) as u8 as f64
                    * 0.25
                    * ((total_hits - 200.0) / 300.0).min(1.0)
                + (total_hits > 500.0) as u8 as f64 * (total_hits - 500.0) / 1600.0;
        }

        // EZ bonus
        if self.mods.ez() {
            let mut base_buff = 1.08_f64;

            if self.attrs.ar <= 8.0 {
                base_buff += (7.0 - self.attrs.ar as f64) / 100.0;
            }

            aim_value *= base_buff;
        }

        // Scale with accuracy
        aim_value *= 0.3 + self.acc / 2.0;
        aim_value *= 0.98 + self.attrs.od * self.attrs.od / 2500.0;

        aim_value
    }

    fn compute_speed_value(&self, total_hits: f64) -> f64 {
        let mut speed_value =
            (5.0 * (self.attrs.speed_strain / 0.0675).max(1.0) - 4.0).powi(3) / 100_000.0;

        // Longer maps are worth more
        let len_bonus = 0.88
            + 0.4 * (total_hits / 2000.0).min(1.0)
            + (total_hits > 2000.0) as u8 as f64 * 0.5 * (total_hits / 2000.0).log10();
        speed_value *= len_bonus;

        // Penalize misses
        if self.effective_miss_count > 0.0 {
            let miss_penalty = self.calculate_miss_penalty(self.effective_miss_count, total_hits);
            speed_value *= miss_penalty;
        }

        // AR bonus
        let mut ar_factor = if self.attrs.ar > 10.33 {
            0.3 * (self.attrs.ar - 10.33)
        } else {
            0.0
        };

        if self.attrs.ar < 8.0 {
            ar_factor = 0.025 * (8.0 - self.attrs.ar);
        }

        speed_value *= 1.0 + ar_factor * len_bonus;

        // HD bonus
        if self.mods.hd() {
            speed_value *= 1.0 + 0.05 * (11.0 - self.attrs.ar) as f64;
        }

        // Scaling the speed value with accuracy and OD
        speed_value *= (0.93 + self.attrs.od * self.attrs.od / 750.0)
            * self
                .acc
                .powf((14.5 - self.attrs.od.max(8.0)) / 2.0);

        speed_value *= 0.98_f64.powf(match (self.state.n50 as f64) < total_hits / 500.0 {
            true => 0.0,
            false => self.state.n50 as f64 - total_hits / 500.0,
        });

        speed_value
    }

    fn compute_accuracy_value(&self, total_hits: f64) -> f64 {
        let n_circles = self.attrs.n_circles as f64;
        let n300 = self.state.n300 as f64;
        let n100 = self.state.n100 as f64;
        let n50 = self.state.n50 as f64;

        let better_acc_percentage = (n_circles > 0.0) as u8 as f64
            * (((n300 - (total_hits - n_circles)) * 6.0 + n100 * 2.0 + n50) / (n_circles * 6.0))
                .max(0.0);

        let mut acc_value =
            1.52163_f64.powf(self.attrs.od) * better_acc_percentage.powi(24) * 2.83;

        // Bonus for many hitcircles
        acc_value *= ((n_circles / 1000.0).powf(0.3)).min(1.15);

        // HD bonus
        if self.mods.hd() {
            acc_value *= 1.08;
        }

        // FL bonus
        if self.mods.fl() {
            acc_value *= 1.02;
        }

        acc_value
    }

    fn calculate_miss_penalty(&self, effective_miss_count: f64, total_hits: f64) -> f64 {
        0.97 * (1.0 - (effective_miss_count / total_hits).powf(0.5))
            .powf(1.0 + (effective_miss_count / 1.5))
    }

    const fn total_hits(&self) -> f64 {
        self.state.total_hits() as f64
    }
}

fn calculate_effective_misses(attrs: &OsuDifficultyAttributes, state: &OsuScoreState) -> f64 {
    // * Guess the number of misses + slider breaks from combo
    let mut combo_based_miss_count = 0.0;

    if attrs.n_sliders > 0 {
        let full_combo_threshold = f64::from(attrs.max_combo) - 0.1 * f64::from(attrs.n_sliders);

        if f64::from(state.max_combo) < full_combo_threshold {
            combo_based_miss_count = full_combo_threshold / f64::from(state.max_combo).max(1.0);
        }
    }

    // * Clamp miss count to maximum amount of possible breaks
    combo_based_miss_count =
        combo_based_miss_count.min(f64::from(state.n100 + state.n50 + state.misses));

    combo_based_miss_count.max(f64::from(state.misses))
}

fn accuracy(
    n_slider_ticks: u32,
    n_slider_ends: u32,
    n300: u32,
    n100: u32,
    n50: u32,
    misses: u32,
    max_slider_ticks: u32,
    max_slider_ends: u32,
) -> f64 {
    if n_slider_ticks + n_slider_ends + n300 + n100 + n50 + misses == 0 {
        return 0.0;
    }

    let numerator = 300 * n300 + 100 * n100 + 50 * n50 + 150 * n_slider_ends + 30 * n_slider_ticks;

    let denominator =
        300 * (n300 + n100 + n50 + misses) + 150 * max_slider_ends + 30 * max_slider_ticks;

    f64::from(numerator) / f64::from(denominator)
}