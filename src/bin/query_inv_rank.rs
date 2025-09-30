use std::collections::HashMap;
use std::env;
use std::fs;

use searchkit::index::inverted::{InvBuilder, InvertedIndex};
use searchkit::rank::bm25::BM25;
// use the same tokenizer your InvBuilder uses, so query terms match
use searchkit::text::tokenize::tokenize;

fn main() {
    // 1) parse CLI
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} <corpus.txt> <query> <topk>", args[0]);
        std::process::exit(1);
    }
    let corpus_path = &args[1];
    let query_raw = &args[2];
    let topk: usize = args[3].parse().unwrap_or(10);

    // 2) load corpus (one document per line)
    let corpus = fs::read_to_string(corpus_path).expect("Failed to read corpus file");
    let docs: Vec<&str> = corpus.lines().collect();
    let n_docs = docs.len() as f32;

    // 3) build inverted index via your InvBuilder (which uses your tokenizer)
    let mut builder = InvBuilder::default();
    for (doc_id, text) in docs.iter().enumerate() {
        builder.add_doc(doc_id as u32, text);
    }
    let inv: InvertedIndex = builder.finalize();

    // 4) compute per-doc lengths (sum of term positions across all terms)
    //    and avgdl for BM25
    let mut lens: HashMap<u32, u32> = HashMap::new();
    for term in inv.dict.keys() {
        for (docid, positions) in inv.postings(term) {
            *lens.entry(docid).or_insert(0) += positions.len() as u32;
        }
    }

    // ensure every doc has a length (even if it had zero tokens)
    for d in 0..docs.len() {
        lens.entry(d as u32).or_insert(1);
    }

    let total_len: u32 = lens.values().copied().sum();
    let avgdl: f32 = if !lens.is_empty() {
        total_len as f32 / lens.len() as f32
    } else {
        1.0
    };
    println!("📊 avgdl = {}, total docs = {}", avgdl, lens.len());

    // 5) tokenize the query using the same tokenizer as the index
    let mut terms: Vec<String> = Vec::new();
    for (tok, _pos) in tokenize(query_raw) {
        terms.push(tok);
    }
    if terms.is_empty() {
        eprintln!("Query produced no tokens after tokenization.");
        std::process::exit(1);
    }

    // 6) score with BM25
    let bm25 = BM25::new(1.5, 0.75, avgdl);
    let mut scores: HashMap<u32, f32> = HashMap::new();

    for term in &terms {
        let postings = inv.postings(term);
        if postings.is_empty() {
            continue;
        }
        let df = postings.len() as f32;

        for (docid, positions) in postings {
            let tf = positions.len() as f32;
            let dl = *lens.get(&docid).unwrap_or(&1) as f32;
            let s = bm25.score(tf, df, n_docs, dl);
            *scores.entry(docid).or_insert(0.0) += s;
        }
    }

    // 7) rank and print
    let mut ranked: Vec<(u32, f32)> = scores.into_iter().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("\n🔎 Top {} results for query: \"{}\"", topk, query_raw);
    for (i, (doc_id, score)) in ranked.iter().take(topk).enumerate() {
        let line = docs.get(*doc_id as usize).unwrap_or(&"<out of range>");
        println!(" {}. doc={}  score={:.6}\n    📄 {}", i + 1, doc_id, score, line);
    }
}

