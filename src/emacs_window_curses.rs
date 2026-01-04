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
use crate::mint_types::{MintCount, MintString};
use ncurses::*;
use std::cmp::{max, min};
use std::collections::HashMap;
use std::io::IsTerminal;

pub struct EmacsWindowCurses {
    win: WINDOW,
    overwriting: bool,
    ovy: i32,
    ovx: i32,
    has_colours: bool,
    curr_colour_pair: i16,
    fore: i32,
    back: i32,
    wsp_fore: i32,
    show_wsp: bool,
    ctrl_fore: i32,
    old_fore: i32,
    old_back: i32,
    decode_key: HashMap<i32, MintString>,
    bot_scroll_percent: MintCount,
    top_scroll_percent: MintCount,
}

impl Default for EmacsWindowCurses {
    fn default() -> Self {
        Self::new()
    }
}

fn key_fn(n: u8) -> i32 {
    // The comment in ncurses.h says:
    /* Function keys.  Space for 64 */
    // I assume that means F0 to F63.
    assert!(n < 64);
    KEY_F0 + n as i32
}

impl EmacsWindowCurses {
    pub fn new() -> Self {
        let is_tty = std::io::stdout().is_terminal();

        let (win, has_colours) = if is_tty {
            let win = initscr();
            let has_colours = has_colors();

            if has_colours {
                start_color();
            }

            raw();
            noecho();
            nl();
            intrflush(win, false);
            keypad(win, true);

            idlok(win, true);
            idcok(win, true);

            scrollok(win, true);
            clearok(win, true);
            leaveok(win, false);

            let lines = getmaxy(win);
            wsetscrreg(win, 0, lines - 3);

            werase(win);

            (win, has_colours)
        } else {
            (std::ptr::null_mut(), false)
        };

        let mut decode_key = HashMap::new();

        // Fill out the defaults
        decode_key.insert(0x00, b"C-@".to_vec());
        for i in 1..32u8 {
            let mut key = b"C-".to_vec();
            key.push(i + b'a' - 1);
            decode_key.insert(i as i32, key);
        }
        for i in 32..127u8 {
            decode_key.insert(i as i32, vec![i]);
        }

        // Fill in the specials
        decode_key.insert(0x08, b"Back Space".to_vec());
        decode_key.insert(0x09, b"Tab".to_vec());
        decode_key.insert(0x0A, b"Return".to_vec());
        decode_key.insert(0x0D, b"Return".to_vec());
        decode_key.insert(0x1B, b"Escape".to_vec());
        decode_key.insert(b',' as i32, b"Comma".to_vec());
        decode_key.insert(b'(' as i32, b"LPar".to_vec());
        decode_key.insert(b')' as i32, b"RPar".to_vec());
        decode_key.insert(0x7F, b"Back Space".to_vec());

        // NCURSES decodes
        decode_key.insert(KEY_DOWN, b"Down Arrow".to_vec());
        decode_key.insert(KEY_UP, b"Up Arrow".to_vec());
        decode_key.insert(KEY_LEFT, b"Left Arrow".to_vec());
        decode_key.insert(KEY_RIGHT, b"Right Arrow".to_vec());
        decode_key.insert(KEY_HOME, b"Home".to_vec());
        decode_key.insert(KEY_BACKSPACE, b"Back Space".to_vec());
        decode_key.insert(key_fn(1), b"F1".to_vec());
        decode_key.insert(key_fn(2), b"F2".to_vec());
        decode_key.insert(key_fn(3), b"F3".to_vec());
        decode_key.insert(key_fn(4), b"F4".to_vec());
        decode_key.insert(key_fn(5), b"F5".to_vec());
        decode_key.insert(key_fn(6), b"F6".to_vec());
        decode_key.insert(key_fn(7), b"F7".to_vec());
        decode_key.insert(key_fn(8), b"F8".to_vec());
        decode_key.insert(key_fn(9), b"F9".to_vec());
        decode_key.insert(key_fn(10), b"F10".to_vec());
        decode_key.insert(key_fn(11), b"F11".to_vec());
        decode_key.insert(key_fn(12), b"F12".to_vec());
        decode_key.insert(key_fn(13), b"S-F1".to_vec());
        decode_key.insert(key_fn(14), b"S-F2".to_vec());
        decode_key.insert(key_fn(15), b"S-F3".to_vec());
        decode_key.insert(key_fn(16), b"S-F4".to_vec());
        decode_key.insert(key_fn(17), b"S-F5".to_vec());
        decode_key.insert(key_fn(18), b"S-F6".to_vec());
        decode_key.insert(key_fn(19), b"S-F7".to_vec());
        decode_key.insert(key_fn(20), b"S-F8".to_vec());
        decode_key.insert(key_fn(21), b"S-F9".to_vec());
        decode_key.insert(key_fn(22), b"S-F10".to_vec());
        decode_key.insert(key_fn(23), b"S-F11".to_vec());
        decode_key.insert(key_fn(24), b"S-F12".to_vec());
        decode_key.insert(KEY_DC, b"Del".to_vec());
        decode_key.insert(KEY_IC, b"Ins".to_vec());
        decode_key.insert(KEY_NPAGE, b"Pg Dn".to_vec());
        decode_key.insert(KEY_PPAGE, b"Pg Up".to_vec());
        decode_key.insert(KEY_END, b"End".to_vec());

        let mut window = Self {
            win,
            overwriting: false,
            ovy: 0,
            ovx: 0,
            has_colours,
            curr_colour_pair: 0,
            fore: 15,
            back: 0,
            wsp_fore: 15,
            show_wsp: false,
            ctrl_fore: 11,
            old_fore: -1,
            old_back: -1,
            decode_key,
            bot_scroll_percent: 0,
            top_scroll_percent: 0,
        };

        if !win.is_null() {
            window.set_curses_attributes(window.fore, window.back);
        }

        window
    }

