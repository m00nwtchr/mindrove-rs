mod support;

use mindrove::{BoardShim, Result};

fn main() -> Result<()> {
    support::init_tracing();

    let (board_id, params) =
        support::BoardArgs::try_from(std::env::args().collect::<Vec<_>>())?.into_parts()?;

    let mut board = BoardShim::new(board_id, params)?;
    board.start_stream()?;

    for marker in 1..=4 {
        board.insert_marker(marker as f64)?;
        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    board.stop_stream()?;
    let data = board.get_board_data()?;
    println!("{data}");
    Ok(())
}
