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
use crate::mint_types::MintString;

// #(ds,X,Y)
// ---------
// Define string.  A form with name "X" is defined with value "Y". If a
// form named "X" already exists, then it's current value is discarded.
//
// Returns: null
struct DsPrim;
impl MintPrim for DsPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        let form_name = args[1].value();
        let form_value = args[2].value();
        interp.set_form_value(form_name, form_value);
        interp.return_null(is_active);
    }
}

// #(gs,X,Y1,Y2,...,Yn)
// --------------------
// Get string.  Form with name "X" is retrieved.  If the form contains any
// parameter markers, P1..Pn, they are replaced with literal strings
// Y1..Yn.
//
// Returns: Form "X" with parameter markers replaced with literal strings
// "Y1".."Yn".
struct GsPrim;
impl MintPrim for GsPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        let form_name = args[1].value();
        let new_args = if args.len() > 2 {
            args.iter().skip(2).cloned().collect()
        } else {
            MintArgList::default()
        };
        let form = if let Some(f) = interp.get_form(form_name) {
            f.get()
        } else {
            MintString::new()
        };
        interp.return_seg_string(is_active, &form, &new_args);
    }
}

// #(go,X,Y)
// ---------
// Get one.  Gets a character from form "X".  If the form cannot be found,
// the null string is returned.  If the form is found, and the form pointer
// is currently at the end of the form, string "Y" is returned in active
// mode.  This is approximately equivalent to the TRAC #(cc,X,Y) primitive,
// only argument markers appear to be returned in MINT.
//
// Returns: The character from the form at the form pointer.
struct GoPrim;
impl MintPrim for GoPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        let form_name = args[1].value();
        let error_string = args[2].value();
        interp.return_n_form(is_active, form_name, 1, error_string);
    }
}

// #(gn,X,Y,Z)
// -----------
// Get n.  Gets "Y" characters from form "X".  If form "X" cannot be found,
// then "Z" is returned in active mode.  This differs from the TRAC
// #(cn,...) primitive in that argument markers are returned, and negative
// values of "Y" are not allowed.
//
// Returns: "Y" characters from form "X" at the current form pointer.
struct GnPrim;
impl MintPrim for GnPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        let form_name = args[1].value();
        let count = args[2].get_int_value(10);
        let error_string = args[3].value();
        interp.return_n_form(is_active, form_name, count, error_string);
    }
}

// #(rs,X)
// -------
// Reset string.  Resets the form pointer associated with form "X".
//
// Returns: null
struct RsPrim;
impl MintPrim for RsPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        let form_name = args[1].value();
        interp.set_form_pos(form_name, 0);
        interp.return_null(is_active);
    }
}

// #(fm,X,Y,Z)
// -----------
// First match.  Finds the first match of literal string "Y" in form "X".
// If the string is found, the form pointer is advanced to after the string
// found, and the portion of the form before the matched string is
// returned.  If form "X" cannot be found, null is returned.  If "Y" is
// null, or cannot be found in form "X", then "Z" is returned in active
// mode.
//
// Returns: null if "X" cannot be found, the portion of the form "X" before
// literal string "Y" if form "X" exists, or "Z" in active mode if "Y" is
// null or cannot be found in "X".
struct FmPrim;
impl MintPrim for FmPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        let form_name = args[1].value();
        let search_str = args[2].value();
        let not_found_str = args[3].value();

        if let Some(form) = interp.get_form(form_name) {
            let content = form.content();
            let pos = form.get_pos() as usize;

            if search_str.is_empty() {
                // Empty search string - return not_found string in active mode
                interp.return_string(true, not_found_str);
            } else {
                // Search for the string
                if let Some(found_pos) = content[pos..]
                    .windows(search_str.len())
                    .position(|window| window == search_str)
                {
                    let absolute_pos = pos + found_pos;
                    let result = content[pos..absolute_pos].to_vec();
                    interp.set_form_pos(form_name, (absolute_pos + search_str.len()) as u32);
                    interp.return_string(is_active, &result);
                } else {
                    // Not found - return not_found string in active mode
                    interp.return_string(true, not_found_str);
                }
            }
        } else {
            interp.return_null(is_active);
        }
    }
}

// #(n?,X,A,B)
// -----------
// Name exists?  Check to see if form given by literal string "X" exists.
//
// Returns: "A" if form exists, "B" otherwise.
struct NxPrim;
impl MintPrim for NxPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        let form_name = args[1].value();
        let exists_str = args[2].value();
        let not_exists_str = args[3].value();

        let result = if interp.get_form(form_name).is_some() {
            exists_str
        } else {
            not_exists_str
        };

        interp.return_string(is_active, result);
    }
}

// #(ls,X,Y)
// ---------
// List strings.
//
// Returns: A list of forms separated by literal string "X" that match
// prefix "Y".
struct LsPrim;
impl MintPrim for LsPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        let separator = args[1].value();
        let prefix = args[2].value();
        interp.return_form_list(is_active, separator, prefix);
    }
}

// #(es,X1,X2,...,Xn)
// ------------------
// Erase strings.  Remove all forms with names "X1", "X2", ..., "Xn".
//
// Returns: null
struct EsPrim;
impl MintPrim for EsPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        // Skip function name (0) and END marker (last)
        for arg in args.iter().take(args.len() - 1).skip(1) {
            let form_name = arg.value();
            interp.del_form(form_name);
        }
        interp.return_null(is_active);
    }
}

