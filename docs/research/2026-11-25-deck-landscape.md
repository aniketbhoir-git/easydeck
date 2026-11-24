# Deck landscape research (2026-11-25)

Scope: open and closed "deck" apps, compared with EasyDeck SPEC v4 (§0, §4) and STRATEGY §1.2.
We take ideas and behaviour only. We copy no code, names, logos or art.

## 1. Summary

- Most projects fall in two groups: Stream Deck clones (OpenDeck, StreamController, streamdeck-ui) and phone or web remotes (Macro Deck, Touch Portal, Deckboard).
- Macro Deck 3 (stable 2026-10-31) is the closest rival. It uses "everything is a widget": buttons, clock, weather, slider, graph, gauge, countdown, stopwatch, counter.
- Live values, shown as variables, drive tile look and logic in Macro Deck and Companion. EasyDeck has `sources` and `gauge`, but no general variable layer.
- EasyDeck lacks small basic tiles that rivals ship: counter, countdown or stopwatch, volume slider, per-state look, long press, weather.
- Most rivals use a plugin store with third-party code. STRATEGY rules this out, so each new tile must be a built-in kind that a deck file can describe.

## 2. Projects

| Project | License | Stack | Input device model | Plugin model | Notable features |
|---|---|---|---|---|---|
| Macro Deck 3 | Apache-2.0 | .NET/C# host (plugins are C#); web client; Companion apps | Phone, tablet or browser as the surface; Windows, macOS, Linux host | Signed C# plugins from a reviewed Store; declarative "Macro Deck UI" for widgets | Widgets, profiles, folders, variables, events, flows with if/else/loops, button states, 33 built-in integrations |
| StreamController | GPL-3.0 | Python, GTK4 | Elgato Stream Deck hardware (plus experimental clones); Linux only | Plugins and an asset store | Auto page switch by active window, auto-lock, sticky actions, wallpapers, screen saver, CLI and D-Bus API |
| OpenDeck | GPL-3.0 | Rust, Tauri, TypeScript | Stream Deck hardware; Linux, Windows, macOS; phone via a third-party app | Runs original Stream Deck SDK plugins (Wine for Windows-only ones) | Multi Actions, Toggle Actions, profile switch by app, brightness |
| Bitfocus Companion | MIT (part 2 of LICENSE.md is a contributor agreement) | Node.js web admin | Stream Deck and other surfaces, built-in emulator, web buttons, remote "Satellite" surfaces | 700+ connection modules | Feedback (look follows device state), stacked actions, action logic and delays, duration groups, triggers by OSC, TCP, UDP, HTTP, WebSocket |
| Touch Portal | Closed; free and Pro tiers | Native mobile apps plus PC app | Phone or tablet over network | Plug-ins (Pro tier) and API | Multi-action buttons, logic, sliders, grids up to 110 buttons, unlimited pages, built-in OBS, Twitch, Hue, HTTP, sounds |
| Deckboard | Closed core (a public repo exists; features only) | Electron (repo topics) | Phone as remote | Limited | Macro buttons, folders, OBS remote, macro recorder (low detail) |
| LioranBoard | Closed | Windows desktop | Phone or Stream Deck as surface | Extensions | OBS and Twitch automation (search listing only; low detail) |
| streamdeck-ui / streamdeck-linux-gui | MIT | Python, Qt | Stream Deck hardware; Linux | None | Brightness control, pages; maintenance mode, no new features |

## 3. Gadget matrix

Key: Y = built in. P = plugin, module or integration. N = no. ? = not seen in sources. Planned = a v1 type in SPEC v4.

| Gadget | Macro Deck 3 | StreamController | OpenDeck | Companion | Touch Portal | Deckboard | EasyDeck (SPEC) |
|---|---|---|---|---|---|---|---|
| Clock / date | Y | P | P | P | P | ? | Planned (`text`) |
| CPU / RAM / GPU stats | Y (history graph, gauges, system variables) | P | P | P | P | ? | Planned (`gauge`, `system_stats`; GPU not named) |
| Media controls | Y (music player widget) | P | P | P | P (Spotify plug-in) | ? | Planned (`media`) |
| Volume / mixer | Y (slider bound to a variable) | P | P | P | Y (sliders) | Y | No (v2 "Should") |
| App launch | Y | Y | P | P | Y | Y | Planned (`launch`) |
| Hotkey / macro | Y (keyboard, mouse) | Y | Y | P | Y | Y | Planned (`macro`) |
| Toggle / state button | Y (button states) | Y (states) | Y (Toggle Actions) | Y (feedback) | Y (logic) | ? | Planned (`toggle`) |
| Folder / page navigation | Y (folders, profiles) | Y (pages) | Y (profiles) | Y (pages) | Y (pages) | Y (folders) | Planned (`page` action) |
| Auto page by app or game | Y (profile auto-activation) | Y (active window) | Y (by app) | ? | ? | ? | Planned (page `trigger`) |
| Web / URL open | Y (HTTP, URL) | P | P | P | Y (HTTP) | Y | Planned (`launch` kind `url`) |
| Web embed | ? | ? | ? | ? | ? | ? | Planned (`embed`) |
| OBS | Y | P | P | P | Y | Y | No (key macro only) |
| Home Assistant | Y | P | P | P | ? | ? | No |
| Timer / countdown | Y (countdown, stopwatch) | P | P | P | ? | ? | No |
| Counter | Y | P | P | P | ? | ? | No |
| Weather | Y (no API key) | P | P | P | ? | ? | No (v2 "recipes") |
| Multi-action (sequence) | Y (flows) | Y | Y | Y (stacked) | Y | Y | Partial (one action per tile; `key_macro` has a key list) |
| Text / label | Y | Y | Y | Y | Y | Y | Planned (`text`) |
| Image / icon | Y (icon packs) | Y | Y | Y | Y | Y | Planned (icons; no image tile) |
| Discord | Y (mute, deafen) | P | P | P | P | ? | No |
| Spotify | Y | P | P | P | P | ? | Via `media` only |
| Calendar / email | Y | ? | P | ? | ? | ? | No |
| Long press / double tap | Y | Y | Y | Y (duration groups) | ? | ? | No |
| Slider | Y | ? | ? | ? | Y | ? | No |
| Variables and conditions | Y | N | P | Y | Y | N | No (only `sources`) |
| Local API or CLI | Y (AI clients guide) | Y (CLI, D-Bus) | ? | Y | Y | ? | Planned (local MCP server) |

