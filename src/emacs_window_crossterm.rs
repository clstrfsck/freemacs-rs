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

use std::cmp::{max, min};
use std::io::{self, BufWriter, IsTerminal, Write};
use std::time::Duration;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute, queue,
    style::{Color, Colors, Print, SetColors},
    terminal::{self, ClearType},
};

use crate::emacs_buffer::EmacsBuffer;
use crate::emacs_window::EmacsWindow;
use crate::mint_types::{MintCount, MintString};

pub struct EmacsWindowCrossterm {
    writer: BufWriter<io::Stdout>,
    is_tty: bool,
    overwriting: bool,
    ovy: u16,
    ovx: u16,
    fore: i32,
    back: i32,
    wsp_fore: i32,
    show_wsp: bool,
    ctrl_fore: i32,
    bot_scroll_percent: MintCount,
    top_scroll_percent: MintCount,
}

impl Default for EmacsWindowCrossterm {
    fn default() -> Self {
        Self::new()
    }
}

impl EmacsWindowCrossterm {
    pub fn new() -> Self {
        let is_tty = io::stdout().is_terminal();
        let mut writer = BufWriter::new(io::stdout());

        if is_tty {
            terminal::enable_raw_mode().expect("failed to enable raw mode");
            execute!(
                writer,
                terminal::EnterAlternateScreen,
                terminal::Clear(ClearType::All),
                cursor::Hide,
            )
            .expect("failed to initialise terminal");
        }

        Self {
            writer,
            is_tty,
            overwriting: false,
            ovy: 0,
            ovx: 0,
            fore: 15,
            back: 0,
            wsp_fore: 15,
            show_wsp: false,
            ctrl_fore: 11,
            bot_scroll_percent: 0,
            top_scroll_percent: 0,
        }
    }

    fn term_size(&self) -> (u16, u16) {
        terminal::size().unwrap_or((80, 27))
    }

    /// Queue a colour change, applying it only when it differs from the previous call.
    /// Callers must flush the writer at appropriate points (end of redisplay, announce, etc.).
    fn queue_colours(&mut self, fore: i32, back: i32) {
        queue!(
            self.writer,
            SetColors(Colors::new(ansi_colour(fore), ansi_colour(back)))
        )
        .ok();
    }

    fn write_line(&mut self, buf: &EmacsBuffer, bol: MintCount, eol: MintCount) {
        let (cols, _) = self.term_size();
        let leftcol = buf.get_left_column();

        let text = buf.read_to_mark_from(crate::emacs_buffer::MARK_EOB, bol);
        let line_len = min((eol - bol) as usize, text.len());
        let line_text = &text[..line_len];

        // Find the last non-space/non-tab character index, for trailing whitespace display.
        let mut nwsp_idx = line_len;
        for (idx, &ch) in line_text.iter().enumerate().rev() {
            if ch != b'\t' && ch != b' ' {
                nwsp_idx = idx + 1;
                break;
            }
        }

        let mut cur_col = 0i32;
        let mut char_idx = 0;

        // Advance past left-scroll column without writing.
        while cur_col < leftcol as i32 && char_idx < line_len {
            let ch = line_text[char_idx];
            cur_col += buf.char_width(cur_col as MintCount, ch) as i32;
            char_idx += 1;
        }

        // Write visible characters.
        while cur_col < (leftcol as i32 + cols as i32) && char_idx < line_len {
            let ch = line_text[char_idx];
            char_idx += 1;

            if ch == b'\t' {
                let mut tabw = buf.char_width(cur_col as MintCount, ch) as i32;
                tabw = min(tabw, leftcol as i32 + cols as i32 - cur_col);

                if self.show_wsp && char_idx > nwsp_idx {
                    self.queue_colours(self.wsp_fore, self.back);
                    for _ in 0..tabw {
                        queue!(self.writer, Print('·')).ok();
                    }
                } else {
                    self.queue_colours(self.fore, self.back);
                    for _ in 0..tabw {
                        queue!(self.writer, Print(' ')).ok();
                    }
                }
                cur_col += tabw;
            } else if ch < 0x20 {
                // Control character — display as ^X.
                self.queue_colours(self.ctrl_fore, self.back);
                queue!(self.writer, Print((ch + b'@') as char)).ok();
                cur_col += 1;
            } else if ch == b' ' {
                if self.show_wsp && char_idx > nwsp_idx {
                    self.queue_colours(self.wsp_fore, self.back);
                    queue!(self.writer, Print('·')).ok();
                } else {
                    self.queue_colours(self.fore, self.back);
                    queue!(self.writer, Print(' ')).ok();
                }
                cur_col += 1;
            } else {
                self.queue_colours(self.fore, self.back);
                queue!(self.writer, Print(ch as char)).ok();
                cur_col += 1;
            }
        }

        // Clear remainder of line.
        if cur_col < (leftcol as i32 + cols as i32) {
            self.queue_colours(self.fore, self.back);
            queue!(self.writer, terminal::Clear(ClearType::UntilNewLine)).ok();
        }
    }
}

