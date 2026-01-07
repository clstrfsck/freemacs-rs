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

use crate::mint_arg::{ArgType, MintArg, MintArgList};
use crate::mint_form::MintForm;
use crate::mint_types::{MintChar, MintCount, MintString};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

pub trait MintPrim {
    fn execute(&self, interp: &mut Mint, is_active: bool, args: &MintArgList);
}

pub trait MintVar {
    fn get_val(&self, interp: &Mint) -> MintString;
    fn set_val(&self, interp: &mut Mint, val: &MintString);
}

struct ActiveString {
    data: VecDeque<MintChar>,
}

impl ActiveString {
    fn new() -> Self {
        Self {
            data: VecDeque::new(),
        }
    }

    fn push_front(&mut self, s: &[MintChar]) {
        for &ch in s.iter().rev() {
            self.data.push_front(ch);
        }
    }

    fn push_front_char(&mut self, ch: MintChar) {
        self.data.push_front(ch);
    }

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    fn clear(&mut self) {
        self.data.clear();
    }

    fn load(&mut self, s: &[MintChar]) {
        self.data.clear();
        self.data.extend(s.iter().copied());
    }

    fn drain<R>(&mut self, range: R) -> std::collections::vec_deque::Drain<'_, MintChar>
    where
        R: std::ops::RangeBounds<usize>,
    {
        self.data.drain(range)
    }
}

struct NeutralString {
    args: VecDeque<MintArg>,
    last_func: usize,
}

impl NeutralString {
    fn new() -> Self {
        let mut ns = Self {
            args: VecDeque::new(),
            last_func: 0,
        };
        ns.clear();
        ns
    }

    fn clear(&mut self) {
        self.args.clear();
        self.args.push_back(MintArg::new(ArgType::Null));
        self.save_func();
    }

    fn append(&mut self, ch: MintChar) {
        if let Some(arg) = self.args.front_mut() {
            arg.append(ch);
        }
    }

    fn append_slice(&mut self, s: &[MintChar]) {
        if let Some(arg) = self.args.front_mut() {
            arg.append_slice(s);
        }
    }

    fn mark_argument(&mut self) {
        self.args.push_front(MintArg::new(ArgType::Arg));
        self.increment_last_func();
    }

    fn mark_active_function(&mut self) {
        self.args.push_front(MintArg::new(ArgType::Active));
        self.save_func();
    }

    fn mark_neutral_function(&mut self) {
        self.args.push_front(MintArg::new(ArgType::Neutral));
        self.save_func();
    }

    fn mark_end_function(&mut self) {
        self.args.push_front(MintArg::new(ArgType::End));
        self.increment_last_func();
    }

    fn save_func(&mut self) {
        self.last_func = 1;
    }

    fn increment_last_func(&mut self) {
        self.last_func += 1;
    }

    fn pop_arguments(&mut self) -> MintArgList {
        let at = self.last_func.min(self.args.len());
        let mut result = MintArgList::new();
        for _ in 0..at {
            if let Some(arg) = self.args.pop_front() {
                result.push_front(arg);
            }
        }

        if self.args.is_empty() {
            self.clear();
        } else {
            self.last_func = self
                .args
                .iter()
                .position(|arg| arg.is_term())
                .map(|p| p + 1)
                .unwrap_or(1);
        }

        result
    }
}

pub struct Mint {
    idle_max: i32,
    idle_count: i32,
    idle_string: MintString,
    default_string_key: MintString,
    default_string_nokey: MintString,
    active_string: ActiveString,
    neutral_string: NeutralString,
    forms: HashMap<MintString, MintForm>,
    vars: HashMap<MintString, Rc<Box<dyn MintVar>>>,
    prims: HashMap<MintString, Rc<Box<dyn MintPrim>>>,
}

impl Default for Mint {
    fn default() -> Self {
        Self::new()
    }
}

const DEFAULT_STRING_KEY: &[MintChar] = b"#(d,#(g))";
const DEFAULT_STRING_NO_KEY: &[MintChar] = b"#(k)#(d,#(g))";
const DFLTA: &[MintChar] = b"dflta";
const DFLTN: &[MintChar] = b"dfltn";

impl Mint {
    pub fn new() -> Self {

        let mut mint = Self {
            idle_max: 0,
            idle_count: 0,
            idle_string: Vec::new(),
            default_string_key: DEFAULT_STRING_KEY.to_vec(),
            default_string_nokey: DEFAULT_STRING_NO_KEY.to_vec(),
            active_string: ActiveString::new(),
            neutral_string: NeutralString::new(),
            forms: HashMap::new(),
            vars: HashMap::new(),
            prims: HashMap::new(),
        };

        mint.active_string.push_front(DEFAULT_STRING_NO_KEY);
        mint
    }

