# Plano de desenvolvimento dos pets autónomos

## 01/11 — Fala nativa e candidata mochi.32 (C6C3)

C6C3 está implementado e integrado nas fontes1003: admissão nativa por resposta/superfície, uso único, revogação e recolha da família. O ensaio nativo283 passou com sintetizador falso; o artefacto final passou o ensaio284 e foi instalado. Seguem identidade física da captura, contexto/anexos antes do áudio e ativação governada de Voz após restauro; depois as restantes componentes e dependências integrais. Não se levanta a suspensão de Voz por este incremento.

[Composição, provas, entrega e limites](releases/0.5.1-mochi.32.md).

A versão.32 está agora instalada e ativa: nome D-Bus, executável empacotado e admissão nativa verificados; três originais de preferências/conversas e cópias nativas byte-exatas. A.30 foi preservada. O ensaio284 do próprio RPM passou com helper sintético; isto não fecha áudio real nem os49 requisitos.

## 31/10 — Seleção TTS e cancelamento por resposta (C6C2, fontes1000)

Fontes1000 integradas: seleção TTS capturada na ocorrência de Voz e no preview, validade após esperas e cancelamento do renderer por resposta exata. Falha do motor escolhido conserva o texto; só a seleção explícita system pode usar síntese do sistema. [Provas e limites C6C2](../.build/pet-goal-execution-next-r1/VOICE-PLAYBACK-CHECKPOINT-C6C2A3.json).

C6C3: vincular também voice_say_native à resposta/superfície/validade e ao worker próprios; concluir alvo nativo exato de captura e revisão/estado/revogação/ativação de Voz após restauro. Sem áudio real para QA. Depois Móvel/Hooks/Credenciais e restantes dependências integrais.

## 31/10 — Entradas imutáveis da Voz (C6C1, fontes999)

Fontes999 integradas: entradas de captura/ASR e seleção de inferência imutáveis por ocorrência de Voz, usando os runners e a fábrica existentes. Alterar pet, perfil, endpoint ou preferências durante a espera não redireciona o pedido; o contexto inválido é recusado. O snapshot não renova grants. [Provas e limites C6C1](../.build/pet-goal-execution-next-r1/VOICE-TURN-CHECKPOINT-C6C1A4.json).

C6C2: fixar também o destino de TTS e o alvo nativo de captura; concluir revisão humana/estado/revogação e ativação da Voz após restauro com os gates existentes, sem abrir dispositivos/modelos reais para QA. Depois Móvel/Hooks/Credenciais e restantes dependências integrais.

## 31/10 — Fontes997: seleção de áudio exata e posse dos workers de captura (C6A/B2)

Integrado na árvore principal, sobre os executores de Voz, ditado e gravação existentes. Um microfone nomeado ausente/ambíguo não abre outro por omissão; monitors/aliases não substituem a seleção explícita. A captura do sistema procura o alvo de loopback por host/ID exatos. O inventário tem limite256. Preferências guardadas, modelos, endpoints e wire schema mantêm-se.

Capture conserva Stop, JoinHandle e holds do Coordinator comum até recolher o worker. Timeout impede play tardio e não liberta um pedido ainda vivo. Os caminhos hotword/manual/sessão recebem o Stop externo; fechar a captura normalmente não cancela o turno de inferência que se segue. Donos tipados de Voz, Ditado, microfone da Gravação e áudio do sistema da Gravação impedem que trocar de alvo fabrique outro dono. As leases de leitura são independentes: Voz e gravação podem partilhar o mesmo endpoint quando o backend permite, sem uma revogar a outra. Não são grants de pet nem exclusividade sobre outras aplicações.

A primeira versão B1 tinha exclusão global demasiado ampla; a auditoria dos callers identificou a quebra que causaria no ditado/gravação enquanto hotword conserva o stream. B2 corrige-a e acrescenta provas de leituras partilhadas e de recusa de retarget do mesmo dono. B1 e os respetivos resultados ficam conservados; não chegou à instalação.

Provas B2:19 dirigidos passaram/2 ignorados;3153 gerais passaram/38 ignorados; build595 inputs/100 avisos existentes. Inventários e streams são sintéticos, exercitando os mesmos seletores/donos de produção. Não foram abertos dispositivos ou modelos reais. Os421 inputs não-Rust e71 outputs web da raiz continuam exatos;1329 testes web/TSC/Vite anteriores são reutilizados por identidade, sem nova execução.

[Checkpoint C6A/B2](../.build/pet-goal-execution-next-r1/VOICE-AUDIO-PARTIAL-CHECKPOINT-R43F10B2C3D2C6B2.json) · [Integração997 B2](../.build/pet-goal-execution-next-r1/integration997-r43f10b2c3d2c6b2/CHECKPOINT.json) · [Contrato de ativação e dependências](../.build/pet-goal-execution-next-r1/VOICE-ACTIVATION-CONTRACT-R43F10B2C3D2C.md). Instalação, HEAD/índice Git e10 ficheiros externos gravity_managed preservados; sem commits/publicação.

Pendente: Revisão de ativação Voz utilizável, com alvo nativo capturado e validado antes de esperar, estado/recibo de captura reais, autoridade distinta para captura/ASR/inferência/TTS/operação e bindings da fábrica; sem alterar modelos escolhidos. Defaults/aliases CPAL não são pinning físico. Depois Móvel/Hooks/Credenciais e restantes critérios integrais. Esta correção não demonstra pinning físico, áudio real/humano ou ativação governada completa. Todos os49 critérios integrais permanecem abertos.

## 31/10 — Fontes995 integradas na árvore principal, instalação preservada

A árvore `/home/pmglo/Projects/coucou-codex`, branch `feat/mochi-cli-accounts`, passa de834 para995 fontes comprovadas. Adoção ficheiro a ficheiro:291 alterações, incluindo161 ficheiros novos; checkpoint recuperável com originais anteriores. As10 fontes externas gravity_managed ficaram intactas, tal como HEAD/índice Git. Nenhum commit, push, merge ou substituição da instalação.

Os593 inputs Rust coincidem byte a byte com a build e os testes privados C4. TypeScript e Vite foram executados na árvore principal; os42 outputs comuns coincidem com o SDK. O bundle principal conserva mais29 WAV existentes de NotchBuddy/Resources/sounds, iguais às fontes e ao bundle anterior. A primeira comparação estrita de bundles falhou por essa diferença real e foi reconciliada por inspeção, sem remover sons nem repetir a build. Isto não prova reprodução de áudio real. Não houve nova execução da suite Rust nem build nativa na árvore principal: a reutilização tem como prova os inputs exatos, não uma alegação de teste adicional.

[Checkpoint de integração](../.build/pet-goal-execution-next-r1/integration995-r43f10b2c3d2b3c4a1/CHECKPOINT.json) · [Verificação reconciliada](../.build/pet-goal-execution-next-r1/ROOT995-CURRENT-SCOPE-VERIFICATION-RECONCILED-R43F10B2C3D2B3C4A1.json). Os checkpoints anteriores que dizem root834 descrevem o estado antes desta adoção; não executar os seus verificadores de root834 sobre a árvore995.

C5 verificado em nativo privado: fechar a janela exata de Settings durante Bind pendente invalida o Frame, fecha a sessão própria e conserva Unconfirmed sem publicação. Uma preparação concorrente é recusada enquanto o worker detém o recurso; reabrir não renova a decisão antiga. Nativo277 passou com as mesmas fontes995/binário;276 não chegou ao fecho por navegação redundante da fixture, corrigida sem mudar produção ou remover asserções. Não se concedeu core:window:allow-close ao renderer: WM_DELETE_WINDOW foi dirigido ao PID/título/geometria observados no Xvfb privado, e hub-visible=false/diário/limpeza foram observados. [Prova e limites C5](../.build/pet-goal-execution-next-r1/SETTINGS-CLOSE-PARTIAL-CHECKPOINT-R43F10B2C3D2B3C5.json).

Próxima unidade: controlo próprio de ativação da Voz, seguido de Móvel/Hooks/Credenciais. Preservar os ASR/TTS/modelos selecionados e usar fixtures, sem abrir áudio real na QA genérica. Objetivos autónomos e fecho de chat têm lifecycle próprio e não ficam validados por C5. Os49 critérios integrais continuam abertos, incluindo autonomia/TEAM/Sistema/Guardian e aceitação/entrega.

## 31/10 — Novo pedido de atalhos no contexto atual, com histórico preservado (C3D2B3C4A1)

A composição privada SDK995/593 inputs permite um novo pedido humano de atalhos no contexto atual quando o original não pode ser observado. A revisão3 retém a cadeia e as identidades originais, mostra o destino atual e exige quatro confirmações, versão exata e uso único. Reutiliza Controller, Frame, gates, executores e diário; não concede permissões a pets. Sucesso atual conserva recibos e incerteza de scope anteriores. O portal pode partilhar configurações por aplicação: não se afirma independência por novo token, cliente, PID ou nome.

Formato8 impede leitores7 de ignorarem a incerteza histórica de Slots Verified/Reconciled. Controlos cumulativos, migração monotónica, reinício sem registo automático, backup e restauro nativo8 foram verificados.

Provas: 68 dirigidos passaram/2 ignorados; 3138 gerais passaram/38 ignorados; 1329 web/0 omitidos, TypeScript, Vite e build. Nativos274/275 atravessam Tauri/WebKit, quatro confirmações, registo/callback, segunda revisão, leitor7, reinício e restauro8. Portal sintético e X11 virtual; não aceitação humana/hardware/instalação.270/271 e272/273 conservam falhas de fixture (bindings deliberadamente desligados no restauro; profundidade2 indicada por Admission3), corrigidas sem alterar fontes/binário nem remover asserções.

[Checkpoint privado C4](../.build/pet-goal-execution-next-r1/SHORTCUTS-CURRENT-SCOPE-PARTIAL-CHECKPOINT-R43F10B2C3D2B3C4A1.json)

Próxima dependência: Lifecycle e adoção exata da composição SDK995: verificar guards após fecho de Settings/chat/janelas, Stop e reinício, sem serviço/autostart persistente; integrar a partir de fontes comprovadas quando os bloqueadores de preservação permitirem. Prosseguir com os controladores Voz/Móvel/Hooks/Credenciais e as dependências integrais. TEAM, dados/fontes/rotinas, Sistema/voz e Guardian mantêm a sequência por dependências; não estão fechados por esta prova.

## R43F10B2C3D2B3C3A2 — comparação nativa original e atual

Privado: 992 fontes/591 entradas nativas. Passaram 55 testes dirigidos, 3134 gerais (38 ignorados), 1326 web/0 ignorados, TypeScript, Vite, build e nativos266/268/269. [Fontes, provas e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-ORIGINAL-OWNER-PARTIAL-CHECKPOINT-R43F10B2C3D2B3C3A2.json).

Comparação e recusa de controlos antigos verificadas em privado. Próxima unidade C4: recuperação utilizável no contexto nativo atual com a origem antiga indisponível, preservando a incerteza antiga e nova autorização exata. Exige definir os efeitos efetivamente limitados por cada transporte; não inferir independência por nonce/nome. Depois lifecycle/adoção e quatro componentes. Os49critérios integrais continuam abertos.

## R43F10B2C3D2B3C2A1 — restauro 7 e suspensão de todas as componentes

Composição privada: 992 fontes/591 entradas nativas. Passaram 43 testes do módulo no A1. A execução geral C2 passou 3130 testes e teve duas expectativas antigas corrigidas no reteste dirigido; 38 ignorados. A1 altera apenas cfg(test), sem nova execução geral/build/nativa. Build e ensaio nativo 265 de C2, web1324/TSC/Vite B3 são reutilizados por identidade explícita dos respetivos inputs. [Fontes, provas e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-RESTORE-PARTIAL-CHECKPOINT-R43F10B2C3D2B3C2A1.json).

