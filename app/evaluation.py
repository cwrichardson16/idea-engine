"""
evaluation.py - OpenAI-powered scoring of Reddit posts.

Pipeline:
  1. Fetch unevaluated posts from the DB that pass filter thresholds.
  2. For each post, build a prompt and call the OpenAI chat API requesting JSON.
  3. Parse and validate the JSON response against PostScore (Pydantic).
  4. On any parse/validation failure, retry ONCE with a correction prompt that
     shows the model its broken output and asks it to fix it.
  5. Persist the PostScore via db.save_score(); skip the post on second failure.

The JSON schema is embedded in the system prompt and also enforced via
response_format={"type": "json_object"} so the model always returns raw JSON.
"""

import json
import logging
from typing import Dict, Tuple

from openai import OpenAI
from pydantic import ValidationError

from app.config import Config
from app.db import Database
from app.models import PostScore

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Prompt templates
# ---------------------------------------------------------------------------

# The scoring schema is described inline so the LLM has the full spec in context.
_SYSTEM_PROMPT = """\
You are a startup analyst evaluating Reddit posts for product and business opportunities.

Analyse the post provided by the user and respond with a single JSON object that
matches **exactly** the following schema (no extra keys, no markdown fences):

{
  "opportunity_score":  <integer 1-10>,   // overall opportunity quality
  "market_size_score":  <integer 1-10>,   // addressable market potential
  "feasibility_score":  <integer 1-10>,   // ease of building / executing
  "pain_point_clarity": <integer 1-10>,   // how clearly the pain is articulated
  "summary":            <string>,         // 2-3 sentence objective analysis
  "key_insights":       [<string>, ...],  // 3-5 actionable insights
  "tags":               [<string>, ...],  // 2-4 category tags, e.g. "SaaS", "B2B"
  "recommendation":     "build" | "skip" | "investigate"
}

Scoring guidelines:
  opportunity_score  — 1 = no opportunity, 10 = exceptional clear gap in the market
  market_size_score  — 1 = hyper-niche, 10 = massive TAM
  feasibility_score  — 1 = technically/legally impossible, 10 = trivial to build
  pain_point_clarity — 1 = vague frustration, 10 = crystal-clear, quantified pain
  recommendation     — "build" if strong signal, "investigate" if promising but unclear,
                       "skip" if not viable

Respond ONLY with valid JSON.  Do not include any explanation outside the JSON object.\
"""

_USER_PROMPT_TEMPLATE = """\
Reddit post from r/{subreddit}

Title: {title}

Body:
{body}

Stats: score={score}, comments={num_comments}\
"""

_CORRECTION_PROMPT_TEMPLATE = """\
Your previous response was not valid JSON or did not match the required schema.

Your previous response:
---
{previous_response}
---

Error: {error}

Please respond ONLY with a valid JSON object matching the schema.
No markdown code fences, no explanation — just the raw JSON object.\
"""


# ---------------------------------------------------------------------------
# Internal helpers
# ---------------------------------------------------------------------------


def _build_user_prompt(post: Dict) -> str:
    return _USER_PROMPT_TEMPLATE.format(
        subreddit=post.get("subreddit", "unknown"),
        title=post.get("title", ""),
        body=post.get("body") or "(no body)",
        score=post.get("score", 0),
        num_comments=post.get("num_comments", 0),
    )


def _strip_markdown_fences(text: str) -> str:
    """
    Strip ``` / ```json fences that some models insert despite being told not to.
    This is a defensive measure; with response_format=json_object it should not
    be needed, but the correction-prompt path re-runs without that guarantee.
    """
    text = text.strip()
    if text.startswith("```"):
        lines = text.splitlines()
        # Drop the opening fence line (```json or ```)
        lines = lines[1:]
        # Drop the closing fence line if present
        if lines and lines[-1].strip() == "```":
            lines = lines[:-1]
        text = "\n".join(lines).strip()
    return text


def _parse_and_validate(raw: str) -> PostScore:
    """Parse raw LLM text into a PostScore, raising on any problem."""
    cleaned = _strip_markdown_fences(raw)
    data = json.loads(cleaned)          # raises json.JSONDecodeError on bad JSON
    return PostScore(**data)            # raises pydantic.ValidationError on bad schema


def _call_openai(client: OpenAI, model: str, messages: list) -> str:
    """Make a single chat completion call and return the content string."""
    response = client.chat.completions.create(
        model=model,
        messages=messages,
        temperature=0.2,
        response_format={"type": "json_object"},
    )
    return response.choices[0].message.content or ""


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------


def evaluate_post(client: OpenAI, config: Config, post: Dict) -> Tuple[PostScore, str]:
    """
    Score a single post using the OpenAI API.

    Attempt 1: standard prompt.
    Attempt 2 (on parse/validation failure): correction prompt that feeds the
               broken output back to the model alongside the error message.

    Returns (PostScore, raw_json_string).
    Raises the second attempt's exception if both attempts fail.
    """
    messages = [
        {"role": "system", "content": _SYSTEM_PROMPT},
        {"role": "user", "content": _build_user_prompt(post)},
    ]

    # --- First attempt ---
    raw = _call_openai(client, config.openai_model, messages)
    try:
        score = _parse_and_validate(raw)
        return score, raw
    except (json.JSONDecodeError, ValidationError, TypeError) as first_err:
        logger.warning(
            "Post %s: first parse failed (%s). Retrying with correction prompt.",
            post.get("reddit_id"),
            first_err,
        )

    # --- Second attempt (correction) ---
    messages.append({"role": "assistant", "content": raw})
    messages.append({
        "role": "user",
        "content": _CORRECTION_PROMPT_TEMPLATE.format(
            previous_response=raw,
            error=str(first_err),
        ),
    })

    raw2 = _call_openai(client, config.openai_model, messages)
    try:
        score = _parse_and_validate(raw2)
        logger.info("Post %s: correction prompt succeeded.", post.get("reddit_id"))
        return score, raw2
    except (json.JSONDecodeError, ValidationError, TypeError) as second_err:
        logger.error(
            "Post %s: correction prompt also failed: %s",
            post.get("reddit_id"),
            second_err,
        )
        raise second_err


def evaluate_posts(config: Config, db: Database) -> int:
    """
    Evaluate all qualifying unevaluated posts in the database.

    A post qualifies if it passes the min_score / min_comments / keyword
    thresholds defined in config.  Each post is scored independently; a
    failure on one post is logged and skipped so others are not affected.

    Returns the number of posts successfully evaluated and stored.
    """
    client = OpenAI(api_key=config.openai_api_key)

    posts = db.get_unevaluated_posts(
        min_score=config.min_score,
        min_comments=config.min_comments,
        keywords=config.keywords,
    )

    logger.info("Found %d posts to evaluate.", len(posts))
    evaluated = 0

    for post in posts:
        reddit_id = post.get("reddit_id", "unknown")
        title_preview = (post.get("title") or "")[:70]

        try:
            logger.info("Evaluating %s  '%s'", reddit_id, title_preview)
            score, raw = evaluate_post(client, config, post)
            db.save_score(reddit_id, score, raw)
            logger.info(
                "  -> recommendation=%-11s  opportunity=%d  feasibility=%d",
                score.recommendation.value,
                score.opportunity_score,
                score.feasibility_score,
            )
            evaluated += 1
        except Exception as exc:
            logger.error("Failed to evaluate post %s: %s", reddit_id, exc)
            continue

    logger.info("Evaluation complete: %d / %d posts scored.", evaluated, len(posts))
    return evaluated
