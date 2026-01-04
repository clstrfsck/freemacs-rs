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
// Primitives from mthprim.rs
//

#[test]
fn bc_prim() {
    // Character to dec, oct, hex, bin
    assert_eq!("64", TestMint::new("#(ow,#(bc,@,a,d))").result());
    assert_eq!("64", TestMint::new("#(ow,#(bc,@,c,d))").result());
    assert_eq!("100", TestMint::new("#(ow,#(bc,@,c,o))").result());
    assert_eq!("40", TestMint::new("#(ow,#(bc,@,c,h))").result());
    assert_eq!("1000000", TestMint::new("#(ow,#(bc,@,c,b))").result());
    // Decimal to character, oct, hex, bin
    assert_eq!("A", TestMint::new("#(ow,#(bc,65,d,a))").result());
    assert_eq!("A", TestMint::new("#(ow,#(bc,65,d,c))").result());
    assert_eq!("101", TestMint::new("#(ow,#(bc,65,d,o))").result());
    assert_eq!("41", TestMint::new("#(ow,#(bc,65,d,h))").result());
    assert_eq!("1000001", TestMint::new("#(ow,#(bc,65,d,b))").result());
}

#[test]
fn add_prim() {
    assert_eq!(
        "Prefix 15",
        TestMint::new("#(ow,##(++,(Prefix 12),3))").result()
    );
}

#[test]
fn sub_prim() {
    assert_eq!(
        "Prefix 9",
        TestMint::new("#(ow,##(--,(Prefix 12),3))").result()
    );
}

#[test]
fn mul_prim() {
    assert_eq!(
        "Prefix 36",
        TestMint::new("#(ow,##(**,(Prefix 12),3))").result()
    );
}

#[test]
fn div_prim() {
    assert_eq!(
        "Prefix 4",
        TestMint::new("#(ow,##(//,(Prefix 12),3))").result()
    );
}

#[test]
fn mod_prim() {
    assert_eq!(
        "Prefix 1",
        TestMint::new("#(ow,##(%%,(Prefix 13),3))").result()
    );
}

#[test]
fn ior_prim() {
    assert_eq!(
        "Prefix 15",
        TestMint::new("#(ow,##(||,(Prefix 13),3))").result()
    );
}

#[test]
fn and_prim() {
    assert_eq!(
        "Prefix 1",
        TestMint::new("#(ow,##(&&,(Prefix 13),3))").result()
    );
}

#[test]
fn xor_prim() {
    assert_eq!(
        "Prefix 14",
        TestMint::new("#(ow,##(^^,(Prefix 13),3))").result()
    );
}

#[test]
fn gt_prim() {
    assert_eq!(OK, TestMint::new("#(ow,#(g?,9,10,BAD,OK))").result());
}

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