Restauro 7/admissão/novo backup estão verificados em privado. A origem nativa de outro namespace continua indeterminada: não há registo ou reconciliação forçada. Segue C3C3, comparação humana visível com a identidade original retida, depois história/recuperação de scope, lifecycle/adoção e quatro componentes. Todos os 49critérios integrais permanecem abertos.

## R43F10B2C3D2B3B8 — recuperação dos atalhos e leitores antigos

Composição privada: 992 ficheiros de fonte e 591 entradas nativas; backend e binário B7, uma correção de fixture cfg(test) B8. 52 testes dirigidos B8 passaram; a execução geral B7 teve 3129 testes aprovados, uma falha de fixture e 38 testes ignorados; a falha foi conservada e corrigida com reteste dirigido.Os 1324 testes web, TypeScript e Vite de B3 e os 18 testes D-Bus de B5 reutilizados por identidade explícita; build B7 e ensaios nativos 261 (X11) e 262 (portal) passaram. Não houve nova execução dessas suites no B8. [Fontes, provas e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-RECONCILIATION-PARTIAL-CHECKPOINT-R43F10B2C3D2B3B8.json).

Próximo critério: restauro/admissão nativos do backup 7 e história do dono/recurso num novo perfil, sem reativação ou renovação de autoridade. Depois lifecycle/adoção e as outras quatro componentes; seguem os restantes requisitos integrais. Nenhum dos49 critérios foi fechado.

## R43F10B2C3D2B3A1 — resultado pendente e publicação

Privado984/B3A1 fecha a janela de admissão após publicação falhada: resultado pendente durável, handle local e commit final exato. Próximo B3B: observação do dono/recurso, reconciliação humana utilizável e reinício, depois histórico/restauro/lifecycle/adoção e quatro componentes restantes. Nenhum critério integral dos43+6 foi fechado. [Provas e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-PUBLICATION-PARTIAL-CHECKPOINT-R43F10B2C3D2B3A1.json).


## R43F10B2C3D2B2B3 — aprovação dos atalhos e suspensão X11

Implementado e validado em privado983: aprovação/despacho/resultado dos atalhos, ambos os executores e correção da suspensão X11. Próximo: B3 — incerteza entre registo/publicação/limpeza, recuperação e nova revisão; depois histórico/restauro/leitores, adoção e quatro componentes restantes. Os43IDs e GUA-01..06 mantêm todos os critérios. [Checkpoint e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-APPROVAL-PARTIAL-CHECKPOINT-R43F10B2C3D2B2B3.json).


## R43F10B2C3D2B2A2 — dados dos atalhos e barreira de novo registo

27/10/2026: privado980/582inputs. O editor usa patch nativeUser por ação com valor anterior/CAS e ACK; snapshots gerais antigos deixam de sobrescrever shortcuts. O mesmo diário Settings6 pausa novos registos antes de publicar dados, preservando IDs desconhecidos, campos alheios e slots incertos. Uma publicação falhada conserva a barreira. Verified de shortcuts continua a ser facto de uma sessão, sem remover a barreira persistida. Recusa de admissão mantém teclas existentes e o callback de navegação; suspensão explícita deixa de mostrar Active obsoleto.

Passaram37dirigidos,3104Rust/29ignorados/build582 e1312web/0skip/TSC/Vite A1 (A2 só converte o erro Rust; web reutilizado por identidade). Build101avisos, menos o novo aviso deprecação B1. Caso nativo247 passou: UI real, CAS/ACK, snapshot geral antigo, callback genuíno, suspensão e leitor976/reinício980 sem novos registos. O wrapper original terminou1 na auditoria posterior de uma fixture derivada; verificador de leitura reconstituiu a derivação exata e concluiu as verificações sem repetir efeitos. O retorno separado do contentor não ficou guardado. [Fontes, comandos e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-DATA-PARTIAL-CHECKPOINT-R43F10B2C3D2B2A2.json).

A aprovação e o despacho duráveis continuam por implementar sobre a mesma captura B1; este incremento é de dados/revogação, não reativação ou paridade instalada. Seguem consumo da reserva, guards durante IO e resultado antes de Active, depois reconciliação/lifecycle/composição. Root834/.30 e os49critérios integrais permanecem abertos/intactos.


## R43F10B2C3D2B1A2 — revisão do plano nativo dos atalhos

27/10/2026: privado976/580inputs. A revisão reutiliza planner, Registry/worker, Context/Revision/Frame e Bridge/definições existentes. Captura preferências completas, ações/conflitos, perfil/revisões e instância nativa; o portal reserva peer/client/session/requests sem criar, registar ou fechar atalhos. Preparar/inspecionar/cancelar não concede ativação. Mudanças de janela, dados ou estado nativo invalidam a captura. O recurso identifica o registrar; mudar teclas ou caminhos reservados não cria independência. Geração u64 mantém decimal exato no TS; dados hostis e IDs legados são dados.

Passaram29dirigidos,9D-Bus privados,3098Rust/29ignorados/build580,1308web/0skip/TSC/Vite A1 e nativos245/246. A2 só corrige duas fixtures Rust: web A1 reutilizado por identidade, sem nova execução.245 usa portal sintético;246 usa plugin X11 real em Xvfb.243/244 conservam falhas da instrumentação do teste, corrigida sem alterar produto/binário. Há um novo aviso de depreciação atómica, registado para B2. [Fontes, comandos e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-REVIEW-PARTIAL-CHECKPOINT-R43F10B2C3D2B1A2.json).

Segue B2: consumir esta mesma captura na aprovação exata, intenção/despacho/resultado Settings6 e gravação só do campo de atalhos; depois reconciliação/lifecycle/composição. B1 não é reativação nem entrega. Root834/.30 intactos;49critérios integrais abertos.


## R43F10B2C3D2A3A1 — perda do dono nativo dos atalhos

27/10/2026: privado970/577inputs. A subscrição do peer único antecede a consulta de vida; só a transição exata emitida pelo daemon retira disponibilidade. Troca do nome do serviço com o peer antigo vivo conserva a sessão; morte do peer termina o worker e cancela Flow sem novo comando, sem seguir substituto ou afirmar rollback remoto. Passaram12portal/8ignorados,8D-Bus privados/0ignorados,3093Rust/28ignorados/build e nativo242. Este último usa Tauri/WebKit reais e broker sintético/Xvfb: não é compositor/áudio/humano/instalado. Web/TSC/Vite reutilizados por identidade exata, sem nova execução. [Fontes, comandos e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-OWNER-LOSS-PARTIAL-CHECKPOINT-R43F10B2C3D2A3A1.json).

Segue revisão/controlo durável próprio de Atalhos pelo mesmo diário Settings6, depois Voz/Móvel/Hooks/Credenciais e composição/adoção. Root834/.30 intactos;49critérios integrais permanecem abertos.


## R43F10B2C3D2A2C3 — origem dos atalhos e conflito CAS

27/10/2026: privado970/577inputs. O portal autentica o peer único real, prende chamadas/sessão/geração e aceita apenas ações pedidas/aceites; release/rebind excluem callbacks antigos e Flow conserva release/cancel. A composição revelou um conflito real na leitura da memória: `load_for_update` partilhado distingue conflito anterior à publicação, recusa do guard, dados inválidos e commit incerto, sem retry/default ou renovação de autoridade. Passaram3092Rust/25ignorados com1024disputas,4provas CAS,11memória/build e nativo241 privado. Cinco testes D-Bus B1 são reutilizados por identidade dos três ficheiros; web/TSC/Vite E4 reutilizados por identidade não-Rust/bundle, sem nova execução. [Fontes, provas, falhas retidas e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-CALLBACK-PARTIAL-CHECKPOINT-R43F10B2C3D2A2C3.json).

Falta detetar desaparecimento do dono nativo; depois revisão/controlo durável próprios de Atalhos, Voz/Móvel/Hooks/Credenciais e composição/adoção/entrega. O nativo241 usa broker sintético em Xvfb, não compositor/áudio/aceitação humana ou instalada. Root834/.30 intactos;49critérios integrais abertos.


## R43F10B2C3D2A1 — início dos atalhos pelo mesmo diário

27/10/2026: a guarda `execution_for_component` reutiliza o dono/Revision/perfil e exige o slot despachado da própria componente; o wrapper autostart continua a usar a mesma regra.18testes do diário passaram, incluindo recusa de outro componente/id/binding/revisão, Prepared, tentativa de usar autostart e revogação após mutação. Isto não concede operação nem torna os atalhos reativáveis. Composição completa/build/nativo continuam a ser E4; A1 é incremento dirigido privado. [Prova e próximo critério](../.build/pet-goal-execution-next-r1/SHORTCUTS-WITNESS-PARTIAL-CHECKPOINT-R43F10B2C3D2A1.json). Seguem vínculo de geração/sessão e controlo durável próprios, preservando os executores existentes.

## R43F10B2C3D1B2E4 — identidade física, origem legada e protocolo6

27/10/2026: identidade física2 e revisão própria da origem legada desconhecida/4ACK implementadas no privado970; Settings6, recibos/histórico e controlo funcional preservam Unknown e suspensão.3082Rust/21ignorados,45dirigidos/build577 e nativo239 (migração5→6, substituição da base e backup/restauro6) passaram.24web/TSC/Vite E4;1302web E3 reutilizados com a falha TSC E3 explicitada. [Prova, fontes e limites](../.build/pet-goal-execution-next-r1/SETTINGS-AUTOSTART-PHYSICAL-PARTIAL-CHECKPOINT-R43F10B2C3D1B2E4.json). Segue Atalhos pelo mesmo diário; outras4componentes, composição/adoção/entrega e critérios integrais49 abertos.

## R43F10B2C3D1B2D5 — observação histórica e arquivo só de leitura

27/10/2026:970fontes privadas/577inputs. O novo controlo seleciona um EffectId incerto na cadeia local validada, apresenta o plano nativo e exige decisão própria; reutiliza diário, artefactos imutáveis, gate, Frame/TTL, cancelamento e cerca física. A observação mantém o Unknown antigo, preferência, registo e suspensão; não prova o resultado passado. Revisão de dados ou perfil diferentes retiram elegibilidade, conservando a evidência. A operação atual incerta no mesmo recurso é reconciliada primeiro. Outra alteração exige novo plano/decisão.

3074Rust/21ignorados/build577 passaram;1299web/TSC/Vite D3 reutilizados por identidade exata (sem nova execução web D5). Nativos236/238 privados passaram: revisão histórica/3ACK, cancelamento/replay/seletores recusados, leitor5 anterior conservador antes/depois de freshSet, revisão de dados, reinício/backup5 e restauro real do arquivo montado read-only para novo perfil, onde observações anteriores ficam inelegíveis.237 recusou corretamente origem/destino no mesmo dispositivo através de mounts distintos;238 usa a mesma origem sintética byte-exata num tmpfs distinto e readonly, sem relaxar essa proteção.235 revelou uma falha real: lock_shared abria o ficheiro com O_RDWR; a correção usa O_RDONLY só na leitura, preservando exclusão e O_RDWR dos escritores. Fixtures dirigidas provam consulta0400, Busy com escritor e recusa de concessão.233 foi preparação incompleta dos pais legados sintéticos;234/236 preservam-nos offline, sem limpar dados. [Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/SETTINGS-AUTOSTART-HISTORICAL-PARTIAL-CHECKPOINT-R43F10B2C3D1B2D5.json).

Limite por resolver antes da integração: o plano prende o destino físico durante IO, mas o identificador persistido D5 ainda deriva do caminho; falta distinguir a substituição da pasta-base pelo mesmo nome e definir tratamento dos registos legados. Segue esse caso concreto, depois composição/outras5componentes. Root834/instalação .30 intactos; sem modelos/quota, aceitação humana/login/hardware ou fecho dos49requisitos.

## R43F10B2C3D1B2C3 — backup5 restaurado e admitido numa terceira geração