    fn write_line(&mut self, buf: &EmacsBuffer, bol: MintCount, eol: MintCount) {
        let cols = getmaxx(self.win);
        let leftcol = buf.get_left_column();

        let text = buf.read_to_mark_from(crate::emacs_buffer::MARK_EOB, bol);
        let line_len = min((eol - bol) as usize, text.len());
        let line_text = &text[..line_len];

        // Find the last non-space character
        let mut nwsp_idx = line_len;
        for (idx, &ch) in line_text.iter().enumerate().rev() {
            if ch != b'\t' && ch != b' ' {
                nwsp_idx = idx + 1;
                break;
            }
        }

        let mut cur_col = 0i32;
        let mut char_idx = 0;

        // Skip to leftcol
        while cur_col < leftcol as i32 && char_idx < line_len {
            let ch = line_text[char_idx];
            cur_col += buf.char_width(cur_col as MintCount, ch) as i32;
            char_idx += 1;
        }

        // Write visible characters
        while cur_col < (leftcol as i32 + cols) && char_idx < line_len {
            let ch = line_text[char_idx];
            char_idx += 1;

            if ch == 0x09 {
                let mut tabw = buf.char_width(cur_col as MintCount, ch) as i32;
                tabw = min(tabw, leftcol as i32 + cols - cur_col);

                let display_ch = if self.show_wsp && char_idx > nwsp_idx {
                    self.set_curses_attributes(self.wsp_fore, self.back);
                    ACS_BULLET()
                } else {
                    self.set_curses_attributes(self.fore, self.back);
                    b' ' as chtype
                };

                for _ in 0..tabw {
                    waddch(self.win, display_ch);
                }
                cur_col += tabw;
            } else if ch < 0x20 {
                self.set_curses_attributes(self.ctrl_fore, self.back);
                waddch(self.win, (ch + b'@') as chtype);
                cur_col += 1;
            } else if ch == 0x20 {
                let display_ch = if self.show_wsp && char_idx > nwsp_idx {
                    self.set_curses_attributes(self.wsp_fore, self.back);
                    ACS_BULLET()
                } else {
                    self.set_curses_attributes(self.fore, self.back);
                    b' ' as chtype
                };
                waddch(self.win, display_ch);
                cur_col += 1;
            } else {
                self.set_curses_attributes(self.fore, self.back);
                waddch(self.win, ch as chtype);
                cur_col += 1;
            }
        }

        if cur_col < (leftcol as i32 + cols) {
            self.set_curses_attributes(self.fore, self.back);
            wclrtoeol(self.win);
        }
    }

