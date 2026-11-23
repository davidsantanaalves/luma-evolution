# Luma Evolution

A remote-first launcher for rooted LG webOS TVs, inspired by Apple TV. Luma Evolution forks
[Luma+](https://github.com/eluz1ve/lg-luma-plus), which builds on [Luma](https://github.com/ecklf/lg-luma)
by ecklf. It includes a Today card, Lampa, CS2 and crypto widgets, and a tvOS-style layout.

<p align="center">
  <img src="docs/slideshow.webp" alt="A tour of Luma Evolution: Home with the Today card, CS2 Scores and a live match, crypto prices, the apps grid, settings with a live preview, and wallpapers">
</p>

## Highlights

- **Today card.** Beside the greeting: live and upcoming CS2 matches, new episodes and coin
  prices at a glance. OK opens it, with a tab for each widget.
- **Home your way.** Today with a CS2 or Lampa row in the center, Today alone, or Clean: just
  the clock, the greeting and your favorites over the wallpaper.
- **Lampa.** A row of posters: what Lampa viewers are watching now, or your own Continue watching
  from your CUB account (sign in with a QR code). OK opens the series or film right in Lampa,
  through a small Lampa plugin. Outside Russian, series show their English posters from
  [TVmaze](https://www.tvmaze.com).
- **CS2 Scores.** Counter-Strike 2 matches from Liquipedia with team logos, live scores and
  results, and a match dialog with maps, rounds and lineups. While a match is played: the round
  score on Home and in Today, and in the dialog a live scoreboard, round history and game log
  from HLTV, with the match's Twitch and YouTube streams one OK away.
- **Crypto prices.** Up to 20 coins, from the usual eight or any Binance coin, running as a ticker
  in the Today card.
- **A tvOS-style screen.** A side menu instead of the tab bar, glass panels, a softer focus, and
  Interface scale for the clock, names and cards.
- **Settings with a live preview** of Home that follows every change.
- **Wallpapers.** Apple TV's aerial stills, changing every fifteen minutes, and dynamic
  wallpapers that follow the sun at your weather location.
- **TV conveniences.** AirPlay as an input, Close all in the app switcher, faster running-app
  switching, and optional automatic updates for Homebrew apps.
- **Vector text rendering**, weather details and place names in your language.

Luma Evolution installs beside Luma and Luma+ under its own app ID, `com.davidsantana.lumaevolution`.
Preferences are independent. Launcher updates and publication are disabled during the foundation.

The Evolution Home can be reviewed with the native desktop option `pnpm dev --home-preview`.
The preview includes configurable All Apps and Settings actions, a separate inputs row,
isolated preferences and labeled fictional content. See [the Stage 02 report](docs/home-stage02.md).
In Settings → Interface → Dock, preview and save the Dock scale for expanded Cinema.
The setting defaults to 40% and accepts 40% to 100% in steps of 5 percentage points.
In Settings → Interface → Cinema, preview and save the compact Home's 16:9 card size.
It defaults to 100% and accepts 40% to 100% in steps of 10 percentage points. With the widget area
expanded, Cinema stays compact and uses 50% of the reference size. See the
[R13 desktop captures](docs/home-stage02-r13-aceite.md).
The [R1–R4 review](docs/home-stage02-review.md) adds horizontal 16:10 favorites, Inputs revealed
after vertical navigation, weather over the wallpaper and a compact header during cinema expansion.
The [R5–R9 review](docs/home-stage02-r5-r9.md) covers Dock/header alignment, expanded upper
widgets with calendar and labeled forecast fixtures, contextual buttons and cinematic rotation.
The [R10 review](docs/home-stage02-r10.md) shows the five Home areas, with bare icons at the top
and calendar and weather details visible together when the widget area has focus.
The [post-R12 review](docs/home-stage02-post-r12.md) compares two compact cinema sizes and
limits widget focus to whole calendar and weather blocks when simulated apps are available.
The [R11/R12 report](docs/home-stage02-r11-r12.md) preserves the earlier review and records
the contextual scroll indicators still used by the desktop Home preview.

## Compatibility

> [!NOTE]
> The table records upstream compatibility reports. Luma Evolution has yet to be verified on
> hardware, and Home takeover is disabled in the foundation builds.

<details>
<summary>webOS 4</summary>

| TV        | Model identifier | Takeover Support |
| --------- | ---------------- | ---------------- |
| LG C8     | `OLED??C8*`      | ❌               |
| LG C9     | `OLED65C97LA`    | Unknown          |
| LG UM7450 | `UM74507LA`      | ❌               |

</details>

<details>
<summary>webOS 5</summary>

| TV        | Model identifier | Takeover Support |
| --------- | ---------------- | ---------------- |
| LG UN7100 | `49UN71006LB`    | ✅               |

</details>

<details>
<summary>webOS 6</summary>

| TV        | Model identifier | Takeover Support |
| --------- | ---------------- | ---------------- |
| LG C1     | `OLED48C1KNB`    | ✅               |
| LG UP8000 | `86UP80009LA`    | Unknown          |

</details>

<details>
<summary>webOS 24</summary>

| TV    | Model identifier | Takeover Support |
| ----- | ---------------- | ---------------- |
| LG B2 | `OLED65B29LA`    | ✅               |

</details>

<details>
<summary>webOS 25</summary>

| TV        | Model identifier | Takeover Support |
| --------- | ---------------- | ---------------- |
| LG B5     | `OLED77B5PUA`    | ✅               |
| LG C2     | `OLED42C2PSA`    | ✅               |
| LG C2     | `OLED65C2*`      | Unknown          |
| LG C2     | `OLED77C2*`      | Unknown          |
| LG C4     | `OLED42C4*`      | ✅               |
| LG G3     | `OLED??G3*`      | Unknown          |
| LG NANO77 | `55NANO776RA`    | ✅               |
| LG QNED86 | `55QNED86T6A`    | Unknown          |
| LG UT8050 | `50UT8050*`      | ✅               |

</details>

<details>
<summary>webOS 26</summary>

| TV    | Model identifier      | Takeover Support |
| ----- | --------------------- | ---------------- |
| LG C2 | `OLED65C2PSA`         | ✅               |
| LG C3 | `OLED65C31LA`         | ✅               |
| LG C3 | `OLED65C36LC`         | ✅               |
| LG C5 | `OLED55C57LA`         | ✅               |
| LG C5 | `OLED55C5PSA`         | Unknown          |
| LG C6 | `OLED65C6*`           | Unknown          |
| LG C6 | `OLED77C6*`           | ✅               |
| LG G5 | `OLED83G5WUA.BUSYDJR` | ✅               |
| LG G5 | `OLED77G5WUA.DUSQLJR` | ✅               |

</details>

## Installation

### Homebrew Channel

The future release and feed source is [Luma Evolution Releases](https://github.com/davidsantanaalves/luma-evolution/releases).
For a local build, use the IPK from [Setup & Building](#setup--building).

### SSH

On your computer, replace `TV_IP`:

```sh
scp -O dist/com.davidsantana.lumaevolution_1.0.0_arm.ipk root@TV_IP:/tmp/luma-evolution.ipk
ssh root@TV_IP
```

In the TV's root shell:

```sh
luna-send -i luna://com.webos.appInstallService/dev/install \
  '{"id":"com.davidsantana.lumaevolution","ipkUrl":"/tmp/luma-evolution.ipk","subscribe":true}'
```

### Lampa plugin

Lampa on webOS ignores links from other apps, so without help OK on a Lampa card only opens
Lampa's home. Add the plugin once, in Lampa: **Settings → Extensions → Add plugin**, enter

```text
https://cdn.jsdelivr.net/gh/eluz1ve/lg-luma-plus@008231201726042896b2abd2dcea421b2f4c89d6/lampa/luma.js
```

and restart Lampa. Luma Evolution shows the same address with a QR code in
**Settings → Widgets → Lampa → Lampa plugin**.

## Replacing LG Home

Home takeover is disabled in this build. The existing integration is documented in the
[development guide](development.md#optional-native-home-integration). Standalone autostart and
manual LG Input Hook mappings can open the launcher as a regular app.

### Autostart Method

Open Luma Evolution 25 seconds after a full cold boot. Run in the TV's root SSH shell:

```sh
sh /media/developer/apps/usr/palm/applications/com.davidsantana.lumaevolution/autostart.sh enable
```

Use this when takeover is disabled. It launches Luma Evolution as a regular app and does not remap the Home button.

Disable before uninstalling Luma Evolution:

```sh
sh /media/developer/apps/usr/palm/applications/com.davidsantana.lumaevolution/autostart.sh disable
```

### LG Input Hook Method

You can use [LG Input Hook](https://repo.webosbrew.org/apps/org.webosbrew.inputhook/), configure an Execute action with this one-line command:

```sh
/usr/bin/luna-send -n 1 -w 5000 luna://com.webos.service.applicationmanager/launch '{"id":"com.davidsantana.lumaevolution"}'
```

Test on a spare button first. Disable the mapping in Input Hook to restore normal Home behavior.

## Features

### Backgrounds

Open **Settings → Background → Upload**, then scan the QR code or open the shown
address on a phone or computer on the same network. Choose a JPEG, PNG or WebP and adjust
its 16:9 crop before choosing **Save to TV**.

The upload link works only while its TV dialog is open and focused.

Enable **Rotation** beside **Upload** to cycle through backgrounds on each Luma Evolution restart and every
15 minutes with a slow cross-fade. Rotation is off by default and leaves your manually chosen
LG splash unchanged.

Toggle **Gradient** beside **Rotation** to turn background shading on or off. It defaults to On
and remembers your choice. Older uploads without an unshaded copy keep their full-resolution
shaded image. This does not change LG's launch splash.

### Recent app switching

Press **Back** on Home, or hold **Back** for 0.6 seconds from any page, to open Recent apps.

- **Left / Right**, the pointer, or the scroll wheel browses recent apps and inputs.
- **OK** or a pointer click opens the selected app or input.
- **Up** closes the selected app and removes its card.
- **Down** selects **Close all**; press **OK** to close the displayed apps one at a time.
- **Back** dismisses the switcher and restores the previous page and focus.

These shortcuts work while Luma Evolution is focused. From another app, return to Luma Evolution first.

In the upstream rooted Home integration, the switcher lists running applications and closes them directly
on its TV worker, without starting Homebrew or JavaScript for each operation. System apps and
inputs are excluded; a process shared by multiple applications is never terminated directly.
Automatically preloaded apps appear only after being opened by the user. Closing their card
does not change the TV's automatic preload settings.

### Shortcut tiles

Shortcut tiles sit on Home like apps but send a web request when selected, for example to a Home
Assistant webhook. Open **Settings → Widgets → Shortcuts → Configure** and scan
the QR code.

### Homebrew app updates

Turn on **Settings → System → Update Homebrew apps automatically** to keep apps installed through
Homebrew Channel, such as Moonlight or YouTube AdFree, current. It is off by default. When on, Luma Evolution
compares them with [Homebrew Channel's repository](https://repo.webosbrew.org) ten minutes after
starting and then twice a day, and installs a newer version only while that app is closed and only
if the package matches the repository's checksum and declares the requested app ID. A short notice
names each updated app. This setting excludes Luma Evolution, Luma+, known Luma predecessors
and Homebrew Channel.

## Development

### Setup & Building

From the repository root, with Nix flakes enabled:

```sh
nix develop
pnpm install --frozen-lockfile
pnpm sdk
export WEBOS_NDK="$PWD/.build/toolchain/arm-webos-linux-gnueabi_sdk-buildroot"
pnpm run package
```

The first native runtime preparation also downloads a
checksum-pinned SDL-webOS library; both downloads are cached under `.build/`.

Output:

```text
dist/com.davidsantana.lumaevolution_1.0.0_arm.ipk
dist/sha256sums
dist/repo.json
dist/app-icon-large.png
```

Open [webOS Dev Manager](https://github.com/webosbrew/dev-manager-desktop), connect to the TV's
IP on port **22** as **root**, and install the IPK. Open **Luma Evolution** from the TV menu. Homebrew
Channel must be elevated; on webOS 26 use **0.8.0-rc1 or newer** for its `lunabus.query` mapping.

The package uses **`com.davidsantana.lumaevolution`**. Preferences are local JSON files scoped to that app ID and
are independent of Luma and Luma+.

Wait for installation success; Ctrl+C closes an ongoing subscription. Fully close and reopen
Luma Evolution after updating. The [development guide](development.md#tv-startup-diagnostics) has a jailed
launch command that exposes errors hidden by the normal menu launch.

## Preview

```sh
nix develop
pnpm install --frozen-lockfile
pnpm assets
pnpm dev
```

The desktop preview uses sample inventory and never calls TV services. To manually exercise a
longer app grid:

```sh
pnpm dev --test-extra-apps 120
```

Preview preferences are separate from normal preferences. See [development.md](development.md)
for controls, vector rendering, diagnostics, tests and the source layout.

CI produces verified build artifacts. Release publication is disabled during the foundation.
See [Releases](development.md#releases).

## License

Luma Evolution, [Luma+](https://github.com/eluz1ve/lg-luma-plus) and [Luma](https://github.com/ecklf/lg-luma) are licensed under MIT.\
Luma Evolution is not affiliated with LG, Apple, Lampa, CUB, TVmaze, Liquipedia, Binance or any other
brands shown in its UI. Series posters outside Russian come from [TVmaze](https://www.tvmaze.com)
(CC BY-SA).
