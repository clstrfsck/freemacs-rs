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

const OK: &str = "OK";

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

struct MintTest {
    interp: Mint,
    output: Rc<RefCell<String>>,
}

impl MintTest {
    fn new(script: &str) -> Self {
        let mut interp = Mint::with_initial_string(script.as_bytes());
        let output = Rc::new(RefCell::new(String::new()));
        let ow_prim = OwPrim::new(output.clone());
        interp.add_prim(b"ow".to_vec(), Box::new(ow_prim));
        // Basic test cases written for these primitives
        freemacs::mthprim::register_mth_prims(&mut interp);
        freemacs::strprim::register_str_prims(&mut interp);
        freemacs::frmprim::register_frm_prims(&mut interp);

        // No tests yet written for these
        // freemacs::sysprim::register_sys_prims(&mut interp, 0, 0, 0);
        // freemacs::libprim::register_lib_prims(&mut interp);
        // freemacs::bufprim::register_buf_prims(&mut interp);

        MintTest { interp, output }
    }

    fn result(&mut self) -> String {
        self.interp.scan();
        self.output.borrow().clone()
    }
}

#[test]
fn ow_prim() {
    let mut test = MintTest::new("#(ow,OK)");
    assert_eq!(OK, test.result());
}

//
// Primitives from mthprim.cpp
//

#[test]
fn bc_prim() {
    // Character to dec, oct, hex, bin
    assert_eq!("64", MintTest::new("#(ow,#(bc,@,a,d))").result());
    assert_eq!("64", MintTest::new("#(ow,#(bc,@,c,d))").result());
    assert_eq!("100", MintTest::new("#(ow,#(bc,@,c,o))").result());
    assert_eq!("40", MintTest::new("#(ow,#(bc,@,c,h))").result());
    assert_eq!("1000000", MintTest::new("#(ow,#(bc,@,c,b))").result());
    // Decimal to character, oct, hex, bin
    assert_eq!("A", MintTest::new("#(ow,#(bc,65,d,a))").result());
    assert_eq!("A", MintTest::new("#(ow,#(bc,65,d,c))").result());
    assert_eq!("101", MintTest::new("#(ow,#(bc,65,d,o))").result());
    assert_eq!("41", MintTest::new("#(ow,#(bc,65,d,h))").result());
    assert_eq!("1000001", MintTest::new("#(ow,#(bc,65,d,b))").result());
}

#[test]
fn add_prim() {
    assert_eq!(
        "Prefix 15",
        MintTest::new("#(ow,##(++,(Prefix 12),3))").result()
    );
}

#[test]
fn sub_prim() {
    assert_eq!(
        "Prefix 9",
        MintTest::new("#(ow,##(--,(Prefix 12),3))").result()
    );
}

#[test]
fn mul_prim() {
    assert_eq!(
        "Prefix 36",
        MintTest::new("#(ow,##(**,(Prefix 12),3))").result()
    );
}

#[test]
fn div_prim() {
    assert_eq!(
        "Prefix 4",
        MintTest::new("#(ow,##(//,(Prefix 12),3))").result()
    );
}

#[test]
fn mod_prim() {
    assert_eq!(
        "Prefix 1",
        MintTest::new("#(ow,##(%%,(Prefix 13),3))").result()
    );
}

#[test]
fn ior_prim() {
    assert_eq!(
        "Prefix 15",
        MintTest::new("#(ow,##(||,(Prefix 13),3))").result()
    );
}

#[test]
fn and_prim() {
    assert_eq!(
        "Prefix 1",
        MintTest::new("#(ow,##(&&,(Prefix 13),3))").result()
    );
}

#[test]
fn xor_prim() {
    assert_eq!(
        "Prefix 14",
        MintTest::new("#(ow,##(^^,(Prefix 13),3))").result()
    );
}

#[test]
fn gt_prim() {
    assert_eq!(OK, MintTest::new("#(ow,#(g?,9,10,BAD,OK))").result());
}

//
// Primitives from strprim.cpp
//

