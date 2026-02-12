"""
run.py - CLI entrypoint for the Reddit Idea Engine.

Usage:
    python -m app.run                  # ingest + evaluate (default)
    python -m app.run --mode ingest    # fetch new posts only
    python -m app.run --mode evaluate  # score existing posts only
    python -m app.run --verbose        # enable DEBUG logging

The module is also executable directly:
    python app/run.py --mode all
"""

import argparse
import logging
import sys

from app.config import Config
from app.db import Database
from app.evaluation import evaluate_posts
from app.ingestion import fetch_and_store_posts


# ---------------------------------------------------------------------------
# Logging setup
# ---------------------------------------------------------------------------


def _setup_logging(verbose: bool = False) -> None:
    level = logging.DEBUG if verbose else logging.INFO
    logging.basicConfig(
        level=level,
        format="%(asctime)s [%(levelname)-8s] %(name)s: %(message)s",
        datefmt="%Y-%m-%d %H:%M:%S",
        handlers=[logging.StreamHandler(sys.stdout)],
    )
    # Quieten noisy third-party loggers unless we're in verbose mode.
    if not verbose:
        logging.getLogger("prawcore").setLevel(logging.WARNING)
        logging.getLogger("urllib3").setLevel(logging.WARNING)
        logging.getLogger("httpx").setLevel(logging.WARNING)


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="python -m app.run",
        description="Reddit Idea Engine — discover startup opportunities from Reddit.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Modes:
  all       (default) Run ingestion then evaluation in sequence.
  ingest    Fetch new posts from Reddit and store them in Supabase.
  evaluate  Score stored posts that have not yet been evaluated.

Environment variables are loaded from .env automatically.
See .env.example for required keys.
        """,
    )
    parser.add_argument(
        "--mode",
        choices=["ingest", "evaluate", "all"],
        default="all",
        help="Which pipeline stage(s) to run (default: all)",
    )
    parser.add_argument(
        "--verbose",
        "-v",
        action="store_true",
        help="Enable DEBUG-level logging",
    )
    return parser


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main() -> None:
    parser = _build_parser()
    args = parser.parse_args()

    _setup_logging(args.verbose)
    logger = logging.getLogger(__name__)

    # Validate config early; missing env vars exit here with a clear message.
    try:
        config = Config()
    except KeyError as exc:
        logger.error("Configuration error: %s", exc)
        sys.exit(1)

    db = Database(config)

    if args.mode in ("ingest", "all"):
        logger.info("=" * 60)
        logger.info("STAGE: Ingestion")
        logger.info("=" * 60)
        try:
            new_posts = fetch_and_store_posts(config, db)
            logger.info("Ingestion complete — %d new posts stored.", new_posts)
        except Exception as exc:
            logger.error("Ingestion failed: %s", exc)
            if args.mode == "ingest":
                sys.exit(1)
            # In 'all' mode continue to evaluation with whatever is already stored.

    if args.mode in ("evaluate", "all"):
        logger.info("=" * 60)
        logger.info("STAGE: Evaluation")
        logger.info("=" * 60)
        try:
            scored = evaluate_posts(config, db)
            logger.info("Evaluation complete — %d posts scored.", scored)
        except Exception as exc:
            logger.error("Evaluation failed: %s", exc)
            sys.exit(1)

    logger.info("Done.")


if __name__ == "__main__":
    main()
