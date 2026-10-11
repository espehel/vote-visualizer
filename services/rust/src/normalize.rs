use crate::{
    error::ImportError,
    importer::ImportVote,
    stortinget_models::{CaseVote, CaseVotes, VoteProposal, VoteProposals, VoteResults},
};
use std::collections::HashSet;
use time::OffsetDateTime;

#[derive(Debug)]
pub struct NormalizedVote {
    pub case_id: i64,
    pub vote_id: i64,
    pub voted_at: OffsetDateTime,
    pub subject: Option<String>,
    pub adopted: bool,
    pub individual: bool,
    pub for_count: i32,
    pub against_count: i32,
    pub absent_count: i32,
    pub ballots: Vec<NormalizedBallot>,
    pub proposals: Vec<VoteProposal>,
}

#[derive(Debug)]
pub struct NormalizedBallot {
    pub representative_id: String,
    pub party_id: Option<String>,
    pub choice: VoteChoice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoteChoice {
    For,
    Against,
    Absent,
}

impl VoteChoice {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::For => "for",
            Self::Against => "against",
            Self::Absent => "absent",
        }
    }
}

fn invalid(message: impl Into<String>) -> ImportError {
    ImportError::InvalidSourceData {
        message: message.into(),
    }
}

/// Finds the requested vote in its case, so the importer can stop
/// before fetching results and proposals for a vote outside the case.
pub fn find_case_vote(
    command: ImportVote,
    case_votes: &CaseVotes,
) -> Result<&CaseVote, ImportError> {
    if case_votes.sak_id != command.case_id {
        return Err(invalid("case response has the wrong case ID"));
    }

    let mut matches = case_votes
        .sak_votering_liste
        .iter()
        .filter(|vote| vote.votering_id == command.vote_id);

    let vote = matches.next().ok_or_else(|| ImportError::NotFound {
        message: format!(
            "vote {} was not found in case {}",
            command.vote_id, command.case_id,
        ),
    })?;

    if matches.next().is_some() {
        return Err(invalid("case response contains duplicate vote IDs"));
    }

    Ok(vote)
}

pub fn normalize_vote(
    command: ImportVote,
    vote: &CaseVote,
    results: &VoteResults,
    proposals: &VoteProposals,
) -> Result<NormalizedVote, ImportError> {
    if results.votering_id != command.vote_id || proposals.votering_id != command.vote_id {
        return Err(invalid("results or proposals have the wrong vote ID"));
    }

    let mut representative_ids = HashSet::new();
    let mut ballots = Vec::new();
    let mut counts = [0_i64; 3]; // for, against, absent

    for result in &results.voteringsresultat_liste {
        let representative = &result.representant;

        if representative.id.trim().is_empty() {
            return Err(invalid("ballot has an empty representative ID"));
        }

        if !representative_ids.insert(representative.id.as_str()) {
            return Err(invalid(format!(
                "duplicate ballot for representative {}",
                representative.id,
            )));
        }

        let (choice, index) = match result.votering {
            1 => (VoteChoice::Absent, 2),
            2 => (VoteChoice::For, 0),
            3 => (VoteChoice::Against, 1),
            code => {
                return Err(invalid(format!(
                    "unknown vote code {code} for {}",
                    representative.id,
                )));
            }
        };

        counts[index] += 1;

        ballots.push(NormalizedBallot {
            representative_id: representative.id.clone(),
            party_id: representative.parti.as_ref().map(|party| party.id.clone()),
            choice,
        });
    }

    if vote.personlig_votering {
        let expected = [
            i64::from(vote.antall_for),
            i64::from(vote.antall_mot),
            i64::from(vote.antall_ikke_tilstede),
        ];

        if ballots.is_empty() || counts != expected {
            return Err(invalid(format!(
                "individual ballot counts {counts:?} \
                 do not match published counts {expected:?}",
            )));
        }
    }

    Ok(NormalizedVote {
        case_id: command.case_id,
        vote_id: command.vote_id,
        voted_at: parse_source_date(&vote.votering_tid)?,
        subject: vote.votering_tema.clone(),
        adopted: vote.vedtatt,
        individual: vote.personlig_votering,
        for_count: vote.antall_for,
        against_count: vote.antall_mot,
        absent_count: vote.antall_ikke_tilstede,
        ballots,
        proposals: proposals.voteringsforslag_liste.clone(),
    })
}

/// Parses `/Date(milliseconds[+-HHMM])/`, e.g. `/Date(1622552993113+0200)/`.
fn parse_source_date(value: &str) -> Result<OffsetDateTime, ImportError> {
    let malformed = || invalid(format!("invalid source date: {value}"));
    let is_digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());

    let inner = value
        .strip_prefix("/Date(")
        .and_then(|rest| rest.strip_suffix(")/"))
        .ok_or_else(malformed)?;

    // Split off the optional offset, skipping a leading minus on the milliseconds.
    let sign_len = usize::from(inner.starts_with('-'));
    let (millis, offset) = match inner[sign_len..].find(['+', '-']) {
        Some(index) => inner.split_at(sign_len + index),
        None => (inner, ""),
    };

    if !is_digits(&millis[sign_len..])
        || !(offset.is_empty() || offset.len() == 5 && is_digits(&offset[1..]))
    {
        return Err(malformed());
    }

    let milliseconds = millis
        .parse::<i64>()
        .map_err(|_| invalid(format!("source date out of range: {value}")))?;

    // The milliseconds already identify the UTC instant.
    // Do not apply the optional offset a second time.
    OffsetDateTime::from_unix_timestamp_nanos(i128::from(milliseconds) * 1_000_000)
        .map_err(|_| invalid(format!("source date out of range: {value}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn parses_date_with_offset_without_applying_it_twice() {
        let parsed = parse_source_date("/Date(1622552993113+0200)/").unwrap();
        assert_eq!(parsed, datetime!(2021-06-01 13:09:53.113 UTC));
    }

    #[test]
    fn parses_date_without_offset() {
        let parsed = parse_source_date("/Date(1622552993113)/").unwrap();
        assert_eq!(parsed, datetime!(2021-06-01 13:09:53.113 UTC));
    }

    #[test]
    fn parses_negative_timestamp() {
        let parsed = parse_source_date("/Date(-1000)/").unwrap();
        assert_eq!(parsed, datetime!(1969-12-31 23:59:59 UTC));
    }

    #[test]
    fn rejects_malformed_dates() {
        for value in [
            "",
            "1622552993113",
            "Date(1)",
            "/Date()/",
            "/Date(-)/",
            "/Date(abc)/",
            "/Date(+1)/",
            "/Date(1+02)/",
            "/Date(1+abcd)/",
            "/Date(1+0200+0200)/",
        ] {
            assert!(
                parse_source_date(value).is_err(),
                "{value:?} should be rejected"
            );
        }
    }

    #[test]
    fn rejects_out_of_range_date() {
        assert!(parse_source_date("/Date(9223372036854775807)/").is_err());
    }
}
