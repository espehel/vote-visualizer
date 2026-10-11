use serde::{Deserialize, Serialize};

// GET /eksport/voteringer?sakid=...
#[derive(Debug, Clone, Deserialize)]
pub struct CaseVotes {
    pub sak_id: i64,
    pub sak_votering_liste: Vec<CaseVote>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CaseVote {
    pub sak_id: i64,
    pub votering_id: i64,

    // The source uses -1 when there is no alternative vote.
    pub alternativ_votering_id: i64,

    // Preserve the source date string; normalize it separately.
    pub votering_tid: String,
    pub votering_tema: Option<String>,

    pub vedtatt: bool,
    pub personlig_votering: bool,

    // Refers to voting on the case as a whole;
    // this does NOT mean representatives have a free vote.
    pub fri_votering: bool,

    pub antall_for: i32,
    pub antall_mot: i32,
    pub antall_ikke_tilstede: i32,

    pub behandlingsrekkefoelge: i32,
    pub mote_kart_nummer: i32,
    pub dagsorden_sak_nummer: i32,

    pub kommentar: Option<String>,
    pub president: Option<Representative>,

    // Numeric source codes. Interpret them in normalization.
    // votering_metode was absent from the inspected response.
    pub votering_metode: Option<i32>,
    pub votering_resultat_type: i32,
    pub votering_resultat_type_tekst: Option<String>,
}

// GET /eksport/voteringsresultat?voteringid=...
#[derive(Debug, Clone, Deserialize)]
pub struct VoteResults {
    pub votering_id: i64,
    pub voteringsresultat_liste: Vec<RepresentativeVote>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RepresentativeVote {
    pub representant: Representative,

    // These contain basic person records, without party information.
    pub vara_for: Option<Person>,
    pub fast_vara_for: Option<Person>,

    // Preserve the raw numeric code.
    pub votering: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Representative {
    pub id: String,
    pub fornavn: String,
    pub etternavn: String,

    pub parti: Option<Party>,
    pub fylke: Option<Constituency>,
    pub vara_representant: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Person {
    pub id: String,
    pub fornavn: String,
    pub etternavn: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Party {
    pub id: String,
    pub navn: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Constituency {
    pub id: String,
    pub navn: String,
}

// GET /eksport/voteringsforslag?voteringid=...
#[derive(Debug, Clone, Deserialize)]
pub struct VoteProposals {
    pub votering_id: i64,
    pub voteringsforslag_liste: Vec<VoteProposal>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VoteProposal {
    pub forslag_id: i64,

    pub forslag_betegnelse: Option<String>,
    pub forslag_betegnelse_kort: Option<String>,

    // Preserve HTML as supplied by the source.
    pub forslag_tekst: Option<String>,

    pub forslag_paa_vegne_av_tekst: Option<String>,

    // This is a person ID, not an embedded representative object.
    pub forslag_levert_av_representant: Option<String>,

    pub forslag_levert_av_parti_liste: Option<Vec<Party>>,

    pub forslag_sorteringsnummer: i32,

    // Numeric source code; normalize separately.
    pub forslag_type: i32,
}
