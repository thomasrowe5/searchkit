use crate::util::bitvec::RankBitVec;
use crate::util::rrr::CompressedFlags;
use crate::util::varint::{decode_varint, encode_varint};
use anyhow::*;
use std::fs::File;
use std::io::{Read, Write};

pub struct FMIndex {
    pub c: [u64; 256],
    pub occ: Vec<RankBitVec>,    // rebuilt from BWT on load
    pub bwt: Vec<u8>,            // stored compressed-on-disk (RLE+varint)
    pub n: usize,
    pub sa_sample: usize,

    // Compressed SA sampling flags + positions
    samp_flags: CompressedFlags, // rank-accelerated sparse structure
    samp_pos: Vec<usize>,        // SA positions for sampled ranks (rank order)
}

pub struct MatchRange { pub l: u64, pub r: u64 }

impl FMIndex {
    pub fn build(text: &[u8], sa: &[usize], bwt: &[u8], sa_sample: usize) -> Self {
        let n = bwt.len();
        let sa_sample = sa_sample.max(1);

        // C array
        let mut freq = [0u64; 256];
        for &ch in bwt { freq[ch as usize] += 1; }
        let mut c = [0u64; 256];
        let mut acc = 0u64;
        for i in 0..256 { c[i] = acc; acc += freq[i]; }

        // occ bitvectors (in-memory)
        let words = (n + 63) / 64;
        let mut mats: Vec<Vec<u64>> = vec![vec![0u64; words]; 256];
        for (i, &ch) in bwt.iter().enumerate() {
            let w = i >> 6; let b = i & 63;
            mats[ch as usize][w] |= 1u64 << b;
        }
        let occ = mats.into_iter().map(RankBitVec::from_bits).collect::<Vec<_>>();

        // Sample flags + positions
        let mut one_positions = Vec::<usize>::new();
        let mut samp_pos = Vec::<usize>::new();
        for (rank, &pos) in sa.iter().enumerate() {
            if rank % sa_sample == 0 {
                one_positions.push(rank);
                samp_pos.push(pos);
            }
        }
        let samp_flags = CompressedFlags::build(n, &one_positions);

        debug_assert_eq!(text.last().copied(), Some(b"$"[0]));

        Self { c, occ, bwt: bwt.to_vec(), n, sa_sample, samp_flags, samp_pos }
    }

    #[inline] fn occ_rank(&self, ch: u8, i: u64) -> u64 { self.occ[ch as usize].rank1(i as usize) }

    #[inline]
    fn lf(&self, r: u64) -> u64 {
        let ch = self.bwt[r as usize];
        self.c[ch as usize] + self.occ_rank(ch, r)
    }

    pub fn backward_search(&self, pat: &[u8]) -> Option<MatchRange> {
        if pat.is_empty() { return Some(MatchRange{ l:0, r:self.n as u64 }); }
        let mut l = 0u64; let mut r = self.n as u64;
        for &ch in pat.iter().rev() {
            let base = self.c[ch as usize];
            l = base + self.occ_rank(ch, l);
            r = base + self.occ_rank(ch, r);
            if l >= r { return None; }
        }
        Some(MatchRange { l, r })
    }

    #[inline]
    fn is_sampled(&self, r: usize) -> bool {
        let before = self.samp_flags.rank1(r);
        let after  = self.samp_flags.rank1(r+1);
        after > before
    }

    #[inline]
    fn sampled_index(&self, r: usize) -> usize {
        (self.samp_flags.rank1(r+1) - 1) as usize
    }

    pub fn locate(&self, mut r: u64) -> usize {
        let mut steps = 0usize;
        loop {
            let ru = r as usize;
            if self.is_sampled(ru) {
                let idx = self.sampled_index(ru);
                let pos = self.samp_pos[idx];
                return (pos + steps) % self.n;
            }
            r = self.lf(r);
            steps += 1;
            debug_assert!(steps <= self.n);
            if steps > self.n { return 0; }
        }
    }

    pub fn locate_range(&self, range: &MatchRange, limit: usize) -> Vec<usize> {
        let take = limit.min((range.r - range.l) as usize);
        (range.l as usize..range.l as usize + take).map(|rr| self.locate(rr as u64)).collect()
    }

    // -------- serialization (v2) ----------

