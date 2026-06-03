use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::domain::operation::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Limits {
    pub result_min: i32,
    pub result_max: i32,
    pub exercise_count: usize,
    pub answer_time_seconds: u64,
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
        let input = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&input).map_err(|err| err.to_string())
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let output = self.to_toml_string().map_err(|err| err.to_string())?;
        fs::write(path, output).map_err(|err| err.to_string())
    }
}

/*

TOML:

player_name = "неизвестно"
results_dir = "results"

operations = ["+", "/", ":"]

[limits]
result_min = 100
result_max = 150
exercise_count = 20
answer_time_seconds = 30

 */

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
answer_time_seconds = 30
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
        assert!(settings
            .operations
            .contains(&Operation::DivisionWithRemainder));
        assert_eq!(settings.limits.result_min, 100);
        assert_eq!(settings.limits.result_max, 150);
        assert_eq!(settings.limits.exercise_count, 20);
        assert_eq!(settings.limits.answer_time_seconds, 30);
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
answer_time_seconds = 30
"#;

        let error = Settings::from_toml_str(input).unwrap_err().to_string();

        assert!(error.contains("Некорректная операция: ?"));
    }
}
