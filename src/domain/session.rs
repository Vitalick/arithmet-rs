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
    settings: Settings,
    answers: Vec<Answer>,
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

    pub fn exercises(&self) -> SessionExerciseIter<'_> {
        SessionExerciseIter {
            session: self,
            generated: 0,
        }
    }

    pub fn add_answer(&mut self, answer: Answer) {
        self.answers.push(answer);
    }

    pub fn get_answers(&self) -> &Vec<Answer> {
        &self.answers
    }
    pub fn get_grade(&self) -> Grade {
        self.grade
    }

    pub fn correct_answers(&self) -> usize {
        self.answers.iter().filter(|a| a.is_correct).count()
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
                answer_time_seconds: 30,
            },
        }
    }

    fn answer() -> Answer {
        Answer {
            exercise: Exercise::new(2, Operation::Addition, 3),
            entered: Ok(5),
            time_elapsed: 1,
            is_correct: true,
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
}
