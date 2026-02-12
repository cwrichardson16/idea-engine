"""
config.py - Centralised configuration loaded from environment variables.

All required keys raise KeyError immediately on startup if missing,
so misconfiguration is caught before any network calls are made.
"""

import os
from dataclasses import dataclass, field
from typing import List

from dotenv import load_dotenv

# Load .env into os.environ before any field defaults are evaluated.
load_dotenv()


def _require(key: str) -> str:
    """Return the value of an env var or raise with a helpful message."""
    value = os.getenv(key)
    if not value:
        raise KeyError(f"Required environment variable '{key}' is not set.")
    return value


def _csv_list(key: str, default: str = "") -> List[str]:
    """Parse a comma-separated env var into a stripped, non-empty list."""
    raw = os.getenv(key, default)
    return [item.strip() for item in raw.split(",") if item.strip()]


@dataclass
class Config:
    # ------------------------------------------------------------------ Reddit
    reddit_client_id: str = field(default_factory=lambda: _require("REDDIT_CLIENT_ID"))
    reddit_client_secret: str = field(default_factory=lambda: _require("REDDIT_CLIENT_SECRET"))
    reddit_user_agent: str = field(
        default_factory=lambda: os.getenv("REDDIT_USER_AGENT", "reddit-idea-engine/1.0")
    )

    # -------------------------------------------------------- Subreddit targets
    subreddits: List[str] = field(
        default_factory=lambda: _csv_list("SUBREDDITS", "SaaS,startups,entrepreneur,smallbusiness")
    )
    posts_per_subreddit: int = field(
        default_factory=lambda: int(os.getenv("POSTS_PER_SUBREDDIT", "25"))
    )

    # --------------------------------------------------------- Filter thresholds
    min_score: int = field(default_factory=lambda: int(os.getenv("MIN_SCORE", "10")))
    min_comments: int = field(default_factory=lambda: int(os.getenv("MIN_COMMENTS", "5")))
    # Empty list means no keyword filter is applied.
    keywords: List[str] = field(default_factory=lambda: _csv_list("KEYWORDS", ""))

    # ------------------------------------------------------------------ OpenAI
    openai_api_key: str = field(default_factory=lambda: _require("OPENAI_API_KEY"))
    openai_model: str = field(
        default_factory=lambda: os.getenv("OPENAI_MODEL", "gpt-4o-mini")
    )

    # ---------------------------------------------------------------- Supabase
    supabase_url: str = field(default_factory=lambda: _require("SUPABASE_URL"))
    supabase_key: str = field(default_factory=lambda: _require("SUPABASE_KEY"))
