use tauri::State;

use crate::db::estimates;
use crate::error::{AppError, AppResult};
use crate::models::Estimate;
use crate::services::ai_estimator;
use crate::DbState;

#[tauri::command]
pub fn list_estimates(state: State<DbState>) -> AppResult<Vec<Estimate>> {
    let conn = state
        .0
        .lock()
        .map_err(|e| AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string())))?;
    estimates::list_estimates(&conn)
}

#[tauri::command]
pub async fn run_ai_estimate(
    state: State<'_, DbState>,
    api_key: String,
    project_description: String,
) -> AppResult<Estimate> {
    if api_key.is_empty() {
        return Err(AppError::Validation(
            "Claude API key is required. Set it in Settings.".to_string(),
        ));
    }

    // Gather historical data under the lock, then release before async call
    let historical_data = {
        let conn = state.0.lock().map_err(|e| {
            AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
        })?;
        ai_estimator::gather_historical_data_external(&conn)?
    };

    let estimate = ai_estimator::estimate_project_with_history(
        &api_key,
        &project_description,
        historical_data,
    )
    .await?;

    // Save the estimate under a new lock
    let conn = state
        .0
        .lock()
        .map_err(|e| AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string())))?;

    let risk_flags = serde_json::to_value(&estimate.risk_flags).unwrap_or_default();
    let similar = serde_json::json!([]);

    estimates::save_estimate(
        &conn,
        &project_description,
        estimate.conservative_hours,
        estimate.realistic_hours,
        estimate.optimistic_hours,
        estimate.confidence_score,
        &risk_flags,
        &similar,
        Some(&estimate.reasoning),
        estimate.raw_response.as_deref(),
    )
}
