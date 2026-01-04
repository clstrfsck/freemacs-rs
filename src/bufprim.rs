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

use crate::emacs_buffers::with_buffers;
use crate::mint::{Mint, MintPrim, MintVar};
use crate::mint_arg::MintArgList;
use crate::mint_string::{self, get_int_value};
use crate::mint_types::MintString;
use std::fs;
use std::io::Write;

// #(ba,X,Y)
// ---------
// Buffer allocate/select.  "X" is interpreted as a decimal number.  If "X"
// is less than zero, the current buffer number is returned.  If "X" equals
// zero, then a new buffer is created, and its buffer number returned.  If
// "X" is greater than zero, that buffer is selected and its number
// returned if it exists, otherwise zero is returned.  If an existing
// buffer is selected, and "Y" is non-null, the buffer is selected without
// necessarily expanding its size, which is cheap and means that the buffer
// cannot be modified.
//
// Returns: The buffer number of the current/selected/created buffer, or
// zero if no such buffer exists.
struct BaPrim;
impl MintPrim for BaPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        if args.len() < 2 {
            return;
        }

        let whattodo = args[1].get_int_value(10);
        let buf_num = with_buffers(|buffers| {
            if whattodo == 0 {
                buffers.new_buffer()
            } else if whattodo < 0 || buffers.select_buffer(whattodo as u32) {
                buffers.get_cur_buffer().borrow().get_buf_number()
            } else {
                0
            }
        });
        interp.return_integer(is_active, buf_num as i32, 10);
    }
}

// #(is,X,Y)
// ---------
// Insert string.  Inserts string "X" into the current buffer.
//
// Returns: Returns "Y" if inserted OK, null otherwise.
struct IsPrim;
impl MintPrim for IsPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        if args.len() < 2 {
            return;
        }

        let string = args[1].value();
        let success =
            with_buffers(|buffers| buffers.get_cur_buffer().borrow_mut().insert_string(string));

        if success && args.len() > 2 {
            interp.return_string(is_active, args[2].value());
        } else if !success {
            interp.return_null(is_active);
        }
    }
}

// #(pm,X,Y)
// -------
// Push/pop mark.  If "X" is greater than zero, that many temporary marks
// are stacked.  If "X" is less than zero, the absolute value of that many
// permanent marks are stacked.  If "X" is zero, temporary marks are
// unstacked.  All newly stacked marks are set to the current value of
// point.
//
// Returns: null if successful, "Y" in active mode if an error occurs (ie
// "X" would case the mark stack to overflow).
struct PmPrim;
impl MintPrim for PmPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        if args.len() < 2 {
            return;
        }

        let whattodo = args[1].get_int_value(10);
        let ok = with_buffers(|buffers| {
            let buf_rc = buffers.get_cur_buffer();
            let mut buf = buf_rc.borrow_mut();
            if whattodo > 0 {
                buf.push_temp_marks(whattodo as u32)
            } else if whattodo == 0 {
                buf.pop_temp_marks()
            } else {
                buf.create_perm_marks((-whattodo) as u32)
            }
        });

        if ok {
            interp.return_null(is_active);
        } else if args.len() > 2 {
            interp.return_string(true, args[2].value());
        }
    }
}

// #(sm,X,Y)
// ---------
// Set mark.  Set user mark "X" to mark "Y".  If mark "Y" is not specified
// or null, then "." is used.  The following values are acceptable for "Y":
//     '0..9'  User temporary marks
//     '@..Z'  User permanent marks
//     '*'     If the current buffer is displayed in both windows, this is
//             the value of point in the other window
//     '>'     Character to the right of the point
//     '<'     Character to the left of the point
//     '['     First character in the file
//     ']'     Last character in the file
//     '^'     Beginning of the current line
//     '$'     End of the current line
//     '-'     First blank char to the left
//     '+'     First blank char to the right
//     '{'     First non-blank char to the left
//     '}'     First non-blank char to the right
//     '.'     Point
// The following are valid values for "X":
//     '0..9'  User temporary marks
//     '@..Z'  User permanent marks
//     '*'     If the current buffer is displayed in both windows, this is
//             the value of point in the other window
//
// Returns: null
struct SmPrim;
impl MintPrim for SmPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        if args.len() < 2 {
            return;
        }

        let user_mark = args[1].value();
        if !user_mark.is_empty() {
            let mark = if args.len() > 2 && !args[2].value().is_empty() {
                args[2].value()[0]
            } else {
                b'.'
            };

            with_buffers(|buffers| {
                buffers
                    .get_cur_buffer()
                    .borrow_mut()
                    .set_mark(user_mark[0], mark)
            });
        }
        interp.return_null(is_active);
    }
}

