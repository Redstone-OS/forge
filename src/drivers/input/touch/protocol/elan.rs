//! # ELAN I2C-HID Protocol
//!
//! Lógica específica para processar relatórios de touchpads ELAN.

use super::super::state::TOUCHPAD_STATE;

pub fn process_report(data: &[u8]) {
    if data.len() < 6 {
        return;
    }

    let mut state = TOUCHPAD_STATE.lock();

    let buttons = data[1];
    state.button_left = (buttons & 0x01) != 0;
    state.button_right = (buttons & 0x02) != 0;

    let contact_count = data[2].min(5);
    state.finger_count = contact_count;

    if contact_count > 0 && data.len() >= 8 {
        let x = u16::from_le_bytes([data[4], data[5]]);
        let y = u16::from_le_bytes([data[6], data[7]]);

        let old_x = state.cursor_x;
        let old_y = state.cursor_y;

        let new_x = (x as i32 * state.screen_width) / 3000;
        let new_y = (y as i32 * state.screen_height) / 2000;

        state.delta_x = new_x - old_x;
        state.delta_y = new_y - old_y;
        state.cursor_x = new_x.clamp(0, state.screen_width - 1);
        state.cursor_y = new_y.clamp(0, state.screen_height - 1);

        state.fingers[0].x = x;
        state.fingers[0].y = y;
        state.fingers[0].tip = true;
    }
}
