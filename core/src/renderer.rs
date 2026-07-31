//! Renderer — produces human-readable and machine-readable views.
//!
//! A Noepedia publication can be rendered as:
//! - An article (Markdown)
//! - A summary (plain text)
//! - A claim map (graph)
//! - A machine-readable API response (JSON)
//! - An LLM context (structured prompt)

use crate::consolidator::ConsolidatedView;
use serde::{Deserialize, Serialize};

/// Output format for rendering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RenderFormat {
    /// Full article in Markdown.
    Article,
    /// Concise summary.
    Summary,
    /// Graph of claims and their relationships.
    ClaimMap,
    /// Machine-readable JSON for API consumption.
    ApiJson,
    /// Structured context for LLM prompts.
    LLMContext,
}

/// The renderer takes a consolidated view and produces output.
pub struct Renderer;

impl Renderer {
    /// Render a consolidated view in the requested format.
    pub fn render(view: &ConsolidatedView, format: RenderFormat, title: &str) -> String {
        match format {
            RenderFormat::Article => Self::render_article(view, title),
            RenderFormat::Summary => Self::render_summary(view, title),
            RenderFormat::ClaimMap => Self::render_claim_map(view, title),
            RenderFormat::ApiJson => Self::render_api_json(view, title),
            RenderFormat::LLMContext => Self::render_llm_context(view, title),
        }
    }

    /// Render as a full Markdown article.
    fn render_article(view: &ConsolidatedView, title: &str) -> String {
        let mut out = String::new();

        // Title
        out.push_str(&format!("# {}\n\n", title));

        // Summary
        out.push_str(&format!("_{}_\n\n", view.summary));

        // Accepted Claims
        if !view.accepted.is_empty() {
            out.push_str("## Accepted Claims\n\n");
            for (i, claim) in view.accepted.iter().enumerate() {
                let subject = claim["subject"].as_str().unwrap_or("Untitled");
                let text = claim["claim"].as_str().unwrap_or("");
                let status = claim["status"].as_str().unwrap_or("PROPOSED");
                let evidence_count = claim["evidence"]
                    .as_array()
                    .map_or(0, |a| a.len());

                out.push_str(&format!(
                    "### {}. {}\n\n**Claim:** {}\n\n**Status:** `{}`\n\n**Evidence:** {} source(s)\n\n---\n\n",
                    i + 1,
                    subject,
                    text,
                    status,
                    evidence_count
                ));
            }
        }

        // Conflicts
        if !view.conflicts.is_empty() {
            out.push_str("## ⚠️ Conflicts\n\n");
            for conflict in &view.conflicts {
                out.push_str(&format!("### {}\n\n", conflict.subject));
                out.push_str("| Position | Evidence |\n|----------|----------|\n");
                for pos in &conflict.positions {
                    out.push_str(&format!(
                        "| {} | {} sources |\n",
                        pos.claim_text, pos.evidence_count
                    ));
                }
                out.push_str("\n");
            }
        }

        // Open Questions
        if !view.open_questions.is_empty() {
            out.push_str("## 🔓 Open Questions\n\n");
            for q in &view.open_questions {
                let subject = q["subject"].as_str().unwrap_or("Open question");
                out.push_str(&format!("- {}\n", subject));
            }
            out.push_str("\n");
        }

        // Rejected
        if !view.rejected.is_empty() {
            out.push_str("## ❌ Rejected Proposals\n\n");
            for r in &view.rejected {
                out.push_str(&format!(
                    "- `{}`: {}\n",
                    r.delta_id, r.reason
                ));
            }
            out.push_str("\n");
        }

        out
    }

    /// Render as a concise summary.
    fn render_summary(view: &ConsolidatedView, title: &str) -> String {
        let mut out = format!("# {} — Summary\n\n", title);
        out.push_str(&view.summary);
        out.push('\n');

        if !view.conflicts.is_empty() {
            out.push_str(&format!("\n⚠️ {} unresolved conflict(s)\n", view.conflicts.len()));
        }
        if !view.open_questions.is_empty() {
            out.push_str(&format!("🔓 {} open question(s)\n", view.open_questions.len()));
        }

        out
    }

