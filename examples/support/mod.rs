#![allow(dead_code)] // Cargo compiles this shared support module separately into each example, so parser entry points unused by that binary are intentional.

use mindrove::{
    BoardId, Error, IpProtocolType, MindRoveClassifier, MindRoveInputParams, MindRoveMetric,
    MindRoveModelParams, Result,
};

pub fn parse_board_args<I, S>(args: I) -> Result<(BoardId, MindRoveInputParams)>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args = collect_args(args);
    let (board_id, params, _) = parse_input_args(&args, true)?;
    Ok((
        board_id.ok_or_else(|| invalid("missing required --board-id"))?,
        params,
    ))
}

pub fn parse_model_args<I, S>(
    args: I,
) -> Result<(BoardId, MindRoveInputParams, MindRoveModelParams)>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args = collect_args(args);
    let mut input_args = Vec::new();
    let mut model = MindRoveModelParams::default();
    let mut classifier = false;
    let mut metric = false;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--classifier" => {
                model.classifier = parse_classifier(value(&args, &mut index, "--classifier")?)?;
                classifier = true;
            }
            "--metric" => {
                model.metric = parse_metric(value(&args, &mut index, "--metric")?)?;
                metric = true;
            }
            "--model-file" => {
                model.file = value(&args, &mut index, "--model-file")?;
            }
            "--output-name" => {
                model.output_name = value(&args, &mut index, "--output-name")?;
            }
            _ => input_args.push(args[index].clone()),
        }
        index += 1;
    }

    let (board_id, params, _) = parse_input_args(&input_args, true)?;
    let board_id = board_id.ok_or_else(|| invalid("missing required --board-id"))?;
    if !classifier {
        return Err(invalid("missing required --classifier"));
    }
    if !metric {
        return Err(invalid("missing required --metric"));
    }
    Ok((board_id, params, model))
}

pub fn parse_syncbox_args<I, S>(args: I) -> Result<Vec<MindRoveInputParams>>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args = collect_args(args);
    let mut mac_addresses = Vec::new();
    let mut ip_ports = Vec::new();
    let mut timeouts = Vec::new();
    let mut common = Vec::new();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--mac-address" => {
                mac_addresses.push(value(&args, &mut index, "--mac-address")?);
            }
            "--ip-port" => {
                ip_ports.push(parse_i32(
                    value(&args, &mut index, "--ip-port")?,
                    "--ip-port",
                )?);
            }
            "--timeout" => {
                timeouts.push(parse_i32(
                    value(&args, &mut index, "--timeout")?,
                    "--timeout",
                )?);
            }
            _ => common.push(args[index].clone()),
        }
        index += 1;
    }

    if mac_addresses.is_empty() && ip_ports.is_empty() {
        return Err(invalid(
            "at least one of --mac-address or --ip-port is required",
        ));
    }

    let (board_id, defaults, _) = parse_input_args(&common, false)?;
    let count = mac_addresses.len().max(ip_ports.len()).max(timeouts.len());
    let mut entries = Vec::with_capacity(count);
    for entry in 0..count {
        let mut params = defaults.clone();
        if let Some(mac) = mac_addresses.get(entry) {
            params.mac_address = mac.clone();
        }
        if let Some(port) = ip_ports.get(entry) {
            params.ip_port = *port;
        }
        if let Some(timeout) = timeouts.get(entry) {
            params.timeout = *timeout;
        }
        if let Some(board_id) = board_id {
            params.master_board = board_id;
        }
        entries.push(params);
    }
    Ok(entries)
}

fn collect_args<I, S>(args: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args: Vec<String> = args.into_iter().map(Into::into).collect();
    if args.first().is_some_and(|arg| !arg.starts_with('-')) {
        args.into_iter().skip(1).collect()
    } else {
        args
    }
}

fn parse_input_args(
    args: &[String],
    require_board: bool,
) -> Result<(Option<BoardId>, MindRoveInputParams, usize)> {
    let mut board_id = None;
    let mut params = MindRoveInputParams::default();
    let mut index = 0;
    while index < args.len() {
        let flag = args[index].as_str();
        match flag {
            "--board-id" => {
                board_id = Some(BoardId::from_raw(parse_i32(
                    value(args, &mut index, flag)?,
                    flag,
                )?))
            }
            "--ip-address" => params.ip_address = value(args, &mut index, flag)?,
            "--ip-address-aux" => params.ip_address_aux = value(args, &mut index, flag)?,
            "--ip-address-anc" => params.ip_address_anc = value(args, &mut index, flag)?,
            "--ip-port" => params.ip_port = parse_i32(value(args, &mut index, flag)?, flag)?,
            "--ip-port-aux" => {
                params.ip_port_aux = parse_i32(value(args, &mut index, flag)?, flag)?
            }
            "--ip-port-anc" => {
                params.ip_port_anc = parse_i32(value(args, &mut index, flag)?, flag)?
            }
            "--serial-port" => params.serial_port = value(args, &mut index, flag)?,
            "--ip-protocol" => {
                params.ip_protocol = parse_protocol(value(args, &mut index, flag)?, flag)?
            }
            "--timeout" => params.timeout = parse_i32(value(args, &mut index, flag)?, flag)?,
            "--other-info" => params.other_info = value(args, &mut index, flag)?,
            "--mac-address" => params.mac_address = value(args, &mut index, flag)?,
            "--serial-number" => params.serial_number = value(args, &mut index, flag)?,
            "--file" => params.file = value(args, &mut index, flag)?,
            "--file-aux" => params.file_aux = value(args, &mut index, flag)?,
            "--file-anc" => params.file_anc = value(args, &mut index, flag)?,
            "--master-board" => {
                params.master_board =
                    BoardId::from_raw(parse_i32(value(args, &mut index, flag)?, flag)?)
            }
            _ => return Err(invalid(format!("unknown argument {flag}"))),
        }
        index += 1;
    }
    if require_board && board_id.is_none() {
        return Err(invalid("missing required --board-id"));
    }
    Ok((board_id, params, index))
}

