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

use crate::buffer::Buffer;
use crate::emacs_buffer::EmacsBuffer;
use crate::mint_types::{MintChar, MintCount, MintString};
use regex::bytes::{Regex, RegexBuilder};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

static S_BUFNO: AtomicUsize = AtomicUsize::new(1);

pub struct EmacsBuffers {
    buffer_factory: fn() -> Box<dyn Buffer>,
    current_buffer: Rc<RefCell<EmacsBuffer>>,
    buffers: HashMap<MintCount, Rc<RefCell<EmacsBuffer>>>,
    regex: Option<Regex>,
}

impl EmacsBuffers {
    pub fn new(factory: fn() -> Box<dyn Buffer>) -> Self {
        let bufno = S_BUFNO.fetch_add(1, Ordering::SeqCst) as MintCount;
        let init_buffer = Rc::new(RefCell::new(EmacsBuffer::new(bufno, factory())));
        let mut buffers = HashMap::new();
        buffers.insert(bufno, Rc::clone(&init_buffer));
        Self {
            buffer_factory: factory,
            current_buffer: Rc::clone(&init_buffer),
            buffers,
            regex: None,
        }
    }

    pub fn get_cur_buffer(&self) -> Rc<RefCell<EmacsBuffer>> {
        Rc::clone(&self.current_buffer)
    }

    pub fn new_buffer(&mut self) -> MintCount {
        let new_buffer = (self.buffer_factory)();
        let bufno = S_BUFNO.fetch_add(1, Ordering::SeqCst) as MintCount;
        self.current_buffer = Rc::new(RefCell::new(EmacsBuffer::new(bufno, new_buffer)));
        let bufno = self.current_buffer.borrow().get_buf_number();
        self.buffers.insert(bufno, Rc::clone(&self.current_buffer));
        bufno
    }

    pub fn select_buffer(&mut self, bufno: MintCount) -> bool {
        if let Some(buf) = self.buffers.get(&bufno) {
            self.current_buffer = Rc::clone(buf);
            true
        } else {
            false
        }
    }

    pub fn set_search_string(&mut self, s: &MintString, fold_case: bool) -> bool {
        if s.is_empty() {
            self.regex = None;
            return true;
        }

        match RegexBuilder::new(&regex::escape(&String::from_utf8_lossy(s)))
            .case_insensitive(fold_case)
            .build()
        {
            Ok(re) => {
                self.regex = Some(re);
                true
            }
            Err(_) => {
                self.regex = None;
                false
            }
        }
    }

    pub fn set_search_regex(&mut self, exp: &MintString, fold_case: bool) -> bool {
        if exp.is_empty() {
            self.regex = None;
            return true;
        }

        let exp_str = String::from_utf8_lossy(exp);
        match RegexBuilder::new(&exp_str)
            .case_insensitive(fold_case)
            .multi_line(true)
            .build()
        {
            Ok(re) => {
                self.regex = Some(re);
                true
            }
            Err(_) => {
                self.regex = None;
                false
            }
        }
    }

    pub fn search(&self, ss: MintChar, se: MintChar, ms: MintChar, me: MintChar) -> bool {
        let mut buf = self.current_buffer.borrow_mut();

        if self.regex.is_none() {
            if cfg!(debug_assertions) {
                eprintln!("Search called with no search string set");
            }
            if ms != 0 {
                buf.set_mark(ms, crate::emacs_buffer::MARK_POINT);
            }
            if me != 0 {
                buf.set_mark(me, crate::emacs_buffer::MARK_POINT);
            }
            return true;
        }

        let ss_n = buf.get_mark_position(ss).min(buf.size());
        let se_n = buf.get_mark_position(se).min(buf.size());

        if cfg!(debug_assertions) {
            eprintln!(
                "Search in buffer {} for {:?} from {} ({}) to {} ({})",
                buf.get_buf_number(),
                self.regex.as_ref().unwrap(),
                ss as char,
                ss_n,
                se as char,
                se_n
            );
        }

        if ss_n <= se_n {
            self.search_forward(&mut buf, ss_n, se_n, ms, me)
        } else {
            self.search_backward(&mut buf, ss_n, se_n, ms, me)
        }
    }

    fn search_forward(
        &self,
        buf: &mut EmacsBuffer,
        ss_n: MintCount,
        se_n: MintCount,
        ms: MintChar,
        me: MintChar,
    ) -> bool {
        self.regex
            .as_ref()
            .and_then(|re| buf.find_forward(re, ss_n as usize, se_n as usize))
            .map(|(match_start, match_end)| {
                if cfg!(debug_assertions) {
                    eprintln!(
                        "Found {:?} at ({}) to ({})",
                        self.regex.as_ref().unwrap(),
                        match_start,
                        match_end
                    );
                }
                if ms != 0 {
                    buf.set_mark_position(ms, match_start as MintCount);
                }
                if me != 0 {
                    buf.set_mark_position(me, match_end as MintCount);
                }
                true
            })
            .unwrap_or(false)
    }

    fn search_backward(
        &self,
        buf: &mut EmacsBuffer,
        ss_n: MintCount,
        se_n: MintCount,
        ms: MintChar,
        me: MintChar,
    ) -> bool {
        self.regex
            .as_ref()
            .and_then(|re| buf.find_backward(re, ss_n as usize, se_n as usize))
            .map(|(match_start, match_end)| {
                if ms != 0 {
                    buf.set_mark_position(ms, match_start as MintCount);
                }
                if me != 0 {
                    buf.set_mark_position(me, match_end as MintCount);
                }
                true
            })
            .unwrap_or(false)
    }
}

// FIXME: This should not be thread local.
thread_local! {
    static EMACS_BUFFERS: RefCell<Option<EmacsBuffers>> = const { RefCell::new(None) };
}

pub fn init_buffers(buffer_factory: fn() -> Box<dyn Buffer>) {
    EMACS_BUFFERS.with(|buffers| {
        *buffers.borrow_mut() = Some(EmacsBuffers::new(buffer_factory));
    });
}

pub fn free_buffers() {
    EMACS_BUFFERS.with(|buffers| {
        *buffers.borrow_mut() = None;
    });
    S_BUFNO.store(1, Ordering::SeqCst);
}

pub fn with_buffers<F, R>(f: F) -> R
where
    F: FnOnce(&mut EmacsBuffers) -> R,
{
    // buffers.borrow_mut().as_mut().unwrap()
    EMACS_BUFFERS.with(|buffers| f(buffers.borrow_mut().as_mut().unwrap()))
}

pub fn with_current_buffer<F, R>(f: F) -> R
where
    F: FnOnce(&mut EmacsBuffer) -> R,
{
    with_buffers(|buffers| {
        let buf_rc = buffers.get_cur_buffer();
        let mut buf = buf_rc.borrow_mut();
        f(&mut buf)
    })
}
