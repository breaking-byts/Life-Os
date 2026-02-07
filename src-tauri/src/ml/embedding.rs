//! ONNX Embedding Service
//!
//! Provides local embedding generation using the Qwen3 embedding model.
//! Uses ONNX Runtime with CoreML acceleration for fast inference.

#![allow(dead_code)] // Utility functions for future use

use ndarray::Array2;
use ort::session::{builder::GraphOptimizationLevel, Session};
use ort::value::Tensor;
use parking_lot::RwLock;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokenizers::Tokenizer;

/// Embedding dimension for Qwen3 model
pub const EMBEDDING_DIM: usize = 1024;

/// Maximum sequence length for the model
const MAX_SEQ_LENGTH: usize = 512;

/// Maximum number of concurrent embedding tasks.
pub const EMBEDDING_MAX_CONCURRENT_TASKS: usize = 2;

/// Cached embedding service singleton
static EMBEDDING_SERVICE: once_cell::sync::OnceCell<Arc<EmbeddingService>> =
    once_cell::sync::OnceCell::new();

static EMBEDDING_TASK_SEMAPHORE: once_cell::sync::Lazy<Arc<Semaphore>> =
    once_cell::sync::Lazy::new(|| Arc::new(Semaphore::new(EMBEDDING_MAX_CONCURRENT_TASKS)));

#[cfg(test)]
mod test_hooks {
    use super::*;
    use std::cell::Cell;

    thread_local! {
        static IN_EMBEDDING_BLOCKING: Cell<bool> = Cell::new(false);
    }

    pub(super) static TEST_GLOBAL_INIT_HOOK: once_cell::sync::OnceCell<
        fn() -> Result<Arc<EmbeddingService>, String>,
    > = once_cell::sync::OnceCell::new();

    pub(super) fn set_test_global_init_hook(
        hook: fn() -> Result<Arc<EmbeddingService>, String>,
    ) {
        let _ = TEST_GLOBAL_INIT_HOOK.set(hook);
    }

    pub(super) fn is_in_embedding_blocking() -> bool {
        IN_EMBEDDING_BLOCKING.with(|flag| flag.get())
    }

    pub(super) fn with_embedding_blocking_flag<T>(f: impl FnOnce() -> T) -> T {
        IN_EMBEDDING_BLOCKING.with(|flag| {
            let previous = flag.replace(true);
            let result = f();
            flag.set(previous);
            result
        })
    }
}

pub(crate) async fn run_embedding_task<F, T>(task: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String> + Send + 'static,
    T: Send + 'static,
{
    let permit = EMBEDDING_TASK_SEMAPHORE
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| "Embedding task queue closed".to_string())?;
    let handle = tauri::async_runtime::spawn_blocking(move || {
        let _permit = permit;
        let result = {
            #[cfg(test)]
            {
                test_hooks::with_embedding_blocking_flag(task)
            }
            #[cfg(not(test))]
            {
                task()
            }
        };
        result
    });
    handle
        .await
        .map_err(|e| format!("Embedding task failed: {}", e))?
}

/// ONNX-based embedding service
pub struct EmbeddingService {
    session: RwLock<Session>,
    tokenizer: Tokenizer,
}

impl EmbeddingService {
    /// Get or initialize the global embedding service
    pub fn global() -> Result<Arc<EmbeddingService>, String> {
        #[cfg(test)]
        if let Some(hook) = test_hooks::TEST_GLOBAL_INIT_HOOK.get() {
            return hook();
        }
        EMBEDDING_SERVICE
            .get_or_try_init(|| {
                let model_dir = std::env::var("EMBEDDING_MODEL_PATH").unwrap_or_else(|_| {
                    dirs::data_dir()
                        .map(|p| {
                            p.join("com.tauri.dev")
                                .join("models")
                                .join("qwen3-embedding")
                        })
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                });
                Self::new(&model_dir)
            })
            .cloned()
    }

