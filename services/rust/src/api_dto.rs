use crate::error::ApiError;
use crate::storage::{BallotRow, VoteRow};
use axum::Json;
use axum::http::StatusCode;
use serde::Serialize;
use time::OffsetDateTime;

pub type ApiPostResult<T> = Result<(StatusCode, Json<T>), ApiError>;
pub type ApiResult<T> = Result<Json<T>, ApiError>;

#[derive(Serialize)]
pub struct VoteDto {
    id: i64,
    case_id: i64,
    #[serde(with = "time::serde::rfc3339")]
    voted_at: OffsetDateTime,
    subject: Option<String>,
    adopted: bool,
    individual: bool,
    for_count: i32,
    against_count: i32,
    absent_count: i32,
    proposals: serde_json::Value,
    ballots: Vec<BallotDto>,
    #[serde(with = "time::serde::rfc3339")]
    imported_at: OffsetDateTime,
}

#[derive(Serialize)]
pub struct BallotDto {
    representative_id: String,
    party_id: Option<String>,
    choice: String,
}

impl From<VoteRow> for VoteDto {
    fn from(vote: VoteRow) -> Self {
        Self {
            id: vote.id,
            case_id: vote.case_id,
            voted_at: vote.voted_at,
            subject: vote.subject,
            adopted: vote.adopted,
            individual: vote.individual,
            for_count: vote.for_count,
            against_count: vote.against_count,
            absent_count: vote.absent_count,
            proposals: vote.proposals.0,
            ballots: vote.ballots.0.into_iter().map(BallotDto::from).collect(),
            imported_at: vote.imported_at,
        }
    }
}

impl From<BallotRow> for BallotDto {
    fn from(ballot: BallotRow) -> Self {
        Self {
            representative_id: ballot.representative_id,
            party_id: ballot.party_id,
            choice: ballot.choice,
        }
    }
}
