//! **Deprecated**: This module contains the 2019-era osu!standard PP formulas.
//!
//! For new code, use the modern [`osu`](crate::osu) module which now supports
//! Relax (RX) and Autopilot (AP) modes with up-to-date formulas.
//!
//! This module is maintained for backwards compatibility only.

mod difficulty_object;
use difficulty_object::DifficultyObject;

mod osu_object;
use osu_object::OsuObject;

mod pp;
pub use pp::{OsuAttributeProvider, OsuPP};

mod skill;
use skill::Skill;

mod skill_kind;
use skill_kind::SkillKind;

pub mod stars;
