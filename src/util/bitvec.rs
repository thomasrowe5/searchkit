pub struct RankBitVec{bits:Vec<u64>,super_:Vec<u64>,block:Vec<u16>}
impl RankBitVec{
    pub fn from_bits(bits:Vec<u64>)->Self{
        let n64=bits.len(); let mut super_=Vec::with_capacity((n64+7)/8+1); let mut block=Vec::with_capacity(n64+1);
        let mut acc_super: u64=0; let mut acc_block:u16=0;
        for (i,&w) in bits.iter().enumerate(){
            if i%8==0{ super_.push(acc_super); acc_block=0; }
            block.push(acc_block);
            let pc=w.count_ones() as u16; acc_block=acc_block.wrapping_add(pc);
            if i%8==7{ acc_super+=acc_block as u64; }
        }
        block.push(acc_block); if n64%8!=0{ super_.push(acc_super); }
        Self{bits,super_,block}
    }
    #[inline] fn word_at(&self,i:usize)->u64{ self.bits[i] }
    pub fn rank1(&self,pos:usize)->u64{
        let word=pos>>6; let bit=pos&63; let sb=word>>3;
        let base=self.super_[sb]+self.block[word] as u64;
        let mask=if bit==63{u64::MAX}else{(1u64<<bit)-1};
        base+(self.word_at(word)&mask).count_ones() as u64
    }
}
#[cfg(test)] mod tests{ use super::*; #[test] fn small(){ let r=RankBitVec::from_bits(vec![0b01101001u8 as u64]); assert_eq!(r.rank1(0),0); assert_eq!(r.rank1(1),1); assert_eq!(r.rank1(4),2); } }
