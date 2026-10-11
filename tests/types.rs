use mindrove::{
    BoardId, FilterType, IpProtocolType, MindRoveInputParams, MindRoveMetric, MindRoveModelParams,
    WaveletType,
};

#[test]
fn input_defaults_match_sdk() {
    let params = MindRoveInputParams::default();

    assert_eq!(params.ip_port, -1);
    assert_eq!(params.ip_port_aux, 0);
    assert_eq!(params.ip_port_anc, 0);
    assert_eq!(params.ip_protocol, IpProtocolType::NO_IP_PROTOCOL);
    assert_eq!(params.master_board, BoardId::NO_BOARD);
    assert_eq!(params.serial_port, "");
}

#[test]
fn model_defaults_match_sdk() {
    let params = MindRoveModelParams::default();

    assert_eq!(params.metric, MindRoveMetric::MINDFULNESS);
    assert_eq!(params.max_array_size, 8192);
    assert_eq!(params.output_name, "");
}

#[test]
fn input_json_uses_sdk_field_names_and_integer_values() {
    let params = MindRoveInputParams::default();
    let value: serde_json::Value = serde_json::from_str(&params.to_json().unwrap()).unwrap();

    assert!(value.get("ip_address_aux").is_some());
    assert_eq!(value["master_board"], -100);
    assert_eq!(value["ip_protocol"], 0);
}

#[test]
fn model_json_uses_sdk_field_names_and_integer_values() {
    let params = MindRoveModelParams::default();
    let value: serde_json::Value = serde_json::from_str(&params.to_json().unwrap()).unwrap();

    assert_eq!(value["metric"], 0);
    assert_eq!(value["classifier"], 0);
    assert_eq!(value["max_array_size"], 8192);
}

#[test]
fn discriminants_match_headers() {
    assert_eq!(FilterType::BUTTERWORTH as i32, 0);
    assert_eq!(FilterType::BESSEL_ZERO_PHASE as i32, 5);
    assert_eq!(WaveletType::SYM10 as i32, 44);
}