impl EmacsWindow for EmacsWindowCrossterm {
    fn get_columns(&self) -> MintCount {
        self.term_size().0 as MintCount
    }

    fn get_lines(&self) -> MintCount {
        // Reserve 3 rows: editing area uses (rows - 2) lines,
        // then the mode line and the message/prompt line.
        self.term_size().1.saturating_sub(3) as MintCount
    }

    fn redisplay(&mut self, buf: &mut EmacsBuffer, force: bool) {
        if !self.is_tty {
            return;
        }

        self.overwriting = false;

        let (cols, rows) = self.term_size();
        let edit_rows = rows.saturating_sub(2);

        queue!(self.writer, cursor::Hide).ok();

        if force {
            queue!(self.writer, terminal::Clear(ClearType::All)).ok();
        }

        buf.force_point_in_window(
            edit_rows as MintCount,
            cols as MintCount,
            self.top_scroll_percent,
            self.bot_scroll_percent,
        );

        let mut curline = buf.get_mark_position(crate::emacs_buffer::MARK_TOPLINE);
        let point = buf.get_mark_position(crate::emacs_buffer::MARK_POINT);
        let screen_line = buf.count_newlines(curline, point);
        let screen_col = buf.get_column() as i32 - buf.get_left_column() as i32;

        for i in 0..edit_rows {
            queue!(self.writer, cursor::MoveTo(0, i)).ok();
            let eol = buf.get_mark_position_from(crate::emacs_buffer::MARK_EOL, curline);
            self.write_line(buf, curline, eol);
            curline = buf.get_mark_position_from(crate::emacs_buffer::MARK_NEXT_CHAR, eol);
        }

        queue!(
            self.writer,
            cursor::MoveTo(screen_col as u16, screen_line as u16),
            cursor::Show,
        )
        .ok();
        self.writer.flush().ok();
    }

    fn overwrite(&mut self, s: &MintString) {
        if self.is_tty {
            if !self.overwriting {
                self.overwriting = true;
                self.ovy = 0;
                self.ovx = 0;
            }

            let (cols, rows) = self.term_size();
            self.queue_colours(self.fore, self.back);
            queue!(self.writer, cursor::MoveTo(self.ovx, self.ovy)).ok();

            for &ch in s.iter() {
                if ch == b'\n' {
                    queue!(self.writer, Print('\r')).ok();
                    self.ovx = 0;
                }
                queue!(self.writer, Print(ch as char)).ok();
                if ch == b'\n' {
                    self.ovy = min(self.ovy + 1, rows - 1);
                } else {
                    self.ovx += 1;
                }
                if self.ovx >= cols {
                    self.ovx = 0;
                    self.ovy = min(self.ovy + 1, rows - 1);
                }
            }
        } else {
            io::stdout().write_all(s).ok();
        }
    }

