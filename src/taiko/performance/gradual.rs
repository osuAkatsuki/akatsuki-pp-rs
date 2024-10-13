use crate::{
    taiko::{difficulty::gradual::TaikoGradualDifficulty, TaikoBeatmap, TaikoScoreState},
    Difficulty,
};

use super::TaikoPerformanceAttributes;

/// Gradually calculate the performance attributes of an osu!taiko map.
///
/// After each hit object you can call [`next`] and it will return the
/// resulting current [`TaikoPerformanceAttributes`]. To process multiple
/// objects at once, use [`nth`] instead.
///
/// Both methods require a [`TaikoScoreState`] that contains the current
/// hitresults as well as the maximum combo so far.
///
/// If you only want to calculate difficulty attributes use
/// [`TaikoGradualDifficulty`] instead.
///
/// # Example
///
/// ```
/// use rosu_pp::{Beatmap, Difficulty};
/// use rosu_pp::taiko::{Taiko, TaikoGradualPerformance, TaikoScoreState};
///
/// let converted = Beatmap::from_path("./resources/1028484.osu")
///     .unwrap()
///     .unchecked_into_converted::<Taiko>();
///
/// let difficulty = Difficulty::new().mods(64); // DT
/// let mut gradual = TaikoGradualPerformance::new(difficulty, &converted);
/// let mut state = TaikoScoreState::new(); // empty state, everything is on 0.
///
/// // The first 10 hitresults are 300s
/// for _ in 0..10 {
///     state.n300 += 1;
///     state.max_combo += 1;
///
///     let attrs = gradual.next(state.clone()).unwrap();
///     println!("PP: {}", attrs.pp);
/// }
///
/// // Then comes a miss.
/// // Note that state's max combo won't be incremented for
/// // the next few objects because the combo is reset.
/// state.misses += 1;
/// let attrs = gradual.next(state.clone()).unwrap();
/// println!("PP: {}", attrs.pp);
///
/// // The next 10 objects will be a mixture of 300s and 100s.
/// // Notice how all 10 objects will be processed in one go.
/// state.n300 += 3;
/// state.n100 += 7;
/// // The `nth` method takes a zero-based value.
/// let attrs = gradual.nth(state.clone(), 9).unwrap();
/// println!("PP: {}", attrs.pp);
///
/// // Now comes another 300. Note that the max combo gets incremented again.
/// state.n300 += 1;
/// state.max_combo += 1;
/// let attrs = gradual.next(state.clone()).unwrap();
/// println!("PP: {}", attrs.pp);
///
/// // Skip to the end
/// # /*
/// state.max_combo = ...
/// state.n300 = ...
/// state.n100 = ...
/// state.misses = ...
/// # */
/// let attrs = gradual.last(state.clone()).unwrap();
/// println!("PP: {}", attrs.pp);
///
/// // Once the final performance has been calculated, attempting to process
/// // further objects will return `None`.
/// assert!(gradual.next(state).is_none());
/// ```
///
/// [`next`]: TaikoGradualPerformance::next
/// [`nth`]: TaikoGradualPerformance::nth
pub struct TaikoGradualPerformance {
    difficulty: TaikoGradualDifficulty,
}

impl TaikoGradualPerformance {
    /// Create a new gradual performance calculator for osu!taiko maps.
    pub fn new(difficulty: Difficulty, converted: &TaikoBeatmap<'_>) -> Self {
        let difficulty = TaikoGradualDifficulty::new(difficulty, converted);

        Self { difficulty }
    }

    /// Process the next hit object and calculate the performance attributes
    /// for the resulting score.
    pub fn next(&mut self, state: TaikoScoreState) -> Option<TaikoPerformanceAttributes> {
        self.nth(state, 0)
    }

    /// Process all remaining hit objects and calculate the final performance
    /// attributes.
    pub fn last(&mut self, state: TaikoScoreState) -> Option<TaikoPerformanceAttributes> {
        self.nth(state, usize::MAX)
    }

    /// Process everything up the the next `n`th hit object and calculate the
    /// performance attributes for the resulting score state.
    ///
    /// Note that the count is zero-indexed, so `n=0` will process 1 object,
    /// `n=1` will process 2, and so on.
    pub fn nth(&mut self, state: TaikoScoreState, n: usize) -> Option<TaikoPerformanceAttributes> {
        let performance = self
            .difficulty
            .nth(n)?
            .performance()
            .state(state)
            .difficulty(self.difficulty.difficulty.clone())
            .passed_objects(self.difficulty.idx as u32)
            .calculate();

        Some(performance)
    }

    /// Returns the amount of remaining objects.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.difficulty.len()
    }
}
