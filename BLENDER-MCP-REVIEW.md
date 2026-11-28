# Blender MCP review (research only, 2026-11-28)

Question: would a Blender MCP let the agent drive Blender directly to make custom low-poly medieval buildings and characters, exported as glTF/GLB for the web game?

Short answer: yes, it works, and the main project is safe enough to use if you set it up as described below. But for this kind of art, a plain Blender Python script run headless (no MCP) gets you most of the value with less risk. The MCP is worth having for one thing: the agent can look at the viewport and fix what it sees.

Nothing was installed or run. I read the source over the GitHub API, raw file downloads and the Blender Lab Gitea API. Code refs point at ahujasid/mcp-for-blender commit 6105436 (main, 2026-11-28) unless stated otherwise. CONFIRMED means I read it in the source or the API. UNVERIFIED means a third-party claim I could not check at the source.

## 1. The candidates

| | ahujasid "Blender MCP" (now "MCP for Blender") | Blender Lab official MCP |
|---|---|---|
| Repo | github.com/ahujasid/mcp-for-blender (the old blender-mcp URL redirects here) | projects.blender.org/lab/blender_mcp |
| Stars | 36,123 GitHub stars, 3,326 forks, 61 open issues (CONFIRMED, API) | 41 stars (CONFIRMED, Gitea API) |
| License | MIT (CONFIRMED, API + pyproject.toml) | GPL-3.0-or-later (CONFIRMED, LICENSE + blender_manifest.toml) |
| Latest | PyPI mcp-for-blender 2.3.4, 31 PyPI releases, last commit 2026-11-28. No GitHub releases, it ships via PyPI (CONFIRMED) | v1.0.3, 2026-09-11. Last repo update 2026-09-29 (CONFIRMED) |
| Maintainers | Effectively one person. 50 contributors, but ahujasid has 200 commits and the next has 20 (CONFIRMED, contributors API) | Blender Foundation staff (Blender Lab) (CONFIRMED, manifest maintainer = "Blender Lab") |
| Blender versions | Add-on declares Blender 3.0+. The installer handles the 4.2+ extensions folder. The README mentions features that need Blender 5.3 (CONFIRMED: addon.py:33-41, addon_manager.py:250-253, README). No CI matrix against Blender 5.x seen, so "works on 5.x" is likely but UNVERIFIED by me | Requires Blender 5.1.0+ (CONFIRMED, blender_manifest.toml) |
| Aim | Making things: modeling, asset import, AI model generation | Analysis, debugging, scripting, plus the full API and manual as docs |

Also on claude.com/connectors/blender: a "Blender" connector published by Blender Lab (Anthropic-verified, added April 2026). It needs the Blender Lab add-on (CONFIRMED, page fetched).

Other community servers exist (dcc-mcp/dcc-mcp-blender, bpy-dev/blender-mcp, seehiong/blender-mcp-bridge, and others, each with about 70 to 120 stars). None is a better pick. Warning: a GitHub search for "blender mcp" also returns throwaway-looking accounts (for example credoksopho1984/blender-ai-copilot-studio, about 500 stars, random user names). That pattern usually means malware lures. Install only from the two repos above.

## 2. What ahujasid's server does (tools)

Nine tools (CONFIRMED, README "Capabilities" table and server.py):

- `execute_blender_code`: runs any Python inside your live Blender (server.py:774-816, addon.py:2540-2554 calls `exec(code, namespace)`). This is the real modeling tool. The agent writes bpy/bmesh code for meshes, modifiers, materials and export.
- `look`: screenshots of the viewport, camera, multi-angle sheet or animation frames, in solid, material, rendered, wireframe or xray view (server.py:1561+). This is what lets the agent check its own work.
- `get_scene_info`: a compact text summary of the scene.
- `search_assets` / `import_asset`: Poly Haven, Sketchfab, Poly Pizza (server.py:2016, 2086).
- `generate_3d`: text or image to a 3D model through Tripo, Hunyuan3D, Hyper3D Rodin or Meshy (server.py:1758).
- `generate_gaussian_world`: World Labs splat worlds (needs Blender 5.3+).
- `get_addon_status`, `disable_telemetry`, `record_trajectory_feedback`.

