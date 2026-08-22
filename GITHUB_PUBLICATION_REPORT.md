# SwissMath — Publication preflight report

Data: 22 agosto 2026
Baseline: SwissMath Core v0.5 / Desktop v0.5 / Web v0.1

## Esito locale

Il preflight pubblico è stato completato senza modifiche matematiche o di GUI.

- Licenza unica alla radice: MIT, con copyright `Copyright (c) 2026 Luca
  Pezzullo`.
- README principale e README desktop riscritti per un progetto pubblico, con
  comandi generici e senza percorsi macchina.
- Report Web e WASM sanitizzati: è mantenuto soltanto l'URL pubblico del Site,
  senza identificativi interni, checkout temporanei o dichiarazioni private.
- `scripts/build-web.ps1` usa il PATH o override espliciti (`-CargoPath` e
  `-WasmBindgenPath`), stampa la versione CLI di `wasm-bindgen` e non installa
  né indovina percorsi locali.
- `scripts/package-source.ps1` accetta `-BundleName` e produce archivi
  riproducibili con un solo prefisso top-level.
- `.gitignore` esclude output di build, cartelle di lavoro, dipendenze locali e
  configurazioni IDE; `Cargo.lock` resta incluso.

## Verifiche

Passate nel workspace e nel crate Web:

```text
cargo fmt --all --check
cargo test --workspace --offline
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo fmt --manifest-path apps/web/Cargo.toml --all --check
cargo test --manifest-path apps/web/Cargo.toml --offline
cargo clippy --manifest-path apps/web/Cargo.toml --all-targets --offline -- -D warnings
node --check dist/web/app.js
scripts/build-web.ps1 -Offline
```

Il Site pubblico `https://swissmath.lucapezzullo.chatgpt.site` era già stato
verificato anonimamente con HTTP 200 e con un calcolo modulare completo. Questa
fase non ha creato un nuovo Site e non ha effettuato un redeploy.

## Git locale

- repository inizializzato sul branch `main`;
- commit iniziale: `b11f5fd1197ba84e36a33cd6e9c5b48a6ae7607e`;
- messaggio: `Initial public release of SwissMath`;
- working tree pulito prima della generazione degli archivi.

## Stato GitHub

La pubblicazione remota è bloccata prima di qualsiasi mutazione: il comando
`gh` non è installato nell'ambiente e non è stato possibile autenticarsi o
determinare l'owner. Di conseguenza:

- repository remoto `SwissMath`: non creato;
- URL GitHub, owner e SHA remoto: non disponibili;
- visibilità pubblica, branch remoto, homepage e topics: non applicati;
- nessun push o altra scrittura remota è stata eseguita.

Per riprendere il flusso in un ambiente con GitHub CLI:

```text
gh auth login
```

Successivamente il preflight potrà verificare l'owner, assicurarsi che
`OWNER/SwissMath` non sia un repository estraneo e creare il repository pubblico
con branch `main`, homepage del Site e i topics `rust`, `mathematics`,
`number-theory`, `computational-mathematics`, `webassembly`, `wasm` e `tauri`.

## Archivi sorgente

Sono previsti e verificati due ZIP, ciascuno con un solo prefisso top-level e
senza `target/`, `work/`, `release/`, `dist/`, `.git` o output binari:

- `release/SwissMath-v0.5-source.zip`;
- `release/SwissMath-Web-v0.1-source.zip`.

Gli archivi sono artefatti locali e non vengono committati.

## Omissioni deliberate

Questa pubblicazione iniziale non crea Release, tag, asset allegati, GitHub
Pages, GitHub Actions/CI, template issue, `CONTRIBUTING.md`,
`CODE_OF_CONDUCT.md`, `SECURITY.md`, Dependabot o `CITATION.cff`.
