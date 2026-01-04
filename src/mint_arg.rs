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

use crate::mint_types::{MintChar, MintString};
use std::collections::VecDeque;
use std::collections::vec_deque::{IntoIter, Iter};
use std::ops::Index;

const ARG_END: &MintArg = &MintArg {
    arg_type: ArgType::End,
    value: Vec::new(),
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgType {
    Null = 0x80,
    Arg = 0x01,
    Active = 0x82,
    Neutral = 0x83,
    End = 0x04,
}

impl ArgType {
    pub fn is_term(self) -> bool {
        (self as u8) & 0x80 != 0
    }
}

#[derive(Debug, Clone)]
pub struct MintArg {
    arg_type: ArgType,
    value: MintString,
}

impl MintArg {
    pub fn new(arg_type: ArgType) -> Self {
        Self {
            arg_type,
            value: Vec::new(),
        }
    }

    pub fn append(&mut self, ch: MintChar) {
        self.value.push(ch);
    }

    pub fn append_slice(&mut self, s: &[MintChar]) {
        self.value.extend_from_slice(s);
    }

    pub fn arg_type(&self) -> ArgType {
        self.arg_type
    }

    pub fn is_term(&self) -> bool {
        self.arg_type.is_term()
    }

    pub fn value(&self) -> &MintString {
        &self.value
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    pub fn get_int_value(&self, base: i32) -> i32 {
        crate::mint_string::get_int_value(&self.value, base)
    }

    pub fn get_int_prefix(&self, base: i32) -> MintString {
        crate::mint_string::get_int_prefix(&self.value, base)
    }

    pub fn get_first_char(&self) -> Option<MintChar> {
        self.value.first().copied()
    }
}

#[derive(Debug, Clone)]
pub struct MintArgList {
    args: VecDeque<MintArg>,
}

impl MintArgList {
    pub fn new() -> Self {
        Self {
            args: VecDeque::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.args.len()
    }

    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }

    pub fn push_front(&mut self, arg: MintArg) {
        self.args.push_front(arg);
    }

    pub fn iter(&'_ self) -> Iter<'_, MintArg> {
        self.args.iter()
    }
}

impl Default for MintArgList {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<usize> for MintArgList {
    type Output = MintArg;

    fn index(&self, index: usize) -> &Self::Output {
        if index < self.args.len() {
            &self.args[index]
        } else {
            ARG_END
        }
    }
}

impl IntoIterator for MintArgList {
    type Item = MintArg;
    type IntoIter = IntoIter<MintArg>;

    fn into_iter(self) -> Self::IntoIter {
        self.args.into_iter()
    }
}

impl FromIterator<MintArg> for MintArgList {
    fn from_iter<I: IntoIterator<Item = MintArg>>(iter: I) -> Self {
        let args: VecDeque<MintArg> = iter.into_iter().collect();
        Self { args }
    }
}
