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
use crate::mint_types::MintString;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::time::SystemTime;

// #(ab,X)
// -------
// Convert path given by "X" to an absolute path.
//
// Returns: the absolute path for "X", or "X" if an error occurs.
struct AbPrim;
impl MintPrim for AbPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        let path_str = args[1].value();
        let path = String::from_utf8_lossy(path_str);
        let path_buf = PathBuf::from(path.as_ref());

        let result = if let Ok(abs_path) = path_buf.canonicalize() {
            abs_path.to_string_lossy().as_bytes().to_vec()
        } else if let Ok(abs_path) = std::fs::canonicalize(&path_buf) {
            abs_path.to_string_lossy().as_bytes().to_vec()
        } else {
            // Fall back to original path
            path_str.to_vec()
        };

        interp.return_string(is_active, &result);
    }
}

// #(hl,X)
// -------
// Halt.  Exit to operating system with return code "X" interpreted as
// decimal number.
//
// Returns: does not return
struct HlPrim;
impl MintPrim for HlPrim {
    fn execute(&self, _interp: &mut Mint, _is_active: bool, args: &MintArgList) {
        let exit_code = if args.len() >= 2 {
            args[1].get_int_value(10)
        } else {
            0
        };
        process::exit(exit_code);
    }
}

// #(ct,X,Y)
// ---------
// Current time.  If "X" is null, returns system date/time.  If "X" is not
// null, it is used as a filename.  If "X" is specified, then if "Y" is
// non-null, binary file attributes and file size are included in the
// output string.
//
// Returns: ("X" null) System date in format "Sun Aug 08 09:01:03 2003".
//
// Returns: ("X" not null, "Y" null) Date of file "X" in above format, or
// null if no such file.
//
// Returns: ("X" not null, "Y" not null) Date of file "X" in above format,
// with file attributes prepended as 6 binary digits, and file size
// appended in the format "010000Sun Aug 08 09:01:03 2003 104323".  The
// bits of the file attributes have the following meanings if set:
//     Bit 0 - File is read only
//     Bit 1 - File is hidden
//     Bit 2 - File is a system file
//     Bit 3 - File is a volume label
//     Bit 4 - File is a directory
//     Bit 5 - File is ready for archiving (modified since backup)
struct CtPrim;
impl MintPrim for CtPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        let file_name = args[1].value();

        let result = if file_name.is_empty() {
            // Get current system time
            format_system_time(SystemTime::now())
        } else {
            // Get file time
            let path_str = String::from_utf8_lossy(file_name);
            let path = Path::new(path_str.as_ref());

            if let Ok(metadata) = fs::metadata(path) {
                if let Ok(modified) = metadata.modified() {
                    let extra_info = args.len() > 2 && !args[2].value().is_empty();

                    if extra_info {
                        // Include file attributes and size
                        let is_dir = metadata.is_dir();
                        let is_file = metadata.is_file();
                        let size = metadata.len();

                        // Build attribute bits
                        let mut attrs = String::new();
                        attrs.push('0'); // Bit 5: archive (not used)
                        attrs.push(if is_dir { '1' } else { '0' }); // Bit 4: directory
                        attrs.push('0'); // Bit 3: volume label (not used)
                        attrs.push(if !is_dir && !is_file { '1' } else { '0' }); // Bit 2: system file
                        attrs.push('0'); // Bit 1: hidden (not used)
                        attrs.push('0'); // Bit 0: read-only (not implemented)

                        format!("{}{} {}", attrs, format_system_time(modified), size)
                    } else {
                        format_system_time(modified)
                    }
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        };

        let result_bytes = result.as_bytes().to_vec();
        interp.return_string(is_active, &result_bytes);
    }
}

