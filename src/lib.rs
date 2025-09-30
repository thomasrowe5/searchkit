// src/lib.rs

// --- Utility modules ---
pub mod util {
    pub mod varint;
    pub mod mmap;
    pub mod timer;
    pub mod bitvec;
    pub mod rrr;
}

// --- Core text processing ---
pub mod text {
    pub mod corpus;
    pub mod tokenize;
}

// --- Index structures ---
pub mod index {
    pub mod inverted;
    pub mod fmindex;
    pub mod bwt;
    pub mod lcp;
    pub mod suffix_array;
}

// --- Query logic ---
pub mod query {
    pub mod boolean;
    pub mod phrase;
    pub mod substring;
    pub mod engine;
}

// --- Adapter layer ---
pub mod adapter {
    pub mod inv_adapter;
}

// --- Ranking ---
pub mod rank {
    pub mod bm25;
pub mod rank;
}

