mod support;

use std::time::Duration;

use mindrove::{
    BoardId, BoardShim, DataFilter, MLModel, MindRoveClassifier, MindRoveMetric,
    MindRoveModelParams,
};

fn main() -> mindrove::Result<()> {
    support::init_tracing();

    let (board_id, input_params) =
        support::BoardArgs::try_from(std::env::args().collect::<Vec<_>>())?.into_parts()?;
    let mut board = BoardShim::new(board_id, input_params)?;
    board.start_stream()?;
    std::thread::sleep(Duration::from_secs(5));
    board.stop_stream()?;

    let data = board.get_board_data()?;
    println!("{data}");

    let sampling_rate = BoardShim::get_sampling_rate(BoardId::SYNTHETIC_BOARD)?;
    let eeg_channels = BoardShim::get_eeg_channels(board_id)?;
    let bands = DataFilter::get_avg_band_powers(&data, &eeg_channels, sampling_rate, true)?;

    let mut mindfulness_model = MLModel::new(MindRoveModelParams {
        metric: MindRoveMetric::MINDFULNESS,
        classifier: MindRoveClassifier::DEFAULT_CLASSIFIER,
        ..Default::default()
    })?;
    mindfulness_model.prepare()?;
    let mindfulness = mindfulness_model.predict(&bands.average)?;
    println!("Mindfulness :{}", mindfulness[0]);
    mindfulness_model.release()?;

    let mut restfulness_model = MLModel::new(MindRoveModelParams {
        metric: MindRoveMetric::RESTFULNESS,
        classifier: MindRoveClassifier::DEFAULT_CLASSIFIER,
        ..Default::default()
    })?;
    restfulness_model.prepare()?;
    let restfulness = restfulness_model.predict(&bands.average)?;
    println!("Restfulness :{}", restfulness[0]);
    restfulness_model.release()?;

    Ok(())
}
