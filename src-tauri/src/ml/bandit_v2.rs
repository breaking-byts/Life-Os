//! Hybrid Contextual Bandit
//!
//! Phase 1 (Day 1-90): Bayesian Linear Regression with Thompson Sampling
//! Phase 2 (Month 3+): Neural Network Ensemble with Epistemic Uncertainty
//!
//! Uses Upper Confidence Bound (UCB) for action selection:
//! UCB = E[reward|context] + beta * uncertainty

#![allow(dead_code)] // Thompson sampling and category filtering for future use

use nalgebra::{DMatrix, DVector};
use ndarray::Array1;
use rand::prelude::*;
use rand_distr::Normal;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

use super::rich_features::{RichContext, FEATURE_DIM};

/// Exploration parameter for UCB
const DEFAULT_BETA: f32 = 2.0;

/// Prior precision for Bayesian linear regression
const PRIOR_PRECISION: f64 = 1.0;

/// Noise precision (inverse variance)
const NOISE_PRECISION: f64 = 1.0;

/// Action definition with bandit parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanditAction {
    pub id: i64,
    pub name: String,
    pub category: String,
    pub description: String,
    pub total_pulls: i64,
    pub total_reward: f64,
    pub is_enabled: bool,
}

/// Result of action selection with full context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionSelection {
    pub action: BanditAction,
    pub expected_reward: f32,
    pub uncertainty: f32,
    pub ucb_score: f32,
    pub feature_contributions: Vec<(String, f32)>, // Top contributing features
}

/// Bayesian Linear Bandit parameters for one action
#[derive(Debug, Clone)]
pub struct LinearBanditParams {
    /// Mean of weight posterior (FEATURE_DIM)
    pub mu: DVector<f64>,
    /// Precision matrix (inverse covariance) of posterior (FEATURE_DIM x FEATURE_DIM)
    pub precision: DMatrix<f64>,
    /// Prior precision
    pub prior_precision: f64,
    /// Noise precision
    pub noise_precision: f64,
}

impl LinearBanditParams {
    /// Create new parameters with prior
    pub fn new() -> Self {
        let dim = FEATURE_DIM;
        Self {
            mu: DVector::zeros(dim),
            precision: DMatrix::identity(dim, dim) * PRIOR_PRECISION,
            prior_precision: PRIOR_PRECISION,
            noise_precision: NOISE_PRECISION,
        }
    }

    /// Create from stored bytes
    pub fn from_bytes(theta_bytes: &[u8], precision_bytes: &[u8]) -> Option<Self> {
        let dim = FEATURE_DIM;

        // Parse theta (mean vector)
        if theta_bytes.len() != dim * 8 {
            return None;
        }
        let theta_vec: Vec<f64> = theta_bytes
            .chunks_exact(8)
            .map(|chunk| f64::from_le_bytes(chunk.try_into().unwrap()))
            .collect();
        let mu = DVector::from_vec(theta_vec);

        // Parse precision matrix (stored as full matrix for now)
        if precision_bytes.len() != dim * dim * 8 {
            return None;
        }
        let precision_vec: Vec<f64> = precision_bytes
            .chunks_exact(8)
            .map(|chunk| f64::from_le_bytes(chunk.try_into().unwrap()))
            .collect();
        let precision = DMatrix::from_vec(dim, dim, precision_vec);

        Some(Self {
            mu,
            precision,
            prior_precision: PRIOR_PRECISION,
            noise_precision: NOISE_PRECISION,
        })
    }

    /// Convert to bytes for storage
    pub fn to_bytes(&self) -> (Vec<u8>, Vec<u8>) {
        let theta_bytes: Vec<u8> = self
            .mu
            .iter()
            .flat_map(|f| f.to_le_bytes())
            .collect();

        let precision_bytes: Vec<u8> = self
            .precision
            .iter()
            .flat_map(|f| f.to_le_bytes())
            .collect();

        (theta_bytes, precision_bytes)
    }

    /// Predict reward for a context
    pub fn predict(&self, features: &Array1<f32>) -> f64 {
        let x = DVector::from_iterator(FEATURE_DIM, features.iter().map(|&f| f as f64));
        self.mu.dot(&x)
    }

