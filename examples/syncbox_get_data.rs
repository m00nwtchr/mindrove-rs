mod support;

use std::time::Duration;

use mindrove::{BoardId, BoardShim, MindRoveInputParams, Result};
use ndarray::Array2;

fn run() -> Result<()> {
    let params =
        support::SyncboxArgs::try_from(std::env::args().collect::<Vec<_>>())?.into_params()?;
    let board_id = BoardId::MINDROVE_SYNCBOX_BOARD;

    let mut boards: Vec<(MindRoveInputParams, BoardShim)> = Vec::with_capacity(params.len());
    for entry in params {
        let mut board = BoardShim::new(board_id, entry.clone())?;
        if let Err(err) = start_board(&mut board) {
            tracing::error!(target: "mindrove", error = %err, "failed to start board");
        }
        boards.push((entry, board));
    }

    std::thread::sleep(Duration::from_secs(5));

    for (params, board) in boards.iter_mut() {
        match stop_and_get(board) {
            Ok(data) => {
                println!(
                    "Data from board with mac_address: {} or idx (ip_port): {}",
                    params.mac_address, params.ip_port
                );
                println!("{data}");
            }
            Err(err) => {
                tracing::error!(target: "mindrove", error = %err, "failed to stop and get data");
            }
        }
    }

    Ok(())
}

fn start_board(board: &mut BoardShim) -> Result<()> {
    board.start_stream()?;
    Ok(())
}

fn stop_and_get(board: &mut BoardShim) -> Result<Array2<f64>> {
    board.stop_stream()?;
    let data = board.get_current_board_data(10)?;
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