// #(sp,X)
// -------
// Set point.  Sets point to mark given by "X".  See #(sm,...) for details
// of valid values for "X".
//
// Returns: null
struct SpPrim;
impl MintPrim for SpPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        if args.len() < 2 {
            return;
        }

        let marks = args[1].value();
        with_buffers(|buffers| {
            buffers
                .get_cur_buffer()
                .borrow_mut()
                .set_point_to_marks(marks)
        });
        interp.return_null(is_active);
    }
}

// #(dm,X)
// -------
// Delete to mark.  Delete from point to marks specified in string "X".
//
// Returns: null
struct DmPrim;
impl MintPrim for DmPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        if args.len() < 2 {
            return;
        }

        let marks = args[1].value();
        with_buffers(|buffers| buffers.get_cur_buffer().borrow_mut().delete_to_marks(marks));
        interp.return_null(is_active);
    }
}

// #(rm,X,Y)
// -------
// Read to mark.  Read from point to mark "X".  If there is insufficient
// space for the string, "Y" is returned in active mode.
//
// Returns: The buffer between point and mark "X" if enough space exists,
// otherwise return "Y" in active mode.
struct RmPrim;
impl MintPrim for RmPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        if args.len() < 2 {
            return;
        }

        let mark = args[1].value();
        if !mark.is_empty() {
            let s = with_buffers(|buffers| buffers.get_cur_buffer().borrow().read_to_mark(mark[0]));
            interp.return_string(is_active, &s);
        } else if args.len() > 2 {
            interp.return_string(true, args[2].value());
        }
    }
}

// #(rc,X)
// -------
// Read count.  Read count of characters between point and mark "X".
//
// Returns: The number of characters between point and mark as a decimal
// number.
struct RcPrim;
impl MintPrim for RcPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        if args.len() < 2 {
            return;
        }

        let mark = args[1].value();
        let count = if !mark.is_empty() {
            with_buffers(|buffers| buffers.get_cur_buffer().borrow().chars_to_mark(mark[0]))
        } else {
            0
        };
        interp.return_integer(is_active, count as i32, 10);
    }
}

// #(mb,X,A,B)
// -----------
// Mark before.
//
// Returns: "A" if mark "X" is before point, "B" otherwise.
struct MbPrim;
impl MintPrim for MbPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        if args.len() < 2 {
            return;
        }

        let mark = args[1].value();
        let before = if !mark.is_empty() {
            with_buffers(|buffers| buffers.get_cur_buffer().borrow().mark_before_point(mark[0]))
        } else {
            false
        };

        let result = if before && args.len() > 2 {
            args[2].value().clone()
        } else if args.len() > 3 {
            args[3].value().clone()
        } else {
            MintString::new()
        };

        interp.return_string(is_active, &result);
    }
}

// #(rf,X)
// -------
// Read file.  File given by literal string "X" is read into current
// buffer.
//
// Returns: null if successful, otherwise returns error message string.
struct RfPrim;
impl MintPrim for RfPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        if args.len() < 2 {
            return;
        }

        let file_name = args[1].value();
        let fn_str = String::from_utf8_lossy(file_name);

        match fs::read(&fn_str as &str) {
            Ok(contents) => {
                with_buffers(|buffers| {
                    buffers
                        .get_cur_buffer()
                        .borrow_mut()
                        .insert_string(&contents)
                });
                interp.return_null(is_active);
            }
            Err(e) => {
                let msg = format!("Error reading file: {}", e);
                interp.return_string(is_active, &msg.into());
            }
        }
    }
}

// #(wf,X,Y)
// ---------
// Write file.  Write text between point and mark "Y" to file given by
// literal string "X".
//
// Returns: null if write is successful, otherwise error message string.
struct WfPrim;
impl MintPrim for WfPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        if args.len() < 2 {
            return;
        }

        let file_name = args[1].value();
        let fn_str = String::from_utf8_lossy(file_name);

        let content = with_buffers(|buffers| {
            let buf_rc = buffers.get_cur_buffer();
            let buf = buf_rc.borrow();
            buf.read_to_mark_from(b']', 0)
        });

        match fs::File::create(&fn_str as &str) {
            Ok(mut file) => match file.write_all(content.as_slice()) {
                Ok(_) => {
                    with_buffers(|buffers| {
                        buffers.get_cur_buffer().borrow_mut().set_modified(false)
                    });
                    interp.return_null(is_active);
                }
                Err(e) => {
                    let msg = format!("Error writing file: {}", e);
                    interp.return_string(is_active, &msg.into());
                }
            },
            Err(e) => {
                let msg = format!("Error creating file: {}", e);
                interp.return_string(is_active, &msg.into());
            }
        }
    }
}

