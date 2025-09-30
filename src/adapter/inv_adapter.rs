// src/adapter/inv_adapter.rs
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::index::inverted::InvertedIndex;

/// Trait describing the minimum API needed by the ranking layer.
pub trait InvertedReader {
    type Term: AsRef<str>;
    type DocId: Copy + Eq + std::hash::Hash + Ord;

    fn num_docs(&self) -> u32;
    fn terms<'a>(&'a self) -> Box<dyn Iterator<Item = Self::Term> + 'a>;
    fn postings(&self, term: &str) -> Vec<(Self::DocId, u32)>;
    fn doc_len(&self, _doc: Self::DocId) -> Option<u32> {
        None
    }
}

/// A real adapter wrapping your on-disk or in-memory inverted index.
pub struct InvertedAdapter {
    pub index: InvertedIndex,
}

impl InvertedAdapter {
    /// Load the inverted index from a serialized file if you have one.
    /// For now, this expects a JSON or placeholder file – you can adapt this
    /// if you have a custom format. If you're only building in memory,
    /// you can create this directly with `InvertedAdapter { index }`.
    pub fn load_from_disk<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        // If you have a serialized format, deserialize it here.
        // If not, you can build the index in memory and pass it directly.
        let _data = fs::read(path)?; // placeholder
        // TODO: deserialize real index here if you have a serialized format
        // For now, we assume you've built it in memory elsewhere.
        panic!("TODO: implement real load logic or pass index directly.");
    }

    /// Wrap an in-memory index directly.
    pub fn from_index(index: InvertedIndex) -> Self {
        Self { index }
    }
}

impl InvertedReader for InvertedAdapter {
    type Term = String;
    type DocId = u32;

    fn num_docs(&self) -> u32 {
        // We can infer this by scanning postings once.
        let mut max_doc = 0;
        for (_term, postings) in self.index.dict.iter() {
            let mut i = 0usize;
            let mut last = 0u64;
            while i < postings.len() {
                let (d, j1) = crate::util::varint::decode_varint(postings, i).unwrap();
                i = j1;
                let docid = (last + d) as u32;
                last += d;

                let (freq, j2) = crate::util::varint::decode_varint(postings, i).unwrap();
                i = j2;

                for _ in 0..freq {
                    let (_, j3) = crate::util::varint::decode_varint(postings, i).unwrap();
                    i = j3;
                }

                if docid > max_doc {
                    max_doc = docid;
                }
            }
        }
        max_doc
    }

    fn terms<'a>(&'a self) -> Box<dyn Iterator<Item = Self::Term> + 'a> {
        Box::new(self.index.dict.keys().cloned())
    }

    fn postings(&self, term: &str) -> Vec<(Self::DocId, u32)> {
        self.index
            .postings(term)
            .into_iter()
            .map(|(doc, positions)| (doc, positions.len() as u32))
            .collect()
    }

    fn doc_len(&self, doc: Self::DocId) -> Option<u32> {
        let mut total = 0;
        for term in self.index.dict.keys() {
            for (d, positions) in self.index.postings(term) {
                if d == doc {
                    total += positions.len() as u32;
                }
            }
        }
        if total == 0 {
            None
        } else {
            Some(total)
        }
    }
}

/// Utility: compute all doc lengths up front for ranking.
pub fn compute_doc_lens<R: InvertedReader>(reader: &R) -> HashMap<R::DocId, u32> {
    let mut lens = HashMap::new();
    for term in reader.terms() {
        for (doc, tf) in reader.postings(term.as_ref()) {
            *lens.entry(doc).or_insert(0) += tf;
        }
    }
    lens
}