    fn gotoxy(&mut self, x: i32, y: i32) {
        if self.is_tty {
            let (cols, rows) = self.term_size();

            self.overwriting = true;
            self.ovx = max(0, min(x, cols as i32 - 1)) as u16;
            self.ovy = max(0, min(y, rows as i32 - 1)) as u16;
            queue!(self.writer, cursor::MoveTo(self.ovx, self.ovy)).ok();
        }
    }

    fn key_waiting(&self) -> bool {
        event::poll(Duration::ZERO).unwrap_or(false)
    }

    fn get_input(&mut self, millisec: MintCount) -> MintString {
        if self.is_tty {
            let timeout = if millisec < 10 {
                Duration::ZERO
            } else {
                Duration::from_millis(millisec as u64)
            };

            match event::poll(timeout) {
                Ok(true) => match event::read() {
                    Ok(Event::Key(ke)) => map_key_event(ke),
                    _ => b"Unknown".to_vec(),
                },
                _ => b"Timeout".to_vec(),
            }
        } else if millisec > 0 {
            let mut buf = [0u8; 1];
            if io::stdin().read(&mut buf).ok().unwrap_or(0) > 0 {
                vec![buf[0]]
            } else {
                b"Timeout".to_vec()
            }
        } else {
            b"Timeout".to_vec()
        }
    }

    fn announce(&mut self, left: &MintString, right: &MintString) {
        if self.is_tty {
            let (cols, rows) = self.term_size();
            let n = min(left.len(), cols as usize - 1);

            self.queue_colours(self.fore, self.back);
            queue!(self.writer, cursor::MoveTo(0, rows - 1)).ok();

            for &ch in left.iter().take(n) {
                if ch == b'\n' {
                    queue!(self.writer, Print('\r')).ok();
                }
                queue!(self.writer, Print(ch as char)).ok();
            }

            // Remember cursor position after left part (for cursor restore after announce).
            let cursor_x = n as u16;
            let cursor_y = rows - 1;

            let m = min(right.len(), (cols as usize).saturating_sub(n + 1));
            for &ch in right.iter().take(m) {
                if ch == b'\n' {
                    queue!(self.writer, Print('\r')).ok();
                }
                queue!(self.writer, Print(ch as char)).ok();
            }

            if (n + m) < cols as usize {
                queue!(self.writer, terminal::Clear(ClearType::UntilNewLine)).ok();
            }

            queue!(self.writer, cursor::MoveTo(cursor_x, cursor_y)).ok();
            self.writer.flush().ok();
        } else {
            io::stdout().write_all(left).ok();
            io::stdout().write_all(right).ok();
            println!();
        }
    }

    fn announce_win(&mut self, left: &MintString, right: &MintString) {
        if self.is_tty {
            let (cols, rows) = self.term_size();
            let n = min(left.len(), cols as usize - 1);

            // Save cursor position, write to mode line, then restore.
            let (saved_x, saved_y) = crossterm::cursor::position().unwrap_or((0, 0));

            self.queue_colours(self.fore, self.back);
            queue!(self.writer, cursor::MoveTo(0, rows - 2)).ok();

            for &ch in left.iter().take(n) {
                if ch == b'\n' {
                    queue!(self.writer, Print('\r')).ok();
                }
                queue!(self.writer, Print(ch as char)).ok();
            }

            let m = min(right.len(), cols as usize - n);
            for &ch in right.iter().take(m) {
                if ch == b'\n' {
                    queue!(self.writer, Print('\r')).ok();
                }
                queue!(self.writer, Print(ch as char)).ok();
            }

            if (n + m) < cols as usize {
                queue!(self.writer, terminal::Clear(ClearType::UntilNewLine)).ok();
            }

            queue!(self.writer, cursor::MoveTo(saved_x, saved_y)).ok();
            self.writer.flush().ok();
        }
    }