    /// Create a new embedding service from a model directory
    pub fn new(model_dir: &str) -> Result<Arc<Self>, String> {
        let model_path = Path::new(model_dir).join("model_q4.onnx");
        let tokenizer_path = Path::new(model_dir).join("tokenizer.json");

        if !model_path.exists() {
            return Err(format!("Model not found at {:?}", model_path));
        }
        if !tokenizer_path.exists() {
            return Err(format!("Tokenizer not found at {:?}", tokenizer_path));
        }

        // Initialize ONNX Runtime session
        let session = Session::builder()
            .map_err(|e| format!("Failed to create session builder: {}", e))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| format!("Failed to set optimization level: {}", e))?
            .with_intra_threads(4)
            .map_err(|e| format!("Failed to set threads: {}", e))?
            .commit_from_file(&model_path)
            .map_err(|e| format!("Failed to load model: {}", e))?;

        // Load tokenizer
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| format!("Failed to load tokenizer: {}", e))?;

        Ok(Arc::new(Self {
            session: RwLock::new(session),
            tokenizer,
        }))
    }

    /// Generate embedding for a single text
    pub fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        let embeddings = self.embed_batch(&[text.to_string()])?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| "No embedding generated".to_string())
    }

    /// Generate embeddings for a batch of texts
    pub fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, String> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        // Tokenize all texts
        let encodings = self
            .tokenizer
            .encode_batch(texts.to_vec(), true)
            .map_err(|e| format!("Tokenization failed: {}", e))?;

        let batch_size = encodings.len();

        // Find max length in batch (capped at MAX_SEQ_LENGTH)
        let max_len = encodings
            .iter()
            .map(|e| e.get_ids().len().min(MAX_SEQ_LENGTH))
            .max()
            .unwrap_or(1);

        // Prepare input tensors with padding
        let mut input_ids = vec![0i64; batch_size * max_len];
        let mut attention_mask = vec![0i64; batch_size * max_len];

        for (i, encoding) in encodings.iter().enumerate() {
            let ids = encoding.get_ids();
            let mask = encoding.get_attention_mask();
            let len = ids.len().min(max_len);

            for j in 0..len {
                input_ids[i * max_len + j] = ids[j] as i64;
                attention_mask[i * max_len + j] = mask[j] as i64;
            }
        }

        // Create ndarray views for ONNX
        let input_ids_array = Array2::from_shape_vec((batch_size, max_len), input_ids)
            .map_err(|e| format!("Failed to create input array: {}", e))?;
        let attention_mask_array = Array2::from_shape_vec((batch_size, max_len), attention_mask)
            .map_err(|e| format!("Failed to create attention mask array: {}", e))?;

        // Create ORT tensors
        let input_ids_tensor = Tensor::from_array(input_ids_array)
            .map_err(|e| format!("Failed to create input_ids tensor: {}", e))?;
        let attention_mask_tensor = Tensor::from_array(attention_mask_array)
            .map_err(|e| format!("Failed to create attention_mask tensor: {}", e))?;

        // Run inference
        let mut session = self.session.write();
        let outputs = session
            .run(ort::inputs![
                "input_ids" => input_ids_tensor,
                "attention_mask" => attention_mask_tensor
            ])
            .map_err(|e| format!("Inference failed: {}", e))?;

        // Extract embeddings - Qwen3 outputs last_hidden_state
        // We need mean pooling over the sequence dimension
        let output = outputs
            .get("last_hidden_state")
            .ok_or_else(|| "No output tensor found".to_string())?;

        let (output_shape, output_data) = output
            .try_extract_tensor::<f32>()
            .map_err(|e| format!("Failed to extract tensor: {}", e))?;

        // output shape: (batch_size, seq_len, hidden_size)
        let shape = output_shape.as_ref();

        if shape.len() != 3 {
            return Err(format!("Unexpected output shape: {:?}", shape));
        }

        let seq_len = shape[1] as usize;
        let hidden_size = shape[2] as usize;

        // Mean pooling with attention mask
        let mut embeddings = Vec::with_capacity(batch_size);

        for i in 0..batch_size {
            let mut sum = vec![0.0f32; hidden_size];
            let mut count = 0.0f32;

            // Get the actual token count for this sample (non-padded)
            let encoding = &encodings[i];
            let token_count = encoding.get_ids().len().min(max_len);

            for j in 0..token_count {
                for k in 0..hidden_size {
                    // Manual 3D indexing: output_data[i * seq_len * hidden_size + j * hidden_size + k]
                    let idx = i * seq_len * hidden_size + j * hidden_size + k;
                    sum[k] += output_data[idx];
                }
                count += 1.0;
            }

            // Normalize
            if count > 0.0 {
                for v in &mut sum {
                    *v /= count;
                }
            }

            // L2 normalize for better similarity comparisons
            let norm: f32 = sum.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for v in &mut sum {
                    *v /= norm;
                }
            }

            embeddings.push(sum);
        }

        Ok(embeddings)
    }

    /// Compute cosine similarity between two embeddings
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a > 0.0 && norm_b > 0.0 {
            dot / (norm_a * norm_b)
        } else {
            0.0
        }
    }

    /// Convert embedding to bytes for SQLite storage
    pub fn embedding_to_bytes(embedding: &[f32]) -> Vec<u8> {
        embedding.iter().flat_map(|f| f.to_le_bytes()).collect()
    }

    /// Convert bytes back to embedding
    pub fn bytes_to_embedding(bytes: &[u8]) -> Vec<f32> {
        bytes
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect()
    }
}

