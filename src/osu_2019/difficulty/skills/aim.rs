use crate::{any::difficulty::{object::IDifficultyObject, skills::{strain_decay, ISkill, Skill}}, osu_2019::difficulty::object::OsuDifficultyObject, util::strains_vec::StrainsVec};

use super::strain::{DifficultyValue, OsuStrainSkill, UsedOsuStrainSkills};

const SKILL_MULTIPLIER: f64 = 26.25;
const STRAIN_DECAY_BASE: f64 = 0.15;

#[derive(Clone)]
pub struct Aim {
    curr_strain: f64,
    inner: OsuStrainSkill,
}

impl Aim {
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
}

impl ISkill for Aim {
    type DifficultyObjects<'a> = [OsuDifficultyObject<'a>];
}

impl<'a> Skill<'a, Aim> {
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
        self.inner.curr_strain *= strain_decay(curr.delta_time, STRAIN_DECAY_BASE);
        self.inner.curr_strain +=
            AimEvaluator::evaluate_diff_of(curr, self.diff_objects)
                * SKILL_MULTIPLIER;
        self.inner.inner.object_strains.push(self.inner.curr_strain);

        self.inner.curr_strain
    }
}

struct AimEvaluator;

impl AimEvaluator {
    const ANGLE_BONUS_BEGIN: f64 = std::f64::consts::FRAC_PI_3;
    const TIMING_THRESHOLD: f64 = 107.0;

    fn evaluate_diff_of<'a>(
        curr: &'a OsuDifficultyObject<'a>,
        diff_objects: &'a [OsuDifficultyObject<'a>],
    ) -> f64 {
        if curr.base.is_spinner() {
            return 0.0;
        }

        let mut result = 0.0;

        if let Some(prev) = curr.previous(0, diff_objects) {
            if let Some(angle) = curr.angle.filter(|a| *a > Self::ANGLE_BONUS_BEGIN) {
                let scale = 90.0;

                let angle_bonus = (((angle - Self::ANGLE_BONUS_BEGIN).sin()).powi(2)
                    * (prev.lazy_jump_dist - scale).max(0.0)
                    * (curr.lazy_jump_dist - scale).max(0.0))
                .sqrt();

                result = 1.5 * apply_diminishing_exp(angle_bonus.max(0.0))
                    / (Self::TIMING_THRESHOLD).max(prev.strain_time)
            }
        }

        let jump_dist_exp = apply_diminishing_exp(curr.lazy_jump_dist);
        let travel_dist_exp = apply_diminishing_exp(curr.travel_dist);

        let dist_exp =
            jump_dist_exp + travel_dist_exp + (travel_dist_exp * jump_dist_exp).sqrt();

        (result + dist_exp / (curr.strain_time).max(Self::TIMING_THRESHOLD))
            .max(dist_exp / curr.strain_time)
    }
}

#[inline]
fn apply_diminishing_exp(val: f64) -> f64 {
    val.powf(0.99)
}