// #(pb)
// -----
// Print contents of current buffer to stderr.
//
// Returns: null.
struct PbPrim;
impl MintPrim for PbPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, _args: &MintArgList) {
        with_buffers(|buffers| {
            let buf = buffers.get_cur_buffer();
            let buf_ref = buf.borrow();
            eprintln!("Buffer number: {}", buf_ref.get_buf_number());
            eprintln!("===== CONTENTS =====");
            let content = buf_ref.read_to_mark(b'Z');
            for ch in content.as_slice() {
                eprint!("{}", *ch as char);
            }
            eprintln!("\n=== END CONTENTS ===");
        });
        interp.return_null(is_active);
    }
}

// #(bi,X,Y,A,B)
// -------------
// Buffer insert.  Insert into the current buffer the text from buffer "X"
// between point and mark "Y".
//
// Returns: "A" if insertion is successful, "B" otherwise.
struct BiPrim;
impl MintPrim for BiPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        if args.len() < 5 {
            interp.return_null(is_active);
            return;
        }

        let buf_num = args[1].get_int_value(10) as u32;
        let mark = args[2].value();
        let success_str = args[3].value();
        let _failure_str = args[4].value();

        let mut success = false;

        if !mark.is_empty() {
            let mark_char = mark[0];

            // Get text from source buffer
            let text = with_buffers(|buffers| {
                let cur_buf_num = buffers.get_cur_buffer().borrow().get_buf_number();
                if buffers.select_buffer(buf_num) {
                    let text = buffers.get_cur_buffer().borrow().read_to_mark(mark_char);
                    buffers.select_buffer(cur_buf_num);
                    Some(text)
                } else {
                    None
                }
            });

            // Insert into current buffer
            if let Some(text) = text {
                success = with_buffers(|buffers| {
                    buffers.get_cur_buffer().borrow_mut().insert_string(&text)
                });
            }
        }

        if success {
            interp.return_string(is_active, success_str);
        } else {
            interp.return_null(is_active);
        }
    }
}

// #(st,X)
// -------
// Syntax table. Sets the syntax table to the form given by "X".
// Syntax bits are as follows:
//     bit 0  0 = blank, 1 = non-blank (used for word matching)
//     bit 1  0 = not newline, 1 = newline
//
// Returns: null
struct StPrim;
impl MintPrim for StPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, _args: &MintArgList) {
        // FIXME: Not implemented
        interp.return_null(is_active);
    }
}

// #(lp,X,Y,A,B)
// -------------
// Look pattern.  Set search pattern to "X".  If "A" is not null, then "X"
// should be a regular expression (otherwise it's a string).  If "B" is not
// null, then case should be folded.
// The following regular expression characters are supported:
//       '*'         Zero or more
//       '[a-z]'     Character class
//       '[~a-z]'    Not character class
//       '.'         Any character
//       '^'         Beginning of line
//       '$'         End of line
// FIXME: need to implement the following
//       '\(' '\)'   Grouping (does not work with closures)
//       '\|'        Alternation
//       '\n'        New-line (does not have to appear at end of regex)
//       '\`'        Beginning of buffer
//       '\''        End of buffer
//       '\b'        Beginning or end of word
//       '\B'        Not beginning or end of word
//       '\<'        Beginning of word
//       '\>'        End of word
//       '\w'        Word character
//       '\W'        Not word character
//
// Returns: "Y" in active mode if an error occurs (eg invalid regex
// syntax), otherwise null.
struct LpPrim;
impl MintPrim for LpPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        if args.len() < 5 {
            interp.return_null(is_active);
            return;
        }

        let pattern = args[1].value();
        let error_str = args[2].value();
        let is_plain = args[3].value().is_empty();
        let fold_case = !args[4].value().is_empty();

        let success = with_buffers(|buffers| {
            if is_plain {
                buffers.set_search_string(pattern, fold_case)
            } else {
                buffers.set_search_regex(pattern, fold_case)
            }
        });

        if success {
            interp.return_null(is_active);
        } else {
            interp.return_string(true, error_str);
        }
    }
}

