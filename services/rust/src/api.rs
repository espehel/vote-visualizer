use crate::api_dto::{ApiResult, VoteDto};
use crate::importer::{self, ImportReport, ImportVote};
use crate::storage;
use crate::stortinget_client::StortingetClient;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub source: StortingetClient,
}

async fn get_health() -> &'static str {
    "OK"
}

async fn get_vote(State(state): State<AppState>, Path(id): Path<i64>) -> ApiResult<VoteDto> {
    let vote = storage::find_vote(&state.pool, id).await?;

    Ok(Json(vote.into()))
}

async fn post_dev_imports_vote(
    State(state): State<AppState>,
    Json(command): Json<ImportVote>,
) -> ApiResult<ImportReport> {
    let report = importer::import_vote(&state.pool, &state.source, command).await?;

    Ok(Json(report))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(get_health))
        .route("/votes/{id}", get(get_vote))
        .route("/dev/imports/vote", post(post_dev_imports_vote))
}
