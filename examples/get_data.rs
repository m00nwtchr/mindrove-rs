mod support;

use mindrove::{BoardId, BoardShim, Result};

fn main() -> Result<()> {
    BoardShim::enable_dev_board_logger()?;

    BoardShim::get_board_presets(BoardId::from_raw(-1))?;

    let (board_id, params) = support::parse_board_args(std::env::args())?;

    let mut board = BoardShim::new(board_id, params)?;
    board.prepare_session()?;
    board.start_stream()?;

    std::thread::sleep(std::time::Duration::from_secs(5));

    board.stop_stream()?;
    let data = board.get_current_board_data(10)?;
    board.release_session()?;
    println!("{data}");
    Ok(())
}
