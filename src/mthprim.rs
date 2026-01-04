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

use crate::mint::{Mint, MintPrim};
use crate::mint_arg::MintArgList;
use crate::mint_string;
use crate::mint_types::MintString;

// Helper for base conversion
fn get_base(base_chr: u8, default: i32) -> i32 {
    match base_chr.to_ascii_uppercase() {
        b'A' | b'C' => 0, // ASCII
        b'B' => 2,        // Binary
        b'O' => 8,        // Octal
        b'D' => 10,       // Decimal
        b'H' => 16,       // Hexadecimal
        _ => default,
    }
}

// #(bc,X,Y,Z)
// -----------
// Base conversion.  Convert "X" from base "Y" to base "Z".  Bases are as
// follows:
//     'a','c' ASCII - converts a single ASCII character to it's ordinal.
//     'd'     Decimal
//     'o'     Octal
//     'h'     Hexadecimal
//     'b'     Binary
//
// Returns: "X" interpreted according to base "Y" in base "Z".
struct BcPrim;
impl MintPrim for BcPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        if args.len() < 4 {
            return;
        }

        let arg1 = &args[1];
        let arg2 = &args[2];
        let arg3 = &args[3];

        let sbase_chr = arg2.get_first_char().unwrap_or(b'a');
        let sbase = get_base(sbase_chr, 0);
        let mut prefix = MintString::new();

        let num = if sbase != 0 {
            prefix = arg1.get_int_prefix(sbase);
            arg1.get_int_value(sbase)
        } else {
            arg1.get_first_char().map(|ch| ch as i32).unwrap_or(0)
        };

        let dbase_chr = arg3.get_first_char().unwrap_or(b'd');
        let dbase = get_base(dbase_chr, 10);

        if dbase != 0 {
            mint_string::append_num(&mut prefix, num, dbase);
            interp.return_string(is_active, &prefix);
        } else {
            let result = vec![num as u8];
            interp.return_string(is_active, &result);
        }
    }
}

// Binary operation helper trait
trait BinaryOp {
    fn perform(&self, a1: i32, a2: i32) -> i32;
}

struct BinaryOpPrim<T: BinaryOp> {
    op: T,
}

impl<T: BinaryOp> MintPrim for BinaryOpPrim<T> {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        if args.len() < 3 {
            return;
        }

        let a1 = args[1].get_int_value(10);
        let prefix = args[1].get_int_prefix(10);

        let a2 = args[2].get_int_value(10);
        let result = self.op.perform(a1, a2);

        interp.return_integer_with_prefix(is_active, &prefix, result, 10);
    }
}

// Math operations
struct AddOp;
impl BinaryOp for AddOp {
    fn perform(&self, a1: i32, a2: i32) -> i32 {
        a1 + a2
    }
}

struct SubOp;
impl BinaryOp for SubOp {
    fn perform(&self, a1: i32, a2: i32) -> i32 {
        a1 - a2
    }
}

struct MulOp;
impl BinaryOp for MulOp {
    fn perform(&self, a1: i32, a2: i32) -> i32 {
        a1 * a2
    }
}

struct DivOp;
impl BinaryOp for DivOp {
    fn perform(&self, a1: i32, a2: i32) -> i32 {
        if a2 == 0 { a1 } else { a1 / a2 }
    }
}

struct ModOp;
impl BinaryOp for ModOp {
    fn perform(&self, a1: i32, a2: i32) -> i32 {
        if a2 == 0 { a1 } else { a1 % a2 }
    }
}

struct IorOp;
impl BinaryOp for IorOp {
    fn perform(&self, a1: i32, a2: i32) -> i32 {
        a1 | a2
    }
}

struct AndOp;
impl BinaryOp for AndOp {
    fn perform(&self, a1: i32, a2: i32) -> i32 {
        a1 & a2
    }
}

struct XorOp;
impl BinaryOp for XorOp {
    fn perform(&self, a1: i32, a2: i32) -> i32 {
        a1 ^ a2
    }
}

// #(g?,X,Y,A,B)
// -------------
// Numeric greater than.
//
// Returns: "A" if "X" is greater than "Y" when interpreted as numbers, "B"
// otherwise.
struct GtPrim;
impl MintPrim for GtPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        if args.len() < 5 {
            return;
        }

        let a1 = args[1].get_int_value(10);
        let a2 = args[2].get_int_value(10);

        let result = if a1 > a2 {
            args[3].value().clone()
        } else {
            args[4].value().clone()
        };

        interp.return_string(is_active, &result);
    }
}

pub fn register_mth_prims(interp: &mut Mint) {
    interp.add_prim(b"bc".to_vec(), Box::new(BcPrim));
    interp.add_prim(b"++".to_vec(), Box::new(BinaryOpPrim { op: AddOp }));
    interp.add_prim(b"--".to_vec(), Box::new(BinaryOpPrim { op: SubOp }));
    interp.add_prim(b"**".to_vec(), Box::new(BinaryOpPrim { op: MulOp }));
    interp.add_prim(b"//".to_vec(), Box::new(BinaryOpPrim { op: DivOp }));
    interp.add_prim(b"%%".to_vec(), Box::new(BinaryOpPrim { op: ModOp }));
    interp.add_prim(b"||".to_vec(), Box::new(BinaryOpPrim { op: IorOp }));
    interp.add_prim(b"&&".to_vec(), Box::new(BinaryOpPrim { op: AndOp }));
    interp.add_prim(b"^^".to_vec(), Box::new(BinaryOpPrim { op: XorOp }));
    interp.add_prim(b"g?".to_vec(), Box::new(GtPrim));
}