    fn set_curses_attributes(&mut self, fo: i32, ba: i32) {
        if self.has_colours && (fo != self.old_fore || ba != self.old_back) {
            self.old_fore = fo;
            self.old_back = ba;

            let forecolour = curses_colour(fo);
            let forebold = curses_bold(fo);
            let backcolour = curses_colour(ba);

            let mut use_pair = COLOR_PAIRS() as i16;

            for i in 0..COLOR_PAIRS() as i16 {
                let mut f: i16 = 0;
                let mut b: i16 = 0;
                if pair_content(i, &mut f, &mut b) != ERR && f == forecolour && b == backcolour {
                    use_pair = i;
                    break;
                }
            }

            if use_pair >= COLOR_PAIRS() as i16 {
                self.curr_colour_pair += 1;
                if self.curr_colour_pair >= COLOR_PAIRS() as i16 {
                    self.curr_colour_pair = 1;
                }
                use_pair = self.curr_colour_pair;
                init_pair(use_pair, forecolour, backcolour);
            }

            wattrset(self.win, COLOR_PAIR(use_pair) | forebold);
            wbkgdset(self.win, COLOR_PAIR(use_pair) | forebold | b' ' as chtype);
        }
    }
}

impl EmacsWindow for EmacsWindowCurses {
    fn get_columns(&self) -> MintCount {
        if !self.win.is_null() {
            getmaxx(self.win) as MintCount
        } else {
            80
        }
    }

    fn get_lines(&self) -> MintCount {
        if !self.win.is_null() {
            (getmaxy(self.win) - 3) as MintCount
        } else {
            24
        }
    }

    fn redisplay(&mut self, buf: &mut EmacsBuffer, force: bool) {
        if !self.win.is_null() {
            self.overwriting = false;

            if force {
                touchwin(self.win);
            }

            let lines = getmaxy(self.win);
            let cols = getmaxx(self.win);

            buf.force_point_in_window(
                (lines - 2) as MintCount,
                cols as MintCount,
                self.top_scroll_percent,
                self.bot_scroll_percent,
            );

            let mut curline = buf.get_mark_position(crate::emacs_buffer::MARK_TOPLINE);
            let point = buf.get_mark_position(crate::emacs_buffer::MARK_POINT);
            let screen_line = buf.count_newlines(curline, point);
            let screen_col = buf.get_column() as i32 - buf.get_left_column() as i32;

            for i in 0..(lines - 2) {
                wmove(self.win, i, 0);
                let eol = buf.get_mark_position_from(crate::emacs_buffer::MARK_EOL, curline);
                self.write_line(buf, curline, eol);
                curline = buf.get_mark_position_from(crate::emacs_buffer::MARK_NEXT_CHAR, eol);
            }

            wmove(self.win, screen_line as i32, screen_col);
        }
    }

    fn overwrite(&mut self, s: &MintString) {
        if !self.win.is_null() {
            if !self.overwriting {
                self.overwriting = true;
                self.ovy = 0;
                self.ovx = 0;
            }

            self.set_curses_attributes(self.fore, self.back);
            wmove(self.win, self.ovy, self.ovx);

            for &ch in s.iter() {
                waddch(self.win, ch as chtype);
            }

            let mut y = 0;
            let mut x = 0;
            getyx(self.win, &mut y, &mut x);
            self.ovy = y;
            self.ovx = x;
        } else {
            use std::io::{self, Write};
            io::stdout().write_all(s).ok();
        }
    }

    fn gotoxy(&mut self, x: i32, y: i32) {
        if !self.win.is_null() {
            if !self.overwriting {
                self.overwriting = true;
            }

            let lines = getmaxy(self.win);
            let cols = getmaxx(self.win);

            self.ovy = max(0, min(y, lines - 1));
            self.ovx = max(0, min(x, cols - 1));
            wmove(self.win, self.ovy, self.ovx);
        }
    }

    fn key_waiting(&self) -> bool {
        if !self.win.is_null() {
            #[cfg(not(target_os = "windows"))]
            {
                nodelay(self.win, true);
                wtimeout(self.win, 0);
                let ch = wgetch(self.win);
                if ch != ERR {
                    ungetch(ch);
                    return true;
                }
            }
        }
        false
    }

    fn get_input(&mut self, millisec: MintCount) -> MintString {
        if !self.win.is_null() {
            if millisec < 10 {
                nodelay(self.win, true);
                wtimeout(self.win, 0);
            } else {
                nodelay(self.win, false);
                wtimeout(self.win, millisec as i32);
            }

            let ch = wgetch(self.win);

            if ch == ERR {
                b"Timeout".to_vec()
            } else {
                self.decode_key
                    .get(&ch)
                    .cloned()
                    .unwrap_or_else(|| b"Unknown".to_vec())
            }
        } else if millisec > 0 {
            use std::io::{self, Read};
            let mut buffer = [0u8; 1];
            if io::stdin().read(&mut buffer).ok().unwrap_or(0) > 0 {
                vec![buffer[0]]
            } else {
                b"Timeout".to_vec()
            }
        } else {
            b"Timeout".to_vec()
        }
    }