27/10/2026: mesmas969fontes B2, novo nativo232 privado/0modelos. O backup5 real foi restaurado, revisto e admitido pelo percurso de época/custody existente; o caminho operacional é o payloadDepth=2 selado nesta cadeia. Novo scope suspende o recibo atual e conserva o ancestral incerto. Outra aprovação nativa altera só o novo recurso, mantém5 e só revalida autostart; reinício conserva ficheiro/inode/diário/autoridade/orçamentos. Root1/Root2 permanecem byte-exatos e o terceiro backup5 inclui os roots completos, controlos, originais e histórico.230/231 conservam erros de expectativa do harness (ordem custody→configuração e prefixo operacional);232 corrige essas expectativas, sem modificar produção.3066Rust/21ignorados,1294web/TSC/Vite/build B2 são reutilizados por identidade exata; não foram repetidos. [Prova e limites](../.build/pet-goal-execution-next-r1/SETTINGS-AUTOSTART-PORTABLE5-PARTIAL-CHECKPOINT-R43F10B2C3D1B2C3.json).

Segue controlo explícito para observar o recurso de um efeito incerto arquivado quando voltar a ser o alvo nativo atual; permanece recusado até essa revisão. Depois composição/outras5componentes. Sem instalação/aceitação humana/login/hardware;49critérios integrais continuam abertos.

## R43F10B2C3D1B2B2 — novo destino com incerteza antiga preservada

27/10/2026, privado969/B2/Linux: setIndependentRegistration exige perfil **e** recurso nativos diferentes e quarto ACK próprio da incerteza externa, ligado ao mesmo plano/hash/Owner-epoch. Reutiliza Controller/gate, diário/Revision, preferência guardada e cerca física. O slot/artefactos anteriores conservam a fase incerta; nenhum caminho arquivado é seguido e não há falsa reconciliação. Status/UI mantêm visíveis os ancestrais incertos após novas operações; leitura local limitada/hash-bound recusa corrupção/ciclos e novo trabalho no recurso antigo até reconciliação própria. Primeira operação independente usa marker/Record5: o leitor4 anterior recusa; gravações comuns, revisão e backup preservam5, sem downgrade.

13testes do controlador/22diário-protocolo,3066Rust/21ignorados,build576 e nativo229 passaram.1294web/0skip/TSC/Vite foram executados B1 e reutilizados B2 por inputs/bundle idênticos.229 prova backup4→restauro/admissão→UI4acks→novo efeito5 com incerteza antiga conservada/visível, cancel/wrongwindow/args/replay,reinício/revisão5,recusa do leitor4 e backup5,0modelos/quota. Unknown é fixture após registo privado real; não powerloss físico. [Fontes, provas e limites](../.build/pet-goal-execution-next-r1/SETTINGS-AUTOSTART-INDEPENDENT-PARTIAL-CHECKPOINT-R43F10B2C3D1B2B2.json).

Seguem restauro do backup5 numa nova Root3 e controlo explícito de reconciliação histórica do recurso antigo, depois composição/outras5componentes. Sem instalação, aceitação humana/login/hardware;49critérios integrais continuam abertos.

## R43F10B2C3D1B2A4 — restauro nativo e nova revisão de autostart

27/10/2026: mesmas969fontes A6, novo ensaio nativo228 privado,38.56s/0modelos. Backup4→restauro→configuração/autoridade independente/admissão→nova revisão nativa no destino Root2 passou. O restauro mantém o recibo anterior como dado suspenso e desativa a preferência; nova aprovação afeta apenas o recurso atual e revalida só autostart, sem modificar pendingFeatures imutável. Reinício não repete efeitos. Nova revisão de dados conserva formato4 e recibos, mas volta a suspender as6componentes; exige outra aprovação. Leitores958/protocolo3 recusam e o segundo backup conserva os roots completos, controlos e originais. Rust/web/build A6 reutilizados por identidade exata, sem nova execução. [Prova e limites](../.build/pet-goal-execution-next-r1/SETTINGS-AUTOSTART-RESTORED-PARTIAL-CHECKPOINT-R43F10B2C3D1B2A4.json).

Continua aberto o efeito antigo incerto num recurso diferente; não alegar reconciliação desse caso. Seguem esse percurso explícito, composição exata e outras5componentes. Sem aceitação humana/instalada/login/host;49critérios integrais continuam abertos.

## R43F10B2C3D1B1A6 — controlo nativo de autostart e reconciliação

27/10/2026, privado969/A6/Linux: percurso funcional de preview/3acks/decisão nativa de uso único e5min, ligada ao Owner/epoch da janela C3B. Reutiliza Controller/gate, diário/artefactos/WriterLock/Target/Witness existentes. Intenção precede preferência guardada com guard; Dispatched precede registo. IO ocorre fora de Shared, a validade usa Witness/Observation e epoch fraco da Revision real, cancelamento atómico não espera esse mutex e o gate dura até retorno do worker. Cerca física com flock prende o recurso entre instâncias. Scope/configuração/plano/recibo e referência da revisão dos dados são revalidados; observação recuperada mantém suspensão e exige outra aprovação para alterar. Não há novo grant/scheduler/ledger de objetivos.

Linux recusa a API antiga de mutação e startup só observa drift. UI/Bridge tipados mostram efeito distinto de preferência e login, sem retry implícito; evento Settings atualiza disponibilidade após revisão de dados.3061Rust/21ignorados A6,1288web/0skip/TSC/Vite A3 reutilizados por bytes iguais e buildA6 passaram. Nativo224 prova UI Root1,6suspensões→sóautostart,wrongwindow/args/replay/close recusados,registo/remover/restart/drift,reconciliação sem repetição e novo consentimento,recusa958 e protocolo3 anterior, recibo3 sódados e backup4completo,0modelos/quota.219 encontrou estado visual desatualizado; corrigidoA3.220/221 identificaram settingsBusy por leitor passivo; A5corrigiu com guard da revisão real,10+2testes e nativo222. A6 cria barreira4 contra escritores legados3 e valida em224. Toda evidência anterior conservada. [Fontes, comandos e limites](../.build/pet-goal-execution-next-r1/SETTINGS-AUTOSTART-REVIEW-PARTIAL-CHECKPOINT-R43F10B2C3D1B1A6.json).

Falta Root2/restauro/overlay/scope e composição conjunta antes de avançar às outras5componentes. Unknown em224 é fixture controlada, não powerloss físico; ficheiro verificado não prova login. Sem aceitação humana/instalada ou host real. Raiz834/.30 preservadas;49critérios integrais continuam abertos. Segue C3D1B2.

## R43F10B2C3D1A4 — diário de efeitos e plano exato de autostart

27/10/2026, privado963/A3/Linux: primeiro incremento de C3D, ainda sem percurso nativo de reativação. O plano congela launcher, alvo, bytes anteriores, diff e pós-condição; reutiliza Directory/Witness e o gerador existente. Record3 estende o mesmo diário com intenção, despacho e outcome por componente, artefactos imutáveis limitados e binding de perfil/configuração/recurso/plano. Preferências normais e revisão de dados preservam estados; restauro não reativa recibos. Observação recuperada não é autorização para replay. Marker3 exige1/2; compatibilidade e promoção de upgrade interrompido são verificadas por fixtures.

3046Rust/21ignorados em A3, 21dirigidos/0ignorados e1ACKperdido/0ignorados em A4 e build573customprotocol A4 passaram. A4 altera apenas3fontes do plano/Target/testes; a regressão A3 é reutilizada com essa diferença explícita, sem alegar suite completa A4. Inclui9testes de plano e16do diário/perfil: efeito real apenas em ficheiros privados, ACK perdido após rename, recusa de replay/configuração trocada, revogação antes de despacho e persistência de suspensões. A1 passou7testes mas revelou hash semântico incorreto; SHA256 e known-answer test corrigidos antes destas provas. A4 conserva o Witness devolvido pelo escritor e recusa substituição do ficheiro/pai/raiz mesmo com bytes iguais antes da confirmação. [Fontes, comandos e limites](../.build/pet-goal-execution-next-r1/SETTINGS-AUTOSTART-JOURNAL-PARTIAL-CHECKPOINT-R43F10B2C3D1A4.json).

Faltam ligar os handlers/UI nativos à aprovação exata/cancelamento/reconciliação e provar o percurso conjunto, backup/leitor antigo e restante reativação. Registo de ficheiro não prova login; fixtures não são aceitação nativa/humana/instalada. Sem novos grants/ledger/scheduler, efeitos reais de host/cofre/quota ou instalação. Raiz834/.30 e49critérios integrais abertos. Segue C3D1B; as outras5componentes continuam pendentes.

## R43F10B2C3B1B4 — revisão exata dos dados, sem reativar componentes

27/10/2026, privado958/B4/Linux: revisão nativa de uma janela,5min e uso único captura valores efetivos/propostos/defaults, origem/revisões, configuração/pets/modelos/destinos e referências de credenciais sem consultar o cofre. Recusa janela errada, argumentos adicionais, replay, alterações e revogação antes da publicação. Epoch nativo invalida após fechar/recriar a janela; os guards não fazem query GTK sob mutex Settings. O original e o outcome anterior ficam retidos antes da intenção nova; aprovar dados conserva6componentes suspensas e não aprova objetivos ou efeitos antigos. [Contrato e limites](../.build/pet-goal-execution-next-r1/SETTINGS-HUMAN-REVIEW-NEXT-CONTRACT-R43F10B2C3B.md).

Marker2 precede Record2 e recusa leitores950/anteriores; gravações normais de v1 mantêm compatibilidade. Registo/backup/restauro conservam suspensões e history. Preferências incompletas persistem os valores revistos; Root1 conserva campos opacos, o restaurado retém originais e publica o formato canónico. Um perfil já admitido com preferências danificadas abre apenas dados protegidos para revisão; root/chave/manifesto/custódia desconhecidos continuam recusados antes do carregamento.

3022Rust/21ignorados,build568 e nativos217/218 passaram.217 prova UI nos3percursos Root1/Root2/Root2danificado,stale recusado,7operações bloqueadas após aprovação,reinícios/runtime/backup e old950Root1+Root2 recusados;2HTTP2CLI sintéticos0quota.218 prova fecho normal do único hub/PID próprio em Xvfb e token recusado após reabrir,0modelos; ACL de fecho JS não foi alargada.1278web/TSC/Vite B2 reutilizados por bytes exatos da web/bundle. [Fontes, provas e diagnósticos](../.build/pet-goal-execution-next-r1/SETTINGS-HUMAN-REVIEW-PARTIAL-CHECKPOINT-R43F10B2C3B1B4.json).

Seguem reativação por componente e intenções/recibos/reconciliação de listeners/chaves/hooks/registo (C3D), antes de integrar/adotar. Sem host/cofre/instalação/áudio reais, aceitação humana, sandbox sameUID ou powerloss físico. Raiz834/.30 preservadas;49critérios integrais abertos.

## R43F10B2C3B0 — origem confirmada antes de componentes opcionais

27/10/2026, privado950/B0A1: Voice/Mobile/Autostart/Shortcuts/Hooks/Credentials reutilizam a origem de IO do Shared atual antes de novos efeitos. Mantêm as barreiras de restauro e repetem a verificação após consulta ao perfil; microfone/listener repetem depois da espera pelo recurso/cofre. Stop e histórico conservam os seus caminhos. As preferências desejadas não são desligadas para contornar a revisão. O relatório de atalhos identifica suspensão por preferências não confirmadas; erro ao eliminar uma credencial vazia deixa de ser sucesso falso. [Contrato e limites](../.build/pet-goal-execution-next-r1/SETTINGS-OPTIONAL-EFFECTS-ADR-R43F10B2C3B0.md).

7dirigidos/3000Rust21ignorados/1272web/TSC/Vite/build563/nativo215 passaram. Tauri real recusa7comandos antes do efeito em Changed e ReviewRequired; flags autostart/phoneLink true conservadas, listener bloqueado após dois reinícios/nova gravação, estado de atalhos visível. Providers/helpers sintéticos,0quota; áudio real, cofre real, aceitação humana e instalação não exercidos. [Fontes, comandos e provas](../.build/pet-goal-execution-next-r1/SETTINGS-OPTIONAL-PARTIAL-CHECKPOINT-R43F10B2C3B0A1.json).

