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
use crate::mint_types::{MintChar, MintString};
use regex::bytes::Regex;
use std::borrow::Cow;

const BLOCK_SIZE: usize = 65536;

#[derive(Debug)]
pub struct GapBuffer {
    bottop: usize,
    topbot: usize,
    buffer: Vec<MintChar>,
}

impl GapBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            bottop: 0,
            topbot: size,
            buffer: vec![0; size],
        }
    }

    pub fn with_default_size() -> Self {
        Self::new(BLOCK_SIZE)
    }

    fn move_gap_to(&mut self, offset: usize) -> bool {
        if offset == self.bottop {
            return true;
        }
        if offset > self.size() {
            return false;
        }

        if offset < self.bottop {
            let move_size = self.bottop - offset;
            self.buffer
                .copy_within(offset..offset + move_size, self.topbot - move_size);
            self.bottop -= move_size;
            self.topbot -= move_size;
        } else {
            let move_size = offset - self.bottop;
            self.buffer
                .copy_within(self.topbot..self.topbot + move_size, self.bottop);
            self.bottop += move_size;
            self.topbot += move_size;
        }
        true // offset - (offset - self.bottop) = self.
    }

    fn expand(&mut self, extra_space: usize) {
        if extra_space > 0 {
            let additional_blocks = (extra_space + BLOCK_SIZE) / BLOCK_SIZE;
            let new_size = self.allocated() + additional_blocks * BLOCK_SIZE;
            if new_size > self.allocated() {
                self.move_gap_to(self.size());
                self.buffer.resize(new_size, 0);
                self.topbot = new_size;
            }
        }
    }

    // FIXME: Move to GapBuffer impl
    fn slice<'a>(&'a self, start: usize, end: usize) -> Cow<'a, [MintChar]> {
        if start >= end {
            return Cow::Borrowed(&[]);
        }

        // Entirely in top contiguous region
        if end <= self.bottop {
            return Cow::Borrowed(&self.buffer[start..end]);
        }

        // Entirely in bottom contiguous region (adjust for gap)
        if start >= self.bottop {
            let actual_start = start + self.free();
            let actual_end = actual_start + (end - start);
            return Cow::Borrowed(&self.buffer[actual_start..actual_end]);
        }

        // FIXME: Spans the gap: quick and dirty implementation.
        // Optimize later. Ideally this would efficiently move the gap out of
        // the way and always return a slice directly.
        // Even better would be regex support for gap-spanning searches without
        // moving the gap.
        let mut v = Vec::with_capacity(end - start);
        for i in start..end {
            if let Some(ch) = self.get(i) {
                v.push(ch);
            }
        }
        Cow::Owned(v)
    }
}

impl Buffer for GapBuffer {
    fn free(&self) -> usize {
        self.topbot - self.bottop
    }

    fn allocated(&self) -> usize {
        self.buffer.len()
    }

    fn size(&self) -> usize {
        self.allocated() - self.free()
    }

    fn get(&self, offset: usize) -> Option<MintChar> {
        if offset >= self.size() {
            return None;
        }
        let actual_offset = if offset >= self.bottop {
            offset + self.free()
        } else {
            offset
        };
        Some(self.buffer[actual_offset])
    }

    fn replace(&mut self, offset: usize, n: usize, replacement: &MintString) -> bool {
        self.erase(offset, n) && self.insert(offset, replacement)
    }

    fn erase(&mut self, offset: usize, n: usize) -> bool {
        if self.size() >= offset && self.size() - offset >= n && self.move_gap_to(offset + n) {
            self.bottop -= n;
            true
        } else {
            false
        }
    }

    fn insert(&mut self, offset: usize, to_insert: &MintString) -> bool {
        let insert_size = to_insert.len();
        if self.free() < insert_size {
            self.expand(insert_size - self.free());
        }
        if self.free() >= insert_size && self.move_gap_to(offset) {
            self.buffer[self.bottop..self.bottop + insert_size].copy_from_slice(to_insert);
            self.bottop += insert_size;
            true
        } else {
            false
        }
    }

    fn find_forward(&self, regex: &Regex, start: usize, end: usize) -> Option<(usize, usize)> {
        let slice = self.slice(start, end);
        regex
            .find(&slice)
            .map(|matched| (start + matched.start(), start + matched.end()))
    }

