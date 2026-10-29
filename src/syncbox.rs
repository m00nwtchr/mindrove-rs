use std::collections::HashMap;

use crate::{BoardId, BoardShim, MindRoveInputParams};

#[derive(Default)]
pub struct SyncBox {
    boards: HashMap<String, BoardShim>,
}

impl SyncBox {
    pub fn add_sensor<T: AsRef<str>>(&mut self, mac: T) -> crate::Result<()> {
        let board = BoardShim::new(
            BoardId::MINDROVE_SYNCBOX_BOARD,
            MindRoveInputParams {
                mac_address: mac.as_ref().into(),
                ..Default::default()
            },
        )?;

        self.boards.insert(mac.as_ref().into(), board);
        Ok(())
    }

    pub fn remove_sensor<T: AsRef<str>>(&mut self, mac: T) {
        self.boards.remove(mac.as_ref());
    }

    pub fn start_stream(&mut self) -> crate::Result<()> {
        for board in &mut self.boards.values_mut() {
            board.start_stream()?;
        }

        Ok(())
    }

    pub fn stop_stream(&mut self) -> crate::Result<()> {
        for board in &mut self.boards.values_mut() {
            board.stop_stream()?;
        }

        Ok(())
    }
}
