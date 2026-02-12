-- =============================================================================
-- Migration 001 — Initial schema for reddit-idea-engine
--
-- Apply this in the Supabase SQL Editor (Dashboard → SQL Editor → New query)
-- or via psql against your Supabase project's connection string.
--
-- Tables:
--   reddit_posts  — raw posts fetched from Reddit (one row per reddit_id)
--   post_scores   — structured LLM analysis (one row per evaluated reddit_id)
-- =============================================================================

-- ---------------------------------------------------------------------------
-- reddit_posts
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS reddit_posts (
    id            BIGSERIAL     PRIMARY KEY,

    -- The Reddit submission short-id (e.g. "abc123"). Used as the dedup key.
    reddit_id     TEXT          NOT NULL UNIQUE,

    subreddit     TEXT          NOT NULL,
    title         TEXT          NOT NULL,
    body          TEXT          NOT NULL DEFAULT '',
    url           TEXT          NOT NULL DEFAULT '',
    author        TEXT          NOT NULL DEFAULT '',

    -- Engagement metrics at time of fetch.
    score         INTEGER       NOT NULL DEFAULT 0,
    num_comments  INTEGER       NOT NULL DEFAULT 0,

    -- When the post was originally published on Reddit.
    created_utc   TIMESTAMPTZ   NOT NULL,

    -- When our engine fetched and stored it.
    fetched_at    TIMESTAMPTZ   NOT NULL DEFAULT NOW(),

    -- Flipped to TRUE after a successful evaluation is written to post_scores.
    is_evaluated  BOOLEAN       NOT NULL DEFAULT FALSE
);

-- Speed up the most common query patterns.
CREATE INDEX IF NOT EXISTS idx_reddit_posts_reddit_id
    ON reddit_posts (reddit_id);

CREATE INDEX IF NOT EXISTS idx_reddit_posts_is_evaluated
    ON reddit_posts (is_evaluated)
    WHERE is_evaluated = FALSE;   -- partial index; only unevaluated rows matter

CREATE INDEX IF NOT EXISTS idx_reddit_posts_subreddit
    ON reddit_posts (subreddit);

-- ---------------------------------------------------------------------------
-- post_scores
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS post_scores (
    id                  BIGSERIAL     PRIMARY KEY,

    -- Foreign key back to the original post.
    reddit_id           TEXT          NOT NULL
                            REFERENCES reddit_posts (reddit_id)
                            ON DELETE CASCADE,

    -- LLM-assigned scores (all 1-10).
    opportunity_score   SMALLINT      NOT NULL CHECK (opportunity_score  BETWEEN 1 AND 10),
    market_size_score   SMALLINT      NOT NULL CHECK (market_size_score  BETWEEN 1 AND 10),
    feasibility_score   SMALLINT      NOT NULL CHECK (feasibility_score  BETWEEN 1 AND 10),
    pain_point_clarity  SMALLINT      NOT NULL CHECK (pain_point_clarity BETWEEN 1 AND 10),

    -- Qualitative outputs.
    summary             TEXT          NOT NULL,
    key_insights        JSONB         NOT NULL DEFAULT '[]'::JSONB,
    tags                JSONB         NOT NULL DEFAULT '[]'::JSONB,

    -- "build" | "skip" | "investigate"
    recommendation      TEXT          NOT NULL
                            CHECK (recommendation IN ('build', 'skip', 'investigate')),

    -- Full raw JSON string from the LLM (useful for debugging / auditing).
    raw_response        TEXT,

    evaluated_at        TIMESTAMPTZ   NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_post_scores_reddit_id
    ON post_scores (reddit_id);

-- Useful for surfacing the highest-opportunity posts quickly.
CREATE INDEX IF NOT EXISTS idx_post_scores_opportunity
    ON post_scores (opportunity_score DESC);

CREATE INDEX IF NOT EXISTS idx_post_scores_recommendation
    ON post_scores (recommendation);

-- ---------------------------------------------------------------------------
-- Convenience view: join scores back to their source posts
-- ---------------------------------------------------------------------------

CREATE OR REPLACE VIEW scored_posts AS
SELECT
    p.reddit_id,
    p.subreddit,
    p.title,
    p.score         AS reddit_score,
    p.num_comments,
    p.url,
    p.fetched_at,
    s.opportunity_score,
    s.market_size_score,
    s.feasibility_score,
    s.pain_point_clarity,
    s.recommendation,
    s.summary,
    s.key_insights,
    s.tags,
    s.evaluated_at
FROM reddit_posts p
JOIN post_scores  s ON s.reddit_id = p.reddit_id
ORDER BY s.opportunity_score DESC, s.evaluated_at DESC;