    /// Save to disk with:
    /// magic "FMX2"
    /// u64 n, u64 sa_sample
    /// C[256]*u64
    /// RLE-BWT: u64 rle_len, [ (u8 symbol, varint run_len) ... ]
    /// CompressedFlags (nbits, n_super, super[u32], n_ones, ones[u32])
    /// samp_pos varint-delta (u64)
    pub fn save(&self, path: &str) -> Result<()> {
        let mut f = File::create(path)?;

        // magic
        f.write_all(b"FMX2")?;

        // header
        f.write_all(&(self.n as u64).to_le_bytes())?;
        f.write_all(&(self.sa_sample as u64).to_le_bytes())?;

        // C
        for &v in &self.c { f.write_all(&v.to_le_bytes())?; }

        // BWT (RLE + varint)
        let rle = Self::encode_bwt_rle(&self.bwt);
        f.write_all(&(rle.len() as u64).to_le_bytes())?;
        f.write_all(&rle)?;

        // compressed flags
        self.samp_flags.save(&mut f)?;

        // samp_pos delta+varint (in rank order)
        let mut buf = Vec::new();
        let mut acc = 0u64;
        for (i, &p) in self.samp_pos.iter().enumerate() {
            let v = p as u64;
            let delta = if i == 0 { v } else { v - acc };
            encode_varint(delta, &mut buf);
            acc = v;
        }
        f.write_all(&(buf.len() as u64).to_le_bytes())?;
        f.write_all(&buf)?;
        Ok(())
    }

    pub fn load(path: &str) -> Result<Self> {
        let mut f = File::open(path)?;
        let mut magic = [0u8; 4];
        f.read_exact(&mut magic)?;
        ensure!(&magic == b"FMX2", "bad FM-index file (magic)");

        // n, sample
        let mut b8 = [0u8; 8];
        f.read_exact(&mut b8)?;
        let n = u64::from_le_bytes(b8) as usize;
        f.read_exact(&mut b8)?;
        let sa_sample = u64::from_le_bytes(b8) as usize;

        // C
        let mut c = [0u64; 256];
        for i in 0..256 { f.read_exact(&mut b8)?; c[i] = u64::from_le_bytes(b8); }

        // BWT (RLE)
        f.read_exact(&mut b8)?;
        let rle_len = u64::from_le_bytes(b8) as usize;
        let mut rle = vec![0u8; rle_len];
        f.read_exact(&mut rle)?;
        let bwt = Self::decode_bwt_rle(&rle, n)?;

        // Rebuild occ from BWT
        let words = (bwt.len() + 63) / 64;
        let mut mats: Vec<Vec<u64>> = vec![vec![0u64; words]; 256];
        for (i, &ch) in bwt.iter().enumerate() {
            let w = i >> 6; let b = i & 63;
            mats[ch as usize][w] |= 1u64 << b;
        }
        let occ = mats.into_iter().map(RankBitVec::from_bits).collect::<Vec<_>>();

        // compressed flags
        let samp_flags = CompressedFlags::load(&mut f)?;

        // samp_pos varint+delta
        f.read_exact(&mut b8)?;
        let vlen = u64::from_le_bytes(b8) as usize;
        let mut vbuf = vec![0u8; vlen];
        f.read_exact(&mut vbuf)?;
        let mut i = 0usize;
        let mut acc = 0u64;
        let mut samp_pos = Vec::<usize>::new();
        while i < vbuf.len() {
            let (d, j) = decode_varint(&vbuf, i)?;
            i = j; acc += d; samp_pos.push(acc as usize);
        }

        Ok(Self { c, occ, bwt, n, sa_sample: sa_sample.max(1), samp_flags, samp_pos })
    }

    // ---- BWT RLE (symbol, run_len varint) ----
    fn encode_bwt_rle(bwt: &[u8]) -> Vec<u8> {
        if bwt.is_empty() { return vec![]; }
        let mut out = Vec::new();
        let mut cur = bwt[0];
        let mut run: u64 = 1;
        for &ch in &bwt[1..] {
            if ch == cur && run < u64::MAX/2 {
                run += 1;
            } else {
                out.push(cur);
                encode_varint(run, &mut out);
                cur = ch; run = 1;
            }
        }
        out.push(cur);
        encode_varint(run, &mut out);
        out
    }
    fn decode_bwt_rle(buf: &[u8], n: usize) -> Result<Vec<u8>> {
        let mut i = 0usize;
        let mut out = Vec::with_capacity(n);
        while i < buf.len() {
            let sym = buf[i]; i += 1;
            let (run, j) = decode_varint(buf, i)?;
            i = j;
            for _ in 0..run { out.push(sym); }
        }
        ensure!(out.len() == n, "BWT RLE decode produced wrong length");
        Ok(out)
    }
}
