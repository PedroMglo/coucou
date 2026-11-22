# Plano de desenvolvimento dos pets autónomos

**Atualização de estado R43F1A — 25/10/2026:** no SDK privado841 passou a preparação durável de memória selecionada para objetivos: referências exatas por pet/conversa de origem, contexto versão2 e objetivo versão11 apenas quando há seleção, sem copiar os dados brutos para o diálogo inicial. Passaram 2612 testes Rust/21 ignorados, check/build e o percurso Tauri privado130, incluindo reinício, revogação e recusa do leitor840 sem alterar os ficheiros. Não houve chamadas a modelos ou quota. A admissão autónoma com memória permanece deliberadamente recusada até R43F2 ligar revisão humana dos dados/destino, consentimento próprio do objetivo e mensagem anterior ao hash/reserva canónicos. A raiz aceite permanece834; a app instalada permanece .30. R43 e os 43 requisitos continuam abertos.

Revisão: **25/10/2026**. Este plano executa a
[arquitetura normativa](PETS-ARCHITECTURE.md). O
[estado de implementação](PETS-IMPLEMENTATION-STATUS.md) identifica as provas.
Os IDs abaixo são estáveis; fechar uma linha exige a sua aceitação, não apenas
código ou um número de testes.

## 1. Ordem de execução atual

| Ordem | Unidade | Resultado que desbloqueia |
| --- | --- | --- |
| 0 | ARC — arquitetura e rastreabilidade | Contratos e lacunas claros; revisão antes de novas dependências |
| 1 | PET — identidade, capacidades, sessões e decisões | Cada pet conhece os seus limites e pode receber trabalho com dono correto |
| 2 | RUN — objetivos, tarefas e fila duráveis | Continuidade em background, orçamento e recuperação por pet |
| 3 | EXE — ferramentas, política e verificação | Executar objetivos e receitas autorizadas com efeitos comprovados |
| 4 | TEAM — comunicação, cooperação e coordenação resilientes | Chamadas diretas, delegação paralela/assíncrona e coordenador opcional, após autonomia/recuperação individual |
| 5 | MEM/CTX e AUTO — contexto e iniciativa dos pets | Completar memória/fontes e rotinas necessárias à autonomia e à cooperação; dependências destas unidades antecipam-se quando indispensáveis |
| 6 | SYS/INT — expansão de Sistema e serviços | Capacidades adicionais consumidas pelos mesmos contratos dos pets |

OBS, DATA, VOICE e REL acompanham as unidades que deles precisem. A matriz de
Sistema é contrato obrigatório desde a arquitetura; a expansão física e os
widgets não precedem o núcleo dos pets. As correções de preservação já abertas
continuam bloqueadores da ativação da composição correspondente.

Cada unidade entrega um percurso funcional utilizável, preserva os restantes e
atualiza o estado. O trabalho de backend pode avançar em peças independentes;
integração e aceitação respeitam as dependências abaixo. Há um único responsável
de desenvolvimento, sem novos agentes.

O objetivo atualizado em 24/10 está integralmente em [PETS-GOAL.md](PETS-GOAL.md).
Preservar os 43 requisitos/IDs, decisões e componentes funcionais. Cada pet é
diretamente acessível e tem identidade/runtime/contexto próprios; o motor de IA
é invocado quando necessário. Ollama é coordenador preferencial configurável,
sem se tornar intermediário obrigatório ou ponto único de falha. A restrição a
um agente de desenvolvimento não impede implementar/testar a cooperação dos
pets do produto. As prioridades ajustam-se às dependências reais, sem refazer
trabalho concluído.

## 2. Contratos, lacunas e aceitação por unidade

### ARC — arquitetura

| ID | Trabalho | Aceitação |
| --- | --- | --- |
| ARC-01 | Separar arquitetura, estado e histórico; atualizar prioridade para arquitetura e pets | Documento normativo, plano com IDs, fontes identificadas e arquivo anterior preservado; ligações coerentes |
| ARC-02 | Rever contratos quando código/decisão revelar incompatibilidade | Registar decisão, motivo, migração, impacto e requisitos afetados antes de implementar um caminho incompatível |
| ARC-03 | Manter definição de sistema completo e cobertura | Cada cenário do dia a dia tem capacidade, implementação, prova e pendência por provider; exceções não desaparecem dos critérios |

### PET — identidade, capacidades e sessões; próximo grupo de desenvolvimento

| ID | Trabalho e ponto de partida | Aceitação |
| --- | --- | --- |
| PET-01 | Implementar `PetDescriptor` e capacidades versionadas a partir de `pet_factory`, `pet_config`, modelos e adaptadores | Matriz por pet distingue suporte, configuração, disponibilidade, frescura e motivo. API/perfil vazio ou hook não aparecem como execução suportada. Nome/modelo não alteram identidade; GitHub entidade continua retirado |
| PET-02 | Associar conversa, sessão nativa, turno, tarefa e decisões com autoridade inequívoca | Trocar de pet/vista/sessão, resultados tardios, sessão ambígua, revogação e Stop preservam dono. Terminal/chat/voz partilham a identidade do pedido nativo e a política da sessão/perfil; uma resposta invalida as outras representações. Descoberta por PID/pasta/título não autoriza controlo |
| PET-03 | Completar continuidade e decisões Gravity/Copilot/Codex segundo capacidades nativas | Continuar a execução original quando suportado, decisões exatas e limpeza dos filhos próprios. Gravity sem canal de aprovação não ganha permissões. Copilot usa fixtures enquanto quota real está vedada. Preservar as capacidades do agente original — ficheiros, comandos, projetos e ferramentas/MCP — conforme a sessão/perfil autorizados; distinguir continuação nativa do dispatcher autónomo ainda sem limites internos garantidos. Verificar paridade de sandbox/política/modos de aprovação, sem flags mais permissivas nem concessões copiadas entre sessões |
| PET-04 | Consolidar identidade, modelos, ferramentas e falhas locais/API | Ollama/LM Studio/API mantêm modelo e contexto escolhidos, recusam ferramentas não suportadas e não fazem fallback silencioso. Completar ligação de pets ainda sem configuração/adaptador resolvido |
| PET-05 | Definir o contrato comum dos pets de serviço e observadores | Spotify permanece determinístico nas capacidades atuais; integrações com executor LLM conservam dono próprio. Só operações efetivamente implementadas ficam disponíveis |

**Primeira execução após esta revisão:** PET-01, com inventário de capacidades
observadas e contratos de sessão/decisão PET-02. A lacuna de aprovação/continuidade
Gravity é discriminada em PET-03; não se dilui numa aceitação genérica do Codex.
Não acrescentar modos autónomos antes de RUN/EXE estarem disponíveis.
O [contrato v1 de capacidades](PET-CAPABILITIES-CONTRACT.md) define os campos,
motivos e origem dos dados. O núcleo privado, as observações de histórico por
alvo e a adoção inicial pelo chat comum, preparação da voz e gestão foram
verificados em 644 fontes;
faltam cobertura/adoção completas e aceitação nativa para fechar PET-01.
Os pets mantêm identidades e autoridade independentes quando reutilizam código
de transporte. Ler o descritor não faz chamadas a providers.