    pub fn with_initial_string(s: &[MintChar]) -> Self {
        let mut mint = Self::new();
        mint.active_string.clear();
        mint.active_string.push_front(s);
        mint
    }

    pub fn add_var(&mut self, name: MintString, var: Box<dyn MintVar>) {
        self.vars.insert(name, Rc::new(var));
    }

    pub fn add_prim(&mut self, name: MintString, prim: Box<dyn MintPrim>) {
        self.prims.insert(name, Rc::new(prim));
    }

    pub fn get_var(&self, var_name: &MintString) -> MintString {
        let var = self.vars.get(var_name).map(|v| v.get_val(self));
        if cfg!(debug_assertions) && var.is_none() {
            eprintln!(
                "Can't find variable '{:?}' while reading",
                String::from_utf8_lossy(var_name)
            );
        }
        var.unwrap_or_default()
    }

    pub fn set_var(&mut self, var_name: &MintString, val: &MintString) {
        if let Some(var) = self.vars.get(var_name).cloned() {
            var.set_val(self, val);
        } else if cfg!(debug_assertions) {
            eprintln!(
                "Can't find variable '{:?}' while writing",
                String::from_utf8_lossy(var_name)
            );
        }
    }

    pub fn return_null(&self, _is_active: bool) {
        if cfg!(debug_assertions) {
            eprintln!(
                "** Function ({}) returned null string",
                if _is_active { "A" } else { "N" }
            );
        }
    }

    pub fn return_string(&mut self, is_active: bool, s: &MintString) {
        if cfg!(debug_assertions) {
            eprintln!(
                "** Function ({}) returned: {}",
                if is_active { "A" } else { "N" },
                String::from_utf8_lossy(s)
            );
        }
        if is_active {
            self.active_string.push_front(s);
        } else {
            self.neutral_string.append_slice(s);
        }
    }

    pub fn return_integer(&mut self, is_active: bool, n: i32, base: i32) {
        let mut s = Vec::new();
        crate::mint_string::append_num(&mut s, n, base);
        self.return_string(is_active, &s);
    }

    pub fn return_integer_with_prefix(
        &mut self,
        is_active: bool,
        prefix: &MintString,
        n: i32,
        base: i32,
    ) {
        let mut s = prefix.clone();
        crate::mint_string::append_num(&mut s, n, base);
        self.return_string(is_active, &s);
    }

    pub fn return_n_form(
        &mut self,
        is_active: bool,
        form_name: &MintString,
        n: i32,
        not_found: &MintString,
    ) {
        if let Some(form) = self.get_form_mut(form_name) {
            if form.at_end() {
                self.return_string(true, not_found);
            } else {
                let result = form.get_n(n);
                self.return_string(is_active, &result);
            }
        } else {
            self.return_null(is_active);
        }
    }

    pub fn return_form_list(&mut self, is_active: bool, sep: &MintString, prefix: &MintString) {
        let mut form_names: Vec<&MintString> = if !prefix.is_empty() {
            // Collect and sort form names that match prefix
            self.forms
                .keys()
                .filter(|name| name.starts_with(prefix))
                .collect()
        } else {
            self.forms.keys().collect()
        };
        form_names.sort();
        let mut need_sep = false;
        let mut result = Vec::new();
        for form_name in form_names {
            if need_sep {
                result.extend_from_slice(sep);
            }
            result.extend_from_slice(form_name);
            need_sep = true;
        }
        self.return_string(is_active, &result);
    }

    pub fn set_idle_max(&mut self, n: i32) {
        if n > 0 {
            self.idle_max = n;
            self.idle_count = n;
        } else {
            self.idle_max = 0;
            self.idle_count = 0;
        }
    }

    pub fn get_idle_max(&self) -> i32 {
        self.idle_max
    }

    pub fn set_form_pos(&mut self, form_name: &MintString, n: MintCount) {
        if let Some(form) = self.forms.get_mut(form_name) {
            form.set_pos(n);
        }
    }

    pub fn get_form(&self, form_name: &MintString) -> Option<&MintForm> {
        self.forms.get(form_name)
    }

    pub fn get_form_mut(&mut self, form_name: &MintString) -> Option<&mut MintForm> {
        self.forms.get_mut(form_name)
    }

    pub fn del_form(&mut self, form_name: &MintString) {
        self.forms.remove(form_name);
    }

    pub fn set_form_value(&mut self, form_name: &MintString, value: &MintString) {
        self.forms
            .insert(form_name.clone(), MintForm::from_string(value));
    }

