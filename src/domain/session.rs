use crate::domain::exercise::Exercise;
use crate::domain::expression::Expression;
use crate::domain::grade::Grade;
use crate::domain::operation::Operation;
use crate::domain::settings::Settings;
use time::OffsetDateTime;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AnswerError {
    Escaped,
    TimedOut,
    SessionAborted,
    InvalidInput,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Answer {
    pub exercise: Exercise,
    pub entered: Result<i32, AnswerError>,

    #[serde(with = "humantime_serde")]
    pub time_elapsed: std::time::Duration,
}

impl Answer {
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Session {
    settings: Settings,
    answers: Vec<Answer>,
    correct_answers: usize,
    grade: Grade,
}

pub struct SessionExerciseIter<'a> {
    session: &'a Session,
    generated: usize,
}

impl Session {
    pub fn new(settings: Settings) -> Self {
        Session {
            settings,
            answers: Vec::new(),
            correct_answers: 0,
            grade: Grade::default(),
        }
    }

    fn random_operation(&self) -> Operation {
        let operations = Vec::from_iter(self.settings.operations.iter().cloned());
        operations[rand::random_range(0..operations.len())]
    }

    fn random_exercise(&self) -> Result<Exercise, String> {
        Exercise::random(
            self.random_operation(),
            self.settings.limits.result_min,
            self.settings.limits.result_max,
        )
    }

    pub fn exercises(&self) -> SessionExerciseIter<'_> {
        SessionExerciseIter {
            session: self,
            generated: 0,
        }
    }

    fn recalc_grade(&mut self) {
        self.grade = Grade::from_quantity(self.correct_answers, self.answers.len());
    }

    pub fn add_answer(&mut self, answer: Answer) {
        if answer.is_correct() {
            self.correct_answers += 1;
            self.recalc_grade();
        }
        self.answers.push(answer);
    }

    pub fn get_answers(&self) -> &Vec<Answer> {
        &self.answers
    }
    pub fn get_grade(&self) -> Grade {
        self.grade
    }

    pub fn total_answers(&self) -> usize {
        self.answers.len()
    }

    pub fn exercises_left(&self) -> usize {
        self.settings.limits.exercise_count - self.answers.len()
    }

    pub fn is_finished(&self) -> bool {
        self.exercises_left() == 0
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        use std::fs;

        let translit_name = deunicode::deunicode(&self.settings.player_name);
        let results_dir = format!("{}/{}", self.settings.results_dir, translit_name);
        if !fs::exists(&results_dir)? {
            fs::create_dir_all(&results_dir)?;
        }
        let file_path = format!("{}/{}.json", results_dir, OffsetDateTime::now_utc());
        let file = fs::File::create(&file_path)?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }
}

