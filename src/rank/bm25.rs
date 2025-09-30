pub struct BM25 {
    pub k1: f32,
    pub b: f32,
    pub avgdl: f32,
}

impl BM25 {
    pub fn new(k1: f32, b: f32, avgdl: f32) -> Self {
        Self { k1, b, avgdl }
    }

    pub fn score(&self, tf: f32, df: f32, n_docs: f32, dl: f32) -> f32 {
        // ✅ IDF: always positive if term is informative
        let idf = ((n_docs - df + 0.5) / (df + 0.5) + 1e-6).ln();
let safe_df = if df > n_docs { n_docs - 1.0 } else { df };
        // ✅ TF normalization
        let norm = tf * (self.k1 + 1.0)
            / (tf + self.k1 * (1.0 - self.b + self.b * dl / self.avgdl));

        idf * norm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bm25_positive() {
        let bm25 = BM25::new(1.2, 0.75, 100.0);
        let score = bm25.score(3.0, 10.0, 1000.0, 120.0);
        assert!(score > 0.0, "BM25 score should be positive, got {}", score);
    }
}

