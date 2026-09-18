# Third-Party Notices

Novel Studio AI is built with the help of the following open-source
software. This list covers the major direct dependencies (Rust backend and
frontend); it does not enumerate every transitive dependency. A complete,
exact-version license report can be regenerated at release time with
`cargo license` (Rust, `src-tauri/`) and `pnpm licenses` (frontend).

## Rust (src-tauri/)

| Crate | License |
|---|---|
| Tauri | MIT OR Apache-2.0 |
| rusqlite (bundled SQLite) | MIT (rusqlite) / SQLite itself is Public Domain |
| serde / serde_json | MIT OR Apache-2.0 |
| uuid | MIT OR Apache-2.0 |
| chrono | MIT OR Apache-2.0 |
| thiserror | MIT OR Apache-2.0 |
| log | MIT OR Apache-2.0 |
| zip | MIT |
| printpdf | MIT |
| tempfile | MIT OR Apache-2.0 |

## Frontend (React / TypeScript)

| Package | License |
|---|---|
| React / React DOM | MIT |
| react-router-dom | MIT |
| Zustand | MIT |
| Zod | MIT |
| Vite | MIT |
| TypeScript | Apache-2.0 |

## Fonts

- **IPA Gothic (IPAゴシック)** — embedded in the app to render Japanese
  text in PDF exports (`src-tauri/assets/fonts/ipag.ttf`). Distributed
  under the IPA Font License Agreement v1.0, which permits redistribution
  and embedding. Full license text:
  `src-tauri/assets/fonts/IPA_Font_License.txt`.

## Notes

- License identifiers above are SPDX short identifiers as commonly
  published for each project at the time of writing; always confirm
  against the dependency's own `LICENSE` file/metadata before relying on
  this list for compliance purposes.
- SQLite itself (bundled inside `rusqlite`'s `bundled` feature) is
  dedicated to the Public Domain by its authors.
