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

// #(==,X,Y,A,B)
// -------------
// Equals.  Compare "X" and "Y" for equality.  To be equal, strings "X" and
// "Y" must be the same length, and have exactly the same characters.
//
// Returns: "A" if "X" and "Y" are equal, "B" otherwise.
struct EqPrim;
impl MintPrim for EqPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        let a1 = args[1].value();
        let a2 = args[2].value();

        let result = if a1 == a2 {
            args[3].value()
        } else {
            args[4].value()
        };

        interp.return_string(is_active, result);
    }
}

// #(!=,X,Y,A,B)
// -------------
// Not equals.  Convenience function equivalent to #(==,X,Y,B,A).
//
// Returns: "A" if "X" and "Y" are not equal, "B" otherwise.
struct NePrim;
impl MintPrim for NePrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        let a1 = args[1].value();
        let a2 = args[2].value();

        let result = if a1 != a2 {
            args[3].value()
        } else {
            args[4].value()
        };

        interp.return_string(is_active, result);
    }
}

// #(nc,X)
// -------
// Number of characters.
//
// Returns: The length of string "X" in characters.
struct NcPrim;
impl MintPrim for NcPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        let s = args[1].value();
        interp.return_integer(is_active, s.len() as i32, 10);
    }
}

// #(a?,X,Y,A,B)
// -------------
// Alphabetically ordered.
//
// Returns: "A" if "X" is lexicographically less than or equal to "Y",
// otherwise returns "B".
struct AoPrim;
impl MintPrim for AoPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        let a1 = args[1].value();
        let a2 = args[2].value();

        let result = if a1 <= a2 {
            &args[3].value()
        } else {
            &args[4].value()
        };

        interp.return_string(is_active, result);
    }
}

// #(sa,X1,X2,X3,...,Xn)
// ------------------
// Sort ascending.
//
// Returns: Parameters "X1" through "Xn" sorted lexicographically and
// separated by ",".
struct SaPrim;
impl MintPrim for SaPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        let mut result = Vec::new();

        if args.len() > 2 {
            // Collect all arguments
            // skip first which is function name and last which is END
            let mut items: Vec<&[u8]> = Vec::new();
            for arg in args.iter().take(args.len() - 1).skip(1) {
                items.push(arg.value());
            }

            // Sort lexicographically
            items.sort();

            // Join with commas
            if !items.is_empty() {
                result.extend_from_slice(items[0]);
                items.iter().skip(1).for_each(|item| {
                    result.push(b',');
                    result.extend_from_slice(item);
                });
            }
        }

        interp.return_string(is_active, &result);
    }
}

// #(si,X,Y)
// ---------
// String index.  Look up each character of literal string "Y" in form
// "X".  The raw ascii value of each character of "Y" is used as an index
// into form "X".  If "X" does not exist, or if the ordinal of the
// character of "Y" is greater than the number of characters in form "X",
// then the character in question is not modified.  Used to translate from
// lower to upper and vice versa.
//
// Returns: Translated string.
struct SiPrim;
impl MintPrim for SiPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        let form_name = args[1].value();
        let orig = args[2].value();
        let form_opt = interp.get_form(form_name);

        let mut result = Vec::new();
        if let Some(form) = form_opt {
            let form_content = form.content();
            for &ch in orig {
                let index = ch as usize;
                if index < form_content.len() {
                    result.push(form_content[index]);
                } else {
                    result.push(ch);
                }
            }
        } else {
            result.extend_from_slice(orig);
        }

        interp.return_string(is_active, &result);
    }
}

// #(nl)
// ---------
// Newline.  Returns the newline string.
//
// Returns: The newline string.
struct NlPrim;
impl MintPrim for NlPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, _args: &MintArgList) {
        let newline = b"\n".to_vec();
        interp.return_string(is_active, &newline);
    }
}

pub fn register_str_prims(interp: &mut Mint) {
    interp.add_prim(b"==".to_vec(), Box::new(EqPrim));
    interp.add_prim(b"!=".to_vec(), Box::new(NePrim));
    interp.add_prim(b"nc".to_vec(), Box::new(NcPrim));
    interp.add_prim(b"a?".to_vec(), Box::new(AoPrim));
    interp.add_prim(b"sa".to_vec(), Box::new(SaPrim));
    interp.add_prim(b"si".to_vec(), Box::new(SiPrim));
    interp.add_prim(b"nl".to_vec(), Box::new(NlPrim));
}