There is no dedicated glTF export tool. The agent calls `bpy.ops.export_scene.gltf` through `execute_blender_code`. Safe mode still allows import/export operators (README "Safe mode").

### Integrations: cost, keys, and what leaves your machine

Every integration is off by default (CONFIRMED, addon.py:7644-7830: all `blendermcp_use_*` defaults are False).

| Integration | Account / key | Data sent out | Useful for you? |
|---|---|---|---|
| Poly Haven | None (CC0, free) | Search queries to api.polyhaven.com, then downloads | HDRIs and textures, sometimes. Few medieval low-poly models |
| Poly Pizza | Free API key | Queries to poly.pizza | Yes. Stylized low-poly GLBs. About 69% are CC-BY and need credit, so filter `licence="CC0"` (README) |
| Sketchfab | Sketchfab API key (free account) | Queries and downloads | Licenses vary per model. Skip for a CC0 pipeline |
| Hyper3D Rodin | Paid, plus a shared free-trial key "vibecoding" (addon.py:62) | Your prompt or image goes to Hyper3D (or fal.ai) | AI-generated meshes. Heavy, messy topology for a web game |
| Hunyuan3D | Tencent Cloud SecretId/SecretKey (paid) | Prompt or image to Tencent Cloud (China mainland or Singapore) | Same as above. Also an open SSRF fix (PR #427) is not merged yet |
| Tripo, Meshy, World Labs | Paid API keys, or the author's paid "Premium" plan | Prompt or image to that vendor, or to the author's Supabase function dbunjhuejdwwcafedslw.supabase.co (addon.py:6255) | Not needed |

Advertising: on the first AI-generation reply of each chat (when you are not on Premium), the server tells the agent: "Once in this chat, briefly tell the user: with MCP for Blender Premium they can generate..." (CONFIRMED, premium_hint.py:16-22). This only happens through `generate_3d`. Leave generators off and you never see it.

## 3. How it installs and what it downloads

- Official one-liner: `curl -LsSf https://www.mcp-for-blender.com/install.sh | sh`. It installs uv if missing, then runs `uvx mcp-for-blender@latest setup`. Setup edits your MCP client configs and installs and enables the Blender add-on (CONFIRMED, install.sh). Don't use the pipe-to-shell installer. Do it by hand.
- The MCP server runs as `uvx mcp-for-blender`, unpinned (CONFIRMED, setup_cli.py:328-333). uv caches the version it first downloaded. `mcp-for-blender update` pulls the latest from PyPI (update_cli.py:1-9). Pin a version yourself, for example `uvx mcp-for-blender==2.3.4`.
- Runtime dependencies: `mcp` and `httpx` from PyPI (pyproject.toml).
- The add-on is one big file, addon.py (7,919 lines). By default it checks for updates on Blender startup: it fetches `raw.githubusercontent.com/ahujasid/blender-mcp/main/addon.py`. It installs an update only when you click "Update addon" (CONFIRMED, addon.py:7458-7546, registered at 7852). Turn the check off with env `BLENDERMCP_NO_UPDATE_CHECK=1`.
- Auto-start: the add-on starts its socket server every time Blender opens (`blendermcp_auto_start_server` default True, addon.py:7638-7642).
- Name trap: the old PyPI package `blender-mcp` (2.0.0) is now a shim that installs mcp-for-blender (CONFIRMED, PyPI). The Blender Lab server's own pyproject is also named `blender-mcp` (CONFIRMED). If you ever install the official one, use its git URL, never `pip install blender-mcp`.

## 4. Security

- Arbitrary code execution, by design. `execute_blender_code` runs any Python with your user's full rights: files, network, subprocesses (CONFIRMED, addon.py:2554). A confused or prompt-injected agent (for example from text inside a downloaded asset's metadata) could do anything you could do.
- Socket: TCP port 9876, bound to `localhost` by default (CONFIRMED, server.py:57-58, addon.py:1341 and 1484-1486, preference default addon.py:6987-6991). Binding to anything other than loopback needs the extra "Allow Remote Connections" box, default False, and its label warns there is no auth or encryption (addon.py:133-147, 6992-7000).
- No authentication on the socket. Any local process can send `execute_code`. Issue #404 asked for a token. The maintainer closed it on 2026-11-28 as "not planned": "A process that can connect to localhost already runs as your user" (CONFIRMED). That reasoning is mostly fair for a single-user Mac.
- Safe mode: off by default. `BLENDER_MCP_SAFE_MODE=1` makes the server check each script before it runs. It allows bpy, bmesh, mathutils and pure stdlib. It blocks os/subprocess/network/open/eval, handlers and timers. Rendering, saving and import/export are still allowed (CONFIRMED, safe_mode.py:78-80, server.py:791-806). It is a static filter in the MCP server, not a true sandbox. Raw socket clients skip it. But it does stop the agent from doing anything outside modeling.
- Past CVEs: CVE-2026-10688 (code injection via execute_blender_code), CVE-2026-10661 (input_image_url injection) and CVE-2026-10662 (SSRF), published around 2026-06-02 against an older commit, with fix commit 5b37be2 / PR #205 referenced (UNVERIFIED, from NVD/OSV/SentinelOne search results. I did not open the NVD pages). The repo has no GitHub security advisories (CONFIRMED, API returned none). One open SSRF fix for Hunyuan downloads, PR #427 (CONFIRMED).

