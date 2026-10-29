use std::fs;

#[test]
fn approved_logging_methods_are_not_public() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let files_and_methods = [
        (
            "src/board_shim.rs",
            [
                "disable_board_logger",
                "enable_board_logger",
                "enable_dev_board_logger",
                "set_log_file",
                "set_log_level",
                "log_message",
            ],
        ),
        (
            "src/data_filter.rs",
            [
                "set_log_level",
                "enable_data_logger",
                "disable_data_logger",
                "enable_dev_data_logger",
                "set_log_file",
                "log_message",
            ],
        ),
        (
            "src/ml_model.rs",
            [
                "disable_ml_logger",
                "enable_ml_logger",
                "enable_dev_ml_logger",
                "set_log_level",
                "set_log_file",
                "log_message",
            ],
        ),
    ];

    for (relative_path, methods) in files_and_methods {
        let path = format!("{manifest_dir}/{relative_path}");
        let source = fs::read_to_string(&path).unwrap();

        for method in methods {
            assert!(
                !source.contains(&format!("pub fn {method}(")),
                "{method} must not remain public in {relative_path}"
            );
        }
    }
}