É uma pré-condição de C3B, não revisão humana nem revalidação operacional. Seguem captura/decisão humana exatas, suspensão persistente separada da aprovação de dados e intenções/recibos de listeners/chaves/hooks. Sem integração/instalação antes da prova conjunta. Raiz834/.30 preservadas;49critérios integrais abertos.

## R43F10B2C3A4 — intenção de gravação e revisão duráveis

27/10/2026, composição privada 948/C3A4/Linux: o writer comum persiste Prepared antes de publicar preferências e Committed depois de fsync/readback; uma falha anterior distingue NotPublished. A revisão de valores recuperados sobrevive a load/save/restart. O registo usa Document/Snapshot/WriterLock existentes e contém hashes/estado, sem conceder permissões. O marcador imutável na raiz impede leitores anteriores de carregar o novo perfil; o registo mutável acompanha o payload no backup e na inicialização revista de restauro. Prepared e revisão pendente não são convertidos em sucesso. A consulta de IO usa um descritor de formato capturado, sem tomar o mutex geral do perfil. [Contrato e limites](../.build/pet-goal-execution-next-r1/SETTINGS-DURABLE-IO-ADR-R43F10B2C3A.md).

Passaram 13 casos dirigidos, 2 993 testes Rust (21 ignorados), build de 561 inputs e nativo212. O Tauri conserva o bloqueio após dados diferentes e reinício; num perfil sintético separado, mantém ReviewRequired por dois reinícios e outra gravação, com o estado visível na UI. Leitores945/915 recusam no arranque, antes de Settings/Chat; esta execução não chega ao RPC Spec13 antigo. Mantém a regressão de runtime/Stop/reinício/backup e pipe parcial de 4 096 bytes; 2HTTP/2CLI sintéticos, quota real zero. Web1272/TSC/Vite reutilizados por igualdade exata de fontes/bundle. [Fontes, comandos, falhas conservadas e alcance](../.build/pet-goal-execution-next-r1/SETTINGS-DURABLE-PARTIAL-CHECKPOINT-R43F10B2C3A4.json).

C3 permanece aberto: seguem a revisão/reconciliação humana exata e as intenções/recibos de listeners e chaves. Não revalidar funcionalidades restauradas nem adotar/instalar antes da prova conjunta. Metadados/hashes não são sandbox, CAS contra sameUID ou prova de powerloss. Raiz834 e instalação.30 preservadas; os 43+6 critérios integrais continuam abertos.

## R43F10A4 — recusa antes de guardar e preferências privadas

26/10/2026,privado925/A4: nativo200 demonstrou autostart guardado como ativo e sucesso devolvido apesar da barreira de restauro, além de settings.json0644 incompatível com o leitor privado. O mesmo escritor comum passa a validar sob mutex antes de guardar/publicar; a verificação antes do efeito mantém-se. Erro de registo posterior indica preferência guardada/efeito não confirmado. Temporário exclusivo0600 conserva ficheiros alheios e a compatibilidade legacy. O controlo funcional consulta o valor nativo após erro, sem repetir a ação. [Decisão e limites](../.build/pet-goal-execution-next-r1/RESTORED-FEATURE-PRECOMMIT-ADR-R43F10A.md).

81dirigidos/2892Rust21ignorados/1266web/TSC/Vite/build542/native201 passaram. Este prova recusa na UI/IPC sem mudar memória/bytes/eventos,erro real de registo num caminho sintético bloqueado,gravação normal0600/reinício/backup e regressão do runtime13/consentimento/decisões/Stop.2HTTP1CLI sintéticos/DBus privado; HOME vazio privado ligado só no container,nenhuma entrada/serviço de arranque criado,nem HOME autenticado copiado. [Fontes,prova negativa e checkpoint](../.build/pet-goal-execution-next-r1/RESTORED-FEATURE-PARTIAL-CHECKPOINT-R43F10A4.json).

Voz,móvel e arranque continuam pendentes; não foi criada uma concessão de revalidação. Antes dela,fechar intenção por campo/concorrência e confirmação após publicação/fsync do diretório no escritor comum. Aceitação humana/instalada/áudio/hardware e49critérios integrais abertos. Raiz834/.30 preservadas,sem commits/publicação; seguir migrações/revalidações/adoção e dependências restantes.

## R43F9B4A1 — limites conjuntos da retenção comprovados

26/10/2026: três casos no RetentionReview/fixture existentes aceitam4096 entradas (diretórios incluídos) e64MiB repartidos por um original retido e a preparação atual; recusam4097,64MiB+1 e um ficheiro8MiB+1. As recusas preservam bytes/inodes e não publicam nova autoridade. A contagem não reinicia por pasta. [Fontes, comando e prova](../.build/pet-goal-execution-next-r1/RETENTION-BOUNDARY-CHECKPOINT-R43F9B4A1.json).

Só mudou o ficheiro de testes; produção/web são byte idênticos à composição923/B3F2A7 comprovada. B4 tem3dirigidos/0ignorados; a suite2886Rust/21ignorados,web1262/build542/native199 pertencem exatamente aA7 e não foram repetidos como se fossem novos ensaios. Sem quota/host/instalação/commits. Seguem migrações/revalidações/adoção das fontes exatas e restantes dependências;49critérios integrais continuam abertos.


## R43F9B3F2 — trabalho futuro com origem restaurada, validado em privado

26/10/2026, composição923/A7: revisão nativa da origem/revisão e intenção/critérios futuros, destino capturado e aceitação explícita da incerteza cria outro GoalId do mesmo pet. Spec.restoredFrom13 sela apenas a criação de dados pelo HMAC/Authority/Store existentes; não concede execução, saldo, memória ou autoridade anterior. Contexto, consentimento, modelo, limites e decisões seguintes passam pelo runtime comum. Idempotência reconhece o original sem renovar escolhas; validade/origem são verificadas nas fronteiras da publicação. [Contrato e decisão](../.build/pet-goal-execution-next-r1/RESTORED-FUTURE-DRAFT-ADR-R43F9B3F2.md).

Objetivos antigos concluídos são consultáveis como estado declarado no snapshot, sem exigir a Authority original para leitura. read_control/mutate/context/admissão recusam controlo antigo; UI identifica o histórico e só oferece consulta e revisão de trabalho futuro. Não se infere consumo zero, morte do dono ou desfazer de efeitos.

32dirigidos,2886Rust/21ignorados,1262web/TSC/Vite e build542 passaram. Nativo199 atravessa cartão/revisão/checkbox/criação, contexto e consentimento próprios, driver/decisão/verificação/conclusão, Stop, reinício/ACK com arquivo offline e backup portátil. Leitor915 real recusa13 preservando os ficheiros. Época seguinte13 e cortes/tamper/expiração são fixtures. [Checkpoint com fontes, comandos, falhas retidas e limites](../.build/pet-goal-execution-next-r1/RESTORED-FUTURE-PARTIAL-CHECKPOINT-R43F9B3F2A7.json).

Aplicação privada com HTTP2/CLI1 sintéticos e DBus privado,0quota/modelos reais; sem aceitação humana/instalada/áudio/hardware. Raiz834 e instalação.30 preservadas,sem publicação. A prova fecha este percurso privado, não os49 requisitos integrais. Seguem fronteiras conjuntas4096/64MiB, migrações/revalidações e adoção das fontes exatas; restantes dependências e entrega continuam abertas. Secções anteriores conservam as fotografias históricas datadas.

## R43F9B3A — histórico restaurado com origem e consumo desconhecido

Privado915/A2 fecha um falso zero demonstrado por prova negativa: o histórico comum ignorava corretamente grants históricos para execução, mas apresentava total zero a um objetivo sem admissões ligadas. A consulta nativa existente agora verifica GoalRef/hash/dono da boundary HMAC e os mesmos bytes/assinaturas/fences do leitor histórico. Resposta2/restoredSnapshotOnly liga a origem exata, mantém cumulativeCharge desconhecido e recusa autoridade, continuação ou retirada de grants históricos. Trabalho novo conserva resposta1; sem migração/escrita passiva, provider ou ledger novo.

O controlo funcional de histórico mostra proveniência e incerteza, sem subtotal zero enganador. Passaram os dirigidos,2870Rust/21ignorados,1256web/TSC/Vite,build536 e nativo196. Este prova IPC real/todas as origens antigas/revisões/bytes só leitura antes de IO e resposta1 para trabalho novo, compondo retenção/admissão/consentimento/Stop/reinício/backup já comprovados. HTTP2/CLI1 sintéticos e DBus privado;0quota/modelos reais. [Fontes, comandos, prova negativa e limites](../.build/pet-goal-execution-next-r1/RESTORED-HISTORY-PARTIAL-CHECKPOINT-R43F9B3A.json).

B3F2/continuação histórica ainda não está implementada: exige revisão própria da origem/dados/efeitos incertos e novo consentimento/contexto/destino/limites/causalidade no mesmo runtime. Três épocas/tamper são fixtures; UI testada em JS e IPC nativo,sem aceitação visual humana/áudio/hardware/instalação. Raiz834/.30 intactas,sem publicação;revalidações/adoção/restantes dependências e49critérios integrais abertos.

## R43F9B2D4D2B — autoridade futura distinta após retenção

Privado914/B1 compõe a retenção opaca913 com preparação nativa separada. Binding1 liga head/fingerprint semântico/count; Intent2/generation2 declaram propósito de nova chave futura, com intenção durável antes de RNG e outra aprovação exata. O hash da retenção não autoriza a chave. Dados retidos continuam não fiáveis, sem grants/saldo novos. Custody2 explicita a ligação; formatos1/receipt3 sem retenção conservam serialização.

Configuração4 apenas inspeciona, num recibo por head; preserva recibos1/3/4 anteriores. Activation usa esse caminho exato e a mesma chave preparada em Admission4/Bound/runtime. Opened/reinício/ACK conservam trabalho novo sem recapturá-lo como histórico e dispensam o arquivo externo. Backup2 conserva a raiz e custódia completas; fixture de época seguinte mantém Custody2 e negações de IDs anteriores.

7dirigidos novos,2864Rust/21ignorados/build535 e nativo195 passaram. Este ensaio usa CLI/Tauri reais: duas retenções, chaves distintas sob aprovações próprias, mesmos inodes/bytes originais, recibos preservados, recusa de leitor913, admissão4, sessão CLI sintética nova, objetivo com consentimento próprio, decisão/2efeitos no DBus privado, Stop, reinício/ACK offline e backup/inspeção portátil. São2pedidos HTTP e1turno CLI sintéticos,0quota/modelos reais. [Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/RETAINED-ADMISSION-PARTIAL-CHECKPOINT-R43F9B2D4D2B.json).

Composição privada, não entregue nem instalada; raiz834/.30 intactas,sem commits/publicação. Época seguinte é fixture neste incremento; áudio/hardware/móvel/autostart/aceitação humana e fronteiras conjuntas4096/64MiB não foram fechados por este ensaio. Seguem migrações/revalidações/B3/adoção a partir de fontes exatas; restantes dependências e49critérios integrais mantêm-se abertos.

## R43F9B2D4D2A — retenção nativa de preparação incompleta

Privado913/A2 acrescenta o comando CLI de retenção, reutilizando Claim/Review/Dir/inventário. Exige Header2/archive1 não admitido, ausência original/operacional e payload exato. Cadeia privada versionada conserva JOB opaco e o seu inode após intenção durável; transferência entre pais fixados sem substituição, fsync e verificação precedem ready. Dados retidos nunca são políticas/grants; não há geração de chave. Leitores normais recusam todas as pendências. Nova preparação permanece bloqueada até a composição separada D4D2B.

11dirigidos,2855Rust/21ignorados/build534 e nativo193 passaram. CLI real prova revisão sem escrita, recusa de hash/leitor antigo, retenção de chave perdida e pre-manifest hostil com bytes/inodes exatos, cadeia2, retoma só do recibo após falha sintética e bloqueio de preparação nova.0modelos/quota/CLIagents/efeitos físicos. A1 passou9/full2853 mas a revisão encontrou contentor apenas com protocolo aceite por leitor normal; A2 fecha essa fase e acrescenta regressões de pendência válida e conteúdo portátil versus identidade física. [Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/INDEPENDENT-RETENTION-PARTIAL-CHECKPOINT-R43F9B2D4D2A.json).

