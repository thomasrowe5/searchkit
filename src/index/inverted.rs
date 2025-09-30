use std::collections::BTreeMap; use crate::util::varint::*;
#[derive(Default)] pub struct InvBuilder{ map:BTreeMap<String,BTreeMap<u32,Vec<u32>>> }
impl InvBuilder{
    pub fn add_doc(&mut self,docid:u32,text:&str){ for (tok,pos) in crate::text::tokenize::tokenize(text){ self.map.entry(tok).or_default().entry(docid).or_default().push(pos as u32); } }
    pub fn finalize(self)->InvertedIndex{ InvertedIndex::from_map(self.map) }
}
pub struct InvertedIndex{ pub dict:BTreeMap<String,Vec<u8>> }
impl InvertedIndex{
    fn from_map(map:BTreeMap<String,BTreeMap<u32,Vec<u32>>>)->Self{
        let mut dict=BTreeMap::new();
        for (term,docs) in map{
            let mut buf=Vec::new(); let mut last=0u64;
            for (docid,mut pos) in docs{
                pos.sort_unstable();
                let d=(docid as u64)-last; last=docid as u64; encode_varint(d,&mut buf);
                encode_varint(pos.len() as u64,&mut buf);
                let mut prev=0u64; for p in pos{ let delta=(p as u64)-prev; prev=p as u64; encode_varint(delta,&mut buf); }
            }
            dict.insert(term,buf);
        }
        Self{dict}
    }
    pub fn postings(&self,term:&str)->Vec<(u32,Vec<u32>)>{
        let Some(bytes)=self.dict.get(term) else { return vec![] };
        let mut i=0usize; let mut res=Vec::new(); let mut last=0u64;
        while i<bytes.len(){
            let (d,j1)=decode_varint(bytes,i).unwrap(); i=j1; let docid=(last+d) as u32; last+=d;
            let (freq,j2)=decode_varint(bytes,i).unwrap(); i=j2;
            let mut pos=Vec::with_capacity(freq as usize); let mut prev=0u64;
            for _ in 0..freq{ let (dv,j3)=decode_varint(bytes,i).unwrap(); i=j3; prev+=dv; pos.push(prev as u32); }
            res.push((docid,pos));
        }
        res
    }
}
#[cfg(test)] mod tests{ use super::*; #[test] fn build_and_read(){ let mut b=InvBuilder::default(); b.add_doc(1,"the cat sat on the mat"); b.add_doc(2,"the cat ate the rat"); let inv=b.finalize(); assert_eq!(inv.postings("cat").len(),2); } }
