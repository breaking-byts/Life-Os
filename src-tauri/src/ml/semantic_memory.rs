//! Semantic Memory System
//!
//! Uses LanceDB for fast vector similarity search to find similar past experiences.
//! This enables memory-based features for the bandit and contextual recommendations.

#![allow(dead_code)] // Methods for future search modes

use arrow_array::{
    Array, FixedSizeListArray, Float32Array, Int64Array, RecordBatch,
    RecordBatchIterator, StringArray,
};
use arrow_schema::{DataType, Field, Schema};
use futures::TryStreamExt;
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::{connect, Connection, Table};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::embedding::{run_embedding_task, EmbeddingService, EMBEDDING_DIM};

/// Cached semantic memory singleton
static SEMANTIC_MEMORY: once_cell::sync::OnceCell<Arc<SemanticMemory>> =
    once_cell::sync::OnceCell::new();

/// A memory event stored in the vector database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEvent {
    pub id: i64,
    pub timestamp: String,
    pub event_type: String,
    pub content: String,
    pub metadata_json: Option<String>,
    pub outcome_score: Option<f32>,
}

/// Search result with similarity score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResult {
    pub event: MemoryEvent,
    pub similarity: f32,
}

/// Semantic memory system using LanceDB
pub struct SemanticMemory {
    connection: RwLock<Connection>,
    table: RwLock<Option<Table>>,
}

impl SemanticMemory {
    /// Get or initialize the global semantic memory
    pub async fn global() -> Result<Arc<SemanticMemory>, String> {
        if let Some(mem) = SEMANTIC_MEMORY.get() {
            return Ok(mem.clone());
        }

        let db_path = dirs::data_dir()
            .map(|p| p.join("com.tauri.dev").join("lancedb"))
            .unwrap_or_else(|| std::path::PathBuf::from("./lancedb"));

        let memory = Self::new(&db_path.to_string_lossy()).await?;
        let arc = Arc::new(memory);

        // Try to set it, but another thread might have beat us
        let _ = SEMANTIC_MEMORY.set(arc.clone());
        Ok(SEMANTIC_MEMORY.get().unwrap().clone())
    }

    /// Create a new semantic memory instance
    pub async fn new(db_path: &str) -> Result<Self, String> {
        // Ensure directory exists
        std::fs::create_dir_all(db_path).map_err(|e| format!("Failed to create db dir: {}", e))?;

        let connection = connect(db_path)
            .execute()
            .await
            .map_err(|e| format!("Failed to connect to LanceDB: {}", e))?;

        let memory = Self {
            connection: RwLock::new(connection),
            table: RwLock::new(None),
        };

        // Initialize table
        memory.ensure_table().await?;

        Ok(memory)
    }