Conteúdo portátil foi verificado em fixture, não backup/admissão operacional. D4D2B ainda deve ligar manifesto2/Custody2/configuração/Activation/Bound/backup/época sob outra revisão, sem renovar autoridade. Limites conjuntos4096entradas/64MiB implementados, fronteiras exatas ainda sem ensaio próprio. Migrações/revalidações/B3/adoção/TEAM/Guardian/entrega e49critérios integrais abertos; raiz834/instalação.30 intactas,sem publicação.

## R43F9B2D4D1 — revisão correta de chave inválida

Privado910/D4D1a reutiliza KeyFile::parse no runtime e na preparação, acrescentando inspeção só de leitura. Chave truncada/inválida é keyInvalid/manualReconciliationRequired, com geração indisponível; geração incerta mantém essa barreira. apply recusa ambos antes de qualquer escrita, mesmo com hash atual aprovado. Preparação válida retoma a mesma chave; ready com custódia alterada continua recusado. Sem formatos/grants/ledger novos, reparação ou regeneração.

Passaram11dirigidos,2844Rust/21ignorados,build531 e nativo192. O CLI real compara o leitor anterior (keyPresent/manualfalse para chave truncada), prova8variantes inválidas com recusa aprovada sem escrita, retoma válida, chave ausente incerta e ready adulterado recusado. Source/payload conservados;0modelos/quota/CLI de agentes/efeitos físicos; web1255/TSC/Vite reutilizada por bytes iguais. [Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/INDEPENDENT-RECOVERY-REVIEW-PARTIAL-CHECKPOINT-R43F9B2D4D1.json).

É pré-condição da recuperação explícita, que continua por implementar para jobs incertos/pre-manifest/staging anónimo: retenção durável e atos separados de abandono/futura autoridade, sem renovar saldo nem confiar em dados retidos. Migrações/revalidações/B3/adoção/TEAM/Guardian/entrega e49critérios integrais continuam abertos. Raiz834/instalação.30 intactas,sem publicação.

## R43F9B2D4C — verificar destino dos resultados antes do modelo

Privado910/C2 acrescenta preflight só de leitura no writer comum, antes de metadata/inferência e novamente antes de criar/reservar a tentativa com o corpo real. Reutiliza Authority/results/protocol_directory: recusa raiz/decisão incompatível sem reparar ou criar namespaces, conserva bytes/orçamento, admite ausência e rondas anteriores válidas. Não prevê falhas futuras; falha depois de dispatch mantém consumo/reconciliação e proíbe replay automático.

18 testes dirigidos (1ignorado),2841Rust (21ignorados),build531 e nativo191 passaram. O nativo conserva o percurso operacional/Stop, introduz depois um catálogo inválido privado e prova recusa funcional sem novo envio/tentativa/alteração de ledger, reinício sem replay e backup portátil. Providers sintéticos,0quota/modelos reais; web1255/TSC/Vite reutilizada por bytes iguais. C1 reteve2erros de preparação dos novos testes LMStudio; C2 só corrige esses helpers/configuração.189 falhou antes de boot por omissão da opção de build custom-protocol;190 usa a composição correta com fontesC2 idênticas, perfil fresco e espera passiva pela página incorporada.190 provou a recusa mas falhou ao esperar validação de conclusão com recibos globais corrompidos;191 exige a recusa dessa conclusão e preservação exata dos bytes, sem alterar produção. [Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/MODEL-RESULT-SINK-PREFLIGHT-PARTIAL-CHECKPOINT-R43F9B2D4C.json).

Próximo: recuperação explícita de geração incerta/seed inválido/pre-manifest/staging; depois migrações/revalidações/B3/adoção antes TEAM/Guardian. Raiz834/instalação.30 intactas;49critérios integrais abertos.

## R43F9B2D4B — admissão independente privada

909/B5 reutiliza a mesma chave/HMAC/Activation/Storage com manifest/admission4 sob Header2, recibo3 separado, adição de diretório tipada e custódia local. Fonte sem Authority conservada; IDs antigos negados, consentimento novo independente, sem renovar saldo.2837Rust/21ignorados/7novos/build530/native188 com CLI/HTTP sintéticos: novoSID/objetivo/decisão/Stop/reinício/ACK/backup4. [Provas, falhas retidas e limites](../.build/pet-goal-execution-next-r1/INDEPENDENT-ADMISSION-PARTIAL-CHECKPOINT-R43F9B2D4B.json). Geração incerta/staging/migrações/revalidações/B3/adoção/aceitação integral e49critérios abertos; raiz834/instalação.30 intactas.

## R43F9B2D4A — preparação independente privada

907/A5 reutiliza o CLI/Authority/HMAC para preparar uma nova chave apenas sob revisão de ausência real original; fonte/payload intactos e IDs antigos negados, sem grants ou publicação operacional. Intenção persistida antes do RNG impede regenerar uma chave desaparecida.2830Rust/21ignorados/8novos/build528/native185,0inferência/quota. [Provas, regressão negativa e limites](../.build/pet-goal-execution-next-r1/INDEPENDENT-AUTHORITY-PREPARATION-PARTIAL-CHECKPOINT-R43F9B2D4A.json). D4B/admissão operacional/proveniência/recuperação incerta continuam pendentes, tal como os49critérios integrais; raiz834/instalação.30 intactas.

## R43F9B2D3 — continuidade privada validada

904/D3A5 reutiliza arquivo2/Header3 e C3/D1/D2/Storage para backup do admitido3 e novo restauro/admissão com depth3; profundidade limitada9, histórico/consumo anteriores preservados.2822Rust/21ignorados/11novos/build525/native184, providers sintéticos. Corrigida a repetição de hashes por entrada na cópia de originais, mantendo revisões/Claim e hashes integrais antes dos efeitos. [Provas, falhas retidas e limites](../.build/pet-goal-execution-next-r1/RECURSIVE-BACKUP-PARTIAL-CHECKPOINT-R43F9B2D3.json). NoAuthority/revalidações/B3/adoção e49critérios integrais abertos; raiz834/instalação.30 intactas,sem publicação/modelos/quota reais.

## R43F9B2D2 — admissão Header3 privada validada

902/D2A4 reutiliza a ativação e Storage existentes, com manifest/admission3, job próprio, mesmos originais C3 e índice2 antes de Chat.2811Rust/21ignorados/7novos/build523/native182: novo SID/objetivo/decisão/Stop/reinício/ACK e independência do arquivo externo, providers sintéticos, sem renovar grants históricos. [Provas, falhas retidas e limites](../.build/pet-goal-execution-next-r1/EPOCH-ADMISSION-PARTIAL-CHECKPOINT-R43F9B2D2.json). Backup do admitido3 ainda bloqueado e segue antes de adoção/entrega; restantes dependências e49critérios integrais abertos. Raiz834/instalação.30 intactas,0quota/modelos reais,sem publicação.

## Contexto anterior a D3 — planeamento conservado (superado pelo checkpoint acima)

O backup do perfil admitido3 recusa por agora. A próxima unidade tem de preservar os roots completos, metadados, trabalho novo e autoridade histórica, com leitura portátil e percurso de restauro coerente. Avaliar a extensão explícita do contentor2/Headerv3 com profundidade validada, sem criar uma versão de header por cada cópia nem flexibilizar leitores antigos. [Contrato, limites e provas necessárias](../.build/pet-goal-execution-next-r1/HEADER3-BACKUP-CONTINUITY-CONTRACT-R43F9B2D3.md). Não é suporte já disponível ou entrega.

## R43F9B2D1 — configuração Header3 revista, ainda sem admissão operacional

901/D1A1 acrescenta revisão/recibo2 nativo de configuração e índice2 que une referências anteriores seladas às atuais; não ativa definições nem trabalho. Corpo1 e recibo1 mantêm bytes; leitores Admission2 exigem índice1.2804Rust/21ignorados/9novos/build522/native180 com providers sintéticos, incluindo hash do índice composto independentemente e ACK sem escrita. [Provas e limites](../.build/pet-goal-execution-next-r1/EPOCH-CONFIGURATION-PARTIAL-CHECKPOINT-R43F9B2D1.json). D2/admissão3, restantes dependências e49critérios integrais abertos; raiz834/instalação.30 intactas,0quota/modelos reais,sem publicação.


## R43F9B2C3 — histórico de autoridade de várias épocas, 26/10/2026

SDK901/C3A6 conserva, na mesma Authority/chave/locks/ledger, a cadeia limitada de índices HMAC anteriores e a união de decisões/objetivos históricos negados. Corpo1/protocolo2 mantém os bytes compatíveis; corpo2/domínio2/protocolo3 liga o antepassado exato e recusa leitores antigos/downgrade/cadeias adulteradas. Grants, consumo e incerteza mantêm os seus bytes; não se renova saldo ou prova de morte do dono. O comando nativo Header3 retém os dois roots originais completos antes de alterar os três metadados previstos, sob o mesmo Claim.

Passaram 2 795 Rust/21 ignorados,19 casos novos desdeC2, build/522 inputs e nativo179 com CLI/Tauri reais e providers sintéticos. Provas de duas/três épocas, recusa de IDs antigos/atuais, consentimento novo independente, consumo incerto com dono original ainda vivo, cortes de publicação, instância em cache, originais, ACK/replay e adulteração. A4/178 também passou e fica conservado. A5/A6 acrescentam recuperação revista de temporários: só prefixos exatos do antepassado autenticado são removidos por Stamp depois de guardar o antepassado inteiro; identidade da transação mantém-se e consentimento inclui a fase. Em179 o resíduo de prefixo é sintético, não uma morte nativa a meio dessa escrita. Web1255/TSC/Vite reutilizada por fontes não Rust idênticas. [Contrato](../.build/pet-goal-execution-next-r1/MULTI-EPOCH-AUTHORITY-CONTRACT-R43F9B2C3.md) e [checkpoint/provas/limites](../.build/pet-goal-execution-next-r1/MULTI-EPOCH-AUTHORITY-PARTIAL-CHECKPOINT-R43F9B2C3.json).

**Header3 continua inativo:** falta inicialização/configuração e admissão CID/SID/tarefa sob revisão nova. Temporários desconhecidos/fora do prefixo ou staging anterior ao manifest não são reparados automaticamente. Seguem NoAuthority/revalidações voz/mobile/autostart/B3/adoção e aceitação integral. Raiz834/instalação.30 intactas;49critérios integrais abertos,0quota/modelos reais, sem commits/publicação.

## R43F9B2D — admissão por fases

D1/revisão nativa e união de referências estão comprovadas acima. D2/seleção, inicialização e admissão operacional3 continuam pendentes: conservar o job/originais C3 e o índice anterior selado, reaproveitando Projection/History/Activation/Storage/Chat. Active3 permanece bloqueado. [Contrato e aceitação](../.build/pet-goal-execution-next-r1/HEADER3-ADMISSION-CONTRACT-R43F9B2D.md).

## R43F9B2C2 — restauro dos dados da época seguinte, 26/10/2026

SDK898/C2A4 restaura arquivos2 completos por revisão/cópia/retoma comuns num Header3 inativo. Preserva ambos os roots antigos integrais sob payload, incluindo trabalho posterior, consumo/efeitos incertos, chaves/índices/originais e a admissão anterior como dados. Arquivos1 mantêm Header2; open()1 continua separado do leitor explícito de restauro1/2. Active/bootstrap e autoridade/configuração/ativação2 recusam3. O backend distingue dataRestorationAvailable de runtimeActivationAvailable e exige revisão; o campo antigo restorationAvailable continua false por compatibilidade de ativação executável.

