pub struct BM25{ pub k1:f32, pub b:f32, pub avgdl:f32 }
impl BM25{ pub fn score(&self, tf:f32, df:f32, n_docs:f32, dl:f32)->f32 { let idf=((n_docs-df+0.5)/(df+0.5)+1e-6).ln(); let denom=tf + self.k1*(1.0 - self.b + self.b*dl/self.avgdl); idf * (tf * (self.k1+1.0)) / denom } }