    /// Get uncertainty (standard deviation) for a context
    pub fn uncertainty(&self, features: &Array1<f32>) -> f64 {
        let x = DVector::from_iterator(FEATURE_DIM, features.iter().map(|&f| f as f64));

        // Compute covariance = precision^(-1)
        // For efficiency, solve precision * cov * x = x
        // Uncertainty = sqrt(x^T * cov * x) = sqrt(x^T * precision^(-1) * x)

        match self.precision.clone().try_inverse() {
            Some(cov) => {
                let var = x.transpose() * &cov * &x;
                (var[(0, 0)] / self.noise_precision).sqrt()
            }
            None => 1.0, // High uncertainty if precision is singular
        }
    }

    /// Sample from posterior using Thompson Sampling
    pub fn thompson_sample(&self, features: &Array1<f32>) -> f64 {
        let mut rng = thread_rng();

        // Sample weights from posterior N(mu, precision^(-1))
        match self.precision.clone().try_inverse() {
            Some(cov) => {
                // Cholesky decomposition for sampling
                match cov.clone().cholesky() {
                    Some(chol) => {
                        let l = chol.l();
                        let z: DVector<f64> = DVector::from_fn(FEATURE_DIM, |_, _| {
                            Normal::new(0.0, 1.0).unwrap().sample(&mut rng)
                        });
                        let sampled_weights = &self.mu + l * z;
                        let x = DVector::from_iterator(
                            FEATURE_DIM,
                            features.iter().map(|&f| f as f64),
                        );
                        sampled_weights.dot(&x)
                    }
                    None => self.predict(features), // Fallback to mean
                }
            }
            None => self.predict(features),
        }
    }

    /// Update posterior with new observation
    /// Uses Bayesian linear regression update:
    /// precision_new = precision_old + noise_precision * x * x^T
    /// mu_new = precision_new^(-1) * (precision_old * mu_old + noise_precision * y * x)
    pub fn update(&mut self, features: &Array1<f32>, reward: f64) {
        let x = DVector::from_iterator(FEATURE_DIM, features.iter().map(|&f| f as f64));

        // Update precision
        let outer = &x * x.transpose();
        self.precision += outer * self.noise_precision;

        // Update mean
        match self.precision.clone().try_inverse() {
            Some(cov) => {
                let old_term = &self.precision * &self.mu;
                let new_term = &x * (reward * self.noise_precision);
                self.mu = cov * (old_term + new_term);
            }
            None => {
                // If inversion fails, just do gradient step
                let pred = self.predict(features);
                let error = reward - pred;
                let step = 0.01;
                self.mu += &x * (step * error);
            }
        }
    }

    /// Get feature contributions for explainability
    pub fn feature_contributions(&self, features: &Array1<f32>) -> Vec<(usize, f32)> {
        let mut contributions: Vec<(usize, f32)> = features
            .iter()
            .enumerate()
            .map(|(i, &f)| (i, f * self.mu[i] as f32))
            .collect();

        contributions.sort_by(|a, b| b.1.abs().partial_cmp(&a.1.abs()).unwrap());
        contributions
    }
}

impl Default for LinearBanditParams {
    fn default() -> Self {
        Self::new()
    }
}

/// Hybrid Contextual Bandit
pub struct HybridBandit;

impl HybridBandit {
    /// Get all enabled actions
    pub async fn get_actions(pool: &Pool<Sqlite>) -> Result<Vec<BanditAction>, String> {
        let actions: Vec<(i64, String, Option<String>, Option<String>, i64, f64, bool)> =
            sqlx::query_as(
                r#"
                SELECT id, action_name, category, description, total_pulls, total_reward, is_enabled
                FROM agent_linear_bandit WHERE is_enabled = 1
                "#,
            )
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(actions
            .into_iter()
            .map(
                |(id, name, category, description, total_pulls, total_reward, is_enabled)| {
                    BanditAction {
                        id,
                        name,
                        category: category.unwrap_or_default(),
                        description: description.unwrap_or_default(),
                        total_pulls,
                        total_reward,
                        is_enabled,
                    }
                },
            )
            .collect())
    }