Notes: "P" for StreamController, OpenDeck and Companion means a plugin or module can cover the gadget. We did not check each plugin. Deckboard cells are thin: sources were repo metadata only. LioranBoard is left out of the matrix for the same reason.

## 4. What EasyDeck lacks

Ranked by value for a touch monitor, cheapest first. Rule fit is against SPEC §0.

| # | Gap | Why | Size | Rule fit |
|---|---|---|---|---|
| 1 | Counter tile | Basic, local, no side effects. Tap adds one. Rivals ship it. | S | Fits. No network, no risk item. |
| 2 | Countdown and stopwatch tile | Cooking, focus and raid timers. Tap starts, pauses, resumes. Hold resets. | S | Fits. Local only. |
| 3 | Per-state look beyond `toggle` | Tiles show state; Companion calls this feedback. | M | Fits if state comes from local sources. |
| 4 | Long press and double tap | Doubles the use of each tile. | M | Fits. Touch layer only; each action stays a risk item. |
| 5 | Volume slider and mute tile | Top need for a monitor next to a game. Already in STRATEGY v2. | M | Fits. Local audio API; no network. |
| 6 | Multi-action tile (sequence with delays) | Rivals chain actions. SPEC has one action per tile. | M | Fits. Each step is a risk item with its own approval hash. |
| 7 | Variables and simple conditions | Macro Deck and Companion drive look and logic by variables. Could extend `sources`. | L | Fits if data-only. Avoid a scripting language. |
| 8 | GPU stats in `gauge` | Gamer persona wants it. | S-M | Fits. Local OS call in the platform crate. |
| 9 | Image tile | Simple display. | S | Fits. A remote image is a risk item. |
| 10 | Weather tile | Common. Macro Deck needs no API key. | M | Needs network: an approved `http_poll` source only. |
| 11 | OBS control | Streaming persona. Local WebSocket. | M | Network on loopback. Risk item; keep the password outside the deck (§0 rule 5). |
| 12 | Home Assistant tile | Popular with home-lab users. LAN call. | M | Needs network: approved endpoint. Keep the token outside the deck (§0 rule 5). |
| 13 | Discord and Spotify controls | Large demand, but accounts, OAuth and cloud calls. | L | Poor fit. Use `media` and key macros instead. |
| 14 | Third-party plugins and store | Core of Macro Deck and Companion. | L | Conflicts with STRATEGY "Won't" (no third-party code). Do not build. |
| 15 | Remote surfaces over the LAN, cloud sync | Companion Satellite, phone apps. | L | Conflicts with the local-only rule. Defer. |

## 5. Recommended first gadget set

Order: cheapest first. Each tile works with no network.

1. **Clock and date** (`text`). Borrow: one tile with a format string; 12 or 24 hour option; date line under the time.
2. **Counter**. Borrow: tap adds one, hold resets, optional step size, value kept between runs.
3. **Countdown and stopwatch**. Borrow: tap starts, pauses, resumes; hold resets; the tile changes colour and pulses when time ends.
4. **Gauge for CPU, RAM, GPU**. Borrow: a number plus a short history line; colour changes at a threshold.
5. **Volume slider and mute**. Borrow: a slider bound to system volume; a mute toggle with a clear off state.
6. **Media controls**. Borrow: title and artist, play or pause, skip, no extra setup.
7. **Toggle with per-state look**. Borrow: each state has its own icon, label and colour; state can follow a local source.
8. **Launch tile with long press**. Borrow: tap launches; long press runs a second action (for example, open the folder). Each action stays a risk item.

Weather, Home Assistant and OBS wait until these ship, because each needs an approved network item.

## 6. Sources

- https://macro-deck.app/ and https://macro-deck.app/why-macro-deck
- https://macro-deck.app/integrations
- https://macro-deck.app/blog/macro-deck-3-0-0
- https://docs.macro-deck.app/ and https://docs.macro-deck.app/guide/concepts/
- https://github.com/Macro-Deck-App/Macro-Deck (Apache-2.0 badge)
- https://github.com/StreamController/StreamController (README) and https://streamcontroller.core447.com
- https://github.com/nekename/OpenDeck (README)
- https://github.com/bitfocus/companion (README) and https://companion.free
- https://www.touch-portal.com
- https://github.com/rivafarabi/deckboard (metadata only)
- https://github.com/streamdeck-linux-gui/streamdeck-linux-gui (README; maintenance mode)
- https://lioranwaters.github.io/lioranboard2docs/download (search listing only; the main site did not resolve)

Gaps: Deckboard and LioranBoard feature lists are thin. Check them before you cite them.
