pub fn build_sa(s:&[u8])->Vec<usize>{
    let n=s.len(); let mut sa:(Vec<usize>)=(0..n).collect();
    let mut rank:Vec<i32>=s.iter().map(|&c| c as i32).collect(); let mut tmp=vec![0i32;n]; let mut k=1usize;
    while k<n{
        sa.sort_unstable_by(|&i,&j|{ (rank[i], if i+k<n{rank[i+k]} else {-1}).cmp(&(rank[j], if j+k<n{rank[j+k]} else {-1})) });
        tmp[sa[0]]=0;
        for i in 1..n{
            let a=sa[i-1]; let b=sa[i];
            let prev=(rank[a], if a+k<n{rank[a+k]} else {-1});
            let cur =(rank[b], if b+k<n{rank[b+k]} else {-1});
            tmp[b]=tmp[a]+ if cur>prev {1} else {0};
        }
        for i in 0..n { rank[i]=tmp[i]; }
        if rank[sa[n-1]] as usize == n-1 { break; }
        k<<=1;
    } sa
}
#[cfg(test)] mod tests{ use super::*; #[test] fn sa_basic(){ let s=b"banana$"; assert_eq!(build_sa(s),vec![6,5,3,1,0,4,2]); } }