### Telemetry: two layers

1. Content collection (prompts, generated code, viewport screenshots, scene data, plus records of the edits you make by hand in Blender). Opt-in and off by default: the `telemetry_consent` checkbox defaults to False (CONFIRMED, addon.py:7008-7013, consent_prompt.py, TERMS section 2). Some clients may show a one-time yes/no prompt. Answer no. Note the server asks the agent to pass your verbatim words as `user_prompt` on every call (server.py:782). Without consent those are stripped before sending (telemetry.py:226-240).
2. Minimal usage record: on by default, opt-out. A random install ID, session ID, tool name, success, duration, versions, OS and timestamp are POSTed to the author's Supabase (`yzasssndwqceclzilcdu.supabase.co/rest/v1/telemetry_events`). CONFIRMED: telemetry.py:292-314 sends. The Supabase URL and `enabled: bool = True` are in `config.py`, which is gitignored and not in the GitHub source, but it ships inside the published wheel (I unpacked integrations/claude/mcp-for-blender/vendor/mcp_for_blender-2.3.4-py3-none-any.whl). So what's on GitHub is not quite what you run.
   - Turn it off: env `DISABLE_TELEMETRY=true` (or `BLENDER_MCP_DISABLE_TELEMETRY` / `MCP_DISABLE_TELEMETRY`). It must be set in the MCP server's environment (CONFIRMED, telemetry.py:114-125). The `disable_telemetry` tool only turns off layer 1, not layer 2 (server.py:537-562).
   - Terms if you ever opt in: stored indefinitely, may be used to train models and released in public datasets, and you grant a perpetual license (TERMS sections 3 and 10).

### Blender Lab official server, security quick look
- Also binds to localhost:9876 by default and runs code with `exec` (CONFIRMED, mcp_to_blender_server.py:39-40 and 252). Its "weak sandbox" says outright: "this isn't really a sandbox" (weak_sandbox.py).
- No telemetry or outbound HTTP calls in the files I read (blmcp/__init__.py, add-on __init__, server module). The add-on refuses to start unless Blender's "Online access" preference is on. I did not read every tool file, so "no telemetry" is likely but not fully confirmed.
- Its tools focus on inspection: execute_blender_code, scene/object/blend-file summaries, window and area screenshots, render_viewport_to_path, render_thumbnail_to_path, API and manual doc search (CONFIRMED, directory listing of mcp/blmcp/tools). There are no asset or AI-generation integrations. It also has a background (headless) mode via `--command blender_mcp` (add-on __init__.py:220).

## 5. How good is agent-driven modeling, honestly