// #(ff,X,Y)
// ---------
// Find file.  "X" is a literal string which may contain globbing
// characters. "Y" is a separator string used in the return value.
//
// Returns: List of matching files, separated by literal string "Y".
struct FfPrim;
impl MintPrim for FfPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        let pattern = args[1].value();
        let separator = args[2].value();
        let pattern_str = String::from_utf8_lossy(pattern);

        let mut results = Vec::new();

        // Use glob pattern matching
        if let Ok(entries) = glob::glob(&pattern_str) {
            for entry in entries.flatten() {
                if let Some(file_name) = entry.file_name() {
                    results.extend_from_slice(file_name.to_string_lossy().as_bytes());
                    results.extend_from_slice(separator);
                }
            }
        }

        interp.return_string(is_active, &results);
    }
}

// #(rn,X,Y)
// ---------
// Rename file.  Rename file given by literal string "X" to "Y".
//
// Returns: null if successful, error text otherwise.
struct RnPrim;
impl MintPrim for RnPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        let from_name = args[1].value();
        let to_name = args[2].value();
        let from_str = String::from_utf8_lossy(from_name);
        let to_str = String::from_utf8_lossy(to_name);

        let result = match fs::rename(from_str.as_ref(), to_str.as_ref()) {
            Ok(_) => Vec::new(),
            Err(e) => format!("{}", e).into_bytes(),
        };

        interp.return_string(is_active, &result);
    }
}

// #(de,X)
// -------
// Delete file.  Delete file given by literal string "X".
//
// Returns: null if successful, error text otherwise.
struct DePrim;
impl MintPrim for DePrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        let file_name = args[1].value();
        let file_str = String::from_utf8_lossy(file_name);

        let result = match fs::remove_file(file_str.as_ref()) {
            Ok(_) => Vec::new(),
            Err(e) => format!("{}", e).into_bytes(),
        };

        interp.return_string(is_active, &result);
    }
}

// #(ev)
// -----
// Read environment.  This reads the operating system environment, and
// defines forms of the name "env.PATH" for each variable found in the
// environment.  In addition, the following forms are defined:
//     env.RUNLINE         The complete command line
//     env.SWITCHAR        The switch character (eg '-')
//     env.FULLPATH        The full path to the executable
//     env.SCREEN          The original contents of the screen
//
// Returns: null
struct EvPrim {
    argv: Vec<String>,
    envp: Vec<(String, String)>,
}

impl EvPrim {
    fn new(argv: &[String], envp: &[(String, String)]) -> Self {
        Self {
            argv: argv.to_vec(),
            envp: envp.to_vec(),
        }
    }
}

const ENV_SWITCHAR: &[u8] = b"env.SWITCHAR";
const SWITCHAR: &[u8] = b"-";
const ENV_SCREEN: &[u8] = b"env.SCREEN";
const ENV_FULLPATH: &[u8] = b"env.FULLPATH";
const ENV_RUNLINE: &[u8] = b"env.RUNLINE";

impl MintPrim for EvPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, _args: &MintArgList) {
        // Set switch character
        interp.set_form_value(ENV_SWITCHAR, SWITCHAR);

        // Set screen (empty - not available)
        interp.set_form_value(ENV_SCREEN, &Vec::new());

        // Set full path and run line
        if !self.argv.is_empty() {
            interp.set_form_value(ENV_FULLPATH, self.argv[0].as_bytes());
            let mut runline = Vec::new();
            for arg in self.argv.iter().skip(1) {
                runline.extend_from_slice(arg.as_bytes());
                runline.push(b' ');
            }
            interp.set_form_value(ENV_RUNLINE, &runline);
        }

        // Set environment variables
        for (key, value) in &self.envp {
            let mut form_name = b"env.".to_vec();
            form_name.extend_from_slice(key.as_bytes());
            interp.set_form_value(&form_name, value.as_bytes());
        }

        interp.return_null(is_active);
    }
}

// System variables

// sd - Swap directory
struct SdVar;
impl MintVar for SdVar {
    fn get_val(&self, _interp: &Mint) -> MintString {
        env::var("EMACSTMP")
            .or_else(|_| env::var("TMP"))
            .or_else(|_| env::var("TEMP"))
            .unwrap_or_else(|_| ".".to_string())
            .into_bytes()
    }

