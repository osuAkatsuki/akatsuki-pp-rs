use crate::{
    osu::{OsuBeatmap, OsuGradualDifficulty},
    Difficulty,
};

use super::{OsuPerformanceAttributes, OsuScoreState};

/// Gradually calculate the performance attributes of an osu!standard map.
///
/// After each hit object you can call [`next`]
/// and it will return the resulting current [`OsuPerformanceAttributes`].
/// To process multiple objects at once, use [`nth`] instead.
///
/// Both methods require an [`OsuScoreState`] that contains the current
/// hitresults as well as the maximum combo so far.
///
/// If you only want to calculate difficulty attributes use
/// [`OsuGradualDifficulty`] instead.
///
/// # Example
///
/// ```
/// use rosu_pp::{Beatmap, Difficulty};
/// use rosu_pp::osu::{Osu, OsuGradualPerformance, OsuScoreState};
///
/// let converted = Beatmap::from_path("./resources/2785319.osu")
///     .unwrap()
///     .unchecked_into_converted::<Osu>();
///
/// let difficulty = Difficulty::new().mods(64); // DT
/// let mut gradual = OsuGradualPerformance::new(difficulty, &converted);
/// let mut state = OsuScoreState::new(); // empty state, everything is on 0.
///
/// // The first 10 hits are 300s and there are no sliders for additional combo
/// for _ in 0..10 {
///     state.n300 += 1;
///     state.max_combo += 1;
///
///     let attrs = gradual.next(state.clone()).unwrap();
///     println!("PP: {}", attrs.pp);
/// }
///
/// // Then comes a miss. Note that state's max combo won't be incremented for
/// // the next few objects because the combo is reset.
/// state.misses += 1;
/// let attrs = gradual.next(state.clone()).unwrap();
/// println!("PP: {}", attrs.pp);
///
/// // The next 10 objects will be a mixture of 300s, 100s, and 50s.
/// // Notice how all 10 objects will be processed in one go.
/// state.n300 += 2;
/// state.n100 += 7;
/// state.n50 += 1;
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
/// state.n50 = ...
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
/// [`next`]: OsuGradualPerformance::next
/// [`nth`]: OsuGradualPerformance::nth
pub struct OsuGradualPerformance {
    difficulty: OsuGradualDifficulty,
}

impl OsuGradualPerformance {
    /// Create a new gradual performance calculator for osu!standard maps.
    pub fn new(difficulty: Difficulty, converted: &OsuBeatmap<'_>) -> Self {
        let difficulty = OsuGradualDifficulty::new(difficulty, converted);

        Self { difficulty }
    }

    /// Process the next hit object and calculate the performance attributes
    /// for the resulting score state.
    pub fn next(&mut self, state: OsuScoreState) -> Option<OsuPerformanceAttributes> {
        self.nth(state, 0)
    }

    /// Process all remaining hit objects and calculate the final performance
    /// attributes.
    pub fn last(&mut self, state: OsuScoreState) -> Option<OsuPerformanceAttributes> {
        self.nth(state, usize::MAX)
    }

    /// Process everything up to the next `n`th hitobject and calculate the
    /// performance attributes for the resulting score state.
    ///
    /// Note that the count is zero-indexed, so `n=0` will process 1 object,
    /// `n=1` will process 2, and so on.
    pub fn nth(&mut self, state: OsuScoreState, n: usize) -> Option<OsuPerformanceAttributes> {
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