Passaram 2 776 Rust/21 ignorados, seis casos novos e build/519 inputs. Nativo177 usa CLI/Tauri reais e providers sintéticos: recibo1 criado pelo binário898 anterior é reconhecido com bytes/hash/par inalterados; uma cópia2 própria foi interrompida/recolhida e retomada sob nova revisão no mesmo par; todos os bytes ficaram preservados, ACK não repete cópia e os leitores/ativadores anteriores recusam3. Zero inferência/efeitos adicionais para restaurar; não é aceitação humana/instalada ou de providers reais. Web1255/TSC/Vite reutilizada por igualdade das fontes não Rust. [Checkpoint, falhas e âmbito](../.build/pet-goal-execution-next-r1/NEXT-EPOCH-DATA-RESTORE-PARTIAL-CHECKPOINT-R43F9B2C2.json).

Na retoma, ficheiros já completos e diretórios também são sincronizados antes de completion: digest igual não demonstra que o writer interrompido terminou o fsync. A3 expôs quatro recusas SourceChanged por usar confirmação de ficheiro num diretório; A4 corrige com binding de Node/Dir e passa. O ensaio de kill/recolha não testa alimentação/disco físicos. **Header3 permanece inativo:** seguem reconciliação de autoridade/consumo de todas as épocas e inicialização/admissão próprias, depois NoAuthority/revalidações/B3/adoção. Raiz834/instalação.30 preservadas,49critérios integrais abertos, sem commits/publicação.

## R43F9B2C1 — backup portátil do perfil restaurado, 26/10/2026

SDK898/C1A1 permite rever/criar/inspecionar backup nativo Root2: formato2 conserva ambos os roots completos, incluindo trabalho posterior ao restauro, admissão/planos/índices/selos e originais retidos. Root1 mantém o formato1 de payloads. Reutiliza Claim/Active, inventário/cópia e limites existentes; os metadados não entram na configuração operacional. A inspeção verifica vínculos e HMAC históricos e não concede execução. Os limites de ficheiros/nós/caminhos/profundidade/bytes contam payload e recuperação em conjunto; excesso recusa, sem omissões silenciosas.

Passaram 2 770 Rust/21 ignorados, seis casos novos, build/519 inputs e nativo174: perfil restaurado com novo objetivo/CLI/Stop/reinício, aprovação exata de backup, todos os bytes dos roots conservados, inspeção com origem indisponível, adulteração recusada sem reparação e leitor897/restauro1 recusam2 sem publicar payload. IPC/CLI são reais; HTTP/CLI de inferência são sintéticos e MPRIS privado, sem quota/modelos reais. Web1255/TSC/Vite reutilizada com fontes não Rust idênticas. [Checkpoint e provas](../.build/pet-goal-execution-next-r1/PORTABLE-RESTORED-BACKUP-PARTIAL-CHECKPOINT-R43F9B2C.json).

A unidade fecha a recusa de backup Root2 da revisão897 para os dados dentro dos limites. **Restauro/ativação da época seguinte continuam pendentes:** o restauro1 recusa2 antes de escrever payloads. Não se transfere autorização pela admissão arquivada. NoAuthority, revalidações voz/mobile/autostart, B3/adoção e aceitação integral continuam abertos; raiz834/instalação.30 preservadas,49critérios integrais abertos, sem commits/publicação.

## R43F9B2B2 — primeira ativação operacional privada, 26/10/2026

SDK897/B2B2C1 acrescenta plano nativo tipado ligado ao recibo B2A e configuração efetiva, cópia integral dos originais antes de transformar Settings/pets/fontes/Flow e Admission2 selada. Reutiliza Claim/Active/Storage/Authority e os escritores existentes. O índice CID/SID/tarefa é agora capturado dos dados reais e carregado antes de Chat; o índice HMAC B1 continua a negar objetivos/decisões antigos. Modelo/endpoint permanecem os revistos e a rota lógica vem da fábrica. Autorização de trabalho novo continua própria; a admissão não renova grants históricos.

Passaram 2 764 Rust/21ignorados,10casos novos,build/518inputs e nativo173 com IPC Tauri real em perfil isolado. HTTP/CLI são sintéticos e MPRIS privado: novo SID, objetivo com decisão exata, Stop durante inferência, reinício e ACK sem repetir inicialização; IDs antigos recusados e fonte/arquivo/grant preservados. Não é aceitação de providers reais/humana/instalada. Web1255/TSC/Vite reutilizada por igualdade das fontes não Rust. [Checkpoint e provas exatas](../.build/pet-goal-execution-next-r1/RESTORED-ACTIVATION-PARTIAL-CHECKPOINT-R43F9B2B2.json). A prova anterior893 limitava-se às guardas/fixtures e regressão Root1.

**Limites abertos:** primeira via exige Authority original genuíno já quarentenado. Backup Root2 recusa até incluir metadados/originais; épocas seguintes, NoAuthority operacional, staging anterior ao manifest, migrações e B3/adoção permanecem por concluir. Voz/mobile/autostart ficam suspensos e bloqueados até revalidação própria. Raiz834 e instalação.30 intactas; todos os 49 critérios integrais abertos, sem commits/publicação. Segue backup/restauro completo e restantes dependências R43 antes de TEAM/Guardian.

## R43F9B2A — revisão tipada da configuração, 26/10/2026

SDK890/F9B2A5 acrescenta `--pets-profile-restore-configuration WORKSPACE ID [--approve FINGERPRINT]` sob o mesmo Claim offline. Settings/pets são lidos de forma limitada/privada/nofollow, sem salvage, criação ou reparação de preferências. Campos desconhecidos e tipos inválidos são recusados. A resolução pura de pets é a mesma do leitor normal; diferenças/defaults/upgrades são apresentados, com modelos/destinos, execução CLI, voz, telefone, integrações e permissões por pet separados na revisão.

A revisão liga arquivo, destino/par/estado, hash histórico, bytes e proposta. O recibo imutável `.configuration-review.json` significa **`configurationInspectionOnly`**: não aprova ativação, não renova grants e não admite o perfil. Perfis sem Authority podem rever estes dados sem criar chave/autoridade. Revalidação antes/depois da publicação recusa alterações e perda do dono; ACK perdido reconhece o mesmo recibo sem mudar payloads. Header2 continua bloqueado.

Passaram12testes novos incluídos nos2737Rust/21ignorados/build511 e nativo168 real/CLI/IPC/HMAC,0inferência/quota. O ensaio prova grupos/modelo/endpoint exatos, revisão sem escrita, hash errado, recibo único/ACK, payloads intactos e recusa pelo leitor anterior. Web1255/TSC/Vite reutilizada com bytes não Rust idênticos. B2B (seleção de ativações, concessões/sessões e Admission2), B3 (continuação histórica), épocas/semAuthority operacional e adoção/entrega permanecem abertos. [Checkpoint e limites](../.build/pet-goal-execution-next-r1/RESTORED-CONFIGURATION-PARTIAL-CHECKPOINT-R43F9B2A.json). Sem alterações à raiz834/instalação.30, commits, anfitrião ou aceitação humana;49critérios integrais abertos.



## R43F9B1C — autoridade histórica em quarentena, 26/10/2026

SDK888/F9B1C acrescenta revisão nativa offline `--pets-profile-restore-authority WORKSPACE ID [--approve FINGERPRINT]` sobre o restauro inativo. Reutiliza Authority/HMAC/lock/grants: índice autenticado de origem/negação e protocolo2 preservam os bytes antigos e bloqueiam decisões **e objetivos** históricos em load/publish. Leitura histórica verifica assinaturas sem criar permit, saldo, recuperação ou prova de morte do dono. Objetivo novo independente usa consentimento, lease e orçamento próprios no mesmo Authority. O índice é metadado autenticado, não outro grant ou ledger.

Passaram12testes novos dirigidos,2725Rust/21ignorados e build509inputs. Web1255/TSC/Vite reutilizada com fontes não Rust byte idênticas. Nativo165 usa consentimento/claim HMAC reais, CLI/IPC e perfis privados: revisão/hash errado, apenas protocolo+índice alterados, assinatura histórica válida, originais intactos, ACK idempotente, tamper recusado e leitores operacionais anterior/atual recusam o perfil. Zero inferência/quota. O consumo incerto e a admissão/claim de trabalho novo têm prova Rust; não foram executados num runtime restaurado ativo.

**Header2 continua inativo**, sem `.admitted.json` ou configuração sensível ativada. F9B2 (admissão/configuração) e F9B3 (continuação histórica com consentimento/saldo próprios) permanecem abertos. Arquivos sem Authority, múltiplas épocas e staging interrompido antes da publicação do índice precisam de percurso próprio; não há reparação automática. Não se prova ausência de dono nem estado posterior ao snapshot. [Checkpoint, falhas retidas e limites](../.build/pet-goal-execution-next-r1/RESTORED-AUTHORITY-PARTIAL-CHECKPOINT-R43F9B1.json). Raiz834/instalação.30 intactas;49critérios integrais abertos, sem commits/publicação.



## R43F9A — publicação revista dos dados de restauro, 26/10/2026

SDK885/F9A implementa `--pets-profile-restore WORKSPACE ID` e `--pets-profile-restore-resume WORKSPACE ID`, com revisão antes da escrita e `--approve FINGERPRINT`. O arquivo atual tem protocolo/manifesto/ready/origem/inventário verificados e descritores físicos retidos. A revisão liga arquivo, origem/conteúdo, paths/layout, inodes dos pais, estado parcial e modo. Destinos fresh têm de ser roots privados vazios; dados/perfis operacionais ou desconhecidos são recusados. Os mesmos Claim nativo, locks, Dir/NoFollow, inventário e staging são reutilizados.

Cada root recebe atomicamente cabeçalho2, intenção vinculada à origem e proveniência portátil completa antes dos payloads. A cópia preserva os dois payloads byte a byte. Retoma exige revisão explícita do mesmo arquivo e valida todo o subset/prefixo antes de acrescentar bytes; não trunca/substitui. Se existir alguma completion, dados desaparecidos ou alterados não são reconstruídos. Recibo perdido regressa ao mesmo par/recibo sem repetir a cópia. Originais e trabalho parcial são conservados.

O resultado é **`dataRestoredInactive`**: não se cria `.admitted.json`, não se ativa configuração/serviços/hooks/modelos e não se recupera autoridade executável. O bootstrap atual e os leitores operacionais anteriores recusam estes roots. Esta é a primeira parte da transação de restauro; F9B tem de implementar admissão revista e reconciliação/quarentena histórica antes de permitir arranque e novos trabalhos. Não equivale a uma aplicação restaurada operacionalmente. [Checkpoint e limites](../.build/pet-goal-execution-next-r1/CURRENT-PROFILE-RESTORE-PARTIAL-CHECKPOINT-R43F9.json).

Passaram2713Rust/21ignorados/check/build506inputs. As fontes não Rust são byte idênticas às1255JS/TSC/Vite já validadas, cuja prova foi reutilizada. Nativo163 usa CLI/IPC reais em perfil privado: origem normalv1, rascunho/arquivo original, backup, revisão antes dos roots, hash errado recusado, par/origem preservados, recusa por leitores novo/anterior, recibo idempotente, interrupção/recolha do processo próprio e retoma do mesmo par; subset divergente recusado antes de mais cópia. Removeu-se apenas o ficheiro de corrupção que o próprio teste tinha inserido. Fonte/arquivo/instalação/anfitrião intactos; zero modelos/quota.

Ainda faltam a validação semântica de assinaturas/história/contabilidade/efeitos incertos e a admissão executável F9B. Os testes de dados conservam esses ficheiros como bytes opacos, sem provar autoridade. Seguem adoção das fontes exatas e restantes dependências. Raiz834 e instalação .30 preservadas;43+6 critérios integrais abertos. Sem novos commits/publicação.


## R43F8B2 — reconciliação de origens divergentes, 26/10/2026