    fn find_backward(&self, regex: &Regex, start: usize, end: usize) -> Option<(usize, usize)> {
        let slice = self.slice(start, end);
        regex
            .find_iter(&slice)
            .last()
            .map(|matched| (start + matched.start(), start + matched.end()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_ms(s: &str) -> Vec<u8> {
        s.bytes().collect()
    }

    fn to_string<T: Buffer>(buf: &T) -> String {
        let mut ms: MintString = Vec::new();
        for i in 0..buf.size() {
            ms.push(buf.get(i).unwrap());
        }
        String::from_utf8(ms).unwrap()
    }

    // fn print_buffer<T: Buffer>(buf: &T) {
    //     let s = to_string(buf);
    //     println!("{}", s);
    // }

    #[test]
    fn gap_buffer_basic_construction() {
        let gb = GapBuffer::with_default_size();
        assert_eq!(65536, gb.allocated());
        assert_eq!(65536, gb.free());
        assert_eq!(0, gb.size());
    }

    #[test]
    fn gap_buffer_basic_insert() {
        let mut gb = GapBuffer::with_default_size();
        assert!(gb.insert(0, &to_ms("0123456789")));
        assert_eq!(65536, gb.allocated());
        assert_eq!(65526, gb.free());
        assert_eq!(10, gb.size());
        assert_eq!("0123456789", to_string(&gb));
    }

    #[test]
    fn gap_buffer_basic_erase() {
        let mut gb = GapBuffer::with_default_size();
        assert!(gb.insert(0, &to_ms("0123456789")));
        assert!(gb.erase(0, 1));
        assert_eq!(65536, gb.allocated());
        assert_eq!(65527, gb.free());
        assert_eq!(9, gb.size());
        assert_eq!("123456789", to_string(&gb));
    }

    #[test]
    fn gap_buffer_erase_nonexistent_returns_false() {
        let mut gb = GapBuffer::with_default_size();
        assert!(!gb.erase(0, 1));
    }

    #[test]
    fn gap_buffer_insert_at_end() {
        let mut gb = GapBuffer::with_default_size();
        assert!(gb.insert(0, &to_ms("0123456789")));
        assert!(gb.insert(10, &to_ms("ABCDEFGHIJ")));
        assert_eq!(65536, gb.allocated());
        assert_eq!(65516, gb.free());
        assert_eq!(20, gb.size());
        assert_eq!("0123456789ABCDEFGHIJ", to_string(&gb));
    }

    #[test]
    fn gap_buffer_insert_at_begin() {
        let mut gb = GapBuffer::with_default_size();
        assert!(gb.insert(0, &to_ms("0123456789")));
        assert!(gb.insert(0, &to_ms("ABCDEFGHIJ")));
        assert_eq!(65536, gb.allocated());
        assert_eq!(65516, gb.free());
        assert_eq!(20, gb.size());
        assert_eq!("ABCDEFGHIJ0123456789", to_string(&gb));
    }

    #[test]
    fn gap_buffer_insert_in_middle() {
        let mut gb = GapBuffer::with_default_size();
        assert!(gb.insert(0, &to_ms("0123456789")));
        assert!(gb.insert(5, &to_ms("ABCDEFGHIJ")));
        assert_eq!(65536, gb.allocated());
        assert_eq!(65516, gb.free());
        assert_eq!(20, gb.size());
        assert_eq!("01234ABCDEFGHIJ56789", to_string(&gb));
    }

    #[test]
    fn gap_buffer_insert_off_end() {
        let mut gb = GapBuffer::with_default_size();
        assert!(gb.insert(0, &to_ms("0123456789")));
        assert!(!gb.insert(20, &to_ms("ABCDEFGHIJ")));
        assert_eq!(65536, gb.allocated());
        assert_eq!(65526, gb.free());
        assert_eq!(10, gb.size());
        assert_eq!("0123456789", to_string(&gb));
    }

    #[test]
    fn gap_buffer_insert_move_forward() {
        let mut gb = GapBuffer::with_default_size();
        assert!(gb.insert(0, &to_ms("56789")));
        assert!(gb.insert(0, &to_ms("01234")));
        assert!(gb.insert(10, &to_ms("ABCDEFGHIJ")));
        assert_eq!(65536, gb.allocated());
        assert_eq!(65516, gb.free());
        assert_eq!(20, gb.size());
        assert_eq!("0123456789ABCDEFGHIJ", to_string(&gb));
    }

    #[test]
    fn gap_buffer_insert_resize() {
        let mut gb = GapBuffer::new(5);
        assert_eq!(5, gb.allocated());
        assert_eq!(5, gb.free());
        assert_eq!(0, gb.size());
        assert!(gb.insert(0, &to_ms("0123456789")));
        assert_eq!(65541, gb.allocated());
        assert_eq!(65531, gb.free());
        assert_eq!(10, gb.size());
        assert_eq!("0123456789", to_string(&gb));
    }

    #[test]
    fn gap_buffer_get_nonexistent_returns_none() {
        let gb = GapBuffer::with_default_size();
        assert_eq!(None, gb.get(0));
    }

    #[test]
    fn gap_buffer_replace_basic() {
        let mut gb = GapBuffer::with_default_size();
        assert!(gb.insert(0, &to_ms("0123456789")));
        assert!(gb.replace(0, 5, &to_ms("ABCDE")));
        assert_eq!(65536, gb.allocated());
        assert_eq!(65526, gb.free());
        assert_eq!(10, gb.size());
        assert_eq!("ABCDE56789", to_string(&gb));
    }

    #[test]
    fn gap_buffer_replace_shorter() {
        let mut gb = GapBuffer::with_default_size();
        assert!(gb.insert(0, &to_ms("0123456789")));
        assert!(gb.replace(0, 5, &to_ms("A")));
        assert_eq!(65536, gb.allocated());
        assert_eq!(65530, gb.free());
        assert_eq!(6, gb.size());
        assert_eq!("A56789", to_string(&gb));
    }

    #[test]
    fn gap_buffer_replace_longer() {
        let mut gb = GapBuffer::with_default_size();
        assert!(gb.insert(0, &to_ms("0123456789")));
        assert!(gb.replace(0, 5, &to_ms("ABCDEFG")));
        assert_eq!(65536, gb.allocated());
        assert_eq!(65524, gb.free());
        assert_eq!(12, gb.size());
        assert_eq!("ABCDEFG56789", to_string(&gb));
    }

    #[test]
    fn gap_buffer_replace_off_end_fails() {
        let mut gb = GapBuffer::with_default_size();
        assert!(!gb.replace(5, 5, &to_ms("ABCDE")));
    }

    #[test]
    fn gap_buffer_find_forward_basic() {
        let mut gb = GapBuffer::with_default_size();
        assert!(gb.insert(0, &to_ms("01234567890123456789")));
        let re = Regex::new("345").unwrap();
        let result = gb.find_forward(&re, 0, gb.size());
        assert_eq!(Some((3, 6)), result);
    }

    #[test]
    fn gap_buffer_find_backward_basic() {
        let mut gb = GapBuffer::with_default_size();
        assert!(gb.insert(0, &to_ms("01234567890123456789")));
        let re = Regex::new("345").unwrap();
        let result = gb.find_backward(&re, 0, gb.size());
        assert_eq!(Some((13, 16)), result);
    }

    #[test]
    fn gap_buffer_find_forward_no_match() {
        let mut gb = GapBuffer::with_default_size();
        assert!(gb.insert(0, &to_ms("01234567890123456789")));
        let re = Regex::new("XYZ").unwrap();
        let result = gb.find_forward(&re, 0, gb.size());
        assert_eq!(None, result);
    }

    #[test]
    fn gap_buffer_find_backward_no_match() {
        let mut gb = GapBuffer::with_default_size();
        assert!(gb.insert(0, &to_ms("01234567890123456789")));
        let re = Regex::new("XYZ").unwrap();
        let result = gb.find_backward(&re, 0, gb.size());
        assert_eq!(None, result);
    }

    #[test]
    fn gap_buffer_find_forward_partial_range() {
        let mut gb = GapBuffer::with_default_size();
        assert!(gb.insert(0, &to_ms("01234567890123456789")));
        let re = Regex::new("345").unwrap();
        let result = gb.find_forward(&re, 5, gb.size());
        assert_eq!(Some((13, 16)), result);
    }

    #[test]
    fn gap_buffer_find_backward_partial_range() {
        let mut gb = GapBuffer::with_default_size();
        assert!(gb.insert(0, &to_ms("01234567890123456789")));
        let re = Regex::new("345").unwrap();
        let result = gb.find_backward(&re, 0, 15);
        assert_eq!(Some((3, 6)), result);
    }

    #[test]
    fn gap_buffer_find_forward_empty_range() {
        let mut gb = GapBuffer::with_default_size();
        assert!(gb.insert(0, &to_ms("01234567890123456789")));
        let re = Regex::new("345").unwrap();
        let result = gb.find_forward(&re, 5, 5);
        assert_eq!(None, result);
    }

    #[test]
    fn gap_buffer_find_backward_empty_range() {
        let mut gb = GapBuffer::with_default_size();
        assert!(gb.insert(0, &to_ms("01234567890123456789")));
        let re = Regex::new("345").unwrap();
        let result = gb.find_backward(&re, 5, 5);
        assert_eq!(None, result);
    }

    #[test]
    fn gap_buffer_find_forward_across_gap() {
        let mut gb = GapBuffer::with_default_size();
        assert!(gb.insert(0, &to_ms("0123456789")));
        assert!(gb.insert(5, &to_ms("ABCDEFGHIJ")));
        let re = Regex::new("34AB").unwrap();
        let result = gb.find_forward(&re, 0, gb.size());
        assert_eq!(Some((3, 7)), result);
    }

    #[test]
    fn gap_buffer_find_backward_across_gap() {
        let mut gb = GapBuffer::with_default_size();
        assert!(gb.insert(0, &to_ms("0123456789")));
        assert!(gb.insert(5, &to_ms("ABCDEFGHIJ")));
        let re = Regex::new("34AB").unwrap();
        let result = gb.find_backward(&re, 0, gb.size());
        assert_eq!(Some((3, 7)), result);
    }

    #[test]
    fn gap_buffer_find_forward_bottom_only() {
        let mut gb = GapBuffer::with_default_size();
        assert!(gb.insert(0, &to_ms("0123456789")));
        assert!(gb.insert(0, &to_ms("A")));
        let re = Regex::new("89").unwrap();
        let result = gb.find_forward(&re, 1, gb.size());
        assert_eq!(Some((9, 11)), result);
    }
}
