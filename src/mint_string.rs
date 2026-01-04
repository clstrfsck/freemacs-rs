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

use crate::mint_types::{MintCount, MintString};

fn digit_char(n: u32) -> u8 {
    if n < 10 {
        b'0' + n as u8
    } else {
        b'A' + (n - 10) as u8
    }
}

fn make_digits(s: &mut MintString, n: MintCount, base: u32) {
    let digit = n % base;
    if n >= base {
        make_digits(s, n / base, base);
    }
    s.push(digit_char(digit));
}

pub fn append_num(s: &mut MintString, n: i32, base: i32) {
    let base = base.clamp(2, 36) as u32;
    if n < 0 {
        s.push(b'-');
        make_digits(s, (-n) as MintCount, base);
    } else {
        make_digits(s, n as MintCount, base);
    }
}

pub fn get_int_value(s: &MintString, base: i32) -> i32 {
    let base = base.clamp(2, 36);
    let end_number = b'0' + (10.min(base) as u8);
    let end_letter = b'A' + (0.max(base - 10) as u8);

    let mut mult_val = 1;
    let mut i = s.len();

    while i > 0 {
        i -= 1;
        let ch = s[i].to_ascii_uppercase();
        if (ch >= b'0' && ch < end_number) || (ch >= b'A' && ch < end_letter) {
            continue;
        } else {
            if ch == b'-' {
                mult_val = -1;
            }
            i += 1;
            break;
        }
    }

    let mut number = 0;
    while i < s.len() {
        let ch = s[i].to_ascii_uppercase();
        if ch >= b'0' && ch < end_number {
            let digit = (ch - b'0') as i32;
            number = number * base + digit;
        } else if base > 10 && ch >= b'A' && ch < end_letter {
            let digit = 10 + (ch - b'A') as i32;
            number = number * base + digit;
        }
        i += 1;
    }

    number * mult_val
}

pub fn get_int_prefix(s: &MintString, base: i32) -> MintString {
    let base = base.clamp(2, 36);
    let end_number = b'0' + (10.min(base) as u8);
    let end_letter = b'A' + (0.max(base - 10) as u8);

    let mut plast = s.len();

    while plast > 0 {
        plast -= 1;
        let ch = s[plast].to_ascii_uppercase();
        if (ch >= b'0' && ch < end_number) || (ch >= b'A' && ch < end_letter) {
            continue;
        } else {
            if ch != b'-' {
                plast += 1;
            }
            break;
        }
    }

    s[..plast].to_vec()
}
