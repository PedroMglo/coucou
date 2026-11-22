# Contrato de identidade e capacidades dos pets — v1

**Atualização de estado R43F1A — 25/10/2026:** no SDK privado841 passou a preparação durável de memória selecionada para objetivos: referências exatas por pet/conversa de origem, contexto versão2 e objetivo versão11 apenas quando há seleção, sem copiar os dados brutos para o diálogo inicial. Passaram 2612 testes Rust/21 ignorados, check/build e o percurso Tauri privado130, incluindo reinício, revogação e recusa do leitor840 sem alterar os ficheiros. Não houve chamadas a modelos ou quota. A admissão autónoma com memória permanece deliberadamente recusada até R43F2 ligar revisão humana dos dados/destino, consentimento próprio do objetivo e mensagem anterior ao hash/reserva canónicos. A raiz aceite permanece834; a app instalada permanece .30. R43 e os 43 requisitos continuam abertos.

**Estado: PET-01 em desenvolvimento; núcleo e adoção inicial por chat/voz/gestão
integrados na composição759, com sessões/decisões nativas, primeira origem própria
de receitas em chat/voz e entrada funcional de objetivos em Tarefas, ainda sem
entrega desta composição.** Complementa a
[arquitetura](PETS-ARCHITECTURE.md) e orienta o primeiro incremento do
[roadmap](PETS-ROADMAP.md). Este ficheiro não é uma descrição de um endpoint já
instalado nem concede autoridade de execução. O comando de leitura e a bridge
tipada já existem nas fontes integradas. Há produtores a partir de histórico nativo e
adoção inicial pelo chat comum, preparação da voz e observabilidade da gestão;
faltam cobertura e adoção completas e aceitação
nativa para fechar PET-01.

