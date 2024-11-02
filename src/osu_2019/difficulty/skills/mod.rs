use aim::Aim;
use speed::Speed;

pub mod aim;
pub mod speed;
pub mod strain;

pub struct OsuSkills {
    pub aim: Aim,
    pub speed: Speed,
}

impl OsuSkills {
    pub fn new() -> Self {
        let aim = Aim::new();
        let speed = Speed::new();

        Self {
            aim,
            speed,
        }
    }
}