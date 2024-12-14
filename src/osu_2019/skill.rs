use crate::util::strains_vec::StrainsVec;

use super::{DifficultyObject, SkillKind};

const SPEED_SKILL_MULTIPLIER: f64 = 1400.0;
const SPEED_STRAIN_DECAY_BASE: f64 = 0.3;

const AIM_SKILL_MULTIPLIER: f64 = 26.4;
const AIM_STRAIN_DECAY_BASE: f64 = 0.15;

const DECAY_WEIGHT: f64 = 0.9;

const REDUCED_STRAIN_BASELINE: f64 = 0.75;

pub(crate) struct Skill {
    current_strain: f64,
    current_section_peak: f64,

    kind: SkillKind,
    pub(crate) strain_peaks: StrainsVec,

    prev_time: Option<f64>,
    pub(crate) object_strains: Vec<f64>,

    difficulty_value: Option<f64>,

    section_length: f64,
}

impl Skill {
    #[inline]
    pub(crate) fn new(kind: SkillKind, section_length: f64) -> Self {
        Self {
            current_strain: 1.0,
            current_section_peak: 1.0,

            kind,
            strain_peaks: StrainsVec::with_capacity(256),

            prev_time: None,
            object_strains: Vec::new(),

            difficulty_value: None,

            section_length,
        }
    }

    #[inline]
    pub(crate) fn save_current_peak(&mut self) {
        self.strain_peaks.push(self.current_section_peak);
    }

    #[inline]
    pub(crate) fn start_new_section_from(&mut self, time: f64) {
        self.current_section_peak = self.peak_strain(time - self.prev_time.unwrap());
    }

    #[inline]
    pub(crate) fn process(&mut self, current: &DifficultyObject<'_>) {
        self.current_strain *= self.strain_decay(current.delta);
        self.current_strain += self.kind.strain_value_of(current) * self.skill_multiplier();

        self.object_strains.push(self.current_strain);

        self.current_section_peak = self.current_section_peak.max(self.current_strain);
        self.prev_time.replace(current.base.time);
    }

    pub(crate) fn difficulty_value(&mut self) -> f64 {
        let mut difficulty = 0.0;
        let mut weight = 1.0;

        let reduced_section_count = 30_000 / self.section_length as usize;

        let peaks_iter = self
            .strain_peaks
            .sorted_non_zero_iter_mut()
            .take(reduced_section_count);

        for (i, strain) in peaks_iter.enumerate() {
            let clamped = f64::from((i as f32 / reduced_section_count as f32).clamp(0.0, 1.0));
            let scale = (lerp(1.0, 10.0, clamped)).log10();
            *strain *= lerp(REDUCED_STRAIN_BASELINE, 1.0, scale);
        }

        self.strain_peaks.sort_desc();

        for strain in self.strain_peaks.iter() {
            difficulty += strain * weight;
            weight *= DECAY_WEIGHT;
        }

        self.difficulty_value = Some(difficulty);

        difficulty
    }

    pub(crate) fn count_difficult_strains(&mut self) -> f64 {
        let difficulty_value = self.difficulty_value.unwrap_or(self.difficulty_value());
        let single_strain = difficulty_value / 10.0;

        self.object_strains
            .iter()
            .map(|strain| 1.1 / (1.0 + (-10.0 * (strain / single_strain - 0.88)).exp()))
            .sum::<f64>()
    }

    pub(crate) fn relevant_note_count(&self) -> f64 {
        self.object_strains
            .iter()
            .copied()
            .max_by(f64::total_cmp)
            .filter(|&n| n > 0.0)
            .map_or(0.0, |max_strain| {
                self.object_strains.iter().fold(0.0, |sum, strain| {
                    sum + (1.0 + (-(strain / max_strain * 12.0 - 6.0)).exp()).recip()
                })
            })
    }

    #[inline]
    fn skill_multiplier(&self) -> f64 {
        match self.kind {
            SkillKind::Aim => AIM_SKILL_MULTIPLIER,
            SkillKind::Speed => SPEED_SKILL_MULTIPLIER,
        }
    }

    #[inline]
    fn strain_decay_base(&self) -> f64 {
        match self.kind {
            SkillKind::Aim => AIM_STRAIN_DECAY_BASE,
            SkillKind::Speed => SPEED_STRAIN_DECAY_BASE,
        }
    }

    #[inline]
    fn peak_strain(&self, delta_time: f64) -> f64 {
        self.current_strain * self.strain_decay(delta_time)
    }

    #[inline]
    fn strain_decay(&self, ms: f64) -> f64 {
        self.strain_decay_base().powf(ms / 1000.0)
    }
}

fn lerp(start: f64, end: f64, amount: f64) -> f64 {
    start + (end - start) * amount
}