**Composição830:** a continuação comum Codex/Gravity/Copilot conserva
a sessão guardada e recusa mudanças incompatíveis de pasta/intenção antes do
prompt. Não anuncia uma nova sessão como continuação da anterior nem interpreta
`full_access` histórico como política atual. A retoma Gravity também não reenvia
bypass; Copilot mantém ACP. A comparação nativa privada cobre três adaptadores
e um pet de serviço assistido por Codex; não acrescenta execução autónoma limitada
às CLI ou adoção de terminais externos. Os descritores continuam sujeitos aos
limites reais de cada provider. [Provas](PETS-VALIDATION.md#continuidade-local830).

**Composição corrente834 — gestão humana de memória:** as APIs nativas
`pet_memory_list` e `pet_memory_mutate` usam identidade da fábrica, âmbito do
pet/conversa, revisões e preservação comum. O catálogo não exige carregar um
motor; os 24 pets não retirados foram testados com isolamento. Criar memória numa
conversa vazia conserva o CID/arquivo sem fabricar mensagens. Esta API de gestão
não é ferramenta de modelo nem inclusão automática de memória nos pedidos.
`memory.retrieve/manage` nos motores e a sua projeção por caller exigem adoção
comum de contexto/consentimento ainda pendente em R43; não promover essa capacidade
apenas porque existe armazenamento. [Provas](PETS-VALIDATION.md#memoria-local834).
O protótipo privado840/E1 tem contexto selecionado por decisão humana nos dois
motores locais e nas três CLI/serviços, incluindo revogação durante decisões e
IO próprio. Os testes das contas CLI são protocolos sintéticos; a revisão declara
que a conta real não foi observada e conserva a autoridade do agente original.
Ainda não promove `memory.retrieve/manage` nos motores: faltam aceitação HTTP API,
admissão autónoma, confirmação humana funcional e projeção exata por caller.
[Âmbito em validação](PETS-VALIDATION.md#contexto-memoria-r43).


## 1. Objetivo e origem dos dados

Fornecer um descritor efetivo de cada pet que possa ser consumido por conversa,
voz, fila, planeador, gestão e ferramentas. O mesmo pet deve apresentar a mesma
identidade e limitações em todas estas superfícies.

A leitura do descritor consulta snapshots de configuração e caches nativos já
existentes. Não inicia inferência, autenticação, ligação/religação a contas,
carregamento de modelos ou serviços. Atualizações ativas são operações separadas
com intenção e autorização próprias.

Usar `pet_factory` como origem de moldes/IDs e `pet_config` como origem da
configuração validada. A fábrica atual não confere grants ou execução. O novo
registo não cria uma segunda fonte de preferências nem altera `provider` para
contornar a sua validação de identidade.

## 2. Forma do descritor público

| Campo | Tipo e regra |
| --- | --- |
| `schemaVersion` | Inteiro `1`; versões desconhecidas são recusadas |
| `petId` | ID estável do catálogo existente |
| `kind` | `llm`, `service` ou `observer`; papel resolvido pela implementação |
| `name` | Nome configurado; apresentação, não identidade/autoridade |
| `lifecycle` | `active`, `inactive` ou `retired`; distinto de estar ligado a uma conta |
| `binding` | Identificação pública do adaptador/provider e do modelo/perfil quando resolvidos; sem credenciais/identidade privada da conta |
| `revision` | Revisão opaca do snapshot, vinculada à instância atual do backend |
| `observedAt` | Instante da observação efetiva ou `null` quando desconhecido |
| `configurationAvailable` | Indica se foi lida configuração existente e validada; identidades bundled não fingem que as preferências pessoais foram carregadas |
| `capabilities` | Coleção limitada e sem nomes duplicados de capacidades tipadas |
| `concurrency` | Limites e âmbito efetivamente impostos pelo backend: sessão/provider/recurso; desconhecidos mantêm-se desconhecidos |

O descritor é uma projeção de estado, não um token de autorização. Uma revisão
em RAM após reinício não é importada como autoridade da instância anterior.
A revisão pública não expõe nem é construída a partir de segredos ou identificadores
privados de conta. Revalidar política e recursos ao admitir/executar trabalho.

### 2.1 Binding e origem

O binding distingue o adaptador implementado, a configuração necessária e o
estado observado. Um `provider` vazio nos defaults não é substituído silenciosamente
por Codex. Perfis/API ainda sem ligação ficam com motivo explícito.

Uma integração de serviço que já usa Codex conserva a identidade própria da
integração e o contexto dela. Um modelo oferecido pelo Gravity continua dentro
desse pet. Modelos locais mantêm endpoint e modelo escolhidos; não passar para
cloud porque o modelo local não suporta uma ferramenta.

## 3. Estado de cada capacidade

| Campo | Tipo e semântica |
| --- | --- |
| `id` | Nome estável de capacidade/operação, validado no registo |
| `support` | `supported`, `unsupported` ou `unknown`; suporta esta implementação/modelo? |
| `availability` | `ready`, `unavailable` ou `unknown`; pode ser admitida no contexto observado? |
| `reason` | Código tipado quando não está pronta ou há limitação; não transportar erro nativo com dados privados |
| `source` | `bundledManifest`, `configuration`, `nativeHandshake`, `providerCatalog`, `runtimeObservation` ou combinação com proveniência explícita |
| `observedAt` / `expiresAt` | Datas da evidência/frescura, ou `null`; uma data de cache não é a data de publicação do provider |
| `modelBinding` | Modelo/revisão a que a capacidade depende; `null` para capacidades independentes do modelo |
| `limits` | Limites tipados de entrada/saída, tempo, ferramentas e concorrência quando conhecidos |
| `limitations` | Limitações tipadas da rota, incluindo PDF sem OCR, execução sem sandbox ou suporte nativo ainda por verificar |
| `decision` | `none`, `nativeApproval`, `exactCommandApproval` ou `scopedGrant`; descreve a política aplicável, sem a aprovar |
| `callers` | Superfícies admitidas: `nativeUser`, `taskRuntime`, `providerModel`; projeção do contrato, sem criar uma concessão |

Motivos mínimos: `notImplemented`, `unsupportedPlatform`, `notConfigured`,
`inactivePet`, `retiredPet`, `authenticationRequired`, `providerUnavailable`,
`modelUnresolved`, `modelUnsupported`, `nativeApprovalUnavailable`,
`ambiguousTarget`, `staleObservation`, `busy`, `permissionDenied`,
`policyDenied`, `budgetUnavailable`, `unknown`.
O núcleo atual também distingue `targetRequired` e `libraryUnavailable`.

Não usar um único booleano para colapsar estes estados. `unknown` não concede
suporte, execução ou consumo zero. Uma informação antiga pode conservar suporte
estático, mas não comprova disponibilidade atual de sessão, modelo ou recurso.

Limites iniciais do contrato: até 64 pets e 128 capacidades por pet; nomes/IDs,
revisões, modelos e motivos têm comprimentos limitados no schema. Manter os
limites já validados das preferências e recusar campos/enumerações desconhecidos.
As datas têm unidade explícita e intervalo compatível com os clientes; valores
ausentes não são convertidos em zero. Rever limites como alteração versionada.

### 3.1 Decisão de utilização

1. Resolver pet e adaptador; recusar novas ações de um pet retirado/inativo.
2. Confirmar implementação e suporte observado, incluindo o modelo exato.
3. Resolver o alvo concreto. Controlo de sessão externa depende da associação
   e das capacidades desse alvo, não apenas de uma capacidade global do provider.
4. Revalidar configuração, permissões, concessão do trabalho e revisões atuais.
5. Reservar orçamento, prazo e recurso; só então admitir a operação.
6. Guardar pedido/recibo e executar através do mecanismo aprovado.

O descritor pode indicar a necessidade de resolver um alvo ou obter uma decisão;
não deve apresentar a operação como já autorizada. Ler histórico local preservado
é diferente de iniciar novos pedidos num pet retirado.

### 3.2 Observações de sessões nativas — primeiro incremento privado

O registo por pet não substitui a observação do alvo externo. Acrescentar uma
projeção limitada a `petId + provider + origin + sessionId + conversationId`,
com revisão opaca, datas em milissegundos, modelo observado, ocupação e turno
exato quando conhecidos. Só os adaptadores do backend publicam evidência,
a partir de leituras explícitas já existentes e validadas; não aceitar
descritores ou tickets enviados pela interface ou pelo processo da extensão.

Manter até 128 alvos em RAM, publicação até 45 segundos após o início da
leitura e disponibilidade durante 15 segundos. Leituras sobrepostas usam tickets
ordenados: uma resposta antiga, expirada, invalidada ou expulsa não pode
republicar estado pronto. Eventos, mutações, perdas de eventos e desconexões
invalidam o alvo; a substituição da ligação Codex invalida todos os seus alvos.
Uma notificação antiga pode invalidar conservadoramente, nunca revalidar.
O comando passivo de consulta não liga a sockets nem religa sessões.

O histórico devolvido ao chat inclui a projeção do mesmo alvo. O consumidor
confirma identidade, ocupação, turno e frescura antes de usar disponibilidade;
uma invalidação durante a leitura impede usar a resposta como nova prova de
controlo. Conservar os controlos e revalidações nativos na execução. Ler um
histórico não comprova aprovações, execução em segundo plano ou autenticação
geral da conta. O bloqueio interno `control_conflict` suprime envio/interrupção,
mesmo que um evento posterior apresente um turno aparentemente válido.

Não guardar mensagens, diretórios, erros livres, endpoints ou credenciais nesta
cache. Os booleanos nativos mantêm a semântica original: envio indisponível por
ocupação não prova ausência de suporte. Copilot e Gravity continuam sem
interrupção por turno nesta versão da bridge. A adoção começa pelo controlador
de chat partilhado; voz, gestão e trabalho durável continuam pendentes.

**Independência dos pets:** a bridge de Copilot e Gravity reutiliza implementação
do transporte local NDJSON e validação, com diretórios/endpoints e identidade
próprios por provider/sessão. Não liga um pet através do outro, não partilha
credenciais ou modelos e não transfere decisões nativas, concessões ou memória.
O registo comum projeta observações separadas pelo alvo completo; código comum
e formato comum não constituem uma identidade ou autoridade comum.

Este incremento passou 1754 testes Rust/12 ignorados, 1060 JS e check/TSC/Vite
nas mesmas fontes. São provas sintéticas offline; não demonstram aceitação na
aplicação instalada nem provam que há uma subscrição ou conta atualmente ligada.
O comando `pet_capabilities_target` consulta a cache e revalida a configuração;
os históricos Tauri Codex/Copilot incluem `capabilityDescriptor`. A fachada de
histórico Gravity alimenta a mesma estrutura por alvo independente, mas a sua
superfície completa e continuidade permanecem trabalho PET-03.

### 3.3 Adoção seguinte: voz e gestão

A inspeção do caminho atual `voice/session.rs::available` mostra configuração
ativa e presença do executável/configuração local; isto não verifica login,
quota ou disponibilidade do modelo. A adoção do descritor tem de conservar essa
distinção e corrigir o motivo genérico que hoje sugere sempre ligar uma conta.
Uma instalação detetada não publica `authenticationRequired` nem conta ligada.

Separar captura/ASR, reprodução/TTS e execução do pet: `voice.dictate` descreve
entrada de áudio, `voice.respond` depende também do caminho de resposta escolhido,
e `voice.interrupt` precisa do token da sessão de voz original. Parar microfone
ou fala não comprova interrupção de um turno nativo. `voice.decisions` exige a
decisão original, pet/provider/alvo e prazo; a transcrição é entrada do utilizador,
não uma concessão geral ao modelo. Preservar tokens das tarefas já lançadas
quando se muda o pet da captura.

Não converter uma capacidade global ainda `unknown` num bloqueio que impeça
consultar o estado ou iniciar uma resolução explicitamente pedida. A descoberta
ativa continua uma operação própria, com resultado observado e política nativa;
não nasce como efeito colateral de ler capacidades. Voz, gestão e admissão
precisam desta resolução comum antes de adotarem o contrato como critério de
execução. A cache de sessões externas não substitui a identidade/conversa Mochi
do caminho de voz atual.

O primeiro passo de adoção resolve o descritor comum do provider lógico, confirma
pet/binding ativo e a rota de voz implementada, e verifica os pré-requisitos
locais já usados no início explícito da voz. Isto admite a **preparação da voz**;
não publica disponibilidade de inferência, conta autenticada ou nova permissão.
Modelos por resolver seguem a resolução explícita existente, sem trocar de pet.
Spotify mantém a rota determinística de voz; Ollama usa a sua configuração de
voz mesmo quando o chat de texto tem outro provider selecionado.
O modelo efetivo da voz resolve-se no perfil `@default` desse pet. A escolha
temporária de modelo no chat de texto pertence a outro contexto; o binding
atual do descritor identifica o pet, mas a preparação da voz ainda não publica
uma observação de inferência pronta para esse modelo. A resolução explícita e
os executores existentes continuam a confirmar modelo e política aplicáveis.

Revalidar o pet antes de cada novo pedido falado. Parar captura/fala, consultar
uma tarefa ou pedir interrupção da tarefa original continuam possíveis após
desativação do pet; desativar impede novos efeitos, não retira a possibilidade
de parar trabalho em curso. Conservar as revalidações de política dos executores.
A gestão recebe o mesmo descritor, sem iniciar consultas a contas/modelos;
metadados de capacidades não alteram contagens ou cobertura de observabilidade.

Resolver primeiro a intenção e a rota conhecida ao ouvir um nome. O campo
`provider` vazio do Spotify é legítimo para a rota determinística; o aviso
genérico para pets sem provider não pode bloqueá-la. Pedidos dirigidos por nome
para consultar/parar trabalho também mantêm o dono original quando o pet é
desativado ou retirado. Um pedido novo dirigido ao mesmo pet continua recusado.

O primeiro incremento desta adoção passou 1768 testes Rust/12 ignorados, 1070 JS,
cargo check, TypeScript e Vite em 644 fontes privadas. O início explícito da voz
e cada novo pedido falado reutilizam a projeção passiva, com motivos tipados;
o teste de pré-requisitos locais não consulta contas. A observabilidade inclui
`petCapabilities`, com datas em milissegundos no seu schema próprio; inventários
e contagens mantêm datas em segundos. A gestão valida limites/identidade e retira
metadados de capacidades após invalidação, falha ou resposta ultrapassada.
Captura/ASR/TTS e providers reais ainda precisam de aceitação nativa; estas
provas são sintéticas e não fecham PET-01, VOICE-01/02 ou a entrega.

### 3.4 Controlo por voz e paridade de permissões — contrato a completar

VOICE-01 cobre todas as operações de utilizador do backend de pets, incluindo
seleção/controlo de pets e sessões, objetivos/tarefas/regras, fontes/memória,
modelos/configuração e Sistema autorizado. Publicar a cobertura e o motivo de
indisponibilidade por operação; não inferir controlo integral a partir de STT,
TTS ou conversa falada. Os adaptadores de voz chamam o mesmo broker tipado,
com os mesmos alvos, orçamento, cancelamento e verificação.

Chat e voz conservam o perfil, sandbox e política de aprovação efetivos do
agente original. O mesmo pedido nativo pendente pode ter representações no
terminal/chat/voz, com uma decisão vencedora e invalidação das restantes.
Concessões nativas já válidas não exigem uma cópia de aprovação sem motivo;
concessões locais adicionais por pet/objetivo/subplugin continuam independentes.
Não traduzir indisponibilidade do canal de aprovação em permissão nem aumentar
o âmbito ao mudar o canal. Ligação externa ambígua continua sem controlo.

Este aditamento não altera o schema1 nem anuncia capacidades novas como
implementadas. Requer testes por adaptador de pedidos concorrentes/tardios,
Stop, revogação e modos nativos suportados; áudio real para aceitação VOICE-02.
Os contratos completos estão na [arquitetura, §9.1 e §11.1](PETS-ARCHITECTURE.md).

### 3.5 Pet, runtime, motor e coordenação — objetivo revisto em 24/10

Identidade/presença do pet, disponibilidade do runtime e prontidão do motor são
dimensões diferentes. Receber eventos/mensagens, consultar tarefas, tratar Stop
ou decisões e usar operações determinísticas pode ser suportado sem modelo
local carregado ou ligação ao fornecedor de inferência. Cada capacidade declara
a dependência real; `modelBinding: null` identifica operações independentes de
modelo. Uma operação não implementada continua indisponível com motivo explícito.

Comunicação direta, descoberta, delegação e coordenação terão capacidades
próprias, observadas e autorizadas, consumidas pelo mesmo controlo de tarefas.
Ollama é o coordenador preferencial configurável; o papel não lhe confere acesso
global nem transforma os restantes pets em subagentes dependentes. A ausência do
coordenador/motor não deve retirar capacidades independentes dos outros pets.
Delegação não transfere permissões, contas ou perfil: admitir/executar exige a
interseção da política do trabalho e das concessões do pet destinatário.

Este aditamento mantém o schema1 e os 43 requisitos; não anuncia capacidades
TEAM ou novos providers como já implementados. A fábrica/registo existentes são
a origem comum. O [protocolo normativo](PETS-ARCHITECTURE.md#12-cooperacao-entre-pets)
e TEAM-01/02 definem a correlação, idempotência, limites e falhas a aceitar.

## 4. Catálogo inicial a resolver

Os nomes são o contrato alvo do registo; mapear para as operações existentes sem
renomear os seus protocolos nativos à força.

| Família | Capacidades |
| --- | --- |
| Conversa | `conversation.respond`, `conversation.stream`, `conversation.cancel`, `conversation.archive`, `conversation.restore`, `history.read` |
| Sessão | `session.discover`, `session.history`, `session.send`, `session.interrupt`, `session.approve`, `session.continueBackground` |
| Modelos | `models.list`, `models.select`, `models.reasoning`, `models.tools` |
| Ferramentas | `tools.native`, `tools.mcp`, `tools.exactCommand`, `tools.typedRecipe`, `desktop.observe`, `desktop.control` |
| Artefactos | `documents.text`, `documents.folder`, `documents.image`, `documents.pdf`, `documents.multi`, `documents.renew`, `documents.retention` |
| Contexto | `sources.calendar`, `sources.rss`, `sources.imapRead`, `memory.retrieve`, `memory.manage` |
| Trabalho | `goals.run`, `tasks.resume`, `tasks.stop`, `rules.schedule`, `rules.events`, `delegation.send` |
| Voz | `voice.dictate`, `voice.respond`, `voice.decisions`, `voice.interrupt` |
| Sistema | Operações/subplugins de SYS-01; estado/permissões por pet e recurso |
| Serviços | Operações de música/aplicação, preparação de rascunhos e restantes integrações explicitamente implementadas |
| Utilização | `usage.observedAttempts`, `usage.nativeThread`, `usage.account`, `usage.quota`, `usage.resourceState` |

O repositório RUN-01 acrescenta `goals.draft` e `goals.inspect` exclusivamente
para `nativeUser`: guardar/rever propostas não concede execução. A disponibilidade
depende do alvo/configuração; o primeiro backend de persistência é Linux.
`goals.run` e execução continuam por implementar. O motor tem contrato e
estado separados na secção 6.4 da [arquitetura](PETS-ARCHITECTURE.md).

Capacidades do motor ainda não implementadas ficam `unsupported` com
`notImplemented`; não se anunciam porque o adaptador já consegue responder a
uma pergunta. A capacidade `rules.schedule` exige o motor durável AUTO/RUN.

A unidade657 acrescenta `goals.admissionReview`, apenas para `nativeUser`, com
alvo obrigatório. Prepara/consome uma revisão de uso único e pode guardar uma
entrada `queued` em schema3, sem dispatch. Prontidão desconhecida, política ou
modelo alterados, prazo expirado e limites não mensuráveis recusam a admissão.
Uma entrada lida do disco ou um token serializado não é um permit de execução;
proveniência de consentimento, leases e writer continuam por integrar. Os comandos
nativos não fazem probes de conta/modelo e as provas são sintéticas/offline.

A unidade660 acrescenta a prova autenticada de consentimento e o endpoint de
proveniência. `verified` confirma correspondência com chave/ledger e prazo do
consentimento; `executionAvailable` permanece `false`. Política/prontidão,
claim/lease, orçamento no writer e decisões nativas continuam obrigatórios antes
de efeitos. O schema4 do objetivo é distinto do schema1 deste descritor; dados
unsigned anteriores mantêm leitura e não recebem autoridade automaticamente.

Na unidade664, o schema5 do objetivo e o schema2 do ledger privado fixam
claim/controlador. Endpoints nativos de preparação/consulta/libertação do lease
continuam com `executionAvailable:false`; `goals.run` permanece unsupported.
Owner/guards físicos não são um permit de modelo/ferramenta. Origens liveController,
storedLease e none distinguem posse conhecida e workers desconhecidos, e preparação
não representa inferência. A posse exata permite reconstruir apenas a projeção sem
busy próprio; mantém os gates de prontidão/política/modelo e o prazo original.

No incremento815, `goals.continuationReview` declara apenas uma revisão nativa
com alvo obrigatório nos adaptadores Codex, Gravity, Copilot CLI e Ollama. Exige
fecho autenticado, orçamento adicional explícito e reconciliação de efeitos
incertos; não ativa o executor de outro provider. O objetivo passa a schema10,
com histórico/limites/modelos anteriores vinculados. A prova de execução da
continuação é limitada a Ollama local/Spotify; `goals.run` geral permanece
unsupported. Serviços/observadores e adaptadores sem esse suporte não o herdam.
[Âmbito verificado](PETS-VALIDATION.md#continuacao-local815).

No incremento822, `goals.retirementReview` permite revisão nativa com alvo e
aprovação para reduzir um grant canónico sem admissão, nunca reclamado e sem IO.
Os quatro adaptadores nativos usam o mesmo contrato de proveniência; a fábrica
não recria aprovação/execução. A assinatura de retirada permanece no grant
schema8, conservando consentimento e consumo anteriores. Não altera o objetivo
nem concede limites adicionais; qualquer continuação exige outra autorização.
Serviços/observadores não herdam esta capacidade. [Âmbito verificado](PETS-VALIDATION.md#retirada-local822).

No incremento823, a recuperação e as consultas de passo/receita distinguem
contabilização congelada por uma continuação autenticada. `accountingFrozen`
e `recordedResult` são projeções de leitura; não constituem uma capacidade nova
de execução ou uma liquidação retroativa. A origem, grants e encargos anteriores
permanecem exatos; resultados ausentes não viram sucesso pelo estado atual do
Sistema. A execução verificada continua limitada ao adaptador Ollama local e ao
leitor privado, sem anunciar paridade nos restantes pets. [Âmbito verificado](PETS-VALIDATION.md#recuperacao-local823).

No incremento825, a revisão de continuação existente recebe uma opção explícita
para preparar trabalho novo mantendo efeitos históricos desconhecidos. Não é
uma capacidade herdada de execução: o backend exige passo/âmbito autênticos e
classificação nativa de operações de estado Spotify; apenas o planeamento local
Ollama com receitas individualmente aprovadas é aceite. Outras operações,
origens ausentes e modos sem esse catálogo mantêm recusa. O manifesto schema2
e o consumo conservam-se no histórico, sem tornar sucesso um efeito desconhecido.
Os descritores dos restantes pets conservam os limites reais de execução.
[Âmbito verificado](PETS-VALIDATION.md#incerteza-local825).

## 5. Condições por grupo de pets

| Grupo | Regras iniciais |
| --- | --- |
| Codex | Associar capacidades à sessão/turno canónicos; alvo ambíguo recusa controlo e pode conservar leitura |
| Gravity | Capturar capacidade real de aprovações; canal headless sem decisões não anuncia aprovação interativa ou execução geral equivalente |
| Copilot CLI | Resolver ACP/ponte e alvos de sessão. Verificar com fixtures; não consultar conta/modelo só para produzir este descritor |
| Ollama | Ferramentas e raciocínio dependem do modelo observado. O executor Ask só fica utilizável com política e decisão exata atuais |
| LM Studio | Adaptador tipado local828; provider vazio permanece sem configuração. Associação explícita, metadados e ligação nativa exata antes de execução limitada; voz/shell não herdados |
| APIs | Perfil/default não prova pet configurado nem adaptador completo; contratos multimodais/ferramentas são por modelo/rota |
| Spotify | Serviço determinístico nas capacidades atuais; `conversation.respond` não é inferido de música/Sistema |
| Observadores | Detectar processo/hook pode permitir observação; não implica `session.send` ou `goals.run` |
| GitHub entidade | `retired`; preservar metadados/histórico local, bloquear novos pedidos do pet |

**Decisão R40 integrada localmente em828:** o LM Studio recebe adaptador próprio
apenas após associação explícita do perfil. O provider vazio permanece sem
configuração. A prontidão da execução limitada exige metadados nativos locais
com chave/instância inequívoca e `engineBinding` exata, conservada na revisão,
consentimento e claim e verificada antes de inferir. Reutilizar streaming não
concede voz, shell ou gestão de modelos. As APIs ainda sem adaptador mantêm o
seu estado anterior. Dez percursos nativos privados passaram; motor/cofre reais,
restante paridade e entrega mantêm aceitação própria.
[Provas](PETS-VALIDATION.md#paridade-local828).

## 6. Concorrência, atualizações e exposição

Começar pelos limites que o backend realmente suporta. A estrutura atual de
conversa corrente por provider não permite anunciar paralelismo de controlo
entre várias conversas desse provider. Sessões/recursos independentes só ganham
esse estado depois de a admissão e o cancelamento estarem implementados.

Eventos apenas indicam mudança e fazem o cliente reler o snapshot. Um evento do
renderer não autentica, fixa revisões ou publica capacidades. Leitura tardia,
modelo trocado, sessão perdida ou configuração revogada não reativam o snapshot
antigo. Propostas em curso conservam a origem e revalidam antes da mutação.

O modelo recebe apenas as ferramentas e limites necessários ao trabalho admitido.
O descritor público não contém tokens, caminhos privados, argumentos arbitrários
de execução ou IDs que concedam controlo de outros pets.

Gestão de concessões, inscrição de fontes, renovação de leitura e purga de cópias
continuam em canais nativos próprios. Uma capacidade de gestão pode aparecer no
inventário para a UI sem ser oferecida em `providerModel` ou em planos autónomos.
O broker não transforma automaticamente o catálogo inteiro em ferramentas LLM.

## 7. Aceitação PET-01

- Inventário completo dos IDs atuais e papéis resolvidos, com GitHub retirado.
- Capacidades/configuração coerentes em conversa, voz, gestão e planeamento.
- Separação support/availability/reason, incógnitas e frescura sem defaults inventados.
- Troca de modelo/provider/perfil, dados fora de ordem e reinício invalidam a origem.
- Leitura do descritor sem inferência, login/religação, modelo carregado ou serviços iniciados.
- Casos Codex, Gravity, Copilot, Ollama, restantes locais/API, observadores e serviços.
- Operações/modelos sem suporte ou alvo inequívoco recusadas também na chamada direta.
- Uma integração/receita/molde não acrescenta grants ou permissões.
- Fixtures sem quota real Copilot; prova nativa e integração discriminadas na entrega.

Esta aceitação fecha o contrato de capacidades; RUN/EXE/AUTO continuam necessários
para oferecer objetivos e rotinas autónomas.

## 8. Primeiro núcleo verificado — 23/10/2026

O checkpoint `.build/pet-capabilities-next-r1/WORKING-CHECKPOINT.json` regista
637 fontes, 1744 Rust/12 ignorados, 1052 JS, cargo check/TypeScript/Vite, com os
mesmos inputs de fonte. A base privada de 633 fontes foi conservada intacta.

O leitor cobre 25 IDs e usa configuração existente e caches em RAM. Os produtores
atuais aproveitam os pedidos explícitos já existentes do seletor de modelos e do
`/api/show` Ollama; não acrescentam consultas ao ler o descritor. Tickets
ordenados, prazo de publicação, frescura e revisão isolam respostas antigas,
modelos/endpoints diferentes e alterações de política. Controlos de raciocínio
usam a semântica do parser de pedidos existente, incluindo valores named/bool.

As declarações ainda sem observação nativa permanecem desconhecidas ou exigem
alvo. Bindings API vazios não são promovidos a pets configurados. Funções do motor
futuro ficam `notImplemented`. A leitura do registo não substitui os checks dos
executores atuais nem passa a ser um grant. CTX-01 continua bloqueador de ativação.
