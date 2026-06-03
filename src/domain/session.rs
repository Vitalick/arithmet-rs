use crate::domain::exercise::Exercise;
use crate::domain::grade::Grade;
use crate::domain::operation::Operation;
use crate::domain::settings::Settings;
use rand::random_range;

#[derive(Debug, Clone)]
pub enum AnswerError {
    Escaped,
    TimedOut,
    SessionAborted,
    InvalidInput,
}

#[derive(Debug, Clone)]
pub struct Answer {
    pub exercise: Exercise,
    pub entered: Result<i32, AnswerError>,
    pub time_elapsed: u64,
    pub is_correct: bool,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub settings: Settings,
    pub answers: Vec<Answer>,
    pub grade: Grade,
}

impl Session {
    pub fn new(settings: Settings) -> Self {
        Session {
            settings,
            answers: Vec::new(),
            grade: Grade::default(),
        }
    }

    fn random_operation(&self) -> Operation {
        let operations = Vec::from_iter(self.settings.operations.iter().cloned());
        operations[random_range(0..operations.len())]
    }

    fn random_exercise(&self) -> Result<Exercise, String> {
        Exercise::random(
            self.random_operation(),
            self.settings.limits.result_min,
            self.settings.limits.result_max,
        )
    }
}
