mod support;

use std::{thread, time::Duration};

use mindrove::{BoardId, BoardShim, MindRoveInputParams, Result};

type Device = (MindRoveInputParams, BoardShim);

fn probe(boards: &mut [Device], previous: &mut [Option<Vec<f64>>], phase: &str) {
    println!("\n=== {phase} ===");

    for ((params, board), previous) in boards.iter_mut().zip(previous.iter_mut()) {
        match board.get_current_board_data(10) {
            Ok(data) if data.ncols() > 0 => {
                let latest = data.column(data.ncols() - 1).to_vec();

                let changed = match previous.as_ref() {
                    None => "baseline",
                    Some(last) if last != &latest => "YES",
                    Some(_) => "NO",
                };

                println!(
                    "{}: {} channels, {} samples, changed: {}",
                    params.mac_address,
                    data.nrows(),
                    data.ncols(),
                    changed,
                );

                *previous = Some(latest);
            }
            Ok(_) => {
                println!("{}: no samples", params.mac_address);
            }
            Err(err) => {
                println!("{}: ERROR: {err}", params.mac_address);
            }
        }
    }
}

fn run() -> Result<()> {
    let params =
        support::SyncboxArgs::try_from(std::env::args().collect::<Vec<_>>())?.into_params()?;

    assert!(params.len() >= 2, "At least two devices are required");

    let board_id = BoardId::MINDROVE_SYNCBOX_BOARD;
    let mut boards = Vec::with_capacity(params.len());

    // Prepare and start all devices.
    for entry in params {
        let mut board = BoardShim::new(board_id, entry.clone())?;
        board.start_stream()?;
        boards.push((entry, board));
    }

    println!("All devices streaming. Waiting 5 seconds...");
    thread::sleep(Duration::from_secs(5));

    let mut previous = vec![None; boards.len() - 1];

    for second in 1..=5 {
        thread::sleep(Duration::from_secs(1));

        probe(
            &mut boards[1..],
            &mut previous,
            &format!("BASELINE +{second}s"),
        );
    }

    // The first device is our test subject.
    let (victim_params, mut victim) = boards.remove(0);

    probe(&mut boards, &mut previous, "BEFORE STOP");

    // Phase 1: stop streaming, but retain the prepared session.
    println!("\nStopping {}...", victim_params.mac_address);
    victim.stop_stream()?;

    thread::sleep(Duration::from_secs(2));

    probe(&mut boards, &mut previous, "AFTER STOP");

    // Phase 2: release only the first device through RAII.
    println!("\nDropping {}...", victim_params.mac_address);
    drop(victim);

    println!("Victim released. Monitoring survivors...");

    // Observe possible interruptions and recovery.
    for second in 1..=15 {
        thread::sleep(Duration::from_secs(1));

        probe(
            &mut boards,
            &mut previous,
            &format!("AFTER DROP +{second}s"),
        );
    }

    // Clean up survivors only after all observations.
    println!("\nStopping remaining devices...");
    for (params, board) in &mut boards {
        println!("Stopping {}", params.mac_address);
        board.stop_stream()?;
    }

    println!("Test complete.");
    Ok(())
}

fn main() -> Result<()> {
    support::init_tracing();

    match run() {
        Ok(()) => Ok(()),
        Err(err) => {
            tracing::error!(
                target: "mindrove",
                error = %err,
                "syncbox release test failed"
            );
            Err(err)
        }
    }
}
