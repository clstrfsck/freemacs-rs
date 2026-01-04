/*
 * Copyright 2026 Martin Sandiford
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or (at
 * your option) any later version.
 *
 * This program is distributed in the hope that it will be useful, but
 * WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
 * General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program; if not, write to: Free Software Foundation
 * Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301 USA
 */

use std::cell::RefCell;

use crate::emacs_buffer::EmacsBuffer;
use crate::mint_types::{MintCount, MintString};

pub trait EmacsWindow {
    fn get_columns(&self) -> MintCount;
    fn get_lines(&self) -> MintCount;

    fn redisplay(&mut self, buf: &mut EmacsBuffer, force: bool);
    fn overwrite(&mut self, s: &MintString);
    fn gotoxy(&mut self, x: i32, y: i32);
    fn key_waiting(&self) -> bool;
    fn get_input(&mut self, millisec: MintCount) -> MintString;

    fn announce(&mut self, left: &MintString, right: &MintString);
    fn announce_win(&mut self, left: &MintString, right: &MintString);

    fn audible_bell(&mut self, freq: MintCount, millisec: MintCount);
    fn visual_bell(&mut self, millisec: MintCount);

    fn set_fore_colour(&mut self, colour: i32);
    fn get_fore_colour(&self) -> i32;
    fn set_back_colour(&mut self, colour: i32);
    fn get_back_colour(&self) -> i32;
    fn set_ctrl_fore_colour(&mut self, colour: i32);
    fn get_ctrl_fore_colour(&self) -> i32;

    fn set_whitespace_display(&mut self, flag: bool);
    fn get_whitespace_display(&self) -> bool;
    fn set_whitespace_colour(&mut self, colour: i32);
    fn get_whitespace_colour(&self) -> i32;

    fn get_bot_scroll_percent(&self) -> MintCount;
    fn set_bot_scroll_percent(&mut self, perc: MintCount);
    fn get_top_scroll_percent(&self) -> MintCount;
    fn set_top_scroll_percent(&mut self, perc: MintCount);
}

// FIXME: This should not be thread local.
thread_local! {
    static EMACS_WINDOW: RefCell<Option<Box<dyn EmacsWindow>>> = RefCell::new(None);
}

pub fn init_window(w: Box<dyn EmacsWindow>) {
    EMACS_WINDOW.with(|window| {
        *window.borrow_mut() = Some(w);
    });
}

pub fn free_window() {
    EMACS_WINDOW.with(|window| {
        *window.borrow_mut() = None;
    });
}

pub fn with_window<F, R>(f: F) -> R
where
    F: FnOnce(&mut dyn EmacsWindow) -> R,
{
    EMACS_WINDOW.with(|window| {
        let mut window_ref = window.borrow_mut();
        let win = window_ref.as_deref_mut().unwrap();
        f(win)
    })
}

pub fn key_waiting() -> bool {
    with_window(|w| w.key_waiting())
}