    pub fn scan(&mut self) {
        if self.active_string.is_empty() {
            self.neutral_string.clear();
            if !self.idle_string.is_empty() {
                self.active_string.load(&self.idle_string.clone());
                self.idle_string.clear();
            } else {
                let default = if key_waiting() {
                    &self.default_string_key
                } else {
                    &self.default_string_nokey
                };
                self.active_string.load(default);
            }
        }

        let mut pos = 0;
        while pos < self.active_string.data.len() {
            let ch = self.active_string.data[pos];
            match ch {
                b'\t' | b'\r' | b'\n' => {
                    pos += 1;
                }
                b'(' => {
                    if !self.copy_to_close_paren(&mut pos) {
                        return;
                    }
                }
                b',' => {
                    pos += 1;
                    self.neutral_string.mark_argument();
                }
                b'#' => {
                    if pos + 1 < self.active_string.data.len()
                        && self.active_string.data[pos + 1] == b'('
                    {
                        pos += 2;
                        self.neutral_string.mark_active_function();
                    } else if pos + 2 < self.active_string.data.len()
                        && self.active_string.data[pos + 1] == b'#'
                        && self.active_string.data[pos + 2] == b'('
                    {
                        pos += 3;
                        self.neutral_string.mark_neutral_function();
                    } else {
                        self.neutral_string.append(b'#');
                        pos += 1;
                    }
                }
                b')' => {
                    pos += 1;
                    self.active_string.drain(0..pos);
                    if !self.execute_function() {
                        return;
                    }
                    pos = 0;
                }
                _ => {
                    self.neutral_string.append(ch);
                    pos += 1;
                }
            }
        }
        self.active_string.clear();
    }

    fn copy_to_close_paren(&mut self, start: &mut usize) -> bool {
        let mut parens = 1;
        let mut next = *start + 1;

        while parens > 0 {
            if next >= self.active_string.data.len() {
                return false;
            }
            let ch = self.active_string.data[next];
            next += 1;
            match ch {
                b'(' => parens += 1,
                b')' => parens -= 1,
                _ => {}
            }
        }

        let content: Vec<MintChar> = self
            .active_string
            .data
            .iter()
            .skip(*start + 1)
            .take(next - *start - 2)
            .copied()
            .collect();
        self.neutral_string.append_slice(&content);
        *start = next;
        true
    }

    fn execute_function(&mut self) -> bool {
        self.neutral_string.mark_end_function();
        let args = self.neutral_string.pop_arguments();

        if args.is_empty() || args[0].arg_type() == ArgType::Null {
            return false;
        }

        let is_active = args[0].arg_type() == ArgType::Active;
        let func_name = args[0].value();

        if let Some(prim) = self.prims.get(func_name) {
            if cfg!(debug_assertions) {
                eprintln!(
                    "Execute function: {} with {} arguments",
                    String::from_utf8_lossy(func_name),
                    args.len() - 1
                );
                for (argn, arg) in args.iter().enumerate().skip(1) {
                    eprintln!(
                        "  Arg {} ({}): {}",
                        argn,
                        arg.arg_type() as u8,
                        String::from_utf8_lossy(arg.value())
                    );
                }
            }
            prim.clone().execute(self, is_active, &args);
        } else if let Some(form) = self.forms.get(func_name) {
            let pos = form.get_pos();
            let content = form.content()[pos as usize..].to_vec();
            self.return_seg_string(is_active, &content, &args);
        } else {
            let default_name: &[MintChar] = if is_active { DFLTA } else { DFLTN };
            if let Some(form) = self.forms.get(default_name) {
                let pos = form.get_pos();
                let content = form.content()[pos as usize..].to_vec();
                self.return_seg_string(is_active, &content, &args);
            }
        }

        true
    }

    pub fn return_seg_string(&mut self, is_active: bool, ss: &MintString, args: &MintArgList) {
        let last_index = args.len().saturating_sub(1);
        let get_arg = |index: usize| args[index].value();

        if is_active {
            for &ch in ss.iter().rev() {
                if ch >= 0x80 {
                    let index = (ch - 0x80).min(last_index as u8) as usize;
                    self.active_string.push_front(get_arg(index));
                } else {
                    self.active_string.push_front_char(ch);
                }
            }
        } else {
            for &ch in ss.iter() {
                if ch >= 0x80 {
                    let index = (ch - 0x80).min(last_index as u8) as usize;
                    self.neutral_string.append_slice(get_arg(index));
                } else {
                    self.neutral_string.append(ch);
                }
            }
        }
    }
}

fn key_waiting() -> bool {
    crate::winprim::key_waiting()
}