    fn announce(&mut self, left: &MintString, right: &MintString) {
        if !self.win.is_null() {
            let cols = getmaxx(self.win);
            let lines = getmaxy(self.win);
            let n = min(left.len(), (cols - 1) as usize);

            self.set_curses_attributes(self.fore, self.back);
            wmove(self.win, lines - 1, 0);

            for &ch in left.iter().take(n) {
                waddch(self.win, ch as chtype);
            }

            let mut y = 0;
            let mut x = 0;
            getyx(self.win, &mut y, &mut x);

            let m = min(right.len(), (cols - (n as i32 + 1)) as usize);
            for &ch in right.iter().take(m) {
                waddch(self.win, ch as chtype);
            }

            if (n + m) < cols as usize {
                wclrtoeol(self.win);
            }

            wmove(self.win, y, x);
            refresh();
        } else {
            use std::io::{self, Write};
            io::stdout().write_all(left).ok();
            io::stdout().write_all(right).ok();
            println!();
        }
    }

    fn announce_win(&mut self, left: &MintString, right: &MintString) {
        if !self.win.is_null() {
            let cols = getmaxx(self.win);
            let lines = getmaxy(self.win);
            let n = min(left.len(), (cols - 1) as usize);

            self.set_curses_attributes(self.fore, self.back);

            let mut y = 0;
            let mut x = 0;
            getyx(self.win, &mut y, &mut x);

            wmove(self.win, lines - 2, 0);

            for &ch in left.iter().take(n) {
                waddch(self.win, ch as chtype);
            }

            let m = min(right.len(), (cols - n as i32) as usize);
            for &ch in right.iter().take(m) {
                waddch(self.win, ch as chtype);
            }

            if (n + m) < cols as usize {
                wclrtoeol(self.win);
            }

            wmove(self.win, y, x);
            refresh();
        }
    }

    fn audible_bell(&mut self, _freq: MintCount, _millisec: MintCount) {
        if !self.win.is_null() {
            beep();
        } else {
            print!("\x07");
        }
    }

    fn visual_bell(&mut self, _millisec: MintCount) {
        if !self.win.is_null() {
            flash();
        }
    }

    fn set_fore_colour(&mut self, colour: i32) {
        self.fore = colour;
    }

    fn get_fore_colour(&self) -> i32 {
        self.fore
    }

    fn set_back_colour(&mut self, colour: i32) {
        self.back = colour;
    }

    fn get_back_colour(&self) -> i32 {
        self.back
    }

    fn set_ctrl_fore_colour(&mut self, colour: i32) {
        self.ctrl_fore = colour;
    }

    fn get_ctrl_fore_colour(&self) -> i32 {
        self.ctrl_fore
    }

    fn set_whitespace_display(&mut self, flag: bool) {
        self.show_wsp = flag;
    }

    fn get_whitespace_display(&self) -> bool {
        self.show_wsp
    }

    fn set_whitespace_colour(&mut self, colour: i32) {
        self.wsp_fore = colour;
    }

    fn get_whitespace_colour(&self) -> i32 {
        self.wsp_fore
    }

    fn get_bot_scroll_percent(&self) -> MintCount {
        self.bot_scroll_percent
    }

    fn set_bot_scroll_percent(&mut self, perc: MintCount) {
        self.bot_scroll_percent = perc;
    }

    fn get_top_scroll_percent(&self) -> MintCount {
        self.top_scroll_percent
    }

    fn set_top_scroll_percent(&mut self, perc: MintCount) {
        self.top_scroll_percent = perc;
    }
}

impl Drop for EmacsWindowCurses {
    fn drop(&mut self) {
        if !self.win.is_null() {
            endwin();
        }
    }
}

fn curses_colour(colour: i32) -> i16 {
    const COLOUR_XLAT: [i16; 8] = [
        COLOR_BLACK,
        COLOR_BLUE,
        COLOR_GREEN,
        COLOR_CYAN,
        COLOR_RED,
        COLOR_MAGENTA,
        COLOR_YELLOW,
        COLOR_WHITE,
    ];
    COLOUR_XLAT[(colour & 0x07) as usize]
}

fn curses_bold(colour: i32) -> chtype {
    if (colour & 0x08) != 0 {
        A_BOLD
    } else {
        A_NORMAL
    }
}
