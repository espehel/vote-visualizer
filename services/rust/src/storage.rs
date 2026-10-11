use crate::error::ApiError;
use crate::normalize::NormalizedVote;
use serde::Deserialize;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Json;
use std::time::Duration;
use time::OffsetDateTime;

pub async fn establish_connection(db_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(50)
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(10))
        .connect(db_url)
        .await?;

    Ok(pool)
}

#[derive(sqlx::FromRow)]
pub(crate) struct VoteRow {
    pub(crate) id: i64,
    pub(crate) case_id: i64,
    pub(crate) voted_at: OffsetDateTime,
    pub(crate) subject: Option<String>,
    pub(crate) adopted: bool,
    pub(crate) individual: bool,
    pub(crate) for_count: i32,
    pub(crate) against_count: i32,
    pub(crate) absent_count: i32,
    pub(crate) proposals: Json<serde_json::Value>,
    pub(crate) ballots: Json<Vec<BallotRow>>,
    pub(crate) imported_at: OffsetDateTime,
}

#[derive(Deserialize)]
pub(crate) struct BallotRow {
    pub(crate) representative_id: String,
    pub(crate) party_id: Option<String>,
    pub(crate) choice: String,
}

pub(crate) async fn find_vote(pool: &PgPool, vote_id: i64) -> Result<VoteRow, ApiError> {
    // One statement, so the vote and its ballots come from the same snapshot.
    sqlx::query_as::<_, VoteRow>(
        r#"
        SELECT
            v.id, v.case_id, v.voted_at, v.subject, v.adopted, v.individual,
            v.for_count, v.against_count, v.absent_count, v.proposals, v.imported_at,
            COALESCE(
                (
                    SELECT json_agg(
                        json_build_object(
                            'representative_id', b.representative_id,
                            'party_id', b.party_id,
                            'choice', b.choice
                        )
                        ORDER BY b.representative_id
                    )
                    FROM ballots b
                    WHERE b.vote_id = v.id
                ),
                '[]'::json
            ) AS ballots
        FROM votes v
        WHERE v.id = $1
        "#,
    )
    .bind(vote_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| ApiError::NotFound {
        message: format!("vote {vote_id} not found"),
    })
}

pub struct SourceSnapshot<'a> {
    pub kind: &'a str,
    pub url: &'a str,
    pub body: &'a [u8],
}

pub(crate) async fn insert_vote(
    pool: &PgPool,
    vote: &NormalizedVote,
    snapshots: &[SourceSnapshot<'_>],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query(
        r#"
        INSERT INTO votes (
            id, case_id, voted_at, subject, adopted, individual,
            for_count, against_count, absent_count, proposals
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT (id) DO UPDATE SET
            case_id = EXCLUDED.case_id,
            voted_at = EXCLUDED.voted_at,
            subject = EXCLUDED.subject,
            adopted = EXCLUDED.adopted,
            individual = EXCLUDED.individual,
            for_count = EXCLUDED.for_count,
            against_count = EXCLUDED.against_count,
            absent_count = EXCLUDED.absent_count,
            proposals = EXCLUDED.proposals,
            imported_at = now()
        "#,
    )
    .bind(vote.vote_id)
    .bind(vote.case_id)
    .bind(vote.voted_at)
    .bind(&vote.subject)
    .bind(vote.adopted)
    .bind(vote.individual)
    .bind(vote.for_count)
    .bind(vote.against_count)
    .bind(vote.absent_count)
    .bind(Json(&vote.proposals))
    .execute(&mut *tx)
    .await?;

    // Replace the entire set so removed source records disappear too.
    sqlx::query("DELETE FROM ballots WHERE vote_id = $1")
        .bind(vote.vote_id)
        .execute(&mut *tx)
        .await?;

    for ballot in &vote.ballots {
        sqlx::query(
            r#"
            INSERT INTO ballots (
                vote_id, representative_id, party_id, choice
            )
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(vote.vote_id)
        .bind(&ballot.representative_id)
        .bind(&ballot.party_id)
        .bind(ballot.choice.as_str())
        .execute(&mut *tx)
        .await?;
    }

    for snapshot in snapshots {
        sqlx::query(
            r#"
            INSERT INTO source_snapshots (
                vote_id, kind, source_url, body
            )
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (vote_id, kind) DO UPDATE SET
                source_url = EXCLUDED.source_url,
                body = EXCLUDED.body,
                saved_at = now()
            "#,
        )
        .bind(vote.vote_id)
        .bind(snapshot.kind)
        .bind(snapshot.url)
        .bind(snapshot.body)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}