- What it does well: blocky low-poly architecture. Houses, walls, towers, roofs, fences, crates, barrels and wells are boxes, cylinders, bevels, arrays and flat-color materials. Claude writes that bmesh/bpy code reliably and can lay out a tidy, consistent kit with shared materials, a sensible origin, real-world scale and a low triangle count. The `look` tool lets it catch obvious mistakes: a floating roof, a wrong scale, flipped normals.
- What it does poorly: organic shapes. Characters, faces, hands, animals, good silhouettes, clean topology for deformation, sensible UV unwraps and rigs. Expect stiff "primitive-built" characters. They are fine as a deliberate style (blocky like Kenney or KayKit), but don't expect hand-modeled quality. AI generators (Meshy, Tripo and so on) produce better-looking characters, but with dense, messy meshes and baked textures. They cost money, and they are wrong for a CC0 low-poly web game without manual cleanup.
- Seeing is the only real advantage over scripts. Everything else the MCP does (building geometry, materials, glTF export) is plain bpy. You can get the same thing without any MCP: the agent writes `build_tower.py`, runs `blender -b -P build_tower.py -- --out tower.glb`, renders a preview PNG in the same run, and reads the PNG back with its image reader. That is headless, reproducible, versioned in git, testable in CI (triangle budget, bounding box, material count, file size), needs no open socket, sends no telemetry, and you can regenerate every asset after a style change. For a kit of buildings, that script pipeline is the better primary method.
- Where the MCP wins: fast trial and error on one hard piece ("make the roof steeper, now show me from the street"), poking at an existing .blend, or a CC0 asset you downloaded and want to adjust.

## 6. Verdict

Worth installing for custom art? Yes, as an optional extra, not the main pipeline. Make the scripted headless Blender pipeline the default (generator scripts in the repo, `blender -b -P`, preview renders, checks on triangles and size, GLB out). Add ahujasid's MCP for sessions where interactive looking and tweaking helps. Skip the Blender Lab server for this job: it's better for inspection than creation. It is still the cleaner choice if you only want the agent to query or debug .blend files.

If you install ahujasid's MCP, take these precautions:

1. Install by hand, not with the `curl | sh` one-liner. Add the server pinned and with telemetry off, for example: `claude mcp add blender --env DISABLE_TELEMETRY=true --env BLENDER_MCP_SAFE_MODE=1 -- uvx mcp-for-blender==2.3.4`. Install the add-on with `uvx mcp-for-blender==2.3.4 install-addon` or from the bundled addon.py.
2. Telemetry: leave "Allow Telemetry" unchecked, and say no if a consent dialog appears. `DISABLE_TELEMETRY=true` stops the default usage ping too.
3. Safe mode on (`BLENDER_MCP_SAFE_MODE=1`). Modeling and glTF export still work, and it blocks file, network and subprocess access from agent-written code.
4. Localhost only: leave Bind Address = localhost and Allow Remote Connections off.
5. Integrations: leave all AI generators off (Hyper3D, Hunyuan3D, Tripo, Meshy, World Labs). They send your prompts and images to third parties, cost money, and bring in the Premium ad. Leave Sketchfab off. Turn on Poly Haven (keyless, CC0) and Poly Pizza (free key, filter CC0) only if you want reference or kit pieces.
6. Only while working on art: turn off "Auto-Start Server" in the sidebar (or disable the add-on) so port 9876 isn't open every time Blender launches. Set `BLENDERMCP_NO_UPDATE_CHECK=1` if you don't want the GitHub update ping. Save before each session, since the agent edits your live file.
7. Treat updates like any dependency: bump the pinned version on purpose, not automatically.

## Sources

- github.com/ahujasid/mcp-for-blender (API metadata, contributors, commits, issues #397, #404, #427, files: README.md, TERMS_AND_CONDITIONS.md, install.sh, pyproject.toml, addon.py, src/blender_mcp/{server,telemetry,telemetry_decorator,consent_prompt,safe_mode,premium_hint,setup_cli,update_cli,addon_manager}.py, vendored 2.3.4 wheel config.py)
- pypi.org/pypi/mcp-for-blender/json, pypi.org/pypi/blender-mcp/json
- projects.blender.org/lab/blender_mcp (Gitea API: repo, releases, tree; files: LICENSE, blender_manifest.toml, mcp/README.md, mcp/pyproject.toml, mcp/blmcp/__init__.py, add-on __init__.py, mcp_to_blender_server.py, weak_sandbox.py)
- claude.com/connectors/blender
- NVD/OSV/SentinelOne entries for CVE-2026-10688, -10661, -10662 (via search results only, UNVERIFIED)
- blender.org/lab/mcp-server returned HTTP 403, not read
