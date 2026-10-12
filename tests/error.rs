use mindrove::Error;

#[test]
fn unknown_status_becomes_typed_error() {
    assert!(matches!(
        Error::from_status(9999),
        Err(Error::UnknownNativeStatus(9999))
    ));
    assert_eq!(
        Error::from_status(9999).unwrap_err().to_string(),
        "unknown MindRove native status: 9999"
    );
}

#[test]
fn zero_status_is_success() {
    assert!(Error::from_status(0).is_ok());
}

#[test]
fn known_statuses_become_named_errors() {
    let statuses = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
    ];

    for status in statuses {
        let error = Error::from_status(status).unwrap_err();
        assert!(!matches!(error, Error::UnknownNativeStatus(_)));
    }

    assert!(matches!(Error::from_status(7), Err(Error::BoardNotReady)));
    assert_eq!(
        Error::from_status(17).unwrap_err().to_string(),
        "MindRove general error"
    );
}
