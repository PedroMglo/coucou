# Desenvolvimento deste fork Linux

Este checkout destina-se apenas a Linux. O frontend partilhado e o backend Tauri
mantêm o nome de pasta `windows/`; isso não amplia o âmbito para Windows ou macOS.
O estado de entrega e os critérios de aceitação estão em
[docs/PETS-ARCHITECTURE.md](docs/PETS-ARCHITECTURE.md). As instruções upstream
sobre Swift/AppKit abaixo são referência histórica para o projeto original.

A fábrica de pets tem uma candidata privada revista, ainda não integrada.
Na sua adoção, novos pets e melhorias comuns devem usar base, molde e extensão;
o comportamento específico pertence à instância ou ao adapter. Evoluir os
moldes e migrar duplicações quando houver benefício concreto. Identidade,
provider, permissões e política de execução local permanecem explícitos por pet
e nunca são herdados dos moldes. O catálogo não concede acessos em runtime.

Conservar o destino completo das sessões e dos pedidos em todas as operações.
Apresentar um pedido não constitui aprovação. Testes simulados, compilação e
aceitação no desktop são evidências distintas; registar os seus limites no guia.
Alterações persistentes do anfitrião pertencem ao sysworkspace; dependências e
artefactos deste projeto permanecem neste repositório/ambiente.

---

# Contributing to Coucou

Thanks for wanting to help Mochi grow up! 🫶

## Getting started

```bash
brew install xcodegen
cd NotchBuddy && xcodegen && open NotchBuddy.xcodeproj
```

Never edit `NotchBuddy.xcodeproj` by hand: change `project.yml` and run `xcodegen`.

Check resting island dimensions on screens with and without a notch:

```bash
bash scripts/test-screen-geometry.sh
bash scripts/test-display-choice.sh
```

Check auto-close timing and live setting changes:

```bash
bash scripts/test-auto-close.sh
```

## Good first contributions

- A new service integration (a poller + an entry in `PillCatalog.swift` in the `.service` category + a detail card). Look at `StripePoller.swift` for a compact example.
- A new agent: any agent already gets its own automatic pill by sending `coucou_agent` in its hook payload (see `docs/AGENTS.md`). Add an entry in `PillCatalog.swift` in the `.agent` or `.workspace` category only if you want it to be declarable in Settings → Active pills.
- A new emote or sound for Mochi.
- Bug fixes — please describe how to reproduce.

## Rules of the house

- Swift 6, SwiftUI + AppKit, **no third-party dependencies** unless there's really no other way.
- Secrets go in the Keychain, never on disk or in git.
- No telemetry, no network calls except to services the user configured.
- Never block Claude Code: if the app doesn't answer, the hook must exit right away.
- Never write `~/.claude/settings.json` without a backup and the user's confirmation.
- Keep it light: 0 % CPU when the island is hidden.

## Pull requests

- One topic per PR, with a short GIF or screenshot for anything visual.
- Build must pass with no new warnings.