#[test]
fn eq_prim() {
    // ==
    assert_eq!(OK, MintTest::new("#(ow,#(==,A,A,OK,BAD))").result());
    assert_eq!(OK, MintTest::new("#(ow,#(==,A,B,BAD,OK))").result());
}

#[test]
fn ne_prim() {
    // !=
    assert_eq!(OK, MintTest::new("#(ow,#(!=,A,A,BAD,OK))").result());
    assert_eq!(OK, MintTest::new("#(ow,#(!=,A,B,OK,BAD))").result());
}

#[test]
fn nc_prim() {
    assert_eq!("5", MintTest::new("#(ow,#(nc,hello))").result());
    assert_eq!("11", MintTest::new("#(ow,#(nc,hello hello))").result());
}

#[test]
fn ao_prim() {
    assert_eq!(OK, MintTest::new("#(ow,#(a?,A,A,OK,BAD))").result());
    assert_eq!(OK, MintTest::new("#(ow,#(a?,A,B,OK,BAD))").result());
    assert_eq!(OK, MintTest::new("#(ow,#(a?,AA,A,BAD,OK))").result());
}

#[test]
fn sa_prim() {
    assert_eq!(
        "b,c,m,n,v,x,z",
        MintTest::new("#(ow,##(sa,z,x,c,v,b,n,m))").result()
    );
}

#[test]
fn si_prim() {
    let input = concat!(
        "#(ds,xlat,(z0123456789))",
        "#(ow,##(si,xlat,(A\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0aZ)))"
    );
    assert_eq!("A0123456789Z", MintTest::new(input).result());
}

#[test]
fn nl_prim() {
    assert_eq!("\n", MintTest::new("#(ow,##(nl))").result());
}

//
// Primitives from frmprim.cpp
//

#[test]
fn ds_prim() {
    assert_eq!(
        "Test string",
        MintTest::new("#(ds,zz,Test string)#(ow,#(zz))").result()
    );
    assert_eq!(
        "Test string",
        MintTest::new("#(ds,zz,Test string)#(ow,##(zz))").result()
    );
}

#[test]
fn gs_prim() {
    assert_eq!(
        "Test string",
        MintTest::new("#(ds,zz,Test string)#(ow,#(gs,zz))").result()
    );
    assert_eq!(
        "Test string",
        MintTest::new("#(ds,zz,Test string)#(ow,##(gs,zz))").result()
    );
}

#[test]
fn go_prim() {
    assert_eq!("", MintTest::new("#(ds,zz,AB)#(ow,##(go,zzz,OK))").result());
    assert_eq!("A", MintTest::new("#(ds,zz,AB)#(ow,#(go,zz,OK))").result());
    assert_eq!("A", MintTest::new("#(ds,zz,AB)#(ow,##(go,zz,OK))").result());
    assert_eq!(
        "ABOK",
        MintTest::new("#(ds,zz,AB)#(ow,##(go,zz,OK)##(go,zz,OK)##(go,zz,OK))").result()
    );
    assert_eq!(
        "AOKB",
        MintTest::new("#(ds,zz,AB)#(ow,##(go,zz,OK)OK##(gs,zz))").result()
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
        MintTest::new("#(ds,zz,AB)#(ow,#(gn,zzz,1,BAD))").result()
    );
    assert_eq!(
        "A",
        MintTest::new("#(ds,zz,AB)#(ow,#(gn,zz,1,BAD))").result()
    );
    assert_eq!(
        "A",
        MintTest::new("#(ds,zz,AB)#(ow,##(gn,zz,1,BAD))").result()
    );
    assert_eq!(
        "ABOK",
        MintTest::new("#(ds,zz,AB)#(ow,##(gn,zz,2,BAD)##(gn,zz,2,OK))").result()
    );
    assert_eq!(
        "AOKB",
        MintTest::new("#(ds,zz,AB)#(ow,##(gn,zz,1,BAD)OK##(gs,zz))").result()
    );
}

