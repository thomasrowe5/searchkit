pub fn kasai_lcp(s:&[u8],sa:&[usize])->Vec<usize>{
    let n=s.len(); let mut rank=vec![0usize;n]; for (r,&i) in sa.iter().enumerate(){ rank[i]=r; }
    let mut k=0usize; let mut lcp=vec![0usize;n];
    for i in 0..n{ let r=rank[i]; if r==0{ k=0; continue; } let j=sa[r-1]; while i+k<n && j+k<n && s[i+k]==s[j+k]{ k+=1; } lcp[r]=k; if k>0 { k-=1; } }
    lcp
}
#[cfg(test)] mod tests{ use super::*; #[test] fn lcp_basic(){ let s=b"banana$"; let sa=vec![6,5,3,1,0,4,2]; assert_eq!(kasai_lcp(s,&sa),vec![0,0,1,3,0,0,2]); } }
