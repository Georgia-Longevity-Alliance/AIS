//! Consolidator — merges multiple proposals into a single coherent view.
//!
//! Consolidation is NOT forced consensus. When evidence doesn't justify
//! closure, the result is OPEN or CONFLICT. The consolidator preserves
//! disagreements while producing the best current model.

use serde::{Deserialize, Serialize};

/// Result of consolidating multiple proposals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidatedView {
    /// The consolidated claims (accepted into current model).
    pub accepted: Vec<serde_json::Value>,
    /// Claims that are in conflict — both sides preserved.
    pub conflicts: Vec<ConflictRecord>,
    /// Open questions — insufficient evidence to decide.
    pub open_questions: Vec<serde_json::Value>,
    /// Deltas that were explicitly rejected.
    pub rejected: Vec<RejectionRecord>,
    /// Summary of the consolidation.
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictRecord {
    pub subject: String,
    pub positions: Vec<Position>,
    pub status: ConflictStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub claim_id: String,
    pub claim_text: String,
    pub supporters: u32,
    pub evidence_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConflictStatus {
    Active,
    Resolved,
    Stale,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectionRecord {
    pub delta_id: String,
    pub reason: String,
}

/// The consolidator takes proposed deltas and produces a consolidated view.
pub struct Consolidator;

impl Consolidator {
    /// Consolidate a set of proposals into a coherent view.
    ///
    /// Strategy (v1): majority vote with evidence weighting.
    /// - Claims with >50% support + at least 1 evidence → accepted
    /// - Claims with conflicting evidence → conflict
    /// - Claims with insufficient evidence → open
    pub fn consolidate(proposals: &[serde_json::Value]) -> ConsolidatedView {
        let mut accepted = Vec::new();
        let mut conflicts = Vec::new();
        let mut open_questions = Vec::new();
        let mut rejected = Vec::new();

        for proposal in proposals {
            let evidence_count = proposal
                .get("evidence")
                .and_then(|e| e.as_array())
                .map_or(0, |a| a.len());

            let status = proposal
                .get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("PROPOSED");

            match status {
                "CONTENTED" => {
                    // Check if we have another position on the same subject
                    let subject = proposal
                        .get("subject")
                        .and_then(|s| s.as_str())
                        .unwrap_or("");
                    if let Some(existing) = conflicts.iter_mut().find(|c: &&mut ConflictRecord| {
                        c.subject == subject
                    }) {
                        // Add position to existing conflict
                        existing.positions.push(Position {
                            claim_id: proposal
                                .get("claim_id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .into(),
                            claim_text: proposal
                                .get("claim")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .into(),
                            supporters: 0,
                            evidence_count: evidence_count as u32,
                        });
                    } else {
                        conflicts.push(ConflictRecord {
                            subject: subject.into(),
                            positions: vec![Position {
                                claim_id: proposal
                                    .get("claim_id")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .into(),
                                claim_text: proposal
                                    .get("claim")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .into(),
                                supporters: 0,
                                evidence_count: evidence_count as u32,
                            }],
                            status: ConflictStatus::Active,
                        });
                    }
                }
                "REFUTED" => {
                    rejected.push(RejectionRecord {
                        delta_id: proposal
                            .get("delta_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .into(),
                        reason: proposal
                            .get("refuted_reason")
                            .and_then(|v| v.as_str())
                            .unwrap_or("No reason provided")
                            .into(),
                    });
                }
                "OPEN" => {
                    open_questions.push(proposal.clone());
                }
                _ => {
                    // SUPPORTED, TESTED, REPLICATED → accept if has evidence
                    if evidence_count > 0 {
                        accepted.push(proposal.clone());
                    } else {
                        open_questions.push(proposal.clone());
                    }
                }
            }
        }

        let summary = format!(
            "Consolidation: {} accepted, {} conflicts, {} open, {} rejected",
            accepted.len(),
            conflicts.len(),
            open_questions.len(),
            rejected.len()
        );

        ConsolidatedView {
            accepted,
            conflicts,
            open_questions,
            rejected,
            summary,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consolidate_accepted() {
        let proposals = vec![serde_json::json!({
            "claim_id": "c1",
            "status": "SUPPORTED",
            "subject": "Centriole elimination",
            "claim": "Pedigree Score predicts elimination timing",
            "evidence": [{"id": "e1", "type": "supporting"}]
        })];

        let view = Consolidator::consolidate(&proposals);
        assert_eq!(view.accepted.len(), 1);
        assert_eq!(view.conflicts.len(), 0);
    }

    #[test]
    fn test_consolidate_conflict() {
        let proposals = vec![
            serde_json::json!({
                "claim_id": "c1",
                "status": "CONTENTED",
                "subject": "Centriole elimination",
                "claim": "Position A: PCM loss precedes SAS-4 loss",
                "evidence": [{"id": "e1"}]
            }),
            serde_json::json!({
                "claim_id": "c2",
                "status": "CONTENTED",
                "subject": "Centriole elimination",
                "claim": "Position B: SAS-4 loss is independent of PCM",
                "evidence": [{"id": "e2"}]
            }),
        ];

        let view = Consolidator::consolidate(&proposals);
        assert_eq!(view.conflicts.len(), 1);
        assert_eq!(view.conflicts[0].positions.len(), 2);
    }
}