    fn set_val(&self, _interp: &mut Mint, _val: &MintString) {
        // Cannot be set
    }
}

// cd
// --
// Set/get the current working directory.
//
// FIXME: This should be a primitive that returns error status.
struct CdVar;
impl MintVar for CdVar {
    fn get_val(&self, _interp: &Mint) -> MintString {
        if let Ok(path) = env::current_dir() {
            let mut result = path.to_string_lossy().as_bytes().to_vec();
            if result.len() > 1 && result[result.len() - 1] != b'/' {
                result.push(b'/');
            }
            result
        } else {
            b"./".to_vec()
        }
    }

    fn set_val(&self, _interp: &mut Mint, val: &MintString) {
        let path_str = String::from_utf8_lossy(val);
        let _ = env::set_current_dir(path_str.as_ref());
    }
}

// cn
// --
// Get computer name/type.  This value cannot be set.
struct CnVar;
impl MintVar for CnVar {
    fn get_val(&self, _interp: &Mint) -> MintString {
        #[cfg(target_os = "windows")]
        let result = b"Windows".to_vec();

        #[cfg(not(target_os = "windows"))]
        let result = {
            use std::process::Command;
            if let Ok(output) = Command::new("uname").arg("-sr").output() {
                let s = String::from_utf8_lossy(&output.stdout).to_string();
                s.trim().as_bytes().to_vec()
            } else {
                b"Unknown".to_vec()
            }
        };

        result
    }

    fn set_val(&self, _interp: &mut Mint, _val: &MintString) {
        // Cannot be set
    }
}

// is
// --
// Get/set "inhibit snow" flag for IBM CGA.
// This isn't a thing in any sane world anymore.
struct IsVar;
impl MintVar for IsVar {
    fn get_val(&self, _interp: &Mint) -> MintString {
        b"0".to_vec()
    }

    fn set_val(&self, _interp: &mut Mint, _val: &MintString) {
        // Cannot be set
    }
}

// bp
// --
// Set the default bell pitch. If < 0 use visible bell.
struct BpVar;
impl MintVar for BpVar {
    fn get_val(&self, _interp: &Mint) -> MintString {
        b"440".to_vec()
    }

    fn set_val(&self, _interp: &mut Mint, _val: &MintString) {
        // Cannot be set
    }
}

// Helper function to format system time
fn format_system_time(time: SystemTime) -> String {
    use std::time::UNIX_EPOCH;

    if let Ok(duration) = time.duration_since(UNIX_EPOCH) {
        let secs = duration.as_secs();

        // Simple time formatting (similar to strftime)
        // This is a basic implementation - format: "Day Mon DD HH:MM:SS YYYY"
        use chrono::Local;
        use chrono::TimeZone;
        let dt = Local.timestamp_opt(secs as i64, 0).unwrap();
        dt.format("%a %b %d %H:%M:%S %Y").to_string()
    } else {
        String::new()
    }
}

pub fn register_sys_prims(interp: &mut Mint, argv: &[String], envp: &[(String, String)]) {
    interp.add_prim(b"ab".to_vec(), Box::new(AbPrim));
    interp.add_prim(b"hl".to_vec(), Box::new(HlPrim));
    interp.add_prim(b"ct".to_vec(), Box::new(CtPrim));
    interp.add_prim(b"ff".to_vec(), Box::new(FfPrim));
    interp.add_prim(b"rn".to_vec(), Box::new(RnPrim));
    interp.add_prim(b"de".to_vec(), Box::new(DePrim));
    interp.add_prim(b"ev".to_vec(), Box::new(EvPrim::new(argv, envp)));

    interp.add_var(b"bp".to_vec(), Box::new(BpVar));
    interp.add_var(b"cd".to_vec(), Box::new(CdVar));
    interp.add_var(b"cn".to_vec(), Box::new(CnVar));
    interp.add_var(b"is".to_vec(), Box::new(IsVar));
    interp.add_var(b"sd".to_vec(), Box::new(SdVar));
}
