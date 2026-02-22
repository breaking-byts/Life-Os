//! Agent Module - Maximum Intelligence Productivity Agent
//!
//! This module provides the intelligent agent system that learns user preferences
//! and optimizes recommendations for both immediate satisfaction and long-term goals.
//!
//! ## Components
//!
//! - **insights**: Legacy insight generation (rule-based + simple bandit)
//! - **intelligence**: New Maximum Intelligence Agent with full ML pipeline

pub mod insights;
pub mod intelligence;

#[cfg(test)]
mod intelligence_test;

// Re-export the main intelligence agent
pub use intelligence::{
    IntelligenceAgent, 
    AgentRecommendation, 
    AgentStatus,
    BigThreeGoal,
};