SDK882/F8B2 acrescenta revisão humana de ambas as origens, política explícita, sourceId/fingerprint, revisão nativa e prazo5min ligados ao hash. `retainOnly` conserva o original; `mergeCompatible` junta apenas tuplos ausentes. Mantém os valores nativos atuais e os registos IndexedDB. Original completo privado/imutável sob o payload precede o catálogo3/CAS/recibo, com até8 artefactos incluindo pendentes e64MiB cada. Arquivos pendentes são visíveis e recuperáveis. O ACK normal continua estrito; substituir recibo após reconciliação relê transacionalmente todos os registos/counters/sourceId. ACK perdido não reimporta sobre edições posteriores. Nenhum envio, modelo ou grant é concedido.

Controlos comuns inline/flutuante permitem rever origens, escolher política e recuperar texto conservado para conversa existente do mesmo dono. Fonte/bytes/revisões/destino/prazo são revalidados. Catálogo3 lê1/2 sem mutação passiva; reconciliação válida migra e escritas posteriores conservam3. Leitor2 congelado e backend anterior real recusam3 sem destruir. Os mesmos private_registry, dono dos dados, retenção, fábrica e backup são reutilizados.

Passaram2700Rust/21ignorados e compilação503inputs. JS/TSC/Vite passaram1255 emB1;B2 só atualiza uma fixture Rust de versão futura3 para4. Produção Rust/JS/testesJS e binários B1/B2 são idênticos. Nativo162 correu nessa produção com IndexedDB e IPC reais, escolhas scriptadas: ambas as origens, fusão conservadora, RAM em conflito conservada, original arquivado, mesma origem alterada, ACK deliberadamente omitido/reinício sem replay, reconhecimento posterior sem nova fusão, cópia do arquivo pela UI, alteração após revisão recusada, leitor anterior e backup byte a byte das três origens. Zero modelos/quota/anfitrião. [Checkpoint e limites](../.build/pet-goal-execution-next-r1/DIVERGENT-DRAFT-RECONCILIATION-PARTIAL-CHECKPOINT-R43F8B.json).

Falhas iniciais preservadas: a fixture Rust antiga tratava3 como desconhecido; agora4 continua a ser recusado. Nativo161 procurava PetId numa mensagem que é substituída pelo erro de migração;162 observa o provider no aria-label real. Não se alterou navegação/IPC para contornar guardas. Mutação da origem durante IO nativo não tem prova temporal nativa; alteração após revisão e releitura transacional têm provas próprias. A ilha compacta usa scroll existente; não houve aceitação humana de legibilidade.

O backup inclui o payload e origens aceites/pendentes conservadas; não copia perfis WebKit nem mutações posteriores ainda não revistas. Restauro/admissão/autoridade histórica sem renovação, adoção e aceitação humana/instalada continuam abertos. Raiz834 e instalação .30 preservadas; os43+6 critérios integrais continuam abertos. Sem novos commits/publicação.


## R43F8A — recuperação explícita de rascunhos nativos, 26/10/2026

SDK874/F8A implementa listagem nativa limitada por dono/provider e revisão, classificação observada de destinos e revisão/cópia explícita para conversa existente do mesmo pet/provider. O backend liga fonte/revisões/bytes UTF16/destino/prazo de5min ao hash, recompõe a revisão e aplica CAS; recusa replay, seleção alterada, falta de confirmação e destino não vazio diferente. Conserva o registo original, usa os mesmos controlos inline/flutuante e publica apenas hints de identidade. Não envia mensagens, invoca motores, concede dados/permissões ou assume controlo de CLI. Reutiliza catálogo2, private_registry, dono, retenção e backup existentes.

Passaram2684Rust/21ignorados,check499inputs,1240JS/TSC/Vite/build. Nativo160 usa a entrada pet-cards e IPC reais num perfil privado: consulta passiva, original órfão, confirmação pelos controlos, ação antiga após trocar de pet recusada, texto hostil como texto, original conservado, sincronização inline/flutuante, replay/conflito recusados, reinício, leitor anterior869 do mesmo formato e backup byte a byte. Escolhas automatizadas, zero modelos/quota/anfitrião; não é aceitação humana/instalada. [Checkpoint, fontes e limites](../.build/pet-goal-execution-next-r1/ORPHAN-DRAFT-RECOVERY-PARTIAL-CHECKPOINT-R43F8.json).

A reconciliação entre IndexedDB e nativo divergentes continua por implementar: a migração normal recusa-a e conserva os dados. Próxima unidade F8B deve preservar a origem perdedora em artefacto imutável incluído no backup e rever as fontes exatas antes do ACK. Depois restauro/adoção e restantes dependências. Raiz834 e instalação .30 preservadas; todos os43+6 critérios integrais continuam abertos. Sem novos commits/publicação.


## R43F7A — rascunhos inline e sem conversa, 26/10/2026

SDK869/F7A: 2674 testes Rust passaram, 21 ignorados, check/compilação nativa e 1230 JS/TSC/Vite. O inline usa agora o mesmo repositório nativo/CAS e controlos de estado/conflito/retry da janela flutuante. A fábrica resolve o dono pelo canal de chat completo. O texto anterior à identificação do CID fica num slot `unbound` por pet/provider, separado de `SessionTarget`, memória, envio e autoridade. Cópia/abertura da conversa exige escolha explícita e conserva a origem. Trocas de pet/CID/modelo e decisões antigas não redirecionam texto; sucesso de envio limpa apenas a captura original ainda inalterada, enquanto falha ou nova edição conservam o rascunho. [Checkpoint e limites](../.build/pet-goal-execution-next-r1/INLINE-DRAFTS-PARTIAL-CHECKPOINT-R43F7.json).

O catálogo nativo2 lê1 sem o alterar; uma mutação válida migra atomicamente. O parser antigo congelado e o backend privado anterior recusam2 sem sobrescrever. Importações concorrentes do mesmo original esperam até1s apenas por contenção anterior à publicação, com dono revalidado e recibo idempotente; incerteza/revogação não se repetem. Os limites UTF16/u64 mantêm-se.

Nativo158, nos cartões autorizados `pet-cards`, prova migração, dados separados Ollama/Codex/Gravity, revisão/edição/cópia explícita de origem sem conversa, ação antiga recusada após mudar de pet, registo comum inline/flutuante, reinício, leitor anterior e backup byte a byte. Zero modelos/quota. A origem `unbound` foi criada antes da primeira consulta Gravity; não se provou digitação nativa inicial antes da resposta de CID, nem se alterou o `invoke` protegido. As corridas de digitação/hidratação foram testadas separadamente em Node.155 respeitou a recusa de navegação legacy;156/157 eram pressupostos do harness sobre roster/invoke e foram conservados.

Compatibilidade não nativa: os rascunhos associados mantêm IndexedDB v1; texto sem CID é explicitamente indisponível/RAM até cópia deliberada para conversa suportada, sem escrita de origem nova no formato antigo. Faltam recuperação funcional de órfãos/importação divergente, restauro/adoção e aceitação humana/instalada. A raiz834, instalação .30 e os43 IDs originais+6 Guardian continuam preservados e abertos integralmente. Sem novos commits/publicação.


## R43F7A em implementação — produtor inline (26/10/2026)

O produtor inline passa a partilhar o repositório nativo/CAS e a identidade completa da conversa. Texto anterior à resolução de CID fica num slot de rascunho `unbound`, separado de sessões/autorização, e só uma escolha explícita o copia para a conversa identificada; a origem conserva-se. Formato nativo2 lê1, migra em mutação válida e deve ser recusado pelo leitor anterior. Controlos funcionais de estado/retry/conflito são comuns às duas superfícies. [Contrato da unidade](../.build/pet-goal-execution-next-r1/INLINE-DRAFTS-CONTRACT-R43F7A.md). Provas em execução futura; inline/reconciliação/restauro continuam abertos até validação própria.


## R43F6D — rascunhos nativos privados, 26/10/2026

SDK865/F6D: 2669 testes Rust passaram, 21 ignorados, compilação nativa e1219 JS/TSC/Vite (fontes web idênticas aA). A janela de conversa usa agora `SessionDrafts` com armazenamento nativo, dono/CAS existentes, revisões u64 e UTF16 exatos. A migração lê todos os registos conhecidos pela API IndexedDB, cerca v1 com versão2, conserva a origem após ACK e não repete importação sobre edições posteriores. Novo rascunho Mochi retém o CID existente antes de publicar, sem mensagem/modelo/consentimento; o arquivo conserva referências após reinício. O teste de capacidade máxima mantém os limites lógicos anteriores. [Checkpoint e limites](../.build/pet-goal-execution-next-r1/NATIVE-DRAFTS-PARTIAL-CHECKPOINT-R43F6.json).

Nativo154 usa IPC e janela reais em perfil privado: origem858/F5D com três rascunhos sintéticos (incluindo Gravity/Copilot inativos e tombstone), importação/edição/conflito/ação explícita/reinício e uma nova conversa vazia retida só pelo rascunho; o catálogo nativo entra byte a byte no backup dos payloads. Zero chamadas de modelo/quota.154 foi executado na produçãoC;D altera só uma fixture de teste e o binário foi comparado byte a byte, preservando o alcance exato da prova.152 é falha do setup de origem;153 revelou CID vazio perdido, corrigido porC; ambos preservados. Não se provou callback de escritor v1 vivo: o backend de origem foi recolhido antes da atualização, e a nova abertura v1 é recusada.

Ainda faltam o produtor inline, resolução funcional de importação divergente e recuperação dos destinos legados órfãos; estes dados não são apagados nem redirecionados. Faltam restauro/adoção e aceitação humana/instalada. A raiz834 e instalação .30 continuam preservadas;43 IDs originais+6 Guardian permanecem abertos integralmente. O objetivo completo continua ativo, sem novos commits ou publicação.


## Próxima dependência em execução — R43F6A (26/10/2026)

Antes do restauro completo: persistência nativa dos rascunhos e migração transacional dos registos conhecidos, incluindo conversas inativas, sem copiar perfis. Reutiliza `SessionDrafts` e `private_registry`, com CAS e import receipt. Primeira superfície: janela de conversa existente; a integração inline e a reconciliação humana de origens divergentes precisam de prova própria. [Contrato privado](../.build/pet-goal-execution-next-r1/NATIVE-DRAFTS-CONTRACT-R43F6A.md). Ainda em implementação; nenhum dos43 IDs originais ou6 Guardian é retirado ou declarado integralmente fechado. Seguem-se cobertura de todos os produtores e restauro/admissão coerentes, depois restantes dependências do roadmap.


