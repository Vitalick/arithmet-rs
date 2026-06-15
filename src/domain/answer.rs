use crate::domain::expression::{Exercise, Expression};
use crate::domain::operation::Operation;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum AnswerError {
    Escaped,
    TimedOut,
    SessionAborted,
    InvalidInput,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Answer {
    pub exercise: Exercise,
    pub entered: Result<i32, AnswerError>,

    #[serde(with = "humantime_serde")]
    pub time_elapsed: std::time::Duration,
}

impl Default for Answer {
    fn default() -> Self {
        Self {
            exercise: Exercise::new(0, Operation::Addition, 0),
            entered: Ok(0),
            time_elapsed: std::time::Duration::from_secs(0),
        }
    }
}

impl Answer {
    pub fn protocol_entered(&self) -> String {
        match self.entered {
            Ok(entered) => format!("введено {:<5}", entered),
            Err(AnswerError::Escaped) => "нажата клавиша <Esc>".to_string(),
            Err(AnswerError::TimedOut) => "вышло время".to_string(),
            Err(AnswerError::SessionAborted) => "нажата клавиша <F10>".to_string(),
            Err(AnswerError::InvalidInput) => "некорректный ввод".to_string(),
        }
    }

    pub fn banner(&self) -> String {
        match self.entered {
            Ok(_) => {
                if self.is_correct() {
                    "Молодец!".to_string()
                } else {
                    "Неверно...".to_string()
                }
            }
            Err(AnswerError::Escaped | AnswerError::SessionAborted) => "Игра прервана".to_string(),
            Err(AnswerError::TimedOut) => "Время вышло!".to_string(),
            Err(AnswerError::InvalidInput) => "Неверно :(".to_string(),
        }
    }

    pub fn is_correct(&self) -> bool {
        match self.entered {
            Ok(entered) => match self.exercise.compare(entered) {
                Ok(true) => true,
                _ => false,
            },
            Err(_) => false,
        }
    }

    pub fn check_expressions(&self) -> Result<[Box<dyn Expression>; 2], String> {
        match self.entered {
            Ok(entered) => self.exercise.check_expressions(entered),
            Err(_) => Err("Cannot check expressions with invalid input".to_string()),
        }
    }
}