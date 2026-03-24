use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

#[derive(Debug, Deserialize)]
struct ClaudeContent {
    text: String,
}

#[derive(Debug, Deserialize)]
pub struct EstimateResult {
    pub optimistic_hours: f64,
    pub realistic_hours: f64,
    pub conservative_hours: f64,
    pub confidence_score: f64,
    pub risk_flags: Vec<String>,
    pub reasoning: String,
    #[serde(skip)]
    pub raw_response: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct HistoricalProject {
    pub name: String,
    pub estimated_hours: f64,
    pub actual_hours: f64,
}

fn build_prompt(project_description: &str, historical_projects: &[HistoricalProject]) -> String {
    let mut prompt = String::from(
        "You are a project estimation expert for freelance software development. \
         Analyze the following project description and provide an hour estimate.\n\n",
    );

    prompt.push_str(&format!(
        "## Project Description\n{}\n\n",
        project_description
    ));

    if historical_projects.is_empty() {
        prompt.push_str(
            "## Context\nNo historical project data available. \
             Use industry benchmarks for similar freelance projects.\n\n",
        );
    } else {
        prompt.push_str("## Historical Projects (for calibration)\n");
        for p in historical_projects {
            prompt.push_str(&format!(
                "- {}: estimated {}h, actual {}h (delta: {:.0}%)\n",
                p.name,
                p.estimated_hours,
                p.actual_hours,
                ((p.actual_hours - p.estimated_hours) / p.estimated_hours * 100.0)
            ));
        }
        prompt.push_str("\n");
    }

    prompt.push_str(
        "## Instructions\n\
         Respond with ONLY a JSON object (no markdown, no code fences) with these fields:\n\
         - optimistic_hours: best case scenario (number)\n\
         - realistic_hours: most likely scenario (number)\n\
         - conservative_hours: worst case with buffer (number)\n\
         - confidence_score: 0.0-1.0 how confident you are (number)\n\
         - risk_flags: array of risk factor strings\n\
         - reasoning: brief explanation of your estimate (string)\n\n\
         Important: confidence_score should be lower (0.3-0.5) when no historical data is available.",
    );

    prompt
}

pub fn gather_historical_data_external(conn: &Connection) -> AppResult<Vec<HistoricalProject>> {
    let mut stmt = conn.prepare(
        "SELECT p.name, p.budget_hours, COALESCE(SUM(te.duration_secs), 0) as total_secs
         FROM projects p
         LEFT JOIN time_entries te ON te.project_id = p.id
         WHERE p.status = 'completed' AND p.budget_hours IS NOT NULL
         GROUP BY p.id
         HAVING total_secs > 0
         ORDER BY p.updated_at DESC
         LIMIT 10",
    )?;

    let projects = stmt
        .query_map([], |row| {
            let name: String = row.get(0)?;
            let budget_hours: f64 = row.get(1)?;
            let total_secs: i64 = row.get(2)?;
            Ok(HistoricalProject {
                name,
                estimated_hours: budget_hours,
                actual_hours: total_secs as f64 / 3600.0,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(projects)
}

pub async fn estimate_project_with_history(
    api_key: &str,
    project_description: &str,
    historical: Vec<HistoricalProject>,
) -> AppResult<EstimateResult> {
    let prompt = build_prompt(project_description, &historical);

    let client = reqwest::Client::new();
    let request = ClaudeRequest {
        model: "claude-sonnet-4-5-20250929".to_string(),
        max_tokens: 1024,
        messages: vec![ClaudeMessage {
            role: "user".to_string(),
            content: prompt,
        }],
    };

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| AppError::AiEstimation(format!("API request failed: {e}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(AppError::AiEstimation(format!(
            "Claude API error ({status}): {body}"
        )));
    }

    let claude_response: ClaudeResponse = response
        .json()
        .await
        .map_err(|e| AppError::AiEstimation(format!("Failed to parse response: {e}")))?;

    let text = claude_response
        .content
        .first()
        .map(|c| c.text.clone())
        .ok_or_else(|| AppError::AiEstimation("Empty response from Claude".to_string()))?;

    let mut estimate: EstimateResult = serde_json::from_str(&text).map_err(|e| {
        AppError::AiEstimation(format!("Failed to parse estimate JSON: {e}. Raw: {text}"))
    })?;

    estimate.raw_response = Some(text);
    Ok(estimate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_prompt_without_history() {
        let prompt = build_prompt("Build a landing page with contact form", &[]);
        assert!(prompt.contains("landing page"));
        assert!(prompt.contains("No historical project data"));
        assert!(prompt.contains("confidence_score"));
    }

    #[test]
    fn test_build_prompt_with_history() {
        let history = vec![
            HistoricalProject {
                name: "Website A".to_string(),
                estimated_hours: 20.0,
                actual_hours: 25.0,
            },
            HistoricalProject {
                name: "Website B".to_string(),
                estimated_hours: 40.0,
                actual_hours: 38.0,
            },
        ];
        let prompt = build_prompt("Build an e-commerce site", &history);
        assert!(prompt.contains("Website A"));
        assert!(prompt.contains("Website B"));
        assert!(prompt.contains("Historical Projects"));
    }

    #[test]
    fn test_parse_estimate_response() {
        let json = r#"{
            "optimistic_hours": 15,
            "realistic_hours": 25,
            "conservative_hours": 40,
            "confidence_score": 0.7,
            "risk_flags": ["scope creep", "unclear requirements"],
            "reasoning": "Based on similar projects"
        }"#;

        let parsed: EstimateResult = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.realistic_hours, 25.0);
        assert_eq!(parsed.risk_flags.len(), 2);
    }
}
