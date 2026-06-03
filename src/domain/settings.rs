use crate::domain::operation::Operation;
use std::collections::HashSet;
use std::path::Path;
use validations::{Error, Errors, Validate};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Limits {
    pub result_min: i32,
    pub result_max: i32,
    pub exercise_count: usize,
    #[serde(with = "humantime_serde")]
    pub answer_time_seconds: std::time::Duration,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            result_min: 100,
            result_max: 150,
            exercise_count: 20,
            answer_time_seconds: std::time::Duration::from_secs(30),
        }
    }
}

impl Validate<String> for Limits {
    fn validate(&self) -> Result<(), Errors<String>> {
        let mut errors = Errors::new();

        if self.result_max - self.result_min < 50 {
            errors.add_error(Error::new(
                "Разница межу минимальным и максимальным значением ответа не может быть меньше 50",
            ));
        }
        if self.result_min < 0 {
            errors.add_error(Error::new("Минимальное значение не может быть меньше нуля"));
        }

        if self.exercise_count < 1 {
            errors.add_error(Error::new("Количество упражнений должно быть больше 0"));
        }
        if self.answer_time_seconds.as_secs() < 1 {
            errors.add_error(Error::new("Время на ответ должно быть как минимум 1 секунда"));
        }

        if errors.is_empty() {
            return Ok(());
        }
        Err(errors)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    pub player_name: String,
    pub results_dir: String,
    pub operations: HashSet<Operation>,
    pub limits: Limits,
}

impl Settings {
    pub fn from_toml_str(input: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(input)
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let input = std::fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&input).map_err(|err| err.to_string())
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let output = self.to_toml_string().map_err(|err| err.to_string())?;
        std::fs::write(path, output).map_err(|err| err.to_string())
    }
}

impl Validate<String> for Settings {
    fn validate(&self) -> Result<(), Errors<String>> {
        let mut errors = Errors::new();
        if self.player_name == "" {
            errors.add_error(Error::new("имя игрока должно быть заполнено"))
        }
        if self.results_dir == "" {
            errors.add_error(Error::new("путь к результатам должен быть заполнен"))
        }

        if self.operations.is_empty() {
            errors.add_error(Error::new("Должна быть установлена хотя бы одна операция"))
        }

        if errors.is_empty() {
            return Ok(());
        }
        Err(errors)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            player_name: "неизвестно".to_string(),
            results_dir: "results".to_string(),
            operations: HashSet::from([Operation::Addition, Operation::DivisionWithRemainder]),
            limits: Limits::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: &str = r#"
player_name = "неизвестно"
results_dir = "results"
operations = ["+", "-", "*", "/", ":"]

[limits]
result_min = 100
result_max = 150
exercise_count = 20
answer_time_seconds = "30s"
"#;

    #[test]
    fn test_settings_from_toml_str() {
        let settings = Settings::from_toml_str(CONFIG).unwrap();

        assert_eq!(settings.player_name, "неизвестно");
        assert_eq!(settings.results_dir, "results");
        assert!(settings.operations.contains(&Operation::Addition));
        assert!(settings.operations.contains(&Operation::Subtraction));
        assert!(settings.operations.contains(&Operation::Multiplication));
        assert!(settings.operations.contains(&Operation::Division));
        assert!(
            settings
                .operations
                .contains(&Operation::DivisionWithRemainder)
        );
        assert_eq!(settings.limits.result_min, 100);
        assert_eq!(settings.limits.result_max, 150);
        assert_eq!(settings.limits.exercise_count, 20);
        assert_eq!(
            settings.limits.answer_time_seconds,
            std::time::Duration::from_secs(30)
        );
    }

    #[test]
    fn test_settings_to_toml_string_round_trip() {
        let settings = Settings::from_toml_str(CONFIG).unwrap();
        let output = settings.to_toml_string().unwrap();
        let parsed_again = Settings::from_toml_str(&output).unwrap();

        assert_eq!(settings.player_name, parsed_again.player_name);
        assert_eq!(settings.results_dir, parsed_again.results_dir);
        assert_eq!(settings.operations, parsed_again.operations);
        assert_eq!(settings.limits.result_min, parsed_again.limits.result_min);
        assert_eq!(settings.limits.result_max, parsed_again.limits.result_max);
        assert_eq!(
            settings.limits.exercise_count,
            parsed_again.limits.exercise_count
        );
        assert_eq!(
            settings.limits.answer_time_seconds,
            parsed_again.limits.answer_time_seconds
        );
    }

    #[test]
    fn test_settings_rejects_unknown_operation() {
        let input = r#"
player_name = "неизвестно"
results_dir = "results"
operations = ["?"]

[limits]
result_min = 100
result_max = 150
exercise_count = 20
answer_time_seconds = "30s"
"#;

        let error = Settings::from_toml_str(input).unwrap_err().to_string();

        assert!(error.contains("Некорректная операция: ?"));
    }
}
