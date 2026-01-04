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

use crate::mint::{Mint, MintPrim, MintVar};
use crate::mint_arg::MintArgList;
use crate::mint_string::{self, get_int_value};
use crate::mint_types::MintString;

// #(lv,X)
// -------
// Load variable.
//
// Returns: The value of variable given by literal string "X".
struct LvPrim;
impl MintPrim for LvPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        if args.len() < 2 {
            interp.return_null(is_active);
            return;
        }

        let var_name = args[1].value();
        let value = interp.get_var(var_name);
        interp.return_string(is_active, &value);
    }
}

// #(sv,X,Y)
// ---------
// Set variable.  Set variable given by literal string "X" to value "Y".
//
// Returns: null
struct SvPrim;
impl MintPrim for SvPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        if args.len() < 3 {
            return;
        }

        let var_name = args[1].value();
        let value = args[2].value();
        interp.set_var(var_name, value);
        interp.return_null(is_active);
    }
}

// vn
// --
// Get version number.  This variable cannot be set.
struct VnVar;
impl MintVar for VnVar {
    fn get_val(&self, _interp: &Mint) -> MintString {
        b"2.0a".to_vec()
    }

    fn set_val(&self, _interp: &mut Mint, _val: &MintString) {
        // Read-only
    }
}

// as
// --
// Auto save limit.  After this many characters have been entered, the
// idling string is set to #(Fauto-save).  Once this has been executed, it
// is reset to the default idle string.
struct AsVar;
impl MintVar for AsVar {
    fn get_val(&self, interp: &Mint) -> MintString {
        let val = interp.get_idle_max();
        let mut s = Vec::new();
        mint_string::append_num(&mut s, val, 10);
        s
    }

    fn set_val(&self, _interp: &mut Mint, val: &MintString) {
        let limit = get_int_value(val, 10);
        _interp.set_idle_max(limit);
    }
}

pub fn register_var_prims(interp: &mut Mint) {
    // Primitives
    interp.add_prim(b"lv".to_vec(), Box::new(LvPrim));
    interp.add_prim(b"sv".to_vec(), Box::new(SvPrim));

    // Variables
    interp.add_var(b"vn".to_vec(), Box::new(VnVar));
    interp.add_var(b"as".to_vec(), Box::new(AsVar));
}
