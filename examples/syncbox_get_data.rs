mod support;

use std::time::Duration;

use mindrove::{BoardId, BoardShim, MindRoveInputParams, Result};
use ndarray::Array2;

fn run() -> Result<()> {
    let params = support::parse_syncbox_args(std::env::args())?;
    let board_id = BoardId::MINDROVE_SYNCBOX_BOARD;

    let mut boards: Vec<(MindRoveInputParams, BoardShim)> = Vec::with_capacity(params.len());
    for entry in params {
        let mut board = BoardShim::new(board_id, entry.clone())?;
        if let Err(err) = prepare_and_start(&mut board) {
            tracing::error!(target: "mindrove", error = %err, "failed to prepare and start board");
        }
        boards.push((entry, board));
    }

    std::thread::sleep(Duration::from_secs(5));

    for (params, board) in boards.iter_mut() {
        match stop_get_release(board) {
            Ok(data) => {
                println!(
                    "Data from board with mac_address: {} or idx (ip_port): {}",
                    params.mac_address, params.ip_port
                );
                println!("{data}");
            }
            Err(err) => {
                tracing::error!(target: "mindrove", error = %err, "failed to stop, get data, and release board");
            }
        }
    }

    Ok(())
}

fn prepare_and_start(board: &mut BoardShim) -> Result<()> {
    board.prepare_session()?;
    board.start_stream()?;
    Ok(())
}

fn stop_get_release(board: &mut BoardShim) -> Result<Array2<f64>> {
    board.stop_stream()?;
    let data = board.get_current_board_data(10)?;
    board.release_session()?;
    Ok(data)
}

fn main() -> Result<()> {
    support::init_tracing();
    match run() {
        Ok(()) => Ok(()),
        Err(err) => {
            tracing::error!(target: "mindrove", error = %err, "syncbox example failed");
            Err(err)
        }
    }
}
