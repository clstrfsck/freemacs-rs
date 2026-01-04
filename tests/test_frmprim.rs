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
// Primitives from frmprim.rs
//

#[test]
fn ds_prim() {
    assert_eq!(
        "Test string",
        TestMint::new("#(ds,zz,Test string)#(ow,#(zz))").result()
    );
    assert_eq!(
        "Test string",
        TestMint::new("#(ds,zz,Test string)#(ow,##(zz))").result()
    );
}

#[test]
fn gs_prim() {
    assert_eq!(
        "Test string",
        TestMint::new("#(ds,zz,Test string)#(ow,#(gs,zz))").result()
    );
    assert_eq!(
        "Test string",
        TestMint::new("#(ds,zz,Test string)#(ow,##(gs,zz))").result()
    );
}

#[test]
fn go_prim() {
    assert_eq!("", TestMint::new("#(ds,zz,AB)#(ow,##(go,zzz,OK))").result());
    assert_eq!("A", TestMint::new("#(ds,zz,AB)#(ow,#(go,zz,OK))").result());
    assert_eq!("A", TestMint::new("#(ds,zz,AB)#(ow,##(go,zz,OK))").result());
    assert_eq!(
        "ABOK",
        TestMint::new("#(ds,zz,AB)#(ow,##(go,zz,OK)##(go,zz,OK)##(go,zz,OK))").result()
    );
    assert_eq!(
        "AOKB",
        TestMint::new("#(ds,zz,AB)#(ow,##(go,zz,OK)OK##(gs,zz))").result()
    );
}

#[test]
fn gn_prim() {
    // Behaviour as implemented (here, and in original Freemacs) does
    // not match the documentation in primitives.txt.  Implemented
    // behaviour is described below.

    // #(gn,X,Y,Z)
    // ---------
    // Get n.  Gets "Y" characters from form "X".  If there are less
    // than "Y" characters remaining between the form pointer and the
    // end of the form, fewer than "Y" characters will be returned. If
    // the form cannot be found, the null string is returned.  If the
    // form is found, and the form pointer is currently at the end of
    // the form, string "Z" is returned in active mode.  This is
    // approximately equivalent to the TRAC #(cn,...) primitive, only
    // argument markers appear to be returned in MINT.
    // Returns: Up to "Y" characters from the form at the form pointer.

    // Code from Freemacs for gn primitive for comparison is as follows:
    //
    // gn_prim:
    //         call    find_arg1
    //         jc      gn_prim_1       ;arg1 form not found -> null
    //         assume  ds:formSeg
    //         jcxz    gn_prim_2       ;form pointer empty -> #(arg3)
    //         push    ds              ;save pointer, count to form.
    //         push    si
    //         push    cx
    //         push    bx
    //         dsdata
    //         mov     cx,2            ;get number of chars to call.
    //         call    get_decimal_arg
    //         mov     dx,ax           ;save in dx.
    //         pop     bx
    //         pop     cx
    //         pop     si
    //         pop     ds
    //         assume  ds:formSeg
    //         di_points_fbgn
    //         cmp     dx,cx           ;are we trying to get more than exists?
    //         jbe     gn_prim_3       ;no - move the requested amount.
    //         mov     dx,cx           ;yes - truncate the count.
    // gn_prim_3:
    //         xchg    dx,cx           ;swap the count remaining and the get count.
    //         sub     dx,cx           ;dec the count remaining by the get count.
    //         chk_room_cnt es         ;check for collision
    //         movmem                  ;move all the chars.
    //         mov     cx,dx           ;return the count remaining in cx.
    //         jmp     return_form
    // gn_prim_2:
    //         dsdata
    //         mov     cx,3
    //         jmp     return_arg_active
    // gn_prim_1:
    //         jmp     return_null
    //         assume  ds:data, es:data

    assert_eq!(
        "",
        TestMint::new("#(ds,zz,AB)#(ow,#(gn,zzz,1,BAD))").result()
    );
    assert_eq!(
        "A",
        TestMint::new("#(ds,zz,AB)#(ow,#(gn,zz,1,BAD))").result()
    );
    assert_eq!(
        "A",
        TestMint::new("#(ds,zz,AB)#(ow,##(gn,zz,1,BAD))").result()
    );
    assert_eq!(
        "ABOK",
        TestMint::new("#(ds,zz,AB)#(ow,##(gn,zz,2,BAD)##(gn,zz,2,OK))").result()
    );
    assert_eq!(
        "AOKB",
        TestMint::new("#(ds,zz,AB)#(ow,##(gn,zz,1,BAD)OK##(gs,zz))").result()
    );
}

#[test]
fn rs_prim() {
    assert_eq!(
        "AAB",
        TestMint::new("#(ow,#(ds,zz,AB)#(go,zz,BAD)#(rs,zz)#(gs,zz,BAD))").result()
    );
}

#[test]
fn fm_prim() {
    assert_eq!(
        "AC",
        TestMint::new("#(ow,#(ds,zz,ABC)#(fm,zz,B,BAD)#(gs,zz,BAD))").result()
    );
    assert_eq!(
        "",
        TestMint::new("#(ow,#(ds,zz,ABC)#(fm,zzz,B,BAD))").result()
    );
    assert_eq!(
        "OK",
        TestMint::new("#(ow,#(ds,zz,ABC)#(fm,zz,,OK))").result()
    );
    assert_eq!(
        "OK",
        TestMint::new("#(ow,#(ds,zz,ABC)#(fm,zz,D,OK))").result()
    );
}

#[test]
fn nx_prim() {
    assert_eq!(
        "OK",
        TestMint::new("#(ow,#(ds,zz,ABC)#(n?,zz,OK,BAD))").result()
    );
    assert_eq!(
        "OK",
        TestMint::new("#(ow,#(ds,zz,ABC)#(n?,zzz,BAD,OK))").result()
    );
}

#[test]
fn ls_prim() {
    assert_eq!(
        "z,zz,zzz",
        TestMint::new("#(ow,#(ds,z,ABC)#(ds,zz,ABC)#(ds,zzz,ABC)##(sa,#(ls,(,),z)))").result()
    );
}

#[test]
fn es_prim() {
    assert_eq!(
        "OKOK",
        TestMint::new("#(ow,#(ds,zz,ABC)#(ds,zzz,ABC)#(es,zz)#(n?,zz,BAD,OK)#(n?,zzz,OK,BAD))")
            .result()
    );
    assert_eq!(
        "OKOK",
        TestMint::new("#(ow,#(ds,zz,ABC)#(ds,zzz,ABC)#(es,zz,zzz)#(n?,zz,BAD,OK)#(n?,zzz,BAD,OK))")
            .result()
    );
}

#[test]
fn mp_prim() {
    let input = concat!(
        "#(ow,",
        "#(ds,test,(Test SELF,ARG1,ARG2,ARG3))",
        "#(mp,test,SELF,ARG1,ARG2,ARG3)",
        "##(test,A,B,C)",
        ")"
    );
    assert_eq!("Test test,A,B,C", TestMint::new(input).result());
}

#[test]
fn hk_prim() {
    let input = concat!("#(ow,", "#(ds,z1,OK)", "##(hk,aa,bb,cc,dd,z1)", ")");
    assert_eq!(OK, TestMint::new(input).result());
}