    /// Get actions by category
    pub async fn get_actions_by_category(
        pool: &Pool<Sqlite>,
        category: &str,
    ) -> Result<Vec<BanditAction>, String> {
        let actions: Vec<(i64, String, Option<String>, Option<String>, i64, f64, bool)> =
            sqlx::query_as(
                r#"
                SELECT id, action_name, category, description, total_pulls, total_reward, is_enabled
                FROM agent_linear_bandit WHERE is_enabled = 1 AND category = ?
                "#,
            )
            .bind(category)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(actions
            .into_iter()
            .map(
                |(id, name, category, description, total_pulls, total_reward, is_enabled)| {
                    BanditAction {
                        id,
                        name,
                        category: category.unwrap_or_default(),
                        description: description.unwrap_or_default(),
                        total_pulls,
                        total_reward,
                        is_enabled,
                    }
                },
            )
            .collect())
    }

    /// Load bandit parameters for an action
    pub async fn load_params(
        pool: &Pool<Sqlite>,
        action_name: &str,
    ) -> Result<LinearBanditParams, String> {
        let row: Option<(Option<Vec<u8>>, Option<Vec<u8>>)> = sqlx::query_as(
            "SELECT theta, precision_matrix FROM agent_linear_bandit WHERE action_name = ?",
        )
        .bind(action_name)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        match row {
            Some((Some(theta), Some(prec))) => {
                LinearBanditParams::from_bytes(&theta, &prec).ok_or_else(|| "Invalid params".into())
            }
            _ => Ok(LinearBanditParams::new()), // Return prior if no data
        }
    }

