use toml::map::Map;
use crate::domain::operation::Operation;

#[derive(Debug, Clone)]
pub struct Limits {
    result_min: i32,
    result_max: i32,
    exercise_count: usize,
    answer_time_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct SettingsSnapshot {
    pub player_name: String,
    pub results_dir: String,
    pub operations: Map<Operation, bool>,
    pub limits: Limits
}

/*

TOML:

player_name = "неизвестно"
results_dir = "results"

[operations]
addition = true
subtraction = false
multiplication = false
division = false
division_with_remainder = true

[limits]
result_min = 100
result_max = 150
exercise_count = 20
answer_time_seconds = 30

 */