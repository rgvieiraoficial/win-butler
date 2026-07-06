# CLAUDE.md — WinButler (fonte de verdade técnica)

App desktop de limpeza/manutenção do Windows 11. Tauri v2 (Rust) + React/Vite/TS/Tailwind.

## Arquitetura
- **Frontend** (`src/`): React 18 · Vite 5 · TS · Tailwind 3 (dark, acento ciano `#22d3ee`).
  - `lib/api.ts` — única camada que chama `invoke`. Toda ação destrutiva recebe `dryRun`.
  - `lib/types.ts` — espelha os structs Rust (serde camelCase). **Manter em sincronia.**
  - `hooks/useAsync.ts` (load+reload), `hooks/useRunAction.ts` (confirmar→executar→notificar).
  - `components/ConfirmDialog.tsx` — provider com `confirm()` (modal) e `notify()` (toast).
  - `sections/` — uma tela por seção; `App.tsx` esconde seções conforme `Capabilities`.
- **Backend** (`src-tauri/src/`): comandos privilegiados via `std::process::Command`.
  - `util.rs` — `ActionResult`, `run()`, `powershell()`, `powershell_json()` (UTF-8, `CREATE_NO_WINDOW`).
  - `log.rs` — log JSONL em `%LOCALAPPDATA%\WinButler\winbutler.log`.
  - `commands/` — `detect`, `dashboard`, `disk`, `docker`, `creds`, `env`, `startup`, `schedule`.
  - `lib.rs` — registra os comandos + **modo headless** (`--run <rotina>` para o Agendador).

## Regras rígidas (do escopo — não violar)
- Ação destrutiva → **confirmação com explicação** (usar `confirm` do `useRunAction`).
- **Nada executa ao abrir.** Sem efeitos colaterais em mount além de leitura (listagens/dashboard).
- **Dry run** em toda ação possível (o backend trata `dry_run` antes de qualquer efeito).
- Seção some se a capability não existir (`detect_capabilities`).
- **Sem telemetria.** Nenhuma chamada de rede. Log só local.
- Toda ação de escrita chama `log::record(...)`.

## Convenções
- Comentários e UI em **PT-BR**; nomes de código/rotinas em inglês (`disk_cleanup`...).
- Rust params snake_case ↔ JS invoke camelCase (conversão automática do Tauri).
- Structs expostos usam `#[serde(rename_all = "camelCase")]`; ao adicionar campo, atualizar `types.ts`.
- Contrato de retorno de ação = `ActionResult { ok, dryRun, message, output, details }`.

## Comandos
```bash
yarn install
yarn build      # tsc -b && vite build — VERIFICAÇÃO de tipos (rodar antes de commitar)
yarn app:dev    # app completo; rodar em terminal ELEVADO (admin)
yarn app:build  # instalador NSIS
```
Sem suíte de testes. Verificação da UI = `yarn build` passar. Backend exige Rust toolchain instalado (não estava presente na criação do projeto).

## Pegadinhas
- `manage-bde`, `diskpart`, `reg HKLM`, `schtasks /rl HIGHEST` **exigem elevação**. Em dev, o `tauri dev` só é elevado se o terminal for elevado. Em release, o manifesto força UAC.
- Saída de `cmdkey`/`manage-bde` é **localizada** — parsing tolera rótulos PT e EN.
- `cleanmgr /sagerun` roda assíncrono; o "espaço recuperado" medido pode subestimar.
- Compactar vhdx: precisa do WSL encerrado e Docker Desktop fechado, senão `diskpart` falha (arquivo em uso).
- Toggle de startup usa os bytes de `StartupApproved` (mesmo mecanismo do Gerenciador de Tarefas): byte[0]=2 habilitado, 3 desabilitado.
- Ícones em `src-tauri/icons` são placeholders gerados (.NET). Trocar por arte final.

## Git
- Identidade global: `yami.hiei87@gmail.com` (regra global do usuário). Nunca commitar/push sem ordem expressa.
