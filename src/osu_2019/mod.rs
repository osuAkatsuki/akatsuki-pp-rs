use convert::OsuRelaxBeatmap;
use rosu_map::util::Pos;
use strains::OsuStrains;

use crate::{model::mode::{ConvertStatus, IGameMode}, Beatmap, Difficulty};

mod difficulty;
mod attributes;
mod performance;
mod strains;
mod object;
mod convert;

pub use performance::OsuPerformance;
pub use attributes::OsuPerformanceAttributes;
pub use attributes::OsuDifficultyAttributes;
pub use performance::gradual::OsuGradualPerformance;
pub use difficulty::gradual::OsuGradualDifficulty;

const PLAYFIELD_BASE_SIZE: Pos = Pos::new(512.0, 384.0);

/// Marker type for [`GameMode::Osu`] with the Relax mod.
///
/// [`GameMode::Osu`]: rosu_map::section::general::GameMode::Osu
pub struct OsuRelax;

impl IGameMode for OsuRelax {
    type DifficultyAttributes = OsuDifficultyAttributes;
    type Strains = OsuStrains;
    type Performance<'map> = OsuPerformance<'map>;
    type GradualDifficulty = OsuGradualDifficulty;
    type GradualPerformance = OsuGradualPerformance;

    fn check_convert(map: &Beatmap) -> ConvertStatus {
        convert::check_convert(map)
    }

    fn try_convert(map: &mut Beatmap) -> ConvertStatus {
        convert::try_convert(map)
    }

    fn difficulty(
        difficulty: &Difficulty,
        converted: &OsuRelaxBeatmap<'_>,
    ) -> Self::DifficultyAttributes {
        difficulty::difficulty(difficulty, converted)
    }

    fn strains(difficulty: &Difficulty, converted: &OsuRelaxBeatmap<'_>) -> Self::Strains {
        strains::strains(difficulty, converted)
    }

    fn performance(map: OsuRelaxBeatmap<'_>) -> Self::Performance<'_> {
        OsuPerformance::new(map)
    }

    fn gradual_difficulty(difficulty: Difficulty, map: &OsuRelaxBeatmap<'_>) -> Self::GradualDifficulty {
        OsuGradualDifficulty::new(difficulty, map)
    }

    fn gradual_performance(
        difficulty: Difficulty,
        map: &OsuRelaxBeatmap<'_>,
    ) -> Self::GradualPerformance {
        OsuGradualPerformance::new(difficulty, map)
    }
}
