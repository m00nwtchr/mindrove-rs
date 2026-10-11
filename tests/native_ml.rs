#![cfg(all(feature = "native-tests", target_os = "linux"))]

use std::path::PathBuf;

use mindrove::{MLModel, MindRoveModelParams};

#[test]
fn model_lifecycle_uses_optional_asset() {
    let Some(model_path) = std::env::var_os("MINDROVE_MODEL_PATH") else {
        println!("skipping native ML lifecycle test: MINDROVE_MODEL_PATH is not set");
        return;
    };

    let params = MindRoveModelParams {
        file: PathBuf::from(model_path).to_string_lossy().into_owned(),
        ..MindRoveModelParams::default()
    };
    let mut model = MLModel::new(params).unwrap();

    model.prepare().unwrap();
    let output = model.predict(&[0.0]).unwrap();
    assert!(!output.is_empty());
    model.release().unwrap();
}
