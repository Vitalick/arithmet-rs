use super::Expression;
use super::comparison::Comparison;
use super::fake_exercise::FakeExercise;
use crate::domain::operation::Operation;
use crate::domain::settings::Settings;
use rand::random_range;
use std::fmt::{Display, Formatter};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Exercise {
    pub left: i32,
    pub operation: Operation,
    pub right: i32,
}

impl Expression for Exercise {
    fn evaluate(&self) -> Result<String, String> {
        let result = self.expected_str()?;

        Ok(format!("{} = {}", self, result))
    }
}

impl Display for Exercise {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.left, self.operation, self.right)
    }
}

impl Exercise {
    pub fn new(left: i32, operation: Operation, right: i32) -> Self {
        Exercise {
            left,
            operation,
            right,
        }
    }

    fn check_zero(&self) -> Result<(), String> {
        if self.left == 0 || self.right == 0 || self.expected()? == 0 {
            return Err("Все три значения упражнения должны быть отличны от нуля".to_string());
        }
        Ok(())
    }

    fn unsafe_random(operation: Operation, result_min: i32, result_max: i32) -> Self {
        match operation {
            Operation::Addition => {
                let result = random_range(result_min..result_max);
                let left = random_range(0..result);
                let right = result - left;

                Self::new(left, operation, right)
            }
            Operation::Subtraction => {
                let left = random_range(result_min..result_max);
                let right = random_range(0..left);

                Self::new(left, operation, right)
            }
            Operation::Multiplication => {
                let result = random_range(result_min..result_max);
                let mut left = random_range(0..result);
                if left == 0 {
                    left = 1;
                }

                let mut right = result / left;
                let mut candidate = left;
                let mut offset = 1;
                let mut direction = 1;
                let mut calculated = left * right;

                while calculated != result {
                    if left + offset > result && left - offset < 0 {
                        return Self::unsafe_random(operation, result_min, result_max);
                    }

                    candidate = left + offset * direction;
                    if candidate == 0 {
                        candidate = 1;
                    }
                    right = result / candidate;
                    calculated = candidate * right;

                    if direction == 1 {
                        direction = -1;
                    } else {
                        direction = 1;
                        offset += 1;
                    }
                }

                if candidate == 1 || right == 1 || result == 1 {
                    return Self::unsafe_random(operation, result_min, result_max);
                }

                Self::new(candidate, operation, right)
            }
            Operation::Division | Operation::DivisionWithRemainder => {
                let mut left = random_range(result_min..result_max);
                let mut right = random_range(0..left / 2) + 1;
                if right == 0 {
                    right = 1;
                }

                let mut result = left / right;
                let mut candidate = right;
                let mut offset = 1;
                let mut direction = 1;
                let mut calculated = result * right;

                while calculated != left {
                    if right + offset > left && right - offset < 0 {
                        return Self::unsafe_random(operation, result_min, result_max);
                    }

                    candidate = right + offset * direction;
                    if candidate == 0 {
                        candidate = 1;
                    }
                    result = left / candidate;
                    calculated = result * candidate;

                    if direction == 1 {
                        direction = -1;
                    } else {
                        direction = 1;
                        offset += 1;
                    }
                }

                right = candidate;
                if left == 1 || right == 1 || result == 1 {
                    return Self::unsafe_random(operation, result_min, result_max);
                }

                if matches!(operation, Operation::DivisionWithRemainder) {
                    left += random_range(1..right);
                }

                Self::new(left, operation, right)
            }
        }
    }

    pub fn random(
        operation: Operation,
        result_min: i32,
        result_max: i32,
    ) -> Result<Exercise, String> {
        if result_max - result_min < 50 {
            return Err(
                "Разница межу минимальным и максимальным значением ответа не может быть меньше 50"
                    .to_string(),
            );
        }
        if result_min < 0 {
            return Err("Минимальное значение не может быть меньше нуля".to_string());
        }
        loop {
            let result = Self::unsafe_random(operation, result_min, result_max);
            if result.check_zero().is_ok() {
                return Ok(result);
            }
        }
    }

    pub fn expected(&self) -> Result<i32, String> {
        self.operation.calculate(self.left, self.right)
    }

    pub fn compare(&self, value: i32) -> Result<bool, String> {
        Ok(self.expected()? == value)
    }

    pub fn expected_remainder(&self) -> Result<i32, String> {
        self.operation.calculate_remainder(self.left, self.right)
    }

