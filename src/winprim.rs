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

use crate::emacs_buffers::with_current_buffer;
use crate::emacs_window;
use crate::mint::{Mint, MintPrim, MintVar};
use crate::mint_arg::{ArgType, MintArgList};
use crate::mint_string;
use crate::mint_types::MintString;

// #(it,X)
// -------
// Input timed.  Reads a character from the keyboard, waiting for "X"
// hundredths of a second, or 0 if "X" is null.
// Note: Key names are defined elsewhere.
//
// Returns: The name of the key pressed, or "Timeout" if no key pressed.
struct ItPrim;
impl MintPrim for ItPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        let timeout = args[1].get_int_value(10) * 10; // Hundredths to millis
        let key = emacs_window::with_window(|w| w.get_input(timeout as u32));
        interp.return_string(is_active, &key);
    }
}

// #(ow,X)
// -------
// Overwrite screen.  Write literal string "X" on screen at the current
// cursor position.
//
// Returns: null
struct OwPrim;
impl MintPrim for OwPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        // Skip first arg (function name) and last arg (END marker)
        for arg in args.iter().skip(1) {
            if arg.arg_type() != ArgType::End {
                emacs_window::with_window(|w| w.overwrite(arg.value()));
            }
        }
        interp.return_null(is_active);
    }
}

// #(an,X,Y,Z)
// -----------
// Announce.  Write on the console after the current window.  If "Y" is not
// null, "X" is displayed after the top window, otherwise "X" and "Z"
// are displayed at the bottom of the screen, with the cursor placed after
// "X".
//
// Returns: null
struct AnPrim;
impl MintPrim for AnPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        let left = args[1].value();
        let flag = args[2].value();
        let right = args[3].value();

        emacs_window::with_window(|w| {
            if flag.is_empty() {
                w.announce(left, right);
            } else {
                w.announce_win(left, right);
            }
        });

        interp.return_null(is_active);
    }
}

// #(xy,X,Y)
// ---------
// Goto X,Y.  Position the cursor at screen column "X", row "Y".  The top
// row is row 0, and the left column is column 0.
//
// Returns: null
struct XyPrim;
impl MintPrim for XyPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        let x = args[1].get_int_value(10);
        let y = args[2].get_int_value(10);

        emacs_window::with_window(|w| w.gotoxy(x, y));
        interp.return_null(is_active);
    }
}

// #(bl,X,Y)
// ---------
// Bell.  Ring the bell at frequency "X" for "Y" 18ths of a second.  If "X"
// is 0, then the default frequency is used.  If "X" is less than zero then
// a "visual bell" is rung instead.
//
// Returns: null
struct BlPrim;
impl MintPrim for BlPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        let freq = args[1].get_int_value(10);
        let millis = args[2].get_int_value(10) * 56; // 18ths of second to millis

        emacs_window::with_window(|w| {
            if freq < 0 {
                w.visual_bell(millis as u32);
            } else {
                w.audible_bell(freq as u32, millis as u32);
            }
        });

        interp.return_null(is_active);
    }
}

// #(rd,X)
// -------
// Redisplay the screen.  If "X" is non-null, the screen is completely
// repainted.
//
// Returns: null
struct RdPrim;
impl MintPrim for RdPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        let force = !args[1].is_empty();

        with_current_buffer(|buf| {
            emacs_window::with_window(|w| w.redisplay(buf, force));
        });

        interp.return_null(is_active);
    }
}

// Variables

// bs - Bottom scroll percent
struct BsVar;
impl MintVar for BsVar {
    fn get_val(&self, _interp: &Mint) -> MintString {
        let val = emacs_window::with_window(|w| w.get_bot_scroll_percent());
        let mut s = Vec::new();
        mint_string::append_num(&mut s, val as i32, 10);
        s
    }

    fn set_val(&self, _interp: &mut Mint, val: &MintString) {
        let n = mint_string::get_int_value(val, 10);
        emacs_window::with_window(|w| w.set_bot_scroll_percent(n as u32));
    }
}

// ts - Top scroll percent
struct TsVar;
impl MintVar for TsVar {
    fn get_val(&self, _interp: &Mint) -> MintString {
        let val = emacs_window::with_window(|w| w.get_top_scroll_percent());
        let mut s = Vec::new();
        mint_string::append_num(&mut s, val as i32, 10);
        s
    }

    fn set_val(&self, _interp: &mut Mint, val: &MintString) {
        let n = mint_string::get_int_value(val, 10);
        emacs_window::with_window(|w| w.set_top_scroll_percent(n as u32));
    }
}

// bc - Background colour
struct BcVar;
impl MintVar for BcVar {
    fn get_val(&self, _interp: &Mint) -> MintString {
        let val = emacs_window::with_window(|w| w.get_back_colour());
        let mut s = Vec::new();
        mint_string::append_num(&mut s, val, 10);
        s
    }

