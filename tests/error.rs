use mindrove::{Error, MindRoveExitCode};

#[test]
fn unknown_status_is_preserved() {
    assert!(matches!(
        Error::from_status(9999),
        Err(Error::Native(code)) if code.as_raw() == 9999
    ));
}

#[test]
fn zero_status_is_success() {
    assert!(Error::from_status(0).is_ok());
}

#[test]
fn known_status_constants_are_lossless() {
    assert_eq!(MindRoveExitCode::STATUS_OK.as_raw(), 0);
    assert_eq!(MindRoveExitCode::GENERAL_ERROR.as_raw(), 17);
}
