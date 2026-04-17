CREATE TABLE accounts (
    id         INTEGER PRIMARY KEY,
    yaml_key   TEXT NOT NULL UNIQUE,
    enabled    BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE info_sources (
    id          INTEGER PRIMARY KEY,
    yaml_key    TEXT NOT NULL UNIQUE,
    account_id  INTEGER NOT NULL REFERENCES accounts(id),
    enabled     BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE raw_materials (
    id            INTEGER PRIMARY KEY,
    account_id    INTEGER NOT NULL REFERENCES accounts(id),
    source_id     INTEGER NOT NULL REFERENCES info_sources(id),
    natural_key   TEXT NOT NULL,
    title         TEXT NOT NULL,
    url           TEXT,
    summary       TEXT,
    raw_json      TEXT NOT NULL,
    metadata_json TEXT,
    status        TEXT NOT NULL DEFAULT 'unprocessed'
                  CHECK(status IN ('unprocessed','processed','filtered_out','error')),
    fetched_at    TEXT NOT NULL DEFAULT (datetime('now')),
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(account_id, natural_key)
);

CREATE TABLE drafts (
    id              INTEGER PRIMARY KEY,
    account_id      INTEGER NOT NULL REFERENCES accounts(id),
    raw_material_id INTEGER NOT NULL REFERENCES raw_materials(id),
    template_type   TEXT NOT NULL CHECK(template_type IN ('T1','T2','T3','T4','T5')),
    body            TEXT NOT NULL,
    media_json      TEXT,
    scheduled_at    TEXT,
    review_status   TEXT NOT NULL DEFAULT 'pending'
                    CHECK(review_status IN ('pending','approved','rejected')),
    reviewed_at     TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE posts (
    id              INTEGER PRIMARY KEY,
    account_id      INTEGER NOT NULL REFERENCES accounts(id),
    draft_id        INTEGER NOT NULL REFERENCES drafts(id),
    template_type   TEXT NOT NULL CHECK(template_type IN ('T1','T2','T3','T4','T5')),
    body            TEXT NOT NULL,
    post_url        TEXT,
    scheduled_at    TEXT,
    posted_at       TEXT,
    recorded_at     TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE post_metrics (
    id               INTEGER PRIMARY KEY,
    post_id          INTEGER NOT NULL REFERENCES posts(id),
    measured_at      TEXT NOT NULL DEFAULT (datetime('now')),
    impressions      INTEGER,
    likes            INTEGER,
    reposts          INTEGER,
    replies          INTEGER,
    bookmarks        INTEGER,
    profile_clicks   INTEGER,
    source           TEXT NOT NULL DEFAULT 'manual'
                     CHECK(source IN ('manual','x_api')),
    created_at       TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_raw_materials_account_status ON raw_materials(account_id, status);
CREATE INDEX idx_drafts_account_review ON drafts(account_id, review_status);
CREATE INDEX idx_posts_account ON posts(account_id);
CREATE INDEX idx_post_metrics_post ON post_metrics(post_id);
