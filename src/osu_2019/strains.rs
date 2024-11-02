use crate::Difficulty;

use super::{
    difficulty::{skills::OsuSkills, DifficultyValues}, OsuRelaxBeatmap,
};

/// The result of calculating the strains on a osu! map.
///
/// Suitable to plot the difficulty of a map over time.
#[derive(Clone, Debug, PartialEq)]
pub struct OsuStrains {
    /// Strain peaks of the aim skill.
    pub aim: Vec<f64>,
    /// Strain peaks of the speed skill.
    pub speed: Vec<f64>,
}

impl OsuStrains {
    /// Time between two strains in ms.
    pub const SECTION_LEN: f64 = 400.0;
}

pub fn strains(difficulty: &Difficulty, converted: &OsuRelaxBeatmap<'_>) -> OsuStrains {
    let DifficultyValues {
        skills:
            OsuSkills {
                aim,
                speed,
            },
        attrs: _,
    } = DifficultyValues::calculate(difficulty, converted);

    OsuStrains {
        aim: aim.get_curr_strain_peaks().into_vec(),
        speed: speed.get_curr_strain_peaks().into_vec(),
    }
}
