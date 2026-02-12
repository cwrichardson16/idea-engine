"""
db.py - Supabase database client.

Wraps the supabase-py client with two focused responsibilities:
  1. Upserting raw Reddit posts (deduplication via ON CONFLICT on reddit_id).
  2. Fetching unevaluated posts that meet filter thresholds.
  3. Persisting structured scores and marking posts as evaluated.

All methods raise on unexpected errors; callers are responsible for catching
and logging if they want to continue past failures.
"""

import logging
from datetime import datetime, timezone
from typing import Dict, List

from supabase import Client, create_client

from app.config import Config
from app.models import PostScore, RedditPost

logger = logging.getLogger(__name__)


class Database:
    def __init__(self, config: Config) -> None:
        self.client: Client = create_client(config.supabase_url, config.supabase_key)
        logger.debug("Supabase client initialised.")

    # ----------------------------------------------------------------- writes

    def upsert_post(self, post: RedditPost) -> bool:
        """
        Insert a post into reddit_posts.

        Uses ON CONFLICT DO NOTHING so a second fetch of the same reddit_id
        is silently dropped.  Returns True if the row was inserted (new post),
        False if it already existed (duplicate).
        """
        data: Dict = post.model_dump(mode="json")
        data["fetched_at"] = datetime.now(timezone.utc).isoformat()
        data["is_evaluated"] = False

        result = (
            self.client.table("reddit_posts")
            .upsert(data, on_conflict="reddit_id", ignore_duplicates=True)
            .execute()
        )
        is_new = len(result.data) > 0
        logger.debug(
            "upsert_post %s -> %s", post.reddit_id, "inserted" if is_new else "duplicate"
        )
        return is_new

    def save_score(self, reddit_id: str, score: PostScore, raw_response: str) -> None:
        """
        Write a PostScore to post_scores and flip is_evaluated on the parent row.

        Both writes are done sequentially; if the score insert fails the post
        remains unevaluated so a future run can retry it.
        """
        score_data: Dict = score.model_dump(mode="json")
        score_data["reddit_id"] = reddit_id
        score_data["raw_response"] = raw_response
        score_data["evaluated_at"] = datetime.now(timezone.utc).isoformat()

        self.client.table("post_scores").insert(score_data).execute()

        self.client.table("reddit_posts").update({"is_evaluated": True}).eq(
            "reddit_id", reddit_id
        ).execute()

        logger.debug("Saved score for %s (recommendation=%s)", reddit_id, score.recommendation)

    # ------------------------------------------------------------------ reads

    def get_unevaluated_posts(
        self, min_score: int, min_comments: int, keywords: List[str]
    ) -> List[Dict]:
        """
        Return reddit_posts rows that:
          - have not yet been evaluated (is_evaluated = false)
          - meet the min_score and min_comments thresholds
          - contain at least one keyword in title or body (if keywords are set)

        Keyword filtering is done client-side after the DB query because
        PostgREST full-text search requires index setup that users may not have.
        The result set is expected to be small enough that this is fine.
        """
        result = (
            self.client.table("reddit_posts")
            .select("*")
            .eq("is_evaluated", False)
            .gte("score", min_score)
            .gte("num_comments", min_comments)
            .execute()
        )

        posts = result.data

        if keywords:
            lower_kws = [kw.lower() for kw in keywords]
            posts = [
                p
                for p in posts
                if any(
                    kw in (p.get("title") or "").lower()
                    or kw in (p.get("body") or "").lower()
                    for kw in lower_kws
                )
            ]

        logger.debug(
            "get_unevaluated_posts -> %d posts (after keyword filter)", len(posts)
        )
        return posts
