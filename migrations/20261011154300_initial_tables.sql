CREATE TABLE votes
(
    id            BIGINT PRIMARY KEY,
    case_id       BIGINT      NOT NULL,
    voted_at      TIMESTAMPTZ NOT NULL,
    subject       TEXT,
    adopted       BOOLEAN     NOT NULL,
    individual    BOOLEAN     NOT NULL,
    for_count     INTEGER     NOT NULL CHECK (for_count >= 0),
    against_count INTEGER     NOT NULL CHECK (against_count >= 0),
    absent_count  INTEGER     NOT NULL CHECK (absent_count >= 0),
    proposals     JSONB       NOT NULL,
    imported_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX votes_case_id_idx ON votes (case_id);

CREATE TABLE ballots
(
    vote_id           BIGINT NOT NULL REFERENCES votes (id) ON DELETE CASCADE,
    representative_id TEXT   NOT NULL,
    party_id          TEXT,
    choice            TEXT   NOT NULL CHECK (choice IN ('for', 'against', 'absent')),
    PRIMARY KEY (vote_id, representative_id)
);

CREATE TABLE source_snapshots
(
    vote_id    BIGINT      NOT NULL REFERENCES votes (id) ON DELETE CASCADE,
    kind       TEXT        NOT NULL,
    source_url TEXT        NOT NULL,
    body       BYTEA       NOT NULL,
    saved_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (vote_id, kind)
);