    fn set_val(&self, _interp: &mut Mint, val: &MintString) {
        let n = mint_string::get_int_value(val, 10);
        emacs_window::with_window(|w| w.set_back_colour(n));
    }
}

// fc - Foreground colour
struct FcVar;
impl MintVar for FcVar {
    fn get_val(&self, _interp: &Mint) -> MintString {
        let val = emacs_window::with_window(|w| w.get_fore_colour());
        let mut s = Vec::new();
        mint_string::append_num(&mut s, val, 10);
        s
    }

    fn set_val(&self, _interp: &mut Mint, val: &MintString) {
        let n = mint_string::get_int_value(val, 10);
        emacs_window::with_window(|w| w.set_fore_colour(n));
    }
}

// cc - Control foreground colour
struct CcVar;
impl MintVar for CcVar {
    fn get_val(&self, _interp: &Mint) -> MintString {
        let val = emacs_window::with_window(|w| w.get_ctrl_fore_colour());
        let mut s = Vec::new();
        mint_string::append_num(&mut s, val, 10);
        s
    }

    fn set_val(&self, _interp: &mut Mint, val: &MintString) {
        let n = mint_string::get_int_value(val, 10);
        emacs_window::with_window(|w| w.set_ctrl_fore_colour(n));
    }
}

// rc - Read columns
struct RcVar;
impl MintVar for RcVar {
    fn get_val(&self, _interp: &Mint) -> MintString {
        let val = emacs_window::with_window(|w| w.get_columns());
        let mut s = Vec::new();
        mint_string::append_num(&mut s, val as i32, 10);
        s
    }

    fn set_val(&self, _interp: &mut Mint, _val: &MintString) {
        // Read-only
    }
}

// bl - Buffer lines
struct BlVar;
impl MintVar for BlVar {
    fn get_val(&self, _interp: &Mint) -> MintString {
        let val = emacs_window::with_window(|w| w.get_lines());
        let mut s = Vec::new();
        mint_string::append_num(&mut s, val as i32, 10);
        s
    }

    fn set_val(&self, _interp: &mut Mint, _val: &MintString) {
        // Could adjust window size
    }
}

// tl - Top line (placeholder)
struct TlVar;
impl MintVar for TlVar {
    fn get_val(&self, _interp: &Mint) -> MintString {
        // FIXME: Placeholder for when windows are implemented
        b"0".to_vec()
    }

    fn set_val(&self, _interp: &mut Mint, _val: &MintString) {
        // FIXME: Does nothing. Placeholder.
    }
}

// wc - Whitespace colour
struct WcVar;
impl MintVar for WcVar {
    fn get_val(&self, _interp: &Mint) -> MintString {
        let val = emacs_window::with_window(|w| w.get_whitespace_colour());
        let mut s = Vec::new();
        mint_string::append_num(&mut s, val, 10);
        s
    }

    fn set_val(&self, _interp: &mut Mint, val: &MintString) {
        let n = mint_string::get_int_value(val, 10);
        emacs_window::with_window(|w| w.set_whitespace_colour(n));
    }
}

// ws - Whitespace display
struct WsVar;
impl MintVar for WsVar {
    fn get_val(&self, _interp: &Mint) -> MintString {
        let val = emacs_window::with_window(|w| w.get_whitespace_display());
        if val { b"1".to_vec() } else { b"0".to_vec() }
    }

    fn set_val(&self, _interp: &mut Mint, val: &MintString) {
        let n = mint_string::get_int_value(val, 10);
        emacs_window::with_window(|w| w.set_whitespace_display(n != 0));
    }
}

pub fn register_win_prims(interp: &mut Mint) {
    // Primitives
    interp.add_prim(b"it".to_vec(), Box::new(ItPrim));
    interp.add_prim(b"ow".to_vec(), Box::new(OwPrim));
    interp.add_prim(b"an".to_vec(), Box::new(AnPrim));
    interp.add_prim(b"xy".to_vec(), Box::new(XyPrim));
    interp.add_prim(b"bl".to_vec(), Box::new(BlPrim));
    interp.add_prim(b"rd".to_vec(), Box::new(RdPrim));

    // Variables
    interp.add_var(b"bc".to_vec(), Box::new(BcVar));
    interp.add_var(b"bl".to_vec(), Box::new(BlVar));
    interp.add_var(b"bs".to_vec(), Box::new(BsVar));
    interp.add_var(b"cc".to_vec(), Box::new(CcVar));
    interp.add_var(b"fc".to_vec(), Box::new(FcVar));
    interp.add_var(b"rc".to_vec(), Box::new(RcVar));
    interp.add_var(b"tl".to_vec(), Box::new(TlVar));
    interp.add_var(b"ts".to_vec(), Box::new(TsVar));
    interp.add_var(b"wc".to_vec(), Box::new(WcVar));
    interp.add_var(b"ws".to_vec(), Box::new(WsVar));
}

pub fn key_waiting() -> bool {
    emacs_window::key_waiting()
}
