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

#[test]
fn test_integration() {
    let mut interp = TestMint::new(concat!(
        "#(ds,zz,(Fish fingers))",
        "#(ds,z1,(This is z1))",
        "#(ds,z2,(This is z2))",
        "#(ds,z3,(This is z3))",
        "#(ds,z4,(This is z4))",
        "#(ds,z5,(This is z5))",
        "#(ds,z6,(This is z6))",
        "#(ds,z7,(This is z7))",
        "#(ds,z8,(This is z8))",
        "#(ds,z9,(This is z9))",
        "#(ow,(##(zz) = )##(zz)(\n))",
        "#(ow,(##(gs,zz) = )##(gs,zz)(\n))",
        "#(ow,(##(go,zz) = )##(go,zz)(\n))",
        "#(ow,(##(go,zz) = )##(go,zz)(\n))",
        "#(ow,(##(gn,zz,5) = )##(gn,zz,5)(\n))",
        "#(ow,(##(gn,zz,5) = )##(gn,zz,5)(\n))",
        "#(ow,(##(go,zz,Failed) = )##(go,zz,Failed)(\n))",
        "#(ow,(##(go,kk,Failed) = )##(go,kk,Failed)(\n))",
        "#(rs,zz)#(ow,(##(gn,zz,4) = )##(gn,zz,4)(\n))",
        "#(ow,(##(fm,zz,h,Failed) = )##(fm,zz,h,Failed)(\n))",
        "#(rs,zz)#(ow,(##(fm,zz,h) = )##(fm,zz,h)(\n))",
        "#(ow,(12 + 15 = )##(++,(Fish 12),15)(\n))",
        "#(ow,(12 - 15 = )##(--,(Fish 12),15)(\n))",
        "#(ow,(12 * 15 = )##(**,(Fish 12),15)(\n))",
        "#(ow,(12 / 15 = )##(//,(Fish 12),15)(\n))",
        "#(ow,(12 % 15 = )##(%%,(Fish 12),15)(\n))",
        "#(ow,(12 & 15 = )##(&&,(Fish 12),15)(\n))",
        "#(ow,(12 | 15 = )##(||,(Fish 12),15)(\n))",
        "#(ow,(12 ^ 15 = )##(^^,(Fish 12),15)(\n))",
        "#(ow,#(g?,9,10,(#(ow,(9>10 true\n))),(#(ow,(9>10 false\n)))))",
        "#(ow,#(g?,10,9,(#(ow,(10>9 true\n))),(#(ow,(10>9 false\n)))))",
        "#(ow,(Before #(es,zz) ##(ls,(,),z) = )##(ls,(,),z)(\n))",
        "#(es,zz)",
        "#(ow,(After  #(es,zz) ##(ls,(,)) = )##(ls,(,))(\n))",
        "#(ow,(##(ct) = )##(ct)(\n))",
        "#(ow,(##(ct,mint) = )##(ct,mint)(\n))",
        "#(ow,(##(ct,.,z) = )##(ct,.,z)(\n))",
        "#(ow,(##(ct,mint,z) = )##(ct,mint,z)(\n))",
        "#(ow,(##(ct,/bin/cat,z) = )##(ct,/bin/cat,z)(\n))",
        "#(ow,(##(ct,/dev/null,z) = )##(ct,/dev/null,z)(\n))",
        "#(ds,test,(Test SELF,ARG1,ARG2,ARG3))#(mp,test,SELF,ARG1,ARG2,ARG3)",
        "#(ow,(Test mp: should be 'Test test,A,B,C' = ')##(test,A,B,C)('\n))",
        "#(ow,(Test hk: should be 'This is z1' = ')##(hk,aa,bb,cc,dd,z1)('\n))",
        "#(ds,xlat,(z0123456789))",
        "#(ow,(Test si: should be 'A0123456789Z' = ')",
        "##(si,xlat,(A\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0aZ))('\n))",
        "#(ow,(Test bc: should be '65' = ')##(bc,A)('\n))",
        "#(ow,(Test bc: should be '41' = ')##(bc,A,a,h)('\n))",
        "#(ow,(Test bc: should be '101' = ')##(bc,A,a,o)('\n))",
        "#(ow,(Test bc: should be '1000001' = ')##(bc,A,a,b)('\n))",
        "#(ow,(Test bc: should be 'Fish 41' = ')##(bc,Fish 65,d,h)('\n))",
        "#(ow,(Test bc: should be 'Fish 41' = ')##(bc,Fish 101,o,h)('\n))",
        "#(ow,(Test ff: ')##(ff,./mint*,(,))('\n))",
        "#(ow,(Test ff: ')##(ff,k*,(,))('\n))",
        "#(ow,(Test rn: ')##(rn,q,qq)('\n))",
        "#(ow,(Test rn: ')##(rn,qq,q)('\n))",
        "#(ow,(Test rn: ')##(rn,q,qq)('\n))",
        "#(ow,(Test de: ')##(de,qwerty)('\n))",
        "#(ow,(z: ')##(ls,(,),z)('\n))",
        "#(ow,(Test sl: ')##(sl,querty,#(ls,(,),z))('\n))",
        "#(ow,(Erase z*\n)##(es,#(ls,(,),z))",
        "#(ow,(z: ')##(ls,(,),z)('\n))",
        "#(ow,(Test ll: ')##(ll,querty)('\n))",
        "#(ow,(z: ')##(ls,(,),z)('\n))",
        "#(ev)",
        "#(ow,(Test ev: )##(ls,(,),env.)(\n))",
        "#(ow,(Test env.PWD: )##(env.PWD)(\n))",
        "#(ow,(Current buffer number: ')##(ba,-1)('\n))",
        "#(ds,buf,##(ba,-1))",
        "#(ow,(Create new buffer: ')##(ba,0)('\n))",
        "#(ow,(Current buffer number: ')##(ba,-1)('\n))",
        "#(ow,(Select old buffer: ')##(ba,##(buf))('\n))",
        "#(ow,(Current buffer number: ')##(ba,-1)('\n))",
        "#(ow,(Insert string 'hello' into buffer: )##(is,he,OK)( )#(is,llo,OK)(\n))",
        "#(pb)",
        "#(ow,(##(rm,[) should be 'hello': ')##(rm,[)('\n))",
        "#(ow,(##(rm,]) should be '': ')##(rm,])('\n))",
        "#(sp,<<<)",
        "#(ow,(##(rm,[) should be 'he': ')##(rm,[)('\n))",
        "#(ow,(##(rm,]) should be 'llo': ')##(rm,])('\n))",
        "#(ow,(##(rc,[) should be '2': ')##(rc,[)('\n))",
        "#(ow,(##(rc,]) should be '3': ')##(rc,])('\n))",
        "#(sp,[)#(wf,qwerty,])#(dm,])",
        "##(pb)##(rf,qwerty)##(pb)",
        "#(sp,[)#(dm,])#(pb)",
        "#(rf,qwerty)#(sp,]>>)",
        "##(pb)",
        "#(ow,(##(mb,<,True,False) should be 'True': ')##(mb,<,True,False)('\n))",
        "#(ow,(##(mb,>,True,False) should be 'False': ')##(mb,>,True,False)('\n))",
        "#(ow,(##(mb,.,True,False) should be 'False': ')##(mb,.,True,False)('\n))",
        "#(ow,(##(pm,1000,Oops) should be 'Oops': ')##(pm,1000,Oops)('\n))",
        "#(ow,(##(pm,10,Oops) should be '': ')##(pm,10,Oops)('\n))",
        "#(ow,(##(pm,10,Oops) should be '': ')##(pm,10,Oops)('\n))",
        "#(ow,(##(pm,10,Oops) should be '': ')##(pm,10,Oops)('\n))",
        "#(ow,(##(pm,10,Oops) should be '': ')##(pm,10,Oops)('\n))",
        "#(ow,(##(pm,10,Oops) should be 'Oops': ')##(pm,10,Oops)('\n))",
        "#(pm)#(pm)#(pm)",
        "#(ow,(##(pm,40,Oops) should be 'Oops': ')##(pm,40,Oops)('\n))",
        "#(ow,(##(pm,30,Oops) should be '': ')##(pm,30,Oops)('\n))",
        "#(pm)#(pm)#(pm)#(pm,10)",
        "#(sp,[)#(dm,])#(is,(This is a test string))",
        "#(sp,[>>>>)#(sm,0)#(sp,[)",
        "#(ow,(##(rm,0) should be 'This': ')##(rm,0)('\n))",
        "#(pm)",
        "#(ow,(##(rm,0) should be '': ')##(rm,0)('\n))",
        "#(sp,[>>>>)#(sm,@)#(sp,[)",
        "#(ow,(##(rm,@) should be 'This': ')##(rm,@)('\n))",
    ));

    interp.result();
    assert!(false);
}