/// Generate a context-aware embedding for a user event
pub async fn embed_user_event(
    event_type: &str,
    content: &str,
    context: Option<&str>,
) -> Result<Vec<f32>, String> {
    // Create a rich text representation for embedding
    let text = if let Some(ctx) = context {
        format!("[{}] {} | Context: {}", event_type, content, ctx)
    } else {
        format!("[{}] {}", event_type, content)
    };

    run_embedding_task(move || {
        let service = EmbeddingService::global()?;
        service.embed(&text)
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}};
    use std::time::Duration;
    use tokio::time::timeout;

    #[test]
    fn test_embedding_bytes_roundtrip() {
        let original = vec![1.0f32, 2.0, 3.0, -4.5];
        let bytes = EmbeddingService::embedding_to_bytes(&original);
        let recovered = EmbeddingService::bytes_to_embedding(&bytes);
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((EmbeddingService::cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);

        let c = vec![0.0, 1.0, 0.0];
        assert!(EmbeddingService::cosine_similarity(&a, &c).abs() < 1e-6);
    }

    #[tokio::test]
    async fn embedding_tasks_are_bounded() {
        let max = EMBEDDING_MAX_CONCURRENT_TASKS;
        let active = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));
        let mut handles = Vec::new();

        for _ in 0..(max + 2) {
            let active = Arc::clone(&active);
            let peak = Arc::clone(&peak);
            handles.push(tokio::spawn(async move {
                run_embedding_task(move || {
                    let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                    peak.fetch_max(current, Ordering::SeqCst);
                    std::thread::sleep(Duration::from_millis(50));
                    active.fetch_sub(1, Ordering::SeqCst);
                    Ok::<_, String>(())
                })
                .await
            }));
        }

        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        assert!(
            peak.load(Ordering::SeqCst) <= max,
            "concurrency exceeded: {} > {}",
            peak.load(Ordering::SeqCst),
            max
        );
    }

    #[tokio::test]
    async fn async_runtime_stays_responsive_during_embedding() {
        let embedding = tokio::spawn(async {
            run_embedding_task(|| {
                std::thread::sleep(Duration::from_millis(150));
                Ok::<_, String>(())
            })
            .await
        });

        let fast_task = timeout(Duration::from_millis(50), async {
            tokio::time::sleep(Duration::from_millis(10)).await;
        })
        .await;

        embedding.await.unwrap().unwrap();
        assert!(fast_task.is_ok(), "async task timed out while embedding ran");
    }

    #[tokio::test]
    async fn embedding_service_init_runs_in_blocking_task() {
        static CALLED_IN_BLOCKING: AtomicBool = AtomicBool::new(false);

        test_hooks::set_test_global_init_hook(|| {
            let in_blocking = test_hooks::is_in_embedding_blocking();
            CALLED_IN_BLOCKING.store(in_blocking, Ordering::SeqCst);
            Err("test init".to_string())
        });

        let result = embed_user_event("note", "payload", None).await;

        assert!(result.is_err(), "expected test init error");
        assert!(
            CALLED_IN_BLOCKING.load(Ordering::SeqCst),
            "expected EmbeddingService::global to run inside spawn_blocking"
        );
    }
}
