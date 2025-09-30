use anyhow::*;

/// RRR-style compressed flags: we store superblock prefix sums (every 512 bits)
/// and the positions of the 1-bits as u32s. rank1(pos) = super_counts[p/512] + count(ones <= pos in that window).
/// This is extremely small for sparse bitmaps (like "every k-th SA rank") and is fast enough.
#[derive(Clone)]
pub struct CompressedFlags {
    nbits: usize,
    super_every: usize,        // 512
    super_counts: Vec<u32>,    // prefix counts every 512 bits
    ones: Vec<u32>,            // absolute positions of 1-bits (sorted)
    // index to speed up local search: start offsets into `ones` per superblock
    super_offsets: Vec<u32>,   // index into `ones` for each superblock
}

impl CompressedFlags {
    pub fn build(nbits: usize, one_positions: &[usize]) -> Self {
        let super_every = 512;
        let n_super = (nbits + super_every - 1) / super_every + 1;

        let mut super_counts = vec![0u32; n_super];
        let mut ones_u32 = Vec::with_capacity(one_positions.len());
        for &p in one_positions {
            ones_u32.push(p as u32);
        }
        ones_u32.sort_unstable();

        // super_counts[i] = number of ones before bit i*512
        let mut idx = 0usize;
        for i in 0..n_super {
            let bound = i.saturating_mul(super_every) as u32;
            while idx < ones_u32.len() && ones_u32[idx] < bound {
                idx += 1;
            }
            super_counts[i] = idx as u32;
        }

        // For fast intra-superblock rank, store the starting index in `ones` for each super.
        let mut super_offsets = vec![0u32; n_super];
        for i in 0..n_super {
            super_offsets[i] = super_counts[i];
        }

        Self {
            nbits,
            super_every,
            super_counts,
            ones: ones_u32,
            super_offsets,
        }
    }

    /// rank1(pos): number of 1s in [0, pos), pos in bits.
    pub fn rank1(&self, pos: usize) -> u64 {
        if pos == 0 { return 0; }
        let pos_u = (pos - 1) as u32;
        let sb = pos / self.super_every;
        let base = self.super_counts[sb] as u64;

        // Search ones in [sb*512, pos)
        let start_idx = self.super_offsets[sb] as usize;
        let start_bit = (sb * self.super_every) as u32;

        // Binary search upper_bound on ones[..] for pos_u
        let slice = &self.ones[start_idx..];
        let mut lo = 0usize;
        let mut hi = slice.len();
        while lo < hi {
            let mid = (lo + hi) / 2;
            if slice[mid] < pos_u { lo = mid + 1; } else { hi = mid; }
        }

        // slice[..lo] are <= pos-1; ensure inside the current superblock
        let mut count = 0usize;
        for k in 0..lo {
            if slice[k] >= start_bit && slice[k] < pos_u { count += 1; }
        }
        base + count as u64
    }

    /// Serialize to bytes (little-endian):
    /// u64 nbits, u64 n_super, then n_super * u32 super_counts[]
    /// u64 n_ones, then n_ones * u32 ones[]
    pub fn save(&self, w: &mut impl std::io::Write) -> Result<()> {
        w.write_all(&(self.nbits as u64).to_le_bytes())?;
        w.write_all(&(self.super_counts.len() as u64).to_le_bytes())?;
        for &v in &self.super_counts {
            w.write_all(&v.to_le_bytes())?;
        }
        w.write_all(&(self.ones.len() as u64).to_le_bytes())?;
        for &v in &self.ones {
            w.write_all(&v.to_le_bytes())?;
        }
        Ok(())
    }

    pub fn load(r: &mut impl std::io::Read) -> Result<Self> {
        let mut b8 = [0u8; 8];
        let mut b4 = [0u8; 4];

        r.read_exact(&mut b8)?;
        let nbits = u64::from_le_bytes(b8) as usize;

        r.read_exact(&mut b8)?;
        let ns = u64::from_le_bytes(b8) as usize;
        let mut super_counts = vec![0u32; ns];
        for i in 0..ns {
            r.read_exact(&mut b4)?;
            super_counts[i] = u32::from_le_bytes(b4);
        }

        r.read_exact(&mut b8)?;
        let no = u64::from_le_bytes(b8) as usize;
        let mut ones = vec![0u32; no];
        for i in 0..no {
            r.read_exact(&mut b4)?;
            ones[i] = u32::from_le_bytes(b4);
        }

        // rebuild super_offsets as a copy of super_counts[]
        let super_offsets = super_counts.clone();

        Ok(Self {
            nbits,
            super_every: 512,
            super_counts,
            ones,
            super_offsets,
        })
    }
}
