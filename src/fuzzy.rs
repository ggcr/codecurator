use std::collections::{HashMap, HashSet};
use std::ffi::OsString;

// Generate character n-grams from text
fn generate_ngrams(text: &str, n: usize) -> Vec<String> {
    if text.len() < n {
        return vec![text.to_string()];
    }

    text.chars()
        .collect::<Vec<_>>()
        .windows(n)
        .map(|window| window.iter().collect())
        .collect()
}

// Compute SuperMinHash signatures for documents
fn compute_signatures(
    documents: &[Record],
    signature_size: usize,
    ngram_size: usize,
) -> Vec<Vec<f64>> {
    let bh = BuildHasherDefault::<DefaultHasher>::default();
    let mut sminhash = SuperMinHash::new(signature_size, bh);

    documents
        .iter()
        .map(|doc| {
            let ngrams = generate_ngrams(&doc.text, ngram_size);
            sminhash.sketch_slice(&ngrams);
            let signature = sminhash.get_hsketch().clone();
            sminhash.reinit();
            signature
        })
        .collect()
}

// Find similar document pairs using Jaccard similarity
fn find_similar_pairs(signatures: &[Vec<f64>], threshold: f64) -> Vec<(usize, usize, f64)> {
    let mut similar_pairs = Vec::new();

    for i in 0..signatures.len() {
        for j in (i + 1)..signatures.len() {
            if let Ok(jaccard) = get_jaccard_index_estimate(&signatures[i], &signatures[j]) {
                if jaccard >= threshold {
                    similar_pairs.push((i, j, jaccard));
                }
            }
        }
    }

    similar_pairs
}

// Build connected components using Union-Find
struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    fn union(&mut self, x: usize, y: usize) {
        let root_x = self.find(x);
        let root_y = self.find(y);

        if root_x != root_y {
            match self.rank[root_x].cmp(&self.rank[root_y]) {
                std::cmp::Ordering::Less => self.parent[root_x] = root_y,
                std::cmp::Ordering::Greater => self.parent[root_y] = root_x,
                std::cmp::Ordering::Equal => {
                    self.parent[root_y] = root_x;
                    self.rank[root_x] += 1;
                }
            }
        }
    }

    fn get_components(&mut self) -> HashMap<usize, Vec<usize>> {
        let mut components: HashMap<usize, Vec<usize>> = HashMap::new();

        for i in 0..self.parent.len() {
            let root = self.find(i);
            components.entry(root).or_insert_with(Vec::new).push(i);
        }

        components
    }
}

fn build_connected_components(
    similar_pairs: Vec<(usize, usize, f64)>,
    documents: &[Record],
) -> HashSet<String> {
    let mut uf = UnionFind::new(documents.len());

    // Build connected components
    for (i, j, _similarity) in similar_pairs {
        uf.union(i, j);
    }

    let components = uf.get_components();
    let mut unique_ids = HashSet::new();

    // Keep one document from each component (the first one)
    for (_root, indices) in components {
        if let Some(&first_idx) = indices.first() {
            unique_ids.insert(documents[first_idx].id.clone());
        }
    }

    unique_ids
}

pub fn fuzzy_deduplication(
    jsonl_paths: &Vec<PathBuf>,
    dest_dir: &Path,
    similarity_threshold: f64,
    signature_size: usize,
    ngram_size: usize,
) -> Result<(), ExactDedupError> {
    fs::create_dir_all(dest_dir)?;

    println!(
        "Starting Fuzzy deduplication on {} files with threshold {:.2}",
        jsonl_paths.len(),
        similarity_threshold
    );

    // Read all documents
    let documents: Vec<Record> = jsonl_paths
        .par_iter()
        .filter_map(|path| read_records(path).ok())
        .flatten()
        .collect();

    println!("Loaded {} documents", documents.len());

    // Compute MinHash signatures
    println!("Computing MinHash signatures...");
    let signatures = compute_signatures(&documents, signature_size, ngram_size);

    // Find similar pairs
    println!("Finding similar document pairs...");
    let similar_pairs = find_similar_pairs(&signatures, similarity_threshold);

    println!("Found {} similar pairs", similar_pairs.len());

    // Build connected components to find duplicate groups
    let unique_ids = build_connected_components(similar_pairs, &documents);

    println!(
        "Keeping {} unique documents out of {}",
        unique_ids.len(),
        documents.len()
    );

    // Write deduplicated results
    jsonl_paths
        .par_iter()
        .for_each(|path| match write_records(path, &unique_ids, dest_dir) {
            Ok(()) => (),
            Err(e) => {
                eprintln!("Error processing {}: {}", path.display(), e);
            }
        });

    println!("Fuzzy dedup written to {}", dest_dir.display());
    Ok(())
}
