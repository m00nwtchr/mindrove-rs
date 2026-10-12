mod support;

use mindrove::{
    BoardId, BoardShim, DataFilter, MLModel, MindRoveInputParams, MindRovePreset, Result,
};

fn main() -> Result<()> {
    support::init_tracing();
    println!("BoardShim Version: {}", BoardShim::get_version()?);
    println!("DataFilter Version: {}", DataFilter::get_version()?);
    println!("MLModel Version: {}", MLModel::get_version()?);

    let params = MindRoveInputParams::default();
    let mut board = BoardShim::new(BoardId::SYNTHETIC_BOARD, params)?;
    board.prepare_session()?;

    board.add_streamer("file://streamer_default.csv:w")?;
    board.start_stream()?;

    std::thread::sleep(std::time::Duration::from_secs(5));

    board.add_streamer_with_preset(
        "file://streamer_aux.csv:w",
        MindRovePreset::AUXILIARY_PRESET,
    )?;

    board.stop_stream()?;
    let data = board.get_board_data()?;
    println!("{data}");
    let data = board.get_current_board_data(10)?;
    println!("{data}");

    board.release_session()?;
    Ok(())
}