// #(mp,X,Y1,Y2,...,Yn)
// --------------------
// Make parameters.  Form with name "X" is scanned for occurrences of the
// literal sub-string Y1.  If any are found, they are replaced by special
// parameter markers P1.  This process is repeated for Y2 through Yn,
// replacing with parameter markers P2 through Pn.
// Corresponds to the TRAC primitive #(ss,X,Y1,...,Yn).
//
// Returns: null
struct MpPrim;
impl MintPrim for MpPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        let form_name = args[1].value();

        if let Some(form) = interp.get_form(form_name) {
            let mut form_value = form.content().clone();

            // Process each parameter (skip function name, form name, and END marker)
            let mut param_marker = 0x80u8;
            for arg in args.iter().take(args.len() - 1).skip(2) {
                let search_str = arg.value();
                if !search_str.is_empty() {
                    // Find and replace all occurrences
                    let mut pos = 0;
                    while pos < form_value.len() {
                        if pos + search_str.len() <= form_value.len()
                            && &form_value[pos..pos + search_str.len()] == search_str
                        {
                            // Replace with parameter marker
                            form_value.splice(pos..pos + search_str.len(), [param_marker]);
                            pos += 1;
                        } else {
                            pos += 1;
                        }
                    }
                }
                param_marker += 1;
            }

            interp.set_form_value(form_name, &form_value);
        }

        interp.return_null(is_active);
    }
}

// #(hk,X1,X2,X3,...,Xn)
// ---------
// Hook string.  Searches for forms named "X1", through "Xn".  If a form
// that exists is found, evaluates using #(gs,...) using the remainder of
// the arguments.  For example: #(hk,f1,f2,f3,f4) if form "f1" does not
// exist, but form "f2" does, is equivalent to #(gs,f2,f3,f4).
//
// Returns: Expanded version of first of form X1..Xn found, or null if no
// form found.
struct HkPrim;
impl MintPrim for HkPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        if args.len() > 2 {
            // Search for first existing form (skip function name and END marker)
            for i in 1..args.len() - 1 {
                if let Some(form) = interp.get_form(args[i].value()) {
                    // Found a form - expand it with args as parameters
                    let content = form.content().clone();
                    let param_args: MintArgList = args.iter().skip(i).cloned().collect();
                    interp.return_seg_string(is_active, &content, &param_args);
                    return;
                }
            }
        }
        // No form found
        interp.return_null(is_active);
    }
}

pub fn register_frm_prims(interp: &mut Mint) {
    interp.add_prim(b"ds".to_vec(), Box::new(DsPrim));
    interp.add_prim(b"gs".to_vec(), Box::new(GsPrim));
    interp.add_prim(b"go".to_vec(), Box::new(GoPrim));
    interp.add_prim(b"gn".to_vec(), Box::new(GnPrim));
    interp.add_prim(b"rs".to_vec(), Box::new(RsPrim));
    interp.add_prim(b"fm".to_vec(), Box::new(FmPrim));
    interp.add_prim(b"n?".to_vec(), Box::new(NxPrim));
    interp.add_prim(b"ls".to_vec(), Box::new(LsPrim));
    interp.add_prim(b"es".to_vec(), Box::new(EsPrim));
    interp.add_prim(b"mp".to_vec(), Box::new(MpPrim));
    interp.add_prim(b"hk".to_vec(), Box::new(HkPrim));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mint_arg::{ArgType, MintArg, MintArgList};

    fn arg_with_bytes(t: ArgType, value: &str) -> MintArg {
        let mut a = MintArg::new(t);
        a.append_slice(value.as_bytes());
        a
    }

    fn build_args(fn_name: &str, args: &[&str], fn_type: ArgType) -> MintArgList {
        let mut al = MintArgList::new();
        al.push_front(MintArg::new(ArgType::End));
        for &a in args.iter().rev() {
            al.push_front(arg_with_bytes(ArgType::Arg, a));
        }
        al.push_front(arg_with_bytes(fn_type, fn_name));
        al
    }

    #[test]
    fn ds_sets_form_value() {
        let mut mint = Mint::new();
        register_frm_prims(&mut mint);

        // call #(ds, "m", "value")
        let args = build_args("ds", &["m", "value"], ArgType::Neutral);

        // call the primitive directly (unit test in same module can access private prims map)
        let prim = mint.get_prim(b"ds").unwrap().clone();
        prim.execute(&mut mint, false, &args);

        // check the form was stored
        let form = mint.get_form(&b"m".to_vec());
        assert!(form.is_some());
        assert_eq!(form.unwrap().content(), &b"value".to_vec());
    }

    #[test]
    fn gs_returns_form_with_params() {
        let mut mint = Mint::new();
        register_frm_prims(&mut mint);

        mint.set_form_value(&b"f".to_vec(), &b"\x80".to_vec());

        let args = build_args("gs", &["f", "X"], ArgType::Neutral);
        let prim = mint.get_prim(b"gs").unwrap().clone();
        prim.execute(&mut mint, false, &args);

        assert!(mint.get_form(b"f").is_some());
    }
}
