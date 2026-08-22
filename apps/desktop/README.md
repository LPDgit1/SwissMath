# SwissMath desktop GUI v0.5

This is the offline Tauri 2 application over `swissmath-core`. The UI remains a
plain local HTML/CSS/JavaScript bundle. All mathematical operations are Rust
commands through Tauri IPC; there is no web service, CDN, telemetry, remote
font, or database.

## Build locally

From the repository root, with Rust and `cargo-tauri` available:

```powershell
cargo check -p swissmath-desktop
cargo tauri build --no-bundle
cargo tauri build --bundles nsis
```

The first command validates the crate, the second produces a standalone
executable, and the third also creates the NSIS installer. Windows WebView2 is
required by Tauri. Release binaries and installers are generated outside the
public source tree.

## GUI scope

- modular addition, subtraction, multiplication, power, and inverse;
- two-congruence generalized CRT;
- `ResidueSet` intersection, union, difference, complement, and queries;
- `Congruenze`: one linear equation plus an editable system table;
- `Filtro modulare`: allowed/excluded residue rows and direct linear-congruence
  rows over an inclusive range, with count, percentage, anchor, and preview;
- `Numeri interi`: exact u64, bounded exact-first u128, or BPSW above u128,
  with explicit labels for unavailable or probable results;
- `Residui quadratici`: Jacobi/Legendre symbols and bounded modular-root
  previews for the supported prime and composite-unit cases;
- local result snapshots and native print/PDF output;
- completion notices and preflight summaries with the measured operation time;
- decimal-string IPC so arbitrary `u64` values are never rounded by JavaScript.

Materialized residue and allowed/excluded GUI filters are capped at modulus
2,000,000 for responsiveness. The Rust core remains the authority for all
mathematical validation and overflow handling.