// #(l?,A,B,C,D,X,Y)
// -----------------
// Look and test.  "A", "B", "C" and "D" are marks.  The search occurs
// between marks "A" and "B".  If the string (set by #(lp,...)) is found,
// mark "C" is set to the start of the matched string, and "D" to the end.
// "A" defaults to the beginning of file, "B" defaults to end of file, if
// "C" is null, defaults to mark 0 and "D" defaults to mark 1.
//
// Returns: "X" if pattern is found, "Y" otherwise.
struct LkPrim;
impl MintPrim for LkPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        if args.len() < 7 {
            interp.return_null(is_active);
            return;
        }

        let mark1 = if args[1].value().is_empty() {
            b'['
        } else {
            args[1].value()[0]
        };
        let mark2 = if args[2].value().is_empty() {
            b']'
        } else {
            args[2].value()[0]
        };
        let mark3 = if args[3].value().is_empty() {
            0
        } else {
            args[3].value()[0]
        };
        let mark4 = if args[4].value().is_empty() {
            0
        } else {
            args[4].value()[0]
        };
        let success_str = args[5].value();
        let failure_str = args[6].value();

        let found = with_buffers(|buffers| buffers.search(mark1, mark2, mark3, mark4));

        if found {
            interp.return_string(is_active, success_str);
        } else {
            interp.return_string(is_active, failure_str);
        }
    }
}

// #(tr,X,Y)
// ---------
// Translate.  Translates from point to mark "X" using string "Y" as a
// translation character set.  Each character is read from the buffer, and
// if the ordinal value is less than the length of "Y", then it is replaced
// with this character.
//
// Returns: null
struct TrPrim;
impl MintPrim for TrPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        if args.len() < 3 {
            return;
        }

        let mark = args[1].value();
        let trstr = args[2].value();

        if !mark.is_empty() {
            with_buffers(|buffers| {
                buffers
                    .get_cur_buffer()
                    .borrow_mut()
                    .translate(mark[0], trstr)
            });
        }
        interp.return_null(is_active);
    }
}

struct ClVar;
impl MintVar for ClVar {
    fn get_val(&self, _interp: &Mint) -> MintString {
        with_buffers(|buffers| {
            let line_no = buffers.get_cur_buffer().borrow().get_point_line() + 1;
            let mut s = MintString::new();
            mint_string::append_num(&mut s, line_no as i32, 10);
            s
        })
    }

    fn set_val(&self, _interp: &mut Mint, val: &MintString) {
        let line_no = get_int_value(val, 10);
        with_buffers(|buffers| {
            let buffer = buffers.get_cur_buffer();
            let mut buf = buffer.borrow_mut();
            buf.set_point_line(std::cmp::max(0, line_no - 1) as u32);
        });
    }
}

struct CsVar;
impl MintVar for CsVar {
    fn get_val(&self, _interp: &Mint) -> MintString {
        with_buffers(|buffers| {
            let col_no = buffers.get_cur_buffer().borrow().get_column() + 1;
            let mut s = Vec::new();
            mint_string::append_num(&mut s, col_no as i32, 10);
            s
        })
    }

    fn set_val(&self, _interp: &mut Mint, val: &MintString) {
        let col_no = get_int_value(val, 10);
        if col_no > 0 {
            with_buffers(|buffers| {
                let buffer = buffers.get_cur_buffer();
                let mut buf = buffer.borrow_mut();
                buf.set_column(col_no as u32 - 1);
            });
        }
    }
}

struct MbVar;
impl MintVar for MbVar {
    fn get_val(&self, _interp: &Mint) -> MintString {
        with_buffers(|buffers| {
            let cb = buffers.get_cur_buffer();
            let mut s = Vec::new();
            let mod_flag = if cb.borrow().is_modified() { 1 } else { 0 };
            let wp_flag = if cb.borrow().is_write_protected() {
                2
            } else {
                0
            };
            mint_string::append_num(&mut s, mod_flag | wp_flag, 10);
            s
        })
    }

    fn set_val(&self, _interp: &mut Mint, val: &MintString) {
        let flags = get_int_value(val, 10);
        with_buffers(|buffers| {
            let buffer = buffers.get_cur_buffer();
            let mut cb = buffer.borrow_mut();
            cb.set_modified((flags & 1) != 0);
            cb.set_write_protected((flags & 2) != 0);
        });
    }
}

