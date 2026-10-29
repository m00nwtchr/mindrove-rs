#![cfg(all(feature = "native-tests", target_os = "linux"))]

use mindrove::{BoardId, BoardShim, MindRoveInputParams};
use std::sync::{Mutex, OnceLock};

static NATIVE_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn native_test_lock() -> std::sync::MutexGuard<'static, ()> {
    NATIVE_TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap()
}

fn running_synthetic_board() -> BoardShim {
    let mut board =
        BoardShim::new(BoardId::SYNTHETIC_BOARD, MindRoveInputParams::default()).unwrap();
    assert!(board.is_prepared().unwrap());
    board.start_stream().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));
    board
}

#[test]
fn synthetic_board_returns_channel_sample_data() {
    let _lock = native_test_lock();
    let mut board = running_synthetic_board();
    let data = board.get_current_board_data(2).unwrap();
    assert_eq!(
        data.nrows(),
        BoardShim::get_num_rows(BoardId::SYNTHETIC_BOARD).unwrap()
    );
    assert!(data.ncols() <= 2);
    board.stop_stream().unwrap();
}

#[test]
fn synthetic_board_reads_available_data_count_and_prefix() {
    let _lock = native_test_lock();
    let mut board = running_synthetic_board();
    let count = board.get_board_data_count().unwrap();
    let data = board.get_board_data_n(count).unwrap();
    assert_eq!(
        data.nrows(),
        BoardShim::get_num_rows(BoardId::SYNTHETIC_BOARD).unwrap()
    );
    assert!(data.ncols() <= count);
    board.stop_stream().unwrap();
}

#[test]
fn synthetic_board_reports_id_and_failed_cleanup_is_safe() {
    let _lock = native_test_lock();
    let board = BoardShim::new(BoardId::SYNTHETIC_BOARD, MindRoveInputParams::default()).unwrap();
    assert_eq!(board.get_board_id().unwrap(), BoardId::SYNTHETIC_BOARD);
    assert!(BoardShim::new(BoardId::from_raw(12345), MindRoveInputParams::default()).is_err());
}
