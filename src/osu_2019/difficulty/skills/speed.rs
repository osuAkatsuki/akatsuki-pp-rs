use crate::{any::difficulty::{object::IDifficultyObject, skills::{strain_decay, ISkill, Skill}}, osu_2019::difficulty::object::OsuDifficultyObject, util::strains_vec::StrainsVec};

use super::strain::{DifficultyValue, OsuStrainSkill, UsedOsuStrainSkills};

const SKILL_MULTIPLIER: f64 = 1400.0;
const STRAIN_DECAY_BASE: f64 = 0.3;

#[derive(Clone)]
pub struct Speed {
    curr_strain: f64,
    inner: OsuStrainSkill,
}

impl Speed {
    pub fn new() -> Self {
        Self {
            curr_strain: 0.0,
            inner: OsuStrainSkill::default(),
        }
    }

    pub fn get_curr_strain_peaks(self) -> StrainsVec {
        self.inner.get_curr_strain_peaks().strains()
    }

    pub fn difficulty_value(self) -> UsedOsuStrainSkills<DifficultyValue> {
        Self::static_difficulty_value(self.inner)
    }

    /// Use [`difficulty_value`] instead whenever possible because
    /// [`as_difficulty_value`] clones internally.
    pub fn as_difficulty_value(&self) -> UsedOsuStrainSkills<DifficultyValue> {
        Self::static_difficulty_value(self.inner.clone())
    }

    fn static_difficulty_value(skill: OsuStrainSkill) -> UsedOsuStrainSkills<DifficultyValue> {
        skill.difficulty_value(
            OsuStrainSkill::DECAY_WEIGHT,
        )
    }

    pub fn relevant_note_count(&self) -> f64 {
        self.inner
            .object_strains
            .iter()
            .copied()
            .max_by(f64::total_cmp)
            .filter(|&n| n > 0.0)
            .map_or(0.0, |max_strain| {
                self.inner.object_strains.iter().fold(0.0, |sum, strain| {
                    sum + (1.0 + (-(strain / max_strain * 12.0 - 6.0)).exp()).recip()
                })
            })
    }
}

impl ISkill for Speed {
    type DifficultyObjects<'a> = [OsuDifficultyObject<'a>];
}

impl<'a> Skill<'a, Speed> {
    fn calculate_initial_strain(&mut self, time: f64, curr: &'a OsuDifficultyObject<'a>) -> f64 {
        let prev_start_time = curr
            .previous(0, self.diff_objects)
            .map_or(0.0, |prev| prev.start_time);

        self.inner.curr_strain * strain_decay(time - prev_start_time, STRAIN_DECAY_BASE)
    }

    fn curr_section_peak(&self) -> f64 {
        self.inner.inner.inner.curr_section_peak
    }

    fn curr_section_peak_mut(&mut self) -> &mut f64 {
        &mut self.inner.inner.inner.curr_section_peak
    }

    fn curr_section_end(&self) -> f64 {
        self.inner.inner.inner.curr_section_end
    }

    fn curr_section_end_mut(&mut self) -> &mut f64 {
        &mut self.inner.inner.inner.curr_section_end
    }

    pub fn process(&mut self, curr: &'a OsuDifficultyObject<'a>) {
        if curr.idx == 0 {
            *self.curr_section_end_mut() = (curr.start_time / OsuStrainSkill::SECTION_LEN).ceil()
                * OsuStrainSkill::SECTION_LEN;
        }

        while curr.start_time > self.curr_section_end() {
            self.inner.inner.save_curr_peak();
            let initial_strain = self.calculate_initial_strain(self.curr_section_end(), curr);
            self.inner.inner.start_new_section_from(initial_strain);
            *self.curr_section_end_mut() += OsuStrainSkill::SECTION_LEN;
        }

        let strain_value_at = self.strain_value_at(curr);
        *self.curr_section_peak_mut() = strain_value_at.max(self.curr_section_peak());
    }

    fn strain_value_at(&mut self, curr: &'a OsuDifficultyObject<'a>) -> f64 {
        self.inner.curr_strain *= strain_decay(curr.strain_time, STRAIN_DECAY_BASE);
        self.inner.curr_strain += SpeedEvaluator::evaluate_diff_of(
            curr
        ) * SKILL_MULTIPLIER;

        self.inner.inner.object_strains.push(self.inner.curr_strain);

        self.inner.curr_strain
    }
}

struct SpeedEvaluator;

impl SpeedEvaluator {
    const SINGLE_SPACING_TRESHOLD: f64 = 125.0;
    const ANGLE_BONUS_BEGIN: f64 = 5.0 * std::f64::consts::FRAC_PI_6;
    const PI_OVER_4: f64 = std::f64::consts::FRAC_PI_4;
    const PI_OVER_2: f64 = std::f64::consts::FRAC_PI_2;
    const MIN_SPEED_BONUS: f64 = 75.0;
    const MAX_SPEED_BONUS: f64 = 45.0;
    const SPEED_BALANCING_FACTOR: f64 = 40.0;

    fn evaluate_diff_of<'a>(curr: &'a OsuDifficultyObject<'a>) -> f64 {
        if curr.base.is_spinner() {
            return 0.0;
        }

        let dist = Self::SINGLE_SPACING_TRESHOLD.min(curr.travel_dist + curr.lazy_jump_dist);
        let delta_time = Self::MAX_SPEED_BONUS.max(curr.delta_time);

        let mut speed_bonus = 1.0;

        if delta_time < Self::MIN_SPEED_BONUS {
            let exp_base = (Self::MIN_SPEED_BONUS - delta_time) / Self::SPEED_BALANCING_FACTOR;
            speed_bonus += exp_base * exp_base;
        }

        let mut angle_bonus = 1.0;

        if let Some(angle) = curr.angle.filter(|a| *a < Self::ANGLE_BONUS_BEGIN) {
            let exp_base = (1.5 * (Self::ANGLE_BONUS_BEGIN - angle)).sin();
            angle_bonus = 1.0 + exp_base * exp_base / 3.57;

            if angle < Self::PI_OVER_2 {
                angle_bonus = 1.28;

                if dist < 90.0 && angle < Self::PI_OVER_4 {
                    angle_bonus += (1.0 - angle_bonus) * ((90.0 - dist) / 10.0).min(1.0);
                } else if dist < 90.0 {
                    angle_bonus += (1.0 - angle_bonus)
                        * ((90.0 - dist) / 10.0).min(1.0)
                        * ((Self::PI_OVER_2 - angle) / Self::PI_OVER_4).sin();
                }
            }
        }

        (1.0 + (speed_bonus - 1.0) * 0.75)
            * angle_bonus
            * (0.95 + speed_bonus * (dist / Self::SINGLE_SPACING_TRESHOLD).powf(3.5))
            / curr.strain_time
    }
}