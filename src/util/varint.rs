use anyhow::*;
pub fn encode_varint(mut n: u64, out: &mut Vec<u8>) { while n >= 0x80 { out.push(((n as u8)&0x7F)|0x80); n >>= 7; } out.push(n as u8); }
pub fn decode_varint(bytes: &[u8], mut i: usize) -> Result<(u64, usize)> {
    let mut shift=0; let mut val:u64=0;
    loop {
        anyhow::ensure!(i<bytes.len(),"varint overflow");
        let b=bytes[i]; i+=1; val|=((b&0x7F) as u64)<<shift; shift+=7;
        if b&0x80==0 { return Ok((val,i)); }
        anyhow::ensure!(shift<=63,"varint too long");
    }
}
pub fn delta_encode(mut xs: Vec<u64>) -> Vec<u64> { if xs.is_empty(){return xs;} xs.sort_unstable(); let mut prev=0; for x in xs.iter_mut(){ let o=*x; *x-=prev; prev=o; } xs }
pub fn delta_decode(mut xs: Vec<u64>) -> Vec<u64> { let mut acc=0; for x in xs.iter_mut(){ acc+=*x; *x=acc; } xs }
pub fn encode_u64s_varint_delta(mut xs: Vec<u64>) -> Vec<u8> { let deltas=delta_encode(xs.drain(..).collect()); let mut out=Vec::with_capacity(deltas.len()*2); for d in deltas{ encode_varint(d,&mut out);} out }
pub fn decode_u64s_varint_delta(bytes: &[u8]) -> Result<Vec<u64>> { let mut i=0; let mut deltas=Vec::new(); while i<bytes.len(){ let (v,j)=decode_varint(bytes,i)?; i=j; deltas.push(v);} Ok(delta_decode(deltas)) }
#[cfg(test)] mod tests { use super::*; #[test] fn roundtrip(){ let xs=vec![1,2,3,10,100,1000,10000,100000]; let buf=encode_u64s_varint_delta(xs.clone()); let ys=decode_u64s_varint_delta(&buf).unwrap(); assert_eq!(xs,ys);} }
