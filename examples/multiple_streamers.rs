mod support;

use mindrove::{BoardId, BoardShim, MindRoveInputParams, MindRovePreset, Result};

fn main() -> Result<()> {
    support::init_tracing();

    let mut board = BoardShim::new(BoardId::SYNTHETIC_BOARD, MindRoveInputParams::default())?;

    board.add_streamer("file://streamer_default_1.csv:w")?;
    board.add_streamer("file://streamer_default_2.csv:w")?;
    board.add_streamer_with_preset(
        "file://streamer_aux_1.csv:w",
        MindRovePreset::AUXILIARY_PRESET,
    )?;
    board.add_streamer_with_preset(
        "file://streamer_aux_2.csv:w",
        MindRovePreset::AUXILIARY_PRESET,
    )?;

    board.start_stream()?;

    std::thread::sleep(std::time::Duration::from_secs(5));

    board.delete_streamer_with_preset(
        "file://streamer_aux_2.csv:w",
        MindRovePreset::AUXILIARY_PRESET,
    )?;

    std::thread::sleep(std::time::Duration::from_secs(5));

    board.stop_stream()?;
    Ok(())
}
