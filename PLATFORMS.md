# Platforms: Anki, web, phones

How the game gets onto each platform, and which tools to use. Researched November 2026.

## Summary

**Build the game once in web tech (Phaser 4 + TypeScript), then ship that same game everywhere:**

| Order | Platform | How | Payments |
|---|---|---|---|
| 1 | **Anki add-on** (desktop) | Game runs in Anki's built-in web view | Web checkout (merchant of record), we keep ~95% |
| 2 | **Web / PWA** (phone companion) | Same game at a URL; "Add to Home Screen" | Web checkout, ~95% |
| 3 | **App Store / Google Play** | Same game wrapped with **Capacitor** | Must offer in-app purchase (Apple/Google take 15% under $1M/yr); web purchases still work across platforms |

Gems live on our server, so a purchase made anywhere works everywhere.

---

## Options compared

| Option | What it is | Fit for this game |
|---|---|---|
| **Phaser 4** | 2D web game engine (MIT). v4.0 stable April 2026, new WebGL renderer, v4.2.x current | **Use it.** Runs in Anki, browsers and Capacitor. |
| **PWA** | Website that installs to the home screen | **Good first phone version:** no app review, instant updates, no app store cut. Downsides on iPhone: users must add it manually from Safari's share menu, push notifications only after it's added to the Home Screen (iOS 16.4+), no real background work, tighter storage, and no standalone PWAs in the EU. Fine for us, since game state is on the server. |
| **Capacitor** | Wraps a web app in a native iOS/Android shell, with plugins for native features | **Use it when we go to the app stores.** Phaser has an official Capacitor tutorial, and shipped Phaser games use it. Gives app store discovery, reliable push and native in-app purchases (e.g. via RevenueCat). Web code updates can ship over the air without a new app review (Capgo / Ionic Appflow), within Apple's rules. |
| **Expo / React Native** | Native UI framework in JS | Great for normal apps (lists, forms, cameras). **Not a fit here:** a game is one big drawn canvas, so React Native's native UI adds nothing, and Phaser would end up in a web view inside it anyway. |
| **Exact by Expo** ([github](https://github.com/expo/exact)) | New "agent-native" all-platform framework (MIT); has an optional game engine | **Watch, don't use yet.** It says itself that it's pre-1.0 and changes daily, with no packaged release or stable API. Gameplay is written in Rust. Too risky as a foundation today. |
| **Hotwire Native** | Server-rendered web app in a native shell (Rails world) | Built for server-rendered business apps, not games. The idea it's known for (ship updates by deploying the server) we get from Capacitor live updates. |
| **Godot** | Full free game engine (MIT); exports to web, mobile, desktop | Strong engine, but its web builds are heavy and riskier inside Anki's web view, and it's a separate language and editor. Phaser fits our web-first plan better. |
| **Defold** | Free engine, small builds, strong on mobile and web (Lua) | Good alternative if Phaser ever falls short. |
| **Unity** | Big commercial engine | Overkill; heavy web builds. |

---

## Payments by platform

- **Anki add-on and web/PWA:** web checkout through a merchant of record (Paddle or Lemon Squeezy). No app store cut.
- **Native iOS/Android:** digital goods like gems must be offered through Apple/Google in-app purchase (15% on the small-business programs).
  - **US link-outs to web checkout:** courts have allowed apps to link out. In December 2025 an appeals court said Apple may charge a "reasonable" commission on those, with the amount still being worked out. **Check the current rules before launch.**
  - Gems bought on the web can be used in the app (cross-platform purchases are allowed as long as in-app purchase is also offered).

---

## Art licensing note (open-source add-on)

The add-on will likely be open source (Anki is AGPL). **Tiny Swords' license forbids redistributing the art**, and a public repo containing it could count as redistribution.
So: **keep paid/licensed art out of the public repo.** Serve it from our server or a private asset repo, and publish only code.
(Tiny Swords' older version is CC0 and could go anywhere.)
