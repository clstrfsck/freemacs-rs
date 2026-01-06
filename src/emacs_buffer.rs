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

use crate::buffer::Buffer;
use crate::mint_types::{MintChar, MintCount, MintString};
use regex::bytes::Regex;
use std::cmp::{max, min};

pub const EOLCHAR: MintChar = b'\n';

/* Mark constants */
pub const MARK_FIRST_TEMP: MintChar = b'0';
pub const MARK_MAX_TEMP: usize = 10;
pub const MARK_LAST_TEMP: MintChar = MARK_FIRST_TEMP + MARK_MAX_TEMP as MintChar - 1;
pub const MARK_FIRST_PERM: MintChar = b'@';
pub const MARK_MAX_PERM: usize = 27;
pub const MARK_LAST_PERM: MintChar = MARK_FIRST_PERM + MARK_MAX_PERM as MintChar - 1;
pub const MARK_PREV_CHAR: MintChar = b'<';
pub const MARK_NEXT_CHAR: MintChar = b'>';
pub const MARK_BOB: MintChar = b'[';
pub const MARK_EOB: MintChar = b']';
pub const MARK_BOL: MintChar = b'^';
pub const MARK_EOL: MintChar = b'$';
pub const MARK_PREV_BLANK: MintChar = b'-';
pub const MARK_NEXT_BLANK: MintChar = b'+';
pub const MARK_PREV_NBLANK: MintChar = b'{';
pub const MARK_NEXT_NBLANK: MintChar = b'}';
pub const MARK_POINT: MintChar = b'.';
pub const MARK_TOPLINE: MintChar = b'!';

const MAX_MARKS: usize = 50;

pub struct EmacsBuffer {
    wp: bool,
    modified: bool,
    point: MintCount,
    topline: MintCount,
    leftcol: MintCount,
    tab_width: MintCount,
    temp_mark_base: usize,
    temp_mark_last: usize,
    perm_mark_count: usize,
    marks_sp: usize,
    marks: Vec<MintCount>,
    mark_stack: Vec<usize>,
    point_line: MintCount,
    topline_line: MintCount,
    count_newlines: MintCount,
    bufno: MintCount,
    text: Box<dyn Buffer>,
}

impl EmacsBuffer {
    pub fn new(bufno: MintCount, text: Box<dyn Buffer>) -> Self {
        Self {
            wp: false,
            modified: false,
            point: 0,
            topline: 0,
            leftcol: 0,
            tab_width: 8,
            temp_mark_base: 1,
            temp_mark_last: 1,
            perm_mark_count: 1,
            marks_sp: 0,
            marks: vec![0; MAX_MARKS],
            mark_stack: vec![0; MAX_MARKS],
            point_line: 0,
            topline_line: 0,
            count_newlines: 0,
            bufno,
            text,
        }
    }

    pub fn is_write_protected(&self) -> bool {
        self.wp
    }

