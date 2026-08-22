# SwissMath Web v0.1 — Implementation report

Data: 22 agosto 2026
Baseline: SwissMath Core v0.5 / desktop v0.5

## Esito

SwissMath Web v0.1 è stato implementato come applicazione statica locale con
adapter Rust/WASM sottile. Il motore matematico resta `swissmath-core`; la GUI
è l'interfaccia HTML/CSS/JS già validata sul desktop, adattata al browser senza
Tauri. Il bundle production è in `dist/web/`.

È stato inoltre pubblicato il Site pubblico ChatGPT:

- URL production: https://swissmath.lucapezzullo.chatgpt.site
- Deployment pubblico riuscito (`succeeded`).

L'accesso anonimo al deployment è stato verificato con HTTP 200 e con il
browser: la pagina mostra la GUI SwissMath e un calcolo modulare completo.

## Architettura

```text
SwissMath Core v0.5 (Rust)
        │
        └── apps/web/src/lib.rs
            adapter WASM + envelope JSON { ok, value | error }
        │
        └── dist/web/pkg/swissmath_web_bg.wasm
            + swissmath_web.js (loader wasm-bindgen)
        │
        └── dist/web/index.html + app.js + styles.css
            GUI browser, preflight, salvataggio e stampa/PDF
```

L'adapter espone dieci entrypoint `wasm_bindgen`, senza duplicare la matematica
del core:

`wasm_calculate_modular`, `wasm_calculate_crt`,
`wasm_calculate_residues`, `wasm_solve_linear`, `wasm_solve_system`,
`wasm_run_sieve`, `wasm_analyze_integer`,
`wasm_calculate_multiplicative_order`, `wasm_calculate_quadratic_symbols`,
`wasm_find_modular_roots`.

La dipendenza `getrandom 0.2` è configurata con la feature `js` soltanto nel
crate standalone `apps/web`, necessaria per il target `wasm32-unknown-unknown`.
Il workspace desktop e `crates/core` non sono stati modificati per la Web UI.

Non sono presenti API applicative, backend, telemetry o CDN necessari al
calcolo: la matematica viene eseguita localmente dal WASM. Il loader
`wasm-bindgen` carica esclusivamente il file `.wasm` incluso nel bundle.

## Build production

Comando verificato:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts\build-web.ps1 -Offline
```

Il risultato contiene esattamente:

| File | Byte | SHA-256 |
|---|---:|---|
| `app.js` | 30.665 | `D70F4C13C9378C50BEFE376F2358815271275772EF0CE87C77ACB02053857EFD` |
| `index.html` | 26.133 | `38C326A9C09D0EB044CDC2BC68200E78C9BFEFC448C49BACB73EACC528C1B390` |
| `styles.css` | 20.693 | `5F430F0F35D4E8E317CF22D4E96586419F264EF8DC0CC1654BFEAB1314AC8716` |
| `pkg/swissmath_web.js` | 16.363 | `0ADB9157437A0DE76FED9ADB8DB41FC82F4CBFE6F48882FD9D1FC6B3C13DE687` |
| `pkg/swissmath_web_bg.wasm` | 509.343 | `AA7D949B11A00337D0CD429CBFB4FB5587845C8671E5E0F1BB4248FC172EC1AB` |

Il bundle non contiene `.d.ts`, `target/` o dipendenze frontend installate.

## Smoke test browser locale

La build è stata servita da `127.0.0.1` e verificata con la GUI reale. I tempi
sono campioni di singole interazioni, non benchmark:

| Area | Caso | Esito osservato |
|---|---|---|
| Modulare | `m=7, a=10, b=5, n=4` | somma 1, differenza 5, prodotto 1, potenza 4, inverso 5; 12,5 ms |
| CRT | `2 (mod 3)`, `3 (mod 5)` | `x ≡ 8 (mod 15)`; 1,3 ms |
| Insiemi | intersezione mod 12 | `1, 4`; 1,4 ms |
| Congruenza | `14x ≡ 8 (mod 30)` | `x ≡ 7 (mod 15)`, residui 7 e 22; 1,1 ms |
| Sistema | righe `14x≡8 (mod 30)`, `x≡0 (mod 2)` | `x ≡ 22 (mod 30)`; 0,3 ms |
| Interi | `97` | primo, verifica esatta, φ=λ=96; 0,8 ms |
| Interi | `360` | composto, `2^3×3^2×5`, φ=96, λ=12 |
| Primalità larga | primo u128 esatto | etichetta `Primo — verifica esatta`; 16,1 ms |
| Primalità oltre u128 | Mersenne M521 | etichetta `Probabile primo` con nota BPSW; 75,9 ms |
| Quadratici | simboli `a=5,n=11` | Jacobi 1, Legendre 1 |
| Quadratici | radici `a=10,n=13` | due radici: `6, 7`; 2,5 ms |
| Sieve | tre filtri, intervallo 0…100 | 18 valori, 17,82%, preview completa; 0,4 ms |

In ogni caso il riquadro preflight è comparso con il tempo espresso in ms e
sono rimasti disponibili i pulsanti `Salva risultato` e `Stampa / PDF`.

## Regressioni

Passati:

```text
cargo fmt --all --check
cargo fmt --manifest-path apps/web/Cargo.toml --all --check
cargo test --workspace --offline                 (tutti i test passati)
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo test --manifest-path apps/web/Cargo.toml --offline  (13 test passati)
cargo clippy --manifest-path apps/web/Cargo.toml --offline --all-targets -- -D warnings
node --check dist/web/app.js
```

La regressione workspace include `swissmath-core`, i test dell'adapter desktop
e le proprietà di CRT, sieve, primalità, quadratici e sistemi. Il check/clippy
del desktop v0.5 passa insieme al workspace; nessun file desktop è stato
modificato dalla fase Web.

## Sites

Il Site usa il layout Cloudflare-compatible:

- `dist/client/`: client statico, inclusi `index.html`, CSS, JavaScript e WASM;
- `dist/server/index.js`: worker statico con fallback SPA a `index.html`.

Il pacchetto di deployment è stato validato prima della pubblicazione e il
deployment pubblico è stato verificato sulla homepage production.

## Limiti noti

- Il Site è pubblico e raggiungibile senza login all'URL indicato sopra.
- La fattorizzazione e le funzioni derivate restano limitate al dominio già
  dichiarato da Core; oltre u128 la UI mantiene esplicitamente la semantica
  probabilistica (`Probabile primo`) quando non esiste una prova esatta.
- Il package Web è volutamente minimale: nessun framework o installazione npm,
  nessuna sorgente desktop incorporata nel bundle e nessuna modifica al core.
