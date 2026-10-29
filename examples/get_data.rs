mod support;

use mindrove::{BoardId, BoardShim, Result};

fn main() -> Result<()> {
    support::init_tracing();

    BoardShim::get_board_presets(BoardId::from_raw(-1))?;

    let (board_id, params) =
        support::BoardArgs::try_from(std::env::args().collect::<Vec<_>>())?.into_parts()?;

    let mut board = BoardShim::new(board_id, params)?;
    board.start_stream()?;

    std::thread::sleep(std::time::Duration::from_secs(5));

    board.stop_stream()?;
    let data = board.get_current_board_data(10)?;
    println!("{data}");
    Ok(())
}
