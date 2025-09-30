use crate::index::inverted::InvertedIndex;
pub fn phrase_query(inv:&InvertedIndex, terms:&[&str])->Vec<u32>{
    if terms.is_empty(){ return vec![]; }
    let mut cur:Vec<(u32,Vec<u32>)>=inv.postings(terms[0]);
    for &t in terms.iter().skip(1){
        let nxt=inv.postings(t);
        let mut merged=Vec::new(); let (mut a,mut b)=(0,0);
        while a<cur.len() && b<nxt.len(){
            match cur[a].0.cmp(&nxt[b].0){
                std::cmp::Ordering::Less => a+=1,
                std::cmp::Ordering::Greater => b+=1,
                std::cmp::Ordering::Equal => {
                    let (doc,ref pa)=cur[a]; let (_,ref pb)=nxt[b];
                    let (mut ia,mut ib)=(0,0); let mut hits=Vec::new();
                    while ia<pa.len() && ib<pb.len(){
                        if pa[ia]+1==pb[ib]{ hits.push(pb[ib]); ia+=1; ib+=1; }
                        else if pa[ia]+1<pb[ib]{ ia+=1; } else { ib+=1; }
                    }
                    if !hits.is_empty(){ merged.push((doc,hits)); }
                    a+=1; b+=1;
                }
            }
        } cur=merged;
    } cur.into_iter().map(|(d,_)| d).collect()
}