impl Iterator for SessionExerciseIter<'_> {
    type Item = Result<Exercise, String>;

    fn next(&mut self) -> Option<Self::Item> {
        let answered_or_generated = self.session.answers.len() + self.generated;
        if answered_or_generated >= self.session.settings.limits.exercise_count {
            return None;
        }

        self.generated += 1;
        Some(self.session.random_exercise())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashSet;

    fn settings(exercise_count: usize) -> Settings {
        Settings {
            player_name: "test".to_string(),
            results_dir: "results".to_string(),
            operations: HashSet::from([Operation::Addition]),
            limits: crate::domain::settings::Limits {
                result_min: 100,
                result_max: 150,
                exercise_count,
                answer_time_seconds: std::time::Duration::from_secs(30),
            },
        }
    }

    fn answer() -> Answer {
        Answer {
            exercise: Exercise::new(2, Operation::Addition, 3),
            entered: Ok(5),
            time_elapsed: std::time::Duration::from_secs(1),
        }
    }

    fn failed_answer(error: AnswerError) -> Answer {
        Answer {
            exercise: Exercise::new(7, Operation::DivisionWithRemainder, 3),
            entered: Err(error),
            time_elapsed: std::time::Duration::from_secs(2),
        }
    }

    #[test]
    fn test_exercises_iter_generates_until_exercise_count() {
        let session = Session::new(settings(3));
        let exercises = session.exercises().collect::<Result<Vec<_>, _>>().unwrap();

        assert_eq!(exercises.len(), 3);
    }

    #[test]
    fn test_exercises_iter_counts_existing_answers() {
        let mut session = Session::new(settings(3));
        session.answers.push(answer());

        let exercises = session.exercises().collect::<Result<Vec<_>, _>>().unwrap();

        assert_eq!(exercises.len(), 2);
    }

    #[test]
    fn test_exercises_iter_stops_when_limit_is_zero() {
        let session = Session::new(settings(0));

        assert!(session.exercises().next().is_none());
    }

    #[test]
    fn test_answer_serializes_to_expected_json() {
        let serialized = serde_json::to_value(answer()).unwrap();

        assert_eq!(
            serialized,
            json!({
                "exercise": {
                    "left": 2,
                    "operation": "+",
                    "right": 3,
                },
                "entered": {
                    "Ok": 5,
                },
                "time_elapsed": "1s",
            })
        );
    }

    #[test]
    fn test_answer_error_deserializes_from_json() {
        let input = json!({
            "exercise": {
                "left": 7,
                "operation": ":",
                "right": 3,
            },
            "entered": {
                "Err": "TimedOut",
            },
            "time_elapsed": "2s",
        });

        let deserialized: Answer = serde_json::from_value(input).unwrap();

        assert_eq!(
            deserialized.exercise,
            Exercise::new(7, Operation::DivisionWithRemainder, 3)
        );
        assert!(matches!(deserialized.entered, Err(AnswerError::TimedOut)));
        assert_eq!(deserialized.time_elapsed, std::time::Duration::from_secs(2));
        assert!(!deserialized.is_correct());
    }

    #[test]
    fn test_session_serialization_round_trip_preserves_fields() {
        let session = Session {
            settings: Settings {
                player_name: "ученик".to_string(),
                results_dir: "results".to_string(),
                operations: HashSet::from([Operation::Addition, Operation::DivisionWithRemainder]),
                limits: crate::domain::settings::Limits {
                    result_min: 10,
                    result_max: 100,
                    exercise_count: 2,
                    answer_time_seconds: std::time::Duration::from_secs(45),
                },
            },
            answers: vec![answer(), failed_answer(AnswerError::Escaped)],
            correct_answers: 1,
            grade: Grade::Three,
        };

        let serialized = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.settings.player_name, "ученик");
        assert_eq!(deserialized.settings.results_dir, "results");
        assert_eq!(
            deserialized.settings.operations,
            HashSet::from([Operation::Addition, Operation::DivisionWithRemainder])
        );
        assert_eq!(deserialized.settings.limits.result_min, 10);
        assert_eq!(deserialized.settings.limits.result_max, 100);
        assert_eq!(deserialized.settings.limits.exercise_count, 2);
        assert_eq!(
            deserialized.settings.limits.answer_time_seconds,
            std::time::Duration::from_secs(45)
        );
        assert_eq!(deserialized.answers.len(), 2);
        assert_eq!(
            deserialized.answers[0].exercise,
            Exercise::new(2, Operation::Addition, 3)
        );
        assert!(matches!(deserialized.answers[0].entered, Ok(5)));
        assert_eq!(
            deserialized.answers[0].time_elapsed,
            std::time::Duration::from_secs(1)
        );
        assert_eq!(
            deserialized.answers[1].exercise,
            Exercise::new(7, Operation::DivisionWithRemainder, 3)
        );
        assert!(matches!(
            deserialized.answers[1].entered,
            Err(AnswerError::Escaped)
        ));
        assert_eq!(
            deserialized.answers[1].time_elapsed,
            std::time::Duration::from_secs(2)
        );
        assert_eq!(deserialized.correct_answers, 1);
        assert!(matches!(deserialized.grade, Grade::Three));
    }

    #[test]
    fn test_session_deserializes_saved_json_shape() {
        let input = r#"{
  "settings": {
    "player_name": "test",
    "results_dir": "results",
    "operations": ["+", ":"],
    "limits": {
      "result_min": 100,
      "result_max": 150,
      "exercise_count": 2,
      "answer_time_seconds": "30s"
    }
  },
  "answers": [
    {
      "exercise": {
        "left": 2,
        "operation": "+",
        "right": 3
      },
      "entered": {
        "Ok": 5
      },
      "time_elapsed": "1s"
    },
    {
      "exercise": {
        "left": 7,
        "operation": ":",
        "right": 3
      },
      "entered": {
        "Err": "InvalidInput"
      },
      "time_elapsed": "250ms"
    }
  ],
  "correct_answers": 1,
  "grade": "Three"
}"#;

        let session: Session = serde_json::from_str(input).unwrap();

        assert_eq!(session.settings.player_name, "test");
        assert_eq!(
            session.settings.operations,
            HashSet::from([Operation::Addition, Operation::DivisionWithRemainder])
        );
        assert_eq!(
            session.settings.limits.answer_time_seconds,
            std::time::Duration::from_secs(30)
        );
        assert_eq!(session.answers.len(), 2);
        assert!(matches!(session.answers[0].entered, Ok(5)));
        assert!(matches!(
            session.answers[1].entered,
            Err(AnswerError::InvalidInput)
        ));
        assert_eq!(
            session.answers[1].time_elapsed,
            std::time::Duration::from_millis(250)
        );
        assert_eq!(session.correct_answers, 1);
        assert!(matches!(session.grade, Grade::Three));
    }
}
