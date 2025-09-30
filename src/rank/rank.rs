use std::collections::HashMap;
use crate::rank::bm25::BM25;

#[derive(Clone, Debug)]
pub struct RankParams {
    pub avgdl: f32,
    pub topk: usize,
}

#[derive(Clone, Debug)]
pub struct RankedDoc {
    pub doc_id: u32,
    pub score: f32,
}

/// Rank documents for a tokenized query using BM25.
///
/// Inputs:
/// - `postings_fn(term) -> Vec<(doc_id, tf)>`
/// - `terms`: query tokens (already lowercased / tokenized)
/// - `lens`: map of doc_id -> document length (sum of term frequencies)
/// - `n_docs`: total number of docs
/// - `params`: BM25 params (avgdl, topk)
pub fn rank_query<F>(
    postings_fn: &F,
    terms: &[String],
    lens: &HashMap<u32, u32>,
    n_docs: u32,
    params: &RankParams,
) -> Vec<RankedDoc>
where
    F: Fn(&str) -> Vec<(u32, u32)>,
{
    // Safe BM25 (avgdl clamped)
    let bm25 = BM25 {
        k1: 1.5,
        b: 0.75,
        avgdl: if params.avgdl > 0.0 { params.avgdl } else { 1.0 },
    };

    let mut scores: HashMap<u32, f32> = HashMap::new();

    for term in terms {
        let postings = postings_fn(term);
        let df = postings.len() as f32;
        if df == 0.0 {
            continue;
        }
        for (doc_id, tf) in postings {
            let dl = *lens.get(&doc_id).unwrap_or(&1) as f32;
            let s = bm25.score(tf as f32, df, n_docs as f32, dl);
            *scores.entry(doc_id).or_insert(0.0) += s;
        }
    }

    let mut ranked: Vec<RankedDoc> = scores
        .into_iter()
        .map(|(doc_id, score)| RankedDoc { doc_id, score })
        .collect();

    ranked.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    if ranked.len() > params.topk {
        ranked.truncate(params.topk);
    }
    ranked
}

