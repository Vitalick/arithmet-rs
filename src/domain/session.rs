use crate::domain::exercise::Exercise;
use crate::domain::grade::Grade;
use crate::domain::settings::Settings;


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
}