fn value(args: &[String], index: &mut usize, flag: &str) -> Result<String> {
    *index += 1;
    args.get(*index)
        .filter(|value| !value.starts_with("--"))
        .cloned()
        .ok_or_else(|| invalid(format!("missing value for {flag}")))
}

fn parse_i32(value: String, flag: &str) -> Result<i32> {
    value
        .parse()
        .map_err(|_| invalid(format!("invalid value for {flag}: {value}")))
}

fn parse_protocol(value: String, flag: &str) -> Result<IpProtocolType> {
    match parse_i32(value, flag)? {
        0 => Ok(IpProtocolType::NO_IP_PROTOCOL),
        1 => Ok(IpProtocolType::UDP),
        2 => Ok(IpProtocolType::TCP),
        _ => Err(invalid(format!("invalid value for {flag}"))),
    }
}

fn parse_classifier(value: String) -> Result<MindRoveClassifier> {
    match parse_i32(value, "--classifier")? {
        0 => Ok(MindRoveClassifier::DEFAULT_CLASSIFIER),
        1 => Ok(MindRoveClassifier::DYN_LIB_CLASSIFIER),
        2 => Ok(MindRoveClassifier::ONNX_CLASSIFIER),
        _ => Err(invalid("invalid value for --classifier")),
    }
}

fn parse_metric(value: String) -> Result<MindRoveMetric> {
    match parse_i32(value, "--metric")? {
        0 => Ok(MindRoveMetric::MINDFULNESS),
        1 => Ok(MindRoveMetric::RESTFULNESS),
        2 => Ok(MindRoveMetric::USER_DEFINED),
        _ => Err(invalid("invalid value for --metric")),
    }
}

fn invalid(message: impl Into<String>) -> Error {
    Error::InvalidArgument(message.into())
}

#[cfg(test)]
mod tests {
    use super::{parse_board_args, parse_model_args, parse_syncbox_args};
    use mindrove::{
        BoardId, IpProtocolType, MindRoveClassifier, MindRoveInputParams, MindRoveMetric,
    };

    #[test]
    fn parses_board_flags_and_preserves_defaults() {
        let args = [
            "get_data",
            "--board-id",
            "42",
            "--ip-address",
            "192.0.2.1",
            "--ip-port-aux",
            "6670",
            "--ip-protocol",
            "2",
            "--master-board",
            "7",
        ];

        let (board_id, params) = parse_board_args(args.into_iter()).unwrap();

        assert_eq!(board_id, BoardId::from_raw(42));
        assert_eq!(params.ip_address, "192.0.2.1");
        assert_eq!(params.ip_port_aux, 6670);
        assert_eq!(params.ip_protocol, IpProtocolType::TCP);
        assert_eq!(params.master_board, BoardId::from_raw(7));
        assert_eq!(
            params.serial_port,
            MindRoveInputParams::default().serial_port
        );
        assert_eq!(params.timeout, MindRoveInputParams::default().timeout);
    }

    #[test]
    fn rejects_missing_flag_value() {
        let error = parse_board_args(["get_data", "--board-id"].into_iter()).unwrap_err();

        assert!(matches!(error, mindrove::Error::InvalidArgument(_)));
    }

    #[test]
    fn model_parser_requires_board_classifier_and_metric() {
        let args = [
            "eeg_metrics_ci",
            "--board-id",
            "1",
            "--classifier",
            "2",
            "--metric",
            "1",
            "--model-file",
            "model.bin",
            "--output-name",
            "score",
        ];

        let (board_id, input, model) = parse_model_args(args.into_iter()).unwrap();

        assert_eq!(board_id, BoardId::from_raw(1));
        assert_eq!(input, MindRoveInputParams::default());
        assert_eq!(model.classifier, MindRoveClassifier::ONNX_CLASSIFIER);
        assert_eq!(model.metric, MindRoveMetric::RESTFULNESS);
        assert_eq!(model.file, "model.bin");
        assert_eq!(model.output_name, "score");
    }

    #[test]
    fn model_parser_rejects_missing_required_flags() {
        for args in [
            vec!["eeg_metrics_ci", "--classifier", "0", "--metric", "0"],
            vec!["eeg_metrics_ci", "--board-id", "1", "--metric", "0"],
            vec!["eeg_metrics_ci", "--board-id", "1", "--classifier", "0"],
        ] {
            assert!(matches!(
                parse_model_args(args.into_iter()),
                Err(mindrove::Error::InvalidArgument(_))
            ));
        }
    }

    #[test]
    fn parses_repeated_syncbox_arguments_into_entries() {
        let args = [
            "syncbox_get_data",
            "--mac-address",
            "AA:BB",
            "--ip-port",
            "6670",
            "--timeout",
            "1000",
            "--mac-address",
            "CC:DD",
            "--ip-port",
            "6671",
            "--timeout",
            "2000",
        ];

        let entries = parse_syncbox_args(args.into_iter()).unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].mac_address, "AA:BB");
        assert_eq!(entries[0].ip_port, 6670);
        assert_eq!(entries[0].timeout, 1000);
        assert_eq!(entries[1].mac_address, "CC:DD");
        assert_eq!(entries[1].ip_port, 6671);
        assert_eq!(entries[1].timeout, 2000);
    }

    #[test]
    fn syncbox_parser_requires_mac_or_ip_port() {
        assert!(matches!(
            parse_syncbox_args(["syncbox_get_data"].into_iter()),
            Err(mindrove::Error::InvalidArgument(_))
        ));
    }
}