#[test]
fn rs_prim() {
    assert_eq!(
        "AAB",
        MintTest::new("#(ow,#(ds,zz,AB)#(go,zz,BAD)#(rs,zz)#(gs,zz,BAD))").result()
    );
}

#[test]
fn fm_prim() {
    assert_eq!(
        "AC",
        MintTest::new("#(ow,#(ds,zz,ABC)#(fm,zz,B,BAD)#(gs,zz,BAD))").result()
    );
    assert_eq!(
        "",
        MintTest::new("#(ow,#(ds,zz,ABC)#(fm,zzz,B,BAD))").result()
    );
    assert_eq!(
        "OK",
        MintTest::new("#(ow,#(ds,zz,ABC)#(fm,zz,,OK))").result()
    );
    assert_eq!(
        "OK",
        MintTest::new("#(ow,#(ds,zz,ABC)#(fm,zz,D,OK))").result()
    );
}

#[test]
fn nx_prim() {
    assert_eq!(
        "OK",
        MintTest::new("#(ow,#(ds,zz,ABC)#(n?,zz,OK,BAD))").result()
    );
    assert_eq!(
        "OK",
        MintTest::new("#(ow,#(ds,zz,ABC)#(n?,zzz,BAD,OK))").result()
    );
}

#[test]
fn ls_prim() {
    assert_eq!(
        "z,zz,zzz",
        MintTest::new("#(ow,#(ds,z,ABC)#(ds,zz,ABC)#(ds,zzz,ABC)##(sa,#(ls,(,),z)))").result()
    );
}

