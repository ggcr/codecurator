# CurateML

A Rust tool for curating and processing GitHub repos as datasets

## Installation

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

Process the file:
```bash
curateml repos.jsonl
```
