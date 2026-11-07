use std::fs;

use skillport_lib::core::logging;

#[test]
fn writes_structured_lines() {
    let temp = tempfile::tempdir().expect("temp");
    let dir = temp.path().join("logs");
    logging::init(&dir).expect("init");
    logging::write(&dir, "info", "startup", Some("data_dir=/tmp/x"));
    let text = fs::read_to_string(logging::log_path(&dir)).expect("log");
    assert!(text.contains("level=info"));
    assert!(text.contains("event=startup"));
    assert!(text.contains("data_dir=/tmp/x"));
}

#[test]
fn rotates_when_file_exceeds_limit() {
    let temp = tempfile::tempdir().expect("temp");
    let dir = temp.path().join("logs");
    fs::create_dir_all(&dir).expect("dir");
    fs::write(
        logging::log_path(&dir),
        vec![b'x'; logging::MAX_LOG_BYTES as usize + 1],
    )
    .expect("big log");
    logging::init(&dir).expect("init");
    assert!(!logging::log_path(&dir).exists());
    assert!(dir.join("skillport.log.1").is_file());
}
