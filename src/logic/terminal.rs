use anyhow::Result;
use crossterm::{
    event,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use std::time::Duration;

pub struct KeyManager {}
impl KeyManager {
    pub fn new() -> Result<Self> {
        Self::ready_to_key_input();

        Ok(Self {})
    }

    pub fn check(&self) -> bool {
        if event::poll(Duration::from_millis(0)).unwrap_or(false) {
            let Ok(event) = event::read() else {
                return false;
            };
            matches!(
                event,
                event::Event::Key(event::KeyEvent {
                    code: _,
                    modifiers: _,
                    kind: _,
                    state: _,
                })
            )
        } else {
            false
        }
    }

    fn ready_to_key_input() {
        enable_raw_mode().expect("Failed to enable raw mode");
    }
}
impl Drop for KeyManager {
    fn drop(&mut self) {
        disable_raw_mode().expect("Failed to disable raw mode");
    }
}

pub fn get_terminal_width() -> Result<usize> {
    terminal::size()
        .map(|(width, _)| width as usize)
        .map_err(From::from)
}
