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

mod test_mint;
use test_mint::TestMint;

//
// Primitives from bufprim.rs
//

#[test]
fn ba_prim() {
    // Note that the default buffer created by init_buffers is buffer 1.
    assert_eq!("1", TestMint::new("#(ow,#(ba,-1))").result());
    assert_eq!("2x3", TestMint::new("#(ow,#(ba)x#(ba,0))").result());
    assert_eq!(
        "2x1x1",
        TestMint::new("#(ow,#(ba)x#(ba,1)x#(ba,-1))").result()
    );
}
