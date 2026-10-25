# Validação da composição dos pets — 25/10/2026

<a id="objetivo-memoria-r43f1a"></a>

## Preparação de memória para objetivos — R43F1A privado

**Atualização de estado R43F1A — 25/10/2026:** no SDK privado841 passou a preparação durável de memória selecionada para objetivos: referências exatas por pet/conversa de origem, contexto versão2 e objetivo versão11 apenas quando há seleção, sem copiar os dados brutos para o diálogo inicial. Passaram 2612 testes Rust/21 ignorados, check/build e o percurso Tauri privado130, incluindo reinício, revogação e recusa do leitor840 sem alterar os ficheiros. Não houve chamadas a modelos ou quota. A admissão autónoma com memória permanece deliberadamente recusada até R43F2 ligar revisão humana dos dados/destino, consentimento próprio do objetivo e mensagem anterior ao hash/reserva canónicos. A raiz aceite permanece834; a app instalada permanece .30. R43 e os 43 requisitos continuam abertos.

Mapa de fontes congeladas `source-goal-memory841-r43f1a.json`: 841 entradas de fonte e 481 inputs nativos de compilação. A composição conserva o bundle web42; não houve alteração web que justificasse repetir os 1186 testes JS/TSC/Vite já vinculados aos mesmos inputs. Os 16 testes dirigidos de contexto incluem seis novos casos de memória. A suite completa passou 2612/21 ignorados. Os resultados originais de teste, check e build foram consumidos sem modificar os inputs.

Prova nativa: `.build/pet-goal-execution-next-r1/APP-NATIVE-GOAL-MEMORY-PREPARATION-R43F1A.json`. IPC Tauri real em WebKit/Xvfb privado, transportes e dados sintéticos, sem display/perfil da estação. O leitor anterior840 recusou leitura/preparação/mutação dos dados novos e conservou os bytes; o leitor novo restaurou-os. O ensaio cobre seleção, âmbito, repetição, exclusão e barreira temporária à admissão; não cobre envio de memória num objetivo, consentimento funcional da interface ou execução F2. Não há ativação da instalação ou aceitação MEM/RUN integral.