    /// Render as a claim map (Mermaid graph).
    fn render_claim_map(view: &ConsolidatedView, title: &str) -> String {
        let mut out = format!("```mermaid\ngraph TD\n");
        out.push_str(&format!("  TITLE[\"{}\"]\n", title));

        for (i, claim) in view.accepted.iter().enumerate() {
            let subject = claim["subject"].as_str().unwrap_or("Claim");
            let status = claim["status"].as_str().unwrap_or("?");
            out.push_str(&format!("  C{}[\"{} ({})\"]\n", i, subject, status));
        }

        for conflict in &view.conflicts {
            out.push_str(&format!(
                "  CONFLICT{}[\"⚠️ {} ({} positions)\"]\n",
                conflict.subject.chars().take(8).collect::<String>(),
                conflict.subject,
                conflict.positions.len()
            ));
        }

        out.push_str("```\n");
        out
    }

    /// Render as machine-readable JSON.
    fn render_api_json(view: &ConsolidatedView, title: &str) -> String {
        let output = serde_json::json!({
            "title": title,
            "summary": view.summary,
            "accepted_claims": view.accepted,
            "conflicts": view.conflicts,
            "open_questions": view.open_questions,
            "rejected": view.rejected
        });
        serde_json::to_string_pretty(&output).unwrap_or_default()
    }

    /// Render as LLM context — structured for a model to read.
    fn render_llm_context(view: &ConsolidatedView, title: &str) -> String {
        let mut ctx = format!(
            "=== KNOWLEDGE FIELD: {} ===\n\n",
            title
        );

        ctx.push_str("## CURRENT CONSENSUS\n\n");
        for claim in &view.accepted {
            let text = claim["claim"].as_str().unwrap_or("");
            let status = claim["status"].as_str().unwrap_or("?");
            ctx.push_str(&format!("- [{}] {}\n", status, text));
        }

        if !view.conflicts.is_empty() {
            ctx.push_str("\n## ACTIVE CONFLICTS (do not resolve — report both sides)\n\n");
            for conflict in &view.conflicts {
                ctx.push_str(&format!("### {}\n", conflict.subject));
                for pos in &conflict.positions {
                    ctx.push_str(&format!("- Position: {}\n", pos.claim_text));
                }
            }
        }

        if !view.open_questions.is_empty() {
            ctx.push_str("\n## OPEN QUESTIONS (insufficient evidence)\n\n");
            for q in &view.open_questions {
                let subject = q["subject"].as_str().unwrap_or("?");
                ctx.push_str(&format!("- {}\n", subject));
            }
        }

        ctx.push_str("\n---\n");
        ctx.push_str("INSTRUCTION: Use only claims with status TESTED or REPLICATED for operational decisions.\n");
        ctx.push_str("When a conflict exists, acknowledge both sides rather than choosing one.\n");
        ctx.push_str("For open questions, state that evidence is insufficient.\n");

        ctx
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consolidator::Consolidator;

    fn make_test_view() -> ConsolidatedView {
        let proposals = vec![
            serde_json::json!({
                "claim_id": "c1",
                "status": "SUPPORTED",
                "subject": "Test claim",
                "claim": "This is a verified claim",
                "evidence": [{"id": "e1"}]
            }),
            serde_json::json!({
                "claim_id": "c2",
                "status": "CONTENTED",
                "subject": "Conflict topic",
                "claim": "Position A",
                "evidence": [{"id": "e2"}]
            }),
            serde_json::json!({
                "claim_id": "c3",
                "status": "OPEN",
                "subject": "Open question",
                "claim": "Unknown",
                "evidence": []
            }),
        ];
        Consolidator::consolidate(&proposals)
    }

    #[test]
    fn test_render_article() {
        let view = make_test_view();
        let article = Renderer::render(&view, RenderFormat::Article, "Test Publication");
        assert!(article.contains("# Test Publication"));
        assert!(article.contains("Accepted Claims"));
        assert!(article.contains("Conflicts"));
        assert!(article.contains("Open Questions"));
    }

    #[test]
    fn test_render_api_json() {
        let view = make_test_view();
        let json = Renderer::render(&view, RenderFormat::ApiJson, "Test");
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["title"], "Test");
        assert!(parsed["accepted_claims"].is_array());
    }

    #[test]
    fn test_render_llm_context() {
        let view = make_test_view();
        let ctx = Renderer::render(&view, RenderFormat::LLMContext, "Test");
        assert!(ctx.contains("CURRENT CONSENSUS"));
        assert!(ctx.contains("ACTIVE CONFLICTS"));
        assert!(ctx.contains("OPEN QUESTIONS"));
    }
}
