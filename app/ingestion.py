"""
ingestion.py - Pull posts from Reddit and persist them to Supabase.

Responsibilities:
  - Build an authenticated PRAW Reddit client from Config.
  - Iterate over each configured subreddit and fetch `posts_per_subreddit`
    hot posts.
  - Convert each submission to a RedditPost and call db.upsert_post().
  - Skip and log individual post failures so one bad post doesn't abort the
    entire subreddit fetch.
  - Skip and log entire subreddit failures so one bad subreddit doesn't abort
    the entire run.

Returns the total count of newly inserted posts so the caller can report it.
"""

import logging
from datetime import datetime, timezone

import praw

from app.config import Config
from app.db import Database
from app.models import RedditPost

logger = logging.getLogger(__name__)


def _build_reddit_client(config: Config) -> praw.Reddit:
    """Construct a read-only PRAW client. PRAW handles rate-limiting internally."""
    return praw.Reddit(
        client_id=config.reddit_client_id,
        client_secret=config.reddit_client_secret,
        user_agent=config.reddit_user_agent,
    )


def _submission_to_post(submission: praw.models.Submission, subreddit_name: str) -> RedditPost:
    """Map a PRAW Submission object to our RedditPost model."""
    return RedditPost(
        reddit_id=submission.id,
        subreddit=subreddit_name,
        title=submission.title,
        body=submission.selftext or "",
        url=submission.url,
        author=str(submission.author) if submission.author else "[deleted]",
        score=submission.score,
        num_comments=submission.num_comments,
        created_utc=datetime.fromtimestamp(submission.created_utc, tz=timezone.utc),
    )


def fetch_and_store_posts(config: Config, db: Database) -> int:
    """
    Main ingestion entry point.

    Iterates over config.subreddits, fetches the top `posts_per_subreddit`
    hot posts from each, and upserts them into the database.

    Returns the total number of *new* posts stored (duplicates are not counted).
    """
    reddit = _build_reddit_client(config)
    total_new = 0

    for subreddit_name in config.subreddits:
        subreddit_name = subreddit_name.strip()
        logger.info("Fetching r/%s (limit=%d)", subreddit_name, config.posts_per_subreddit)

        try:
            subreddit = reddit.subreddit(subreddit_name)
            submissions = list(subreddit.hot(limit=config.posts_per_subreddit))
        except Exception as exc:
            logger.error("Failed to fetch r/%s: %s", subreddit_name, exc)
            continue

        new_this_sub = 0
        for submission in submissions:
            try:
                post = _submission_to_post(submission, subreddit_name)
                is_new = db.upsert_post(post)
                if is_new:
                    new_this_sub += 1
                    logger.debug("Stored  %s  %s", post.reddit_id, post.title[:80])
                else:
                    logger.debug("Skip dup %s", post.reddit_id)
            except Exception as exc:
                logger.warning(
                    "Failed to store post %s from r/%s: %s",
                    submission.id,
                    subreddit_name,
                    exc,
                )
                continue

        logger.info(
            "r/%s done: %d new / %d fetched",
            subreddit_name,
            new_this_sub,
            len(submissions),
        )
        total_new += new_this_sub

    return total_new
