use std::{cmp, pin::Pin};

use object::OsuDifficultyObject;
use skills::{strain::{DifficultyValue, UsedOsuStrainSkills}, OsuSkills};

use crate::{any::difficulty::skills::Skill, model::beatmap::BeatmapAttributes, osu::difficulty::scaling_factor::ScalingFactor, Difficulty};

use super::{attributes::OsuDifficultyAttributes, convert::{convert_objects, OsuRelaxBeatmap}, object::OsuObject};

pub mod skills;
pub mod gradual;
pub mod object;

const DIFFICULTY_MULTIPLIER: f64 = 0.0675;

pub fn difficulty(difficulty: &Difficulty, converted: &OsuRelaxBeatmap) -> OsuDifficultyAttributes {
    let DifficultyValues {
        skills:
            OsuSkills {
                aim,
                speed,
            },
        mut attrs,
    } = DifficultyValues::calculate(difficulty, converted);

    let aim_difficulty_value = aim.difficulty_value();
    let speed_difficulty_value = speed.difficulty_value();

    DifficultyValues::eval(
        &mut attrs,
        &aim_difficulty_value,
        &speed_difficulty_value,
    );

    attrs
}

pub struct OsuDifficultySetup {
    scaling_factor: ScalingFactor,
    map_attrs: BeatmapAttributes,
    attrs: OsuDifficultyAttributes,
    time_preempt: f64,
}

impl OsuDifficultySetup {
    pub fn new(difficulty: &Difficulty, beatmap: &OsuRelaxBeatmap) -> Self {
        let clock_rate = difficulty.get_clock_rate();
        let map_attrs = beatmap.attributes().difficulty(difficulty).build();
        let scaling_factor = ScalingFactor::new(map_attrs.cs);

        let attrs = OsuDifficultyAttributes {
            ar: map_attrs.ar,
            hp: map_attrs.hp,
            od: map_attrs.od,
            ..Default::default()
        };

        let time_preempt = f64::from((map_attrs.hit_windows.ar * clock_rate) as f32);

        Self {
            scaling_factor,
            map_attrs,
            attrs,
            time_preempt,
        }
    }
}

pub struct DifficultyValues {
    pub skills: OsuSkills,
    pub attrs: OsuDifficultyAttributes,
}

impl DifficultyValues {
    pub fn calculate(difficulty: &Difficulty, converted: &OsuRelaxBeatmap) -> Self {
        let mods = difficulty.get_mods();
        let take = difficulty.get_passed_objects();

        let OsuDifficultySetup {
            scaling_factor,
            map_attrs: _,
            mut attrs,
            time_preempt,
        } = OsuDifficultySetup::new(difficulty, converted);

        let mut osu_objects = convert_objects(
            converted,
            &scaling_factor,
            mods.hr(),
            time_preempt,
            take,
            &mut attrs,
        );

        let osu_object_iter = osu_objects.iter_mut().map(Pin::new);

        let diff_objects =
            Self::create_difficulty_objects(difficulty, &scaling_factor, osu_object_iter);

        let mut skills = OsuSkills::new();

        {
            let mut aim = Skill::new(&mut skills.aim, &diff_objects);
            let mut speed = Skill::new(&mut skills.speed, &diff_objects);

            // The first hit object has no difficulty object
            let take_diff_objects = cmp::min(converted.hit_objects.len(), take).saturating_sub(1);

            for hit_object in diff_objects.iter().take(take_diff_objects) {
                aim.process(hit_object);
                speed.process(hit_object);
            }
        }

        Self { skills, attrs }
    }

    /// Process the difficulty values and store the results in `attrs`.
    pub fn eval(
        attrs: &mut OsuDifficultyAttributes,
        aim: &UsedOsuStrainSkills<DifficultyValue>,
        speed: &UsedOsuStrainSkills<DifficultyValue>,
    ) {
        let aim_rating = aim.difficulty_value().sqrt() * DIFFICULTY_MULTIPLIER;
        let speed_rating = speed.difficulty_value().sqrt() * DIFFICULTY_MULTIPLIER;

        let aim_difficult_strain_count = aim.count_difficult_strains();
        let speed_difficult_strain_count = speed.count_difficult_strains();

        let stars = aim_rating + speed_rating + (aim_rating - speed_rating).abs() / 2.0;

        attrs.aim_strain = aim_rating;
        attrs.speed_strain = speed_rating;
        attrs.aim_difficult_strain_count = aim_difficult_strain_count;
        attrs.speed_difficult_strain_count = speed_difficult_strain_count;
        attrs.stars = stars;
    }

    pub fn create_difficulty_objects<'a>(
        difficulty: &Difficulty,
        scaling_factor: &ScalingFactor,
        osu_objects: impl ExactSizeIterator<Item = Pin<&'a mut OsuObject>>,
    ) -> Vec<OsuDifficultyObject<'a>> {
        let take = difficulty.get_passed_objects();
        let clock_rate = difficulty.get_clock_rate();

        let mut osu_objects_iter = osu_objects
            .map(|h| OsuDifficultyObject::compute_slider_cursor_pos(h, scaling_factor.radius))
            .map(Pin::into_ref);

        let Some(mut last) = osu_objects_iter.next().filter(|_| take > 0) else {
            return Vec::new();
        };

        let mut last_last = None;

        osu_objects_iter
            .enumerate()
            .map(|(idx, h)| {
                let diff_object = OsuDifficultyObject::new(
                    h.get_ref(),
                    last.get_ref(),
                    last_last.as_deref(),
                    clock_rate,
                    idx,
                    scaling_factor,
                );

                last_last = Some(last);
                last = h;

                diff_object
            })
            .collect()
    }
}