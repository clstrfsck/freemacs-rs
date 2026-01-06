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
use regex::bytes::Regex;

pub trait Buffer {
    fn size(&self) -> MintCount;
    fn get(&self, offset: MintCount) -> Option<MintChar>;
    fn replace(&mut self, offset: MintCount, n: MintCount, replacement: &MintString) -> bool;
    fn erase(&mut self, offset: MintCount, n: MintCount) -> bool;
    fn insert(&mut self, offset: MintCount, to_insert: &MintString) -> bool;
    fn find_forward(
        &self,
        regex: &Regex,
        start: MintCount,
        end: MintCount,
    ) -> Option<(MintCount, MintCount)>;
    fn find_backward(
        &self,
        regex: &Regex,
        start: MintCount,
        end: MintCount,
    ) -> Option<(MintCount, MintCount)>;
}
