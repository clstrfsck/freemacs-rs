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

/* Library entry so integration tests can depend on the crate API. */
pub mod buffer;
pub mod bufprim;
pub mod emacs_buffer;
pub mod emacs_buffers;
pub mod emacs_window;
pub mod emacs_window_crossterm;
pub mod emacs_window_curses;
pub mod emacs_window_debug;
pub mod frmprim;
pub mod gap_buffer;
pub mod libprim;
pub mod mint;
pub mod mint_arg;
pub mod mint_form;
pub mod mint_string;
pub mod mint_types;
pub mod mthprim;
pub mod strprim;
pub mod sysprim;
pub mod varprim;
pub mod winprim;