**R43F5D validado em privado — 26/10:** SDK858 acrescenta revisão/confirmação/inspeção de backup dos dois payloads nativos admitidos e corrige a conclusão sob locks transitórios, conservando prova/aprovação e prioridade de Stop. Passaram2655 Rust/21 ignorados,23 dirigidos/1 ignorado,check-tests/build e composição nativa151;149 cobre as recusas e integridade do arquivo. São perfis, motores e escolhas sintéticos. **Rascunhos IndexedDB/inline ainda ficam fora desta cobertura**: segue persistência nativa/migração dos rascunhos, depois restauro/adoção. Raiz834 e instalação .30 preservadas; os49 critérios integrais continuam abertos. [Provas e limites](PETS-VALIDATION.md#backup-payload-conclusao-r43f5) · [Uso e cobertura](PETS-DATA-BACKUP.md).

**R43F3/F4 validado em privado — 25/10:** a composição854/F4 conserva a memória selecionada ao continuar objetivos com novo consentimento e orçamento adicional explícitos. Tauri146/147 provam duas rondas, aprovações separadas, revogação durante IO e preservação por reinício/leitor antigo, com motores sintéticos. A importação legada passa a recusar proveniência nova escondida em formato antigo e catálogos de runtime que seriam omitidos. Passaram2641 Rust/21 ignorados,check/build e a composição nativa147. O timeout145 permanece sem causa exata confirmada; não é uma regressão dada como corrigida. [Provas e limites](PETS-VALIDATION.md#retoma-memoria-importacao-r43f4). Seguem backup/restauro coerente do perfil atual e adoção; raiz834 e instalação .30 preservadas. Sem novos commits ou aceitação integral dos49 requisitos.

**R43F2E2 validado em privado — 25/10:** SDK854 liga a seleção/revisão de memória aos chats manuais, vincula anexos/contexto ao token e apresenta destinos API conhecidos sem prometer contenção do fornecedor. Passaram2638 Rust/21 ignorados,1212 JS/TSC/Vite/check/build e Tauri143/144. A janela de conversa passou nos dois motores locais, três CLI e Notion/Codex; o inline tem provas de controller/unidade, com aceitação nativa pendente após falha de ativação em Xvfb. São escolhas automatizadas e providers sintéticos. [Alcance e limites](PETS-VALIDATION.md#memoria-chat-contexto-r43f2e2). Raiz834/instalação .30 preservadas; seguem retoma com memória e migração/adoção.

**Histórico R43F2E1 — 25/10:** SDK850 acrescenta catálogo/seleção por revisão e revisão completa com consentimento próprio nos controlos reais de objetivos Ollama/LMStudio, além da projeção de memória por caller. Reutiliza CRUD/CAS, admissão, concessão e writer existentes. Passaram2634 Rust/21 ignorados (487 inputs nativos iguais entre E1a/E1b),1200 JS/TSC/Vite/check/build e Tauri139. Os cliques/consentimentos foram automatizados e os motores sintéticos: não é aceitação humana ou instalada. Seguem revisão manual de chat, retoma com memória e adoção/migração/entrega; os49 requisitos continuam abertos nos critérios integrais. [Provas e limites](PETS-VALIDATION.md#memoria-controlos-objetivos-r43f2e1). Raiz834 e instalação .30 preservadas.

**Histórico R43F2D — 25/10:** SDK846 prova os envios de memória de Anthropic/OpenAI/Google através da implementação nativa comum, com HTTP loopback sintético, dados separados da pergunta e recusa de destino/modelo/credencial/pergunta alterados. Foi retirado o opt-in de fallback automático Anthropic; APIs e percursos locais usam o destino exato sem seguir redirecionamentos. Passaram2632 Rust/21 ignorados, check/build e Tauri136:13 pedidos HTTP e17 prompts CLI sintéticos, incluindo memória/decisões/recolha/retoma dos três CLI e serviço separado, objetivos locais e dois redirecionamentos recusados sem pedidos ao destino alternativo. Não certifica contas reais, routing interno do fornecedor, UI humana ou instalação. Raiz834 e instalação .30 preservadas. Seguem interface humana/callers, retoma de objetivos com memória e adoção/migração/entrega; R43 e os49 requisitos continuam abertos.

**Revisão posterior de 25/10 — Guardian e execução integral:** os 43 requisitos originais conservam os seus critérios; acrescentam-se GUA-01..06 no [contrato System Guardian](SYSTEM-GUARDIAN-ARCHITECTURE.md) e na [auditoria rastreável](PETS-REQUIREMENTS-AUDIT-2026-10-25.md). Pet, runtime comum, motor de inferência e papel funcional são entidades distintas. `ai_ollama` continua coordenador preferencial configurável, sem ser router obrigatório ou concedente de permissões. Guardian é especialista da mesma fábrica, sem autoridade própria adicional. Não há um pet Guardian operacional por esta atualização documental.

**Estado verificado nesta continuação:** raiz integrada834 e instalação .30 preservadas. SDK843/R43F2A passou 21 testes dirigidos de contexto, 2617 Rust/21 ignorados e cargo check em 483 inputs nativos imutáveis. O novo contrato de revisão mostra dados completos e destino nativo, separa consentimento de objetivo e verifica âmbito/credencial/revogação. É fundação privada: ainda falta a ligação à admissão, concessão assinada e corpo real anterior ao hash/reserva. Não foi acrescentada prova Tauri ou build instalada por este incremento. A barreira F1 permanece fechada; não declarar R43 concluído.

**Restrições vigentes:** um único agente, sem novos commits/pushes/merges de qualquer pasta; a publicação anterior do dossier foi uma exceção histórica já executada. Sem alteração da instalação/autostart/serviços, Wi-Fi/VPN/firewall/dispositivos ou dados pessoais. Sem quota real Copilot/GitHub, segredos em modelos/logs/Git ou bases de outros projetos. Alterações persistentes do anfitrião seguem sysworkspace e autorização específica; o desenvolvimento atual fica no projeto. Estados/estimativas antigos são históricos e não medem conclusão.

**Atualização de estado R43F1A — 25/10/2026:** no SDK privado841 passou a preparação durável de memória selecionada para objetivos: referências exatas por pet/conversa de origem, contexto versão2 e objetivo versão11 apenas quando há seleção, sem copiar os dados brutos para o diálogo inicial. Passaram 2612 testes Rust/21 ignorados, check/build e o percurso Tauri privado130, incluindo reinício, revogação e recusa do leitor840 sem alterar os ficheiros. Não houve chamadas a modelos ou quota. A admissão autónoma com memória permanece deliberadamente recusada até R43F2 ligar revisão humana dos dados/destino, consentimento próprio do objetivo e mensagem anterior ao hash/reserva canónicos. A raiz aceite permanece834; a app instalada permanece .30. R43 e os 43 requisitos continuam abertos.

Revisão: **25/10/2026**. Este plano executa a
[arquitetura normativa](PETS-ARCHITECTURE.md). O
[estado de implementação](PETS-IMPLEMENTATION-STATUS.md) identifica as provas.
Os IDs abaixo são estáveis; fechar uma linha exige a sua aceitação, não apenas
código ou um número de testes.

Correção em curso R43F6C: os rascunhos Mochi novos retêm o CID existente pelo mecanismo de `Chat`, depois de validar fábrica/destino/CAS e antes de publicar. O arquivo posterior consulta referências de rascunhos sem obter locks de escrita; desconhecido conserva. Não cria conversa, mensagem ou consentimento. Rascunhos legados órfãos permanecem dados e exigem recuperação explícita ainda pendente. O limite físico é64MiB para representar os limites lógicos anteriores, sem aumentar estes últimos. Provas finais em execução.

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

## 6. Extensão System Guardian — seis requisitos adicionais

Os 43 IDs originais mantêm todos os critérios. Estes seis IDs são adicionais; a implementação completa e as recusas estão no [contrato Guardian](SYSTEM-GUARDIAN-ARCHITECTURE.md). Estado atual: desenho normativo, sem operações Guardian implementadas/entregues. A próxima unidade continua R43.

| ID | Trabalho | Dependência e aceitação |
| --- | --- | --- |
| GUA-01 | Inventário limitado, evidência e classificação de gestão | PET-01/04/05, EXE-01/02, OBS-01, SYS-01. Componentes/versões/serviços/processos/exposição/recursos/logs selecionados e estado desejado, com origem/data/cobertura e distinção gerido/observado/externo/privado/dinâmico; percurso determinístico sem LLM e sem varrimento pessoal indiscriminado |
| GUA-02 | Findings de segurança e vulnerabilidades | GUA-01, CTX-05, EXE-02/03, DATA-01. Componente/versão, evidência/origem/data, severidade/confiança/estado/recomendação; distinguir confirmação/candidata/suspeita, conferir advisories da distribuição e pacote; não inventar CVE, inferir vulnerabilidade só por upstream ou executar exploração/scans intrusivos |
| GUA-03 | Saúde/desempenho com baseline e resultado | GUA-01, OBS-01/02, SYS-03. CPU/memória/IO/GPU suportada/arranque/falhas/conflitos, hipótese/métrica/impacto/custos e validação; fixtures de consumo, dock Thunderbolt e VPN/Tailscale/firewall sem mutações reais, tuning genérico, limpeza destrutiva ou desativação de proteções |
| GUA-04 | Adaptador governado ao sysworkspace | GUA-01, EXE-01/02/03, DATA-01, REL-01. Contratos reais de plano/validação/backup/aplicação/recuperação, estado desejado efetivamente gerido e exclusões Coucou/Mochi/projetos preservados; integração de excluído é proposta separada, sem comandos ou fonte de verdade inventados |
| GUA-05 | Remediação autorizada completa | GUA-01/04, RUN, EXE, SYS-01, REL-01/02. Finding→proposta/diff→impacto/pré-condições→autorização→execução limitada→verificação→resultado; receitas específicas sob grants explícitos e aprovação exata para sensíveis; sem shell/root geral, autoaprovação ou replay; rollback só suportado/provado, compensação/irreversibilidade explícitas |
| GUA-06 | Proatividade e cooperação sem runtime paralelo | GUA-01/05, AUTO, TEAM, OBS. Gatilhos leves/debounce/deduplicação/limites no scheduler, orçamento e notificações comuns; LLM só quando útil; pedidos diretos/coordenador opcional, delegação autorizada sem ampliar acesso, ausência de motor/coordenador e conteúdo hostil sem alertas/efeitos repetidos |

Aceitação adicional: pet/serviço determinístico com modelo descarregado; comunicação direta sem coordenador; delegação paralela com orçamento repartido/conflito/ciclo/Stop por ramo; Guardian com finding sem LLM, remediação permitida completa e outra recusada; conteúdo hostil sem alteração de política; lifecycle/crash, migração/restauro e leitor antigo sem perda. Os cenários originais de memória/anexos, decisões multicanal, multicalendário/DST, Som partilhado, voz real e entrega permanecem obrigatórios.

### Próxima dependência real após R43F3/F4

A retoma selecionada passou em146/147, mantendo o timeout145 como falha sem diagnóstico causal exato. Antes da adoção de854, falta um snapshot/restauro coerente do perfil atual, pelo gate de startup/publicação existente, com dados/configuração, memória, contextos, autoridade, resultados e efeitos incertos. O ensaio não pode renovar grants ou tratar ficheiros de lock copiados como posse. A recusa .30 validada emF4 é um requisito de admissão, não fecha DATA-01/REL-01. Manter também pendentes inline nativo e aceitação humana/provider/dispositivos/instalação nos seus âmbitos corretos.

### Backup do perfil atual e restauro — decisão R43F5

O backup do perfil atual reutiliza o inventário limitado e a cópia com verificação existentes, mas tem protocolo/namespace próprio, distinto da importação .30. Captura integralmente os dois payloads admitidos, incluindo memória, anexos, objetivos, concessões, contextos, resultados e recibos incertos. Executa em manutenção offline, antes de iniciar Chat/workers, sob o mesmo nome D-Bus e lock físico de startup; uma aplicação ativa causa recusa, sem tomada de posse. A revisão nativa mostra origem/destino/resumo e exige confirmação do fingerprint exato antes da cópia. Os ficheiros da aplicação podem conter informação e chaves privadas: o arquivo é local/privado e não se publica em Git ou em modelos.

Um arquivo verificado contém dados, não execução autorizada. Não transfere locks, identidade física do dono ou saldo renovado, nem é prova de autenticidade contra código do mesmo UID. O restauro completo continua obrigatório e exige transação própria pelo gate de publicação/admissão, com recuperação explícita da autoridade histórica e efeitos incertos; não fica concluído por este incremento de backup. Não alterar a instalação ou executar este modo sobre o perfil pessoal sem a autorização específica aplicável.

### Cobertura dos rascunhos antes do restauro — descoberta R43F6

A inspeção do produtor real confirmou `IndexedDraftStorage`/`coucou.sessionDrafts.v1` na janela e Map em memória no prompt inline. O IndexedDB observado no próprio QA143 fica em `data/fr.louisraille.coucou/databases/indexeddb`, fora do payload nativo. Portanto, backup F5 significa cópia íntegra dos dois payloads admitidos; não fecha a preservação integral de rascunhos. Antes de restauro/adoção, mover estes produtores para armazenamento nativo revisto, reutilizando dono/CAS e a API de drafts existente, com migração dos registos conhecidos, conflitos, revisões, reinício e ACK incerto. Preservar a base antiga até à confirmação; não copiar perfis WebKit/autenticados nem enviar o rascunho automaticamente. Esta dependência foi descoberta no código e não elimina DATA-01/REL-01 ou critérios originais.
