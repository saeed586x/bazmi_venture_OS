//! Decision Memory - stores and retrieves decision history and patterns

use crate::capabilities::decision_gateway::{DecisionContext, DecisionOutcome};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Decision Memory - stores and retrieves decision history and patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionMemory {
    /// Stored decision records
    decision_records: HashMap<String, DecisionRecord>,
    /// Decision patterns for learning
    decision_patterns: HashMap<String, DecisionPattern>,
    /// Index for fast retrieval
    decision_index: DecisionIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub id: String,
    pub decision_outcome: DecisionOutcome,
    pub decision_context: DecisionContext,
    pub stored_at: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<String>,
    pub similarity_hash: String, // For finding similar decisions
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionPattern {
    pub id: String,
    pub pattern_name: String,
    pub description: String,
    pub conditions: Vec<PatternCondition>,
    pub typical_outcomes: Vec<TypicalOutcome>,
    pub confidence: f64, // 0.0 to 1.0
    pub frequency: u32,  // How often this pattern occurs
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternCondition {
    pub field: String,
    pub operator: PatternOperator,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternOperator {
    Equals,
    Contains,
    GreaterThan,
    LessThan,
    Between,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypicalOutcome {
    pub outcome_type: String,
    pub frequency: f64, // 0.0 to 1.0
    pub average_confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionIndex {
    pub by_decision_type: HashMap<String, Vec<String>>, // decision_type -> decision_ids
    pub by_maker: HashMap<String, Vec<String>>,         // maker -> decision_ids
    pub by_date: DateIndex,
    pub by_tags: HashMap<String, Vec<String>>, // tag -> decision_ids
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateIndex {
    pub by_day: HashMap<String, Vec<String>>, // YYYY-MM-DD -> decision_ids
    pub by_month: HashMap<String, Vec<String>>, // YYYY-MM -> decision_ids
    pub by_year: HashMap<String, Vec<String>>, // YYYY -> decision_ids
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityQuery {
    pub context_fields: Vec<String>,
    pub threshold: f64, // 0.0 to 1.0
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarDecision {
    pub decision_id: String,
    pub similarity_score: f64,
    pub outcome: DecisionOutcome,
}

impl DecisionMemory {
    /// Create a new decision memory
    pub fn new() -> Self {
        Self {
            decision_records: HashMap::new(),
            decision_patterns: HashMap::new(),
            decision_index: DecisionIndex {
                by_decision_type: HashMap::new(),
                by_maker: HashMap::new(),
                by_date: DateIndex {
                    by_day: HashMap::new(),
                    by_month: HashMap::new(),
                    by_year: HashMap::new(),
                },
                by_tags: HashMap::new(),
            },
        }
    }

    /// Store a decision record
    pub fn store_decision(
        &mut self,
        outcome: DecisionOutcome,
        context: DecisionContext,
        tags: Vec<String>,
    ) {
        let decision_id = outcome.decision_id.clone();
        let decision_type = match &outcome.decision_type {
            crate::capabilities::decision_gateway::DecisionType::PlanApproval => {
                "plan_approval".to_string()
            }
            crate::capabilities::decision_gateway::DecisionType::RiskMitigation => {
                "risk_mitigation".to_string()
            }
            crate::capabilities::decision_gateway::DecisionType::ResourceAllocation => {
                "resource_allocation".to_string()
            }
            crate::capabilities::decision_gateway::DecisionType::TimelineAdjustment => {
                "timeline_adjustment".to_string()
            }
            crate::capabilities::decision_gateway::DecisionType::Custom(s) => s.clone(),
        };

        let maker = match &outcome.made_by {
            crate::capabilities::decision_gateway::DecisionMaker::Automated => {
                "automated".to_string()
            }
            crate::capabilities::decision_gateway::DecisionMaker::Human { role, .. } => {
                role.clone()
            }
            crate::capabilities::decision_gateway::DecisionMaker::Hybrid { .. } => {
                "hybrid".to_string()
            }
        };

        let date = outcome.made_at;
        let date_key = date.format("%Y-%m-%d").to_string();
        let month_key = date.format("%Y-%m").to_string();
        let year_key = date.format("%Y").to_string();

        let similarity_hash = self.compute_similarity_hash(&context);

        let record = DecisionRecord {
            id: decision_id.clone(),
            decision_outcome: outcome,
            decision_context: context,
            stored_at: chrono::Utc::now(),
            tags: tags.clone(),
            similarity_hash,
        };

        // Store the record
        self.decision_records.insert(decision_id.clone(), record);

        // Update indexes
        self.index_decision(
            &decision_id,
            &decision_type,
            &maker,
            &date_key,
            &month_key,
            &year_key,
            &tags,
        );

        // Learn patterns from the decision
        self.learn_pattern(&decision_id);
    }

    /// Index a decision for fast retrieval
    #[allow(clippy::too_many_arguments)]
    fn index_decision(
        &mut self,
        decision_id: &str,
        decision_type: &str,
        maker: &str,
        date_key: &str,
        month_key: &str,
        year_key: &str,
        tags: &[String],
    ) {
        // Index by decision type
        self.decision_index
            .by_decision_type
            .entry(decision_type.to_string())
            .or_default()
            .push(decision_id.to_string());

        // Index by maker
        self.decision_index
            .by_maker
            .entry(maker.to_string())
            .or_default()
            .push(decision_id.to_string());

        // Index by date
        self.decision_index
            .by_date
            .by_day
            .entry(date_key.to_string())
            .or_default()
            .push(decision_id.to_string());

        self.decision_index
            .by_date
            .by_month
            .entry(month_key.to_string())
            .or_default()
            .push(decision_id.to_string());

        self.decision_index
            .by_date
            .by_year
            .entry(year_key.to_string())
            .or_default()
            .push(decision_id.to_string());

        // Index by tags
        for tag in tags {
            self.decision_index
                .by_tags
                .entry(tag.clone())
                .or_default()
                .push(decision_id.to_string());
        }
    }

    /// Learn patterns from stored decisions
    fn learn_pattern(&mut self, decision_id: &str) {
        if let Some(record) = self.decision_records.get(decision_id) {
            // Create a simple pattern based on decision type and outcome
            let pattern_id = format!(
                "pattern-{:?}-{:?}",
                record.decision_outcome.decision_type, record.decision_outcome.outcome
            );

            let pattern = self
                .decision_patterns
                .entry(pattern_id.clone())
                .or_insert_with(|| {
                    DecisionPattern {
                        id: pattern_id,
                        pattern_name: format!(
                            "Pattern for {:?}",
                            record.decision_outcome.decision_type
                        ),
                        description: "Auto-generated pattern".to_string(),
                        conditions: vec![], // Would be populated with actual conditions
                        typical_outcomes: vec![],
                        confidence: 0.5,
                        frequency: 0,
                        last_seen: chrono::Utc::now(),
                    }
                });

            pattern.frequency += 1;
            pattern.last_seen = chrono::Utc::now();

            // Update typical outcomes
            let outcome_type = match &record.decision_outcome.outcome {
                crate::capabilities::decision_gateway::DecisionResult::Approved => {
                    "approved".to_string()
                }
                crate::capabilities::decision_gateway::DecisionResult::Rejected => {
                    "rejected".to_string()
                }
                crate::capabilities::decision_gateway::DecisionResult::Conditional { .. } => {
                    "conditional".to_string()
                }
                crate::capabilities::decision_gateway::DecisionResult::Deferred { .. } => {
                    "deferred".to_string()
                }
                crate::capabilities::decision_gateway::DecisionResult::Escalated { .. } => {
                    "escalated".to_string()
                }
            };

            if let Some(existing_outcome) = pattern
                .typical_outcomes
                .iter_mut()
                .find(|o| o.outcome_type == outcome_type)
            {
                // Update existing outcome frequency
                let current_freq = existing_outcome.frequency;
                let new_freq = (current_freq * (pattern.frequency - 1) as f64 + 1.0)
                    / pattern.frequency as f64;
                existing_outcome.frequency = new_freq;
                existing_outcome.average_confidence = (existing_outcome.average_confidence
                    + record.decision_outcome.confidence)
                    / 2.0;
            } else {
                // Add new outcome
                pattern.typical_outcomes.push(TypicalOutcome {
                    outcome_type,
                    frequency: 1.0 / pattern.frequency as f64,
                    average_confidence: record.decision_outcome.confidence,
                });
            }

            // Update confidence based on consistency
            if pattern.typical_outcomes.len() == 1 {
                pattern.confidence = 0.9; // High confidence if consistent
            } else {
                pattern.confidence = 1.0 / pattern.typical_outcomes.len() as f64;
                // Lower confidence if varied outcomes
            }
        }
    }

    /// Retrieve a decision by ID
    pub fn get_decision(&self, decision_id: &str) -> Option<&DecisionRecord> {
        self.decision_records.get(decision_id)
    }

    /// Find decisions by decision type
    pub fn find_by_decision_type(&self, decision_type: &str) -> Vec<&DecisionRecord> {
        if let Some(ids) = self.decision_index.by_decision_type.get(decision_type) {
            ids.iter()
                .filter_map(|id| self.decision_records.get(id))
                .collect()
        } else {
            vec![]
        }
    }

    /// Find decisions by maker
    pub fn find_by_maker(&self, maker: &str) -> Vec<&DecisionRecord> {
        if let Some(ids) = self.decision_index.by_maker.get(maker) {
            ids.iter()
                .filter_map(|id| self.decision_records.get(id))
                .collect()
        } else {
            vec![]
        }
    }

    /// Find decisions by date range
    pub fn find_by_date_range(
        &self,
        start: &chrono::DateTime<chrono::Utc>,
        end: &chrono::DateTime<chrono::Utc>,
    ) -> Vec<&DecisionRecord> {
        let mut results = Vec::new();

        for record in self.decision_records.values() {
            if &record.decision_outcome.made_at >= start && &record.decision_outcome.made_at <= end
            {
                results.push(record);
            }
        }

        results
    }

    /// Find similar decisions based on context
    pub fn find_similar_decisions(
        &self,
        context: &DecisionContext,
        query: &SimilarityQuery,
    ) -> Vec<SimilarDecision> {
        let query_hash = self.compute_similarity_hash(context);
        let mut similarities = Vec::new();

        for (id, record) in &self.decision_records {
            let similarity = self.calculate_similarity(&query_hash, &record.similarity_hash);
            if similarity >= query.threshold {
                similarities.push(SimilarDecision {
                    decision_id: id.clone(),
                    similarity_score: similarity,
                    outcome: record.decision_outcome.clone(),
                });
            }
        }

        // Sort by similarity and limit results
        similarities.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());
        similarities.truncate(query.limit);

        similarities
    }

    /// Compute a similarity hash for a decision context
    fn compute_similarity_hash(&self, context: &DecisionContext) -> String {
        // In a real implementation, this would create a meaningful hash
        // based on key context fields. For now, we'll use a simplified approach.

        let task_count = context.plan.tasks.len();
        let goal_count = context.plan.goals.len();
        let constraint_count = context.plan.constraints.len();

        format!("{}-{}-{}", task_count, goal_count, constraint_count)
    }

    /// Calculate similarity between two hashes
    fn calculate_similarity(&self, hash1: &str, hash2: &str) -> f64 {
        // In a real implementation, this would use a proper similarity algorithm
        // For now, we'll use a simple approach based on hash equality

        if hash1 == hash2 {
            1.0
        } else {
            // Parse hashes and calculate numeric similarity
            let parts1: Vec<&str> = hash1.split('-').collect();
            let parts2: Vec<&str> = hash2.split('-').collect();

            if parts1.len() == 3 && parts2.len() == 3 {
                let mut similarity_sum = 0.0;
                for i in 0..3 {
                    if let (Ok(num1), Ok(num2)) =
                        (parts1[i].parse::<usize>(), parts2[i].parse::<usize>())
                    {
                        let max_val = num1.max(num2).max(1);
                        let diff = (num1 as f64 - num2 as f64).abs();
                        similarity_sum += 1.0 - (diff / max_val as f64);
                    }
                }
                similarity_sum / 3.0
            } else {
                0.0
            }
        }
    }

    /// Get decision patterns
    pub fn get_patterns(&self) -> &HashMap<String, DecisionPattern> {
        &self.decision_patterns
    }

    /// Get statistics about stored decisions
    pub fn get_statistics(&self) -> DecisionStatistics {
        DecisionStatistics {
            total_decisions: self.decision_records.len(),
            decision_types: self.decision_index.by_decision_type.len(),
            makers: self.decision_index.by_maker.len(),
            patterns_learned: self.decision_patterns.len(),
            average_confidence: if self.decision_records.is_empty() {
                0.0
            } else {
                self.decision_records
                    .values()
                    .map(|r| r.decision_outcome.confidence)
                    .sum::<f64>()
                    / self.decision_records.len() as f64
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionStatistics {
    pub total_decisions: usize,
    pub decision_types: usize,
    pub makers: usize,
    pub patterns_learned: usize,
    pub average_confidence: f64,
}

impl Default for DecisionMemory {
    fn default() -> Self {
        Self::new()
    }
}