    pub fn set_write_protected(&mut self, iswp: bool) {
        self.wp = iswp;
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    pub fn set_modified(&mut self, ismodified: bool) {
        self.modified = ismodified;
    }

    pub fn insert_string(&mut self, s: &MintString) -> bool {
        if self.wp {
            return false;
        }

        if !self.text.insert(self.point, s) {
            return false;
        }

        let newline_count = s.iter().filter(|&&ch| ch == EOLCHAR).count() as MintCount;

        self.adjust_marks_ins(s.len() as MintCount);
        self.point += s.len() as MintCount;
        self.point_line += newline_count;
        self.count_newlines += newline_count;
        self.modified = true;

        true
    }

    pub fn push_temp_marks(&mut self, n: MintCount) -> bool {
        let n = n as usize;
        if (self.temp_mark_last + n) <= MAX_MARKS {
            self.mark_stack[self.marks_sp] = self.temp_mark_base;
            self.marks_sp += 1;
            self.temp_mark_base = self.temp_mark_last;
            self.temp_mark_last = self.temp_mark_base + n;

            for i in 0..n {
                self.marks[self.temp_mark_base + i] = self.point;
            }
            true
        } else {
            false
        }
    }

    pub fn pop_temp_marks(&mut self) -> bool {
        if self.marks_sp > 0 {
            self.temp_mark_last = self.temp_mark_base;
            self.marks_sp -= 1;
            self.temp_mark_base = self.mark_stack[self.marks_sp];
            true
        } else {
            false
        }
    }

    pub fn create_perm_marks(&mut self, n: MintCount) -> bool {
        let n = n as usize;
        if n <= MARK_MAX_PERM {
            self.perm_mark_count = n;
            self.temp_mark_base = n;
            self.temp_mark_last = n;
            self.marks_sp = 0;
            true
        } else {
            false
        }
    }

    pub fn set_mark(&mut self, mark: MintChar, dest_mark: MintChar) -> bool {
        let dest_pos = self.get_mark_position(dest_mark);
        self.set_mark_position(mark, dest_pos)
    }

    pub fn delete_to_marks(&mut self, marks: &MintString) -> bool {
        for &mark in marks {
            if !self.delete_to_mark(mark) {
                return false;
            }
        }
        true
    }

    fn delete_to_mark(&mut self, mark: MintChar) -> bool {
        if self.wp {
            return false;
        }

        let mark_pos = self.get_mark_position(mark);
        let min_pos = min(mark_pos, self.point);
        let max_pos = max(mark_pos, self.point);
        let delete_len = max_pos - min_pos;

        if delete_len == 0 {
            return true;
        }

        let newline_count = self.count_newlines(min_pos, max_pos);

        if !self.text.erase(min_pos, delete_len) {
            return false;
        }

        self.point = min_pos;
        self.adjust_marks_del(delete_len);

        if mark_pos < self.point {
            self.point_line -= newline_count;
        }
        self.count_newlines -= newline_count;
        self.modified = true;

        true
    }

    pub fn read_to_mark(&self, mark: MintChar) -> MintString {
        self.read_to_mark_from(mark, self.point)
    }

    pub fn read_to_mark_from(&self, mark: MintChar, frompos: MintCount) -> MintString {
        self.read(frompos, self.get_mark_position_from(mark, frompos))
    }

    pub fn read(&self, from_pos: MintCount, to_pos: MintCount) -> MintString {
        let min_pos = min(from_pos, to_pos);
        let max_pos = max(from_pos, to_pos);

        let mut result = Vec::new();
        for i in min_pos..max_pos {
            if let Some(ch) = self.text.get(i) {
                result.push(ch);
            }
        }
        result
    }

    pub fn translate(&mut self, mark: MintChar, trstr: &MintString) -> bool {
        if self.wp || trstr.len() < 2 {
            return false;
        }

        let half = trstr.len() / 2;
        let from_str = &trstr[0..half];
        let to_str = &trstr[half..];

        let mark_pos = self.get_mark_position(mark);
        let min_pos = min(mark_pos, self.point);
        let max_pos = max(mark_pos, self.point);

        let mut changed = false;
        for pos in min_pos..max_pos {
            if let Some(ch) = self.text.get(pos)
                && let Some(idx) = from_str.iter().position(|&c| c == ch)
                && idx < to_str.len()
            {
                let replacement = vec![to_str[idx]];
                self.text.replace(pos, 1, &replacement);
                changed = true;
            }
        }

        if changed {
            self.modified = true;
        }
        changed
    }

    pub fn chars_to_mark(&self, mark: MintChar) -> MintCount {
        let mark_pos = self.get_mark_position(mark);
        let min_pos = min(mark_pos, self.point);
        let max_pos = max(mark_pos, self.point);
        max_pos - min_pos
    }

    pub fn mark_before_point(&self, mark: MintChar) -> bool {
        self.get_mark_position(mark) < self.point
    }

    pub fn get_buf_number(&self) -> MintCount {
        self.bufno
    }

    pub fn size(&self) -> MintCount {
        self.text.size() as MintCount
    }

    pub fn set_mark_position(&mut self, mark: MintChar, position: MintCount) -> bool {
        let adjusted_pos = min(self.text.size() as MintCount, position);

        if mark >= MARK_FIRST_TEMP {
            let temp_markno = (mark - MARK_FIRST_TEMP) as usize;
            if (self.temp_mark_base + temp_markno) < self.temp_mark_last {
                self.marks[self.temp_mark_base + temp_markno] = adjusted_pos;
                return true;
            }
        }

        if mark >= MARK_FIRST_PERM {
            let perm_markno = (mark - MARK_FIRST_PERM) as usize;
            if perm_markno < self.perm_mark_count {
                self.marks[perm_markno] = adjusted_pos;
                return true;
            }
        }

        false
    }

    pub fn get_mark_position(&self, mark: MintChar) -> MintCount {
        self.get_mark_position_from(mark, self.point)
    }

    pub fn get_mark_position_from(&self, mark: MintChar, frompos: MintCount) -> MintCount {
        match mark {
            MARK_POINT => self.point,
            MARK_BOB => 0,
            MARK_EOB => self.text.size() as MintCount,
            MARK_TOPLINE => self.topline,
            MARK_BOL => self.find_bol(frompos),
            MARK_EOL => self.find_eol(frompos),
            MARK_PREV_CHAR => {
                if frompos > 0 {
                    frompos - 1
                } else {
                    0
                }
            }
            MARK_NEXT_CHAR => {
                let size = self.text.size() as MintCount;
                if frompos < size { frompos + 1 } else { size }
            }
            MARK_PREV_BLANK => self.find_prev_blank(frompos),
            MARK_NEXT_BLANK => self.find_next_blank(frompos),
            MARK_PREV_NBLANK => self.find_prev_nblank(frompos),
            MARK_NEXT_NBLANK => self.find_next_nblank(frompos),
            _ => {
                if (MARK_FIRST_TEMP..=MARK_LAST_TEMP).contains(&mark) {
                    let temp_markno = (mark - MARK_FIRST_TEMP) as usize;
                    if (self.temp_mark_base + temp_markno) < self.temp_mark_last {
                        return self.marks[self.temp_mark_base + temp_markno];
                    }
                }
                if (MARK_FIRST_PERM..=MARK_LAST_PERM).contains(&mark) {
                    let perm_markno = (mark - MARK_FIRST_PERM) as usize;
                    if perm_markno < self.perm_mark_count {
                        return self.marks[perm_markno];
                    }
                }
                frompos
            }
        }
    }

    pub fn get_point_line(&self) -> MintCount {
        self.point_line
    }

    pub fn count_newlines_total(&self) -> MintCount {
        self.count_newlines
    }

    pub fn set_point_line(&mut self, lno: MintCount) {
        if lno <= self.point_line {
            let lines_back = self.point_line - lno;
            self.point = self.backward_lines(self.point, lines_back);
        } else {
            let lines_forward = lno - self.point_line;
            self.point = self.forward_lines(self.point, lines_forward);
        }
        self.point_line = lno;
    }

    pub fn get_column(&self) -> MintCount {
        let bol = self.find_bol(self.point);
        self.count_columns(bol, self.point)
    }

    pub fn set_column(&mut self, col: MintCount) {
        let bol = self.find_bol(self.point);
        let eol = self.find_eol(self.point);
        let mut cur_col = 0;
        let mut pos = bol;

        while pos < eol && cur_col < col {
            if let Some(ch) = self.text.get(pos) {
                cur_col += self.char_width(cur_col, ch);
                pos += 1;
            } else {
                break;
            }
        }
        self.point = pos;
    }

    pub fn count_newlines(&self, from: MintCount, to: MintCount) -> MintCount {
        let mut count = 0;
        for i in from..to {
            if let Some(ch) = self.text.get(i)
                && ch == EOLCHAR
            {
                count += 1;
            }
        }
        count
    }

    pub fn count_columns(&self, from: MintCount, to: MintCount) -> MintCount {
        let mut col = 0;
        for i in from..to {
            if let Some(ch) = self.text.get(i) {
                col += self.char_width(col, ch);
            }
        }
        col
    }

    pub fn get_left_column(&self) -> MintCount {
        self.leftcol
    }

    pub fn set_tab_width(&mut self, n: MintCount) {
        self.tab_width = n;
    }

    pub fn get_tab_width(&self) -> MintCount {
        self.tab_width
    }

    pub fn char_width(&self, cur_col: MintCount, ch: MintChar) -> MintCount {
        if ch == b'\t' {
            self.tab_width - (cur_col % self.tab_width)
        } else if !(32..127).contains(&ch) {
            2
        } else {
            1
        }
    }

    pub fn force_point_in_window(
        &mut self,
        li: MintCount,
        _co: MintCount,
        tp: MintCount,
        bp: MintCount,
    ) {
        let tl = li * tp / 100;
        if self.point_line <= tl {
            self.topline = 0;
            self.topline_line = 0;
        } else {
            let bl = li * bp / 100;
            if self.point_line >= self.count_newlines - bl {
                let size = self.text.size() as MintCount;
                self.topline = self.backward_lines(self.find_bol(size), li - 1);
                self.topline_line = self.count_newlines - (li - 1);
            } else if self.point_line < (self.topline_line + tl) {
                let blines = (self.topline_line + tl) - self.point_line;
                self.topline = self.backward_lines(self.topline, blines);
                self.topline_line -= blines;
            } else if self.point_line >= (self.topline_line + (li - bl)) {
                let flines = self.point_line - (self.topline_line + (li - bl));
                self.topline = self.forward_lines(self.topline, flines);
                self.topline_line += flines;
            }
        }
    }

    pub fn set_point_row(&mut self, li: MintCount) {
        if self.point_line <= li {
            /* Not enough lines to have point at row 'li' */
            self.topline = 0;
            self.topline_line = 0;
        } else {
            self.topline = self.backward_lines(self.get_mark_position(MARK_BOL), li);
            self.topline_line = self.count_newlines(self.get_mark_position(MARK_BOL), self.topline);
        }
    }

    pub fn get_point_row(&self) -> MintCount {
        self.point_line - self.topline_line
    }

    fn adjust_marks_ins(&mut self, n: MintCount) {
        for i in 0..MAX_MARKS {
            if self.marks[i] > self.point {
                self.marks[i] += n;
            }
        }
        self.topline = if self.topline > self.point {
            self.topline + n
        } else {
            self.topline
        };
    }

    fn adjust_marks_del(&mut self, n: MintCount) {
        for i in 0..MAX_MARKS {
            if self.marks[i] > self.point {
                self.marks[i] = self.marks[i].saturating_sub(n);
            }
        }
        if self.topline > self.point {
            self.topline = self.topline.saturating_sub(n);
        }
    }

    fn find_bol(&self, frompos: MintCount) -> MintCount {
        let mut pos = frompos;
        while pos > 0 {
            pos -= 1;
            if let Some(ch) = self.text.get(pos)
                && ch == EOLCHAR
            {
                return pos + 1;
            }
        }
        0
    }

    fn find_eol(&self, frompos: MintCount) -> MintCount {
        let size = self.text.size() as MintCount;
        let mut pos = frompos;
        while pos < size {
            if let Some(ch) = self.text.get(pos)
                && ch == EOLCHAR
            {
                return pos;
            }
            pos += 1;
        }
        size
    }

    fn find_prev_blank(&self, frompos: MintCount) -> MintCount {
        let mut pos = frompos;
        while pos > 0 {
            pos -= 1;
            if let Some(ch) = self.text.get(pos)
                && ch.is_ascii_whitespace()
            {
                return pos;
            }
        }
        0
    }

    fn find_next_blank(&self, frompos: MintCount) -> MintCount {
        let size = self.text.size() as MintCount;
        let mut pos = frompos;
        while pos < size {
            if let Some(ch) = self.text.get(pos)
                && ch.is_ascii_whitespace()
            {
                return pos;
            }
            pos += 1;
        }
        size
    }

    fn find_prev_nblank(&self, frompos: MintCount) -> MintCount {
        let mut pos = frompos;
        while pos > 0 {
            pos -= 1;
            if let Some(ch) = self.text.get(pos)
                && !ch.is_ascii_whitespace()
            {
                return pos;
            }
        }
        0
    }

    fn find_next_nblank(&self, frompos: MintCount) -> MintCount {
        let size = self.text.size() as MintCount;
        let mut pos = frompos;
        while pos < size {
            if let Some(ch) = self.text.get(pos)
                && !ch.is_ascii_whitespace()
            {
                return pos;
            }
            pos += 1;
        }
        size
    }

    fn forward_lines(&self, pos: MintCount, lines: MintCount) -> MintCount {
        let mut current_pos = pos;
        for _ in 0..lines {
            current_pos = self.get_mark_position_from(MARK_EOL, current_pos);
            current_pos = self.get_mark_position_from(MARK_NEXT_CHAR, current_pos);
        }
        current_pos
    }

    fn backward_lines(&self, pos: MintCount, lines: MintCount) -> MintCount {
        let mut current_pos = pos;
        for _ in 0..lines {
            current_pos = self.get_mark_position_from(MARK_PREV_CHAR, current_pos);
            current_pos = self.get_mark_position_from(MARK_BOL, current_pos);
        }
        current_pos
    }

    pub fn set_point_to_mark(&mut self, mark: MintChar) {
        self.point = self.get_mark_position(mark);
        self.point_line = self.count_newlines(0, self.point);
    }

    pub fn set_point_to_marks(&mut self, marks: &MintString) {
        for &mark in marks {
            self.set_point_to_mark(mark);
        }
    }

    pub fn find_forward(
        &self,
        regex: &Regex,
        start: MintCount,
        end: MintCount,
    ) -> Option<(MintCount, MintCount)> {
        self.text.find_forward(regex, start, end)
    }

    pub fn find_backward(
        &self,
        regex: &Regex,
        start: MintCount,
        end: MintCount,
    ) -> Option<(MintCount, MintCount)> {
        self.text.find_backward(regex, start, end)
    }
}
