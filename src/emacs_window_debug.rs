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

use crate::emacs_buffer::EmacsBuffer;
use crate::emacs_window::EmacsWindow;
use crate::mint_types::{MintChar, MintCount, MintString};

pub struct EmacsWindowDebug {
    columns: MintCount,
    lines: MintCount,
    fore: i32,
    back: i32,
    wsp_fore: i32,
    show_wsp: bool,
    ctrl_fore: i32,
    bot_scroll_percent: MintCount,
    top_scroll_percent: MintCount,
}

fn to_s(s: &[MintChar]) -> String {
    String::from_utf8_lossy(s).to_string()
}

impl EmacsWindowDebug {
    pub fn new(cols: MintCount, lines: MintCount) -> Self {
        EmacsWindowDebug {
            columns: cols,
            lines,
            fore: 7,
            back: 0,
            wsp_fore: 6,
            show_wsp: false,
            ctrl_fore: 2,
            bot_scroll_percent: 90,
            top_scroll_percent: 10,
        }
    }
}

impl EmacsWindow for EmacsWindowDebug {
    fn get_columns(&self) -> MintCount {
        self.columns
    }

    fn get_lines(&self) -> MintCount {
        self.lines
    }

    fn redisplay(&mut self, _buf: &mut EmacsBuffer, force: bool) {
        println!("Redisplay(force={})", force);
    }

    fn overwrite(&mut self, s: &MintString) {
        println!("overwrt|{:?}|", to_s(s));
    }

    fn gotoxy(&mut self, x: i32, y: i32) {
        println!("gotoxy({}, {})", x, y);
    }

    fn key_waiting(&self) -> bool {
        println!("key_waiting()");
        false
    }

    fn get_input(&mut self, millisec: MintCount) -> MintString {
        println!("get_input({})", millisec);
        b"Timeout".to_vec()
    }

    fn announce(&mut self, left: &MintString, right: &MintString) {
        println!("ann    |{:?}| |{:?}|", to_s(left), to_s(right));
    }

    fn announce_win(&mut self, left: &MintString, right: &MintString) {
        println!("annw   |{:?}| |{:?}|", to_s(left), to_s(right));
    }

    fn audible_bell(&mut self, freq: MintCount, millisec: MintCount) {
        println!("audible_bell(freq={}, millisec={})", freq, millisec);
    }

    fn visual_bell(&mut self, millisec: MintCount) {
        println!("visual_bell(millisec={})", millisec);
    }

    fn set_fore_colour(&mut self, colour: i32) {
        println!("set_fore_colour({})", colour);
        self.fore = colour;
    }

    fn get_fore_colour(&self) -> i32 {
        self.fore
    }

    fn set_back_colour(&mut self, colour: i32) {
        println!("set_back_colour({})", colour);
        self.back = colour;
    }

    fn get_back_colour(&self) -> i32 {
        self.back
    }

    fn set_ctrl_fore_colour(&mut self, colour: i32) {
        println!("set_ctrl_fore_colour({})", colour);
        self.ctrl_fore = colour;
    }

    fn get_ctrl_fore_colour(&self) -> i32 {
        self.ctrl_fore
    }

    fn set_whitespace_display(&mut self, flag: bool) {
        println!("set_whitespace_display({})", flag);
        self.show_wsp = flag;
    }

    fn get_whitespace_display(&self) -> bool {
        self.show_wsp
    }

    fn set_whitespace_colour(&mut self, colour: i32) {
        println!("set_whitespace_colour({})", colour);
        self.wsp_fore = colour;
    }

    fn get_whitespace_colour(&self) -> i32 {
        self.wsp_fore
    }

    fn get_bot_scroll_percent(&self) -> MintCount {
        self.bot_scroll_percent
    }

    fn set_bot_scroll_percent(&mut self, perc: MintCount) {
        println!("set_bot_scroll_percent({})", perc);
        self.bot_scroll_percent = perc;
    }

    fn get_top_scroll_percent(&self) -> MintCount {
        self.top_scroll_percent
    }

    fn set_top_scroll_percent(&mut self, perc: MintCount) {
        println!("set_top_scroll_percent({})", perc);
        self.top_scroll_percent = perc;
    }
}
