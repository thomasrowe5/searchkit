# 🔍 Searchkit – A Simple Information Retrieval Engine (BM25 + Inverted Index)

Searchkit is a lightweight search engine written in **Rust** that demonstrates the fundamentals of information retrieval:

- 🧠 **Inverted Index** construction from plain text documents  
- 📊 **BM25 ranking** for relevance scoring  
- 🧰 CLI interface for querying a text corpus  
- ⚡ Fast and memory-efficient design using Rust

---

## 🚀 Features

- Builds an inverted index from a corpus of text documents
- Computes BM25 scores for query terms
- Supports ranked retrieval of documents by relevance
- Command-line tool to query a corpus and display results

---

## 📦 Installation

Clone and build the project:

```bash
git clone https://github.com/YOUR_USERNAME/searchkit.git
cd searchkit
cargo build --release
🧪 Usage
Prepare a small text corpus:
echo "Neural networks are cool." > corpus.txt
echo "Cats are smarter than dogs." >> corpus.txt
echo "Neural network training uses gradients." >> corpus.txt
Run the query tool:
./target/release/query-inv-rank corpus.txt "cat neural" 10
Expected output:
📊 avgdl = 4.667, total docs = 3

🔎 Top 10 results for query: "cat neural"
 1. doc=2  score=0.89
    📄 Neural network training uses gradients.
 2. doc=0  score=0.78
    📄 Neural networks are cool.
🛠️ Tech Stack
🦀 Rust
📚 Custom Inverted Index
📊 BM25 Relevance Scoring
📜 License
MIT License – free to use, modify, and share.
