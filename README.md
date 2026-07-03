# pokeroll

A small cross-platform desktop widget: press one button and it rolls a random
Pokémon from [PokéAPI](https://pokeapi.co) and shows it as a card — sprite,
name, dex number, types, base stats, and a bit of trainer XP.

Built with **Tauri v2** + **SvelteKit** (Svelte 5 runes, TypeScript) and a
small **Rust** backend.

> ⚠️ **Beta.** This is an early, hobby build — expect rough edges. Bug reports,
> ideas, and pull requests are very welcome.

## Features

- **One-button roll** — random Pokémon (1–1025) fetched from PokéAPI.
- **Three image modes** — official art, blocky pixel render, or colored ASCII,
  both generated on the fly from the artwork (with an adjustable grain slider).
- **Stats** — six base stats as animated bars or a hex radar, or hidden.
- **XP & level** — every roll grants base experience; the profile ring fills up
  and animates level-ups.
- **Widget behavior** — frameless, transparent, always-on-top; hides to a tray
  icon instead of closing.
- **Persistence** — settings, XP, and seen Pokémon are saved to localStorage
  and survive restarts.
- **Copy ASCII** — grab the ASCII render as text.

## Install

Grab a build for your OS from the [Releases](../../releases) page.

- **Windows** — run the `.msi` or `-setup.exe`. Unsigned, so click through the
  SmartScreen warning.
- **Linux** — use the `.AppImage` (`chmod +x`, then run) or the `.deb`. The tray
  icon needs `libayatana-appindicator` present on the system:
  - Debian/Ubuntu: `libayatana-appindicator3-1`
  - Fedora: `libappindicator-gtk3`
  - Arch: `libayatana-appindicator`

## Build from source

Prerequisites: [Rust](https://rustup.rs), Node.js, and
[pnpm](https://pnpm.io). On Linux also install the Tauri system deps
(`libwebkit2gtk-4.1-dev`, `librsvg2-dev`, `patchelf`,
`libayatana-appindicator3-dev`).

```sh
pnpm install
pnpm tauri dev     # run in dev
pnpm tauri build   # produce installers in src-tauri/target/release/bundle
```

Releases are built automatically by GitHub Actions (`.github/workflows/release.yml`)
on any `v*` tag.

## Roadmap

Planned improvements:

- **Offline-first** — cache the dataset locally with a Sync button and a weekly
  auto-refresh, so rolling works without a network round-trip (today each roll
  hits PokéAPI live).
- **Pokédex catalog** — a screen listing the Pokémon you've caught, backed by a
  local SQLite database (today only the count is kept in localStorage).
- **Localization** — multi-language UI (currently English only).
- **Custom arts** — allow user-supplied or alternative artwork per Pokémon.
- **Farm / gacha mode** — count clicks as a currency; spend it to buy rolls and
  pull Pokémon.

## Contributing

It's a beta and open for fixes — feel free to open an
[issue](../../issues) or a pull request. Small fixes, new roadmap features, and
localization help are all appreciated.

## License

[MIT](LICENSE)
