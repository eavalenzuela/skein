// Embedder trait + implementations.
//
// Phase 5 shipped HashBagEmbedder as the always-available fallback. Phase 5b
// adds OnnxBgeEmbedder using fastembed-rs: it downloads BGE-small-en-v1.5
// (ONNX, 384-dim) on demand into the app data dir and runs inference via
// the bundled ORT runtime. The rest of the indexer doesn't need to change —
// only the trait impl behind SharedEmbedder swaps.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use parking_lot::Mutex;

pub trait Embedder: Send + Sync {
    /// Stable identifier for this model. Stored alongside vectors so a model
    /// swap triggers a reindex rather than mixing dimensionalities.
    fn name(&self) -> &str;
    /// Output dimensionality. Used by Phase 5b ONNX implementations to
    /// validate model load; the hash-bag implementation always returns
    /// the same width as its constructor configured.
    #[allow(dead_code)]
    fn dim(&self) -> usize;
    fn embed(&self, text: &str) -> Vec<f32>;
}

pub type SharedEmbedder = Arc<dyn Embedder>;

/// Hashed bag-of-tokens embedder. Cheap to compute, no model download.
/// Cosine similarity over these vectors approximates Jaccard-like keyword
/// overlap, which is good enough to make the related-notes UI useful while
/// the real BGE-small ONNX path is being wired up.
pub struct HashBagEmbedder {
    dim: usize,
}

impl HashBagEmbedder {
    pub fn new() -> Self {
        Self { dim: 384 }
    }
}

impl Default for HashBagEmbedder {
    fn default() -> Self {
        Self::new()
    }
}

impl Embedder for HashBagEmbedder {
    fn name(&self) -> &str {
        "hashbag-v1"
    }
    fn dim(&self) -> usize {
        self.dim
    }
    fn embed(&self, text: &str) -> Vec<f32> {
        let mut bag = vec![0f32; self.dim];
        for tok in tokenize(text) {
            // FNV-1a 32-bit, then fold into dim.
            let mut h: u32 = 2166136261;
            for b in tok.as_bytes() {
                h ^= *b as u32;
                h = h.wrapping_mul(16777619);
            }
            let slot = (h as usize) % self.dim;
            bag[slot] += 1.0;
            // Two extra slots so we don't get pathological all-in-one collisions
            // for very short documents.
            bag[((h.wrapping_mul(2654435761)) as usize) % self.dim] += 0.5;
            bag[((h.wrapping_mul(40503)) as usize) % self.dim] += 0.25;
        }
        normalize(&mut bag);
        bag
    }
}

const STOPWORDS: &[&str] = &[
    "a", "an", "the", "and", "or", "but", "if", "to", "of", "in", "on", "for", "with", "as", "is",
    "are", "was", "were", "be", "been", "being", "this", "that", "these", "those", "it", "its",
    "into", "from", "by", "at", "i", "you", "he", "she", "we", "they", "them", "his", "her",
    "their", "our", "your", "my", "me", "us", "him", "do", "does", "did", "have", "has", "had",
    "not", "no", "so", "than", "then", "too", "very", "can", "will", "just", "about", "over", "up",
    "down", "out", "off", "all", "any", "both", "each", "few", "more", "most", "other", "some",
    "such", "only", "own", "same", "than", "also",
];

fn tokenize(text: &str) -> impl Iterator<Item = String> + '_ {
    text.split(|c: char| !c.is_alphanumeric())
        .map(|s| s.to_ascii_lowercase())
        .filter(|s| s.len() > 1 && !STOPWORDS.contains(&s.as_str()))
}

pub fn normalize(v: &mut [f32]) {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}

pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    let mut s = 0f32;
    for i in 0..a.len() {
        s += a[i] * b[i];
    }
    s
}

pub fn vec_to_bytes(v: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(v.len() * 4);
    for x in v {
        out.extend_from_slice(&x.to_le_bytes());
    }
    out
}

pub fn bytes_to_vec(b: &[u8]) -> Vec<f32> {
    let mut out = Vec::with_capacity(b.len() / 4);
    let mut i = 0;
    while i + 4 <= b.len() {
        out.push(f32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]));
        i += 4;
    }
    out
}

/// Marker file written into the model cache dir after a successful BGE
/// download. Lets us detect "model already on disk" at app startup without
/// touching the network.
pub const BGE_READY_MARKER: &str = ".bge-ready";

pub fn bge_cache_dir(app_data_dir: &std::path::Path) -> PathBuf {
    app_data_dir.join("models")
}

pub struct OnnxBgeEmbedder {
    inner: Mutex<TextEmbedding>,
}

impl OnnxBgeEmbedder {
    /// Load (and download if necessary) BGE-small-en-v1.5 into `cache_dir`.
    pub fn load_or_download(cache_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&cache_dir).ok();
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::BGESmallENV15)
                .with_cache_dir(cache_dir.clone())
                .with_show_download_progress(false),
        )
        .with_context(|| {
            format!(
                "loading BGE-small-en-v1.5 (cache dir: {})",
                cache_dir.display()
            )
        })?;
        // Drop a marker so future startups know the cache is populated.
        std::fs::write(cache_dir.join(BGE_READY_MARKER), b"1").ok();
        Ok(Self {
            inner: Mutex::new(model),
        })
    }
}

impl Embedder for OnnxBgeEmbedder {
    fn name(&self) -> &str {
        "bge-small-en-v1.5"
    }
    fn dim(&self) -> usize {
        384
    }
    fn embed(&self, text: &str) -> Vec<f32> {
        let model = self.inner.lock();
        match model.embed(vec![text.to_string()], None) {
            Ok(mut v) => v.pop().unwrap_or_else(|| vec![0.0; 384]),
            Err(_) => vec![0.0; 384],
        }
    }
}
