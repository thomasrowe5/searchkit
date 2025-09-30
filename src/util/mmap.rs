use memmap2::{Mmap, MmapMut, MmapOptions};
use std::fs::File;
use std::io::{Result, Write};

/// Open a file and return an owned read-only memory map.
/// Use it like: `let mmap = mmap_read(path)?; let bytes: &[u8] = &mmap[..];`
pub fn mmap_read(path: &str) -> Result<Mmap> {
    let file = File::open(path)?;
    // SAFETY: mapping read-only; file lives as long as mmap handle.
    unsafe { MmapOptions::new().map(&file) }
}

/// Create/resize a file and return an owned writable memory map plus the file handle.
pub fn mmap_create(path: &str, len: usize) -> Result<(File, MmapMut)> {
    let f = File::create(path)?;
    f.set_len(len as u64)?;
    // SAFETY: mapping writable; caller is responsible for flushing if needed.
    let mut m = unsafe { MmapOptions::new().len(len).map_mut(&f)? };
    // zero-init (optional; keep for determinism)
    for b in m.iter_mut() {
        *b = 0;
    }
    Ok((f, m))
}

/// Get file length in bytes.
pub fn file_len(path: &str) -> Result<u64> {
    let f = File::open(path)?;
    Ok(f.metadata()?.len())
}
