# CodeCurator

An end-to-end tool for curating and processing GitHub repos as datasets

- Fast and parallel download and processing
- Filters out non-code files
- GPT2 BPE tokenization

Use it to curate training data at scale, run static analytics, or archive code snapshots.


### Installation

```bash
cargo install --path .
```

### Usage

Example input file (each line contains a GitHub repository):
```jsonl
"microsoft/vscode"
"vercel/next.js"
"tensorflow/tensorflow"
"bitcoin/bitcoin"
"rust-lang/rust"
"kubernetes/kubernetes"
"facebook/react"
"docker/compose"
"ansible/ansible"
"elastic/elasticsearch"
```

Download the repos:
```bash
codecurator download ./configs/repos.jsonl
```
This creates a folder containing all of the repository ZIP files (`/zip/<repo>.zip`). By default, it attempts to download from `main` branch, if that fails, it falls back to `master`.

Extract them onto JSON Lines files:
```bash
codecurator extract ./configs/repos.jsonl
```
Parses all the valid coding files, tokenizes content and dumps it into a `jsonl` file (`/jsonl/<repo>.jsonl`).