**Incremento718 integrado:** donos/adaptadores de objetivos validados pela fábrica,
incluindo assistência opcional de serviços; os registos antigos mantêm leitura e
a criação nativa não promove identidades retiradas/sem executor. Sete regressões
novas, composição2141 Rust/17 ignorados, 1109 JS e 181 ensaios privados do runtime.
Paridade nativa e executores adicionais continuam abertos.
[Prova](PETS-VALIDATION.md#evidencia-local).

**Incremento742 integrado — PET-02/03 e EXE-02:** unidade completa revista de
origem/escolhas/formulários/IO e callbacks nativos ligados ao broker comum.
Preservação da política ACP, configuração em ordem de receção, cancelamento exato
de pedidos e retoma Gravity vinculada à identidade/pasta têm regressões próprias.
Prazo/Stop/caller, namespace anunciado, recusa de `callId` repetido e cancelamento
local de calendário conservam as concessões existentes. 51 ficheiros aplicados
com pré-imagens, 387 inputs nativos/757 web exatos; 2314 Rust/17 ignorados,
1129 JS, check/TSC/Vite e 367 ensaios privados no anfitrião, sem avisos novos.
A app continua na .30. Cobertura nativa parcial/desconhecida, providers reais,
terminal/GUI/áudio e entrega mantêm-se pendentes; PET-02/03 e EXE-02 não fecham.
Segue recuperação/reconciliação durável das tentativas de passos de objetivos,
antes de ampliar planeamento/executores. Todos os 43 requisitos continuam
sob os respetivos critérios de aceitação.
[Prova](PETS-VALIDATION.md#evidencia-local).
[Matriz e limites](PETS-VALIDATION.md#evidencia-local).

**R40 integrado localmente em828 — PET-04 e paridade RUN/EXE:** LM Studio usa
o contrato comum da fábrica e do runtime, com associação explícita,
identidade nativa do modelo/instância e credencial de âmbito próprio. Preservar
provider vazio, configuração e histórico antigos; não herdar voz ou shell.
O ciclo nativo de aprovação/execução/verificação passou com HTTP/MPRIS privados.
A composição passou 2564 Rust/21 ignorados, 1186 JS/TSC/Vite, check, 554 ensaios
privados no anfitrião e dez percursos Tauri: sete anteriores e três de LM Studio,
incluindo conversas, Stop/reinícios/continuação e mudança de modelo sem inferência
ou débito. Adoção de 36 ficheiros, três novos, sem commit ou instalação.
Motor/cofre operacionais, restantes providers e PET/RUN/EXE/43 requisitos
continuam abertos. [Provas e limites](PETS-VALIDATION.md#paridade-local828).

**R41 integrado localmente em 830 fontes — PET-02/03 e preservação DATA:** o resolvedor
comum conserva a associação nativa quando pasta ou intenção de criação mudam,
recusando antes do prompt em vez de criar outra sessão silenciosamente.
Nova conversa continua explícita e conserva o arquivo. Gravity deixa de
reenviar bypass na retoma; Copilot conserva a política ACP. A validação de
diretórios vivos passa a verificar identidade/posse sem confundir alterações
legítimas das entradas com substituição, mantendo snapshots estritos das fontes.
2571 Rust/21 ignorados,1186 JS/TSC/Vite, check/build, 720 ensaios privados no
anfitrião e 12 percursos Tauri privados passaram. Há comparação anterior/corrigido
dos quatro donos CLI e do erro de perfil; não há prova de paridade operacional
com terminais externos ou conclusão da autonomia. Dez ficheiros adotados, dois
novos; instalação, HEAD/index e protótipo Gravity não registado preservados.
[Provas e limites](PETS-VALIDATION.md#continuidade-local830).

### RUN — objetivos e tarefas duráveis

| ID | Trabalho | Dependência e aceitação |
| --- | --- | --- |
| RUN-01 | Schema e repositório de objetivos, tarefas, planos, passos e eventos; migração do estado atual | PET-01/02. Separar atividade de estado durável; guardar admissão antes de efeitos, preservar IDs e trabalhos ativos além dos dez do journal público |
| RUN-02 | Admissão, fila, leases e associação da conversa de background | RUN-01. Não trocar seleção/rascunhos da UI; uma sessão não recebe dois controladores. Prazos incluem fila, slots físicos aguardam fim do worker, outros pets continuam. Prontidão fresca antes do efeito; validade durante IO não depende apenas da idade da cache e continua vinculada a política/posse/contexto/prazo |
| RUN-03 | Ciclo comum de planeamento, progresso, orçamento e conclusão | RUN-01/02 + EXE-01. Critérios verificáveis, limites de ciclos/tentativas/recursos, falta de progresso detetada; texto do modelo não basta para concluir |
| RUN-04 | Recuperação, pausa, Stop e retoma após falha | RUN-01/02. Reinício não reproduz efeitos; reconciliar processos/sessões/recibos, preservar incerteza e permitir retoma vinculada ao objetivo original |
| RUN-05 | Prioridades e justiça entre trabalho manual e rotinas | RUN-02 + OBS-01. Stop/decisões responsivos, nenhuma rotina monopoliza provider/modelo/dispositivo; limites configurados e observáveis |

**Incremento815 integrado — RUN-04, continuação explícita:** o objetivo original
em recuperação recebe nova admissão/dono por revisão nativa, com fecho confirmado,
histórico/consumo vinculados e orçamento adicional escolhido. Schema10 conserva
a proveniência ao publicar planos e concluir. Quinze testes Rust e cinco JS novos;
2526 Rust/21 ignorados,516 ensaios privados/check,1169 JS/TSC/Vite. A app privada
concluiu a continuação mantendo a chamada antiga desconhecida e voltou a passar
o ciclo anterior. Revisão comum na fábrica não implica execução equivalente nos
outros adaptadores. Seguem autorizações órfãs entre publicações e efeitos sem
resultado recuperável; RUN-04, migração e entrega continuam abertos.
[Âmbito e falhas conservadas](PETS-VALIDATION.md#continuacao-local815).

**Incremento822 integrado — RUN-04, autorização sem admissão após crash:** origem,
canónico e ausência nativa de claim/IO sustentam revisão/retirada explícita e ACK
assinado, sem reconstruir a aprovação perdida ou escrever um objetivo novo.
Schema8 conserva o consentimento e a auditoria; um temporário isolado, claim ou
origem contraditória recusam. Cinco crashes/reinícios privados, nova continuação
com consumo antigo e o ciclo anterior passaram; 2546 Rust/21 ignorados, 536 ensaios
privados/check e 1177 JS/TSC/Vite. A contenção de leitura/escrita recebeu certificação partilhada, aquisição limitada
antes da escrita e registo de receita com a mesma escolha; sete regressões novas
cobrem locks, assinatura e conservação da decisão. [Provas](PETS-VALIDATION.md#retirada-local822).

**Incremento823 integrado — RUN-04, recuperação e prefixos assinados:** a
recuperação retém o lock partilhado do objetivo até terminar a transação da
autoridade. Um prefixo autenticado por continuação torna a contabilização antiga
imutável: a consulta acrescenta conhecimento autêntico sem inventar Finished,
liquidar reservas ou reescrever grants. Antes de selar, conserva o percurso nativo
existente. Passaram 2549 Rust/21 ignorados, 539 ensaios privados/check e
1180 JS/TSC/Vite. Três percursos novos da app privada cobrem o erro original,
Cancel/ausência/corrupção, Finished/Settled e execução posterior até conclusão;
os dois ciclos nativos anteriores passaram. [Provas](PETS-VALIDATION.md#recuperacao-local823).

**Incremento825 integrado — RUN-04, revisão de operações de estado desconhecidas:**
a opção separada vincula cada tentativa à operação/âmbito assinados e conserva
o manifesto schema2 e o consumo antigo. A continuação normal continua bloqueada.
Só pausar/reproduzir/volume absoluto suportados recebem revisão para novo
planeamento Ollama local; cada receita atual exige aprovação exata. Não inventa
resultado, reembolso, controlador ou permissão antiga. Três regressões Rust e
três JS novas, 2552 Rust/21 ignorados, 542 ensaios privados/check e 1183 JS/TSC/Vite.
A app privada passou falha real/reinício/fecho, revisão, Busy/publicação parcial,
ACK e nova execução até conclusão, conservando uma ferramenta antiga desconhecida;
outro percurso confirmou Cancel antes da escolha. Os cinco ciclos anteriores
passaram. [Provas e limites](PETS-VALIDATION.md#incerteza-local825).

**Unidade seguinte — PET-03/04 e RUN/EXE, paridade dos adaptadores:** inventariar
os transportes/sessões reais e os limites verificáveis de Codex, Gravity,
Copilot, Ollama e motores API/configuráveis. Reutilizar a fábrica e o runtime
comuns; escolher o próximo executor implementável com contexto, política,
orçamento, cancelamento e recibos próprios. Não promover chat nativo a execução
autónoma equivalente sem provar os limites das ferramentas internas. Usar
fixtures/modelos locais gratuitos, sem quota real Copilot/GitHub. RUN-04 continua
com aceitação delimitada; origens ausentes e outras operações desconhecidas
mantêm recusa e critérios próprios, além de justiça da fila, janela fechada,
memória/fontes, cooperação, voz, Sistema e migração/entrega. Os 43 requisitos
permanecem no âmbito.

**Incremento811 integrado — RUN-04, histórico cumulativo:** consulta nativa
das admissões originais, respetivos recibos e inventário assinado, com subtotal,
origens incompletas, medição indisponível e consumo incerto preservado. Não
certifica um head integral dos formatos antigos, nem concede continuação.
Quinze testes novos;2511 Rust/21 ignorados,501 ensaios privados/check; web exato à
prova1164 JS/TSC/Vite. A app privada contou duas admissões sem apagar o consumo
anterior; o ciclo e quatro crashes/reinícios passaram. Segue revisão/admissão de
continuação com histórico/consumo vinculados e verba adicional só por aprovação
explícita. [Provas e limites](PETS-VALIDATION.md#historico-cumulativo-local811).

**Incremento808 integrado — RUN-04, revisão e fecho:** a revisão nativa
retém origem/ausência/autoridade e hashes; a escolha fecha só a execução antiga.
Recibo schema7 e projeção schema9 autenticados/idempotentes conservam consumo,
cancelamento/conclusão e incerteza. ACK após reinício confirma a escolha assinada,
sem nova aprovação/inferência. Treze testes novos; total2496 Rust/21 ignorados,
486 ensaios privados no anfitrião,1164 JS/check/TSC/Vite. A app privada passou
quatro crashes/reinícios no conjunto, falha de publicação após o fecho, conclusão
prévia preservada e o ciclo completo anterior. Segue revisão de continuação com
nova admissão e uso acumulado, sem replay de efeitos desconhecidos ou passagem
por ResumeDraft genérico. Fonte de planeamento perdida/corrompida mantém recusa
conservadora; os restantes cenários de RUN-04 continuam abertos.
[Aceitação delimitada](PETS-VALIDATION.md#fecho-recuperacao-local808).

**Critérios de continuação RUN-04, ainda sujeitos à cobertura por cenário:** usar a vista autenticada
das admissões registadas, sem lhe atribuir cobertura histórica integral não
verificada, e integrar
a revisão nativa de continuação, mantendo intenção/critérios/contexto e o fecho
anterior confirmado. Mostrar uso acumulado e limites novos autorizados; nenhum
reset implícito de saldo/prazo. Planos e aprovações antigos não executam outra
vez. Aceitação inclui múltiplas admissões, histórico alterado/ausente, efeito
desconhecido, cancelamento, revisão stale, ACK parcial e repetição de token
histórico após uma admissão posterior. A retoma só será anunciada após o percurso
nativo integrado e a prova delimitada dos adaptadores suportados.

**Incremento801 integrado — RUN-04, fase de leitura:** nova consulta de
reconciliação vinculada ao objetivo/revisão, com recibos assinados, consumo e
incerteza, sem escrita ou permissão de execução/recuperação. Origem de ausência
autenticada e flock original retido; processo vivo ou identidade/perfil/inode
divergentes recusam a prova. Oito testes novos, total2483 Rust/21 ignorados,
473 ensaios privados no anfitrião e check passaram; web exato à prova1160 JS/
TSC/Vite. Na app privada, dois crashes/reinícios conservaram consumo e ficheiros;
o ciclo completo confirmou também leitura da conclusão e do Stop desconhecido.
Seguem revisão explícita, fecho aprovado/ACK e continuação por nova admissão
conservando o uso anterior. RUN-04 e os 43 requisitos mantêm aceitação própria.

**Incremento796 integrado — RUN-02/04:** preparações sem ativação fecham após
Pausar/Cancelar ou expirar, mesmo se o ledger recebeu claim e o objetivo ainda
não. Manutenção determinística confirma apenas a limpeza do objeto original,
com ACK antes da libertação e sem nova inferência/admissão. Cancel em fecho
parcial permanece terminal e conserva uso/data/dono; fecho cedido não readquire
nem afeta outro dono nativo. Onze testes novos, total2475 Rust/21 ignorados,
465 ensaios privados no anfitrião e check passaram; fontes web exatas à prova
1160 JS/TSC/Vite. A app privada confirmou os três controlos sem ativação nem
efeitos e um claim independente seguinte. Segue reconciliação e revisão explícita
após perda do processo original, antes da aceitação geral RUN-04. Os 43 requisitos
e o objetivo completo continuam abertos; os incrementos abaixo são históricos.

**Incremento793 integrado — RUN-02/04:** fecho original retido com backoff e ACK,
repetição explícita limitada ao mesmo drain, fila com leitores partilhados e Busy
preservado. Dez testes Rust e dois JS novos; total2464 Rust/21 ignorados,
1160 JS e 454 ensaios privados no anfitrião. A app Tauri passou o ciclo com claim
concorrente e Stop sob locks dos dois catálogos, zero workers antes de fechar,
reserva até ACK e uso started/desconhecido conservado, sem repetir efeitos.
R29B/C permanecem como falhas históricas, com estes percursos corrigidos793.
Seguem claims não ativados após pausa/cancelamento/expiração, escalada Cancel em
fecho parcial e fecho cedido com outro dono; depois reconciliação e nova revisão
após perda do processo original. Não declara o sistema/entrega concluídos.

**Incremento790 integrado — RUN-02:** a confirmação retém Review/Approved nativos
e autoridade exata perante contenção/incerteza. ACK de evento já publicado é
apenas confirmação, sem nova autorização, mesmo após expirar. Recibo antigo e
nova revisão após Pausar/ResumeDraft coexistem com tokens/consentimentos distintos.
Dez testes Rust, três JS e a app Tauri isolada confirmam duas recusas temporárias,
uma só admissão, mesma aprovação/orçamento/origem e ACK expirado sem novos efeitos.
Total 2454 Rust/21 ignorados, 1158 JS e 444 ensaios privados no anfitrião.
Stop durante HTTP sintético ainda falhou em R29B; segue RUN-04 com retenção e
repetição da mesma prova de fecho/ACK, sem novos efeitos, e classificação da
recusa Conflict no claim concorrente R29C. Depois, reconciliação
e revisão da retoma após perda do dono original.

**Incremento787 integrado — RUN-04:** identidade assinada do processo/perfil,
bloqueio real por lease e classificação só de leitura, compatível com dados
anteriores. Dois crashes/reinícios reais isolados confirmam ausência, preservação
e recusa de reaproveitar controlador/aprovação. 2444 Rust/21 ignorados, 1155 JS,
check/TSC/Vite e 434 ensaios privados no anfitrião. Retoma após falha continua
aberta. Corrigir primeiro a perda da aprovação de admissão perante `Busy`
transitório, observada na fixture paralela; depois reconciliar/fechar o dono
antigo e rever continuação com histórico e orçamento conservados.

**Correção da confirmação após atualizar metadados — 24/10:** a revisão pública
das capacidades inclui os instantes de observação. Atualizar o Ollama antes de
confirmar mudava essa revisão e fazia recusar o mesmo trabalho. A evidência nativa
passa a distinguir a observação renovável do âmbito semântico: ignora apenas a
revisão pública e os instantes/expiração, preservando identidade, modelo/perfil,
suporte, disponibilidade, decisões, callers, limites, restrições e proveniência.
Continuam obrigatórias disponibilidade fresca, política, contexto, plano, prazo
e orçamento originais. Uma capacidade ou política efetivamente diferente exige
nova revisão. Três regressões incluem o produtor real de revisões e confirmação
após expirar a observação inicial. A composição passou 2373 Rust/20 ignorados,
1153 JS, check/TSC/Vite e 277 ensaios privados; sem inferência/quota real.
[Validação versionada](PETS-VALIDATION.md).

**Incremento769 integrado — mesma unidade RUN/EXE aberta:**
consentimento/journal/orçamento de planeamento, writer Ollama, recibos tipados e
publicação canónica com gate original/CAS/readback/ACK estão no código. Leitores
validam a pertença autenticada e detetam a remoção do histórico de planeamento;
Stop/drain e recuperação do mesmo ACK não criam workers nem renovam autorização.
Passaram 2401 Rust/20 ignorados, check e 305 ensaios privados no anfitrião.
Os âmbitos gerados de passo/receita, aprovação humana exata e dispatcher com
feedback/replaneamento nativo continuavam pendentes; não existia ativação pública
do planeador. [Âmbito e avisos](PETS-VALIDATION.md#planeamento-integrado769).
Não alterar a percentagem de conclusão nem fechar requisitos por este núcleo.

**Incremento local770 — Step2/Recipe3, mesma unidade aberta e sem commit:**
planos gerados confirmados ligam-se a revisões/manifestos de receita assinados,
com os critérios originais, aprovação humana exata e orçamento partilhado. Os
ensaios nativos privados percorrem os passos, a medição final e a conclusão do
objetivo sem repetir efeitos/provas. Passaram 2409 Rust/20 ignorados, check e 313
ensaios no anfitrião. O dispatcher, feedback/replaneamento, equidade, revisão de
intenções grandes e ativação operacional continuavam pendentes no checkpoint770.
[Validação e limites](PETS-VALIDATION.md#execucao-gerada-local770).
Commits e pushes suspensos até nova ordem; instalação .30 e âmbito dos 43
requisitos preservados. A estimativa global permanece aproximadamente 40/100.

**Incremento local778 — ciclo nativo integrado, mesma unidade ainda aberta:**
um dispatcher por dono/lease percorre proposta, publicação, aprovação exata,
manifesto confirmado, execução, verificação e replaneamento por feedback
assinado/saldo original. A thread mantém-se independente da vista, preserva as
aprovações originais em falhas de ACK e fecha/liberta a reserva só após prova
nativa de IO terminado. A admissão operacional usa uma projeção nativa única;
os controlos de objetivos ativam e supervisionam o ciclo. A fábrica anuncia
apenas as capacidades limitadas efetivamente implementadas para Ollama/Spotify.

Passaram 2426 Rust/20 ignorados, 1155 JS, check/TypeScript/Vite e 336 ensaios
privados no anfitrião. Continuam por concluir a aceitação da app isolada, a
equidade/retoma durante aprovação e após perder o dono, os restantes adaptadores,
operadores/verificadores e a entrega. Os 43 requisitos e a estimativa global
mantêm-se; o percurso privado inicial não equivale ao produto completo.
[Provas atuais](PETS-VALIDATION.md#ciclo-nativo-local778). Sem commit/push;
instalação .30 intacta.

**Paridade de retoma — correção posterior, mesma contagem778:** o adaptador
Codex deixa de reenviar permissões mais amplas ao retomar um thread existente.
A opção de acesso total só se aplica à criação explicitamente autorizada; a
retoma conserva a política e o nome nativos. As quatro variantes criação/retoma
com a opção ligada/desligada passaram no protocolo sintético. Biblioteca
2426/20, check e 416 ensaios privados no anfitrião; web sem alterações.
[Provas e âmbito](PETS-VALIDATION.md#retoma-nativa-codex-local778). Continuam
necessárias a paridade integral dos adaptadores e a aceitação/entrega; nenhum
pet ganha permissões por herdar código comum.

**Incremento779 — prova adicional e bloqueador de aceitação identificado:** três
regressões protegem o comportamento já correto de aprovação/ferramentas após
expirar a observação do modelo. Passaram 2429 Rust/20 ignorados, check e 419
ensaios privados no anfitrião; web sem alterações. No ensaio Tauri real isolado,
a consulta concorrente de uma aprovação reproduziu o encerramento prematuro do
pedido por disputa de um bloqueio de provas. A correção e a aceitação completa
estão em curso na mesma unidade; não se considera RUN/EXE fechado.
[Provas e limites](PETS-VALIDATION.md#observacao-expirada-local779).

**Correção posterior779 — aprovações concorrentes e aceitação Tauri:** a disputa
transitória das provas conserva o pedido original e exige validação atual para
aceitar/consumir a decisão. Leituras coexistem; alterações continuam exclusivas.
Cinco testes adicionais cobrem essa fronteira. Passaram 2434 Rust/20 ignorados,
check e 424 ensaios privados no anfitrião. A aplicação isolada percorreu Tauri
real, duas aprovações/receitas, feedback/replaneamento, conclusão verificada,
recarga da vista, Stop durante HTTP e eventos nativos. Modelo/leitor sintéticos;
paridade real, janela fechada, voz, recuperação e entrega continuam pendentes.
Segue a cedência/retoma autenticada do recurso durante espera de aprovação;
RUN/EXE e o objetivo completo permanecem abertos. Sem commit/push ou mudança da
.30. [Provas e limites atuais](PETS-VALIDATION.md#aprovacoes-concorrentes-local779).

**Incremento783 — cedência/retoma durante aprovação integrada:** o dono original
cede o recurso após prova de zero workers e transição assinada confirmada.
Outros objetivos podem reservá-lo; a aprovação recebida fica retida enquanto
espera e a reaquisição exige nova geração/ACK, com a mesma lease/prazo/orçamento.
Falhas de ACK reconciliam o estado nativo original. Stop e provas antigas não
libertam outra reserva. Seis regressões, 2440 Rust/20 ignorados, 1155 JS,
check/TypeScript/Vite e 430 ensaios no anfitrião passaram. O Tauri real privado
verificou outra reserva, espera aprovada, retoma, execução/replaneamento e
conclusão, com modelo/leitor sintéticos. Não fecha RUN-05: prioridades e justiça
geral da fila continuam por completar. Seguem recuperação após perda do dono e
as restantes capacidades/aceitação/entrega. Instalação .30 intacta; sem
commit/push. [Âmbito atual](PETS-VALIDATION.md#cedencia-e-retoma-local783).

**Incremento759 integrado — entrada funcional de objetivos RUN-01/02/EXE-02:**
a área existente de Tarefas passa a chamar criação/contexto/admissão/claim/resposta/
receita/consulta/Stop nativos. O formulário usa identidades da fábrica, prepara
contexto dedicado e conserva a revisão e os limites originais. A gravação incerta
interrompe a sequência de chamadas; abrir/atualizar a vista não inicia inferência,
ações ou aprovação. Os controlos permitem propor uma sequência de música e os
critérios finais explícitos, sem converter a proposta em permissão. A gestão tem
transporte de decisões apenas de objetivos; não ganha acesso às respostas de
permissões nativas de terminais. A inspeção explícita de metadados Ollama resolve
a preparação sem inferência, com alvo/configuração locais e cancelamento; o
modelo resolvido é selado na admissão, sem trocar a identidade do dono. Uma
reserva guardada sem controlador vivo não dá origem a outra execução.

Foram aplicados 20 ficheiros com 15 pré-imagens e cinco fontes novas. Passaram
2370 Rust/20 ignorados, 1153 JS, check/TSC/Vite e 274 ensaios privados no anfitrião;
cinco helpers do runtime são invocados pelos pais, sem novos avisos/modelos/quota.
O controlador e os controlos funcionais têm fixtures; os metadados e o broker
exercitam IO nativo privado. Não se comprovou o ciclo gráfico completo da app,
microfone/TTS, inferência real ou entrega. A app instalada continua na .30.
[Aceitação e limites](PETS-VALIDATION.md#evidencia-local).
[Checkpoint](PETS-VALIDATION.md#evidencia-local).

A mesma unidade continua: planeamento/replaneamento limitado, despacho nativo,
restantes operações/verificadores/adaptadores e aceitação operacional/entrega.
As revisões v2 selam os planos originais; replanear exige uma transação nativa
explícita que preserve o orçamento comum, e não remover a guarda dos planos ou
das chamadas internas das CLI. Os 43 requisitos permanecem no âmbito.
[Contrato seguinte](PETS-VALIDATION.md#evidencia-local).

**Incremento754 integrado — decisões de receitas RUN-02/EXE-02/VOICE:** origem
própria `goal` com alvo separado das sessões, fonte nativa retida e revalidação de
contexto/controlador/revisão/Stop/configuração/prazo. Um pedido partilhado entre
vistas e voz é consumido uma vez; a voz exige código explícito e o pet correto.
A revisão completa não altera conversa/rascunho, não concede planeamento e não
inicia a execução. A composição passou 2365 Rust/20 ignorados, 1138 JS, check/TSC/
Vite e 269 ensaios privados no anfitrião; 19 ficheiros aplicados e 17 pré-imagens,
sem novos avisos/modelos/quota. API compilada e percursos privados de broker,
controlos e execução posterior; ciclo gráfico, microfone/TTS e aceitação real
continuam pendentes. A mesma unidade prossegue com adoção real, operações/
verificadores/adaptadores e planeamento/replaneamento; os 43 requisitos continuam
abertos à aceitação aplicável. A app instalada permanece .30.
[Aceitação e limites](PETS-VALIDATION.md#evidencia-local).
[Checkpoint](PETS-VALIDATION.md#evidencia-local).

**Incremento752 integrado — critérios/conclusão nativa RUN-03/EXE-03:** receita v2
associa índice/hash dos critérios a predicados revistos; observação final nativa
sob o mesmo recurso/worker. Cobertura desconhecida ou estado diferente não conclui.
A prova imutável liga o registo original à conclusão schema7 e à revisão de
destino; recuperação não repete ações. Leitura nativa valida a cadeia/autoridade,
compatibilidade dos formatos anteriores preservada. 27 ficheiros com pré-imagens,
2359 Rust/20 ignorados, 1135 JS, check/TSC/Vite e 263 ensaios privados no anfitrião.
A MESMA unidade prossegue com revisões utilizáveis em chat/voz, outras operações/
verificadores/adaptadores e planeamento/replaneamento. Migração/backup/rollback,
aceitação nativa real e entrega continuam pendentes; RUN/EXE não fecham.
[Checkpoint](PETS-VALIDATION.md#evidencia-local).
[Continuação](PETS-VALIDATION.md#evidencia-local).

**Incremento749 integrado — primeiro kernel RUN-03/EXE-02:** revisão da sequência
exata, manifesto imutável e concessões individuais; ativação só depois de
publicação confirmada. Execução ordenada com worker/reserva física durante a
receita, orçamento comum, saída repartida e progresso durável, sem repetir
passos verificados. Revogação, Stop e incerteza interrompem os passos restantes.
16 ficheiros com pré-imagens, 2342 Rust/19 ignorados, 1133 JS, check/TSC/Vite e
246 ensaios privados no anfitrião, sem novos avisos/modelos. A MESMA unidade
continua aberta: critérios tipados e observação final, conclusão nativa durável,
aprovações utilizáveis em chat/voz e expansão de operações/adaptadores pelos
limites reais. Nesse kernel, `goalComplete` era falso; RUN/EXE continuam abertos.
[Checkpoint](PETS-VALIDATION.md#evidencia-local).
[Continuação](PETS-VALIDATION.md#evidencia-local).

**Incremento744 integrado — RUN-04/EXE-03:** prova imutável da observação nativa
antes da liquidação/recibo final e recuperação explícita da mesma tentativa,
sem novo efeito. Consulta coerente de fases, orçamento e prova; publicação
interrompida, pausa/outro processo, dados estrangeiros/adulterados e falta de
prova têm regressões. Reservas incertas, âmbito histórico e estado do objetivo
são preservados. 12 ficheiros com pré-imagens, 2330 Rust/18 ignorados, 1131 JS,
check/TSC/Vite e 234 ensaios privados no anfitrião, sem novos avisos. Segue ciclo
limitado de execução/progresso, concessões para planos/receitas e critérios,
com expansão pelos adaptadores conforme suporte real. Casos sem prova nativa,
retoma/replaneamento, autonomia integral e entrega continuam abertos.
[Prova](PETS-VALIDATION.md#evidencia-local).

**Primeiro kernel RUN-01 verificado:** repositório privado por objetivo, separado
do histórico de dez tarefas, com intenção completa limitada sem truncação,
critérios/dono/contexto, tarefas e planos propostos, passos, eventos e importação
conservadora de referências/recibos antigos. 650 fontes: 1792 Rust/12 ignorados,
1074 JS e check/TSC/Vite. A publicação usa revisão/idempotência e recibos de
durabilidade/incerteza; abrir não executa. Comandos nativos e bridge existem,
mas faltam integração/aceitação, migração e admissão/fila/leases/execução/verificação.
RUN-01 e os restantes requisitos RUN continuam abertos.

**Incremento710 integrado:** propostas de operação por ID/versão/argumentos nos
passos, revisão nativa exata e selo do conteúdo estático na admissão ativa.
O writer textual recebe intenção, critérios e propostas revistas com limite
explícito, sem expor o envelope na pergunta visível. 2106 Rust/16 ignorados,
1107 JS e 155 casos privados no host, sem inferência/providers reais. A admissão
continua `conversationRespond`; propostas e orçamento não concedem ferramentas.
RUN/EXE permanecem abertos para autoridade própria, dispatch/recibos de operações,
alvos/recursos e pós-condições verificadas.
[Evidência](PETS-VALIDATION.md#evidencia-local).

**Incremento702 integrado:** recibos autenticados de respostas, recuperação
idempotente sem nova inferência e consulta nativa após pausa/reinício. Validado
com 2077 Rust/16 ignorados, 1105 JS e 14 ensaios privados no anfitrião; o helper de
outro processo é executado pelo teste pai. O percurso atual é textual Ollama;
os contratos de recibo são comuns e não conferem execução ou conclusão de
critérios. Continuam abertos planos, ferramentas, limites/decisões nativos e
recuperação de efeitos externos. [Evidência](PETS-VALIDATION.md#evidencia-local).

### EXE — ferramentas, aprovações e verificação

**Incremento717 integrado:** primeiro passo Spotify play/pause/volume ligado ao
objetivo, com autorização própria, consumo único, recurso físico, orçamento comum
antes do comando, observação independente e recibo autenticado. Consulta após
pausa/noutro processo sem repetir efeitos. 2134 Rust/17 ignorados, 1109 JS e 174
ensaios privados do runtime no host. O caso inicial usa objetivo Ollama já possuído;
UI de aprovação, restantes executores, receitas, reconciliação explícita, critérios
e ciclo autónomo continuam pendentes. Nenhum requisito EXE/RUN é fechado apenas
por este incremento. [Evidência](PETS-VALIDATION.md#evidencia-local).

**Incremento707 integrado:** primeiro broker das cinco operações interativas já
oferecidas aos modelos, com schema/validação comuns, dono/concessões/cancelamento
e saída limitada com entrega tipada. 2092 Rust/16 ignorados, 1105 JS e 25 ensaios
privados no host; nenhum teste com dispositivos/providers reais. Não fecha os
requisitos EXE: faltam os restantes adaptadores, autoridade e ledger por objetivo,
idempotência, alvos/recursos vinculados e verificação independente de efeitos.
[Evidência](PETS-VALIDATION.md#evidencia-local).

| ID | Trabalho | Dependência e aceitação |
| --- | --- | --- |
| EXE-01 | Registo/broker de operações tipadas para ferramentas nativas, locais, MCP e desktop | PET-01/02. Schema, versão, efeitos, alvos, limites, cancelamento, idempotência e verificação; ferramenta descoberta não fica automaticamente autorizada |
| EXE-02 | Concessões por objetivo/regra e receitas limitadas; integrar aprovações existentes | RUN-01 + EXE-01. Uma aprovação não se torna shell genérica; política revogada invalida pedidos pendentes, sem conceder privilégios de outro pet. Chat/voz acompanham a política e os pedidos do agente original, com consumo único, sem duplicar pedidos já satisfeitos nem alargar a decisão nativa |
| EXE-03 | Recibos e verificação de pós-condições | EXE-01. Separar ACK/saída/efeito observado; erro após possível efeito exige reconciliação, sem repetir automaticamente |
| EXE-04 | Uso do computador e capacidades de observação | EXE-01/02. CLI/API/acessibilidade primeiro; alvos exatos, observação antes/depois em controlo visual. Sessão bloqueada ou falta de imagem tem limitação explícita |

### MEM e CTX — memória, documentos e fontes

| ID | Trabalho | Dependência e aceitação |
| --- | --- | --- |
| CTX-01 | Fechar anexos múltiplos, retenção e recuperação do protótipo de 633 fontes | Bloqueador REL-01. Envelope terminal e aceitação nativa de restauro parcial/nome reutilizado/limpeza interrompida/motivos de recusa verificados nos dois incrementos675 privados; completar integração e migração/backup/rollback v4/v5/leitores antigos |
| CTX-02 | Apresentação por mensagem e seleção de anexos para contexto | CTX-01. IDs/proveniência/cobertura visíveis onde necessário; não reautorizar cópias nem inferir leitura integral do modelo |
| MEM-01 | Schema de memória: preferências, contexto de projeto, episódios, derivados e resumos | PET-01 + RUN-01. Dono/âmbito/proveniência/validade/revisão; dados derivados não viram instrução nem autorização |
| MEM-02 | Recuperação seletiva, correção, exclusão, revogação e retenção | MEM-01 + CTX-01. Pesquisa estruturada/lexical inicial, isolamento entre pets/projetos e invalidação de derivados/índices; embeddings externos só com concessão própria |
| CTX-03 | Completar gestão e aceitação do catálogo ICS já implementado | Formulário/consentimento nativos e fonte selecionada; calendário exportado não aparece como sincronização ou capacidade de escrita |
| CTX-04 | Evoluir fontes de pet/CID para objetivos/regras e coleções persistentes | RUN-01 + EXE-02 + CTX-03. Migração sem alargar concessões antigas; seleção explícita de vários calendários, revisão e revogação por ocorrência |
| CTX-05 | Adaptadores RSS/IMAP com leitura limitada e proveniência | EXE-02 + CTX-04. TLS, redirecionamentos/destinos, bytes/prazos e mensagens limitados antes de materializar conteúdo; IMAP sem envio/eliminação implícitos |

**R42 integrado localmente em 834:** o arquivo de conversas conserva o formato.
Um catálogo próprio de preferências/notas usa o componente privado de CAS
extraído das fontes, com dono da fábrica, âmbito, proveniência, sensibilidade,
validade e revisões. As APIs de gestão nativa criam, pesquisam, corrigem e apagam;
conversa vazia conserva identidade e arquivo sem mensagens fabricadas. Passaram
2582 Rust/21 ignorados,1186 JS/TSC/Vite por identidade dos inputs, check/build,
751 ensaios privados e 13 percursos Tauri. O novo cobre 24 pets não retirados,
reinícios, concorrência, corrupção e preservação das fontes. Sete ficheiros
adotados, quatro novos; sem instalação ou commit.
[Provas e limites](PETS-VALIDATION.md#memoria-local834).

**Segue R43 — seleção e contexto de memória:** captura explícita por IDs/revisões,
origem, validade e destino; API comum nos transportes aplicáveis e vínculo à
admissão dos objetivos antes do envio. Revogação/exclusão invalida nova
reutilização pela app; contexto já entregue a uma CLI/fornecedor exige limites e
tratamento próprios. Atualizar o descritor sem confundir gestão humana e
capacidade dos motores. Projetos, episódios/derivados/resumos, fontes/índices,
partilha e aceitação integral MEM-01/02 continuam no âmbito.

**R43 em curso, fora da raiz834:** incremento840/E1 com 2606 Rust/21 ignorados,
35 testes de memória, check/build e provas Tauri na mesma fonte. Os três
adaptadores CLI e um serviço passaram 17 pedidos sintéticos, incluindo reinício,
revogação/recolha, ID desconhecido e falha de adoção antes do prompt. Os oito
pedidos locais anteriores passaram como regressão. Faltam os três envios API
sintéticos, a seleção/contexto dos objetivos antes da admissão/hash canónico,
confirmação humana e descritores. Nenhuma aceitação MEM/RUN é fechada por este
incremento. [Provas e limites](PETS-VALIDATION.md#contexto-memoria-r43).

### AUTO — rotinas e proatividade

| ID | Trabalho | Dependência e aceitação |
| --- | --- | --- |
| AUTO-01 | Repositório de regras, gatilhos e concessões; ativar/desativar por utilizador | RUN-01/02 + EXE-02 + CTX-04. Regra tem dono, revisão, fontes, limites e validade; conteúdo externo não inscreve regras |
| AUTO-02 | Agendador de horários/eventos e deduplicação de ocorrências | AUTO-01. Fuso IANA, mudança de hora, suspensão/offline e políticas skip/coalesce/decisão; nenhum efeito duplicado por evento repetido |
| AUTO-03 | Primeira rotina: preparação diária a partir de calendários escolhidos | AUTO-01/02 + EXE-03. Dados sintéticos desconhecidos pelo modelo, várias fontes, frescura/cobertura, falha parcial e uma única notificação no horário permitido |
| AUTO-04 | Rotina de acompanhamento de objetivo/projeto e pedido de decisão | RUN-03/04 + MEM-02. Progresso verificável, interrupção/retoma sem repetir efeitos, orçamento repartido e comunicação sem spam |
| AUTO-05 | Notificações, silêncio e escalada de bloqueios | OBS-01 + AUTO-01. Conteúdo sensível limitado, deduplicação e validade; indisponibilidade não causa reenvio infinito ou mudança de canal/conta |

### TEAM — cooperação entre pets

| ID | Trabalho | Dependência e aceitação |
| --- | --- | --- |
| TEAM-01 | Protocolo comum de comunicação direta, descoberta e delegação | PET-01/02 + RUN-01 + EXE-02. Mensagens/pedidos/respostas/eventos/resultados com origem/destino, correlação/causa, tarefa/objetivo, idempotência, artefactos, autoridade, orçamento e prazo explícitos; reutilizar mecanismos existentes. Utilizador e pets chamam cada pet diretamente; nenhuma delegação amplia permissões |
| TEAM-02 | Execução paralela/assíncrona, verificação, cancelamento e coordenação resiliente | TEAM-01 + RUN-03/04 + EXE-03. Mesmo controlo para tarefas diretas/delegadas; um dono por efeito, entrega deduplicada, limites de profundidade/concorrência/custo, ciclos/conflitos recusados e falhas isoladas. Coordenador Ollama configurável, com visão autorizada dos trabalhos, sem intermediação obrigatória; restantes pets operacionais sem coordenador/motor indisponível. Resultados com proveniência, sem copiar o histórico inteiro |

O caminho de preparação de texto `mochi/delegation.ts` não fecha TEAM-01/02.
Os ensaios têm de demonstrar mensagens diretas, ramos independentes, ausência do
coordenador, motores locais descarregados, ACK perdido, ciclos, conflitos e
revogação, sem permissões herdadas. A separação entre pet/runtime/motor é também
critério de PET-01/04/05 e RUN-01/02, sem acrescentar IDs ou apagar os 43 originais.

### SYS, INT e VOICE — expansão funcional

| ID | Trabalho | Dependência e aceitação |
| --- | --- | --- |
| SYS-01 | Matriz pet × subplugin × operação × recurso; autoridade dos pets ligada ao executor comum | PET-01 + EXE-01/02. Plugin Sistema não concede tudo; Som pode ser partilhado por Spotify/Ollama; revogação de um pet preserva o outro |
| SYS-02 | Rede e Bluetooth como uma unidade de execução | SYS-01 + EXE-03. Estado/scan/alvos/prompts/perfis e operações suportadas; distinguir implementação/ACK/efeito. Sem desligar Wi-Fi nem alterar dispositivos em QA sem autorização específica |
| SYS-03 | Som/multimédia; ecrã/energia/sessão em subunidades completas | SYS-01 + EXE-03. Streams/dispositivos/perfis e restante âmbito da arquitetura; partilha/concorrência verificadas e indicadores preservados |
| INT-01 | Integrações de serviço e comunicação externa com operações distintas | PET-05 + EXE-02/03. Abrir rascunho não envia; envio futuro exige concessão própria. Autenticação/tokens e identidades nativas preservados |
| VOICE-01 | Controlo e interação integral do sistema de pets por voz | PET-02/03 + EXE-01/02 + CTX-04 + SYS-01. Cobertura por operação de pets/sessões/conversas/objetivos/tarefas/regras/fontes/memória/configuração/modelos e Sistema autorizado; mesmos dono, permissões e pedidos nativos que chat/terminal. Decisão única por pedido, clarificação falada de ambiguidade, Stop prioritário e sem reenvio após reconexão |
| VOICE-02 | Aceitação real de microfone/Bluetooth/TTS PT/EN/misto | VOICE-01. Latência/pronúncia/alternância/falhas, conversa contínua, interrupção de TTS e reconexão com áudio real; matriz de cobertura funcional por voz. Uma transcrição escrita não fecha este requisito |

### OBS, DATA e REL — continuidade e entrega

| ID | Trabalho | Aceitação |
| --- | --- | --- |
| OBS-01 | Estado, decisões, prazos, recursos e consumo por tarefa/pet | Distinguir terminais/TTYs, clientes e sessões, com atividade/espera/suspensão/inatividade confirmadas ou desconhecido; lease preparado não é inferência. Fontes/cobertura explícitas, sem segredos/documentos privados em logs |
| OBS-02 | Cobertura/rotação dos ledgers e captura de consumo nativo | Separar tentativas/thread/conta/quota; não perder notificação antes do ACK, não somar snapshots como deltas e não autenticar por abrir painel |
| DATA-01 | Repositórios, migrações, retenção e backups | Falhas antes/depois de publicação, escritor concorrente, reinício e restauro; nenhum dado ativo desaparece por rotação |
| DATA-02 | Decisão e eventual backend PostgreSQL | Benefício medido e esquema/base/utilizador próprios; IDs e concessões preservados, migração/restauro comprovados; estado atual continua sem integração |
| REL-01 | Composição, adoção das fontes e migração/rollback | Fontes/hash exatos, preservação de dados/configuração/rascunhos e resolução de bloqueadores; journal v5 e marcadores pendentes tratados antes de regressar a leitores antigos |
| REL-02 | Entrega e aceitação integrada por capacidades | Executável/assets/helper/autostart/login/reinício, hotplug e trabalhos interrompidos; paridade funcional dos centros legados antes de descontinuar capacidades, mantendo os indicadores. Rádio/dispositivos/contas respeitam o âmbito autorizado |

## 3. Matriz de aceitação dos pets

Cada célula tem estado e prova próprios: `desenhado`, `implementado`, `fixture`,
`nativo`, `integrado`, `entregue`, `indisponível` ou `pendente`. Estes são os
campos obrigatórios do relatório de aceitação, não uma declaração de que todos
já foram preenchidos ou passaram.

| Capacidade | Codex | Gravity | Copilot CLI | Ollama | Outros locais/API | Serviços |
| --- | --- | --- | --- | --- | --- | --- |
| Identidade, modelo e configuração | PET-01/02 | PET-01/02 | PET-01/02 | PET-01/04 | PET-01/04 | PET-05 |
| Conversa, arquivos e anexos | PET-02, CTX-01/02 | Idem | Fixtures sem quota real | Idem | Só rotas configuradas e suportadas | Só se a integração declarar conversa |
| Sessão externa/decisões | PET-03 | PET-03; lacuna de aprovação explícita | PET-03; fixtures | Não pressupor TUI nativa | Conforme adaptador | Conforme executor declarado |
| Ferramentas e efeitos | EXE-01/03 nativos | Canal nativo aplicável | Fixtures e decisões nativas | Ask/modelo com tools | Conforme capacidade real | Operações tipadas |
| Objetivo/background/recuperação | RUN-01/04 | RUN-01/04 | Fixtures; nativo pendente | RUN-01/04 | RUN-01/04 se declarado | Contrato comum no executor |
| Regras/fontes/memória | MEM/CTX/AUTO | MEM/CTX/AUTO | Fixtures | MEM/CTX/AUTO | Conforme declaração | Apenas fontes/operações concedidas |
| Stop/revogação/concorrência | Obrigatório | Obrigatório | Fixtures; quota respeitada | Obrigatório | Obrigatório nas capacidades declaradas | Obrigatório |
| Voz/áudio real | VOICE aplicável | VOICE aplicável | Fixtures sem quota real | VOICE-01/02 | Conforme rota suportada | Comandos determinísticos aplicáveis |

Metadados sintéticos dos oito percursos de provider já foram exercitados no
protótipo de anexos. Isso não prova oito pets configurados, inferência real ou
aceitação nativa destas linhas.

## 4. Percursos de conclusão do sistema

| Cenário | Requisitos que o fecham |
| --- | --- |
| Preparação diária útil | PET-01/02, RUN, EXE-02/03, CTX-03/04, AUTO-01/03, OBS-01 |
| Objetivo que continua com a janela fechada | PET-02/03, RUN-01/04, EXE-01/03, OBS-01, REL-01/02 |
| Vários documentos com memória e retenção | CTX-01/02, MEM-01/02, DATA-01, REL-01 |
| Reinício com efeito incerto | RUN-04, EXE-03, DATA-01, REL-02 |
| Assistente falado | PET-02/04, VOICE-01/02, RUN/EXE aplicáveis |
| Pets a colaborar sem partilhar privilégios | TEAM-01/02, PET-01/02, EXE-02, OBS-01 |
| Spotify/Ollama partilham apenas Som autorizado | SYS-01/03, PET-05, EXE-02/03 |

## 5. Limites da declaração de conclusão

As exceções por quota, falta de dispositivo ou autorização de teste devem ficar
visíveis com o percurso exato por aceitar. Não se declara aceitação realizada
para contornar uma dessas condições. Também não se reduz o objetivo completo
aos cenários que já passam na máquina atual.

A janela de falha da limpeza final dos metadados de retenção continua aberta em
CTX-01. A sua correção é prévia à ativação dessa composição; não impede desenvolver
PET-01 em âmbito separado e coerente. Melhorias visuais históricas permanecem
adiadas. Requisitos detalhados preservados no arquivo são associados à unidade
que os implemente antes de retirar uma capacidade existente.
