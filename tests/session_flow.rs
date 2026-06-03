use std::collections::HashSet;
use std::fs;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use arithmet::domain::operation::Operation;
use arithmet::domain::session::{Answer, Session};
use arithmet::domain::settings::{Limits, Settings};
use serde_json::Value;

fn unique_results_dir() -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    std::env::temp_dir().join(format!(
        "arithmet-session-flow-{}-{}",
        std::process::id(),
        suffix
    ))
}

struct Cleanup(std::path::PathBuf);

impl Drop for Cleanup {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn session_generates_answers_and_writes_result() {
    let results_dir = unique_results_dir();
    let _cleanup = Cleanup(results_dir.clone());
    let settings = Settings {
        player_name: "integration_player".to_string(),
        results_dir: results_dir.to_string_lossy().into_owned(),
        operations: HashSet::from([Operation::Addition]),
        limits: Limits {
            result_min: 100,
            result_max: 150,
            exercise_count: 3,
            answer_time_seconds: Duration::from_secs(30),
        },
    };
    let mut session = Session::new(settings);

    let exercises = session.exercises().collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(exercises.len(), 3);

    for exercise in exercises {
        session.add_answer(Answer {
            exercise,
            entered: Ok(exercise.expected().unwrap()),
            time_elapsed: Duration::from_secs(1),
        });
    }

    assert!(session.is_finished());
    assert_eq!(session.total_answers(), 3);
    assert_eq!(session.get_answers().len(), 3);

    session.save().unwrap();

    let player_dir = results_dir.join("integration_player");
    let result_files = fs::read_dir(&player_dir)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(result_files.len(), 1);

    let result_path = result_files[0].path();
    let output = fs::read_to_string(result_path).unwrap();
    let saved: Value = serde_json::from_str(&output).unwrap();

    assert_eq!(saved["settings"]["player_name"], "integration_player");
    assert_eq!(saved["settings"]["operations"], serde_json::json!(["+"]));
    assert_eq!(saved["settings"]["limits"]["exercise_count"], 3);
    assert_eq!(saved["answers"].as_array().unwrap().len(), 3);
    assert_eq!(saved["correct_answers"], 3);
    assert_eq!(saved["grade"], "Five");
}
