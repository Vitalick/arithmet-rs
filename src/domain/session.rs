use crate::domain::answer::{Answer, AnswerError};
use crate::domain::expression::ExerciseWithStartTime;
use crate::domain::grade::Grade;
use crate::domain::settings::Settings;
use std::cmp::min;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use time::OffsetDateTime;
use validations::Validate;

pub enum StepResult {
    TimedOut,
    Nothing,
    ExerciseCreated,
    Finished,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Session {
    pub settings: Settings,
    answers: Vec<Answer>,
    pub correct_answers: usize,
    grade: Grade,
    #[serde(skip)]
    pub exercise_now: Option<ExerciseWithStartTime>,
    #[serde(skip)]
    pub last_answer: Option<Answer>,
    interrupted: bool,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            answers: Vec::new(),
            correct_answers: 0,
            grade: Grade::default(),
            exercise_now: None,
            last_answer: None,
            interrupted: false,
        }
    }
}

impl Session {
    pub fn new(settings: Settings) -> Result<Self, String> {
        match settings.validate() {
            Ok(_) => Ok(Session {
                settings,
                answers: Vec::new(),
                correct_answers: 0,
                grade: Grade::default(),
                exercise_now: None,
                last_answer: None,
                interrupted: false,
            }),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn have_next(&self) -> bool {
        self.answers.len() < self.settings.limits.exercise_count
    }

    pub fn next(&self) -> Result<ExerciseWithStartTime, String> {
        if !self.have_next() {
            return Err("Достигнут максимум количества упражнений".to_string());
        }
        self.settings.random_exercise_with_time()
    }

    fn recalc_grade(&mut self) {
        self.grade = Grade::from_quantity(self.correct_answers, self.answers.len());
    }

    pub fn answer(&mut self, entered: Result<i64, AnswerError>) {
        let exercise_now = self.exercise_now.unwrap();
        let elapsed = exercise_now.start_time.elapsed();

        let entered = if matches!(entered, Ok(_)) && elapsed > self.settings.limits.answer_time {
            Err(AnswerError::TimedOut)
        } else {
            entered
        };
        let answer = Answer {
            exercise: exercise_now.exercise,
            entered,
            time_elapsed: min(self.settings.limits.answer_time, elapsed),
        };
        self.add_answer(answer).unwrap();
        self.last_answer = Some(answer);
    }

    fn add_answer(&mut self, answer: Answer) -> Result<(), String> {
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

    pub fn prepare_next_exercise(&mut self) -> Result<(), String> {
        self.exercise_now = None;
        Ok(())
    }

    pub fn game_step(&mut self) -> Result<StepResult, String> {
        if self.is_finished() {
            return Ok(StepResult::Finished);
        }
        match self.exercise_now {
            Some(exercise_now) => {
                if exercise_now.start_time.elapsed() > self.settings.limits.answer_time {
                    self.answer(Err(AnswerError::TimedOut));
                    return Ok(StepResult::TimedOut);
                }
            }
            None => {
                if self.have_next() {
                    self.exercise_now = Some(self.next()?);
                    self.last_answer = None;
                    return Ok(StepResult::ExerciseCreated);
                }
            }
        }
        Ok(StepResult::Nothing)
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

    pub fn last_answer_banner(&self) -> String {
        if self.answers.is_empty() {
            return String::default();
        }
        let answer = self.answers.last();
        if answer.is_none() {
            return String::default();
        }
        answer.unwrap().banner()
    }

    pub fn exercises_left(&self) -> usize {
        self.settings.limits.exercise_count - self.answers.len()
    }

    pub fn is_finished(&self) -> bool {
        self.interrupted || self.exercises_left() == 0
    }

    pub fn interrupt(&mut self) {
        self.interrupted = true;
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
        let operations = self.settings.enabled_operations();

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
                .map(|operation| format!(" {}", operation.label().to_lowercase()))
                .collect::<Vec<_>>()
                .join(",")
        );
        let _ = writeln!(
            output,
            "Количество: {},  пределы: от {} до {},  сложность: {}",
            self.settings.limits.exercise_count,
            self.settings.limits.result_min,
            self.settings.limits.result_max,
            self.settings.limits.answer_time.as_secs()
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
                operation.label().to_lowercase(),
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
    use crate::domain::answer::AnswerError;
    use crate::domain::expression::Exercise;
    use crate::domain::operation::Operation;
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
                answer_time: std::time::Duration::from_secs(30),
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
    fn test_next_generates_exercise_while_session_has_room() {
        let session = Session::new(settings(3)).unwrap();
        let exercise = session.next().unwrap();

        assert_eq!(exercise.exercise.operation, Operation::Addition);
    }

    #[test]
    fn test_have_next_counts_existing_answers() {
        let mut session = Session::new(settings(3)).unwrap();
        session.add_answer(answer()).unwrap();

        assert!(session.have_next());
        assert_eq!(session.exercises_left(), 2);
    }

    #[test]
    fn test_next_rejects_finished_session() {
        let mut session = Session::new(settings(1)).unwrap();
        session.add_answer(answer()).unwrap();

        assert!(!session.have_next());
        assert!(session.next().is_err());
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
            settings: Settings {
                player_name: "ученик".to_string(),
                results_dir: "results".to_string(),
                operations: HashSet::from([Operation::Addition, Operation::DivisionWithRemainder]),
                limits: crate::domain::settings::Limits {
                    result_min: 10,
                    result_max: 100,
                    exercise_count: 2,
                    answer_time: std::time::Duration::from_secs(45),
                },
            },
            answers: vec![answer(), failed_answer(AnswerError::Escaped)],
            correct_answers: 1,
            grade: Grade::Three,
            last_answer: None,
            exercise_now: None,
            interrupted: false,
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
            deserialized.settings.limits.answer_time,
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
      "answer_time": "30s"
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
            session.settings.limits.answer_time,
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