    pub fn expected_str(&self) -> Result<String, String> {
        self.operation.calculate_str(self.left, self.right)
    }

    fn validate_operands(&self) -> Result<(), String> {
        self.operation.validates_operands(self.left, self.right)
    }

    pub fn check_expressions(&self, entered: i32) -> Result<[Box<dyn Expression>; 2], String> {
        if entered == 0 {
            return Err("Ответ не должен быть нулём".to_string());
        }
        self.validate_operands()?;
        match self.operation {
            Operation::Addition => Ok([
                Box::new(Self::new(entered, Operation::Subtraction, self.left)),
                Box::new(Self::new(entered, Operation::Subtraction, self.right)),
            ]),
            Operation::Subtraction => Ok([
                Box::new(Self::new(self.left, Operation::Subtraction, entered)),
                Box::new(Self::new(self.right, Operation::Addition, entered)),
            ]),
            Operation::Multiplication => Ok([
                Box::new(Self::new(entered, Operation::Division, self.left)),
                Box::new(Self::new(entered, Operation::Division, self.right)),
            ]),
            Operation::Division => Ok([
                Box::new(Self::new(self.left, Operation::Division, entered)),
                Box::new(Self::new(self.right, Operation::Multiplication, entered)),
            ]),
            Operation::DivisionWithRemainder => Ok([
                Box::new(
                    self.make_fake_exercise()?
                        .with_answer(entered)
                        .with_remainder(self.left - self.right * entered),
                ),
                Box::new(Comparison::new(
                    self.left - self.right * entered,
                    self.right,
                )),
            ]),
        }
    }

    fn make_fake_exercise(&self) -> Result<FakeExercise, String> {
        Ok(FakeExercise {
            left: self.left,
            operation: self.operation,
            right: self.right,
            answer: self.expected()?,
            remainder: self.expected_remainder()?,
        })
    }
}

impl Operation {
    pub fn make_exercise(&self, left: i32, right: i32) -> Exercise {
        Exercise::new(left, *self, right)
    }
}

