# Security review: AnkiWeb add-on 124672614 ("Anki MCP Server - AI Assistant Integration")

Date: 2026-11-28. Read-only review. Nothing was installed or run. Source and packages were downloaded only to read them, into `/private/tmp/claude-501/-Users-connerrichardson-Projects-idea-engine/2450836a-499e-482d-919d-4a3f50256f74/scratchpad/anki-mcp-review/`.

File:line references below point to the copy AnkiWeb actually serves (v0.41.1, the `awx/` folder in that scratch directory). The same files are in the GitHub tag `v0.41.1` at `anki_mcp_server/<file>`.

## Verdict (plain English)

Safe to install for local testing, with the precautions listed below. This is a real, actively maintained open-source project. The code is readable, not obfuscated, and has no telemetry. The copy AnkiWeb hands out is byte-for-byte the same as the maintainer's public GitHub release. The server only listens on your own machine by default, and a normal web page in your browser cannot drive it.

The real risks are about what an AI connected to it can do, not about hidden malware:
1. Any program running on your computer can use it with no password, because the API key is blank by default.
2. Its default tool set can delete notes, reschedule or reset cards, change note types and templates, read image/audio/video files from anywhere on disk into Anki, fetch URLs, and run an AnkiWeb sync, including a forced full upload or download that can overwrite your other devices.
3. The first time it starts, it downloads one compiled package (pydantic_core) from PyPI.

## Who, license, activity

- Maintainer: Anatoly Tarnavsky (GitHub `anatoly314`, account since 2012), under the `ankimcp` GitHub organization. He wrote 248 of the ~255 commits; 6 other people have 1 commit each. That makes it effectively a one-person project. CONFIRMED (GitHub API `repos/ankimcp/anki-mcp-server-addon/contributors`).
- Source: https://github.com/ankimcp/anki-mcp-server-addon. CONFIRMED. Several third-party repos link AnkiWeb code 124672614 to this repo and to `anatoly314`, e.g. github.com/jalliet/flashcards README.md and CuriousN006/codex-anki-skills README.md. The AnkiWeb page itself is rendered by JavaScript and did not load as text, so I could not read its own GitHub link. UNVERIFIED that the page links there. The package match below settles the question anyway.
- License: AGPL-3.0-or-later. CONFIRMED (repo `LICENSE` lines 1-4). GitHub shows "NOASSERTION" only because the file is a short notice and not the full license text.
- Activity: created 2025-12-05. Latest release v0.41.2 on 2026-11-17. Last push to main on 2026-11-28 (today). 111 stars, 15 forks, 1 open issue. CONFIRMED (GitHub API).
- Security history: one published advisory, GHSA-j9xx-59ph-wmr6 (high, 2026-06-18): "DNS rebinding reaches unauthenticated local AnkiMCP HTTP tools". It is fixed in the current code, which turns DNS-rebinding protection on (see below). CONFIRMED (GitHub API `security-advisories`).
- AnkiWeb "last updated" date: UNVERIFIED, because the page needs JavaScript.

## Does the AnkiWeb download match GitHub?

- I downloaded the AnkiWeb package (`/shared/download/124672614`). For Anki 25.02-25.09 it redirects to version 0.41.1. Its SHA-256 is `4816108718c3749b8df4749ed166fe8825d0a0e0c6256abe99fa23c7d23aca58`, identical to the GitHub release asset `v0.41.1/anki_mcp_server.ankiaddon`. CONFIRMED (both files hashed with `shasum -a 256`).
- Every non-vendored file in that package is identical to the GitHub source tag `v0.41.1` (`diff -rq` showed no differences). CONFIRMED.
- The `vendor/` folder holds third-party libraries (mcp 1.30.0, uvicorn 0.54.0, starlette 1.10.0, websockets 17.4, httpx, pydantic 2.13.5, pyjwt, and others). GitHub Actions builds them with `package.sh` (`pip download` from PyPI) during the release job (`.github/workflows/release.yml` around line 143). I did not check each vendored file against PyPI. UNVERIFIED, low concern.
- AnkiWeb may serve a different upload to other Anki versions (for example 25.10+ or 26.x). I only checked the 25.02-25.09 build. UNVERIFIED for other versions.
- Bundled binary: only `vendor/shared/websockets/speedups.cpython-313-darwin.so`, an optional C speed-up from the official websockets wheel. CONFIRMED (find for .so/.dylib/.pyd/.dll/.exe).

## What it runs

