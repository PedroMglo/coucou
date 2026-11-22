**14/11 — projeção móvel:** a retirada declarada na fábrica aplica-se também aos cartões e à seleção efetiva dos serviços do telefone, incluindo cache e seleções antigas; não apaga preferências/dados nem retira ferramentas GitHub separadamente autorizadas.

<!-- rule-probe-recovery-20261110 -->
**Infraestrutura comum10/11/2026:** a recuperação de prontidão pertence ao runtime/Authority e conserva binding/identidade por pet; não acrescenta permissões por molde. A fábrica e a projeção de compatibilidade não mudaram nesta unidade. A validação usa Ollama HTTP sintético, mantendo LM Studio adiado e os restantes contratos/limites dos providers. [Contrato, fontes e limites](PET-RULES.md#recuperação-de-prontidão10112026--aceitação-parcial).

<!-- rule-scheduled-dispatch-20261110 -->
**Prioridade10/11/2026:** preservar o adaptador LM Studio, adiar novo desenvolvimento/QA do pet porque o utilizador não o tem instalado; não é gate desta entrega. A recorrência automática opt-in é infraestrutura comum e conserva os bindings explícitos por pet; não promove grants manuais ou observadores. A prova atual usa Ollama HTTP sintético, sem declarar operacionais todos os pets. [Contrato, fontes exatas e limites](PET-RULES.md#agendamento-automático-explícito10112026--aceitação-parcial).

**Extensão em integração, 09/11/2026:** catálogo26 com System Guardian independente e sem motor/grants herdados. A configuração inclui catalogRevision1; cada identidade nova declara introducedIn na fábrica. O resolvedor nativo acrescenta em memória apenas identidades posteriores à revisão reconhecida, com permissões/modelos/ferramentas vazios. Não recria uma identidade ausente num catálogo já reconhecido, não aceita revisão futura e não substitui os25 perfis existentes. O snapshot pets.default.json é gerado pelo script existente. A consulta de Saúde usa o mesmo writer/captura revogável de Sistema, com grant próprio Health/Observe/HostHealth e controlo funcional no editor existente; não habilita inferência, TaskRuntime ou ProviderModel para Saúde. Ver [contrato Guardian](SYSTEM-GUARDIAN-ARCHITECTURE.md#12-primeira-unidade-operacional-de-saúde-09112026-em-implementação). As entradas anteriores conservam o contexto histórico.

# Pet factory

## Funcionalidades por domínio —13/11/2026, reafirmadas16/11

Por instrução do proprietário, recuperar uma funcionalidade do Coucou oficial
exige que ela faça sentido para o pet representado. Os moldes existentes
`llm` e `application` são a camada de domínio: `features.inference` pertence
apenas a essa camada. A base comum, as extensões de conta e os pets individuais
não podem emitir essa regra. A base conserva identidade, configuração, contexto,
memória, sessões históricas, objetivos e mecanismos comuns do runtime.

O domínio LLM admite conversa e modelos quando existe o adaptador/configuração
necessário. O domínio aplicação/serviço conserva as suas operações próprias;
não obtém conversa, modelos, métricas de inferência ou execução por um LLM de
uma extensão de conta. Spotify conserva os controlos nativos de música/som e
a voz determinística. n8n conserva a observação de execuções/workflows. Criar,
editar e executar workflows pelo pet continua a exigir implementação e prova
próprias; uma conta Codex não demonstra essas capacidades.

O resolvedor aplica o limite do domínio depois das extensões; UI, capacidades
nativas, admissão de conversa e listagem/seleção de modelos seguem esse limite.
As referências antigas de conta/modelo e os aliases de dono ficam preservados
para leitura e recuperação. Não há migração de preferências nem eliminação de
conversas: conservar dados históricos não ativa inferência no serviço.
A separação está implementada e tem prova nativa nas composições40/42, com os limites registados na auditoria de paridade. A instalação44/Source35 mantém
o comportamento anterior até uma entrega própria autorizada. Entradas antigas
abaixo que descrevem assistentes genéricos nos serviços são contexto histórico.

**R8 C2 F1 — fecho e continuação no mesmo backend integrados localmente1062 (03/11):** o DrainProof original sela o fecho no mesmo Grant13 antes de publicar o Goal19. ACK apenas publica o alvo exato já assinado. Reference4/Spec7/Goal20 permitem nova revisão sem modelo após fecho confirmado, preservando recibos e encargos; Closed sozinho, Uncertain, cancelamento e JSON não concedem continuação. Native343 comprovou dois fechos/três admissões no mesmo processo, UI, reinício/ACK e leitor C1 anterior; native344 comprovou restauro/ativação com o mesmo binário, apenas como dados históricos. [Contrato](PET-DETERMINISTIC-RUNTIME.md), [composição](../.build/pet-goal-execution-next-r1/integration1062-native-controlled-continuation-r8c2/FINAL-CHECKPOINT.json) e [provas/limites](../.build/pet-goal-execution-next-r1/NATIVE-CONTROLLED-CONTINUATION-R8C2-FINAL-VERIFICATION-F1.json). Segue C3: revisão explícita de planos/passos expirados sem reenviar os cobertos, por mecanismos comuns.43+6requisitos integrais preservados/abertos; instalação32 inalterada, sem commits/push/merge/host/quota. As composições anteriores abaixo são histórico.

**R8 C1 F1 — continuação após perda do executor integrada localmente1057 (03/11):** a revisão explícita sem inferência conserva recibos e encargos originais; os passos cobertos não são enviados novamente. Reference3/NativeCarry, Spec6/Goal18 e Report2/Grant12 reutilizam Recovery/Store/Authority/Owned, sem outro ledger, modelo, saldo renovado ou autoridade restaurada. Provas privadas: duas perdas reais de processo, três admissões do mesmo objetivo, revisão/recusa pela UI, leitor B anterior e restauro do produto. O restauro só conserva dados históricos. [Contrato](PET-DETERMINISTIC-RUNTIME.md), [composição](../.build/pet-goal-execution-next-r1/integration1057-native-continuation-r8c/FINAL-CHECKPOINT.json) e [provas/limites](../.build/pet-goal-execution-next-r1/NATIVE-CONTINUATION-R8C-FINAL-VERIFICATION-F1.json). Segue C2: retoma após fecho controlado com prova do drain e revisão explícita dos passos expirados, sem ModelAttempt fictício. Critérios livres, respostas/admissão do destinatário e delegação mantêm-se abertos;43+6requisitos integrais preservados. Instalação32 inalterada; sem commits/push/merge/host/quota real. As composições anteriores abaixo são histórico.

**R8 B F1 — conclusão nativa integrada localmente1056 (03/11):** a revisão original inclui os predicados nativos suportados antes de certificar a admissão. Spec5/Goal17 selam Binding2; Grant11 guarda o relatório imutável no mesmo ledger de autoridade. O mesmo `Store.complete_certified` verifica e publica a conclusão, com origem explícita em recibos locais e sem recipeId fictício, novo modelo, orçamento ou aprovação no fim. Critérios livres continuam Unknown; persistência local não verifica trabalho do destinatário. Native339 demonstrou quatro conclusões automáticas do remetente, parciais/critério livre/Stop/reinício/ACK e revisão funcional pela UI automatizada; native340 verificou leitor A4 anterior e backup/inspeção/restauro/ativação reais do produto, conservando os dados históricos e recusando a autoridade antiga. [Contrato](PET-DETERMINISTIC-RUNTIME.md), [composição](../.build/pet-goal-execution-next-r1/integration1056-native-completion-r8b/FINAL-CHECKPOINT.json) e [provas/limites](../.build/pet-goal-execution-next-r1/NATIVE-COMPLETION-R8B-FINAL-VERIFICATION-F1.json). Segue R8 C: recuperação/continuação explícita do modo nativo, sem renovar autoridade ou saldo.43+6requisitos integrais preservados/abertos; instalação32 inalterada, sem commits/push/merge/host/quota real. As composições abaixo são histórico.

**R8 A4 — bases integradas localmente1051 (03/11):** o mesmo runtime admite trabalho literal de `pet.message` sem modelo, readiness, autenticação ou reserva de inferência. Reutiliza Store/Review/Approved/Authority, fila, Owned, recursos, orçamento e recuperação. Spec4/Goal16, Proof3 `goalDeterministic`, Lease2 e Grant10 selam `runtimeOnly` com0chamadas de modelo; os formatos legados conservam as suas guardas. [Contrato](PET-DETERMINISTIC-RUNTIME.md), [composição](../.build/pet-goal-execution-next-r1/integration1051-deterministic-runtime-r8a4/FINAL-CHECKPOINT.json) e [provas/limites](../.build/pet-goal-execution-next-r1/DETERMINISTIC-RUNTIME-R8-A4-VERIFICATION.json). Native338 comprovou API/controlo funcional/reinício em Ollama, LM Studio, OpenAI e Spotify sem pedidos de metadata/inferência; native336 comprovou leitor R7 anterior e backup/inspeção/restauro/ativação com recibo histórico inativo. Não é aceitação humana, hardware, inferência real ou entrega instalada. Faltam verificação nativa dos critérios e continuação explícita deste modo; receber uma mensagem não conclui o objetivo. Respostas, admissão do destinatário e delegação continuam abertas.43+6requisitos preservados; instalação32 inalterada; sem commits/push/merge/host. As entradas de composições anteriores abaixo conservam o histórico; o próximo passo vigente é R8 B, verificação/recuperação determinísticas.

**R6 F1 — integração local1034:** `runtime.inferenceAdapters` declara motores permitidos por molde/identidade; `inference` e `inferenceRevision` pertencem apenas ao pet concreto, com alteração nativa por revisão. `pets.default.json` continua projeção inalterada de25identidades. Fábrica Rust/TypeScript e runtime mantêm o canal/dono lógico quando o motor muda; a declaração não concede acessos. Ver [contrato](PET-INFERENCE-BINDING.md) e [provas](../.build/pet-goal-execution-next-r1/integration1034-inference-binding-r6f1/CHECKPOINT.json).

Private candidate for the next Linux build; integration and native acceptance are coordinated separately from the hover/approval hotfix.

The bundled source is `windows/src-tauri/pets.factory.json`. It describes a shared base, two product molds (`llm` and `application`), optional named extensions, and stable per-pet overrides. Common changes are inherited rather than copied between Codex, Copilot, Gravity, Ollama or application pets.

Resolution order is **base → mold → extension → pet override**. Nested objects merge; arrays and scalar values replace. Each resolved blueprint is immutable. Editable configuration documents are independent clones.

## Ownership

- `base.preferences` and `base.presentation`: shared profile structure/mode and presentation. Stable identity, provider and access policy are prohibited here and in molds/extensions.
- `molds.llm`: LLM-family differences. Provider-specific models and local/native execution remain explicit.
- `molds.application`: application/service operations. Domain policy excludes LLM chat/model/session UI, even when a legacy account extension remains configured.
- `extensions.codex-account`: configured Codex account defaults. Historical references on service pets remain readable; this extension cannot grant inference or override the domain limit.
- `pets[id].preferences`: required explicit stable name/aliases, configured provider and initial integration grants; optional per-pet local-execution policy and model overrides. These protected fields cannot be inherited.
- `pets[id].routing`: explicit existing chat/voice/task adapter metadata; never inferred from a mold or an AI-looking name.
- `pets[id].presentation`: specific controls or card labels. Spoty keeps its music/audio route and two relevant access fields.

Service pets remain application pets even when they retain a historical Codex account reference. Only integration_github is temporarily retired; its data stays preserved. Spoty remains application-only: no new LLM chat, model picker, task execution or session route is created by the factory.

The TypeScript resolver is `windows/src/core/pet-factory.ts`. Current consumers are provider lookup, task ownership/names, profile lookup, capabilities, the pet gallery and pet settings. Gallery avatar size, radius, gap, context action, metrics and labels come from the same resolved presentation. The existing shared chat/approval renderers, hover logic, native adapters and habitat remain in their existing modules. The overview avatar slot is reserved metadata and is not wired in this candidate, to avoid the concurrent hotfix.

The native resolver in `windows/src-tauri/src/pet_factory.rs` resolves **configuration defaults only**, caches the bundled result once and returns clones through `pet_config::defaults()`. It does not consume frontend presentation/routing as authorization. The resolved document retains the existing version1 schema and all25 current default identities, models and access values. Both resolvers and the snapshot generator reject inherited name/aliases/provider/permissions/localExecution; every pet explicitly declares its identity/provider/grants.

## Making a common change

Change the shared base for every pet, or the relevant mold for that family. For example:

```json
{
  "molds": {
    "llm": {"presentation": {"galleryCardRadius": 17}},
    "application": {"presentation": {"profiles": false, "sessions": false, "metrics": false, "contextAction": "open"}}
  }
}
```

An instance can override one field without copying its mold. An application adapter cannot obtain LLM chat merely by opting into an account extension. A real LLM domain capability requires its own reviewed contract and implementation. A new integration still needs its actual native adapter, platform/catalog entry and any provider contract registration; a template does not manufacture implementation or credentials.

Model defaults are specific to each provider or the named account extension. Changing a generic mold never swaps a pet's CLI/account. Runtime saved names, aliases, model choices, grants and local-execution policies retain their existing read/save/validation paths; the factory performs no bulk rewrite or migration of user files.

`pets.default.json` remains a compatibility snapshot for existing tooling/tests, not the production default source. After deliberately reviewing a bundled default change:

```sh
node scripts/generate-pet-defaults.mjs --check
node scripts/generate-pet-defaults.mjs
```

The first command checks semantic equality; the second regenerates only that project snapshot. Neither reads or writes HOME. Tests compare the native and frontend resolved defaults with the legacy snapshot, including Ollama's explicit local-execution policy and all integration grants.

## Boundaries

Molds describe product reuse. Native original request authority, provider/session identity, local-execution approvals, application permissions and service capabilities remain enforced in their reviewed adapters. A public class/capability, a template change or a routing label cannot grant access or authorize an operation.

No existing stable ID is renamed. No model/account/device/GUI operation, global installation or host configuration is needed to resolve or test the factory. The initial candidate intentionally preserves the current appearance and default values. Future common styling changes can use the inherited presentation fields without editing each pet.

## Team input and next shared seams

The integration, frontend and backend owners contributed the following boundaries. This candidate covers the configuration/catalog/lookup/gallery/settings slice above. The remaining seams should be migrated in successors after the independent hover/approval hotfix; they are not claimed as implemented here.

| Shared seam | Preserve when moving it into the factory |
| --- | --- |
| Identity, appearance and catalogue | Stable ID and canonical configured name; aliases remain identity-specific. Default colour/avatar/outfit and platform visibility can be deduplicated, while a native dated snapshot continues to describe observed state. |
| State and card shell | One vocabulary for idle, working, needs approval, needs information, finished and error; shared header action slots for Sessions, Configure and Close, plus scrolling/content slots. Approval/question ownership must win over home/terminal metadata. Do not duplicate or modify the concurrent hover/header/overview/CSS hotfix. |
| Asynchronous feedback | Carry exact pet + runtime target + request, fence late results and retain current focus. A template must never invent a session ID, conversation ID, source or native authority. |
| LLM conversation | Fast/reasoning/auto structure is reusable; actual model availability is provider-specific and an explicit selected model wins. Chat, history and drafts retain the full runtime SessionTarget and origin. Native global history is not the pet's library; observed terminals are not active turns; unknown usage remains null rather than false zero. |
| Application extension | Explicit app actions and attachment/plugin slots; Spoty owns its own music/PC-audio controls. MPRIS observation is not System permission. The application domain excludes generic LLM chat/model/session controls; a historical account reference cannot grant them. Existing history stays preserved. |
| Settings and drafts | Preserve provider-specific dirty/version guards, retained drafts and unknown/present/absent secret state; saving or displaying a mold never opens an approval or starts work. |
| Voice | Obtain the canonical current config.name for each request. Do not replace speech words, force a language, or turn learned recognition aliases into the spoken owner name. Current voice and lexicon implementations are unchanged. |
| Runtime access | Public capability/availability and a dated native snapshot are observations, not authorization. System operations retain original Prepared, current context/fence, owner/producer, TTL and actual writer checks. Always Allow remains untouched; email stays Ollama-only by default. |

No live account, installed-model or runtime capability is inferred from a factory entry. Routing metadata replaces existing static UI tables only. Logical owners are unique per chat/voice/task channel; sharing a physical Codex account never aliases GitHub's logical pet. Unknown or malformed provider lookups fail closed rather than matching an absent route.

## Sistema por pet —09/11/2026

`systemAccess` não pode ser emitido pela fábrica, herdado de moldes/extensões ou sobrescrito pelo writer legado do perfil. A gestão específica cria revisões por pet; observar/volume/silêncio de saídas é projetado apenas para NativeUser, sem depender de modelo e sem anunciar prontidão por mera presença do grant. TaskRuntime/ProviderModel, legado e restantes subplugins conservam as suas dependências. [Contrato, implementação e prova](PET-SYSTEM-ACCESS-CONTRACT.md).

A identidade para trabalho nativo passa por `Blueprint::runtime_identity_adapter`. Um pet sem adaptador base conserva o seu namespace mesmo quando permite motores explicitamente configuráveis. Um binding explícito incompatível é recusado; esta resolução não escolhe um motor nem concede autoridade. Prova dirigida: `runtime-unbound-identity/VERIFIED.json` no checkpoint privado vigente.

## Coleções de calendário — adoção comum09/11/2026

A gestão usa `buildCalendarSources` nos dois compositores geridos e o broker comum `pet_sources`; não há implementação ou permissões especiais para Codex/Ollama. Identidade é PetId/CID da fábrica, distinta de conta/modelo, e a resolução não ativa modelos. O suporte de gestão NativeUser é separado da consulta realmente disponível no adaptador/turno. [Contrato e limites](PET-CALENDAR-SOURCES.md).

### Recuperação OpenAI-compatible —12/11, validação em curso

A fábrica passa a declarar `custom` nos `inferenceAdapters` dos dois pets que
já admitem Ollama/LM Studio. É uma escolha explícita do motor da conversa, com
servidor/modelo/referência de credencial próprios; não é outra conta nativa nem
outro pet. As escolhas e grants continuam proibidos nos templates, e os defaults
serializados, identidades, aliases e permissões mantêm-se. Guardian e contas
Codex/Copilot/Gravity conservam os contratos anteriores.

O editor e a revisão comuns fazem a escolha; a ligação global nas definições
não a distribui aos pets. No modo de pets, “Configurar num pet” abre essa gestão.
Desligar uma ligação global também não apaga uma escolha explícita do pet; a
remoção da chave impede envios que a exijam. O transporte original compatível é
reutilizado. O catálogo não promove capacidades de ferramentas ou tarefas
apenas por aceitar HTTP OpenAI-compatible. As novas variantes são recusadas por
leitores anteriores; a preservação e a janela nativa estão ainda por validar.

### Roster por voz: continuação da própria gravação

Os nomes continuam a ser resolvidos pelo catálogo e controlos existentes. Nos
turnos de voz, a alteração é agora aplicada pelo mesmo CAS nativo do roster,
sob o writer de preferências, com a origem capturada verificada antes da
persistência. Só o commit confirmado dessa operação pode avançar a testemunha
do próprio turno, se o pet que responde continuar ativo. Retirar esse pet não
autoriza outra ronda de inferência ou reprodução. Identidade, modelo, conversa, configuração dos pets e seleção
de áudio continuam capturados; uma gravação geral, substituição do dono,
alteração externa, cancelamento anterior ao despacho ou falha de persistência
não renova a captura. A resposta usa o recibo normal do broker e o histórico
existente. Não há novo formato persistente, grant ou ledger. A validação nativa
e a experiência instalada têm gates separados; consultar os recibos atuais.

### Rotinas com notas de projeto fixadas — contrato17/11/2026

A camada de domínio de inferência pode compor uma rotina local Ollama com memória
de projeto. O runtime comum conserva scheduler, autoridade, reserva, execução e
persistência; não dá chat ou inferência aos pets de serviços. A seleção contém
um projeto, CID público original e revisões exatas de notas desse projeto.
A revisão tem consentimento próprio de repetição, além da recorrência e do seu
saldo finito; consentimentos manuais ou de objetivo único não são promovidos.
Cada ocorrência usa uma captura, uma chamada e zero ferramentas. Mudanças de
projeto, associação, nota ou destino bloqueiam novos usos; edição/revisão/sucessão
existentes são a via de atualização. O contrato não autoriza escolher futuras
revisões automaticamente nem considerar critérios cumpridos por haver resposta.
A implementação está em validação privada; não certifica instalação ou AUTO04.


### Acompanhamento dos objetivos do projeto — contrato17/11/2026

A camada de domínio de inferência local Ollama pode escolher objetivos próprios
por identidade/definição, com última resposta finita opcional e notas opcionais.
O consentimento próprio de progresso futuro não deriva de chat, objetivo único
ou notas fixadas. A fábrica não acrescenta esta superfície a aplicação/serviço,
n8n ou observador sem inferência; o Guardian conserva apenas a autoridade revista.
A base comum transporta os dados e mantém scheduler/autoridade/execução/recuperação,
sem transformar a opção do domínio em capacidade universal dos pets.

Mudanças normais entram na próxima fotografia, não refrescam a atual. A resposta
continua dependente dos dados originais que a produziram; não permite encaminhar
texto revogado, executar sugestões ou verificar critérios por inferência.
[Contrato, provas atuais e limites](PETS-PROJECT-FOLLOWUP-2026-11-17.md).


### Pedido de decisão como dados de projeto — domínio local,17/11

A faceta de acompanhamento local Ollama/Context12/Admission32 pode formular
uma pergunta limitada quando o formato foi escolhido na intenção revista da
rotina. A resposta do utilizador é uma nota ordinária de projeto, com revisão
própria; não concede execução, conclusão, partilha ou inferência. Esta condição
é verificada no backend por origem/recibo/contexto, além da UI. Não se acrescenta
uma capacidade universal à base dos pets, chat a serviços/n8n ou domínio de
inferência a observadores. O runtime/persistência/CAS existentes são reutilizados.
Outros motores mantêm os seus percursos anteriores; não há fallback de provider.
[Provas e limites](PETS-PROJECT-DECISION-2026-11-17.md).
