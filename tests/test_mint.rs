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

use std::cell::RefCell;
use std::rc::Rc;

use freemacs::mint::{Mint, MintPrim};
use freemacs::mint_arg::MintArgList;
use freemacs::{buffer, emacs_buffers, gap_buffer};

struct OwPrim {
    output: Rc<RefCell<String>>,
}

impl OwPrim {
    fn new(output: Rc<RefCell<String>>) -> Self {
        OwPrim { output }
    }
}

impl MintPrim for OwPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList) {
        let mut output = self.output.borrow_mut();
        for arg in args.iter().skip(1) {
            output.extend(String::from_utf8(arg.value().clone()).unwrap().chars());
        }
        interp.return_null(is_active);
    }
}

pub struct TestMint {
    interp: Mint,
    output: Rc<RefCell<String>>,
}

fn gap_buffer_factory() -> Box<dyn buffer::Buffer> {
    Box::new(gap_buffer::GapBuffer::with_default_size())
}

impl TestMint {
    pub fn new_with_env(script: &str, args: &[String], envp: &[(String, String)]) -> Self {
        let mut interp = Mint::with_initial_string(script.as_bytes());
        let output = Rc::new(RefCell::new(String::new()));
        let ow_prim = OwPrim::new(output.clone());
        interp.add_prim(b"ow".to_vec(), Box::new(ow_prim));

        emacs_buffers::init_buffers(gap_buffer_factory);

        freemacs::bufprim::register_buf_prims(&mut interp);
        freemacs::frmprim::register_frm_prims(&mut interp);
        freemacs::libprim::register_lib_prims(&mut interp);
        freemacs::mthprim::register_mth_prims(&mut interp);
        freemacs::strprim::register_str_prims(&mut interp);
        freemacs::sysprim::register_sys_prims(&mut interp, args, envp);
        freemacs::varprim::register_var_prims(&mut interp);
        // FIXME: Work out how to make this work without full windowing.
        // freemacs::winprim::register_win_prims(&mut interp);

        TestMint { interp, output }
    }

    pub fn new(script: &str) -> Self {
        TestMint::new_with_env(script, &[], &[])
    }

    pub fn result(&mut self) -> String {
        self.interp.scan();
        self.output.borrow().clone()
    }
}

impl Drop for TestMint {
    fn drop(&mut self) {
        emacs_buffers::free_buffers();
    }
}