- An HTTP MCP server (FastMCP streamable HTTP on uvicorn) inside Anki, on by default. CONFIRMED (`config.json:2` `"http_enabled": true`).
- Port 3141, bound to 127.0.0.1 (localhost only) by default. CONFIRMED (`config.json:3-4`, `mcp_server.py:787`). If someone changes `http_host` to 0.0.0.0, the server becomes reachable from your network.
- Auth: none by default. `http_api_key` is empty, and the Bearer-key check is only added when a key is set. CONFIRMED (`config.json:6`, `mcp_server.py:763`). When set, the key is compared in constant time (`http_auth.py:91`). Without a key, any local process (another app, a script, a malicious npm package you run) can call every enabled tool.
- CORS: off by default (`config.json:9` `"cors_origins": []`; the middleware is only added if that list is set, `mcp_server.py:767`). If you ever set it, it uses `allow_credentials=True` (`mcp_server.py:776`), so only add origins you trust.
- DNS-rebinding and cross-origin protection: on. The Host header must be 127.0.0.1, localhost or [::1], and any Origin header present must be a loopback http origin. CONFIRMED (`transport_security_config.py:31-36, 63`). Enforcement is in the vendored MCP SDK: wrong Host gets 421, wrong Origin gets 403, and a POST without `Content-Type: application/json` gets 400 (`vendor/shared/mcp/server/transport_security.py:114-132`). Because JSON POSTs force a CORS preflight, and no CORS headers are sent, an ordinary web page (even one served from localhost) cannot call the tools from your browser. CONFIRMED by reading the code; not tested live.
- Optional remote "tunnel": a WebSocket to `wss://tunnel.ankimcp.ai` (the maintainer's service, `config.json:15`) that lets cloud AI clients reach your Anki. It only starts when you click "Connect Tunnel" in its settings dialog and log in with a device code. CONFIRMED (the only `connect_tunnel()` callers are UI handlers, `tunnel/ui/settings_section.py:320` and nearby). With the tunnel on, all tool traffic goes through a third-party relay. Leave it off. Tokens are saved to `user_files/credentials.json` with 0600 permissions (`credentials.py:75, 155`).

## What operations it exposes (66 tool modules)

Enabled by default:
- Read: find_notes, notes_info, list_decks, cards_stats, get_due_cards, get_card_memory_state, deck and model info, tag lists, media file names, FSRS params, plus collection stats resources that read review history. CONFIRMED (`primitives/essential/tools/`, `primitives/essential/resources/stats_resources.py`).
- Write: add_note(s), update_note_fields, update_notes, create_deck, create_model, update_model_templates and styling, model field add/rename/reposition, tag add/remove/replace/batch/clear_unused, store_media_file, delete_media_file. CONFIRMED.
- Delete notes: `delete_notes` is on by default. It needs `confirmDeletion=true` (the AI can simply pass that) and is capped at 100 notes per call (`max_notes_per_batch`, `config.json:14`). CONFIRMED (`delete_notes_tool.py:14-16`).
- Scheduling: rate_card, card_forget (resets progress), card_set_due_date, card_reposition, suspend/bury/flag/change deck, set_fsrs_params, optimize_fsrs_params. CONFIRMED.
- Filtered decks: create/rebuild/empty/delete (filtered decks only). There is no tool to delete a normal deck. CONFIRMED (no such tool module exists).
- Sync: the `sync` tool starts an AnkiWeb sync and can settle a full-sync conflict with `resolve="upload"` or `"download"`, which overwrites one side. CONFIRMED (`sync_tool.py:87, 100`). This is the biggest risk to real data on your other devices.
- Files and network: `store_media_file` can read any local file whose name looks like image/audio/video from anywhere on disk (no folder limit unless `media_import_dir` is set), and can download URLs. It blocks private IPs, but its own comments admit redirects and DNS rebinding can get around that. CONFIRMED (`store_media_file_tool.py:60, 87`; `media_validators.py:233-241`).
- GUI control: open the browser, add/edit windows, show question/answer, undo. CONFIRMED (`primitives/gui/tools/`).

Hidden unless you opt in: `change_note_type` and `model_field_remove` (marked destructive, `change_note_type_tool.py:145`, `model_field_remove_tool.py:32`), and the GUI review tools `gui_answer_card` and `gui_deck_review` (`opt_in=True`). CONFIRMED.

There is no arbitrary-code or arbitrary-SQL tool. The only SQL is fixed, parameterized internal queries (e.g. `cards_stats_tool.py` around line 166). There are no subprocess, eval or exec calls in the add-on's own code. CONFIRMED (grep across the code, vendor folder excluded).

## Outbound network calls

- Runtime package download: at startup it fetches `pydantic_core` (pinned to the exact version the bundled pydantic needs) from `https://pypi.org`, plus `rpds-py` 0.30.0 only if Anki lacks it. It checks the version number but not a file hash; it relies on HTTPS to PyPI. CONFIRMED (`dependency_loader.py:1261, 1500, 1074`). Low risk, but it does mean compiled code is fetched at runtime.
- `store_media_file` with a URL: only when a tool call asks for it. CONFIRMED.
- Tunnel to tunnel.ankimcp.ai: only after you opt in. CONFIRMED.
- AnkiWeb sync: only through the `sync` tool. CONFIRMED.
- No telemetry, analytics, crash reporting or self-update code found. The ankimcp.ai links are static labels and an icon URL in the server metadata. CONFIRMED (grep for telemetry/analytics/sentry/posthog/urlopen/httpx).
- Obfuscation: none. No long encoded blobs; the code is well commented. CONFIRMED.

## Precautions for local testing

1. Back up first: in Anki, File > Export > "Anki Collection Package" (.colpkg) with media, or close Anki and copy the profile folder.
2. Use a separate Anki profile (File > Switch Profile > Add) that holds only a test deck. Do not log that profile into AnkiWeb, so the `sync` tool cannot touch your real collection or devices. The add-on is installed for all profiles, but its tools only act on whichever profile is open.
3. In the add-on config (Tools > Add-ons > Config): keep `http_host` at `127.0.0.1`; set a long random `http_api_key` and give it to your MCP client as a Bearer token; add `"sync"`, `"delete_notes"`, `"store_media_file"` and `"card_forget"` to `disabled_tools` unless you are testing them; leave `cors_origins`, `http_allowed_hosts`/`origins`, `enabled_destructive_tools` and `enabled_opt_in_tools` empty.
4. Never click "Connect Tunnel".
5. Turn it off when not testing: disable the add-on (Tools > Add-ons > toggle) or close Anki. While Anki is open on any profile, the server is listening.
6. Expect one download from pypi.org on first start (pydantic_core). That is normal.
7. Pin to the version reviewed here (0.41.1), or review again before updating, since this is a one-person project that ships often.
