use crate::domain::exercise::Exercise;
use crate::domain::expression::Expression;
use crate::domain::grade::Grade;
use crate::domain::operation::Operation;
use crate::domain::settings::Settings;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use time::OffsetDateTime;
use validations::Validate;

const PROTOCOL_OPERATION_ORDER: [Operation; 5] = [
    Operation::Addition,
    Operation::Subtraction,
    Operation::Multiplication,
    Operation::Division,
    Operation::DivisionWithRemainder,
];

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
    fn protocol_entered(&self) -> String {
        match self.entered {
            Ok(entered) => format!("введено {:<5}", entered),
            Err(AnswerError::Escaped) => "нажата клавиша <Esc>".to_string(),
            Err(AnswerError::TimedOut) => "вышло время".to_string(),
            Err(AnswerError::SessionAborted) => "нажата клавиша <F10>".to_string(),
            Err(AnswerError::InvalidInput) => "некорректный ввод".to_string(),
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Session {
    settings: Settings,
    answers: Vec<Answer>,
    // iterator: SessionExerciseIter,
    correct_answers: usize,
    grade: Grade,
}

impl Session {
    pub fn new(settings: Settings) -> Result<Self, String> {
        match settings.validate() {
            Ok(_) => Ok(Session {
                settings,
                answers: Vec::new(),
                correct_answers: 0,
                // iterator: SessionExerciseIter::new(settings),
                grade: Grade::default(),
            }),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn have_next(&self) -> bool {
        self.answers.len() < self.settings.limits.exercise_count
    }

    pub fn next(&self) -> Result<Exercise, String> {
        if !self.have_next() {
            return Err("Достигнут максимум количества упражнений".to_string());
        }
        self.settings.random_exercise()
    }

    fn recalc_grade(&mut self) {
        self.grade = Grade::from_quantity(self.correct_answers, self.answers.len());
    }

    pub fn add_answer(&mut self, answer: Answer) -> Result<(), String> {
        if !self.have_next() {
            return Err("Достигнут максимум количества ответов".to_string());
        }

        let is_correct = answer.is_correct();
        self.answers.push(answer);

        if is_correct {
            self.correct_answers += 1;
        }
        self.recalc_grade();
        Ok(())
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
        let results_dir = self.results_dir();
        std::fs::create_dir_all(&results_dir)?;
        let saved_at = OffsetDateTime::now_utc();
        let result_path = results_dir.join(Self::filename_datetime(saved_at));

        self.save_json(&result_path)?;
        self.save_txt(&result_path, saved_at)?;
        Ok(())
    }

    fn results_dir(&self) -> PathBuf {
        let translit_name = deunicode::deunicode(&self.settings.player_name);
        Path::new(&self.settings.results_dir).join(translit_name)
    }

    fn save_json(&self, result_path: &Path) -> Result<(), std::io::Error> {
        let file = std::fs::File::create(result_path.with_extension("json"))?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }

    fn save_txt(&self, result_path: &Path, saved_at: OffsetDateTime) -> Result<(), std::io::Error> {
        std::fs::write(
            result_path.with_extension("txt"),
            self.human_results(saved_at),
        )
    }

    fn filename_datetime(date_time: OffsetDateTime) -> String {
        format!(
            "{:04}-{:02}-{:02}_{:02}-{:02}-{:02}",
            date_time.year(),
            u8::from(date_time.month()),
            date_time.day(),
            date_time.hour(),
            date_time.minute(),
            date_time.second()
        )
    }

    fn human_results(&self, saved_at: OffsetDateTime) -> String {
        let mut output = String::new();
        let operations = self.enabled_operations();

        let _ = writeln!(
            output,
            "Имя: {}     дата: {:2}.{:02}.{}  время: {:2}:{:02}:{:02}",
            self.settings.player_name,
            saved_at.day(),
            u8::from(saved_at.month()),
            saved_at.year(),
            saved_at.hour(),
            saved_at.minute(),
            saved_at.second()
        );
        let _ = writeln!(
            output,
            "Действия: {}",
            operations
                .iter()
                .map(|operation| format!(" {}", Self::operation_protocol_name(*operation)))
                .collect::<Vec<_>>()
                .join(",")
        );
        let _ = writeln!(
            output,
            "Количество: {},  пределы: от {} до {},  сложность: {}",
            self.settings.limits.exercise_count,
            self.settings.limits.result_min,
            self.settings.limits.result_max,
            self.settings.limits.answer_time_seconds.as_secs()
        );
        let _ = writeln!(
            output,
            "Верных ответов: {},  оценка: {}",
            self.correct_answers, self.grade as u8
        );

        let _ = write!(output, "Количество действий: ");
        for (index, operation) in operations.iter().enumerate() {
            if index > 0 {
                let _ = write!(output, "                     ");
            }
            let total = self
                .answers
                .iter()
                .filter(|answer| answer.exercise.operation == *operation)
                .count();
            let correct = self
                .answers
                .iter()
                .filter(|answer| answer.exercise.operation == *operation && answer.is_correct())
                .count();
            let _ = writeln!(
                output,
                "{:>19} - {}, верных ответов - {}",
                Self::operation_protocol_name(*operation),
                total,
                correct
            );
        }

        let _ = writeln!(output, "Предложенные действия:");
        for (index, answer) in self.answers.iter().enumerate() {
            let correctness = if answer.is_correct() {
                "  верно"
            } else {
                "неверно"
            };
            let exercise = format!(
                "{}{}{}={}",
                answer.exercise.left,
                answer.exercise.operation.symbol(),
                answer.exercise.right,
                answer.exercise.expected().unwrap_or(0)
            );
            let _ = write!(
                output,
                "{:3}. {}:   {:<18}",
                index + 1,
                correctness,
                exercise
            );
            let _ = write!(output, "{:<21}", answer.protocol_entered());
            let seconds = answer.time_elapsed.as_secs();
            let (verb, noun) = Self::seconds_words(seconds);
            let _ = writeln!(output, "  {} {:3} {}", verb, seconds, noun);
        }

        if let Some((best, worst, average)) = self.time_stats() {
            let _ = writeln!(
                output,
                "                     Время:  лучшее - {}, худшее - {}, среднее - {}",
                best, worst, average
            );
        }

        output
    }

    fn enabled_operations(&self) -> Vec<Operation> {
        PROTOCOL_OPERATION_ORDER
            .iter()
            .copied()
            .filter(|operation| self.settings.operations.contains(operation))
            .collect()
    }

    fn operation_protocol_name(operation: Operation) -> String {
        operation.label().to_lowercase()
    }

    fn seconds_words(seconds: u64) -> (&'static str, &'static str) {
        let remainder = if (11..=14).contains(&(seconds % 100)) {
            0
        } else {
            seconds % 10
        };
        match remainder {
            1 => ("прошла", "секунда"),
            2..=4 => ("прошло", "секунды"),
            _ => ("прошло", "секунд"),
        }
    }

    fn time_stats(&self) -> Option<(u64, u64, u64)> {
        let seconds = self
            .answers
            .iter()
            .filter(|answer| answer.entered.is_ok())
            .map(|answer| answer.time_elapsed.as_secs())
            .collect::<Vec<_>>();
        let count = seconds.len() as u64;
        if count == 0 {
            return None;
        }

        let best = seconds.iter().min().copied().unwrap();
        let worst = seconds.iter().max().copied().unwrap();
        let average = seconds.iter().sum::<u64>() / count;
        Some((best, worst, average))
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
        let session = Session::new(settings(3)).unwrap();
        let exercises = session.exercises().collect::<Result<Vec<_>, _>>().unwrap();

        assert_eq!(exercises.len(), 3);
    }

    #[test]
    fn test_exercises_iter_counts_existing_answers() {
        let mut session = Session::new(settings(3)).unwrap();
        session.add_answer(answer());

        let exercises = session.exercises().collect::<Result<Vec<_>, _>>().unwrap();

        assert_eq!(exercises.len(), 2);
    }

    #[test]
    fn test_session_settings_validation() {
        assert!(Session::new(settings(0)).is_err());
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
            iterator: SessionExerciseIter::new(Settings {
                player_name: "ученик".to_string(),
                results_dir: "results".to_string(),
                operations: HashSet::from([Operation::Addition, Operation::DivisionWithRemainder]),
                limits: crate::domain::settings::Limits {
                    result_min: 10,
                    result_max: 100,
                    exercise_count: 2,
                    answer_time_seconds: std::time::Duration::from_secs(45),
                },
            }),
            answers: vec![answer(), failed_answer(AnswerError::Escaped)],
            correct_answers: 1,
            grade: Grade::Three,
        };

        let serialized = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.iterator.settings.player_name, "ученик");
        assert_eq!(deserialized.iterator.settings.results_dir, "results");
        assert_eq!(
            deserialized.iterator.settings.operations,
            HashSet::from([Operation::Addition, Operation::DivisionWithRemainder])
        );
        assert_eq!(deserialized.iterator.settings.limits.result_min, 10);
        assert_eq!(deserialized.iterator.settings.limits.result_max, 100);
        assert_eq!(deserialized.iterator.settings.limits.exercise_count, 2);
        assert_eq!(
            deserialized.iterator.settings.limits.answer_time_seconds,
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

        assert_eq!(session.iterator.settings.player_name, "test");
        assert_eq!(
            session.iterator.settings.operations,
            HashSet::from([Operation::Addition, Operation::DivisionWithRemainder])
        );
        assert_eq!(
            session.iterator.settings.limits.answer_time_seconds,
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