    fn audible_bell(&mut self, _freq: MintCount, _millisec: MintCount) {
        // Crossterm has no beep primitive — emit the ASCII BEL character.
        queue!(self.writer, Print('\x07')).ok();
        self.writer.flush().ok();
    }

    fn visual_bell(&mut self, _millisec: MintCount) {
        // Brief colour inversion to simulate a flash.
        if self.is_tty {
            queue!(
                self.writer,
                SetColors(Colors::new(ansi_colour(self.back), ansi_colour(self.fore)))
            )
            .ok();
            self.writer.flush().ok();
            std::thread::sleep(Duration::from_millis(50));
            self.queue_colours(self.fore, self.back);
            self.writer.flush().ok();
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

impl Drop for EmacsWindowCrossterm {
    fn drop(&mut self) {
        if self.is_tty {
            execute!(
                self.writer,
                cursor::Show,
                terminal::LeaveAlternateScreen,
            )
            .ok();
            terminal::disable_raw_mode().ok();
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Map a 0-15 DOS/ANSI colour index to a crossterm `Color`.
///
/// The low 3 bits select the hue (matching the classic CGA/EGA colour order),
/// and bit 3 selects bright/bold versus dark.
fn ansi_colour(colour: i32) -> Color {
    // Crossterm's `Color::AnsiValue` maps exactly to the standard 16-colour
    // ANSI palette (indices 0-15), so we can pass the value through directly
    // after clamping to the valid range.
    Color::AnsiValue((colour & 0x0F) as u8)
}

/// Translate a crossterm `KeyEvent` into the `MintString` token that the
/// editor expects (matching the key names used in the ncurses implementation).
fn map_key_event(ke: KeyEvent) -> MintString {
    // Ignore key-release and key-repeat events emitted by some terminals
    // in the "enhanced keyboard" mode.
    if ke.kind != KeyEventKind::Press {
        return b"Unknown".to_vec();
    }

    let ctrl = ke.modifiers.contains(KeyModifiers::CONTROL);
    let shift = ke.modifiers.contains(KeyModifiers::SHIFT);

    match ke.code {
        // Control characters
        KeyCode::Char('@') if ctrl => b"C-@".to_vec(),
        KeyCode::Char(c) if ctrl => format!("C-{}", c.to_ascii_lowercase()).into_bytes(),

        // Characters with special names
        KeyCode::Char(',') => b"Comma".to_vec(),
        KeyCode::Char('(') => b"LPar".to_vec(),
        KeyCode::Char(')') => b"RPar".to_vec(),

        // Printable characters
        KeyCode::Char(c) => vec![c as u8],

        // Named keys
        KeyCode::Backspace => b"Back Space".to_vec(),
        KeyCode::Tab | KeyCode::BackTab => b"Tab".to_vec(),
        KeyCode::Enter => b"Return".to_vec(),
        KeyCode::Esc => b"Escape".to_vec(),
        KeyCode::Delete => b"Del".to_vec(),
        KeyCode::Insert => b"Ins".to_vec(),
        KeyCode::Up => b"Up Arrow".to_vec(),
        KeyCode::Down => b"Down Arrow".to_vec(),
        KeyCode::Left => b"Left Arrow".to_vec(),
        KeyCode::Right => b"Right Arrow".to_vec(),
        KeyCode::Home => b"Home".to_vec(),
        KeyCode::End => b"End".to_vec(),
        KeyCode::PageUp => b"Pg Up".to_vec(),
        KeyCode::PageDown => b"Pg Dn".to_vec(),

        // Function keys (shifted variants use S-Fn naming)
        KeyCode::F(n) if shift => format!("S-F{}", n).into_bytes(),
        KeyCode::F(n) => format!("F{}", n).into_bytes(),

        _ => b"Unknown".to_vec(),
    }
}

// Bring Read into scope for the non-tty stdin fallback in get_input.
use std::io::Read;
