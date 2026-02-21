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

use freemacs::buffer;
use freemacs::emacs_buffers;
use freemacs::emacs_window;
use freemacs::gap_buffer;
use freemacs::mint;

use freemacs::bufprim;
use freemacs::frmprim;
use freemacs::libprim;
use freemacs::mthprim;
use freemacs::strprim;
use freemacs::sysprim;
use freemacs::varprim;
use freemacs::winprim;

use std::env;

const INITIAL_STRING: &[u8] = b"#(rd)#(ow,(\n\
Freemacs, a programmable editor - Version )##(lv,vn)(\n\
Copyright (C) Martin Sandiford 2003\n\
MINT code copyright (C) Russell Nelson 1986-1998\n\
This is free software, and you are welcome to redistribute it\n\
under the conditions of the GNU General Public License.\n\
Type F1 C-c to see the conditions.\n\
))\
#(ds,Farglist,(SELF,arg1,arg2,arg3,arg4,arg5,arg6,arg7,arg8,arg9))\
#(ds,Fsearch-path,(#(SELF-do,##(fm,env.PATH,;,(##(gn,env.PATH,1000))))\
#(rs,env.PATH)))\
#(mp,Fsearch-path,#(Farglist))\
#(ds,Fsearch-path-do,(#(==,arg1,,,(\
\t#(==,#(ff,arg1/emacs.ed,;),,(\
\t\t#(SELF,##(fm,env.PATH,;,(##(gn,env.PATH,1000))))\
\t),(#(ds,env.EMACS,arg1/)))\
))))\
#(mp,Fsearch-path-do,#(Farglist))\
#(ev)\
#(n?,env.EMACS,(\
\t#(mp,env.EMACS,,/)\
\t#(ds,env.EMACS,##(env.EMACS,/))\
\t#(gn,env.EMACS,#(--,#(nc,##(env.EMACS)),1))\
\t#(==,##(go,env.EMACS)#(rs,env.EMACS),/,,(\
\t\t#(ds,env.EMACS,##(env.EMACS)/)\
\t))\
))\
#(n?,env.EMACS,,(\
\t#(ds,temp,##(env.FULLPATH))\
\t#(mp,temp,,emacs)\
\t#(==,#(ff,##(temp,emacs.ed),;),,,(\
\t\t#(ds,env.EMACS,##(temp))\
\t))\
))\
#(n?,env.EMACS,,(#(Fsearch-path)))\
#(an,Loading #(env.EMACS)emacs.ed...)\
#(==,#(ll,#(env.EMACS)emacs.ed),,(\
\t#(an,Starting editor...)\
\t#(##(lib-name)&setup)\
),(\
\t#(an)\
\t#(ow,(\n\
Cannot find the Freemacs .ED files))\
\t#(==,#(rf,#(env.EMACS)boot.min),,(\
\t\t#(ow,(, but we did find the boot files.\n\
Compiling the .ED files from the .MIN sources...\n\
))\
\t\t#(sp,[)#(rm,])#(dm,])\
\t),(\
\t\t#(ow,(\
. - Set the environment string EMACS to the subdirectory\n\
containing the Freemacs .ED files.  For example, EMACS=/emacs/\n\
Press any key to exit...))\
\t\t#(it,10000)#(ow,(\n))#(hl,1)\
\t))\
))";

fn new_window() -> Box<dyn emacs_window::EmacsWindow> {
    #[cfg(feature = "crossterm")]
    {
        use freemacs::emacs_window_crossterm;
        Box::new(emacs_window_crossterm::EmacsWindowCrossterm::new())
    }
    #[cfg(not(feature = "crossterm"))]
    {
        use freemacs::emacs_window_curses;
        Box::new(emacs_window_curses::EmacsWindowCurses::new())
    }
}

fn gap_buffer_factory() -> Box<dyn buffer::Buffer> {
    Box::new(gap_buffer::GapBuffer::with_default_size())
}

fn main() {
    emacs_buffers::init_buffers(gap_buffer_factory);
    emacs_window::init_window(new_window());

    let args: Vec<String> = env::args().collect();
    let envp: Vec<(String, String)> = env::vars().collect();

    let mut interp = mint::Mint::with_initial_string(INITIAL_STRING);

    bufprim::register_buf_prims(&mut interp);
    winprim::register_win_prims(&mut interp);
    mthprim::register_mth_prims(&mut interp);
    libprim::register_lib_prims(&mut interp);
    frmprim::register_frm_prims(&mut interp);
    strprim::register_str_prims(&mut interp);
    sysprim::register_sys_prims(&mut interp, &args, &envp);
    varprim::register_var_prims(&mut interp);

    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        loop {
            interp.scan();
        }
    })) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Exception: {:?}", e);
        }
    }
    emacs_window::free_window();
    emacs_buffers::free_buffers();
}