#[test]
fn es_prim() {
    assert_eq!(
        "OKOK",
        MintTest::new("#(ow,#(ds,zz,ABC)#(ds,zzz,ABC)#(es,zz)#(n?,zz,BAD,OK)#(n?,zzz,OK,BAD))")
            .result()
    );
    assert_eq!(
        "OKOK",
        MintTest::new("#(ow,#(ds,zz,ABC)#(ds,zzz,ABC)#(es,zz,zzz)#(n?,zz,BAD,OK)#(n?,zzz,BAD,OK))")
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
    assert_eq!("Test test,A,B,C", MintTest::new(input).result());
}

#[test]
fn hk_prim() {
    let input = concat!("#(ow,", "#(ds,z1,OK)", "##(hk,aa,bb,cc,dd,z1)", ")");
    assert_eq!(OK, MintTest::new(input).result());
}

// int zmain(int, char **, char **) {
//     try {
//         Mint interp(
//             "#(ds,zz,(Fish fingers))"
//             "#(ds,z1,(This is z1))"
//             "#(ds,z2,(This is z2))"
//             "#(ds,z3,(This is z3))"
//             "#(ds,z4,(This is z4))"
//             "#(ds,z5,(This is z5))"
//             "#(ds,z6,(This is z6))"
//             "#(ds,z7,(This is z7))"
//             "#(ds,z8,(This is z8))"
//             "#(ds,z9,(This is z9))"
//             "#(ow,(##(zz) = )##(zz)(\n))"
//             "#(ow,(##(gs,zz) = )##(gs,zz)(\n))"
//             "#(ow,(##(go,zz) = )##(go,zz)(\n))"
//             "#(ow,(##(go,zz) = )##(go,zz)(\n))"
//             "#(ow,(##(gn,zz,5) = )##(gn,zz,5)(\n))"
//             "#(ow,(##(gn,zz,5) = )##(gn,zz,5)(\n))"
//             "#(ow,(##(go,zz,Failed) = )##(go,zz,Failed)(\n))"
//             "#(ow,(##(go,kk,Failed) = )##(go,kk,Failed)(\n))"
//             "#(rs,zz)#(ow,(##(gn,zz,4) = )##(gn,zz,4)(\n))"
//             "#(ow,(##(fm,zz,h,Failed) = )##(fm,zz,h,Failed)(\n))"
//             "#(rs,zz)#(ow,(##(fm,zz,h) = )##(fm,zz,h)(\n))"
//             "#(ow,(12 + 15 = )##(++,(Fish 12),15)(\n))"
//             "#(ow,(12 - 15 = )##(--,(Fish 12),15)(\n))"
//             "#(ow,(12 * 15 = )##(**,(Fish 12),15)(\n))"
//             "#(ow,(12 / 15 = )##(//,(Fish 12),15)(\n))"
//             "#(ow,(12 % 15 = )##(%%,(Fish 12),15)(\n))"
//             "#(ow,(12 & 15 = )##(&&,(Fish 12),15)(\n))"
//             "#(ow,(12 | 15 = )##(||,(Fish 12),15)(\n))"
//             "#(ow,(12 ^ 15 = )##(^^,(Fish 12),15)(\n))"
//             "#(ow,#(g?,9,10,(#(ow,(9>10 true\n))),(#(ow,(9>10 false\n)))))"
//             "#(ow,#(g?,10,9,(#(ow,(10>9 true\n))),(#(ow,(10>9 false\n)))))"
//             "#(ow,(Before #(es,zz) ##(ls,(,),z) = )##(ls,(,),z)(\n))"
//             "#(es,zz)"
//             "#(ow,(After  #(es,zz) ##(ls,(,)) = )##(ls,(,))(\n))"
//             "#(ow,(##(ct) = )##(ct)(\n))"
//             "#(ow,(##(ct,mint) = )##(ct,mint)(\n))"
//             "#(ow,(##(ct,.,z) = )##(ct,.,z)(\n))"
//             "#(ow,(##(ct,mint,z) = )##(ct,mint,z)(\n))"
//             "#(ow,(##(ct,/bin/cat,z) = )##(ct,/bin/cat,z)(\n))"
//             "#(ow,(##(ct,/dev/null,z) = )##(ct,/dev/null,z)(\n))"
//             "#(ds,test,(Test SELF,ARG1,ARG2,ARG3))#(mp,test,SELF,ARG1,ARG2,ARG3)"
//             "#(ow,(Test mp: should be 'Test test,A,B,C' = ')##(test,A,B,C)('\n))"
//             "#(ow,(Test hk: should be 'This is z1' = ')##(hk,aa,bb,cc,dd,z1)('\n))"
//             "#(ds,xlat,(z0123456789))"
//             "#(ow,(Test si: should be 'A0123456789Z' = ')"
//             "##(si,xlat,(A\001\002\003\004\005\006\007\010\011\012Z))('\n))"
//             "#(ow,(Test bc: should be '65' = ')##(bc,A)('\n))"
//             "#(ow,(Test bc: should be '41' = ')##(bc,A,a,h)('\n))"
//             "#(ow,(Test bc: should be '101' = ')##(bc,A,a,o)('\n))"
//             "#(ow,(Test bc: should be '1000001' = ')##(bc,A,a,b)('\n))"
//             "#(ow,(Test bc: should be 'Fish 41' = ')##(bc,Fish 65,d,h)('\n))"
//             "#(ow,(Test bc: should be 'Fish 41' = ')##(bc,Fish 101,o,h)('\n))"
//             "#(ow,(Test ff: ')##(ff,./mint*,(,))('\n))"
//             "#(ow,(Test ff: ')##(ff,k*,(,))('\n))"
//             "#(ow,(Test rn: ')##(rn,q,qq)('\n))"
//             "#(ow,(Test rn: ')##(rn,qq,q)('\n))"
//             "#(ow,(Test rn: ')##(rn,q,qq)('\n))"
//             "#(ow,(Test de: ')##(de,qwerty)('\n))"
//             "#(ow,(z: ')##(ls,(,),z)('\n))"
//             "#(ow,(Test sl: ')##(sl,querty,#(ls,(,),z))('\n))"
//             "#(ow,(Erase z*\n)##(es,#(ls,(,),z))"
//             "#(ow,(z: ')##(ls,(,),z)('\n))"
//             "#(ow,(Test ll: ')##(ll,querty)('\n))"
//             "#(ow,(z: ')##(ls,(,),z)('\n))"
//             "#(ev)"
//             "#(ow,(Test ev: )##(ls,(,),env.)(\n))"
//             "#(ow,(Test env.PWD: )##(env.PWD)(\n))"
//             "#(ow,(Current buffer number: ')##(ba,-1)('\n))"
//             "#(ds,buf,##(ba,-1))"
//             "#(ow,(Create new buffer: ')##(ba,0)('\n))"
//             "#(ow,(Current buffer number: ')##(ba,-1)('\n))"
//             "#(ow,(Select old buffer: ')##(ba,##(buf))('\n))"
//             "#(ow,(Current buffer number: ')##(ba,-1)('\n))"
//             "#(ow,(Insert string 'hello' into buffer: )##(is,he,OK)( )#(is,llo,OK)(\n))"
//             "#(pb)"
//             "#(ow,(##(rm,[) should be 'hello': ')##(rm,[)('\n))"
//             "#(ow,(##(rm,]) should be '': ')##(rm,])('\n))"
//             "#(sp,<<<)"
//             "#(ow,(##(rm,[) should be 'he': ')##(rm,[)('\n))"
//             "#(ow,(##(rm,]) should be 'llo': ')##(rm,])('\n))"
//             "#(ow,(##(rc,[) should be '2': ')##(rc,[)('\n))"
//             "#(ow,(##(rc,]) should be '3': ')##(rc,])('\n))"
//             "#(sp,[)#(wf,qwerty,])#(dm,])"
//             "##(pb)##(rf,qwerty)##(pb)"
//             "#(sp,[)#(dm,])#(pb)"
//             "#(rf,qwerty)#(sp,]>>)"
//             "##(pb)"
//             "#(ow,(##(mb,<,True,False) should be 'True': ')##(mb,<,True,False)('\n))"
//             "#(ow,(##(mb,>,True,False) should be 'False': ')##(mb,>,True,False)('\n))"
//             "#(ow,(##(mb,.,True,False) should be 'False': ')##(mb,.,True,False)('\n))"
//             "#(ow,(##(pm,1000,Oops) should be 'Oops': ')##(pm,1000,Oops)('\n))"
//             "#(ow,(##(pm,10,Oops) should be '': ')##(pm,10,Oops)('\n))"
//             "#(ow,(##(pm,10,Oops) should be '': ')##(pm,10,Oops)('\n))"
//             "#(ow,(##(pm,10,Oops) should be '': ')##(pm,10,Oops)('\n))"
//             "#(ow,(##(pm,10,Oops) should be '': ')##(pm,10,Oops)('\n))"
//             "#(ow,(##(pm,10,Oops) should be 'Oops': ')##(pm,10,Oops)('\n))"
//             "#(pm)#(pm)#(pm)"
//             "#(ow,(##(pm,40,Oops) should be 'Oops': ')##(pm,40,Oops)('\n))"
//             "#(ow,(##(pm,30,Oops) should be '': ')##(pm,30,Oops)('\n))"
//             "#(pm)#(pm)#(pm)#(pm,10)"
//             "#(sp,[)#(dm,])#(is,(This is a test string))"
//             "#(sp,[>>>>)#(sm,0)#(sp,[)"
//             "#(ow,(##(rm,0) should be 'This': ')##(rm,0)('\n))"
//             "#(pm)"
//             "#(ow,(##(rm,0) should be '': ')##(rm,0)('\n))"
//             "#(sp,[>>>>)#(sm,@)#(sp,[)"
//             "#(ow,(##(rm,@) should be 'This': ')##(rm,@)('\n))"
//             );
//         interp.addPrim("ow", std::make_shared<owPrim>(std::cout));
//         registerSysPrims(interp, 0, 0, 0);
//         registerStrPrims(interp);
//         registerFrmPrims(interp);
//         registerMthPrims(interp);
//         registerLibPrims(interp);

//         registerBufPrims(interp);

//         for (mintcount_t i = 0; i < 1; i++) {
//             interp.scan();
//         } // for
//     } catch (const std::exception& e) {
//         std::cerr << "Exception: " << e.what() << std::endl;
//     } // catch
//     return 0;
// } // main
