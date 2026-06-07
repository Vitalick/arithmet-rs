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
            answer_time: Duration::from_secs(30),
        },
    };
    let mut session = Session::new(settings).unwrap();

    while session.have_next() {
        let exercise = session.next().unwrap();
        let expected = exercise.exercise.expected().unwrap();
        session.add_answer(Answer {
            exercise: exercise.exercise,
            entered: Ok(expected),
            time_elapsed: Duration::from_secs(1),
        })
        .unwrap();
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
    assert_eq!(result_files.len(), 2);

    let json_path = result_files
        .iter()
        .map(|entry| entry.path())
        .find(|path| path.extension().unwrap() == "json")
        .unwrap();
    let txt_path = result_files
        .iter()
        .map(|entry| entry.path())
        .find(|path| path.extension().unwrap() == "txt")
        .unwrap();
    assert_eq!(
        json_path.file_stem().unwrap(),
        txt_path.file_stem().unwrap()
    );

    let output = fs::read_to_string(json_path).unwrap();
    let saved: Value = serde_json::from_str(&output).unwrap();

    assert_eq!(saved["settings"]["player_name"], "integration_player");
    assert_eq!(saved["settings"]["operations"], serde_json::json!(["+"]));
    assert_eq!(saved["settings"]["limits"]["exercise_count"], 3);
    assert_eq!(saved["answers"].as_array().unwrap().len(), 3);
    assert_eq!(saved["correct_answers"], 3);
    assert_eq!(saved["grade"], "Five");

    let human_output = fs::read_to_string(txt_path).unwrap();
    assert!(human_output.contains("Имя: integration_player"));
    assert!(human_output.contains("Действия:  сложение"));
    assert!(human_output.contains("Верных ответов: 3,  оценка: 5"));
    assert!(
        human_output.contains("Количество действий:            сложение - 3, верных ответов - 3")
    );
    assert!(human_output.contains("Предложенные действия:"));
    assert!(human_output.contains("прошла   1 секунда"));
    assert!(human_output.contains("Время:  лучшее - 1, худшее - 1, среднее - 1"));
}
