# reddit-idea-engine

Pulls Reddit posts from configurable subreddits, stores them in Supabase, and
uses OpenAI to score each post as a potential product or business opportunity.

---

## How it works

```
Reddit (PRAW)
    │
    ▼ fetch hot posts
reddit_posts table (Supabase)
    │
    ▼ filter: min_score, min_comments, keywords
OpenAI (gpt-4o-mini)
    │
    ▼ structured JSON score
post_scores table (Supabase)
```

1. **Ingestion** (`app/ingestion.py`) — fetches `hot` posts from each configured
   subreddit via PRAW and upserts them into `reddit_posts`.  A post is only
   inserted once; duplicate `reddit_id` values are silently ignored.

2. **Evaluation** (`app/evaluation.py`) — queries for unevaluated posts that
   meet the score/comment/keyword thresholds, sends each to OpenAI with a
   structured scoring prompt, validates the JSON response with Pydantic, and
   writes results to `post_scores`.  If the model returns malformed JSON, one
   automatic retry is performed with a correction prompt before the post is
   skipped.

---

## Project structure

```
reddit-idea-engine/
├── app/
│   ├── __init__.py       package marker
│   ├── config.py         loads and validates all env vars via python-dotenv
│   ├── models.py         Pydantic models: RedditPost, PostScore, Recommendation
│   ├── db.py             Supabase client wrapper (upsert, fetch, save score)
│   ├── ingestion.py      PRAW fetch logic → db.upsert_post()
│   ├── evaluation.py     OpenAI scoring logic → db.save_score()
│   └── run.py            CLI entrypoint (argparse)
├── migrations/
│   └── 001_initial.sql   CREATE TABLE statements for Supabase
├── .env.example          template for required environment variables
├── requirements.txt
└── README.md
```

---

## Prerequisites

- Python 3.9 or later
- A [Reddit app](https://www.reddit.com/prefs/apps) (type: **script**)
- An [OpenAI API key](https://platform.openai.com/api-keys)
- A [Supabase](https://supabase.com) project

---

## Setup

### 1. Clone and create a virtual environment

```bash
git clone <repo-url> reddit-idea-engine
cd reddit-idea-engine

python -m venv .venv
source .venv/bin/activate        # Windows: .venv\Scripts\activate
```

### 2. Install dependencies

```bash
pip install -r requirements.txt
```

### 3. Configure environment variables

```bash
cp .env.example .env
```

Edit `.env` and fill in all required values:

| Variable | Description |
|---|---|
| `REDDIT_CLIENT_ID` | From reddit.com/prefs/apps |
| `REDDIT_CLIENT_SECRET` | From reddit.com/prefs/apps |
| `REDDIT_USER_AGENT` | Any descriptive string, e.g. `reddit-idea-engine/1.0 by u/you` |
| `SUBREDDITS` | Comma-separated list, e.g. `SaaS,startups,entrepreneur` |
| `POSTS_PER_SUBREDDIT` | How many hot posts to fetch per subreddit (default: `25`) |
| `MIN_SCORE` | Minimum Reddit score for a post to be evaluated (default: `10`) |
| `MIN_COMMENTS` | Minimum comment count for a post to be evaluated (default: `5`) |
| `KEYWORDS` | Optional comma-separated keyword filter; leave empty to disable |
| `OPENAI_API_KEY` | Your OpenAI secret key |
| `OPENAI_MODEL` | Model name (default: `gpt-4o-mini`) |
| `SUPABASE_URL` | Your project URL from Supabase Dashboard → Settings → API |
| `SUPABASE_KEY` | Your `service_role` or `anon` key (service_role recommended for writes) |

### 4. Apply the database migration

Open the Supabase Dashboard → **SQL Editor** → **New query**, paste the
contents of `migrations/001_initial.sql`, and click **Run**.

Alternatively, apply via `psql` using your project's connection string
(found in Dashboard → Settings → Database → Connection string):

```bash
psql "postgresql://postgres:<password>@<host>:5432/postgres" \
     -f migrations/001_initial.sql
```

### 5. Run the engine

```bash
# Run ingestion + evaluation (default)
python -m app.run

# Run ingestion only
python -m app.run --mode ingest

# Run evaluation only (uses posts already stored from a previous ingest)
python -m app.run --mode evaluate

# Enable verbose / DEBUG logging
python -m app.run --verbose
```

---

## Database schema

### `reddit_posts`

| Column | Type | Notes |
|---|---|---|
| `id` | bigserial | Primary key |
| `reddit_id` | text | Unique Reddit submission ID — dedup key |
| `subreddit` | text | |
| `title` | text | |
| `body` | text | selftext, empty for link posts |
| `url` | text | |
| `author` | text | |
| `score` | integer | Upvotes at fetch time |
| `num_comments` | integer | Comment count at fetch time |
| `created_utc` | timestamptz | Original post time |
| `fetched_at` | timestamptz | When we stored it |
| `is_evaluated` | boolean | Flipped to `true` after scoring |

### `post_scores`

| Column | Type | Notes |
|---|---|---|
| `id` | bigserial | Primary key |
| `reddit_id` | text | FK → `reddit_posts.reddit_id` |
| `opportunity_score` | smallint | 1–10 overall opportunity |
| `market_size_score` | smallint | 1–10 addressable market |
| `feasibility_score` | smallint | 1–10 ease of execution |
| `pain_point_clarity` | smallint | 1–10 clarity of the problem |
| `summary` | text | 2–3 sentence analysis |
| `key_insights` | jsonb | Array of insight strings |
| `tags` | jsonb | Array of category tag strings |
| `recommendation` | text | `"build"`, `"skip"`, or `"investigate"` |
| `raw_response` | text | Full raw JSON from the LLM (for auditing) |
| `evaluated_at` | timestamptz | |

A `scored_posts` view joins both tables ordered by `opportunity_score DESC`.

---

## OpenAI scoring schema

Each post is sent to the model with a system prompt that enforces the
following JSON structure:

```json
{
  "opportunity_score":  8,
  "market_size_score":  7,
  "feasibility_score":  6,
  "pain_point_clarity": 9,
  "summary": "Users are struggling to X because Y...",
  "key_insights": [
    "Pain is recurring and unresolved by existing tools",
    "Community size suggests a viable niche"
  ],
  "tags": ["SaaS", "B2B", "automation"],
  "recommendation": "build"
}
```

If the response fails JSON parsing or Pydantic validation, a correction
prompt is automatically sent once before the post is skipped for that run.

---

## Extending the project

- **Change subreddits or thresholds** — edit `.env`; no code changes needed.
- **Change the scoring criteria** — edit `_SYSTEM_PROMPT` in `app/evaluation.py`.
- **Add more fields to PostScore** — update `app/models.py`, add the columns
  to the migration (or write a new migration file), and update the system prompt.
- **Schedule recurring runs** — add a cron job or GitHub Actions workflow that
  calls `python -m app.run`.