impl Settings {
    fn random_exercise(&self) -> Result<Exercise, String> {
        Exercise::random(
            self.random_operation(),
            self.limits.result_min,
            self.limits.result_max,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ExerciseWithStartTime {
    pub exercise: Exercise,
    pub start_time: Instant,
}

impl ExerciseWithStartTime {
    pub fn new(exercise: Exercise) -> Self {
        Self {
            exercise,
            start_time: Instant::now(),
        }
    }
}

impl Settings {
    pub fn random_exercise_with_time(&self) -> Result<ExerciseWithStartTime, String> {
        Ok(ExerciseWithStartTime::new(self.random_exercise()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::expression::Expression;

    const RANDOM_MIN: i32 = 100;
    const RANDOM_MAX: i32 = 150;
    const RANDOM_SAMPLES: usize = 50;

    fn checked_expressions(exercise: Exercise, entered: i32) -> Result<Vec<String>, String> {
        exercise
            .check_expressions(entered)?
            .into_iter()
            .map(|expression| expression.evaluate())
            .collect()
    }

    fn assert_check_error(exercise: Exercise, entered: i32, expected: &str) {
        match exercise.check_expressions(entered) {
            Ok(_) => panic!("expected error: {expected}"),
            Err(error) => assert_eq!(error, expected),
        }
    }

    fn assert_random_exercise_is_valid(exercise: Exercise, operation: Operation) {
        assert_eq!(exercise.operation, operation);
        assert!(exercise.check_zero().is_ok());
        assert!(exercise.evaluate().is_ok());
    }

    fn random_exercise(operation: Operation) -> Exercise {
        Exercise::random(operation, RANDOM_MIN, RANDOM_MAX).unwrap()
    }

    fn print_check_result(exercise: Exercise, answer_kind: &str, answer: i32) {
        println!("    {answer_kind} answer {answer}:");

        match exercise.check_expressions(answer) {
            Ok(expressions) => {
                for expression in expressions {
                    println!("      {}", expression.evaluate().unwrap());
                }
            }
            Err(error) => println!("      error: {error}"),
        }
    }

    #[test]
    fn test_exercise_calculate_expression_for_each_operation() {
        let cases = [
            (Exercise::new(2, Operation::Addition, 3), "2 + 3 = 5"),
            (Exercise::new(7, Operation::Subtraction, 4), "7 - 4 = 3"),
            (Exercise::new(6, Operation::Multiplication, 5), "6 * 5 = 30"),
            (Exercise::new(20, Operation::Division, 4), "20 / 4 = 5"),
            (
                Exercise::new(10, Operation::DivisionWithRemainder, 3),
                "10 / 3 = 3 (остаток 1)",
            ),
        ];

        for (exercise, expression) in cases {
            assert_eq!(exercise.evaluate().unwrap(), expression);
        }
    }

    #[test]
    fn test_exercise_for_check_addition() {
        assert_eq!(
            checked_expressions(Exercise::new(2, Operation::Addition, 3), 5).unwrap(),
            ["5 - 2 = 3", "5 - 3 = 2"]
        );
    }

    #[test]
    fn test_exercise_for_check_subtraction() {
        assert_eq!(
            checked_expressions(Exercise::new(7, Operation::Subtraction, 4), 3).unwrap(),
            ["7 - 3 = 4", "4 + 3 = 7"]
        );
    }

    #[test]
    fn test_exercise_for_check_multiplication() {
        assert_eq!(
            checked_expressions(Exercise::new(6, Operation::Multiplication, 5), 30).unwrap(),
            ["30 / 6 = 5", "30 / 5 = 6"]
        );
    }

    #[test]
    fn test_exercise_for_check_division() {
        assert_eq!(
            checked_expressions(Exercise::new(20, Operation::Division, 4), 5).unwrap(),
            ["20 / 5 = 4", "4 * 5 = 20"]
        );
    }

    #[test]
    fn test_exercise_for_check_division_with_remainder() {
        assert_eq!(
            checked_expressions(Exercise::new(10, Operation::DivisionWithRemainder, 3), 3).unwrap(),
            ["10 / 3 = 3 (остаток 1)", "1 < 3"]
        );
    }

    #[test]
    fn test_exercise_for_check_division_with_remainder_uses_entered_answer() {
        assert_eq!(
            checked_expressions(Exercise::new(10, Operation::DivisionWithRemainder, 3), 2).unwrap(),
            ["10 / 3 = 2 (остаток 4)", "4 > 3"]
        );
    }
    #[test]
    fn test_exercise_calculate_expression_with_zero_operands() {
        let cases = [
            (Exercise::new(0, Operation::Addition, 5), "0 + 5 = 5"),
            (Exercise::new(5, Operation::Subtraction, 0), "5 - 0 = 5"),
            (Exercise::new(0, Operation::Multiplication, 5), "0 * 5 = 0"),
            (Exercise::new(0, Operation::Division, 5), "0 / 5 = 0"),
            (
                Exercise::new(0, Operation::DivisionWithRemainder, 5),
                "0 / 5 = 0",
            ),
        ];

        for (exercise, expression) in cases {
            assert_eq!(exercise.evaluate().unwrap(), expression);
        }
    }

    #[test]
    fn test_exercise_rejects_division_by_zero() {
        let division = Exercise::new(5, Operation::Division, 0);
        let division_with_remainder = Exercise::new(5, Operation::DivisionWithRemainder, 0);

        assert_eq!(division.evaluate().unwrap_err(), "Деление на ноль");
        assert_eq!(
            division_with_remainder.evaluate().unwrap_err(),
            "Деление на ноль"
        );
    }

    #[test]
    fn test_exercise_for_check_rejects_zero_answer_for_each_operation() {
        let cases = [
            Exercise::new(2, Operation::Addition, 3),
            Exercise::new(7, Operation::Subtraction, 4),
            Exercise::new(6, Operation::Multiplication, 5),
            Exercise::new(20, Operation::Division, 4),
            Exercise::new(10, Operation::DivisionWithRemainder, 3),
        ];

        for exercise in cases {
            assert_check_error(exercise, 0, "Ответ не должен быть нулём");
        }
    }

    #[test]
    fn test_exercise_for_check_rejects_source_division_by_zero() {
        let cases = [
            Exercise::new(5, Operation::Division, 0),
            Exercise::new(5, Operation::DivisionWithRemainder, 0),
        ];

        for exercise in cases {
            assert_check_error(exercise, 1, "Деление на ноль");
        }
    }

    #[test]
    fn test_random_generates_valid_exercise_for_each_operation() {
        let operations = [
            Operation::Addition,
            Operation::Subtraction,
            Operation::Multiplication,
            Operation::Division,
            Operation::DivisionWithRemainder,
        ];

        for operation in operations {
            for _ in 0..RANDOM_SAMPLES {
                let exercise = random_exercise(operation);

                assert_random_exercise_is_valid(exercise, operation);
            }
        }
    }

    #[test]
    fn test_random_addition_matches_c_generation_rules() {
        for _ in 0..RANDOM_SAMPLES {
            let exercise = random_exercise(Operation::Addition);
            let expected = exercise.expected().unwrap();

            assert_random_exercise_is_valid(exercise, Operation::Addition);
            assert!((RANDOM_MIN..RANDOM_MAX).contains(&expected));
            assert_eq!(exercise.left + exercise.right, expected);
        }
    }

    #[test]
    fn test_random_subtraction_matches_c_generation_rules() {
        for _ in 0..RANDOM_SAMPLES {
            let exercise = random_exercise(Operation::Subtraction);
            let expected = exercise.expected().unwrap();

            assert_random_exercise_is_valid(exercise, Operation::Subtraction);
            assert!((RANDOM_MIN..RANDOM_MAX).contains(&exercise.left));
            assert!(exercise.right < exercise.left);
            assert_eq!(exercise.left - exercise.right, expected);
        }
    }

    #[test]
    fn test_random_multiplication_matches_c_generation_rules() {
        for _ in 0..RANDOM_SAMPLES {
            let exercise = random_exercise(Operation::Multiplication);
            let expected = exercise.expected().unwrap();

            assert_random_exercise_is_valid(exercise, Operation::Multiplication);
            assert!((RANDOM_MIN..RANDOM_MAX).contains(&expected));
            assert!(exercise.left > 1);
            assert!(exercise.right > 1);
            assert!(expected > 1);
            assert_eq!(exercise.left * exercise.right, expected);
        }
    }

    #[test]
    fn test_random_division_matches_c_generation_rules() {
        for _ in 0..RANDOM_SAMPLES {
            let exercise = random_exercise(Operation::Division);
            let expected = exercise.expected().unwrap();

            assert_random_exercise_is_valid(exercise, Operation::Division);
            assert!((RANDOM_MIN..RANDOM_MAX).contains(&exercise.left));
            assert!(exercise.left > 1);
            assert!(exercise.right > 1);
            assert!(expected > 1);
            assert_eq!(exercise.left % exercise.right, 0);
        }
    }

    #[test]
    fn test_random_division_with_remainder_matches_c_generation_rules() {
        for _ in 0..RANDOM_SAMPLES {
            let exercise = random_exercise(Operation::DivisionWithRemainder);
            let expected = exercise.expected().unwrap();
            let remainder = exercise.left % exercise.right;

            assert_random_exercise_is_valid(exercise, Operation::DivisionWithRemainder);
            assert!(exercise.left > 1);
            assert!(exercise.right > 1);
            assert!(expected > 1);
            assert!(remainder > 0);
            assert!(remainder < exercise.right);
        }
    }

    #[test]
    fn test_random_rejects_invalid_ranges() {
        assert_eq!(
            Exercise::random(Operation::Addition, 100, 100).unwrap_err(),
            "Разница межу минимальным и максимальным значением ответа не может быть меньше 50",
        );
        assert_eq!(
            Exercise::random(Operation::Addition, 150, 100).unwrap_err(),
            "Разница межу минимальным и максимальным значением ответа не может быть меньше 50",
        );
        assert_eq!(
            Exercise::random(Operation::Addition, 100, 120).unwrap_err(),
            "Разница межу минимальным и максимальным значением ответа не может быть меньше 50",
        );
        assert_eq!(
            Exercise::random(Operation::Addition, -1, 100).unwrap_err(),
            "Минимальное значение не может быть меньше нуля",
        );
    }

    #[test]
    #[ignore = "prints generated random examples for manual inspection"]
    fn test_random_prints_generated_examples() {
        let operations = [
            Operation::Addition,
            Operation::Subtraction,
            Operation::Multiplication,
            Operation::Division,
            Operation::DivisionWithRemainder,
        ];

        for operation in operations {
            println!("{operation:?}:");
            for _ in 0..5 {
                let exercise = random_exercise(operation);
                let correct_answer = exercise.expected().unwrap();
                let wrong_answer = if matches!(operation, Operation::DivisionWithRemainder) {
                    correct_answer - 1
                } else {
                    correct_answer + 1
                };

                println!("  {}", exercise.evaluate().unwrap());
                print_check_result(exercise, "correct", correct_answer);
                print_check_result(exercise, "wrong", wrong_answer);
                print_check_result(exercise, "zero", 0);
            }
        }
    }
}