O checkpoint atual é [local834 com gestão explícita de memória](#memoria-local834), sem commit/push
ou instalação. As composições anteriores conservam o seu âmbito histórico.

<a id="contexto-memoria-r43"></a>

## Contexto de memória R43 — desenvolvimento privado, não entregue

A raiz mantém o checkpoint834. O núcleo privado837/R43B passou **2592 Rust/21
ignorados**, cargo check e build. Na app Tauri/WebKit privada, o leitor834 recusou
leitura/reset/envio de versão7 sem alterar o ficheiro nem iniciar inferência.
O leitor837 preservou histórico/proveniência após escrita por outro pet, arquivo
e reinício; recusou retoma nativa ainda sem controlador e permitiu os chats
comuns. São provas do codec nessa composição, não uma ativação dos transportes.

**Incremento838/R43D em SDK privado:** passaram **2600 Rust/21 ignorados**, cargo
check e build, com 478 inputs nativos imutáveis. Os 29 testes de memória incluem
isolamento dos cinco donos sem sessão remota, revisão de um só turno mesmo entre
clones do canal, parâmetros reais alterados, exclusão durante IO e supervisão
durante espera/execução de comandos. O supervisor pertence à reserva existente,
usa referências fracas e termina com ela; não mantém um recurso físico próprio.
Admissão de passos e classificação terminal também verificam a revogação.

O percurso Tauri nativo126 passou com **Ollama e LM Studio sintéticos**. A revisão
exata enviou uma mensagem temporária de dados; pergunta alterada, token repetido,
cancelamento e exclusão antes do envio recusaram sem inferência. O histórico e
o journal público não receberam os registos brutos, e o reinício não os voltou
a incluir automaticamente. Na ponte nativa de aprovação, revogar a memória
retirou o pedido pendente sem iniciar comando. Após uma aprovação exata, revogar
com o comando privado em execução sinalizou-o e confirmou a sua recolha antes
de libertar o recurso; o estado conservou `stopped`/recibos, sem nova inferência.
Foram oito pedidos sintéticos loopback, zero quota real, sem perfil/display do
anfitrião montado e sem alterar a instalação .30.

**Incremento840/R43E1 — contexto nativo comum às CLI:** passaram **2606 Rust/21
ignorados**, 35 testes de memória, check/build e 480 inputs nativos imutáveis.
A revisão humana `NativeSessionContext` suporta as três contas e serviços
explicitamente associados. O mesmo ponto de entrada de execução aplica a
fronteira a texto, voz e retoma, por construção do código; não é prova de áudio.
Referências antigas são validadas sem reenviar registos brutos; novos dados são
unidos sem esquecer referências anteriores. Destino/modelo/pasta/ID diferentes
não criam outra sessão nem modificam as permissões do agente original.

Tauri privado128 passou **Codex, Copilot, Gravity e Notion com assistência Codex**.
Os 17 pedidos dos protocolos sintéticos comprovaram proveniência durável antes
do prompt, adoção exata do ID, histórico sem registos brutos, token de uso único
e recusa de destino alterado. Após reinício, cada dono retomou a sessão original
sem repetir o prefixo de memória. Revogação durante espera nativa cancelou os
quatro turnos e confirmou recolha dos CLI próprios; o serviço Codex conservou
estado independente do pet Codex. Nova conversa explícita manteve os arquivos
revogados. A proveniência Gravity sem ID confirmado sobreviveu ao erro/reinício
e impediu criação silenciosa de outra sessão. Falhar a escrita do ID nos callbacks
Codex/ACP recusou o prompt com dados, conservando a indicação de possível entrega.

A fixture127 esperava incorretamente sucesso quando omitia o ID Gravity: o
transporte já recusava essa identidade inválida. A prova128 corrigiu só a
expectativa e repetiu o percurso completo na mesma fonte/binário. A falha127
continua preservada. O percurso129 repetiu, na mesma composição840, os oito
pedidos locais e a revogação durante aprovação/comando do ensaio126. O scriptD
reutilizado conserva uma anotação descritiva antiga de CLI pendente; essa frase
não é uma asserção e não substitui a prova128. A admissão autónoma permanece
pendente. São fixtures de protocolo, sem contas/providers/voz reais nem quota;
a cobertura da conta real é explicitamente `originalNativeAccountNotObserved`
na revisão de dados. A política e a conta do agente original continuam nativas.
A raiz aceite continua834;840 não foi adotado nem instalado.

Anthropic/OpenAI/Google têm a ligação escrita no canal comum e ensaios dos seus
donos/parametrização; **não têm ainda prova do envio HTTP real dos seus adaptadores
com credenciais/servidores sintéticos**. Não confundir esse âmbito com os dois
transportes locais acima. A superfície humana de seleção/confirmação também
continua por ligar. O incremento não foi adotado na raiz nem instalado.

**Falhas preservadas:** a suite838/C teve duas falhas no runtime (`Invalidated`
numa receita e `Busy` na projeção de Stop). Os mesmos dois testes passaram
isoladamente com o mesmo binário no anfitrião privado, e a suiteD passou inteira;
a causa original não ficou determinada. Não atribuir uma correção causal à nova
supervisão de memória. A compilaçãoC foi interrompida durante a pausa explícita;
o processo do container já não existia e a limpeza foi confirmada, sem declarar
essa compilação aprovada. O primeiro preflightD parou antes de lançar a app
privada porque o PID histórico da instalação já tinha sido substituído. A prova126
capturou o PID atual2730749 e confirmou a sua preservação.

**Pendências R43:** envio online com credenciais/servidores sintéticos,
vínculo à revisão/admissão/recibos dos objetivos, confirmação humana funcional e
projeção das capacidades. A auditoriaF confirmou que o contexto isolado, a revisão,
o hash de admissão e a fronteira canónica do writer já existem: a seleção de
memória deve entrar nesses mecanismos antes do envio. O `ContextFence` é também
verificado durante efeitos/decisões pelo controlador comum. Não reutilizar um
token manual para autorizar rondas autónomas, trocar o CID de origem pelo CID
privado do objetivo ou acrescentar dados depois do hash/consentimento.
Nenhuma destas provas fecha MEM-01/02, os 43 requisitos ou a entrega. O frontend
mantém prioridade apenas para bloqueadores funcionais, aprovações, preservação
e entrega.

<a id="memoria-local834"></a>

## Memória local834 — gestão explícita e vínculo durável à conversa

As APIs nativas `pet_memory_list` e `pet_memory_mutate` gerem preferências/notas
com identidade da fábrica e âmbito exato de pet ou conversa. Proveniência é de
utilizador explícito; sensibilidade, validade e retenção são tipadas. Criação,
pesquisa lexical, páginas com registos inteiros/revisão vinculada, correção e
exclusão respeitam a revisão do catálogo e do item. A validade expirada suprime
disponibilidade e continua consultável explicitamente. A exclusão também pode
remover um registo exato cujo CID já desapareceu, sem criar/reassociar dados.

O componente privado de CAS foi extraído das fontes com adaptador tipado,
conservando o seu schema1, limites, nomes e erros. Memória usa um catálogo próprio
`assistant/memories.json` no payload admitido, até 1 MiB/256 itens; cada conteúdo
tem até 8192 bytes. Leituras ausentes não criam dados. Corrupção, ligações,
permissões inválidas, conflitos e revogação da revisão preservam o original.
O catálogo não é ferramenta de modelo e não inclui conteúdo nos pedidos.

O controlo C na app real privada reproduziu memória inacessível após perder um
CID vazio que só existia em RAM. D conserva essa identidade pelo escritor de
diálogo durante a revisão explícita, após validar input/revisão. Um pin em RAM
protege reset concorrente; depois de reiniciar, o catálogo preserva o arquivo
da conversa vazia. Catálogo ausente mantém o comportamento anterior; catálogo
inválido conserva a identidade por precaução. Não se inventam mensagens nem
se altera o formato de conversa. Reset anterior à conservação pode recusar a
mutação; reset posterior conserva o âmbito original arquivado.

Passaram **2582 Rust/21 ignorados**, cargo check/build e **751 ensaios privados
no anfitrião**, incluindo 11 de memória,20 de fontes e 87 de dados. **1186 JS**,
TSC/Vite são reutilizados com comparação explícita: só sete ficheiros Rust
diferem da composição 830; todos os inputs web e bundle42 são exatos.
Snapshot849/834 fontes/474 inputs nativos. Foram adotados sete ficheiros, quatro
novos, com pré-imagens e HEAD/index/protótipo Gravity não registado intactos.

Passaram **13 percursos Tauri/WebKit privados**, com processo/IPC reais:

- Gestão de memória nos 24 pets não retirados do catálogo, incluindo serviços
  com o mesmo adaptador, sem exigir os seus motores/contas.
- Conversa vazia conserva o mesmo CID após reinício; Nova conversa arquiva o CID
  anterior sem mensagens fabricadas; outro reinício conserva acesso à memória.
- CRUD/pesquisa/CAS concorrente, páginas completas, histórico preservado nas
  operações ordinárias de memória e corrupção isolada sem impedir ler o chat.
- O adaptador anterior de fontes conserva schema/revisões/consentimento.
  Uma inferência sintética confirma ausência de inclusão automática de memória.
- Os12 percursos anteriores de recuperação, incerteza, Stop/continuação,
  LM Studio, continuidade CLI e mutações do perfil passaram na mesma fonte/build.

As falhas ficam conservadas: A falhou compilação de testes por visibilidade/
imports/inferência de tipo; B passou 2578 casos mas aceitou campos extra no
âmbito unitário Pet. C tornou esse codec estrito e passou 2579, mas a app reproduziu
a perda de CID. D acrescenta conservação e três testes de integração persistente.
Três regressões D de LM Studio recusaram corretamente um pet inativo: a preparação
esquecera `activeIntegrations/mainPill`. Só esses três casos foram repetidos em
perfis novos com a ativação prevista; fonte/binário/scripts permanecem exatos.
O agregado D original falhado e o agregado D2 corrigido são separados.

Provas privadas em `.build/pet-goal-execution-next-r1/`:
`SCOPED-MEMORY-INTEGRATED-CHECKPOINT-R42.json`, `ROOT-APPLICATION-SCOPED-MEMORY-R42.json`,
`SCOPED-MEMORY-STORE-DECISION-R42.json`, `SCOPED-MEMORY-SCOPE-CODEC-DECISION-R42C.json`,
`SCOPED-MEMORY-EMPTY-CID-DECISION-R42D.json`, `SCOPED-MEMORY-IMPLEMENTATION-FINDINGS-R42.json`,
`SCOPED-MEMORY-IMPLEMENTATION-AUDIT-R42D.json`,
`APP-NATIVE-SCOPED-MEMORY-EMPTY-CID-BASELINE-R42C.json`,
`APP-NATIVE-SCOPED-MEMORY-R42D.json`, `SCOPED-MEMORY-NATIVE-REGRESSIONS-R42D2.json`,
`SCOPED-MEMORY-LM-REGRESSION-FIXTURE-FINDING-R42D.json`,
`scoped-memory-full-r42d-RESULT.json`, `scoped-memory-check-r42d-RESULT.json`,
`scoped-memory-app-build-r42d-RESULT.json`, `SCOPED-MEMORY-WEB-REUSE-R42D.json`,
`HOST-NATIVE-SCOPED-MEMORY-R42D.json` e `SCOPED-MEMORY-DOCS-AUDIT-R42.json`.

Não houve quota/modelos reais, novos agentes, commits/pushes, instalação ou
alteração do anfitrião/perfil pessoal. A app .30 e o processo original ficaram
intactos. Inclusão no contexto dos motores, projetos, episódios/derivados/resumos,
fontes/índices/partilha, restantes providers, cooperação, voz integral, Sistema
e migração/entrega conservam os seus critérios. MEM-01/02 e o sistema completo
continuam abertos.

<a id="continuidade-local830"></a>

## Continuidade local830 — sessões CLI e diretórios vivos

O resolvedor comum valida dono/backend e identidade nativa guardada. Uma pasta
diferente ou alteração da preferência de criação Codex/Gravity recusa antes de
criar cliente/enviar prompt, preservando sessão e histórico. Repor a preferência
ou escolher Nova conversa são ações explícitas. Copilot continua a ignorar essa
opção global e conserva a política ACP. Gravity só acrescenta o bypass existente
na criação realmente nova; nunca na retoma. `full_access` histórico não prova
permissões correntes. Os rótulos de configuração esclarecem esse âmbito.

A app anterior reproduziu a substituição silenciosa nos três adaptadores e
num serviço assistido por Codex. A app corrigida passou retoma exata, recusa sem
prompt, reposição, Nova conversa com arquivo preservado e retoma dessa conversa.
São processos de protocolo privados ligados ao IPC Tauri/WebKit real, sem CLI
operacional, conta ou quota. A adoção/controlo de um terminal externo e os limites
das ferramentas internas das CLI mantêm aceitação própria.

Uma regressão nativa anterior expôs indisponibilidade do perfil com
`SourceChanged`/`OwnershipLost`. O controlo reproduziu a mesma classe: zero
consultas bem-sucedidas após 2466 mutações legítimas, mantendo marcadores e
dispositivo/inode/UID/modo. Não se reconstruiu o instante exato da falha original.
A correção separa o binding de diretórios vivos do snapshot de conteúdo das
fontes. Após a correção, 200 consultas passaram durante 583260 criações/remoções,
sem alterar marcadores ou identidade. Substituições, links, alterações de modo,
perda de lock e mudanças de conteúdo nas fontes continuam recusadas.

Passaram **2571 Rust/21 ignorados**, cargo check/build, **1186 JS**, TSC/Vite e
**720 ensaios privados no anfitrião**, incluindo 87 de dados/migração e 419 de
runtime. Snapshot845/830 fontes/470 inputs nativos, bundle42. O web A é reutilizado
em B com comparação explícita: só quatro ficheiros Rust nativos diferem; JS/TS,
JSON, fábrica e assets são idênticos. Foram adotados dez ficheiros, dois novos,
com pré-imagens, HEAD/index e protótipo Gravity não registado preservados.

Passaram **12 percursos Tauri privados**: os dez anteriores de incerteza,
Cancel/recibos/continuação/retirada/ciclo completo e LM Studio, mais os dois novos
de continuidade CLI e mutações do perfil. Incluem o cenário anterior de retirada
após cinco crashes que tinha falhado. Os dois controlos anteriores são provas
separadas de reprodução, sem serem contados como percursos positivos.

As falhas anteriores ficam conservadas: notificação ACP inválida na primeira
fixture; publicação não atómica do sinal JSON de pronto; erro nativo do perfil;
e caminho de socket D-Bus longo no primeiro ensaio de dados no anfitrião.
A fixture passou a publicar pronto por rename; a repetição no anfitrião usa
um diretório temporário privado mais curto, com as mesmas fontes/binário.
Nenhum timeout foi aumentado nem recusa de armazenamento substituída por retry.
O primeiro helper de adoção errou o caminho do index de um worktree, antes de
qualquer cópia; a versão corrigida usa `git rev-parse --git-path index`.

Provas privadas em `.build/pet-goal-execution-next-r1/`:
`NATIVE-CONTINUATION-INTEGRATED-CHECKPOINT-R41.json`,
`ROOT-APPLICATION-NATIVE-CONTINUATION-R41.json`,
`NATIVE-CONTINUATION-ARCHITECTURE-R41.json`,
`LIVE-PROFILE-DIRECTORY-ARCHITECTURE-R41B.json`,
`NATIVE-CONTINUATION-IMPLEMENTATION-FINDINGS-R41.json`,
`APP-NATIVE-CONTINUATION-BASELINE-R41B.json`, `APP-NATIVE-CONTINUATION-R41B.json`,
`APP-NATIVE-LIVE-PROFILE-BASELINE-R41A.json`, `APP-NATIVE-LIVE-PROFILE-R41B.json`,
`NATIVE-CONTINUATION-NATIVE-REGRESSIONS-R41B.json`,
`native-continuation-full-r41b-RESULT.json`,
`native-continuation-check-r41b-RESULT.json`,
`native-continuation-app-build-r41b-RESULT.json`,
`native-continuation-web-r41a-RESULT.json`, `NATIVE-CONTINUATION-WEB-REUSE-R41B.json`,
`HOST-NATIVE-CONTINUATION-R41B.json`, `HOST-NATIVE-R41B-SOCKET-FIXTURE-FINDING.json`
e `NATIVE-CONTINUATION-DOCS-AUDIT-R41.json`.

Não houve chamadas reais a providers, quota Copilot/GitHub, novos agentes,
commits/pushes, mudança do anfitrião/perfil pessoal ou instalação. A app .30 e
o processo original ficaram intactos. Memória, fontes, cooperação, voz integral,
Sistema, restante autonomia e migração/entrega conservam os seus critérios.
Os 43 requisitos e o objetivo completo continuam ativos.

<a id="paridade-local828"></a>

## Paridade local828 — LM Studio com dono e autoridade próprios

Associação explícita `provider=lmstudio`, preservando provider vazio e preferências
antigas. A fábrica declara adaptador, chat e tarefas; voz/shell continuam sem
suporte. Metadados GET/api/v1/models limitados distinguem LLM, chave de modelo e
instância carregada. A ligação nativa autenticada inclui endpoint, chave,
instância e impressão de identidade/âmbito da credencial. Ausente, o novo campo
opcional conserva os bytes antigos. Revisão, confirmação, claim e HTTP comparam
essa ligação; cache de revisão não substitui a escolha por outra identidade.

Reutilizam-se contexto, transporte limitado, ledger, planeamento, receitas,
critérios, Stop, recuperação e continuação. Nenhuma descoberta carrega/descarrega
modelos ou faz inferência. JIT do servidor só pode ocorrer na inferência
explicitamente admitida. Uma credencial opcional tem entrada de cofre própria,
vinculada ao URL, com geração de autoridade em memória; nunca é copiada de outro
servidor nem exposta em preferências/revisões. Os metadados são uma afirmação do
fornecedor local, não atestação dos pesos.

Passaram **2564 Rust/21 ignorados**, cargo check e build da app, **1186 JS**, TSC,
Vite e **554 ensaios privados no anfitrião**. Snapshot843/828 fontes/468 inputs
nativos, bundle42. O web E é reutilizado em F com prova de que apenas mudou um
ficheiro cfg(test) Rust de compatibilidade de donos; produção e frontend são
exatos. Foram adotados 36 ficheiros, três novos, com pré-imagens e HEAD/index
intactos. A instalação .30 e o seu processo original ficaram preservados.

Passaram **dez percursos Tauri/WebKit privados**, com IPC e processo reais:

- Sete percursos anteriores de recuperação, recusa, Cancel, recibos ausentes/
  adulterados, continuação, retirada após crashes, replaneamento, Stop e locks.
- LM Studio: provider vazio não cria dono; associação por CAS não altera outros
  pets; nova execução após efeito desconhecido exige outra aprovação, conserva
  encargo/histórico e sobrevive a dois reinícios.
- Conversa manual antiga e conversa associada conservam ID/histórico; uma
  conversa Ollama independente não as altera. Pause interrompe o IO do modelo,
  conserva consumo desconhecido, não executa ferramentas e permanece após reinício.
- Modelo alterado após revisão recusa o token antigo sem grant/escrita/inferência.
  Alteração após claim é descoberta no GET real antes de inferir: nenhum débito,
  receita ou efeito; Pause explícito continua disponível.

As falhas iniciais de compilação/fixture e as expectativas antigas de dono foram
corrigidas e conservadas. O primeiro full F observou Timeout num teste existente
de helper400ms e depois faltou o seu marcador PID. A causa original não está
comprovada; o ensaio isolado e o full F2 passaram com as mesmas fontes e sem
alargar timeout. Ensaios adicionais corrigiram pressupostos das fixtures sobre
campos opcionais de ConversationView/admissions e finish_reason do stream; não
alteraram o produto. Os avisos do compilador constam das provas.

A autenticação HTTP usou credencial sintética capturada. Motor LM Studio/cofre
operacionais, outras APIs/CLI, voz real, restante autonomia/cooperação/Sistema e
migração/entrega continuam fora desta aceitação. Não houve quota real, novos
agentes, alteração do anfitrião, commits/pushes ou ativação da build.

Provas privadas em `.build/pet-goal-execution-next-r1/`:
`LMSTUDIO-INTEGRATED-CHECKPOINT-R40.json`, `ROOT-APPLICATION-LMSTUDIO-R40.json`,
`LMSTUDIO-NATIVE-ARCHITECTURE-R40.json`, `LMSTUDIO-IMPLEMENTATION-FINDINGS-R40.json`,
`lmstudio-full-r40f2-RESULT.json`, `lmstudio-check-r40f-RESULT.json`,
`lmstudio-app-build-r40f-RESULT.json`, `lmstudio-web-r40e-RESULT.json`,
`LMSTUDIO-WEB-REUSE-R40F.json`, `HOST-NATIVE-LMSTUDIO-R40F.json`,
`LMSTUDIO-NATIVE-REGRESSIONS-R40F.json`,
`APP-NATIVE-LMSTUDIO-CONTINUATION-R40F.json`,
`APP-NATIVE-LMSTUDIO-DIRECT-STOP-R40F3.json`,
`APP-NATIVE-LMSTUDIO-MODEL-CHANGE-R40F2.json`, `LMSTUDIO-DOCS-AUDIT-R40.json`.

<a id="incerteza-local825"></a>

## Continuação nativa local825 com incerteza histórica revista

`pet_runtime_prepare_continuation` conserva a recusa normal `unreconciledEffects`.
A opção explícita `reviewUncertainEffects:true` apenas pede uma revisão nativa.
Não recebe operações, grants, alvos, resultados ou manifestos do cliente. O backend
autentica cada tentativa desconhecida contra o seu passo/âmbito e os argumentos
originais. Apenas pausar, reproduzir e volume absoluto do leitor se qualificam;
origem ausente/ambígua, corrupção e operações relativas/desconhecidas recusam.

Uma referência de continuação schema2 conserva o manifesto exato revisto,
incluindo decisão/tentativa/grant/passo, hash do âmbito, operação e alvo original.
Schema1 sem o campo opcional conserva os bytes antigos. O registo antigo não é
reescrito ou reembolsado: permanece Started e desconhecido, com o mesmo encargo.
A aprovação admite apenas novo planeamento Ollama local SpotifyTransportV1 e
limites adicionais; cada receita atual continua a exigir a aprovação exata.
Não reconstrói a tentativa, lease ou controlador antigo, nem usa estado atual
como prova de autoria passada. O planeador recebe a incerteza como contexto.

O percurso Tauri/WebKit C executou uma pausa privada e sofreu uma falha real antes
de conservar evidência. Depois de matar/reiniciar o processo e fechar a execução,
a continuação normal recusou; o modo textual também recusou a opção. A nova
revisão mostrou a operação/âmbito e uma ferramenta desconhecida com16642 B de
consumo anterior. Recusar não escreveu. Outra escolha conservou a mesma aprovação
perante lock real Busy e grant publicado sem projeção no objetivo. Confirmar
completou essa transação uma vez; o ACK posterior não escreveu.

Nova lease, uma chamada sintética e outra receita explicitamente aprovada
executaram duas operações privadas e concluíram o mesmo objetivo. A admissão,
grant e consumo antigos ficaram exatos. O total passou a duas chamadas,
três ferramentas e17929 B, mantendo uma ferramenta histórica desconhecida.
A consulta antiga permaneceu Started/ResultMissing e contabilização congelada.
Outro crash/reinício conservou objetivo legível, histórico e sourceHash. Um
segundo percurso preparou a revisão e cancelou antes da confirmação: a escolha
antiga foi recusada sem novo grant/dono/IO. Cancel permaneceu terminal.

Três testes Rust novos verificam classificação de âmbito/evidência nativos,
recusa de argumentos alterados e passo corrupto/ausente, e exclusão de operações
relativas/lançamento/correio. Uma regressão existente confirma que a opção não
inventa um passo para um ledger isolado. Três JS verificam a opção separada,
filtragem de autoridade/manifestos forjados e apresentação da incerteza antes da
escolha. Passaram **2552 Rust/21 ignorados**, **542 ensaios privados no anfitrião**,
check, **1183 JS**, TypeScript e Vite. O web B é reutilizado com prova explícita:
apenas três ficheiros Rust de testes/adapter cfg(test) mudaram até C, bundle42
e fontes web exatos. Snapshot840/825 fontes/466 inputs nativos. Foram adotados
19 ficheiros, dois novos; HEAD/index ficaram intactos.

Na mesma fonte/build C passaram os cinco percursos anteriores: recibo autêntico
com Cancel/ausência/corrupção, nova execução depois de consulta congelada,
Finished/Settled com sidecar opcional, cinco crashes com retirada/ACK e
continuação, e ciclo completo de replaneamento/Stop/locks/histórico/preparações.
As provas usam IPC Tauri real, D-Bus/modelo/leitor/perfil privados e rede apenas
loopback isolada. A instalação .30 ficou intacta; sem quota real, commits/pushes
ou novos agentes. O primeiro full B falhou na compilação dos testes por método
privado e campo novo ausente; C usa um adapter cfg(test) com lock próprio e o
campo false no teste legado. Um comando de build B errou a feature; B2 e C
usam a feature correta. Estas falhas ficam conservadas e não contam como provas
verdes; a visibilidade de produção não foi aberta para os testes.

Provas privadas em `.build/pet-goal-execution-next-r1/`:
`UNCERTAIN-STATE-INTEGRATED-CHECKPOINT-R39.json`, `ROOT-APPLICATION-UNCERTAIN-STATE-R39.json`,
`UNCERTAIN-STATE-CONTINUATION-ARCHITECTURE-R39.json`,
`APP-NATIVE-UNCERTAIN-STATE-CONTINUATION-R39C.json`,
`APP-NATIVE-UNCERTAIN-STATE-CANCEL-R39C.json`,
`UNCERTAIN-STATE-NATIVE-REGRESSIONS-R39C.json`,
`HOST-NATIVE-UNCERTAIN-STATE-R39C.json`, `uncertain-state-full-r39c-RESULT.json`,
`uncertain-state-check-r39c-RESULT.json`, `uncertain-state-app-build-r39c-RESULT.json`,
`uncertain-state-web-r39b-RESULT.json`, `UNCERTAIN-STATE-WEB-REUSE-R39C.json`,
`UNCERTAIN-STATE-SNAPSHOT-R39C.json` e `UNCERTAIN-STATE-FINDINGS-R39.json`.

Âmbito delimitado: não autoriza repetir efeitos não classificados ou de origem
ausente, não repara perfis corrompidos e não comprova paridade dos restantes
adaptadores. Justiça geral da fila, janela fechada, legado, memória/fontes,
voz integral, rotinas, cooperação, matriz de Sistema e migração/rollback/entrega
conservam os seus critérios. RUN-04 e o sistema completo continuam ativos.

<a id="recibos-prefixo-r38"></a>

## Auditoria R38A — falha original conservada; correção local823 abaixo

Na fonte822M já integrada, uma falha real de escrita no perfil privado conserva
uma observação nativa assinada do leitor, mas impede liquidar o orçamento. Depois
de matar/reiniciar a app, o fecho é confirmado e uma nova continuação é autorizada:
o resultado da ferramenta é conhecido, apesar do ledger antigo ainda Started.
O endpoint existente `pet_runtime_recover_step` devolve Finished sem novos efeitos,
mas altera o grant cujo hash a continuação já fixou; `pet_runtime_read` passa a
devolver invalidRecord. O teste falha deliberadamente nessa invariável; não é
aceitação verde nem altera o âmbito das provas R37 abaixo.

Prova: `.build/pet-goal-execution-next-r1/APP-NATIVE-TOOL-RECOVERY-AUDIT-R38A.json`,
com script/binary/source bindings, processo realmente interrompido, uma operação
no leitor privado e zero chamadas reais/quota. A reconciliação não iniciou
modelo nem ferramenta adicionais. `TOOL-RECOVERY-SOURCE-AUDIT-R38.json` identifica
a API atual, os testes existentes de recuperação após pausa/reinício e a lacuna de
serialização/prefixo. `UNKNOWN-TOOL-RECOVERY-NEXT-CONTRACT-R38.json` conserva a
aceitação originalmente pendente, concretizada no âmbito local823 abaixo. Não duplicar a recuperação nativa nem presumir sucesso de um
resultado sem prova. Arquitetura/decisão antes da correção; instalação .30 intacta.

<a id="recuperacao-local823"></a>

## Recuperação nativa local823 sem alterar o prefixo histórico

A API existente `pet_runtime_recover_step` mantém o catálogo original do objetivo
em leitura durante a transação exclusiva da autoridade. Novas admissões precisam
do lock exclusivo do objetivo. A revisão atual, pet, decisão, passo, grant e perfil
nativo são validados antes de qualquer liquidação. Sob o lock da autoridade,
a classificação autentica os grants da continuação e os hashes do prefixo exato;
não recebe um indicador de congelamento do cliente.

Antes de existir uma continuação que sele o grant, conserva-se a normalização
nativa da evidência recuperável. Quando a contabilização já está selada, a API
devolve a fase canónica original, o encargo conservador e a observação assinada
em leitura. `accountingFrozen:true` implica `reconciliationAvailable:false`;
`recordedResult` só é exposto quando o resultado canónico ainda não foi publicado.
Não há novo schema persistente, journal, dono, reembolso ou efeito. As consultas
de passo e receita usam a mesma classificação, e a ação histórica escolhe a
admissão original em vez de assumir a mais recente.

O percurso E reproduziu a falha original com uma operação privada e falha real de
escrita, crash/reinício, fecho e continuação. A recuperação manteve a fase Started,
a reserva e todos os ficheiros assinados, expôs a observação autêntica e conservou
o objetivo legível. Repetir e consultar passo/receita não escreveu. Cancelar a
admissão posterior permaneceu terminal. Retirar o sidecar manteve o resultado
desconhecido apesar de o leitor estar em pausa; adulterar a assinatura foi recusado.
Só a fixture restaurou os bytes originais, sem reparação automática pelo produto.

O percurso F iniciou depois dessa consulta uma nova execução com outra lease,
nova aprovação explícita da receita e os limites adicionais autorizados. Concluiu
com uma chamada sintética e duas operações privadas novas. O grant, admissão e
encargo antigos ficaram inalterados; a consulta histórica após conclusão continuou
Started e não atribuiu a conclusão à receita antiga. O percurso G conservou um
resultado genuinamente Finished/Settled e a chamada seguinte interrompida. Retirar
o sidecar opcional não apagou o resultado já autenticado no recibo canónico.
Em E/F/G, a própria recuperação fez zero chamadas ao modelo e zero ações.

Três testes Rust novos cobrem locks reais/serialização objetivo→autoridade,
recusa de referências/revisões/perfil divergentes sem escrita e resultado
genuinamente ausente. Três JS cobrem o alvo histórico exato, resposta divergente
e projeção de evidência conhecida/desconhecida sem botão de liquidação congelada.
Passaram **2549 Rust/21 ignorados**, **539 ensaios privados no anfitrião**, check,
**1180 JS**, TypeScript e Vite. O check B é reutilizado com prova de igualdade de
todos os 464 inputs nativos de C; a única diferença posterior era TypeScript.
O snapshot imutável contém 838 ficheiros/823 fontes, bundle42 exato. Foram
adotados 16 ficheiros, um novo. HEAD/index permanecem intactos.

Na mesma fonte/build C, passaram os dois ciclos anteriores: cinco crashes/reinícios,
retirada/ACK e continuação; e replaneamento, Stop, locks, ACK expirado, preparações,
histórico cumulativo e conclusão já certificada. As fixtures usam Tauri/WebKit e
IPC reais em perfil, D-Bus, leitor e HTTP sintético privados, sem montar o ecrã
ou perfil do utilizador e sem acesso à rede. Não houve quota real, novos agentes,
commits/pushes ou alteração da instalação .30.

A prova A reprovada mantém-se. C falhou por nome errado de helper na fixture;
D recriou evidência com modo0644 e o backend recusou-a corretamente. E usa0600
para testar realmente a assinatura adulterada. A primeira bateria web B passou
os testes mas falhou TypeScript num acesso opcional; C corrige-o e passou toda
a bateria. Não se anunciam esses erros de fixture/compilação como testes verdes.

Provas privadas em `.build/pet-goal-execution-next-r1/`:
`TOOL-RECOVERY-INTEGRATED-CHECKPOINT-R38.json`, `ROOT-APPLICATION-TOOL-RECOVERY-R38.json`,
`APP-NATIVE-TOOL-RECOVERY-R38E.json`, `APP-NATIVE-TOOL-RECOVERY-R38F.json`,
`APP-NATIVE-TOOL-RECOVERY-R38G.json`,
`APP-NATIVE-TOOL-RECOVERY-RETIREMENT-REGRESSION-R38C.json`,
`APP-NATIVE-TOOL-RECOVERY-LIFECYCLE-REGRESSION-R38C.json`,
`HOST-NATIVE-TOOL-RECOVERY-R38C.json`, `tool-recovery-full-r38c-RESULT.json`,
`tool-recovery-check-r38b-RESULT.json`, `TOOL-RECOVERY-CHECK-REUSE-R38C.json`,
`tool-recovery-web-r38c-RESULT.json`, `tool-recovery-app-build-r38c-RESULT.json`,
`TOOL-RECOVERY-SNAPSHOT-R38C.json`, `TOOL-RECOVERY-NATIVE-FINDINGS-R38.json`
e `TOOL-RECOVERY-DOCS-AUDIT-R38.json`.

Aceitação delimitada: não repara prefixos já corrompidos, não resolve autoria de
efeitos sem recibo e não autoriza repetir efeitos desconhecidos. Cobertura integral
do legado, justiça geral da fila, continuidade com janela fechada, paridade por
provider, memória/fontes, voz, rotinas, cooperação, matriz de Sistema e migração/
entrega conservam aceitação própria. RUN-04 e o objetivo completo continuam ativos.

<a id="efeitos-desconhecidos-r39"></a>

## Auditoria R39A — efeito genuinamente sem resultado, resolução ainda aberta

Na mesma fonte823/build C, a app privada executou uma pausa e recebeu uma falha
real de escrita antes de conservar qualquer observação autenticada. O processo
foi realmente morto/reiniciado; o fecho nativo conservou o passo Started, uma
ferramenta desconhecida e o encargo original de16642 B. Este caso difere do recibo
recuperável de R38: não existe ficheiro de evidência a reconciliar.

O leitor em pausa, alterado por outro ator privado ou indisponível produziu o
mesmo resultado histórico desconhecido. Em todos os casos, recuperar/usar o
controlador antigo recusou; a continuação devolveu `unreconciledEffects`. Nenhuma
consulta alterou grants, ficheiros, sourceHash ou consumo, nem chamou modelo ou
ferramentas. Cancel conservou o estado terminal. Isto comprova a recusa
conservadora existente, não uma resolução explícita implementada nem autonomia
completa. A continuidade de outra tarefa após esta perda conserva aceitação
própria; não se atribui a esta fixture uma execução independente que não ocorreu.

Prova privada: `APP-NATIVE-UNKNOWN-TOOL-AUDIT-R39A.json`, com bindings C e IPC
Tauri reais, perfil/D-Bus/leitor sintéticos isolados e zero quota real. O contrato
`UNRESOLVED-TOOL-EFFECTS-NEXT-CONTRACT-R39.json` exige decidir a resolução antes
de alterar schemas/permissões: reconhecer uma incerteza, fechar uma execução e
conceder nova autoridade são decisões diferentes. Não inventar autoria/sucesso,
reembolso, confirmação histórica ou autorização para repetir a operação.

<a id="retirada-local822"></a>
<a id="retirada-em-validacao"></a>

## Retirada nativa local822

`pet_runtime_history` permite consultar explicitamente as autorizações sem
admissão; `pet_runtime_prepare_retirement` e `pet_runtime_confirm_retirement`
reutilizam perfil/chave/catálogos e a ordem de locks objetivo→autoridade. A
revisão pertence ao backend e à janela original. Exige um grant canónico
assinado Issued, nunca reclamado, sem lease, reservas, rondas ou efeitos, e sem
admissão associada. Não transforma o hash de uma autorização em consentimento,
controlador, fila ou orçamento. Temporário isolado, origem substituída, HMAC
inválido, claim, admissão ligada ou artefactos contraditórios recusam a operação.

A retirada é uma redução explícita de autoridade. O grant revogado usa schema8
e uma referência de auditoria schema1 vinculada ao consentimento original,
objetivo/revisão/sequência, inventário, artefactos e perfil nativo. Não reescreve
o objetivo nem elimina dados históricos. A ausência deste campo mantém os bytes
dos formatos anteriores. Todos os artefactos partilhados usados como prova são
autenticados antes de os classificar como pertencentes a outro objetivo. Só os
temporários assinados da própria operação são limpos após revogação durável.

A escolha recebida conserva token, instante e âmbito através de Busy, expiração
da revisão e ACK perdido. Depois de reiniciar, o recibo assinado permite apenas
confirmar a retirada existente, incluindo perante um objetivo já cancelado ou
mais recente. A revisão perdida não é reconstruída. A interface envia referências
e escolha, conserva confirmação incerta e só consulta/revê por ação explícita.
A fábrica declara `goals.retirementReview` nos quatro adaptadores nativos;
serviços/observadores e adaptadores sem suporte não recebem execução por herança.

A fixture Tauri/WebKit privada passou cinco crashes/reinícios: publicação de
fecho anterior sem projeção, duas perdas do dono, grant Issued sem admissão e
retirada assinada antes de perder a cache. A recusa não escreveu; retirar e
confirmar após reinício conservaram os mesmos bytes assinados, instante, objetivo
e consumo desconhecido. Outra revisão autorizou limites adicionais, novo dono e
uma continuação até conclusão: uma chamada sintética nova e duas operações no
leitor privado. O resultado antigo conservou uma chamada/2048 B desconhecidos;
o total ficou em duas chamadas, duas ferramentas e 3336 B, com um resultado de
modelo ainda desconhecido. Não houve quota real nem alterações à instalação.

O primeiro ensaio D errou ao procurar decisionId na proposta; E recebeu unsafePath
após publicar realmente o canónico e impedir a escrita do objetivo; F esperou uma
revisão onde o fecho anterior já confirmado devolveu Closed. Estas provas ficam
preservadas. G chegou à conclusão e feedback assinados, mas a leitura final
recebeu Busy e o executor conservou a aprovação. A certificação de leitura de
conclusão usava exclusividade desnecessária. H usa lock partilhado para autenticar
e estabilizar evidência imutável; a validação transacional de publicação mantém
lock exclusivo. Dois testes com flock real cobrem coexistência de leitores,
recusa de escritor e assinatura corrompida. A aprovação/evidência não permitem
repetir efeitos. H/I/J ainda pararam após aprovação e retoma do recurso, incluindo uma execução
sequencial. J conservou a aprovação com StorageUnavailable, sem ferramentas.
L avançou até duas operações com recibos assinados, mas não publicou a verificação
final. A consulta de claim também usava a leitura estrita de execução durante
publicação Prepared; passou à leitura autenticada de controlo, mantendo a validade
calculada pelo dono nativo e executionAvailable:false.

M usa um orçamento de 100 ms para adquirir o lock exclusivo no mesmo descritor, antes de
qualquer escrita. Leitores mantêm ausência de espera; os fences de perfil/chave/
inode são revistos após aquisição. Três testes de flock cobrem leitor transitório,
Busy persistente sem publicação e lock substituído sem consentimento. Não repete
publicação, fsync incerto, modelo, ferramenta ou decisão. A transação original
revalida fonte/Stop/prazo sob o guard. No registo de receita, a causa nativa fica
preservada e só Busy anterior à publicação admite espera limitada de um segundo,
conservando a aprovação/Ready originais; dois testes cobrem contenção breve e
persistente/repetição explícita sem nova aprovação ou IO. O erro K foi de acesso
a campos privados nos testes e ficou corrigido sem abrir visibilidade de produção.
O percurso M completo e o ciclo nativo anterior passaram; isso não explica a causa
de todos os Busy ou invalidRecord possíveis.

Passaram **2546 testes Rust/21 ignorados**, check, **536 ensaios privados no
anfitrião**, **1177 JavaScript**, TypeScript e Vite. São 20 testes Rust novos nesta
unidade e oito JS; os sete ficheiros alterados após o web são apenas Rust, com prova explícita de
reutilização do web e bundle42 inalterado. Snapshot imutável de 837 ficheiros, 822 fontes e 463
inputs nativos; 35 ficheiros adotados, sete novos. HEAD/index e instalação .30
permanecem intactos, sem commits/pushes, novos agentes ou quota paga.

Provas privadas em `.build/pet-goal-execution-next-r1/`:
`RETIREMENT-INTEGRATED-CHECKPOINT-R37.json`, `ROOT-APPLICATION-RETIREMENT-R37.json`,
`APP-NATIVE-UNLINKED-RETIREMENT-R37M.json`,
`APP-NATIVE-RETIREMENT-REGRESSION-R37M.json`,
`HOST-NATIVE-UNLINKED-RETIREMENT-R37M.json`,
`retirement-full-r37m-RESULT.json`, `retirement-check-r37m-RESULT.json`,
`retirement-app-build-r37m-RESULT.json`, `RETIREMENT-WEB-REUSE-R37M.json`,
`RETIREMENT-SNAPSHOT-R37M.json` e `UNLINKED-RETIREMENT-NATIVE-FINDINGS-R37.json`.

Continua aberta a reconciliação de ferramentas com resultado desconhecido sem
recibo recuperável. Estado atual semelhante não prova autoria/efeito passado;
nem retirada nem novo orçamento permitem repetir essa ação. Cobertura integral
do legado, paridade por provider, memória/fontes, voz integral, rotinas,
cooperação/coordenação resilientes, matriz de Sistema e migração/entrega conservam
aceitação própria. RUN-04 e o objetivo completo continuam ativos.

<a id="continuacao-local815"></a>

## Continuação nativa local815

`pet_runtime_prepare_continuation` usa o objetivo original em RecoveryRequired,
com fecho nativo autenticado e projeção confirmada. A revisão devolve o histórico
revisto, os limites/modelos anteriores, consumo e resultados incertos, e os novos
limites adicionais. A confirmação existente conserva o objeto/autoridade nativos
originais e assina uma nova admissão. Não passa por um Draft artificial, não
trunca o objetivo e não reabre o controlador/lease antigo. Política, modelo,
contexto, revisão, origem assinada, inventário e recibos continuam sujeitos aos
controlos nativos. O objetivo passa a schema10 e mantém essa versão ao publicar
planos, pausar, drenar ou concluir. A referência opcional ausente conserva os
bytes de consentimento dos formatos anteriores.

O claim moderno assinado sem primeira reserva passa a `nativeUnstarted`, com
zero chamadas comprovadas pelo protocolo e sem fabricar um ledger. Legado sem
medição continua desconhecido. Preparações canónicas revogadas sem claim/IO
continuam na auditoria; outras autorizações sem admissão e temporários alheios
bloqueiam. Só os temporários assinados da própria transação aprovada podem ser
retirados nesse percurso. A perda de cache depois de um crash ainda exige o
percurso de inspeção/retirada nativa definido para a unidade seguinte.

A cache distingue revisão não aprovada expirada de escolha recebida ainda sem
ACK. A segunda conserva o token, âmbito, prazo, aprovação e orçamento originais.
Resposta perdida, Busy e publicação incerta não renovam consentimento. Um token
antigo confirma o seu evento sem sobrescrever a admissão mais recente ou o
Cancel. A vista funcional mostra o consumo anterior, a cobertura não comprovada
e o orçamento adicional antes da decisão; confirmação de recibo não exige nova
prontidão de modelo. Não houve desenvolvimento de apresentação/UX adicional.

A fábrica declara `goals.continuationReview` como revisão nativa com alvo, fecho
autenticado, verba adicional e reconciliação de efeitos desconhecidos. Os quatro
adaptadores nativos têm cobertura comum de proveniência. Isso não declara
execução autónoma equivalente nos quatro: o percurso executado continua limitado
a Ollama local e ao catálogo Spotify, com aprovação exata de cada receita.

Passaram **2526 Rust/21 ignorados**, cargo check e **516 ensaios privados no
anfitrião**, incluindo 388 do runtime e o novo teste da fábrica. Os **1169 testes
JavaScript**, TypeScript e Vite passaram na fonte C; as diferenças posteriores
são apenas Rust e estão enumeradas/vinculadas à fonte final F. O bundle tem
42 ficheiros exatos. Mantêm-se 41 avisos nos testes e 54 no check. Quinze testes
Rust novos cobrem continuação, matriz dos adaptadores, origem histórica,
publicação, permissões, incerteza, expiração, temporários e locks reais; cinco
casos JavaScript novos cobrem bridge, transação/repetição e supervisão funcional.

A app privada passou três crashes/reinícios de fecho, incluindo falha de gravação
da projeção depois da assinatura, e continuou o objetivo cuja chamada ao modelo
fora interrompida. Conservou intenção/critérios/contexto, admissão/grant antigos e
2048 bytes cobrados à tentativa desconhecida. A autorização adicional criou
outro dono, uma chamada sintética nova e dois efeitos no leitor privado; concluiu
os critérios nativamente. O histórico final manteve duas chamadas ao modelo,
duas ferramentas, 3336 bytes e um resultado antigo desconhecido. A cobertura
histórica integral e tokens/custo não medidos não foram inventados.

O percurso anterior voltou a passar: replaneamento com feedback assinado,
justiça entre objetivos, aprovação retida, Stop sob locks reais, ACK expirado,
preparações sem IO, leitura cumulativa e um quarto crash/reinício com conclusão
prévia preservada. As constantes de indisponibilidade do roteiro antigo têm
âmbito histórico; o suporte novo é demonstrado pelo ensaio de continuação acima.

As primeiras provas D/E da app falharam e estão conservadas. D expôs a recusa de
metadados em RecoveryRequired; E já publicou a nova admissão e o novo resultado,
mas o plano repunha schema8. A revisão para a versão necessária e o novo teste
nativo fecharam essa falha no percurso F. O ensaio anterior E de replaneamento
terminou com Conflict depois do segundo resultado, sem causa interna completa
registada. Preservar Busy nos controlos de publicação, esperar no máximo um
segundo com o mesmo dono/pedido/resultado e dispensar o novo lock em objetivos
sem continuação foi validado com flock real e pelos dois percursos F. Não se
atribui uma causa retroativa comprovada a cada falha original. Apenas Busy pode
ser esperado; ACK incerto/outros erros não causam replay de modelo ou efeito.

A primeira bateria F em concorrência teve 2525 aprovações e uma falha na fixture
de sessão Gravity, sem identificar a causa original. As fontes Gravity não
mudaram: os seis casos isolados passaram, e a bateria completa em série passou
2526 casos com a mesma fonte F. Conservam-se ambas as provas; não se declara
corrigida uma falha de concorrência Gravity ainda sem reprodutor/causa.

Provas locais em `.build/pet-goal-execution-next-r1/`:
`CONTINUATION-INTEGRATED-CHECKPOINT-R36.json`,
`ROOT-APPLICATION-CONTINUATION-R36.json`,
`APP-NATIVE-FRESH-CONTINUATION-R36F.json`,
`APP-NATIVE-CONTINUATION-REGRESSION-R36F.json`,
`HOST-NATIVE-FRESH-CONTINUATION-R36F.json`,
`CONTINUATION-SNAPSHOT-R36F.json`, `CONTINUATION-WEB-REUSE-R36F.json` e
`NATIVE-CONTINUATION-INTEGRATION-FINDINGS-R36.json`.
São 815 fontes, snapshot de 830 ficheiros imutáveis, 456 inputs nativos e
32 ficheiros adotados, quatro novos, com pré-imagens e HEAD/index preservados.
Sem commits/pushes, novos agentes, quota real, alterações ao perfil/anfitrião ou
à instalação .30/PID4091547. O percurso foi executado em Tauri/WebKit real num
display/bus/perfil privados; o provider e o leitor eram sintéticos.

Continuam pendentes a retirada nativa após perda do processo entre publicações,
efeitos desconhecidos sem recibo recuperável, continuidade de histórico legado,
paridade dos outros adaptadores, migração/backup/rollback e entrega. O ensaio F
conclui este percurso delimitado de continuação; RUN-04 e o objetivo completo
permanecem ativos, assim como memória/fontes, rotinas/cooperação/coordenação, voz
integral e matriz de Sistema.

<a id="historico-cumulativo-local811"></a>

## Histórico cumulativo local811

`pet_runtime_history` consulta as admissões originais sob leitores partilhados
do objetivo e da autoridade. Reutiliza a mesma inspeção de recibos da reconciliação
existente, selecionando cada admissão com a sua prova, sem construir objetivos
truncados. Devolve limites por admissão, hashes/origem, chamadas e bytes cobrados,
resultados conhecidos/incertos e um hash da fonte estável quando só muda a hora
da consulta. A consulta anterior conserva o âmbito da última admissão.

Os grants assinados presentes sem referência no objetivo e as publicações
temporárias são expostos; não são ignorados nem promovidos. Uma origem incompleta
retém subtotal autenticado, mas não um total apresentado como completo. Ledger
legado ausente conserva consumo desconhecido; só uma admissão autenticada nunca
reclamada permite zero sem ledger. Tokens e custo não medidos continuam null.
Contagens acumuladas recusam overflow sem atualização parcial. Não há escritas,
fsync, recibos reparados, inferência, ações, controlador ou permissão de retoma.

O âmbito é `allRecordedAdmissions`, com cobertura declarada das linhas e grants
presentes. As assinaturas antigas não vinculam um head de todo o histórico: uma
linha antiga e o respetivo grant podem ter sido removidos antes da consulta.
Por isso `historyContinuityVerified` e `continuationAvailable` permanecem false;
o total dos registos presentes não certifica que todo o histórico existiu sem
omissões. O teste reproduz essa remoção sem forjar a assinatura restante.
Um claim moderno pode ainda não ter ledger antes da primeira reserva; a leitura
atual conserva também esse caso como consumo não verificável. A distinção entre
essa proveniência moderna sem IO e o legado realmente não medido é pré-requisito
da próxima revisão de continuação, sem fabricar ledgers ou apagar tentativas.
A futura admissão deve vincular o histórico/consumo revistos, preservar a cobertura
não verificável e exigir aprovação de qualquer verba adicional. Um checksum novo
de JSON não substitui essa proveniência nem permite concluir saldo livre.

Passaram **2511 Rust/21 ignorados**, cargo check e **501 ensaios privados no
anfitrião**, incluindo 374 do runtime. Quinze testes novos cobrem os quatro pets,
múltiplas admissões, resposta nativa com recibo, consumo antigo started ainda
desconhecido, histórico omitido, grants/recibos ausentes ou alterados, formatos
legados, publicações pendentes, leitores/locks reais, todos os 32 registos,
isolamento entre objetivos e overflow. Mantêm-se 41 avisos nos testes e 54 no
check. As fontes web e bundle42 são exatos à prova R34 de **1164 JS/TSC/Vite**;
só fontes Rust mudaram. A primeira compilação dos testes recusou um import
privado de DrainOutcome; o import passou a apontar ao módulo original, sem
alargar visibilidade ou alterar produção. As provas originais estão preservadas.

Os primeiros ensaios R35/R35B/C da app falharam em fases de cedência/replaneamento,
antes da nova consulta. Sete consultas de saldo/contexto/journal/candidato/receita
pediam locks exclusivos apesar de serem só leitura. Passam a leitores partilhados;
reservar/iniciar/publicar continuam exclusivos. Duas regressões mantêm um leitor
real e verificam leituras sem escrita, enquanto as mutações recebem Busy. O driver
conserva também a mensagem nativa de falha de planeamento, em vez de mostrar apenas
StorageUnavailable. As falhas privadas originais permanecem preservadas, e a
composição R35E voltou a ser validada pelos percursos completos abaixo.

O ensaio D isolou a falha na reserva do orçamento antes do segundo envio. O mesmo
ModelCall/worker/tentativa conserva agora uma espera limitada a um segundo por etapa apenas
perante Busy, antes do HTTP. Mantém autoridade, ID, âmbito e prazo originais;
Stop/prazo interrompem a espera. Uncertain e restantes erros não são repetidos.
Duas regressões com lock real confirmam um só pedido de inferência/tentativa e
Stop sem inferência nem tentativa iniciada. Nenhum envio incerto é reproduzido.

A primeira execução do teste Stop já confirmou só metadados e zero inferência,
mas a asserção final esperava um ledger vazio quando ainda não havia ledger.
A verificação aceita agora ausência ou lista vazia, mantendo zero tentativas.
Só writer_tests.rs mudou depois da prova E da app/check/build; é módulo cfg(test).
A equivalência de produção está vinculada em
`CUMULATIVE-HISTORY-PRODUCTION-REUSE-R35F.json`. A biblioteca completa e o ensaio
no anfitrião foram repetidos com a fonte final F; não se infere aceitação do
teste corrigido a partir da prova de produção anterior.

Na app Tauri real privada, a consulta reconheceu duas chamadas de modelo/quatro
ferramentas da conclusão e manteve uma chamada interrompida com resultado incerto
no Stop. Uma segunda admissão explícita pelo percurso já existente de Pausa →
Draft manteve a admissão e o consumo incerto anteriores; a leitura contou ambas
sem novas chamadas/ações ou alterações a ficheiros. Isto não implementa retoma
de RecoveryRequired. A fixture anterior voltou a passar, incluindo conclusão,
Stop/locks, decisões retidas, preparações e um crash/reinício com conclusão
conservada. Outro ensaio voltou a passar os três crashes/reinícios e ACK do fecho
parcial. Foram quatro processos privados terminados/recolhidos no conjunto; as
consultas de histórico não acrescentaram inferência ou efeitos. Modelos/leitor
sintéticos, sem quota ou dispositivos reais.

Provas locais em `.build/pet-goal-execution-next-r1/`:
`CUMULATIVE-HISTORY-INTEGRATED-CHECKPOINT-R35.json`,
`ROOT-APPLICATION-CUMULATIVE-HISTORY-R35.json`,
`APP-NATIVE-CUMULATIVE-HISTORY-R35E.json`,
`APP-NATIVE-CUMULATIVE-HISTORY-RESTART-R35E.json`,
`HOST-NATIVE-CUMULATIVE-HISTORY-R35F.json` e `CUMULATIVE-HISTORY-SNAPSHOT-R35F.json`.
Mapa811, snapshot `cumulative-history-r35f-source` com 826 ficheiros imutáveis e
452 inputs nativos. Dezanove ficheiros integrados, três novos, com pré-imagens e
HEAD/index intactos. Sem commits/push, agentes, quota real ou alterações à app
instalada .30/PID4091547, perfil/display ou configuração do anfitrião.

Segue a revisão/admissão nativa de continuação, vinculada ao fecho confirmado,
histórico e limites explicitamente autorizados, sem replay de efeitos incertos.
RUN-04, os 43 requisitos, paridade real, background, voz/memória/fontes, cooperação,
rotinas, Sistema, migração/backup/rollback e entrega continuam abertos.

<a id="fecho-recuperacao-local808"></a>

## Revisão e fecho após falha local808

A revisão nativa retém autoridade original, prova de ausência, origem e hashes
do objetivo/grant/recibos. A aprovação fecha apenas a execução antiga, preservando
identidade, lease, orçamento, consumo, reservas e resultados desconhecidos. A
publicação usa a ordem objetivo → autoridade e confirma os bytes dos recibos
antes da primeira escrita. O fecho fica autenticado no grant schema7; o objetivo
schema9 recebe um evento de fecho perdido ou de preparação não publicada. Uma
conclusão genuína conserva o objetivo e certificado exatos, sem nova versão.

Contenção, resposta perdida e ACK incerto conservam a mesma escolha nativa. O
frontend não repete escritas automaticamente. Um fecho já assinado permite,
mesmo após reinício, confirmar apenas a projeção aprovada, com o mesmo instante e
alvo. Não concede execução, não recria o controlador antigo e não reembolsa
consumo incerto. Enquanto essa projeção estiver pendente, outras mutações recebem
Busy. Tokens históricos só confirmam o seu recibo; cancelamento posterior não é
substituído. A cache mantém âmbito de janela/pet/objetivo/revisão e limite64,
sem mutex global durante IO. Recusar uma revisão nova não escreve ficheiros.

Passaram **2496 Rust/21 ignorados**, cargo check e **486 ensaios privados no
anfitrião**, incluindo 359 do runtime. Treze testes novos cobrem os quatro pets,
processo vivo/ausente, recusa/expiração, alterações de origem/recibos, locks reais,
falhas antes/depois das publicações, decisões retidas, perda da cache, cancelamento
posterior e outro dono operacional. Mantêm-se 41 avisos nos testes e 54 no check.
Passaram **1164 JavaScript**, TypeScript e Vite; bundle42. A única diferença
posterior à prova web foi retirar um import Rust de teste não utilizado.

Na app Tauri real isolada, três processos privados foram terminados e recolhidos
nos casos de aprovação pendente e streaming interrompido. A gravação do objetivo
foi impedida depois do fecho assinado; após novo arranque, o ACK conservou a
escolha/data/alvo e só publicou a projeção original. O streaming permaneceu
desconhecido, com os 2048 bytes reservados. Repetir o recibo não escreveu nem
executou ações. Duas chamadas sintéticas, zero efeitos no leitor neste ensaio.

O ciclo privado completo anterior voltou a passar: aprovação/reload, claim
concorrente, cedência/retoma, duas rondas/conclusão, Stop sob locks e projeção
atrasada, ACK expirado e limpeza de preparações não ativadas. Um quarto crash/
reinício confirmou o fecho do dono de um objetivo já concluído: objetivo,
certificado e data de fecho anteriores exatos; só mudou o grant original. A
fixture completa usou três chamadas sintéticas e quatro efeitos no leitor
privado. O fecho após reinício não acrescentou chamadas ou efeitos.

Provas locais em `.build/pet-goal-execution-next-r1/`:
`RECOVERY-CLOSURE-INTEGRATED-CHECKPOINT-R34.json`,
`ROOT-APPLICATION-RECOVERY-CLOSURE-R34.json`,
`APP-NATIVE-RECOVERY-CLOSURE-R34.json`,
`APP-NATIVE-RECOVERY-CLOSURE-FULL-R34.json`,
`HOST-NATIVE-RECOVERY-CLOSURE-R34.json` e `RECOVERY-CLOSURE-SNAPSHOT-R34.json`.
Mapa808, snapshot `recovery-closure-r34-source` com 823 ficheiros imutáveis e
449 inputs nativos. Vinte e três ficheiros integrados, sete novos, com pré-imagens
e HEAD/index intactos. Sem commits/push, agentes, quota real ou alterações à app
instalada .30/PID4091547, perfil/display ou configuração do anfitrião.

Segue uma revisão separada de continuação com nova admissão, conservando intenção,
critérios, contexto, uso acumulado e incerteza. ResumeDraft genérico não contorna
RecoveryRequired. Recibos/fonte de planeamento ausentes ou corrompidos mantêm
recusa conservadora; não se comprovou recuperação arbitrária de perda de dados.
Paridade real por provider, background com janela fechada, voz, memória/fontes,
cooperação, rotinas, matriz de Sistema, migração/backup/rollback e entrega continuam
pendentes. Esta aceitação não conclui RUN-04 nem o objetivo completo.

<a id="reconciliacao-leitura-local801"></a>

## Reconciliação de leitura local801

`pet_runtime_reconcile` vincula pet, objetivo e revisão. Conserva o leitor do
objetivo e leitores partilhados da autoridade durante a leitura coerente dos
recibos existentes. Expõe hashes, consumo, identidade e projeção de controlo,
resultados conhecidos e provas ausentes/inválidas/indisponíveis. Reutiliza
respostas, saídas de planeamento, passos/observações, receitas, verificação e
conclusão assinados. Não publica, confirma durabilidade, consulta alvos físicos
ou dá permissão de execução/recuperação. O saldo refere-se à última admissão;
o restante histórico continua vinculado ao hash do objetivo para a revisão futura.

Uma origem privada só nasce depois da autenticação do grant. A prova nativa de
ausência conserva o flock original, origem/revisão/hash e identidade do processo
que a capturou; revalida perfil, UID, boot, namespace, processo ausente, diretório
e inode. Processo original ainda vivo mesmo sem flock, PID reutilizado, origem
divergente e ficheiro substituído são recusados. Na consulta a prova é efémera;
DTOs/JSON não a recriam. Aprovação de fecho e continuação ainda não são oferecidas.

Passaram **2483 Rust/21 ignorados**, cargo check e **473 ensaios privados no
anfitrião**, incluindo 346 do runtime. Oito testes novos cobrem os quatro pets,
leitores reais, ausência de escrita/criação, fases de consumo sem recibo,
recibos válidos/alterados/ausentes, projeção parcial, observação de ferramenta
antes do recibo final e conclusão nativa sob leitores partilhados. Os testes de
perda real do processo passaram também exclusão entre provas e substituição do
inode. Uma observação válida pode confirmar o efeito enquanto a fase de consumo
continua started e a reserva integral se conserva. Mantêm-se 41 avisos nos
testes e 54 no check. As fontes web e 42 ficheiros do bundle permanecem exatos
à prova R30 de 1160 JS/TypeScript/Vite; não houve alteração de frontend.

Na app Tauri privada, dois processos originais foram terminados e recolhidos,
seguidos de novo arranque com o mesmo perfil/namespace isolados. A consulta
reconheceu o recibo da resposta e manteve desconhecido o streaming interrompido,
com 2048 bytes cobrados e a mesma lease/orçamento/hash. Repetir a consulta e
enviar dados de aprovação falsos não alterou ficheiros nem criou autoridade.
Tokens/controladores antigos continuaram recusados. Foram duas chamadas sintéticas
e zero ações no leitor. A primeira fixture R33 falhou antes da API, ao continuar
a inspecionar uma janela secundária depois de encontrar as definições; R33B
seleciona essa janela observada e passou com a mesma fonte/binário. A falha
original permanece preservada.

O ciclo completo privado também passou: aprovação/reload, justiça e claim
concorrente, duas rondas com conclusão, Stop com publicação atrasada, ACK
expirado e preparações não ativadas após pausa/cancelamento/expiração. A nova
consulta reconheceu a conclusão e manteve o Stop desconhecido, com 82 ficheiros
duráveis exatos antes/depois, sem novas chamadas/ações. A fixture teve 20 eventos
nativos, três chamadas sintéticas e quatro efeitos no leitor privado.

Provas locais em `.build/pet-goal-execution-next-r1/`:
`RECONCILIATION-INSPECTION-INTEGRATED-CHECKPOINT-R33.json`,
`ROOT-APPLICATION-RECONCILIATION-R33.json`,
`APP-NATIVE-RECONCILIATION-R33B.json`,
`APP-NATIVE-RECONCILIATION-FULL-R33.json`,
`HOST-NATIVE-RECONCILIATION-R33.json` e `RECONCILIATION-SNAPSHOT-R33.json`.
Mapa801, snapshot `reconciliation-r33-source` com 816 ficheiros imutáveis e
442 inputs nativos. Dezassete ficheiros integrados, cinco novos, com pré-imagens
e HEAD/index intactos. Sem commits/push, agentes, quota real ou alterações à app
instalada .30/PID4091547, perfil/display ou configuração do anfitrião.

Esta fase não conclui recuperação após falha. Seguem cache/revisão nativa
explícita, fecho aprovado do grant antigo/ACK incerto e nova admissão com intenção,
contexto, uso anterior e incerteza conservados. Os restantes providers/dispositivos,
voz, cooperação, integrações e entrega mantêm a sua aceitação. O objetivo completo
e os 43 requisitos permanecem ativos.

<a id="controlo-nativo-local796"></a>

## Controlo nativo local796

Preparações originais sem ativação passam a fechar após Pausar/Cancelar e
expiração, incluindo claim assinado sem publicação no objetivo. Publica-se
auditoria terminal, sem ativação artificial. A manutenção percorre até oito das
64 entradas a cada 200 ms, conservando objetos/perfil/lease e a reserva até ACK
durável; não agenda trabalho nem infere. Estado é só leitura. ResumeDraft pode
confirmar antes a própria limpeza, com revisão exata e CAS. Recibos históricos
de controlo não revogam uma admissão posterior.

Cancel após fecho do grant e antes do ACK do objetivo prevalece sem reabrir
permissões, alterar closedAt ou apagar uso. A revogação assinada ainda sem
projeção espera essa projeção. Fecho durante cedência preserva o outro dono
nativo e o ledger de espera/consumo, sem readquirir o recurso para fechar.

Passaram **2475 Rust/21 ignorados**, cargo check e **465 ensaios privados no
anfitrião**, incluindo 338 do runtime. Onze testes novos abrangem os quatro
pets, locks reais, publicação incerta antes/depois dos dois ledgers, Cancel em
fecho parcial e outro dono nativo operacional durante o fecho cedido. Os três
testes iniciais reproduziram as falhas; a primeira correção passou sete novos,
mas revelou uma falha existente na limpeza de claim só publicado no ledger.
Essa variante foi corrigida e a bateria completa passou. Os avisos mantêm-se
em 41 nos testes e 54 no check. As fontes web e o bundle de 42 ficheiros são
exatos à prova R30 de 1160 JS/TypeScript/Vite; não houve alteração de frontend.

Na app Tauri isolada real, os três claims receberam Busy sob um lock partilhado
do catálogo de objetivos, conservando preparação/reserva com zero workers e
nenhuma lease gravada no objetivo. Pausar/Cancelar retiraram a preparação; a
expiração retirou-a sem outra mutação. Cada caso registou um ClaimAbandoned,
nenhum LeaseClaimed e o mesmo orçamento/lease original, com grant terminal e
sem runtimeOwner. Um claim independente seguinte passou. Não houve novas
chamadas de modelo nem ações nestes casos. A fixture anterior também passou:
reload/aprovação retida, claim concorrente, duas rondas/conclusão, Stop sob
locks/projeção atrasada com uso started desconhecido e ACK expirado. Foram
observados 20 eventos nativos, três chamadas sintéticas e quatro efeitos no
leitor privado no conjunto do ensaio. Cancel no intervalo de fecho parcial tem
prova unitária nativa; esse intervalo específico não foi induzido na app gráfica.

Provas locais em `.build/pet-goal-execution-next-r1/`:
`NATIVE-CONTROL-INTEGRATED-CHECKPOINT-R32.json`,
`ROOT-APPLICATION-NATIVE-CONTROL-R32.json`, `APP-NATIVE-CONTROL-R32.json`,
`HOST-NATIVE-CONTROL-R32.json` e `NATIVE-CONTROL-SNAPSHOT-R32.json`.
Mapa796, snapshot `native-control-r32-source` com 811 ficheiros imutáveis e
437 inputs nativos. Dez ficheiros integrados, três novos, com pré-imagens e
HEAD/index intactos. Sem commits/push, agentes, quota real ou alterações à app
instalada .30/PID4091547, perfil/display ou configuração do anfitrião.

Esta aceitação é local e delimitada. Faltam reconciliação/revisão após perder o
dono, providers/dispositivos reais, voz/áudio, restantes integrações e entrega.
Os 43 requisitos e o objetivo completo permanecem ativos.

<a id="fecho-retido-local793"></a>

## Fecho retido local793

Corrige os percursos de Stop e claim concorrente reprovados em R29B/C. A prova
opaca original de drain, o resultado terminal e a autoridade aberta ficam retidos
no driver. O estado `closing` repete apenas fecho/ACK, libertação do handle exato
e retirada do serviço, com backoff até cinco segundos. Se ainda não houver ACK,
a mesma prova continua disponível para repetição explícita de fecho; o driver
não volta a planear/inferir/pedir aprovação. O botão funcional confirma a presença
da prova nativa antes de enviar essa repetição. Dados de sessão guardados não a
recriam. Nenhum prazo/orçamento de execução é renovado.

A verificação da fila passa a usar lock partilhado; escritores continuam
exclusivos. `Busy` conserva a sua classificação em preparação/claim/Stop/fecho,
enquanto consentimento revogado ou origem divergente continuam conflitos reais.
O fecho aguarda a projeção da intenção de Stop já autenticada no ledger, antes
de publicar com base numa versão antiga do objetivo. Prontidão/política
indisponível conserva a observação como desconhecida, sem a converter em Stop;
a validade continua limitada ao prazo original. Uma prova com worker vivo,
perfil/chave/lease/origem trocados ou nova admissão é recusada.

Passaram **2464 Rust/21 ignorados**, **1160 JavaScript**, cargo check,
TypeScript/Vite e **454 ensaios privados no anfitrião**. Dez testes Rust novos
incluem HTTP real de loopback, locks reais, atraso da projeção de Stop,
publicação incerta em ambos os ledgers, janela de cinco segundos e repetição só
de fecho, rejeição de worker vivo/autoridade trocada/nova admissão e proteção do
novo dono físico. Dois testes JS protegem o controlo explícito sem prontidão de
modelo, claim ou nova inferência. Avisos: 41 nos testes, 54 no check; os 21
ignorados mantêm o âmbito anterior. O primeiro runtime R30 teve 326 aprovados e
uma falha ao montar um cenário com ResumeDraft a partir de RecoveryRequired;
a fixture foi corrigida para uma pausa autenticada e orçamento com tempo real,
sem relaxar a regra de produção. A bateria completa R31 passou. O web R30 é
exato em R31; só duas fontes Rust mudaram depois dessa validação.

Na app Tauri real isolada, o ciclo completo do leitor privado passou: decisão
original após reload, duas respostas Busy na admissão, claim concorrente do
segundo objetivo, espera pelo recurso sem novos efeitos, reaquisição e duas
rondas com conclusão nativa. O terceiro objetivo foi interrompido durante SSE.
Um leitor real bloqueou o catálogo dos objetivos: Stop autenticou a intenção,
mas recebeu Busy antes de a projetar. Com ambos os catálogos bloqueados, o
mesmo controlador manteve `closing`, prova retida, zero workers e a reserva
física. Depois de libertar os locks e projetar o mesmo pedido, terminou em
`stopped`/objetivo `paused`/grant `closed`, sem repetição de modelo ou ação.

A tentativa interrompida permaneceu **started/desconhecida**, com o mesmo uso
assinado antes/depois; não virou sucesso nem recebeu reembolso. O controlador
terminado foi recusado. Foram observados 19 eventos nativos. Mantiveram-se os
três pedidos sintéticos totais e quatro efeitos no leitor privado, todos fora
do perfil/display do anfitrião. O quarto objetivo comprovou ACK após expirar o
orçamento: mesmo token, uma admissão e 58 ficheiros duráveis exatos, sem novos
efeitos. As falhas R29B/C permanecem como evidência histórica; estes percursos
passaram na nova prova controlada, sem repetição genérica de Conflict.

Provas locais em `.build/pet-goal-execution-next-r1/`:
`RUNTIME-CLOSE-INTEGRATED-CHECKPOINT-R31.json`,
`ROOT-APPLICATION-RUNTIME-CLOSE-R31.json`,
`APP-NATIVE-RUNTIME-CLOSE-R31.json`, `HOST-NATIVE-RUNTIME-CLOSE-R31.json` e
`NATIVE-CONTROL-EDGE-CASES-NEXT-CONTRACT-R32.json`.
Snapshot `runtime-close-r31-source` com 808 ficheiros imutáveis, mapa793 e
434 entradas nativas. Sem commits/push, agentes auxiliares, quota real,
instalação ou alterações à app .30/PID4091547.

Isto aceita o percurso local descrito. Não aceita providers/modelos/leitores
reais, áudio/voz, todos os controlos visuais, fecho de janelas ou entrega.
Seguem primeiro a limpeza de claims ainda não ativados após pausa/cancelamento/
expiração, a escalada de Cancel após fecho do grant e os limites de fecho cedido
com outro dono. Depois, reconciliação/revisão após perder o processo original.
Os 43 requisitos e o objetivo completo permanecem ativos.

<a id="confirmacao-retida-local790"></a>

## Confirmação retida local790

Corrige a perda da aprovação perante `Busy` identificada na app privada. A cache
mantém o objeto nativo original por janela/pet/objetivo/revisão e usa um mutex
próprio por confirmação; o mutex global não acompanha IO. Depois de aprovar,
spec/instante/evidência/prazo/orçamento e autoridade aberta ficam fixos. Uma
repetição não adota outra chave/perfil nem volta a aprovar a partir de JSON.
Política/capacidades/contexto frescos são verificados antes de uma publicação nova.

Um evento de admissão já publicado permite apenas estabilizar o grant e o registo
originais, incluindo após expirar o prazo; não cria consentimento nem execução.
Recusar antes de publicar revoga apenas o grant original ainda sem lease. Se a
admissão já estava publicada, devolve o recibo real; interrompê-la usa os controlos
Stop/Cancelar existentes. O recibo retido não impede nova revisão após
Pausar/ResumeDraft, e o token antigo continua limitado ao seu ACK. Aprovação ainda
por publicar não pode ser substituída por uma revisão diferente.

O frontend conserva o mesmo token perante falha de transporte, recibo incerto ou
resposta transitória com `retryable`; uma repetição é explícita e não inicia
claim/modelo/receita. Não houve alteração de schema de dados persistidos.

Passaram **2454 Rust/21 ignorados**, **1158 JavaScript**, check/TypeScript/Vite e
**444 ensaios privados no anfitrião**. Dez testes novos exercitam locks reais,
incerteza antes/depois das publicações, idempotência, expiração/política/origem,
recusa, troca de autoridade e nova revisão após pausa/retoma. Os 21 ignorados
conservam o âmbito anterior; nenhum teste falhado foi convertido em ignorado.
Os avisos mantêm-se em 41 nos testes e 54 no check. O web validado em R28 é exato
em R29; só três fontes Rust mudaram na correção do ciclo da cache.

Na prova específica R29D da app Tauri real isolada, uma segunda admissão recebeu
dois `Busy` forçados por um flock real, enquanto outro objetivo já estava na fila.
O mesmo token confirmou uma só admissão, com o instante da primeira aprovação,
orçamento e origem conservados. Um objetivo adicional confirmou o ACK com o
orçamento já expirado: mesmo token, uma admissão e 22 ficheiros duráveis exatos.
Não houve chamadas ao modelo nem ações no leitor nessa prova específica.

Na prova parcial R29B, o primeiro objetivo conservou a decisão, esperou pelo
recurso e retomou para executar/replanear/concluir, com duas chamadas sintéticas
e quatro ações no leitor privado. Reload do renderer também passou. A fixture
falhou depois no Stop durante HTTP sintético: objetivo stopping, grant draining,
driver recoveryRequired/StorageUnavailable e tentativa do modelo ainda started.
R29C recusou com Conflict o claim concorrente de um segundo objetivo já admitido;
a causa ainda exige reprodução/classificação controladas. Ambas as falhas estão
preservadas e a aceitação completa do ciclo permanece reprovada. A prova R29D
de admissão/ACK não as substitui. Segue o grupo de transações nativas RUN-02/04.

Provas locais em `.build/pet-goal-execution-next-r1/`:
`ADMISSION-RETAIN-INTEGRATED-CHECKPOINT-R29.json`,
`ROOT-APPLICATION-ADMISSION-RETAIN-R29.json`,
`APP-NATIVE-ADMISSION-RETAIN-R29D.json` e `HOST-NATIVE-ADMISSION-RETAIN-R29.json`.
Snapshot `admission-retain-r29-source` com 805 ficheiros imutáveis, mapa790 e
431 entradas nativas. O primeiro ensaio da app R28 confirmou dois bloqueios mas
parou porque o script assumia a presença de um array vazio omitido pelo serde;
a fixture corrigida conserva o resultado anterior e não altera o produto.
R29 expôs Busy tipado em outra escrita; o cliente privado repete só esse erro
com argumentos idênticos. R29B expôs a falha real de fecho de Stop, registada em
STOP-CLOSURE-RETRY-NEXT-CONTRACT-R30.json e excluída expressamente da prova790.

Não se testaram providers/modelos/leitores reais, quota Copilot, áudio/voz,
fecho de janelas ou instalação. A app instalada .30/PID4091547 e os seus dados
permaneceram intactos, sem commits/push ou novos agentes. A recuperação efetiva
depois de perder o processo original segue pendente, com reconciliação e nova
revisão. Corrigir primeiro o fecho de Stop observado em R29B; os 43 requisitos e o objetivo completo continuam ativos.

<a id="identidade-do-dono-local787"></a>

## Identidade do dono local787

Incremento integrado localmente em 21 ficheiros, quatro fontes novas, sem
commit/push ou instalação. Novos claims autenticam PID/incarnação, UID, boot,
namespace e perfil/chave/inodes; um bloqueio nativo por lease acompanha o dono e
todos os workers. O estado distingue processo original vivo, ausência verificada,
identidade divergente, fonte indisponível e registo antigo sem proveniência.
É observação: não recria controlador, aprovação ou autoridade de execução.

Passaram **2444 Rust/21 ignorados**, **1155 JavaScript**, cargo check,
TypeScript/Vite e **434 ensaios privados no anfitrião**. O novo ignorado é o
helper de subprocesso invocado explicitamente pelo teste real de morte/reaquisição
do perfil; não representa um caso falhado ocultado. Mantêm-se 41 avisos nos
testes e 54 no check. O primeiro runtime teve uma expectativa antiga de schema
(306 passaram/um falhou); corrigida para o novo schema6, a bateria completa passou.
O web foi validado em R26; a única diferença para R27 é essa asserção Rust.

Na aplicação Tauri real isolada, foram terminados e recolhidos dois processos
sucessivos pelo supervisor; novas instâncias abriram o perfil com o mesmo perfil privado e namespace: um durante aprovação com
reserva cedida, outro durante streaming HTTP sintético. Após cada reinício,
`claim_status` confirmou `ownerAbsent`, `storedLease`, zero posse/reserva e
workers desconhecidos. As revisões e 25/31 ficheiros do runtime ficaram exatos;
a aprovação antiga e pedidos de claim/driver/release guardados foram recusados.
Houve duas chamadas ao modelo sintético e zero ações no leitor privado, sem
repetição após reinício. Não houve mocks de IPC nem comandos de teste no produto.

A primeira fixture paralela expôs um defeito existente: `Busy` ao certificar a
admissão pode consumir o token antes de enfileirar. Esse ensaio falhado fica
preservado; a prova de identidade usa falhas sequenciais e não resolve o defeito.
O supervisor privado também foi corrigido para recolher filhos reais adotados,
em vez de deixar zombies no PID1 do container. Segue a correção da transação de
admissão; reconciliação/revisão/retoma após perder o dono permanece pendente.

Provas locais em `.build/pet-goal-execution-next-r1/`:
`NATIVE-OWNER-INTEGRATED-CHECKPOINT-R27.json`,
`ROOT-APPLICATION-NATIVE-OWNER-R27.json`,
`APP-NATIVE-OWNER-RESTART-R27D.json`, `HOST-NATIVE-OWNER-R27.json` e
`ADMISSION-TRANSACTION-RETAIN-NEXT-CONTRACT-R28.json`.
Snapshot `native-owner-r27-source` com 802 ficheiros imutáveis, mapa787 e
428 entradas nativas. A app instalada .30/PID4091547 permaneceu intacta;
nenhum modelo real, quota Copilot, perfil/display do anfitrião ou instalação foi usado.
As restantes aceitações de providers, retoma efetiva, voz, widgets, cooperação,
fecho de janelas e entrega mantêm-se abertas.

<a id="cedencia-e-retoma-local783"></a>

## Cedência e retoma local783

Incremento integrado localmente em 16 ficheiros, quatro fontes novas, sem
commit/push. O dono nativo conserva controlador/lease, revisão exata, prazo e
orçamento. Prova opaca de zero workers, publicação assinada e ACK precedem a
libertação física. Leituras/decisões pendentes usam modo próprio; nenhum worker
pode iniciar enquanto a reserva está cedida. Uma aprovação recebida fica retida
durante a espera pelo recurso. Reaquisição e ACK confirmam nova geração física;
Stop e uma prova antiga não afetam a reserva de outro dono.

O grant schema5 regista receita, revisão/hash, validade, geração e instantes de
cedência/retoma. Essa publicação não altera a revisão do objetivo nem renova o
consentimento. Publicação incerta conserva a proposta/reserva/revisão/decisão
nativas e permite reconciliar a mesma transição. Os schemas anteriores continuam
legíveis; o histórico novo não permite reconstruir `Owned`, aprovação ou `Ready`
após reinício.

| Verificação | Resultado e âmbito |
| --- | --- |
| Rust completo / check | 2440 passaram, 20 ignorados; check passou; avisos 41/54 |
| Runtime / mesmo binário no anfitrião | 303 runtime passaram; 430 no conjunto privado do anfitrião |
| Web | 1155 JS, TypeScript e Vite passaram nas fontes atuais; 42 ficheiros do bundle exatos entre build nativa e nova validação web |
| Seis regressões novas | Worker presente impede cedência; trabalho manual/outro objetivo pode reservar; espera conserva aprovação; Stop protege outro dono; falhas reais de publicação/ACK reconciliam; prova antiga não liberta nova geração/alvo |
| Tauri real privado | Outro objetivo fez claim enquanto o primeiro estava cedido; o primeiro recebeu aprovação, esperou sem inferência/efeitos adicionais, readquiriu, executou/replaneou e concluiu |
| Histórico real da fixture da app | Duas cedências/retomas assinadas, gerações1/2, mesma lease/prazo, duas tentativas de modelo e quatro de ferramenta; grant encerrado |
| Continuidade/cancelamento | Recarga da vista conservou o pedido; Stop durante HTTP, referência antiga recusada, 18 eventos nativos incluindo espera pelo recurso |
| Providers/dados reais | Zero modelos/quota reais; perfil/display do utilizador ausentes; .30, Wi-Fi, PostgreSQL e configuração do anfitrião preservados |

O segundo objetivo reservou o provider mas não executou inferência ou ações na
fixture Tauri. A prova usa modelo/leitor sintéticos, comandos Tauri reais e
perfil/D-Bus/Xvfb privados num container do projeto. Não comprova prioridade ou
ausência geral de starvation na fila, recuperação após perder o dono, fecho de
janela, todos os controlos gráficos, voz/áudio, providers/hardware reais ou
migração/backup/rollback/instalação. Expiração e revogação mantêm as guardas do
código; aceitação adicional específica durante transições cedidas fica
identificada no contrato seguinte. RUN/EXE e os 43 requisitos continuam abertos.

Raiz/SDK: 783 fontes exatas; snapshot de 798 ficheiros e 424 inputs nativos.
Mapa SHA-256 `ea52fc73513e9affc76fad21531c3404add928b8a5d5a597f48d1851b725e59f`.
Provas na unidade `.build/pet-goal-execution-next-r1/`:
`source-resource-wait783-r25.json`, `resource-wait-r25-source`,
`resource-wait-targeted-r24` (primeira revisão, quatro testes),
`resource-wait-runtime-r25`, `resource-wait-full-r25`,
`resource-wait-check-r25`, `resource-wait-web-r25`,
`HOST-NATIVE-RESOURCE-WAIT-R25.json`, `APP-WEB-DIST-BINDING-R25.json`,
`APP-NATIVE-RESOURCE-FAIRNESS-ACCEPTANCE-R25.json` e
`ROOT-APPLICATION-RESOURCE-WAIT-R25.json`. Logs/perfis/capturas privados em
`.build/app-webdriver-r19/run6/`, fora de Git. Segue
`ORIGINAL-OWNER-RECOVERY-NEXT-CONTRACT-R26.json`.

<a id="aprovacoes-concorrentes-local779"></a>

## Aprovações concorrentes local779

Correção integrada localmente em 13 ficheiros, sem commit/push. O ensaio Tauri
reproduziu o encerramento indevido de um pedido perante disputa de provas. A
validação passou a distinguir alteração real de indisponibilidade temporária:
esta conserva o pedido original, mas bloqueia aceitar/consumir a resposta até
validar. Uma resposta já recebida é revalidada sob o prazo original. Stop,
revogação e expiração mantêm o efeito de encerramento. `Busy` deixa de ser
classificado como registo inválido. As leituras autenticadas do percurso podem
coexistir; as alterações ao consentimento/recibos continuam exclusivas e mantêm
as mesmas guardas de chave/diretório/inode.

Cinco testes novos cobrem disputa real do grant e do repositório de objetivos,
configuração indisponível, revogação, Stop durante a disputa e revalidação da
resposta original; a prova de um plano gerado permite leitores concorrentes e
recusa a alteração do grant enquanto um leitor externo conserva o bloqueio.
Os 120 testes dirigidos ao controlador e a biblioteca completa passaram.

| Verificação atual | Resultado e âmbito |
| --- | --- |
| Rust completo / check | 2434 passaram, 20 ignorados; check passou; 41/54 avisos, respetivamente |
| Mesmo binário no anfitrião | 424 passaram; 297 runtime e 80 RPC, mais os grupos anteriores |
| Web | 1155 JS, TypeScript e Vite anteriores; 321 inputs web exatos e alterações posteriores só em Rust |
| App custom-protocol e Tauri reais | Criação/contexto, metadados, revisão/consentimento, claim e dispatcher; sem mock de IPC ou comandos de QA no produto |
| Execução/replaneamento | Duas propostas e aprovações; quatro efeitos no leitor privado; feedback nativo do volume18, segunda proposta com saldo restante, volume17 e conclusão nativa assinada |
| Recarga da vista | Mesmo pedido/âmbito/controlador, sem nova inferência ou novo consentimento |
| Stop durante HTTP | Segundo objetivo interrompido, driver inativo, objetivo pausado, zero novos efeitos e referência antiga recusada |
| Eventos | 14 eventos Tauri reais, incluindo espera de aprovação, conclusão e Stop |
| Modelo/quota reais e anfitrião | Zero; rede do container limitada a loopback, perfil/display do utilizador ausentes, .30 preservada |

A aceitação é do **percurso nativo através de Tauri**, com modelo e leitor
sintéticos. Não comprova inferência/protocolos de providers reais, Spotify real,
botões/formulários completos, fecho da janela (foi recarga), microfone/STT/TTS,
assets de som, migração ou instalação. A cedência do recurso durante aprovação,
recuperação após perder o dono e restantes requisitos continuam em aberto.

Raiz/SDK: 779 fontes exatas; snapshot de 794 ficheiros e 420 inputs nativos.
Mapa SHA-256 `5d31c1db0e846a13b6c487bddca910390f1b0101965fc8ce0f5dabba13e5cadc`.
Provas na unidade `.build/pet-goal-execution-next-r1/`:
`source-pending-readers779-r23.json`, `pending-readers-r23-source`,
`pending-readers-targeted-r23`, `pending-readers-full-r23`,
`pending-readers-check-r23`, `HOST-NATIVE-PENDING-READERS-R23.json`,
`WEB-BYTE-BINDING-R23.json`, `APP-NATIVE-DRIVER-ACCEPTANCE-R23.json` e
`ROOT-APPLICATION-PENDING-READERS-R23.json`. Perfis/capturas/logs privados em
`.build/app-webdriver-r19/`, fora de Git. O contrato seguinte é
`WAITING-RESOURCE-FAIRNESS-NEXT-CONTRACT-R24.json`; o objetivo completo mantém-se
ativo e os 43 requisitos continuam no âmbito.

<a id="observacao-expirada-local779"></a>

## Observação expirada local779

Incremento integrado localmente, sem commit/push: três testes novos com produtor
real do registo de capacidades, consentimento/decisão nativos e leitor privado.
Não foi necessária alteração de produção: o código já separava disponibilidade
fresca do modelo da política da receita/ferramentas independentes. Expirar a
observação conserva essas ferramentas e aprovação, exige frescura para nova
inferência e mantém Stop, revogação e alvo original.

Passaram 2429 Rust/20 ignorados, cargo check e 419 ensaios privados no anfitrião
(292 runtime, 80 RPC e os restantes grupos anteriores). Os inputs web mantêm-se
iguais aos 1155 JS/TypeScript/Vite aceites. Raiz/SDK: 779 fontes; snapshot de
794 ficheiros e 420 inputs nativos. Mapa SHA-256
`e09a58d4c47383f3c42607ea289579831430b1d68245f35613549639a6b5ce4b`.
Provas locais: `source-tool-readiness779-r18.json`, `tool-readiness-full-r18`,
`tool-readiness-check-r18`, `HOST-NATIVE-TOOL-READINESS-R18.json`,
`ROOT-APPLICATION-TOOL-READINESS-R18.json` e
`TOOL-READINESS-INTEGRATED-CHECKPOINT-R18.json` na unidade `.build` habitual.

O ensaio posterior da aplicação custom-protocol usa dependências num container
do projeto, rede desativada exceto loopback, Xvfb e D-Bus/perfil/leitor/modelo
sintéticos privados. O WebDriver comunica com os comandos Tauri reais; não há
mock de IPC nem comando de teste acrescentado ao produto. Foram observados
criação/contexto, metadados, revisão/consentimento, claim, planeamento e primeira
aprovação pendente. O ciclo falhou porque a consulta concorrente de provas pôde
encerrar o pedido. A correção e a repetição do ciclo estão em curso; a recarga
da vista e o percurso completo ainda não estão aceites. Detalhes privados em
`.build/app-webdriver-r19/run2/`. Não houve modelos/quota reais, acesso ao ecrã ou
perfil do utilizador, instalação de pacotes no anfitrião ou atualização da .30.

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

O incremento769 foi integrado e publicado no commit assinado `bbe12e2`, na
branch `feat/mochi-cli-accounts`; o seu snapshot e mapa permanecem preservados. A política/consentimento próprios,
o writer limitado e os recibos autenticados evoluíram para publicação canónica
com gate do controlador original, zero workers, CAS, readback durável e ACK no
journal. A recuperação preserva o orçamento e a autorização originais.

| Verificação | Resultado | Âmbito |
| --- | --- | --- |
| Fontes | 769 | Raiz/SDK no checkpoint publicado; snapshot e mapa preservados |
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

Na composição769, o executor recusava efeitos para planos gerados enquanto
faltavam os novos âmbitos de passo/receita. A evolução local770 está abaixo.
Continuam a faltar a ativação operacional e o dispatcher
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

## Execucao gerada local770

Código integrado localmente, **sem commit nem push**, por instrução do utilizador.
No checkpoint770, a raiz, o SDK e o mapa de 770 fontes tinham os mesmos bytes. O snapshot tem
785 ficheiros, incluindo 411 inputs nativos.

- Biblioteca completa: **2409 passaram, 20 ignorados**; `cargo check` passou.
- Anfitrião privado: **313 passaram**, incluindo 273 do runtime.
- Avisos: 77 no check e 41 nos testes; sem novos avisos neste incremento.
- Fontes web inalteradas; JS/TSC/Vite anteriores não foram repetidos neste
  incremento apenas Rust. Nenhum resultado é declarado como aceitação gráfica.
- Modelos/quota reais: 0. Perfis, instalação .30, host, Wi-Fi e PostgreSQL intactos.

Oito casos novos verificam os âmbitos Step2/Recipe3 ligados ao plano confirmado,
aprovação sem execução antes do ACK do manifesto, orçamento sem bypass por JSON,
critérios assinados não substituíveis, publicação pendente, perda de evidência,
Stop durante IO e incompatibilidade de âmbitos antigos com planos gerados. O
percurso nativo privado chega à prova final e conclusão autenticada em schema8;
a repetição conserva a prova original e não repete comandos nem observações.

No checkpoint770 continuavam pendentes o dispatcher/ativação operacional, feedback autenticado e
replaneamento, equidade, payloads de revisão grandes e aceitação real de
app/Tauri/áudio/providers. Migração/backup/rollback/entrega e os restantes 43
requisitos continuam exigidos. Os ensaios privados não concluem a unidade RUN/EXE.

Mapa local: SHA-256 `8050a434229901afbf10bb6b09e39bf151fc3b51f08fa98b537841a2830f1a34`.
Evidência em `.build/pet-goal-execution-next-r1/`: snapshot
`generated-scopes-r8-source`, mapa `source-generated-scopes770-r7.json`, logs
`generated-scopes-full-r8`/`generated-scopes-check-r8` e
`HOST-NATIVE-GENERATED-SCOPES-R8.json`.

## Ciclo nativo local778

No checkpoint do ciclo r14, a integração era local, **sem commit nem push**.
Raiz, SDK e mapa de 778 fontes tinham os mesmos bytes; o snapshot imutável
continha 793 ficheiros, incluindo 419 inputs
nativos. O incremento altera 40 ficheiros relativamente ao checkpoint770,
incluindo oito módulos novos. As pré-imagens locais foram preservadas.

| Verificação | Resultado | Âmbito |
| --- | --- | --- |
| Biblioteca Rust completa | 2426 passaram; 20 ignorados | Protocolos, HTTP sintético e bus/leitor privados |
| Runtime no anfitrião | 289 passaram | Mesmo binário, XDG/TMP privados |
| Outros ensaios no anfitrião | 47 passaram; total 336 | Leitor 9, operações 28, voz determinística 2, registo/admissão 1, capacidades limitadas 1, recursos 6 |
| JavaScript | 1155 passaram | Inclui admissão separada e arranque por referências do dono original |
| cargo check / TypeScript / Vite | Passaram | Inputs nativos e web exatos no snapshot atual |
| Avisos | 54 no check; 41 nos testes | O check anterior tinha 77 |
| Modelos/quota reais | 0 | Incluindo o replaneamento e cancelamento por HTTP simulado |
| Instalação .30 / perfil / host / Wi-Fi / PostgreSQL | Intactos | Executável original e processo preservados; a app não foi iniciada neste ensaio |

Os 17 casos Rust adicionais cobrem feedback assinado, falta de progresso,
orçamento restante, decisão ligada à revisão original, revisão grande, ciclo
nativo, recusa, dono único, execução independente do chamador, ACK incerto,
replaneamento real pelo protocolo HTTP, limites antes de inferência, tentativa
desconhecida, cancelamento durante o HTTP, libertação comprovada e capacidades
por adaptador. Uma prova antiga de drain não liberta a reserva posterior.

A validação web `driver-web-r13` inclui testes, TypeScript e Vite. O incremento
r14 altera apenas o tratamento nativo de interrupção e a sua regressão; os inputs
web foram comparados byte a byte com o snapshot atual. As validações nativas
r14 e o ensaio no anfitrião usam as fontes finais e o mesmo binário verificado.

O percurso atual restringe o modelo a Ollama local e as ações a reproduzir,
pausar e definir volume absoluto do leitor Spotify capturado. Não corresponde
a acesso geral ao Sistema ou a suporte de execução dos outros pets. Continuam
pendentes aceitação real de app/Tauri/eventos/áudio/providers, cedência/retoma
durante aprovação, recuperação sem dono vivo, outros operadores/verificadores,
migração/backup/rollback/entrega e os restantes 43 requisitos. RUN/EXE e o goal
completo permanecem abertos.

Mapa SHA-256 `f6e6e12ba35a8dda8daf6c122786a119c8ad8acbd82f1cdbffa6299bf40e3362`.
Evidência local em `.build/pet-goal-execution-next-r1/`: `driver-r14-source`,
`source-driver778-r14.json`, `driver-full-r14`, `driver-check-r14`,
`driver-web-r13`, `DRIVER778-VERIFICATION-BINDING-R14.json`,
`HOST-NATIVE-DRIVER-R14.json` e `ROOT-APPLICATION-DRIVER-R14.json`.

## Aceitacao parcial da app isolada — fonte778

As mesmas fontes foram compiladas como app com `tauri/custom-protocol`, sem
rede no build. A app .31 arrancou numa sandbox com namespace de rede próprio
(só loopback), HOME real oculto, XDG/D-Bus privados e socket Wayland observado.
Foi mostrado apenas o perfil sintético Ollama QA; atalhos globais e integrações
externas ficaram desativados nesse perfil. O leitor MPRIS e o endpoint Ollama
eram simulados e separados da sessão instalada.

Foi observada a área de objetivos e confirmado no repositório nativo um objetivo
com intenção/critérios exatos, tarefa/plano propostos e contexto dedicado,
criados através da interface Tauri. Não houve admissão, inferência ou comandos
no leitor. O primeiro draft teve texto misturado pelo harness de teclado e não
é usado como prova de preservação; o segundo foi conferido nos dados guardados.
A instância e os processos auxiliares foram encerrados; a .30 continuou no
processo/executável original.

Este ensaio **não fecha a aceitação do ciclo pela app**: faltam autorização,
claim, execução/replaneamento, eventos, continuidade com a vista fechada e Stop
nesse percurso gráfico. A acessibilidade privada foi apenas parcial. Os avisos
de áudio `text/html` mantêm a aceitação dos assets de som em aberto; não se
instalaram pacotes do anfitrião para contornar esses avisos.

Prova local: `APP-STARTUP-AND-DRAFT-CONTEXT-PARTIAL-R15.json`; detalhes privados,
configuração e capturas em `.build/app-qa-r15/`, excluídos de Git. Fonte nativa
continua em 778; modelos/quota reais, alterações persistentes do host, commits e
pushes: zero.

## Retoma nativa Codex local778

Correção integrada localmente, **sem commit/push**, num ficheiro comum do
adaptador Codex. A opção explícita `full_access` só fixa sandbox/aprovação ao
criar uma sessão nova. `thread/resume` conserva a política nativa atual: não
envia `sandbox` nem `approvalPolicy`, mesmo que a opção da aplicação continue
ligada. O nome original da sessão e a configuração nativa de ferramentas/MCP
mantêm-se. Os dois caminhos de criação e os dois de retoma foram exercitados
pelo processo RPC sintético, incluindo pedido de aprovação e stream.

- 80 testes RPC passaram; a regressão existente passou a cobrir quatro cenários.
- Biblioteca completa: 2426 passaram, 20 ignorados; `cargo check` passou.
- Mesmo binário no anfitrião: 416 passaram, incluindo 289 runtime e 80 RPC.
- Avisos: 54 no check e 41 nos testes. Fontes web inalteradas; os inputs dos
  1155 JS/TypeScript/Vite de r13 foram comparados com o snapshot atual.
- Modelos/quota reais: zero. A .30, perfis reais, Wi-Fi e PostgreSQL continuam
  fora das alterações. A aceitação parcial da app anterior usa o binário r15;
  não é declarada como aceitação gráfica desta correção de RPC.

A raiz, SDK e mapa atual de 778 fontes são exatos. Snapshot de 793 ficheiros,
419 inputs nativos; SHA-256 do mapa
`3963655130c7ba5851a75c9f9df8181e30ac0b0d080df6ace9aa6c7ad67e8226`.
Provas locais em `.build/pet-goal-execution-next-r1/`:
`native-resume-r17-source`, `source-native-resume778-r17.json`,
`native-resume-targeted-r17`, `native-resume-full-r17`,
`native-resume-check-r17`, `HOST-NATIVE-RESUME-R17.json` e
`ROOT-APPLICATION-NATIVE-RESUME-R17.json`.

Conservar as capacidades do agente e respeitar as permissões nativas são partes
da paridade. Esta correção não comprova todos os modos, ferramentas, TUI,
continuidade, voz ou providers reais. Copilot continua sem ensaios que consumam
quota. RUN/EXE, PET-03, entrega e os 43 requisitos permanecem abertos.

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
