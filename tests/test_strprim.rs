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

const OK: &str = "OK";

//
// Primitives from strprim.rs
//

#[test]
fn eq_prim() {
    // ==
    assert_eq!(OK, TestMint::new("#(ow,#(==,A,A,OK,BAD))").result());
    assert_eq!(OK, TestMint::new("#(ow,#(==,A,B,BAD,OK))").result());
}

#[test]
fn ne_prim() {
    // !=
    assert_eq!(OK, TestMint::new("#(ow,#(!=,A,A,BAD,OK))").result());
    assert_eq!(OK, TestMint::new("#(ow,#(!=,A,B,OK,BAD))").result());
}

#[test]
fn nc_prim() {
    assert_eq!("5", TestMint::new("#(ow,#(nc,hello))").result());
    assert_eq!("11", TestMint::new("#(ow,#(nc,hello hello))").result());
}

#[test]
fn ao_prim() {
    assert_eq!(OK, TestMint::new("#(ow,#(a?,A,A,OK,BAD))").result());
    assert_eq!(OK, TestMint::new("#(ow,#(a?,A,B,OK,BAD))").result());
    assert_eq!(OK, TestMint::new("#(ow,#(a?,AA,A,BAD,OK))").result());
}

#[test]
fn sa_prim() {
    assert_eq!(
        "b,c,m,n,v,x,z",
        TestMint::new("#(ow,##(sa,z,x,c,v,b,n,m))").result()
    );
}

#[test]
fn si_prim() {
    let input = concat!(
        "#(ds,xlat,(z0123456789))",
        "#(ow,##(si,xlat,(A\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0aZ)))"
    );
    assert_eq!("A0123456789Z", TestMint::new(input).result());
}

#[test]
fn nl_prim() {
    assert_eq!("\n", TestMint::new("#(ow,##(nl))").result());
}