struct NlVar;
impl MintVar for NlVar {
    fn get_val(&self, _interp: &Mint) -> MintString {
        with_buffers(|buffers| {
            let cb = buffers.get_cur_buffer();
            let newline_count = cb.borrow().count_newlines_total() as i32;
            let mut s = Vec::new();
            mint_string::append_num(&mut s, newline_count + 1, 10);
            s
        })
    }

    fn set_val(&self, _interp: &mut Mint, _val: &MintString) {
        // Value can't be set
    }
}

struct PbVar;
impl MintVar for PbVar {
    fn get_val(&self, _interp: &Mint) -> MintString {
        with_buffers(|buffers| {
            let cb = buffers.get_cur_buffer();
            let point_line = cb.borrow().get_point_line() as i32;
            let newline_count = cb.borrow().count_newlines_total() as i32;
            let mut s = Vec::new();
            mint_string::append_num(&mut s, (point_line + 1) * 100 / (newline_count + 1), 10);
            s
        })
    }

    fn set_val(&self, _interp: &mut Mint, _val: &MintString) {
        // Value can't be set
    }
}

struct RsVar;
impl MintVar for RsVar {
    fn get_val(&self, _interp: &Mint) -> MintString {
        with_buffers(|buffers| {
            let cb = buffers.get_cur_buffer();
            let get_point_row = cb.borrow().get_point_row() as i32;
            let mut s = Vec::new();
            mint_string::append_num(&mut s, get_point_row, 10);
            s
        })
    }

    fn set_val(&self, _interp: &mut Mint, val: &MintString) {
        with_buffers(|buffers| {
            let cb = buffers.get_cur_buffer();
            cb.borrow_mut().set_point_row(get_int_value(val, 10) as u32);
        });
    }
}

struct TcVar;
impl MintVar for TcVar {
    fn get_val(&self, _interp: &Mint) -> MintString {
        with_buffers(|buffers| {
            let cb = buffers.get_cur_buffer();
            let tab_width = cb.borrow().get_tab_width() as i32;
            let mut s = Vec::new();
            mint_string::append_num(&mut s, tab_width, 10);
            s
        })
    }

    fn set_val(&self, _interp: &mut Mint, val: &MintString) {
        with_buffers(|buffers| {
            let cb = buffers.get_cur_buffer();
            cb.borrow_mut().set_tab_width(get_int_value(val, 10) as u32);
        });
    }
}

pub fn register_buf_prims(interp: &mut Mint) {
    interp.add_prim(b"ba".to_vec(), Box::new(BaPrim));
    interp.add_prim(b"is".to_vec(), Box::new(IsPrim));
    interp.add_prim(b"pm".to_vec(), Box::new(PmPrim));
    interp.add_prim(b"sm".to_vec(), Box::new(SmPrim));
    interp.add_prim(b"sp".to_vec(), Box::new(SpPrim));
    interp.add_prim(b"dm".to_vec(), Box::new(DmPrim));
    interp.add_prim(b"rm".to_vec(), Box::new(RmPrim));
    interp.add_prim(b"rc".to_vec(), Box::new(RcPrim));
    interp.add_prim(b"mb".to_vec(), Box::new(MbPrim));
    interp.add_prim(b"rf".to_vec(), Box::new(RfPrim));
    interp.add_prim(b"wf".to_vec(), Box::new(WfPrim));
    interp.add_prim(b"tr".to_vec(), Box::new(TrPrim));
    interp.add_prim(b"bi".to_vec(), Box::new(BiPrim));
    interp.add_prim(b"pb".to_vec(), Box::new(PbPrim));
    interp.add_prim(b"st".to_vec(), Box::new(StPrim));
    interp.add_prim(b"lp".to_vec(), Box::new(LpPrim));
    interp.add_prim(b"l?".to_vec(), Box::new(LkPrim));

    interp.add_var(b"cl".to_vec(), Box::new(ClVar));
    interp.add_var(b"cs".to_vec(), Box::new(CsVar));
    interp.add_var(b"mb".to_vec(), Box::new(MbVar));
    interp.add_var(b"nl".to_vec(), Box::new(NlVar));
    interp.add_var(b"pb".to_vec(), Box::new(PbVar));
    interp.add_var(b"rs".to_vec(), Box::new(RsVar));
    interp.add_var(b"tc".to_vec(), Box::new(TcVar));
}
