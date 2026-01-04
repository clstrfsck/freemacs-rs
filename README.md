# Freemacs

* [Wikipedia entry](http://en.wikipedia.org/wiki/Freemacs)

## Description

Freemacs is an Emacs-style editor, originally written in assembler and a
[TRAC-like](http://en.wikipedia.org/wiki/TRAC_programming_language) language
called MINT.  The original assembler version ran on MS-DOS based systems and was
written by Russell Nelson.

Freemacs is interesting in as much as the assembler code portion is relatively
small, and most of the editing functionality is actually implemented in the
embedded macro language.

I originally re-implemented the interpreter portion in C++ circa 2010.
I feel like now, at the beginning of 2026, that it is time to re-implement
in Rust.

This is the initial Rust version.  There is quite a bit of stuff implemented,
but not yet complete enough to load and compile the `*.min` editor files.

If you want something that actually works right now, the C++
[Freemacs](https://github.com/clstrfsck/Freemacs) code is what you want.
Hopefully in the next few weeks there will be enough of this working that
the C++ code will be unnecessary.

## Installing

I'm using `rustc` 1.91.0 on a MacBook Pro, installed using `rustup`.
I think the rest you should be able to work out from `Cargo.toml`.

Once you have an executable, you will need to compile the MINT files.  This is
most easily accomplished by navigating into the "Editor" directory and executing
the compiled Freemacs.  This does not work right now for the Rust version.

There is no real install process beyond this.

## License

Russell Nelson's original Freemacs code (Editor/*.min) files are copyright
Russell Nelson and are GPL licensed.  It's not clearly stated in the
documentation, but based on the dates and timing, it really has to be GPL V2.

The remainder of the code in the src/** was written by me and is released under
GPL V2 as below.  The full text of the GPL V2 is in
[`gpl-2.0.txt`](gpl-2.0.txt).

This program is free software; you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation; either version 2 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program; if not, write to the Free Software
Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA  02110-1301  USA
