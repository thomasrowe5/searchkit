use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

#[inline]
fn bm25_score(df: u32, tf: u32, doc_len: u32, avgdl: f32, n_docs: u32, k1: f32, b: f32) -> f32 {
    let idf = ((n_docs as f32 - df as f32 + 0.5) / (df as f32 + 0.5)).ln().max(0.0);
    let denom = tf as f32 + k1 * (1.0 - b + b * (doc_len as f32 / avgdl));
    (tf as f32 * (k1 + 1.0) / denom) * idf
}

#[derive(Debug, Clone)]
pub struct RankParams {
    pub k1: f32,
    pub b: f32,
    pub topk: usize,
}

impl Default for RankParams {
    fn default() -> Self {
        Self { k1: 1.2, b: 0.75, topk: 10 }
    }
}

#[derive(Debug)]
pub struct Hit<D> {
    pub doc: D,
    pub score: f32,
}

#[derive(Debug)]
struct HeapItem<D> {
    score: f32,
    doc: D,
}

impl<D: Eq> PartialEq for HeapItem<D> {
    fn eq(&self, other: &Self) -> bool { self.score.eq(&other.score) }
}
impl<D: Eq> Eq for HeapItem<D> {}
impl<D: Eq> PartialOrd for HeapItem<D> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { self.score.partial_cmp(&other.score) }
}
impl<D: Eq> Ord for HeapItem<D> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

pub fn rank_query<R, D>(
    reader: &R,
    query_terms: &[String],
    df_lookup: &mut dyn FnMut(&str) -> u32,
    doc_len_lookup: &dyn Fn(D) -> u32,
    n_docs: u32,
    avgdl: f32,
    params: RankParams,
) -> Vec<Hit<D>>
where
    R: Fn(&str) -> Vec<(D, u32)>,
    D: Copy + Eq + std::hash::Hash + Ord,
{
    let mut scores: HashMap<D, f32> = HashMap::new();

    for term in query_terms {
        let df = (df_lookup)(term);
        if df == 0 { continue; }
        let postings = reader(term);
        for (doc, tf) in postings {
            let dl = doc_len_lookup(doc);
            let s = bm25_score(df, tf, dl, avgdl, n_docs, params.k1, params.b);
            *scores.entry(doc).or_insert(0.0) += s;
        }
    }

    let mut heap: BinaryHeap<HeapItem<D>> = BinaryHeap::new();
    for (doc, score) in scores {
        heap.push(HeapItem { score, doc });
    }

    let mut out = Vec::with_capacity(params.topk);
    for _ in 0..params.topk {
        if let Some(HeapItem { score, doc }) = heap.pop() {
            out.push(Hit { doc, score });
        } else {
            break;
        }
    }
    out
}

