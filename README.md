# CodeCurator

An end-to-end tool for curating GitHub repositories into structured code datasets.

- **Fast parallel processing** - Download and extract with configurable workers
- **Smart filtering** - Only processes programming files using GitHub Linguist
- **GPT-2 tokenization** - Ready-to-use token counts for ML workflows
- **Efficient caching** - Uses ETags to avoid re-downloading unchanged repos

Perfect for curating training data, running code analysis, or creating repository archives.


### Installation

```bash
cargo install --path .
```

### Usage

Create an input file with one GitHub repository per line:

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

**Download repositories:**
```bash
codecurator download ./configs/repos.jsonl
```

This creates ZIP files in `/zip/` directory. Downloads from `main` branch first, falls back to `master` if needed.

**Extract and process:**
```bash
codecurator extract ./configs/repos.jsonl
```

Processes all programming files, tokenizes content, and outputs structured data to `/jsonl/` directory.

### CLI Reference

```
$ codecurator --help

USAGE:
    codecurator <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    download
    extract
```

```
$ codecurator download --help

USAGE:
    codecurator download [OPTIONS] <source> [zip-dir]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -u, --user-agent <user-agent>
    -w, --workers <workers>

ARGS:
    <source>
    <zip-dir>
```

```
$ codecurator extract --help

USAGE:
    codecurator extract [OPTIONS] <source> [ARGS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    --max-file-size <max-file-size>

ARGS:
    <source>
    <zip-dir>
    <jsonl-dir>
    <linguist-path>
```
