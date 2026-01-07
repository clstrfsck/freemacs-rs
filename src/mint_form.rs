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

use crate::mint_types::{MintChar, MintCount, MintString};
use std::cmp::min;

#[derive(Debug, Clone)]
pub struct MintForm {
    content: MintString,
    index: MintCount,
}

impl MintForm {
    pub fn from_string(s: &[MintChar]) -> Self {
        Self {
            content: s.to_vec(),
            index: 0,
        }
    }

    pub fn set_pos(&mut self, n: MintCount) {
        self.index = min(n, self.content.len() as MintCount);
    }

    pub fn get_pos(&self) -> MintCount {
        min(self.index, self.content.len() as MintCount)
    }

    pub fn at_end(&self) -> bool {
        self.index >= self.content.len() as MintCount
    }

    pub fn get_n(&mut self, n: i32) -> MintString {
        self.index = min(self.index, self.content.len() as MintCount);
        let len = min(
            (self.content.len() as MintCount) - self.index,
            n.max(0) as MintCount,
        );
        let start = self.index as usize;
        let result = self.content[start..start + len as usize].to_vec();
        self.index += len;
        result
    }

    pub fn get(&self) -> MintString {
        let index = min(self.index, self.content.len() as MintCount);
        self.content[index as usize..].to_vec()
    }

    pub fn content(&self) -> &MintString {
        &self.content
    }
}
