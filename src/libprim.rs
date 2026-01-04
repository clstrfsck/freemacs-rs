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
use std::fs::File;
use std::io::{Read, Write};

// Library file header structure
#[repr(C)]
#[derive(Debug)]
struct LibHdr {
    total_length: u32,
    name_length: u32,
    reserved: u32,
    form_pos: u32,
    data_length: u32,
}

impl LibHdr {
    const SIZE: usize = 20; // 5 * 4 bytes

    fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut bytes = [0u8; Self::SIZE];
        bytes[0..4].copy_from_slice(&self.total_length.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.name_length.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.reserved.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.form_pos.to_le_bytes());
        bytes[16..20].copy_from_slice(&self.data_length.to_le_bytes());
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::SIZE {
            return None;
        }
        Some(Self {
            total_length: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            name_length: u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            reserved: u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
            form_pos: u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
            data_length: u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]),
        })
    }
}

// #(sl,X,Y1,Y2,...,Yn)
// --------------------
// Save library.  Writes forms "Y1", ..., "Yn" complete with argument
// separators into file "X".
// File format is as follows:
//     Each form is written out with the following header:
//         word   Total form length, including header
//         word   Length of form name
//         word   Hash link -> only used while form in memory
//         word   Current form pointer (see #(go,X) etc)
//         word   Data length (size of form)
//     Followed by the form name
//     Followed by the form data, with parameter markers as byte 128+arg
//
// Returns: An error message if an error occurs, otherwise null.
struct SlPrim;
impl MintPrim for SlPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        if args.len() < 2 {
            interp.return_null(is_active);
            return;
        }

        let file_name = args[1].value();
        let file_name_str = String::from_utf8_lossy(file_name);

        // Try to create/open the file
        let mut file = match File::create(file_name_str.as_ref()) {
            Ok(f) => f,
            Err(e) => {
                let error_msg = format!("{}", e).into_bytes();
                interp.return_string(is_active, &error_msg);
                return;
            }
        };

        // Write each form (skip function name at index 0 and END marker at end)
        for arg in args.iter().take(args.len() - 1).skip(2) {
            let form_name = arg.value();

            if let Some(form) = interp.get_form(form_name) {
                let form_content = form.content();
                let form_pos = form.get_pos();

                // Create header
                let hdr = LibHdr {
                    total_length: (LibHdr::SIZE + form_name.len() + form_content.len()) as u32,
                    name_length: form_name.len() as u32,
                    reserved: 0,
                    form_pos,
                    data_length: form_content.len() as u32,
                };

                // Write header, name, and content
                if file.write_all(&hdr.to_bytes()).is_err()
                    || file.write_all(form_name).is_err()
                    || file.write_all(form_content).is_err()
                {
                    let error_msg = b"Write error".to_vec();
                    interp.return_string(is_active, &error_msg);
                    return;
                }
            }
        }

        // Success - return null
        interp.return_null(is_active);
    }
}

// #(ll,X)
// -------
// Load library.  Load library from file "X".  This library file should be
// in a form written by #(sl,...).
//
// Returns: Error message or null if no error.
struct LlPrim;
impl MintPrim for LlPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        if args.len() < 2 {
            interp.return_null(is_active);
            return;
        }

        let file_name = args[1].value();
        let file_name_str = String::from_utf8_lossy(file_name);

        // Try to open the file
        let mut file = match File::open(file_name_str.as_ref()) {
            Ok(f) => f,
            Err(e) => {
                let error_msg = format!("{}", e).into_bytes();
                interp.return_string(is_active, &error_msg);
                return;
            }
        };

        // Read entire file
        let mut buffer = Vec::new();
        if let Err(e) = file.read_to_end(&mut buffer) {
            let error_msg = format!("{}", e).into_bytes();
            interp.return_string(is_active, &error_msg);
            return;
        }

        // Parse the library file
        let mut offset = 0;
        while offset + LibHdr::SIZE <= buffer.len() {
            // Read header
            let hdr = match LibHdr::from_bytes(&buffer[offset..]) {
                Some(h) => h,
                None => break,
            };

            offset += LibHdr::SIZE;

            let name_len = hdr.name_length as usize;
            let data_len = hdr.data_length as usize;

            // Check we have enough data
            if offset + name_len + data_len > buffer.len() {
                break;
            }

            // Extract form name and content
            let form_name = buffer[offset..offset + name_len].to_vec();
            offset += name_len;

            let form_value = buffer[offset..offset + data_len].to_vec();
            offset += data_len;

            // Set the form in the interpreter
            interp.set_form_value(form_name.clone(), form_value);
            interp.set_form_pos(&form_name, hdr.form_pos);
        }

        // Success - return null
        interp.return_null(is_active);
    }
}

pub fn register_lib_prims(interp: &mut Mint) {
    interp.add_prim(b"ll".to_vec(), Box::new(LlPrim));
    interp.add_prim(b"sl".to_vec(), Box::new(SlPrim));
}
