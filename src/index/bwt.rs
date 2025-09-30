pub fn bwt_from_sa(s:&[u8],sa:&[usize])->(Vec<u8>,usize){
    let n=s.len(); let mut bwt=Vec::with_capacity(n); let mut primary=0usize;
    for (r,&i) in sa.iter().enumerate(){ if i==0 { bwt.push(s[n-1]); primary=r; } else { bwt.push(s[i-1]); } }
    (bwt,primary)
}
#[cfg(test)] mod tests{ use super::*; #[test] fn bwt_banana(){ let s=b"banana$"; let sa=vec![6,5,3,1,0,4,2]; let (bwt,primary)=bwt_from_sa(s,&sa); assert_eq!(String::from_utf8(bwt).unwrap(),"annb$aa"); assert_eq!(primary,1); } }