    /// Save bandit parameters for an action
    pub async fn save_params(
        pool: &Pool<Sqlite>,
        action_name: &str,
        params: &LinearBanditParams,
    ) -> Result<(), String> {
        let (theta_bytes, prec_bytes) = params.to_bytes();

        sqlx::query(
            r#"
            UPDATE agent_linear_bandit 
            SET theta = ?, precision_matrix = ?, last_pulled = datetime('now')
            WHERE action_name = ?
            "#,
        )
        .bind(&theta_bytes)
        .bind(&prec_bytes)
        .bind(action_name)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Select the best action using UCB
    pub async fn select_action(
        pool: &Pool<Sqlite>,
        context: &RichContext,
        beta: Option<f32>,
    ) -> Result<Option<ActionSelection>, String> {
        let selections = Self::select_top_actions(pool, context, 1, beta).await?;
        Ok(selections.into_iter().next())
    }

    /// Select top N actions using UCB
    pub async fn select_top_actions(
        pool: &Pool<Sqlite>,
        context: &RichContext,
        n: usize,
        beta: Option<f32>,
    ) -> Result<Vec<ActionSelection>, String> {
        let beta = beta.unwrap_or(DEFAULT_BETA);
        let features = context.to_feature_vector();
        let actions = Self::get_actions(pool).await?;

        if actions.is_empty() {
            return Ok(vec![]);
        }

        let feature_names = RichContext::feature_names();
        let mut scored_actions: Vec<ActionSelection> = Vec::new();

        for action in actions {
            let params = Self::load_params(pool, &action.name).await?;

            let expected_reward = params.predict(&features) as f32;
            let uncertainty = params.uncertainty(&features) as f32;
            let ucb_score = expected_reward + beta * uncertainty;

            // Get top 5 contributing features
            let contributions = params.feature_contributions(&features);
            let top_contributions: Vec<(String, f32)> = contributions
                .into_iter()
                .take(5)
                .map(|(i, v)| (feature_names[i].to_string(), v))
                .collect();

            scored_actions.push(ActionSelection {
                action,
                expected_reward,
                uncertainty,
                ucb_score,
                feature_contributions: top_contributions,
            });
        }

        // Sort by UCB score descending
        scored_actions.sort_by(|a, b| {
            b.ucb_score
                .partial_cmp(&a.ucb_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(scored_actions.into_iter().take(n).collect())
    }

    /// Select action using Thompson Sampling (alternative to UCB)
    pub async fn select_action_thompson(
        pool: &Pool<Sqlite>,
        context: &RichContext,
    ) -> Result<Option<ActionSelection>, String> {
        let features = context.to_feature_vector();
        let actions = Self::get_actions(pool).await?;

        if actions.is_empty() {
            return Ok(None);
        }

        let feature_names = RichContext::feature_names();
        let mut best: Option<ActionSelection> = None;
        let mut best_sample = f64::NEG_INFINITY;

        for action in actions {
            let params = Self::load_params(pool, &action.name).await?;

            let sample = params.thompson_sample(&features);
            let expected_reward = params.predict(&features) as f32;
            let uncertainty = params.uncertainty(&features) as f32;

            if sample > best_sample {
                best_sample = sample;

                let contributions = params.feature_contributions(&features);
                let top_contributions: Vec<(String, f32)> = contributions
                    .into_iter()
                    .take(5)
                    .map(|(i, v)| (feature_names[i].to_string(), v))
                    .collect();

                best = Some(ActionSelection {
                    action,
                    expected_reward,
                    uncertainty,
                    ucb_score: sample as f32,
                    feature_contributions: top_contributions,
                });
            }
        }

        Ok(best)
    }

    /// Update bandit with observed reward
    pub async fn update(
        pool: &Pool<Sqlite>,
        action_name: &str,
        context: &RichContext,
        reward: f64,
    ) -> Result<(), String> {
        let features = context.to_feature_vector();

        // Load current params
        let mut params = Self::load_params(pool, action_name).await?;

        // Update with new observation
        params.update(&features, reward);

        // Save updated params
        Self::save_params(pool, action_name, &params).await?;

        // Update statistics
        sqlx::query(
            r#"
            UPDATE agent_linear_bandit 
            SET total_pulls = total_pulls + 1,
                total_reward = total_reward + ?,
                avg_reward = (total_reward + ?) / (total_pulls + 1)
            WHERE action_name = ?
            "#,
        )
        .bind(reward)
        .bind(reward)
        .bind(action_name)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Log a reward for later multi-scale computation
    pub async fn log_reward(
        pool: &Pool<Sqlite>,
        action_name: &str,
        context: &RichContext,
        immediate_reward: f32,
        feedback_type: &str,
    ) -> Result<i64, String> {
        let context_bytes = context.to_bytes();

        let result = sqlx::query(
            r#"
            INSERT INTO agent_reward_log (action_name, context_features, reward_immediate, feedback_type)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(action_name)
        .bind(&context_bytes)
        .bind(immediate_reward)
        .bind(feedback_type)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.last_insert_rowid())
    }

    /// Get the current bandit mode (linear or neural)
    pub async fn get_mode(pool: &Pool<Sqlite>) -> Result<String, String> {
        let mode: Option<String> = sqlx::query_scalar(
            "SELECT value_json FROM agent_state WHERE key = 'bandit_mode'",
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(mode
            .and_then(|m| serde_json::from_str(&m).ok())
            .unwrap_or_else(|| "linear".to_string()))
    }

    /// Get total training samples
    pub async fn total_samples(pool: &Pool<Sqlite>) -> Result<i64, String> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(total_pulls), 0) FROM agent_linear_bandit",
        )
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(count)
    }

    /// Check if ready for neural upgrade
    pub async fn ready_for_neural(pool: &Pool<Sqlite>) -> Result<bool, String> {
        let samples = Self::total_samples(pool).await?;
        let threshold: i64 = sqlx::query_scalar(
            "SELECT CAST(value_json AS INTEGER) FROM agent_state WHERE key = 'min_samples_for_neural'",
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .unwrap_or(100);

        Ok(samples >= threshold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_bandit_params() {
        let mut params = LinearBanditParams::new();

        let features = Array1::from(vec![0.5f32; FEATURE_DIM]);

        // Initial prediction should be 0
        let pred = params.predict(&features);
        assert!(pred.abs() < 1e-6);

        // Update with positive reward
        params.update(&features, 1.0);

        // Prediction should increase
        let new_pred = params.predict(&features);
        assert!(new_pred > pred);

        // Uncertainty should decrease
        let initial_uncertainty = LinearBanditParams::new().uncertainty(&features);
        let new_uncertainty = params.uncertainty(&features);
        assert!(new_uncertainty < initial_uncertainty);
    }

    #[test]
    fn test_params_serialization() {
        let mut params = LinearBanditParams::new();
        let features = Array1::from(vec![0.5f32; FEATURE_DIM]);
        params.update(&features, 0.8);

        let (theta_bytes, prec_bytes) = params.to_bytes();
        let restored = LinearBanditParams::from_bytes(&theta_bytes, &prec_bytes).unwrap();

        // Predictions should match
        let orig_pred = params.predict(&features);
        let restored_pred = restored.predict(&features);
        assert!((orig_pred - restored_pred).abs() < 1e-6);
    }
}
