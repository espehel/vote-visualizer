use crate::error::ImportError;
use crate::normalize::{find_case_vote, normalize_vote};
use crate::storage::{SourceSnapshot, insert_vote};
use crate::stortinget_client::StortingetClient;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::types::Json;
use time::OffsetDateTime;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ImportVote {
    pub case_id: i64,
    pub vote_id: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportReport {
    pub case_id: i64,
    pub vote_id: i64,

    /// Number of representative vote records saved,
    /// including records indicating absence.
    pub ballot_count: usize,

    /// Number of proposals associated with this vote and saved.
    pub proposal_count: usize,
}

/// Concurrent imports are safe without extra locking: the vote upsert locks
/// the vote row, so transactions for the same vote run one after another.
pub async fn import_vote(
    pool: &PgPool,
    source: &StortingetClient,
    command: ImportVote,
) -> Result<ImportReport, ImportError> {
    // Finish HTTP requests before opening a database transaction.
    let case_votes = source.votes_for_case(command.case_id).await?;
    let case_vote = find_case_vote(command, &case_votes.data)?;

    let results = source.results(command.vote_id).await?;
    let proposals = source.proposals(command.vote_id).await?;

    let vote = normalize_vote(command, case_vote, &results.data, &proposals.data)?;

    let snapshots = [
        SourceSnapshot {
            kind: "case_votes",
            url: case_votes.url.as_str(),
            body: case_votes.raw.as_ref(),
        },
        SourceSnapshot {
            kind: "vote_results",
            url: results.url.as_str(),
            body: results.raw.as_ref(),
        },
        SourceSnapshot {
            kind: "vote_proposals",
            url: proposals.url.as_str(),
            body: proposals.raw.as_ref(),
        },
    ];

    insert_vote(pool, &vote, &snapshots).await?;

    Ok(ImportReport {
        case_id: vote.case_id,
        vote_id: vote.vote_id,
        ballot_count: vote.ballots.len(),
        proposal_count: vote.proposals.len(),
    })
}

/// A stored vote together with the raw source responses it was imported from.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ImportedVote {
    pub vote_id: i64,
    pub case_id: i64,
    pub subject: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub imported_at: OffsetDateTime,

    /// Source response bodies, `None` if the snapshot is missing.
    pub case_votes: Option<Json<serde_json::Value>>,
    pub results: Option<Json<serde_json::Value>>,
    pub proposals: Option<Json<serde_json::Value>>,
}

pub(crate) async fn find_imported_vote(
    pool: &PgPool,
    vote_id: i64,
) -> Result<ImportedVote, ImportError> {
    // Cast to json rather than jsonb to keep the bodies as Stortinget sent them.
    sqlx::query_as::<_, ImportedVote>(
        r#"
        SELECT
            v.id AS vote_id, v.case_id, v.subject, v.imported_at,
            (
                SELECT convert_from(s.body, 'UTF8')::json
                FROM source_snapshots s
                WHERE s.vote_id = v.id AND s.kind = 'case_votes'
            ) AS case_votes,
            (
                SELECT convert_from(s.body, 'UTF8')::json
                FROM source_snapshots s
                WHERE s.vote_id = v.id AND s.kind = 'vote_results'
            ) AS results,
            (
                SELECT convert_from(s.body, 'UTF8')::json
                FROM source_snapshots s
                WHERE s.vote_id = v.id AND s.kind = 'vote_proposals'
            ) AS proposals
        FROM votes v
        WHERE v.id = $1
        "#,
    )
    .bind(vote_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| ImportError::NotFound {
        message: format!("vote {vote_id} not found"),
    })
}

/// A stored vote as listed in the admin overview.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ImportedVoteSummary {
    pub id: i64,
    pub subject: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub imported_at: OffsetDateTime,
}

/// Lists every stored vote, most recently imported first.
pub(crate) async fn get_imported_votes(
    pool: &PgPool,
) -> Result<Vec<ImportedVoteSummary>, ImportError> {
    let votes = sqlx::query_as::<_, ImportedVoteSummary>(
        r#"
        SELECT id, subject, imported_at
        FROM votes
        ORDER BY imported_at DESC, id DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(votes)
}
