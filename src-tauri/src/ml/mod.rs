//! Machine Learning module for Maximum Intelligence Productivity Agent
//!
//! This module implements a 2025 SOTA reinforcement learning system:
//!
//! ## Core Components
//!
//! - **Embedding Service**: ONNX-based Qwen3 embeddings for semantic understanding
//! - **Semantic Memory**: LanceDB vector store for similarity-based retrieval
//! - **Rich Features**: 50+ dimensional context vector for ML
//! - **Hybrid Bandit**: Linear→Neural contextual bandit with UCB/Thompson Sampling
//! - **Multi-Scale Rewards**: Balanced immediate/daily/weekly/monthly optimization
//!
//! ## Architecture
//!
//! ```text
//! User Action → Rich Context (50-dim) → Hybrid Bandit → Action Selection
//!                     ↓                      ↓
//!              Semantic Memory        Multi-Scale Rewards
//!                     ↓                      ↓
//!              Similar Experiences    Balanced Learning
//! ```
//!
//! ## Phases
//!
//! - **Phase 1 (Day 1-90)**: Bayesian Linear Regression with Thompson Sampling
//! - **Phase 2 (Month 3+)**: Neural Ensemble with Epistemic Uncertainty

pub mod embedding;
pub mod semantic_memory;
pub mod rich_features;
pub mod bandit_v2;
pub mod models;
pub mod pattern_miner;
pub mod user_profile;
pub mod bandit;  // Legacy bandit for backwards compatibility
pub mod feature_store;  // Legacy feature store for backwards compatibility

// Re-export core components
pub use semantic_memory::SemanticMemory;
pub use rich_features::{RichContext, RichFeatureStore};

pub use pattern_miner::PatternMiner;
pub use user_profile::UserProfile;

// Legacy exports for backwards compatibility
pub use bandit::ContextualBandit;
pub use feature_store::FeatureStore;
