pub mod util { pub mod varint; pub mod bitvec; pub mod mmap; pub mod timer; pub mod rrr; }
pub mod text { pub mod tokenize; pub mod corpus; }
pub mod index { pub mod inverted; pub mod suffix_array; pub mod lcp; pub mod bwt; pub mod fmindex; }
pub mod rank { pub mod bm25; }
pub mod query { pub mod boolean; pub mod phrase; pub mod substring; pub mod engine; }
