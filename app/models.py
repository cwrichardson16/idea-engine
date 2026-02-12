"""
models.py - Pydantic data models shared across ingestion and evaluation.

RedditPost   -> represents a raw post fetched from Reddit.
PostScore    -> represents the structured LLM analysis of a post.
Recommendation -> enum constraining the LLM's verdict to known values.
"""

from datetime import datetime
from enum import Enum
from typing import List

from pydantic import BaseModel, Field


class RedditPost(BaseModel):
    """A raw Reddit submission as stored in the reddit_posts table."""

    reddit_id: str
    subreddit: str
    title: str
    body: str
    url: str
    author: str
    score: int
    num_comments: int
    created_utc: datetime


class Recommendation(str, Enum):
    BUILD = "build"
    SKIP = "skip"
    INVESTIGATE = "investigate"


class PostScore(BaseModel):
    """
    Structured LLM analysis of a Reddit post.

    All integer scores are in the range [1, 10].
    This model is used both for:
      1. Validating the raw JSON returned by OpenAI.
      2. Serialising the result before writing to post_scores.
    """

    opportunity_score: int = Field(ge=1, le=10, description="Overall opportunity quality")
    market_size_score: int = Field(ge=1, le=10, description="Addressable market potential")
    feasibility_score: int = Field(ge=1, le=10, description="Ease of building / executing")
    pain_point_clarity: int = Field(ge=1, le=10, description="How clearly the pain is articulated")
    summary: str = Field(description="2-3 sentence objective analysis")
    key_insights: List[str] = Field(description="3-5 actionable insights")
    tags: List[str] = Field(description="2-4 category tags")
    recommendation: Recommendation
