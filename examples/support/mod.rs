#![allow(dead_code)]

use clap::{Args, Parser};
use mindrove::{
    BoardId, Error, IpProtocolType, MindRoveClassifier, MindRoveInputParams, MindRoveMetric,
    MindRoveModelParams, Result,
};

pub(crate) fn init_tracing() {
    tracing_subscriber::fmt::init();
}

pub struct BoardArgs {
    input: InputArgs,
}

impl TryFrom<Vec<String>> for BoardArgs {
    type Error = Error;

    fn try_from(args: Vec<String>) -> Result<Self> {
        let parsed = parse::<BoardCli>(args)?;
        if parsed.input.board_id.is_none() {
            return Err(invalid("missing required --board-id"));
        }
        Ok(Self {
            input: parsed.input,
        })
    }
}

impl BoardArgs {
    pub fn into_parts(self) -> Result<(BoardId, MindRoveInputParams)> {
        let board_id = self
            .input
            .board_id
            .ok_or_else(|| invalid("missing required --board-id"))?;
        Ok((BoardId::from_raw(board_id), self.input.into_params()))
    }
}

pub struct ModelArgs {
    input: InputArgs,
    classifier: MindRoveClassifier,
    metric: MindRoveMetric,
    model_file: String,
    output_name: String,
}

impl TryFrom<Vec<String>> for ModelArgs {
    type Error = Error;

    fn try_from(args: Vec<String>) -> Result<Self> {
        let parsed = parse::<ModelCli>(args)?;
        if parsed.input.board_id.is_none() {
            return Err(invalid("missing required --board-id"));
        }
        Ok(Self {
            input: parsed.input,
            classifier: parsed.classifier,
            metric: parsed.metric,
            model_file: parsed.model_file,
            output_name: parsed.output_name,
        })
    }
}

impl ModelArgs {
    pub fn into_parts(self) -> Result<(BoardId, MindRoveInputParams, MindRoveModelParams)> {
        let board_id = self
            .input
            .board_id
            .ok_or_else(|| invalid("missing required --board-id"))?;
        let model = MindRoveModelParams {
            classifier: self.classifier,
            metric: self.metric,
            file: self.model_file,
            output_name: self.output_name,
            ..Default::default()
        };
        Ok((BoardId::from_raw(board_id), self.input.into_params(), model))
    }
}

pub struct SyncboxArgs {
    input: SyncboxInputArgs,
    mac_addresses: Vec<String>,
    ip_ports: Vec<i32>,
    timeouts: Vec<i32>,
}

impl TryFrom<Vec<String>> for SyncboxArgs {
    type Error = Error;

    fn try_from(args: Vec<String>) -> Result<Self> {
        let parsed = parse::<SyncboxCli>(args)?;
        if parsed.mac_addresses.is_empty() && parsed.ip_ports.is_empty() {
            return Err(invalid(
                "at least one of --mac-address or --ip-port is required",
            ));
        }
        Ok(Self {
            input: parsed.input,
            mac_addresses: parsed.mac_addresses,
            ip_ports: parsed.ip_ports,
            timeouts: parsed.timeouts,
        })
    }
}

impl SyncboxArgs {
    pub fn into_params(self) -> Result<Vec<MindRoveInputParams>> {
        let parsed = self;
        let count = parsed
            .mac_addresses
            .len()
            .max(parsed.ip_ports.len())
            .max(parsed.timeouts.len());
        let mut entries = Vec::with_capacity(count);
        for index in 0..count {
            let mut params = parsed.input.clone().into_params();
            if let Some(mac) = parsed.mac_addresses.get(index) {
                params.mac_address = mac.clone();
            }
            if let Some(port) = parsed.ip_ports.get(index) {
                params.ip_port = *port;
            }
            if let Some(timeout) = parsed.timeouts.get(index) {
                params.timeout = *timeout;
            }
            if let Some(board_id) = parsed.input.board_id {
                params.master_board = BoardId::from_raw(board_id);
            }
            entries.push(params);
        }
        Ok(entries)
    }
}