    /// Get the schema for memory events
    fn schema() -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("timestamp", DataType::Utf8, false),
            Field::new("event_type", DataType::Utf8, false),
            Field::new("content", DataType::Utf8, false),
            Field::new("metadata_json", DataType::Utf8, true),
            Field::new("outcome_score", DataType::Float32, true),
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    EMBEDDING_DIM as i32,
                ),
                false,
            ),
        ]))
    }

    /// Ensure the memory_events table exists
    async fn ensure_table(&self) -> Result<(), String> {
        let conn = self.connection.read().await;
        
        // Check if table exists
        let table_names = conn
            .table_names()
            .execute()
            .await
            .map_err(|e| format!("Failed to list tables: {}", e))?;

        if table_names.contains(&"memory_events".to_string()) {
            let table = conn
                .open_table("memory_events")
                .execute()
                .await
                .map_err(|e| format!("Failed to open table: {}", e))?;
            drop(conn);
            *self.table.write().await = Some(table);
        }
        // Table will be created on first insert

        Ok(())
    }

    /// Add a memory event with automatic embedding generation
    pub async fn add_event(
        &self,
        id: i64,
        timestamp: &str,
        event_type: &str,
        content: &str,
        metadata_json: Option<&str>,
        outcome_score: Option<f32>,
    ) -> Result<(), String> {
        // Generate embedding on blocking runtime
        let content_owned = content.to_string();
        let embedding = run_embedding_task({
            let content = content_owned.clone();
            move || {
                let embedding_service = EmbeddingService::global()?;
                embedding_service.embed(&content)
            }
        })
        .await?;

        self.add_event_with_embedding(
            id,
            timestamp,
            event_type,
            &content_owned,
            metadata_json,
            outcome_score,
            &embedding,
        )
        .await
    }

    /// Add a memory event with pre-computed embedding
    pub async fn add_event_with_embedding(
        &self,
        id: i64,
        timestamp: &str,
        event_type: &str,
        content: &str,
        metadata_json: Option<&str>,
        outcome_score: Option<f32>,
        embedding: &[f32],
    ) -> Result<(), String> {
        // Create record batch
        let id_array = Int64Array::from(vec![id]);
        let timestamp_array = StringArray::from(vec![timestamp]);
        let event_type_array = StringArray::from(vec![event_type]);
        let content_array = StringArray::from(vec![content]);
        let metadata_array = StringArray::from(vec![metadata_json]);
        let outcome_array = Float32Array::from(vec![outcome_score]);

        // Create fixed size list for vector using from_iter_primitive
        let embedding_iter: Vec<Option<Vec<Option<f32>>>> = vec![
            Some(embedding.iter().map(|&v| Some(v)).collect())
        ];
        let vector_array = FixedSizeListArray::from_iter_primitive::<arrow_array::types::Float32Type, _, _>(
            embedding_iter,
            EMBEDDING_DIM as i32,
        );

        let batch = RecordBatch::try_new(
            Self::schema(),
            vec![
                Arc::new(id_array),
                Arc::new(timestamp_array),
                Arc::new(event_type_array),
                Arc::new(content_array),
                Arc::new(metadata_array),
                Arc::new(outcome_array),
                Arc::new(vector_array),
            ],
        )
        .map_err(|e| format!("Failed to create record batch: {}", e))?;

        // Check if table exists
        let table_guard = self.table.read().await;
        let table_exists = table_guard.is_some();
        drop(table_guard);

        if !table_exists {
            // Create table with initial data
            let conn = self.connection.read().await;
            let batches = RecordBatchIterator::new(vec![Ok(batch.clone())], Self::schema());
            let new_table = conn
                .create_table("memory_events", Box::new(batches))
                .execute()
                .await
                .map_err(|e| format!("Failed to create table: {}", e))?;
            drop(conn);
            *self.table.write().await = Some(new_table);
            return Ok(());
        }

        // Add to existing table
        let table_guard = self.table.read().await;
        let table = table_guard.clone().unwrap();
        drop(table_guard);
        
        let batches = RecordBatchIterator::new(vec![Ok(batch)], Self::schema());
        table
            .add(Box::new(batches))
            .execute()
            .await
            .map_err(|e| format!("Failed to add event: {}", e))?;

        Ok(())
    }

    /// Search for similar memories
    pub async fn search_similar(
        &self,
        query_text: &str,
        limit: usize,
        event_type_filter: Option<&str>,
    ) -> Result<Vec<MemorySearchResult>, String> {
        // Get table reference
        let table_guard = self.table.read().await;
        let table = match table_guard.clone() {
            Some(t) => t,
            None => return Ok(vec![]), // No memories yet
        };
        drop(table_guard);

        // Generate query embedding on blocking runtime
        let query_text = query_text.to_string();
        let query_embedding = run_embedding_task(move || {
            let embedding_service = EmbeddingService::global()?;
            embedding_service.embed(&query_text)
        })
        .await?;

        // Build query - LanceDB expects owned Vec<f32>
        let mut query = table.vector_search(query_embedding)
            .map_err(|e| format!("Failed to create vector search: {}", e))?;
        query = query.limit(limit);

        if let Some(event_type) = event_type_filter {
            query = query.only_if(format!("event_type = '{}'", event_type));
        }

        // Execute query
        let results = query
            .execute()
            .await
            .map_err(|e| format!("Failed to execute search: {}", e))?
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| format!("Failed to collect results: {}", e))?;

        // Parse results
        Self::parse_search_results(results)
    }

    /// Search with a pre-computed embedding
    pub async fn search_with_embedding(
        &self,
        embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<MemorySearchResult>, String> {
        // Get table
        let table_guard = self.table.read().await;
        let table = match table_guard.clone() {
            Some(t) => t,
            None => return Ok(vec![]),
        };
        drop(table_guard);

        // LanceDB expects owned Vec<f32>
        let query = table.vector_search(embedding.to_vec())
            .map_err(|e| format!("Failed to create search: {}", e))?;

        let results = query
            .limit(limit)
            .execute()
            .await
            .map_err(|e| format!("Failed to execute search: {}", e))?
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| format!("Failed to collect results: {}", e))?;

        Self::parse_search_results(results)
    }

    /// Parse search results from RecordBatches
    fn parse_search_results(results: Vec<RecordBatch>) -> Result<Vec<MemorySearchResult>, String> {
        let mut memories = Vec::new();

        for batch in results {
            let num_rows = batch.num_rows();

            let id_col = batch
                .column_by_name("id")
                .and_then(|c| c.as_any().downcast_ref::<Int64Array>());
            let timestamp_col = batch
                .column_by_name("timestamp")
                .and_then(|c| c.as_any().downcast_ref::<StringArray>());
            let event_type_col = batch
                .column_by_name("event_type")
                .and_then(|c| c.as_any().downcast_ref::<StringArray>());
            let content_col = batch
                .column_by_name("content")
                .and_then(|c| c.as_any().downcast_ref::<StringArray>());
            let metadata_col = batch
                .column_by_name("metadata_json")
                .and_then(|c| c.as_any().downcast_ref::<StringArray>());
            let outcome_col = batch
                .column_by_name("outcome_score")
                .and_then(|c| c.as_any().downcast_ref::<Float32Array>());
            let distance_col = batch
                .column_by_name("_distance")
                .and_then(|c| c.as_any().downcast_ref::<Float32Array>());

            for i in 0..num_rows {
                if let (Some(id), Some(ts), Some(et), Some(c)) = (
                    id_col.map(|col| col.value(i)),
                    timestamp_col.and_then(|col| col.value(i).into()),
                    event_type_col.and_then(|col| col.value(i).into()),
                    content_col.and_then(|col| col.value(i).into()),
                ) {
                    let metadata = metadata_col.and_then(|col| {
                        if col.is_null(i) {
                            None
                        } else {
                            Some(col.value(i).to_string())
                        }
                    });
                    let outcome = outcome_col.and_then(|col| {
                        if col.is_null(i) {
                            None
                        } else {
                            Some(col.value(i))
                        }
                    });
                    // Convert L2 distance to similarity (1 / (1 + distance))
                    let distance = distance_col.map(|col| col.value(i)).unwrap_or(0.0);
                    let similarity = 1.0 / (1.0 + distance);

                    memories.push(MemorySearchResult {
                        event: MemoryEvent {
                            id,
                            timestamp: ts.to_string(),
                            event_type: et.to_string(),
                            content: c.to_string(),
                            metadata_json: metadata,
                            outcome_score: outcome,
                        },
                        similarity,
                    });
                }
            }
        }

        Ok(memories)
    }

    /// Get average outcome from similar past experiences
    pub async fn get_similar_context_outcomes(
        &self,
        context_description: &str,
        limit: usize,
    ) -> Result<(f32, Vec<MemorySearchResult>), String> {
        let results = self.search_similar(context_description, limit, None).await?;

        if results.is_empty() {
            return Ok((0.5, vec![])); // Neutral prior if no history
        }

        // Weighted average by similarity
        let mut weighted_sum = 0.0f32;
        let mut weight_total = 0.0f32;

        for result in &results {
            if let Some(outcome) = result.event.outcome_score {
                weighted_sum += outcome * result.similarity;
                weight_total += result.similarity;
            }
        }

        let avg_outcome = if weight_total > 0.0 {
            weighted_sum / weight_total
        } else {
            0.5
        };

        Ok((avg_outcome, results))
    }

    /// Get total event count
    pub async fn count_events(&self) -> Result<usize, String> {
        let table_guard = self.table.read().await;
        let table = match table_guard.clone() {
            Some(t) => t,
            None => return Ok(0),
        };
        drop(table_guard);
        
        let count = table
            .count_rows(None)
            .await
            .map_err(|e| format!("Failed to count rows: {}", e))?;
        Ok(count)
    }
}
