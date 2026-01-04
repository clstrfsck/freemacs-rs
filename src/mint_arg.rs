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

// FIXME: need something that is efficient to put elements on the front.
pub type MintArgList = Vec<MintArg>;