fn parse<T>(args: Vec<String>) -> Result<T>
where
    T: Parser,
{
    let args = if args.first().is_some_and(|arg| !arg.starts_with('-')) {
        args
    } else {
        std::iter::once("example".to_owned()).chain(args).collect()
    };
    T::try_parse_from(args).map_err(|error| invalid(error.to_string()))
}

#[derive(Debug, Args)]
struct InputArgs {
    #[arg(long)]
    board_id: Option<i32>,
    #[arg(long, default_value = "")]
    ip_address: String,
    #[arg(long, default_value = "")]
    ip_address_aux: String,
    #[arg(long, default_value = "")]
    ip_address_anc: String,
    #[arg(long, default_value_t = -1)]
    ip_port: i32,
    #[arg(long, default_value_t = 0)]
    ip_port_aux: i32,
    #[arg(long, default_value_t = 0)]
    ip_port_anc: i32,
    #[arg(long, value_parser = parse_protocol, default_value = "0")]
    ip_protocol: IpProtocolType,
    #[arg(long, default_value = "")]
    other_info: String,
    #[arg(long, default_value_t = 0)]
    timeout: i32,
    #[arg(long, default_value = "")]
    serial_port: String,
    #[arg(long, default_value = "")]
    mac_address: String,
    #[arg(long, default_value = "")]
    serial_number: String,
    #[arg(long, default_value = "")]
    file: String,
    #[arg(long, default_value = "")]
    file_aux: String,
    #[arg(long, default_value = "")]
    file_anc: String,
    #[arg(long)]
    master_board: Option<i32>,
}

impl InputArgs {
    fn into_params(self) -> MindRoveInputParams {
        MindRoveInputParams {
            serial_port: self.serial_port,
            mac_address: self.mac_address,
            ip_address: self.ip_address,
            ip_address_aux: self.ip_address_aux,
            ip_address_anc: self.ip_address_anc,
            ip_port: self.ip_port,
            ip_port_aux: self.ip_port_aux,
            ip_port_anc: self.ip_port_anc,
            ip_protocol: self.ip_protocol,
            other_info: self.other_info,
            timeout: self.timeout,
            serial_number: self.serial_number,
            file: self.file,
            file_aux: self.file_aux,
            file_anc: self.file_anc,
            master_board: self
                .master_board
                .map(BoardId::from_raw)
                .unwrap_or(BoardId::NO_BOARD),
        }
    }
}

#[derive(Debug, Parser)]
struct BoardCli {
    #[command(flatten)]
    input: InputArgs,
}

#[derive(Debug, Parser)]
struct ModelCli {
    #[command(flatten)]
    input: InputArgs,
    #[arg(long, value_parser = parse_classifier)]
    classifier: MindRoveClassifier,
    #[arg(long, value_parser = parse_metric)]
    metric: MindRoveMetric,
    #[arg(long, default_value = "")]
    model_file: String,
    #[arg(long, default_value = "")]
    output_name: String,
}

#[derive(Debug, Parser)]
struct SyncboxCli {
    #[command(flatten)]
    input: SyncboxInputArgs,
    #[arg(long = "mac-address", action = clap::ArgAction::Append)]
    mac_addresses: Vec<String>,
    #[arg(long = "ip-port", action = clap::ArgAction::Append)]
    ip_ports: Vec<i32>,
    #[arg(long = "timeout", action = clap::ArgAction::Append)]
    timeouts: Vec<i32>,
}

