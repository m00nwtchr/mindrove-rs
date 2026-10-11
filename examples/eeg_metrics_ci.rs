mod support;

use std::time::Duration;

use mindrove::{BoardShim, DataFilter, MLModel};

fn main() -> mindrove::Result<()> {
    BoardShim::enable_dev_board_logger()?;
    MLModel::enable_dev_ml_logger()?;

    let (board_id, input_params, model_params) = support::parse_model_args(std::env::args())?;
    let mut board = BoardShim::new(board_id, input_params)?;
    board.prepare_session()?;
    let master_board_id = board.get_board_id()?;
    board.start_stream()?;
    std::thread::sleep(Duration::from_secs(5));
    board.stop_stream()?;

    let data = board.get_board_data()?;
    board.release_session()?;
    println!("{data}");

    let sampling_rate = BoardShim::get_sampling_rate(master_board_id)?;
    let eeg_channels = BoardShim::get_eeg_channels(master_board_id)?;
    let bands = DataFilter::get_avg_band_powers(&data, &eeg_channels, sampling_rate, true)?;

    let mut model = MLModel::new(model_params)?;
    model.prepare()?;
    println!("Score :");
    for score in model.predict(&bands.average)? {
        println!("{score}");
    }
    model.release()?;

    Ok(())
}
