use clap::{Parser, Subcommand};
use anyhow::Result;

mod util { pub mod varint; pub mod bitvec; pub mod mmap; pub mod timer;  pub mod rrr; }
mod text { pub mod tokenize; pub mod corpus; }
mod index { pub mod inverted; pub mod suffix_array; pub mod lcp; pub mod bwt; pub mod fmindex; }
mod rank { pub mod bm25; }
mod query { pub mod boolean; pub mod phrase; pub mod substring; pub mod engine; }

#[derive(Parser)]
#[command(name = "searchkit")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Build inverted index from a single text file (each line = one doc)
    BuildInv { corpus: String, out: String },

    /// Build and save FM-index from a text file (appends '$' if missing)
    BuildFm { text: String, out: String, sa_sample: usize },

    /// Query term/phrase via inverted index
    QueryInv { index: String, q: String, k: usize },

    /// Substring query using a saved FM-index file
    Substr { fm: String, pat: String, max: usize },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::BuildInv { corpus, out } => {
            use std::{fs::File, io::{BufRead, BufReader, Write}};
            let f = File::open(&corpus)?;
            let mut b = index::inverted::InvBuilder::default();
            let mut docid: u32 = 0;
            for line in BufReader::new(f).lines() {
                let text = line?;
                b.add_doc(docid, &text);
                docid += 1;
            }
            let inv = b.finalize();
            let mut w = File::create(&out)?;
            let json = serde_json::to_vec(&inv.dict)?;
            w.write_all(&json)?;
            eprintln!("Inverted index built: {} docs -> {}", docid, out);
        }

        Cmd::QueryInv { index, q, k } => {
            use std::fs::File;
            use std::io::Read;
            use std::collections::BTreeMap;
            let mut buf = Vec::new();
            File::open(&index)?.read_to_end(&mut buf)?;
            let dict: BTreeMap<String, Vec<u8>> = serde_json::from_slice(&buf)?;
            let inv = index::inverted::InvertedIndex { dict };

            let terms: Vec<_> = q.split_whitespace().collect();
            let hits = if terms.len() == 1 {
                inv.postings(terms[0]).into_iter().map(|(d,_)| d).collect::<Vec<_>>()
            } else {
                query::phrase::phrase_query(&inv, &terms.iter().map(|s| *s).collect::<Vec<_>>())
            };
            println!("hits[{}]: {:?}", hits.len().min(k), &hits.into_iter().take(k).collect::<Vec<_>>());
        }

        Cmd::BuildFm { text, out, sa_sample } => {
            use std::fs::File;
            use std::io::Read;
            use index::suffix_array::build_sa;
            use index::lcp::kasai_lcp;
            use index::bwt::bwt_from_sa;

            let mut s = String::new();
            File::open(&text)?.read_to_string(&mut s)?;
            if !s.ends_with('$') { s.push('$'); }
            let bytes = s.as_bytes();

            let sa = build_sa(bytes);
            let _lcp = kasai_lcp(bytes, &sa);
            let (bwt, _primary) = bwt_from_sa(bytes, &sa);
            let fm = index::fmindex::FMIndex::build(bytes, &sa, &bwt, sa_sample);
            fm.save(&out)?;
            eprintln!("✅ FM-index built and saved to {}", out);
        }

        Cmd::Substr { fm, pat, max } => {
            let fm = index::fmindex::FMIndex::load(&fm)?;
            match fm.backward_search(pat.as_bytes()) {
                None => println!("no matches"),
                Some(range) => {
                    let mut locs = fm.locate_range(&range, max);
                    locs.sort_unstable();
                    println!("matches = {}:", locs.len());
                    println!("{locs:?}");
                }
            }
        }
    }
    Ok(())
}
