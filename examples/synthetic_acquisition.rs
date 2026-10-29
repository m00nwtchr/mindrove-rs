mod support;

use mindrove::{BoardId, BoardShim, MindRoveInputParams, Result};

fn main() -> Result<()> {
    support::init_tracing();
    let mut board = BoardShim::new(BoardId::SYNTHETIC_BOARD, MindRoveInputParams::default())?;
    board.start_stream()?;
    std::thread::sleep(std::time::Duration::from_secs(1));
    board.stop_stream()?;
    let data = board.get_current_board_data(10)?;
    println!("{} channels × {} samples", data.nrows(), data.ncols());
    Ok(())
}
