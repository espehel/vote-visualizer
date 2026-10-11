use crate::error::ImportError;
use crate::normalize::{find_case_vote, normalize_vote};
use crate::storage::{SourceSnapshot, insert_vote};
use crate::stortinget_client::StortingetClient;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

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
