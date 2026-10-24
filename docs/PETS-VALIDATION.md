# Validação da composição dos pets — 24/10/2026

A composição anterior foi publicada no commit assinado [6798584](https://github.com/PedroMglo/coucou/commit/67985842fd904296c7c82aee55d2e29a1ef9cfb1),
branch `feat/mochi-cli-accounts`. A assinatura SSH ED25519-SK foi verificada
localmente. Esta composição é desenvolvimento integrado; a aplicação instalada
continua na `0.5.1-mochi.30` e o objetivo completo permanece aberto.

| Verificação | Resultado | Âmbito |
| --- | --- | --- |
| Fontes da composição anterior | 759 | Snapshot e mapa da publicação6798584 preservados |
| Inputs compilados | 400 nativos; 774 na composição completa | Snapshot imutável; inputs iguais antes/depois dos comandos |
| Rust | 2373 passaram; 20 ignorados | Biblioteca completa, com fixtures privadas |
| JavaScript | 1153 passaram | Bridge, controladores e controlos funcionais com DOM/IPC simulado |
| cargo check / TypeScript / Vite | Passaram | Mesma composição de fontes |
| Ensaios privados no anfitrião | 277 passaram | Runtime 237, leitor 9, operações 28, voz determinística 2, registo/admissão 1 |
| Avisos | 48 no check; 41 nos testes | Sem novos avisos |
| Modelos e quota Copilot reais | 0 | Protocolos/HTTP/bus privados e respostas sintéticas |
| Perfil original / instalação / rádio / PostgreSQL | Sem alterações | Original .30 preservado |

Os cinco helpers privados do runtime são invocados explicitamente pelos testes
pais. Os restantes testes ignorados não são declarados como aceitação satisfeita.
O fixture `legacy30_memory.rs` conserva os bytes históricos, incluindo a linha
vazia final, porque os testes verificam o seu hash com `include_bytes!`.

Mapa da composição anterior: SHA-256 `da1929bd5652695a8aa9e465d1a4508c7b1a6322d75aab9fb2da8ab812f38f45`. O snapshot e os logs completos
permanecem locais; o resumo versionado identifica os resultados e os limites.

## Planeamento integrado769

O incremento está integrado na branch `feat/mochi-cli-accounts`, com os mesmos
bytes validados no SDK e no snapshot congelado. A política/consentimento próprios,
o writer limitado e os recibos autenticados evoluíram para publicação canónica
com gate do controlador original, zero workers, CAS, readback durável e ACK no
journal. A recuperação preserva o orçamento e a autorização originais.

| Verificação | Resultado | Âmbito |
| --- | --- | --- |
| Fontes | 769 | Raiz, SDK e mapa com bytes iguais |
| Inputs | 410 nativos; snapshot de 784 ficheiros | Bytes iguais antes/depois dos comandos |
| Rust | 2401 passaram; 20 ignorados | Biblioteca completa, incluindo 10 casos novos de publicação/recuperação |
| cargo check | Passou | Mesmas fontes |
| Anfitrião privado | 305 passaram | Runtime 265, leitor 9, operações 28, voz determinística 2, registo/admissão 1 |
| Avisos | 77 no check; 41 nos testes | Caminhos ainda sem ativação pública; sem novos avisos nos testes |
| Web | Fontes inalteradas | 1153 JS/TSC/Vite herdados da composição759; não repetidos neste incremento apenas Rust |
| Modelos/quota reais | 0 | Protocolos/HTTP/bus privados e respostas sintéticas |

Os casos novos cobrem preparação, gravação e ACK incertos, falta do gate original,
workers ativos, deriva da origem, remoção de recibos/histórico/referências,
substituição de conteúdo e Stop/drain durante publicação. Abandonar uma preparação
exige ausência canónica e concessão encerrada; recuperar um plano já guardado
estabiliza o mesmo ACK sem reabrir a concessão. O teste de falha anterior à
publicação do recibo também está coberto.

O executor continua a recusar efeitos para planos gerados enquanto faltarem os
novos âmbitos de passo/receita. Também faltam a ativação operacional e o dispatcher
com feedback/replaneamento nativo. Não é aceitação de app/GUI/áudio, paridade real
dos providers, migração/backup/rollback ou entrega. A aplicação .30, os dados e
preferências originais, host, Wi-Fi e PostgreSQL não foram alterados; o objetivo
completo e a unidade RUN/EXE permanecem abertos.

Mapa local: SHA-256 `02240c7e5234c63b4f7404a0d12f753cbcc680efc89b559349d67564f6978190`.
Evidência local: `.build/pet-goal-execution-next-r1/`, snapshot
`canonical-plan-r6-source`, mapas `source-canonical-plan769-r6.json`, logs
`canonical-plan-full-r6`/`canonical-plan-check-r6` e
`HOST-NATIVE-CANONICAL-PLAN-R6.json`. O anterior núcleo privado764 fica preservado
em `PLANNING-EVIDENCE-PRIVATE-CHECKPOINT-R3.json` (2391 Rust/295 ensaios privados).

## Evidencia local

Os caminhos `.build/` são relatórios, snapshots, pré-imagens e recibos locais,
intencionalmente fora do Git. As ligações dos documentos atuais apontam para este
resumo para que sejam consultáveis no remoto. O arquivo histórico de arquitetura
conserva os seus links e bytes originais. Uma referência local histórica não
substitui a prova corrente nem demonstra entrega.

<details><summary>Referências locais preservadas</summary>

- `../.build/multi-attachments-next-r1/CONTINUATION.json`
- `../.build/multi-attachments-next-r1/COPY-PURGE-CONTRACT.json`
- `../.build/multi-attachments-next-r1/WORKING-CHECKPOINT.json`
- `../.build/multi-attachments-next-r1/sdk/windows/src-tauri/pets.default.json`
- `../.build/multi-attachments-next-r1/sdk/windows/src-tauri/src/assistant_memory.rs`
- `../.build/multi-attachments-next-r1/sdk/windows/src-tauri/src/assistant_tasks.rs`
- `../.build/multi-attachments-next-r1/sdk/windows/src-tauri/src/chat.rs`
- `../.build/multi-attachments-next-r1/sdk/windows/src-tauri/src/codex_chat/mod.rs`
- `../.build/multi-attachments-next-r1/sdk/windows/src-tauri/src/copilot_chat/mod.rs`
- `../.build/multi-attachments-next-r1/sdk/windows/src-tauri/src/gravity_session/mod.rs`
- `../.build/multi-attachments-next-r1/sdk/windows/src-tauri/src/local_tasks/executor.rs`
- `../.build/multi-attachments-next-r1/sdk/windows/src-tauri/src/local_tasks/run.rs`
- `../.build/multi-attachments-next-r1/sdk/windows/src-tauri/src/local_tasks/types.rs`
- `../.build/multi-attachments-next-r1/sdk/windows/src-tauri/src/pet_config.rs`
- `../.build/multi-attachments-next-r1/sdk/windows/src-tauri/src/pet_documents.rs`
- `../.build/multi-attachments-next-r1/sdk/windows/src-tauri/src/pet_documents/copy_retention.rs`
- `../.build/multi-attachments-next-r1/sdk/windows/src-tauri/src/pet_documents/purge_quarantine.rs`
- `../.build/multi-attachments-next-r1/sdk/windows/src-tauri/src/pet_factory.rs`
- `../.build/multi-attachments-next-r1/sdk/windows/src-tauri/src/pet_observability/process_census.rs`
- `../.build/multi-attachments-next-r1/sdk/windows/src-tauri/src/pet_observability/usage.rs`
- `../.build/multi-attachments-next-r1/sdk/windows/src-tauri/src/pet_sources/mod.rs`
- `../.build/multi-attachments-next-r1/sdk/windows/src-tauri/src/pet_sources/model.rs`
- `../.build/multi-attachments-next-r1/sdk/windows/src-tauri/src/session_chat/mod.rs`
- `../.build/multi-attachments-next-r1/sdk/windows/src-tauri/src/voice/local_task.rs`
- `../.build/multi-attachments-next-r1/sdk/windows/src-tauri/src/voice/session.rs`
- `../.build/multi-attachments-next-r1/source-retention-native-working-r2.json`
- `../.build/pet-capabilities-next-r1/WORKING-CHECKPOINT.json`
- `../.build/pet-data-compatibility-next-r1/LAYOUT-DECISION.json`
- `../.build/pet-data-compatibility-next-r1/LEGACY30-SOURCE-AUDIT.json`
- `../.build/pet-data-compatibility-next-r1/WORKING-CHECKPOINT.json`
- `../.build/pet-data-publication-next-r1/STARTUP-OWNERSHIP-CONTRACT.json`
- `../.build/pet-data-publication-next-r1/WORKING-CHECKPOINT.json`
- `../.build/pet-data-publication-next-r2/PUBLICATION-CONTRACT.json`
- `../.build/pet-data-publication-next-r2/WORKING-CHECKPOINT.json`
- `../.build/pet-data-staging-next-r1/LEGACY30-REFERENCES-AUDIT.json`
- `../.build/pet-data-staging-next-r1/STAGING-DESIGN.json`
- `../.build/pet-data-staging-next-r1/WORKING-CHECKPOINT.json`
- `../.build/pet-factory-native-next-r1/WORKING-CHECKPOINT.json`
- `../.build/pet-goal-effect-recovery-next-r1/ACCEPTANCE-MATRIX.json`
- `../.build/pet-goal-effect-recovery-next-r1/CONTRACT.json`
- `../.build/pet-goal-effect-recovery-next-r1/WORKING-CHECKPOINT.json`
- `../.build/pet-goal-execution-next-r1/BOUNDED-PLANNING-NEXT-CONTRACT-R1.json`
- `../.build/pet-goal-execution-next-r1/CONTINUATION.json`
- `../.build/pet-goal-execution-next-r1/CONTRACT.json`
- `../.build/pet-goal-execution-next-r1/OPERATIONAL-GOALS-ACCEPTANCE-R1.json`
- `../.build/pet-goal-execution-next-r1/PHASE1-KERNEL-ACCEPTANCE.json`
- `../.build/pet-goal-execution-next-r1/PHASE2-CRITERIA-ACCEPTANCE.json`
- `../.build/pet-goal-execution-next-r1/PHASE2-CRITERIA-AND-APPROVAL-CONTRACT.json`
- `../.build/pet-goal-execution-next-r1/PHASE2-CRITERIA-INTEGRATED-CHECKPOINT.json`
- `../.build/pet-goal-execution-next-r1/PHASE3-DECISIONS-ACCEPTANCE-R1.json`
- `../.build/pet-goal-execution-next-r1/PHASE3-DECISIONS-INTEGRATED-CHECKPOINT-R1.json`
- `../.build/pet-goal-execution-next-r1/PHASE3-GOAL-DECISION-ADOPTION-CONTRACT.json`
- `../.build/pet-goal-execution-next-r1/ROOT-APPLICATION-OPERATIONAL-GOALS-R1.json`
- `../.build/pet-goal-execution-next-r1/WORKING-CHECKPOINT.json`
- `../.build/pet-goal-steps-next-r1/STEP-EXECUTION-PROTOCOL.json`
- `../.build/pet-goal-steps-next-r1/WORKING-CHECKPOINT.json`
- `../.build/pet-history-compatibility-next-r1/WORKING-CHECKPOINT.json`
- `../.build/pet-native-policy-next-r1/CONTINUATION.json`
- `../.build/pet-native-policy-next-r1/DYNAMIC-NATIVE-OPERATIONS-AUDIT.json`
- `../.build/pet-native-policy-next-r1/LEGACY-MCP-COVERAGE-AUDIT.json`
- `../.build/pet-native-policy-next-r1/NATIVE-DECISION-MODES-CONTRACT.json`
- `../.build/pet-native-policy-next-r1/PHASE11-ACCEPTANCE-MATRIX.json`
- `../.build/pet-native-policy-next-r1/PHASE11-ADAPTER-AUDIT.json`
- `../.build/pet-native-policy-next-r1/WORKING-CHECKPOINT.json`
- `../.build/pet-operations-next-r1/HOST-NATIVE-OPERATIONS.json`
- `../.build/pet-operations-next-r1/WORKING-CHECKPOINT.json`
- `../.build/pet-owner-contracts-next-r1/WORKING-CHECKPOINT.json`
- `../.build/pet-plan-bindings-next-r1/HOST-NATIVE-PLANS.json`
- `../.build/pet-plan-bindings-next-r1/WORKING-CHECKPOINT.json`
- `../.build/pet-profile-acceptance-next-r1/NATIVE-CODEC-ACCEPTANCE.json`
- `../.build/pet-profile-acceptance-next-r1/PROFILE-CENSUS.json`
- `../.build/pet-profile-acceptance-next-r1/TYPED-PATH-CENSUS.json`
- `../.build/pet-profile-acceptance-next-r1/WORKING-CHECKPOINT.json`
- `../.build/pet-profile-backup-next-r1/NATIVE-BACKUP-RESTORATION.json`
- `../.build/pet-profile-backup-next-r1/WORKING-CHECKPOINT.json`
- `../.build/pet-profile-bootstrap-next-r1/BOOTSTRAP-CONTRACT.json`
- `../.build/pet-profile-bootstrap-next-r1/WORKING-CHECKPOINT.json`
- `../.build/pet-profile-resume-next-r1/WORKING-CHECKPOINT.json`
- `../.build/pet-retention-recovery-next-r1/WORKING-CHECKPOINT.json`
- `../.build/pet-retention-recovery-next-r1/recovery-increment-r4.patch`
- `../.build/pet-retention-terminal-next-r1/WORKING-CHECKPOINT.json`
- `../.build/pet-retention-terminal-next-r1/terminal-increment-r2.patch`
- `../.build/pet-root-integration-next-r1/ROOT-APPLICATION.json`
- `../.build/pet-root-integration-next-r1/TEXT-RECONCILIATION.json`
- `../.build/pet-runtime-admission-next-r1/WORKING-CHECKPOINT.json`
- `../.build/pet-runtime-authority-next-r1/WORKING-CHECKPOINT.json`
- `../.build/pet-runtime-leases-next-r1/CONTINUATION.json`
- `../.build/pet-runtime-leases-next-r1/WORKING-CHECKPOINT.json`
- `../.build/pet-runtime-native-validity-next-r1/WORKING-CHECKPOINT.json`
- `../.build/pet-runtime-native-validity-next-r1/validity-increment-r1.patch`
- `../.build/pet-runtime-next-r1/WORKING-CHECKPOINT.json`
- `../.build/pet-runtime-next-r1/sdk/windows/src-tauri/src/pet_runtime/adapter.rs`
- `../.build/pet-runtime-next-r1/sdk/windows/src-tauri/src/pet_runtime/model.rs`
- `../.build/pet-runtime-next-r1/sdk/windows/src-tauri/src/pet_runtime/store.rs`
- `../.build/pet-runtime-results-next-r1/HOST-NATIVE-RESULTS.json`
- `../.build/pet-runtime-results-next-r1/WORKING-CHECKPOINT.json`
- `../.build/pet-runtime-writer-next-r1/CONTINUATION.json`
- `../.build/pet-runtime-writer-next-r1/WORKING-CHECKPOINT.json`
- `../.build/pet-runtime-writer-next-r1/sdk/windows/src-tauri/src/codex_chat/rpc.rs`
- `../.build/pet-runtime-writer-next-r1/sdk/windows/src-tauri/src/gravity_chat/rpc.rs`
- `../.build/pet-runtime-writer-next-r1/writer-increment-r3.patch`
- `../.build/pet-session-states-next-r1/WORKING-CHECKPOINT.json`
- `../.build/pet-session-states-next-r1/session-states-review.patch`
- `../.build/runtime-context-next-r1/WORKING-CHECKPOINT.json`
- `../.build/sole-owner30/`
- `../.build/sole-owner31/CONTINUATION.json`
- `../.build/voice-capabilities-next-r1/WORKING-CHECKPOINT.json`
- `../.build/voice-capabilities-next-r1/sdk/windows/src-tauri/src/pet_capabilities/mod.rs`
- `../.build/voice-capabilities-next-r1/sdk/windows/src-tauri/src/voice/admission.rs`
- `../.build/voice-capabilities-next-r1/sdk/windows/src/core/pet-capabilities.ts`

</details>

## Reprodução e casos relevantes

As dependências e ambientes pertencem ao projeto. Os comandos de validação foram
executados com raízes privadas de XDG/dados e rede desligada no contentor, salvo
os testes de HTTP/bus em loopback privados no anfitrião. Os ensaios não devem usar
perfis pessoais ou quota real de Copilot/GitHub CLI.

Com as dependências do projeto preparadas, os comandos usados em `windows/` são:

```sh
cargo test --offline --lib -- --test-threads=4
cargo check --offline --lib
npm test
node node_modules/typescript/bin/tsc --noEmit
node node_modules/vite/bin/vite.js build
```

A imagem local de construção usada está fixada pelo digest
`824cea00ee675fdffb75a89ce50014aee772d58b6c0af7ca8d98c601b7f2e6b6`.
Os ensaios no anfitrião usam o mesmo binário de testes e verificam o hash das
fontes/binário e a permanência do executável .30 original antes/depois.

- [Registo de capacidades](../windows/src-tauri/src/pet_capabilities_tests.rs): atualização real do registo muda a revisão pública e conserva o âmbito semântico.
- [Admissão](../windows/src-tauri/src/pet_runtime/admission_tests.rs): observação renovada é aceite; alterações reais de modelo/perfil/decisão/callers/limites e política são recusadas.
- [Receitas e decisões](../windows/src-tauri/src/pet_runtime/recipe_tests.rs): origem própria, decisão única, cancelamento e gestão isolada das permissões dos terminais.
- [Critérios/conclusão](../windows/src-tauri/src/pet_runtime/criteria_tests.rs): cadeia nativa, adulteração, confirmação perdida, compatibilidade e recuperação sem repetir efeitos.
- [Metadados locais](../windows/src-tauri/src/pet_runtime/readiness_api.rs): apenas `/api/show` local, sem inferência; alvos cloud/não locais e cancelamento são recusados.
- [Controlador funcional](../windows/tests/runtime-goals.test.mjs) e [vista](../windows/tests/runtime-goal-view.test.mjs): chamadas explícitas, confirmação incerta, preservação de rascunhos, Stop e ausência de repetição automática.

## Aceitação que continua pendente

Faltam planeamento/replaneamento e despacho nativo completos, os restantes
executores/verificadores/adaptadores, paridade nativa integral de permissões e
sessões, memória/fontes, rotinas/cooperação, voz integral e matriz de Sistema.
O primeiro writer autónomo é Ollama local e o primeiro executor/verificador
físico cobre as operações Spotify suportadas.

Testes com DOM/IPC simulado não demonstram o ciclo completo da aplicação gráfica.
Microfone/TTS, providers reais, migração/backup/rollback dos schemas novos e
aceitação/entrega instalada permanecem necessários. Os 43 requisitos continuam
no [roadmap](PETS-ROADMAP.md), com o estado em
[PETS-IMPLEMENTATION-STATUS.md](PETS-IMPLEMENTATION-STATUS.md).