#[derive(Debug, Args, Clone)]
struct SyncboxInputArgs {
    #[arg(long)]
    board_id: Option<i32>,
    #[arg(long, default_value = "")]
    ip_address: String,
    #[arg(long, default_value = "")]
    ip_address_aux: String,
    #[arg(long, default_value = "")]
    ip_address_anc: String,
    #[arg(long, default_value_t = 0)]
    ip_port_aux: i32,
    #[arg(long, default_value_t = 0)]
    ip_port_anc: i32,
    #[arg(long, value_parser = parse_protocol, default_value = "0")]
    ip_protocol: IpProtocolType,
    #[arg(long, default_value = "")]
    other_info: String,
    #[arg(long, default_value = "")]
    serial_port: String,
    #[arg(long, default_value = "")]
    serial_number: String,
    #[arg(long, default_value = "")]
    file: String,
    #[arg(long, default_value = "")]
    file_aux: String,
    #[arg(long, default_value = "")]
    file_anc: String,
    #[arg(long)]
    master_board: Option<i32>,
}

impl SyncboxInputArgs {
    fn into_params(self) -> MindRoveInputParams {
        MindRoveInputParams {
            serial_port: self.serial_port,
            mac_address: String::new(),
            ip_address: self.ip_address,
            ip_address_aux: self.ip_address_aux,
            ip_address_anc: self.ip_address_anc,
            ip_port: -1,
            ip_port_aux: self.ip_port_aux,
            ip_port_anc: self.ip_port_anc,
            ip_protocol: self.ip_protocol,
            other_info: self.other_info,
            timeout: 0,
            serial_number: self.serial_number,
            file: self.file,
            file_aux: self.file_aux,
            file_anc: self.file_anc,
            master_board: self
                .master_board
                .map(BoardId::from_raw)
                .unwrap_or(BoardId::NO_BOARD),
        }
    }
}

fn parse_protocol(value: &str) -> std::result::Result<IpProtocolType, String> {
    match value.parse::<i32>().map_err(|_| value.to_owned())? {
        0 => Ok(IpProtocolType::NO_IP_PROTOCOL),
        1 => Ok(IpProtocolType::UDP),
        2 => Ok(IpProtocolType::TCP),
        _ => Err(value.to_owned()),
    }
}

fn parse_classifier(value: &str) -> std::result::Result<MindRoveClassifier, String> {
    match value.parse::<i32>().map_err(|_| value.to_owned())? {
        0 => Ok(MindRoveClassifier::DEFAULT_CLASSIFIER),
        1 => Ok(MindRoveClassifier::DYN_LIB_CLASSIFIER),
        2 => Ok(MindRoveClassifier::ONNX_CLASSIFIER),
        _ => Err(value.to_owned()),
    }
}

fn parse_metric(value: &str) -> std::result::Result<MindRoveMetric, String> {
    match value.parse::<i32>().map_err(|_| value.to_owned())? {
        0 => Ok(MindRoveMetric::MINDFULNESS),
        1 => Ok(MindRoveMetric::RESTFULNESS),
        2 => Ok(MindRoveMetric::USER_DEFINED),
        _ => Err(value.to_owned()),
    }
}

fn invalid(message: impl Into<String>) -> Error {
    Error::InvalidArgument(message.into())
}

#[cfg(test)]
mod tests {
    use super::{BoardArgs, ModelArgs, SyncboxArgs};
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
        let (board_id, params) =
            BoardArgs::try_from(args.iter().map(ToString::to_string).collect::<Vec<_>>())
                .unwrap()
                .into_parts()
                .unwrap();
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
        assert!(matches!(
            BoardArgs::try_from(vec!["get_data".to_owned(), "--board-id".to_owned()]),
            Err(mindrove::Error::InvalidArgument(_))
        ));
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
        let (board_id, input, model) =
            ModelArgs::try_from(args.iter().map(ToString::to_string).collect::<Vec<_>>())
                .unwrap()
                .into_parts()
                .unwrap();
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
                ModelArgs::try_from(args.iter().map(ToString::to_string).collect::<Vec<_>>()),
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
        let entries =
            SyncboxArgs::try_from(args.iter().map(ToString::to_string).collect::<Vec<_>>())
                .unwrap()
                .into_params()
                .unwrap();
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
            SyncboxArgs::try_from(vec!["syncbox_get_data".to_owned()]),
            Err(mindrove::Error::InvalidArgument(_))
        ));
    }
}
