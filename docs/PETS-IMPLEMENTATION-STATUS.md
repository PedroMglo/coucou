# Estado de implementação dos pets

## 01/11 — Fala nativa e candidata mochi.32 (C6C3)

Fontes1003 integradas; 23 caminhos adotados sobre1000, incluindo três módulos novos de fala nativa. Passaram3182 testes Rust/38 ignorados,1333 web/0 ignorados, TypeScript e Vite. Nativo283 validou Tauri/GTK/IPC e processos sintéticos, não áudio. Versão0.5.1-mochi.32 instalada e ativa; prefixo.30 preservado. Os42 outputs web e os29 sons anteriores estão preservados.

[Composição, provas, entrega e limites](releases/0.5.1-mochi.32.md).

A versão.32 está agora instalada e ativa: nome D-Bus, executável empacotado e admissão nativa verificados; três originais de preferências/conversas e cópias nativas byte-exatas. A.30 foi preservada. O ensaio284 do próprio RPM passou com helper sintético; isto não fecha áudio real nem os49 requisitos.

## 31/10 — Seleção TTS e cancelamento por resposta (C6C2, fontes1000)

Fontes1000 integradas: seleção TTS capturada na ocorrência de Voz e no preview, validade após esperas e cancelamento do renderer por resposta exata. Falha do motor escolhido conserva o texto; só a seleção explícita system pode usar síntese do sistema. [Provas e limites C6C2](../.build/pet-goal-execution-next-r1/VOICE-PLAYBACK-CHECKPOINT-C6C2A3.json).

Passaram94 dirigidos (2 físicos ignorados),3167 gerais (38 ignorados),1332web/0skip, TypeScript, Vite e build598 inputs.42 outputs compilados integrados com29 WAVs existentes preservados. A instalação .30 não mudou. Ativação após restauro e áudio real continuam pendentes.

## 31/10 — Entradas imutáveis da Voz (C6C1, fontes999)

Fontes999 integradas: entradas de captura/ASR e seleção de inferência imutáveis por ocorrência de Voz, usando os runners e a fábrica existentes. Alterar pet, perfil, endpoint ou preferências durante a espera não redireciona o pedido; o contexto inválido é recusado. O snapshot não renova grants. [Provas e limites C6C1](../.build/pet-goal-execution-next-r1/VOICE-TURN-CHECKPOINT-C6C1A4.json).

Passaram88 testes de Voz (2 físicos ignorados) e3161 gerais (38 ignorados); build597 inputs. Provas sintéticas, sem áudio/modelos/quota reais. A instalação .30 continua intacta. O ditado conserva voice::transcribe; a configuração ASR desse caller ainda é capturada na transcrição. TTS e reativação governada não estão concluídos.

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

Implementado e validado no privado: recuperação atual em ambos os transportes; não integrado na árvore834 nem entregue na instalação. Continuam pendentes lifecycle/adoção, quatro controladores, autonomia/CLI integral, mesh, fontes/rotinas, Sistema/voz, Guardian e aceitação/entrega aplicáveis.

## R43F10B2C3D2B3C3A2 — comparação nativa original e atual

Privado: 992 fontes/591 entradas nativas. Passaram 55 testes dirigidos, 3134 gerais (38 ignorados), 1326 web/0 ignorados, TypeScript, Vite, build e nativos266/268/269. [Fontes, provas e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-ORIGINAL-OWNER-PARTIAL-CHECKPOINT-R43F10B2C3D2B3C3A2.json).

Disponível no privado: origem original e atual visíveis na revisão humana; recibo retém a comparação; decisão antiga é recusada.266 prova portal,269 X11 com callbacks no servidor virtual,268 restauro7/identidade antiga desconhecida.267 conserva timeout JS e ausência de retorno externo;269 usa abertura explícita pelo comando de UI existente, sem mudar binário ou asserções. Ainda falta recuperação utilizável quando o contexto antigo é indisponível, além de lifecycle/adoção, quatro controladores e restante sistema.

## R43F10B2C3D2B3C2A1 — restauro 7 e suspensão de todas as componentes

Composição privada: 992 fontes/591 entradas nativas. Passaram 43 testes do módulo no A1. A execução geral C2 passou 3130 testes e teve duas expectativas antigas corrigidas no reteste dirigido; 38 ignorados. A1 altera apenas cfg(test), sem nova execução geral/build/nativa. Build e ensaio nativo 265 de C2, web1324/TSC/Vite B3 são reutilizados por identidade explícita dos respetivos inputs. [Fontes, provas e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-RESTORE-PARTIAL-CHECKPOINT-R43F10B2C3D2B3C2A1.json).

Nativo265 restaurou/admitiu o backup 7 exato, manteve slots e artefactos, exibiu o motivo de dono indeterminado, recusou confirmação/novo registo, preservou reinício e novo backup de root completo.263 conserva erro da fixture de Admission4;264 conserva a falha que revelou a omissão corrigida. Ainda falta mostrar a encarnação original no controlo de comparação e recuperar de forma utilizável um novo scope sem resolver falsamente o antigo.

## R43F10B2C3D2B3B8 — recuperação dos atalhos e leitores antigos

Composição privada: 992 ficheiros de fonte e 591 entradas nativas; backend e binário B7, uma correção de fixture cfg(test) B8. 52 testes dirigidos B8 passaram; a execução geral B7 teve 3129 testes aprovados, uma falha de fixture e 38 testes ignorados; a falha foi conservada e corrigida com reteste dirigido.Os 1324 testes web, TypeScript e Vite de B3 e os 18 testes D-Bus de B5 reutilizados por identidade explícita; build B7 e ensaios nativos 261 (X11) e 262 (portal) passaram. Não houve nova execução dessas suites no B8. [Fontes, provas e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-RECONCILIATION-PARTIAL-CHECKPOINT-R43F10B2C3D2B3B8.json).

Disponível em privado: rever e registar observação sem teclas, conservar resultado anterior e aprovar nova tentativa distinta; após restart há nova observação. Leitor984/6 recusa7 pendente e reconciliado sem alterar ficheiros. Backup 7 preserva diário e artefactos. Ainda faltam restauro 7/admissão nativos, lifecycle/composição/adoção e restantes4componentes; o produto instalado continua .30.

## R43F10B2C3D2B3A1 — resultado pendente e publicação

Privado984/586inputs: resultado nativo pendente antes de publicação, finalização com referência exata e limpeza limitada no aborto.51dirigidos,16D-Bus,3118Rust/36ignorados,1318web/TSC/Vite/build e nativos257portal/259X11 passaram. A fixture de publicação pendente é recusada pelo leitor983 e pelo atual, com hashes dos executáveis realmente arrancados. Observação/reconciliação própria ainda por implementar. [Provas e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-PUBLICATION-PARTIAL-CHECKPOINT-R43F10B2C3D2B3A1.json).


## R43F10B2C3D2B2B3 — aprovação dos atalhos e suspensão X11

Privado983/585inputs: aprovação com três confirmações, intenção/despacho/resultado, reserva exata do portal, cancelamento, estado e libertação tipada própria. Native252 confirmou plugin/callbacks/suspensão em Xvfb;253 confirmou portal sintético/cancelamento; ambos leitor980/reinício sem novos registos. B3 ainda bloqueia adoção/entrega. [Checkpoint e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-APPROVAL-PARTIAL-CHECKPOINT-R43F10B2C3D2B2B3.json).


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


## R43F9B2D4B — admissão independente com proveniência, 26/10/2026

SDK909/B5 admite a MESMA chave preparada pela variante NoOriginalAuthority. Manifest/admission4 explícitos sob Header2 ligam custódia/HMAC/referências/proveniência; recibo3 inspectionOnly separado conserva o recibo1 anterior. Activation/Approved/Bound/Storage e o mesmo ledger permanecem. A adição da Authority é um pacote tipado não serializável: só prefixos exatos em staging e bytes completos no nome final, com originais preservados antes da cópia e publicação de diretório sem substituição. Sem RNG/ensure, novos grants, saldo renovado ou morte de dono inferida na admissão.

Antes de opened valida originais sem Authority, referências/objetivos antigos e pacote/custódia exatos; depois usa metadados/Blobs locais sem recapturar novos objetivos/grants como históricos ou depender do arquivo externo. IDs antigos são negados antes de Chat/IO; objetivos novos têm consentimento e orçamento próprios. Backup2 reconhece4 e conserva trabalho novo, preparação e originais; fixture de nova época C3/D1/D2 mantém chave/proveniência e nega todas as gerações.

Passaram2837Rust/21ignorados/7novos,focused7,build530 e nativo188: configuração3, mesma chave/admission4, leitor907/wronghash recusados sem escrita, novo SID sintético, objetivo/decisão próprios com2efeitos no leitor privado, Stop na segunda HTTP sintética, reinício/ACK com arquivo ausente sem replay e backup4 portátil com fonte offline. Providers sintéticos,0modelos/quota reais; web1255/TSC/Vite reutilizada por bytes iguais. B2 reteve erros de acesso a wrappers privados nos testes; B3 reteve ContextRequired num helper novo sem contexto; B4 corrige a preparação real do contexto no helper;186/187 conservaram recusa de um catálogo de resultados criado sem protocolo pela fixture. B5 move o dado opaco para inbox, produção B2/B3/B4/B5 idêntica. Falta antecipar a deteção de catálogo inválido antes do IO de modelo. [Checkpoint/fontes/provas/ignorados/limites](../.build/pet-goal-execution-next-r1/INDEPENDENT-ADMISSION-PARTIAL-CHECKPOINT-R43F9B2D4B.json).

Continuam por resolver explicitamente geração incerta/seed inválido/pre-manifest/staging anónimo, migrações/revalidações voz/mobile/autostart/B3/adoção e aceitação integral. Nova época Admission4 tem prova de fixtures, não nativa. Raiz834/instalação.30 intactas,sem publicação;49critérios integrais abertos.


## R43F9B2D4A — preparação sem autoridade original, 26/10/2026

SDK907/A5 liga o comando nativo existente a uma variante só para ausência real da Authority na origem e no destino Header2. O job tipado .restore-authority persiste manifest, referências e intenção de geração antes do RNG; o mesmo índice/HMAC nega objetivos/decisões antigos, sem grants nem alteração do payload. Modelos/configuração e revisões exatas fazem parte da revisão. A chave preparada permanece fora do caminho operacional.

A3 reproduziu uma falha da primeira implementação: remover a chave única fazia o seed vazio parecer não iniciado. A4/A5 consome a oportunidade de geração na chamada que persiste generation.json; chave ausente depois disso significa keyGenerationUncertain/manualReconciliationRequired, sem regeneração. Cortes duráveis com chave válida retomam o mesmo manifest/chave/índice; seed parcial ilegível ou desconhecido é conservado/recusado. A4 reteve uma falha de observação imediata da morte de um descendente num teste existente de helper; A5 usa a mesma espera limitada1s do teste vizinho, mantendo timeout400ms e produção idênticos. Passaram2830Rust/21ignorados/8novos,build528 e nativo185 sem inferência/quota: app cria perfil privado, catálogo sintético é inserido offline sem Authority, CLI prepara e reconhece o mesmo recibo/chave sem escrever no payload; leitor904 recusa sem escrita. Web1255/TSC/Vite reutilizada por bytes iguais. [Checkpoint, prova negativa, fontes e limites](../.build/pet-goal-execution-next-r1/INDEPENDENT-AUTHORITY-PREPARATION-PARTIAL-CHECKPOINT-R43F9B2D4A.json).

**NoAuthority ainda não tem admissão operacional.** D4B tem de publicar/admitir a mesma chave sob revisão, preservar proveniência na admissão/backup e resolver a recuperação explícita de geração incerta. Não é autoridade recuperada da origem ou consentimento autónomo. Restantes migrações/revalidações/B3/adoção e49critérios integrais abertos; raiz834/instalação.30 intactas,sem publicação.


## R43F9B2D3 — backup e continuidade Header3, 26/10/2026

SDK904/D3A5 conserva o contentor2 para os roots completos de um admitido Header3, incluindo trabalho atual/desconhecido, originais, history2 e custody C3. A verificação portátil permanece pura. O novo destino usa o mesmo Header3; paths literais derivam da admissão verificada, limitados a9 e coerentes com HMAC e lineage. Em9 preserva backup/inspeção mas recusa outro restauro antes dos efeitos. C3/D1/D2/Storage e a mesma Authority executam a nova revisão, sem renovar permissões ou consumo históricos.

A cópia de originais C3 valida as revisões imutáveis por descritores/Stamp, bindings, controlos e Claim em cada entrada, conservando hashes integrais antes de ready e efeitos. Não usa TTL ou ausência de eventos como integridade; se o orçamento de FDs não permitir retenção, percorre as mesmas revisões. Native183 reteve timeout120s provocado por validação integral repetida por entrada;184 passa o mesmo cenário/limite com seed sintético182 fresco. A3 reteve dois erros de chamada nos testes; A4 corrige o acesso à variante Next. A4 full reteve uma expectativa errada de hash após interrupção; A5 exige nova revisão da fase e recusa o hash antigo, conservando o mesmo manifest/custody.

Passaram2822Rust/21ignorados/11novos desdeD2,build525 e nativo184: backup3 atual completo, origem offline, leitor902 recusa sem escrita, restauro Header3/depth3, nova autoridade/configuração/admissão, quarto SID distinto e objetivo com consentimento próprio, efeitos verificados no leitor privado, reinício/ACK com arquivo ausente sem replay e grants anteriores intactos. Providers sintéticos; web1255/TSC/Vite reutilizada por bytes iguais. [Checkpoint, fontes, ignorados, falhas e limites](../.build/pet-goal-execution-next-r1/RECURSIVE-BACKUP-PARTIAL-CHECKPOINT-R43F9B2D3.json).

Seguem NoAuthority/revalidações voz/mobile/autostart/B3/adoção, migrações e aceitação integral. A unidade privada não está integrada na raiz834 nem entregue na instalação.30. Todos os49critérios integrais continuam abertos; sem commits/publicação/modelos ou quota reais.


## R43F9B2D2 — admissão operacional Header3, 26/10/2026

SDK902/D2A4 reutiliza Activation/Projection/Approved/Bound/Storage/Chat para manifest/admission3 com extensão tipada (payloadDepth2 e Scope D1 exato). O job .restore-admission não substitui a custody .restore-activation nem duplica os originais. O mesmo inicializador admite só os deltas canónicos/staging previstos, sob escolhas/revisão nativa nova. Índice2 entra antes de Chat; IDs anteriores e atuais ficam históricos. HMAC/chave/protocolo/antepassados/custody são ligados por Blobs e dados operacionais usam os caminhos aninhados corretos. Arranque e ACK após admissão dependem dos dados locais, não da localização do arquivo externo. Manifest1/admission2 mantêm bytes; leitores antigos recusam3.

Passaram2 811Rust/21ignorados/7novos,build523 e nativo182 com Tauri/CLI reais e providers sintéticos. O percurso cria terceira sessão CLI distinta e objetivo/decisão novos, executa duas operações no leitor privado, interrompe inferência por Stop, reinicia e reconhece ACK sem repetir efeitos ou reescrever trabalho posterior; arquivo externo ausente não impede reabertura/ACK. Grants/consumo anteriores e custody/originais ficam intactos. A2 reteve falhas da preparação create_new; A3 reteve Busy de duas posses na mesma fixture; A4 corrige só esses testes, produção idêntica.181 parou no peer de teste com path antigo;182/A5 declara depth2 e verifica Admission3, sem alterar produção ou repetir suites sem necessidade. [Checkpoint, fontes, ignorados e limites](../.build/pet-goal-execution-next-r1/EPOCH-ADMISSION-PARTIAL-CHECKPOINT-R43F9B2D2.json). Web1255/TSC/Vite reutilizada por igualdade exata.

**Backup do admitido3 ainda recusa sem escrita** e é a próxima dependência antes de adoção/entrega. NoAuthority/revalidações voz/mobile/autostart/B3/adoção e aceitação integral permanecem abertos.902 privados não foram integrados na raiz834 nem substituíram a instalação.30;49critérios integrais abertos,sem commits/publicação ou modelos/quota reais.


## R43F9B2D1 — revisão da configuração e referências Header3, 26/10/2026

SDK901/D1A1 liga o comando nativo existente à configuração do payload operacional de Header3, após C3 inteiramente publicado e originais verificados. Recibo2 continua configurationInspectionOnly: liga manifest/ready/histórico C3, modelo/endpoint e índice2 de referências. Esse índice une CID/SID/tarefa atuais aos anteriores selados, conserva proveniência limitada e não copia mensagens nem concede controlo. Índice1/recibo1 conservam bytes; Admission2 continua a exigir índice1.

Passaram2 804Rust/21ignorados/9novos,build522 e nativo180 com CLI/Tauri reais e providers sintéticos. O ensaio compõe independentemente o índice a partir dos bytes anteriores/atuais e confirma o mesmo hash e os dois conjuntos de CID/SID; verifica revisão sem escrita, só recibo2 acrescentado, hash errado/replay/ACK, modelos/destinos e recusa pelo binário901 anterior. Web1255/TSC/Vite reutilizada com fontes não Rust idênticas. [Checkpoint, fontes exatas, ignorados e limites](../.build/pet-goal-execution-next-r1/EPOCH-CONFIGURATION-PARTIAL-CHECKPOINT-R43F9B2D1.json).

**Header3 continua inativo.** Segue D2: seleção/inicialização/admissão operacional com os mesmos originais e referências, sem substituir o manifest C3 nem reativar sessões/aprovações antigas. NoAuthority/revalidações/B3/adoção/entrega permanecem abertos;49critérios integrais abertos,raiz834/instalação.30 preservadas,sem commits/publicação ou quota/modelos reais.


## R43F9B2C3 — histórico de autoridade de várias épocas, 26/10/2026

SDK901/C3A6 conserva, na mesma Authority/chave/locks/ledger, a cadeia limitada de índices HMAC anteriores e a união de decisões/objetivos históricos negados. Corpo1/protocolo2 mantém os bytes compatíveis; corpo2/domínio2/protocolo3 liga o antepassado exato e recusa leitores antigos/downgrade/cadeias adulteradas. Grants, consumo e incerteza mantêm os seus bytes; não se renova saldo ou prova de morte do dono. O comando nativo Header3 retém os dois roots originais completos antes de alterar os três metadados previstos, sob o mesmo Claim.

Passaram 2 795 Rust/21 ignorados,19 casos novos desdeC2, build/522 inputs e nativo179 com CLI/Tauri reais e providers sintéticos. Provas de duas/três épocas, recusa de IDs antigos/atuais, consentimento novo independente, consumo incerto com dono original ainda vivo, cortes de publicação, instância em cache, originais, ACK/replay e adulteração. A4/178 também passou e fica conservado. A5/A6 acrescentam recuperação revista de temporários: só prefixos exatos do antepassado autenticado são removidos por Stamp depois de guardar o antepassado inteiro; identidade da transação mantém-se e consentimento inclui a fase. Em179 o resíduo de prefixo é sintético, não uma morte nativa a meio dessa escrita. Web1255/TSC/Vite reutilizada por fontes não Rust idênticas. [Contrato](../.build/pet-goal-execution-next-r1/MULTI-EPOCH-AUTHORITY-CONTRACT-R43F9B2C3.md) e [checkpoint/provas/limites](../.build/pet-goal-execution-next-r1/MULTI-EPOCH-AUTHORITY-PARTIAL-CHECKPOINT-R43F9B2C3.json).

**Header3 continua inativo:** falta inicialização/configuração e admissão CID/SID/tarefa sob revisão nova. Temporários desconhecidos/fora do prefixo ou staging anterior ao manifest não são reparados automaticamente. Seguem NoAuthority/revalidações voz/mobile/autostart/B3/adoção e aceitação integral. Raiz834/instalação.30 intactas;49critérios integrais abertos,0quota/modelos reais, sem commits/publicação.


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


## R43F6D — rascunhos nativos privados, 26/10/2026

SDK865/F6D: 2669 testes Rust passaram, 21 ignorados, compilação nativa e1219 JS/TSC/Vite (fontes web idênticas aA). A janela de conversa usa agora `SessionDrafts` com armazenamento nativo, dono/CAS existentes, revisões u64 e UTF16 exatos. A migração lê todos os registos conhecidos pela API IndexedDB, cerca v1 com versão2, conserva a origem após ACK e não repete importação sobre edições posteriores. Novo rascunho Mochi retém o CID existente antes de publicar, sem mensagem/modelo/consentimento; o arquivo conserva referências após reinício. O teste de capacidade máxima mantém os limites lógicos anteriores. [Checkpoint e limites](../.build/pet-goal-execution-next-r1/NATIVE-DRAFTS-PARTIAL-CHECKPOINT-R43F6.json).

Nativo154 usa IPC e janela reais em perfil privado: origem858/F5D com três rascunhos sintéticos (incluindo Gravity/Copilot inativos e tombstone), importação/edição/conflito/ação explícita/reinício e uma nova conversa vazia retida só pelo rascunho; o catálogo nativo entra byte a byte no backup dos payloads. Zero chamadas de modelo/quota.154 foi executado na produçãoC;D altera só uma fixture de teste e o binário foi comparado byte a byte, preservando o alcance exato da prova.152 é falha do setup de origem;153 revelou CID vazio perdido, corrigido porC; ambos preservados. Não se provou callback de escritor v1 vivo: o backend de origem foi recolhido antes da atualização, e a nova abertura v1 é recusada.

Ainda faltam o produtor inline, resolução funcional de importação divergente e recuperação dos destinos legados órfãos; estes dados não são apagados nem redirecionados. Faltam restauro/adoção e aceitação humana/instalada. A raiz834 e instalação .30 continuam preservadas;43 IDs originais+6 Guardian permanecem abertos integralmente. O objetivo completo continua ativo, sem novos commits ou publicação.


**R43F5D validado em privado — 26/10:** SDK858 acrescenta revisão/confirmação/inspeção de backup dos dois payloads nativos admitidos e corrige a conclusão sob locks transitórios, conservando prova/aprovação e prioridade de Stop. Passaram2655 Rust/21 ignorados,23 dirigidos/1 ignorado,check-tests/build e composição nativa151;149 cobre as recusas e integridade do arquivo. São perfis, motores e escolhas sintéticos. **Rascunhos IndexedDB/inline ainda ficam fora desta cobertura**: segue persistência nativa/migração dos rascunhos, depois restauro/adoção. Raiz834 e instalação .30 preservadas; os49 critérios integrais continuam abertos. [Provas e limites](PETS-VALIDATION.md#backup-payload-conclusao-r43f5) · [Uso e cobertura](PETS-DATA-BACKUP.md).

**R43F3/F4 validado em privado — 25/10:** a composição854/F4 conserva a memória selecionada ao continuar objetivos com novo consentimento e orçamento adicional explícitos. Tauri146/147 provam duas rondas, aprovações separadas, revogação durante IO e preservação por reinício/leitor antigo, com motores sintéticos. A importação legada passa a recusar proveniência nova escondida em formato antigo e catálogos de runtime que seriam omitidos. Passaram2641 Rust/21 ignorados,check/build e a composição nativa147. O timeout145 permanece sem causa exata confirmada; não é uma regressão dada como corrigida. [Provas e limites](PETS-VALIDATION.md#retoma-memoria-importacao-r43f4). Seguem backup/restauro coerente do perfil atual e adoção; raiz834 e instalação .30 preservadas. Sem novos commits ou aceitação integral dos49 requisitos.

**R43F2E2 validado em privado — 25/10:** SDK854 liga a seleção/revisão de memória aos chats manuais, vincula anexos/contexto ao token e apresenta destinos API conhecidos sem prometer contenção do fornecedor. Passaram2638 Rust/21 ignorados,1212 JS/TSC/Vite/check/build e Tauri143/144. A janela de conversa passou nos dois motores locais, três CLI e Notion/Codex; o inline tem provas de controller/unidade, com aceitação nativa pendente após falha de ativação em Xvfb. São escolhas automatizadas e providers sintéticos. [Alcance e limites](PETS-VALIDATION.md#memoria-chat-contexto-r43f2e2). Raiz834/instalação .30 preservadas; seguem retoma com memória e migração/adoção.

**Histórico R43F2E1 — 25/10:** SDK850 acrescenta catálogo/seleção por revisão e revisão completa com consentimento próprio nos controlos reais de objetivos Ollama/LMStudio, além da projeção de memória por caller. Reutiliza CRUD/CAS, admissão, concessão e writer existentes. Passaram2634 Rust/21 ignorados (487 inputs nativos iguais entre E1a/E1b),1200 JS/TSC/Vite/check/build e Tauri139. Os cliques/consentimentos foram automatizados e os motores sintéticos: não é aceitação humana ou instalada. Seguem revisão manual de chat, retoma com memória e adoção/migração/entrega; os49 requisitos continuam abertos nos critérios integrais. [Provas e limites](PETS-VALIDATION.md#memoria-controlos-objetivos-r43f2e1). Raiz834 e instalação .30 preservadas.

**Histórico R43F2D — 25/10:** SDK846 prova os envios de memória de Anthropic/OpenAI/Google através da implementação nativa comum, com HTTP loopback sintético, dados separados da pergunta e recusa de destino/modelo/credencial/pergunta alterados. Foi retirado o opt-in de fallback automático Anthropic; APIs e percursos locais usam o destino exato sem seguir redirecionamentos. Passaram2632 Rust/21 ignorados, check/build e Tauri136:13 pedidos HTTP e17 prompts CLI sintéticos, incluindo memória/decisões/recolha/retoma dos três CLI e serviço separado, objetivos locais e dois redirecionamentos recusados sem pedidos ao destino alternativo. Não certifica contas reais, routing interno do fornecedor, UI humana ou instalação. Raiz834 e instalação .30 preservadas. Seguem interface humana/callers, retoma de objetivos com memória e adoção/migração/entrega; R43 e os49 requisitos continuam abertos.

**Revisão posterior de 25/10 — Guardian e execução integral:** os 43 requisitos originais conservam os seus critérios; acrescentam-se GUA-01..06 no [contrato System Guardian](SYSTEM-GUARDIAN-ARCHITECTURE.md) e na [auditoria rastreável](PETS-REQUIREMENTS-AUDIT-2026-10-25.md). Pet, runtime comum, motor de inferência e papel funcional são entidades distintas. `ai_ollama` continua coordenador preferencial configurável, sem ser router obrigatório ou concedente de permissões. Guardian é especialista da mesma fábrica, sem autoridade própria adicional. Não há um pet Guardian operacional por esta atualização documental.

**Histórico anterior R43F2A:** raiz integrada834 e instalação .30 preservadas. SDK843/R43F2A passou 21 testes dirigidos de contexto, 2617 Rust/21 ignorados e cargo check em 483 inputs nativos imutáveis. O novo contrato de revisão mostra dados completos e destino nativo, separa consentimento de objetivo e verifica âmbito/credencial/revogação. É fundação privada: ainda falta a ligação à admissão, concessão assinada e corpo real anterior ao hash/reserva. Não foi acrescentada prova Tauri ou build instalada por este incremento. A barreira F1 permanece fechada; não declarar R43 concluído.

**Restrições vigentes:** um único agente, sem novos commits/pushes/merges de qualquer pasta; a publicação anterior do dossier foi uma exceção histórica já executada. Sem alteração da instalação/autostart/serviços, Wi-Fi/VPN/firewall/dispositivos ou dados pessoais. Sem quota real Copilot/GitHub, segredos em modelos/logs/Git ou bases de outros projetos. Alterações persistentes do anfitrião seguem sysworkspace e autorização específica; o desenvolvimento atual fica no projeto. Estados/estimativas antigos são históricos e não medem conclusão.

**Atualização de estado R43F1A — 25/10/2026:** no SDK privado841 passou a preparação durável de memória selecionada para objetivos: referências exatas por pet/conversa de origem, contexto versão2 e objetivo versão11 apenas quando há seleção, sem copiar os dados brutos para o diálogo inicial. Passaram 2612 testes Rust/21 ignorados, check/build e o percurso Tauri privado130, incluindo reinício, revogação e recusa do leitor840 sem alterar os ficheiros. Não houve chamadas a modelos ou quota. A admissão autónoma com memória permanece deliberadamente recusada até R43F2 ligar revisão humana dos dados/destino, consentimento próprio do objetivo e mensagem anterior ao hash/reserva canónicos. A raiz aceite permanece834; a app instalada permanece .30. R43 e os 43 requisitos continuam abertos.

Observação documental: **25/10/2026**. A
[arquitetura](PETS-ARCHITECTURE.md) define requisitos e prioridade; o
[roadmap](PETS-ROADMAP.md) define unidades e aceitação. Este documento distingue
código, provas e entrega. Não há percentagem global de conclusão comprovada.

**Objetivo revisto em 24/10:** lido integralmente e conservado em
[PETS-GOAL.md](PETS-GOAL.md), com os 43 requisitos e o trabalho existente.
Arquitetura/roadmap explicitam pets de primeira classe, comunicação direta,
delegação paralela/assíncrona, Ollama como coordenador opcional e separação entre
pet/runtime/motor. São requisitos, não capacidades novas anunciadas como prontas.
O caminho `mochi/delegation.ts` inspecionado prepara texto editável; não comprova
o protocolo durável nem a coordenação resiliente. Segue a recuperação individual
em curso, antes da implementação/aceitação de TEAM-01/02 segundo as dependências.

A revisão atual não apresenta percentagem ou horas de conclusão sem base mensurável. O alcance é acompanhado pelos requisitos, provas e lacunas da auditoria.


**Estado corrente integrado localmente:** 834 fontes, sem novo commit após a
suspensão pedida pelo utilizador. Passaram **2582 testes Rust/21 ignorados e
751 ensaios privados no anfitrião**, cargo check/build, **1186 JavaScript**, TypeScript
e Vite. Bundle42 exato; os avisos de compilação permanecem registados nas provas. O último commit
publicado continua `bbe12e2` (769 fontes), na branch `feat/mochi-cli-accounts`.
[Provas e limites atuais](PETS-VALIDATION.md#memoria-local834).

**Trabalho atual R43 em ambiente privado:** o incremento840/E1 passou 2606
Rust/21 ignorados, 35 testes de memória e check/build. Tauri privado provou
contexto nativo para Codex, Copilot, Gravity e um serviço com assistência Codex,
com 17 pedidos de protocolo: destino/ID exatos, reinício sem repetir memória bruta,
revogação com recolha dos CLI próprios, isolamento entre donos do mesmo backend,
proveniência incerta e falha da escrita do ID antes do prompt. Os oito pedidos
locais de Ollama/LM Studio e revogação durante aprovação/comando também passaram
na mesma fonte840. Sem quota real; são protocolos sintéticos, não aceitação das
contas operacionais ou do áudio. A raiz aceite continua834, sem instalação.

Seguem a memória na revisão/admissão/recibos dos objetivos e os três caminhos API
com envio HTTP sintético, depois confirmação humana e descritores. A auditoria
já identificou os pontos comuns de contexto e hash a reutilizar.
[Âmbito e pendências](PETS-VALIDATION.md#contexto-memoria-r43). As falhas anteriores
ficam preservadas; não alterar a estimativa global por estes testes nem confundir
o protótipo com a aplicação instalada.

**Incremento830 — continuidade CLI e disponibilidade do perfil:** Codex,
Gravity e Copilot usam o mesmo resolvedor de sessão; uma pasta ou preferência
de criação incompatível causa recusa explícita antes do prompt, sem substituir
silenciosamente a associação. Nova conversa conserva o arquivo. A retoma Gravity
não volta a enviar bypass e a opção global não altera a política ACP do Copilot.
Os quatro donos testados incluem um serviço com assistência Codex.

A validação de diretórios vivos deixa de confundir criação/remoção legítima de
pastas com perda de posse. Preserva identidade, permissões, locks, marcadores e
snapshots estritos das fontes. A app privada passou 200 consultas sob 583260
mutações, o percurso CLI de quatro donos e os dez ciclos anteriores de
recuperação/Stop/continuação. Comparações anteriores reproduziram ambos os erros.
Transporte CLI/modelos sintéticos, sem quota real. A política dos terminais
operacionais, memória, restantes executores e a entrega mantêm aceitação própria.

**Incremento828 — LM Studio pela fábrica e runtime comuns:** associação explícita
do pet, metadados nativos locais e ligação exata de servidor/modelo/instância/
âmbito da credencial. O provider vazio antigo continua válido e sem ativação
automática. Objetivos, contexto, consentimento, orçamento, planeamento, receitas,
Stop, recibos e recuperação usam os mecanismos comuns. A cache de revisão
compara a identidade exata, mesmo com época do descritor igual. Voz, shell e
gestão de modelos não são herdados.

Passaram dez percursos na app Tauri privada: os sete anteriores e três novos
de LM Studio. Verificaram associação por CAS sem alterar outros pets, conversa
manual antiga, conversa associada com o mesmo histórico, independência face ao
Ollama, execução/continuação após falha e dois reinícios, Stop com consumo
desconhecido conservado e recusa de modelo alterado antes da confirmação ou do
HTTP. Esta última não emite inferência nem debita uma tentativa. A autenticação
HTTP foi testada com credencial sintética capturada; motor LM Studio real e
cofre operacional não foram alterados/testados. Outros providers, voz, memória,
cooperação, Sistema e entrega mantêm aceitação própria.

**Incremento834 — gestão explícita de memória comum aos pets:** preferências e
notas têm dono da fábrica, âmbito de pet/conversa, proveniência de utilizador,
sensibilidade, validade e retenção até exclusão. Criação, consulta/pesquisa com
páginas limitadas, correção e exclusão usam revisões concorrentes e o mesmo
armazenamento privado das fontes. Expirar suprime disponibilidade, conservando
consulta explícita. Corrupção fica isolada e preservada.

A app privada verificou os 24 pets não retirados do catálogo, incluindo donos de
serviço com o mesmo adaptador. Uma conversa vazia recebe identidade durável na
criação explícita de memória, sobrevive a reinícios e fica no arquivo após Nova
conversa, sem mensagens fabricadas. O catálogo restaura a retenção; o pin em RAM
protege a concorrência entre criação e reset. Passaram os 12 percursos anteriores,
três após corrigir apenas a ativação LM Studio nas fixtures. São 13 percursos
positivos, mais o controlo C que reproduziu perda do CID antes da correção.

O backend não inclui estas recordações automaticamente nos modelos. Segue R43:
seleção/revisão/validação comum aos transportes e admissão dos objetivos, com
tratamento explícito de revogação e do contexto já entregue a sessões nativas.
Projetos, derivados/resumos/episódios, partilha e MEM-01/02 conservam os seus
critérios. A app instalada continua .30 e commits/pushes suspensos.

**Incremento825 — trabalho novo com incerteza explicitamente revista:** a
continuação normal conserva `unreconciledEffects`. Uma opção separada prepara
revisão nativa apenas quando cada efeito desconhecido tem passo/âmbito assinado
e corresponde a pausar, reproduzir ou definir volume absoluto. A referência
schema2 guarda o manifesto revisto; o grant antigo, consumo e resultado
desconhecido permanecem exatos. Novo planeamento Ollama local e cada receita
exigem as suas próprias aprovações; não se reabre a tentativa antiga.
A app privada passou recusa normal/textual, recusa explícita, Busy, publicação
parcial, ACK, nova lease e conclusão, seguida de outro reinício e consulta do
histórico. Cancel antes da escolha bloqueou a autorização sem IO. Os cinco
percursos anteriores passaram. Falta paridade de execução nos restantes
adaptadores; operações não classificadas e origens ausentes continuam bloqueadas.
[Âmbito e provas](PETS-VALIDATION.md#incerteza-local825).

**Continuação após R40:** auditar a paridade de execução e decisões dos restantes
providers, preservando os limites reais. Codex, Gravity e Copilot mantêm as suas
pendências próprias; streaming não garante limites sobre as ferramentas internas
das CLI. Usar a fábrica e controladores comuns e distinguir capacidades
observadas de execução efetivamente admitida. Não consumir quota real de
Copilot/GitHub. Os 43 requisitos permanecem abertos.

**Incremento823 — recuperação de recibos sem alterar o histórico selado:** a API
existente autentica a continuação e conserva o lock do objetivo durante a
transação da autoridade. Antes de selar, mantém a liquidação nativa original;
depois de selar, devolve a evidência autêntica em leitura, mantendo fase, reserva e
bytes assinados. Passo e receita partilham a classificação `accountingFrozen`.
A app privada passou o erro original, consulta repetida, Cancel posterior,
recibo ausente/adulterado e resultado já Finished/Settled. Uma nova execução
explicitamente aprovada concluiu depois da consulta, sem alterar a admissão antiga.
Os percursos anteriores de cinco crashes/reinícios e replaneamento/Stop passaram.
São três regressões Rust e três JS novas; sem schema persistente adicional.
Faltam o tratamento explícito dos efeitos genuinamente desconhecidos, restantes
providers e aceitação dos demais requisitos. Perfis anteriormente corrompidos não
são reparados automaticamente. [Provas](PETS-VALIDATION.md#recuperacao-local823).

**Auditoria R39A anterior, conservada:** uma falha antes de
conservar evidência deixou uma operação realmente executada com resultado
histórico desconhecido. Após reinício/fecho, o backend conservou o consumo e
recusou continuar com `unreconciledEffects`, mesmo com estado atual coincidente,
alterado por outro ator ou indisponível. Cancel permaneceu terminal. A recusa é
correta; o incremento825 acima acrescenta revisão separada apenas para operações
de estado autenticadas, conservando sucesso histórico desconhecido e sem replay.
[Âmbito da auditoria](PETS-VALIDATION.md#efeitos-desconhecidos-r39).

**Incremento822 — retirada de autorização sem admissão:** o backend revê apenas
um canónico assinado nunca reclamado e retira-o por decisão explícita, com recibo
auditável/schema8. Não recria consentimento, admissão ou execução e não altera o
objetivo. Busy/resposta perdida conservam a escolha; depois de reiniciar, o recibo
permite confirmar a retirada assinada. A app privada passou o crash entre as duas
publicações, recusa, retirada, novo crash/ACK e continuação separadamente autorizada
até conclusão, mantendo o consumo desconhecido anterior. As falhas G/H/I/J/L motivaram correções de contenção entre consulta e publicação. A certificação
partilhada, aquisição limitada antes da escrita e preservação da causa/decisão no
registo receberam sete testes; o percurso M e o ciclo nativo anterior passaram. Revisão
comum aos quatro pets não significa execução autónoma equivalente. Seguem efeitos
de ferramenta sem recibo recuperável e os restantes requisitos. A app .30 continua
intacta, e o sistema completo mantém-se em desenvolvimento.

**Incremento815 — continuação do objetivo após falha:** revisão/admissão nativas
vinculam o fecho confirmado, histórico, consumo incerto e limites adicionais
escolhidos. O mesmo objetivo recebe nova autorização/dono, sem reabrir lease,
receita ou decisão antigos. Fonte/schema10, política/modelo/contexto, bytes de
recibos e proveniência continuam autenticados. A fábrica declara revisão comum
nos quatro adaptadores; a execução real verificada continua limitada a Ollama
local/Spotify. A app privada passou a continuação até conclusão, conservando a
chamada interrompida, e o ciclo anterior voltou a passar. Seguem autorizações
sem admissão após crash e reconciliação de efeitos desconhecidos. A fixture
Gravity teve uma falha concorrente ainda sem causa; os seis testes isolados e
a bateria completa em série passaram sem mudar as suas fontes. A instalação .30
permanece intacta, e o objetivo completo continua ativo.

**Incremento811 — histórico cumulativo autenticado:** a consulta contabiliza
todas as admissões registadas e procura grants assinados omitidos, sem fabricar
objetivos truncados. Conserva consumo incerto, distingue ledger legado ausente
de uma execução nunca reclamada e não inventa medição de tokens/custo. A cobertura
dos formatos antigos é explícita: linhas autenticadas presentes não provam um
head histórico integral. Quinze testes novos e o percurso nativo privado passaram;
uma segunda admissão manteve o consumo desconhecido anterior. O ciclo anterior
e os quatro crashes/reinícios também passaram. A consulta não dá execução ou
retoma. Segue revisão/admissão de continuação com histórico/limites vinculados.
A instalação .30 permanece intacta, e o objetivo completo continua ativo.

**Incremento808 — revisão e fecho após falha:** a escolha nativa conserva a
autoridade original, prova de ausência, origem/recibos e consumo. Fecha apenas
a execução antiga, com recibo assinado/idempotente; resposta perdida, contenção
e ACK incerto retêm a escolha. Depois do reinício, um fecho já assinado permite
confirmar a sua projeção exata sem nova aprovação, controlador ou inferência.
O objetivo conserva cancelamento/conclusão genuínos ou fica RecoveryRequired.
Treze testes novos e a app isolada passaram quatro crashes/reinícios, incluindo
falha de escrita depois do fecho e uma conclusão já verificada. Consumo e efeito
desconhecidos não foram apagados/repetidos. O ciclo completo privado voltou a
passar. Segue continuação explícita com nova admissão e uso acumulado; RUN-04,
restantes providers e o sistema completo mantêm aceitação própria. A app .30
permanece intacta; commits/pushes continuam suspensos.

**Incremento801 — reconciliação de leitura e ausência nativa:** a consulta
vincula pet/objetivo/revisão e reutiliza os recibos assinados de modelo,
planeamento, ferramentas, receitas, verificação e conclusão. Expõe consumo e
resultados desconhecidos sem alterar registos ou inferir sucesso a partir de
settled. A origem da ausência é autenticada e o flock antigo fica retido durante
a consulta; processo vivo, origem trocada ou inode substituído recusam a prova.
O saldo tem âmbito explícito da última admissão. Dois crashes/reinícios reais
na app privada conservaram ficheiros, reconheceram a resposta com recibo e
mantiveram streaming incerto com 2048 bytes reservados. O ciclo completo passou
e a consulta distinguiu conclusão de Stop desconhecido. Seguem revisão explícita,
fecho aprovado/ACK e nova admissão; a retoma após perda do dono continua aberta.

**Incremento796 — controlo de preparações e cancelamento terminal:** uma
preparação original ainda sem ativação pode fechar após Pausar/Cancelar ou
expirar, incluindo quando só o ledger recebeu o claim. O backend limpa entradas
nativas abandonadas sem novo pedido/inferência e só liberta a reserva após ACK.
Cancel em fecho parcial conserva data, dono e uso anterior; uma revogação ainda
sem projeção aguarda essa projeção. Outro dono nativo continua operacional quando
o primeiro fecha durante cedência. Onze testes novos e o percurso privado da app
passaram; este último confirmou Pausar/Cancelar/expirar sem LeaseClaimed,
workers, modelo ou ações, seguido de um claim independente. A app .30 permanece
intacta. Segue recuperação após perda real do processo original; o sistema
completo e a entrega continuam abertos.

**Incremento793 — fecho original retido e claim concorrente:** a prova nativa
sobrevive a contenção/publicação incerta; `closing` repete só fecho/ACK e conserva
pedido, prazo, orçamento e uso desconhecido. Após cinco segundos pode repetir-se
explicitamente só o fecho, sem reconstruir um controlador ou iniciar inferência.
Leituras da fila coexistem e Busy deixa de ser conflito falso. A app privada
passou o ciclo com claim concorrente e Stop sob dois locks reais/projeção
atrasada: objetivo pausado, grant fechado e tentativa started ainda desconhecida,
sem novos efeitos. As falhas R29B/C estão preservadas e estes percursos passaram
na fonte793. A aceitação global/entrega continua aberta; seguem os casos de
controlo ainda não abrangidos, antes da recuperação de dono perdido.

**Incremento790 — confirmação de admissão retida:** o mesmo objeto nativo e a
mesma autoridade sobrevivem a `Busy`, indisponibilidade e ACK incerto. Repetir
não renova consentimento/orçamento; se o evento já foi publicado, só confirma o
recibo, mesmo após expiração. O frontend conserva o token e não repete escritas
automaticamente. O recibo anterior permite nova revisão após Pausar/ResumeDraft,
mantendo o token antigo limitado ao seu ACK. A app privada real confirmou dois
bloqueios e uma só admissão, prova parcial de justiça entre objetivos e ACK expirado sem novas
chamadas/ações. O Stop da fixture conjunta revelou fecho incompleto perante
StorageUnavailable; essa aceitação continua reprovada e é a próxima correção,
antes da reconciliação/retoma após perda do dono original. O claim concorrente
de outro objetivo também foi recusado com Conflict em R29C; a causa exige
reprodução/classificação e a aceitação global de execução permanece aberta.

**Incremento787 — identificação nativa do dono:** novos claims guardam identidade
assinada do processo e do perfil, com bloqueio real retido por workers. A consulta
separa processo vivo, ausente, divergente, indisponível e legado desconhecido,
sem conceder execução/recuperação. Dois crashes/reinícios na app privada real
conservaram registos e recusaram aprovações/controladores antigos, incluindo
streaming interrompido. A retoma efetiva ainda exige reconciliação e revisão.
O ensaio paralelo descobriu perda do token de admissão quando a certificação
recebe `Busy`; a correção está no incremento790, antes de prosseguir essa retoma.

**Incremento783 — cedência e retoma durante aprovação:** o executor conserva o
mesmo dono/controlador/lease, receita, prazo e orçamento, mas cede a reserva física
depois da transição assinada confirmada e de provar zero workers. Outro objetivo
pode reservar esse provider. Se a aprovação chegar enquanto o recurso está
ocupado, o executor conserva a decisão original e apresenta `waitingResource`;
requer nova geração física e ACK antes de executar. Falhas de ACK conservam o
estado nativo e reconciliam a mesma transição. Stop não cancela nem liberta a
reserva de outro dono; provas antigas não libertam a geração readquirida.

Seis testes novos e o runtime completo passaram. No Tauri real privado, outro
objetivo reservou o recurso enquanto o primeiro esperava; este conservou a
aprovação, retomou depois da libertação, executou/replaneou e concluiu. O registo
schema5 confirmou duas cedências/retomas, a mesma lease/prazo e duas chamadas de
modelo/quatro ferramentas. Modelo/leitor simulados, sem quota real. Não se
comprovou execução de inferência pelo segundo objetivo nessa fixture da app,
prioridade geral da fila ou retoma após perder o dono; a identificação desse processo está no incremento787.

**Incremento779 — proteção do comportamento existente:** três testes usam o
registo real de capacidades, consentimento/decisão nativos e leitor privado.
Confirmam que expirar metadados do modelo não invalida uma receita ou operação
independente do leitor; novas chamadas ao modelo exigem observação fresca.
Stop, revogação e substituição do leitor continuam a bloquear o percurso.
Este incremento acrescenta prova e não altera o comportamento de produção.

**Correção integrada — aprovações concorrentes:** uma disputa transitória do
registo de provas deixa de encerrar a receita pendente ou aparecer como registo
inválido. O pedido conserva o mesmo âmbito/prazo; aceitar e consumir a resposta
exigem validação atual. As leituras de provas partilham o bloqueio; alterações
continuam exclusivas. Cinco regressões adicionais cobrem bloqueios reais,
configuração indisponível, revogação, Stop e revalidação da mesma resposta.

**Aceitação Tauri atual — ciclo nativo passou:** a aplicação custom-protocol,
num container do projeto com Xvfb, D-Bus/perfil/leitor/modelo privados, percorreu
criação/contexto/admissão/claim, duas propostas e aprovações, quatro operações e
conclusão verificada. O resultado inicial com volume incorreto gerou feedback
nativo e nova proposta dentro do saldo original. Recarregar a vista conservou
o mesmo dono/pedido; Stop durante HTTP terminou sem novos efeitos e a referência
antiga foi recusada. Foram recebidos 14 eventos nativos. A prova usa comandos
Tauri reais; não comprova modelos/leitores reais, botões/formulários completos,
fecho da janela, voz, migração ou entrega. A instalação .30 e o ecrã do
utilizador não foram usados por esta instância privada.

**Incremento778 — execução e replaneamento nativos:** o ciclo mantém o dono e
orçamento originais, pede aprovação da receita exata e usa a observação final
assinada para concluir ou replanear. Conserva as aprovações em falhas de ACK,
respeita Stop durante o HTTP e liberta o recurso só após IO terminado. A admissão
usa configuração/modelo/política da mesma projeção nativa. Os controlos mínimos
permitem ativar e consultar o executor; a fábrica declara o suporte limitado de
Ollama local e das três operações do leitor Spotify. Código integrado na raiz e
SDK não significa aceitação real da app.

**Correção de paridade Codex integrada após o ciclo778:** a opção explícita de
acesso total só é enviada ao criar uma sessão própria. Retomar uma sessão
existente conserva a política nativa atual, incluindo alterações entretanto
feitas no terminal, e o nome original. O teste de protocolo cobre criação e
retoma com a opção ligada/desligada, aprovação e stream preservados. Passaram
80 testes RPC, a biblioteca completa e 416 ensaios privados no anfitrião; o
número de fontes/testes completos não aumentou, porque foi ampliada uma
regressão existente. Isto não fecha a paridade real de todos os adaptadores.

**Aceitação anterior da app — prova parcial conservada:** a build custom-protocol das mesmas 778 fontes
arrancou numa sandbox com perfil/D-Bus/leitor/modelo privados, HOME real oculto,
rede limitada a loopback e sem atalhos globais. A área de objetivos foi observada
na app .31. Isto comprova arranque e apresentação inicial; ainda não comprova o
percurso completo de autorização/execução/replaneamento pela interface. A
acessibilidade da sandbox está incompleta; a instância foi encerrada sem usar
providers ou quota reais; o restante ensaio gráfico continua pendente. O perfil da instalação .30 permanece separado.

**A seguir, na mesma unidade RUN/EXE:** recuperação ao perder o dono nativo,
prioridades/equidade geral da fila, continuidade com janela fechada e expansão
dos adaptadores/operadores/verificadores. A cedência durante aprovação já tem
o âmbito privado descrito acima. Voz integral,
memória/fontes, rotinas, cooperação, matriz de Sistema e entrega mantêm o seu
âmbito no roadmap. A app instalada permanece na **.30**; commits e pushes
continuam suspensos. Não foram usados modelos/quota reais nem alterados perfis,
Wi-Fi ou PostgreSQL. O sistema completo continua por concluir.

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

A composição preserva as decisões partilhadas de receita com origem própria,
os critérios tipados de reprodução/volume e a conclusão schema7 autenticada.
O primeiro writer é Ollama local e o executor/verificador físico continua nas
operações Spotify suportadas. Aprovar, executar passos e concluir o objetivo
continuam fases distintas; texto do modelo não verifica critérios.

Migração, backup, rollback e aceitação da entrega continuam necessários antes
de instalar. Prova guardada antes da primeira publicação não substitui um
controlador válido; lease guardado não prova que o worker terminou. Providers
reais, paridade com terminais, fontes/memória, rotinas, cooperação, voz integral
e matriz de Sistema continuam no âmbito. A estimativa global mantém-se em cerca
de 40/100, aproximada e sem substituir aceitação por requisito.

## 1. Autoridades e versões

| Estado | Evidência e limite |
| --- | --- |
| Checkout de trabalho | `/home/pmglo/Projects/coucou-codex`, branch `feat/mochi-cli-accounts`; último publicado `bbe12e2`, com alterações locais não commitadas e publicação suspensa |
| Composição atual integrada | 825 fontes; revisão explícita de novo trabalho de estado com incerteza antiga conservada; 2552 Rust/21 ignorados, 1183 JS e 542 ensaios privados; [provas e limites](PETS-VALIDATION.md#incerteza-local825) |
| Recuperação823 anterior | 823 fontes; preserva prefixo/encargos selados e leitura do objetivo; 2549 Rust/21 ignorados, 1180 JS e 539 ensaios privados; [âmbito histórico](PETS-VALIDATION.md#recuperacao-local823) |
| Execução778 anterior | 778 fontes; ciclo nativo limitado, feedback/replaneamento, aprovação exata e libertação comprovada; 2426 Rust/20 ignorados, 1155 JS e 416 ensaios privados; [provas e limites históricos](PETS-VALIDATION.md#retoma-nativa-codex-local778) |
| Decisões754 anteriores | 754 fontes; primeira origem própria e decisão partilhada em chat/voz determinística, 2365 Rust/20 ignorados, 1138 JS e 269 ensaios privados; [checkpoint](PETS-VALIDATION.md#evidencia-local) |
| Critérios752 anteriores | 752 fontes; prova nativa e conclusão schema7, 2359 Rust/20 ignorados, 1135 JS e 263 ensaios privados; [checkpoint](PETS-VALIDATION.md#evidencia-local) |
| Receitas749 anteriores | 749 fontes; primeiro executor limitado de receitas revistas, 2342 Rust/19 ignorados, 1133 JS e 246 ensaios privados de runtime/leitor/operações no anfitrião; [checkpoint parcial da unidade](PETS-VALIDATION.md#evidencia-local) |
| Recuperação744 anterior | 744 fontes; prova nativa durável e recuperação explícita de passos, 2330 Rust/18 ignorados, 1131 JS e 234 ensaios privados de runtime/leitor/operações no anfitrião; [checkpoint](PETS-VALIDATION.md#evidencia-local) |
| Sessões742 anteriores | 742 fontes; unidade completa revista de sessões/permissões/decisões e callbacks, 2314 Rust/17 ignorados, 1129 JS e 367 ensaios privados no anfitrião; [checkpoint](PETS-VALIDATION.md#evidencia-local) |
| Composição718 anterior | 718 fontes; contrato de donos/adaptadores de objetivos pela fábrica, assistentes opcionais e recusa nativa de donos retirados/sem executor. 2141 Rust/17 ignorados, 1109 JS e 181 ensaios privados do runtime no host; [checkpoint](PETS-VALIDATION.md#evidencia-local) |
| Passos717 | Primeiro percurso de execução Spotify com concessão própria, orçamento, recurso e recibos/observações duráveis; mantido na composição718. [Checkpoint](PETS-VALIDATION.md#evidencia-local) |
| Planos710 | Propostas tipadas e revisão/consentimento vinculados ao conteúdo exato; preservados na composição717. 2106 Rust/16 ignorados e 1107 JS na validação original; [checkpoint](PETS-VALIDATION.md#evidencia-local) |
| Broker707 | Cinco operações interativas, validação/owner/grants/cancelamento e entrega tipada, mantidos na composição710; [checkpoint](PETS-VALIDATION.md#evidencia-local) |
| Recibos do executor702 | Resultados textuais duráveis e consulta após pausa/reinício. Mantidos na composição707; [checkpoint](PETS-VALIDATION.md#evidencia-local) |
| Integração inicial698 | 180 ficheiros atualizados com pré-imagens conservadas; incrementos seguintes preservam essa base. [Reconciliação](PETS-VALIDATION.md#evidencia-local) |
| Aplicação observada | `app-Coucou@autostart.service` ativo, PID `4091547`; `/proc/4091547/exe` resolve para `~/.local/opt/coucou/0.5.1-mochi.30/usr/bin/coucou` |
| Entrega .30 | 597 fontes congeladas; 1541 Rust/12 ignorados, 1017 JS e verificações de composição anteriores; [evidência da entrega](PETS-VALIDATION.md#evidencia-local) |
| Candidata .31 | 600 fontes congeladas; catálogo ICS, 1566 Rust/12 ignorados e composição aprovada; RPM `db9a934f4008…`, não instalado; [continuação](PETS-VALIDATION.md#evidencia-local) |
| Protótipo de anexos/retenção | 633 fontes privadas; 1725 Rust/12 ignorados, 1052 JS, cargo check, TypeScript e Vite aprovados; não integrado na aplicação instalada; [checkpoint](PETS-VALIDATION.md#evidencia-local) |
| Base PET-01 de sessões | Unidade separada da base633, com 640 fontes; 1754 Rust/12 ignorados, 1060 JS, check/TSC/Vite aprovados; registo, observações por alvo e adoção inicial no chat comum; [checkpoint](PETS-VALIDATION.md#evidencia-local) |
| Base PET-01 de voz/gestão | Unidade derivada da base640, com 644 fontes; 1768 Rust/12 ignorados, 1070 JS, check/TSC/Vite aprovados; preparação da voz e gestão usam a identidade comum; [checkpoint](PETS-VALIDATION.md#evidencia-local) |
| RUN-01 em desenvolvimento | Repositório derivado da base644, com 650 fontes; 1792 Rust/12 ignorados, 1074 JS, check/TSC/Vite aprovados; objetivos/tarefas/planos/passos propostos e eventos persistidos por objetivo; integração e execução pendentes; [checkpoint](PETS-VALIDATION.md#evidencia-local) |
| PET-02/RUN-02 — contextos e reservas | Unidade derivada da base650, com 653 fontes; 1811 Rust/12 ignorados, 1078 JS, check/TSC/Vite aprovados; contextos privados por objetivo, reservas físicas partilhadas e correção do censo CLI; fila/execução e integração pendentes; [checkpoint](PETS-VALIDATION.md#evidencia-local) |
| Fábrica — metadados nativos | Unidade derivada da base653, com 654 fontes; 1818 Rust/12 ignorados, 1082 JS, check/TSC/Vite aprovados; identidades, serviços, classificação/binding e preparação de voz usam o catálogo tipado; integração e contratos comportamentais completos pendentes; [checkpoint](PETS-VALIDATION.md#evidencia-local) |
| RUN-02 — revisão e fila | Unidade derivada da base654, com 657 fontes; 1833 Rust/12 ignorados, 1084 JS, check/TSC/Vite aprovados; revisão nativa exata e histórico de admissão `queued` por objetivo; proveniência recuperável, leases, dispatch e integração pendentes; [checkpoint](PETS-VALIDATION.md#evidencia-local) |
| RUN-02 — proveniência de consentimento | Unidade derivada da base657, com 660 fontes; 1854 Rust/12 ignorados, 1085 JS, check/TSC/Vite aprovados; prova HMAC nativa e ledger independente de revogação; claims/workers e execução pendentes; [checkpoint](PETS-VALIDATION.md#evidencia-local) |
| RUN-02 — claims e vida dos workers | Unidade derivada da base660, com 664 fontes; 1872 Rust/12 ignorados, 1087 JS, check/TSC/Vite aprovados; posse autenticada, `leased/stopping`, guards físicos e drenagem conservadora; writer/dispatch e aceitação pendentes; [checkpoint](PETS-VALIDATION.md#evidencia-local) |
| OBS-01 — estados por fonte | Unidade derivada da base664, com 666 fontes; 1887 Rust/12 ignorados, 1101 JS, check/TSC/Vite aprovados; processos/TTYs e conversas carregadas `idle/reserved/busy/waiting/stopping/unknown` com âmbitos separados; não instalado; [checkpoint](PETS-VALIDATION.md#evidencia-local) |
| RUN/EXE — writer com orçamento | Unidade derivada da base666, com 671 fontes; 1916 Rust/12 ignorados, 1101 JS, check/TSC/Vite aprovados, sem novos avisos; ledger de tentativas schema3, hold partilhado e primeira resposta textual nativa Ollama validada com HTTP simulado; sem instalação nem fecho de critérios; [checkpoint](PETS-VALIDATION.md#evidencia-local) |
| RUN/EXE — prontidão e IO em curso | Unidade derivada da base671, com 673 fontes; 1928 Rust/12 ignorados, 1101 JS, check/TSC/Vite aprovados; prontidão fresca antes de efeitos separada da validade de política/posse durante IO, captura de configuração coerente e metadados locais genuínos; [checkpoint](PETS-VALIDATION.md#evidencia-local) |
| CTX-01 — prova terminal e limpeza de metadados | Unidade derivada da base673, com 675 fontes; 1946 Rust/12 ignorados, 1101 JS, check/TSC/Vite e nenhum aviso novo; envelope durável antes do WAL final, descoberta com WAL/pasta ausentes e revisão nativa de metadados apenas; restante CTX-01/aceitação/entrega pendente; [checkpoint](PETS-VALIDATION.md#evidencia-local) |
| CTX-01 — aceitação de recuperação nativa | Segundo incremento sobre as 675 fontes; 1954 Rust/12 ignorados, 1101 JS, check/TSC/Vite e nenhum aviso novo; 8 regressões de restauro parcial, destinos ocupados, cleanup recursivo e confirmação perdida; motivos de recusa precisos e seguros; migração/rollback/entrega pendentes; [checkpoint](PETS-VALIDATION.md#evidencia-local) |
| DATA/REL — admissão de migração | 683 fontes privadas; 1966 Rust/12 ignorados, 1101 JS, check/TSC/Vite sem avisos novos. 12 regressões novas: leitor/sweep .30 exatos, revisão só de leitura e layout protegido. Cópia, barreira de arranque, ativação e rollback continuam pendentes; caminhos de produção intactos; [checkpoint](PETS-VALIDATION.md#evidencia-local) |
| DATA/REL — cópia privada de perfis | 686 fontes privadas; 1980 Rust/12 ignorados, 1101 JS, check/TSC/Vite sem avisos novos. 14 regressões novas: cópia privada e revalidação, leitura antes de concluir, interrupções, adulteração e identidade de montagem. Sem publicação/ativação nem migração real; [checkpoint](PETS-VALIDATION.md#evidencia-local) |
| DATA/REL — posse nativa do arranque | Componente privado688 fontes; 1992 Rust/13 ignorados (12 de host e1 helper executado explicitamente), 1101 JS e check/TSC/Vite sem avisos novos. 12 regressões com bus privado real/outro PID e lock do perfil. Publicação/barreira e ligação antes de Settings/Chat/Recap ainda pendentes; [checkpoint](PETS-VALIDATION.md#evidencia-local) |
| DATA/REL — publicação dos dois perfis | Núcleo privado690 fontes; 2009 Rust/13 ignorados, 1101 JS, check/TSC/Vite sem avisos novos. 17 regressões novas: destinos inativos, cópia/validação, conclusão/admissão atómica, recuperação de cópias completas e dois filesystems reais. Cópias parciais/arranque/caminhos da app/entrega pendentes; [checkpoint](PETS-VALIDATION.md#evidencia-local) |
| Arquitetura atual | Reorganizada nesta revisão: contratos de autonomia, plano e rastreabilidade. A revisão documental não executou modelos nem testes de implementação |

A observação do serviço/executável pertence à revisão de arquitetura de 23/10.
As provas .30/.31/633/640/644/650/653/654/657/660/664/666/671/673 são anteriores; as verificações dos incrementos675 de recuperação,683 de admissão,686 de
cópia privada,688 de posse nativa e690 de publicação foram executadas posteriormente, nesta sessão. O conteúdo privado das
conversas/documentos não foi inventariado.

O checkout canónico `coucou` noutra branch não contém necessariamente estes
documentos. A última base congelada validada está em
`.build/pet-goal-execution-next-r1/admission-refresh-source`, e as suas fontes estão agora adotadas
na raiz deste projeto. O executável instalado continua num estado anterior.
As linhas de incrementos acima e as secções históricas abaixo registam o âmbito
à data de cada ensaio; as pendências correntes são as da secção 2.

A unidade671 liga o orçamento a uma resposta textual Ollama ao intento original,
com contexto privado, identidade da fábrica e cancelamento durante HTTP. Ainda
não executa planos/ferramentas ou verifica critérios; CLI requer controlo nativo
dos limites internos. A unidade673 separou prontidão fresca da validade do IO em curso, preservando
Stop e alterações de política/configuração. A unidade675 resolveu a prova terminal
perdida após remoção do WAL final e a recuperação da limpeza de metadados. O incremento675 seguinte comprovou o restauro parcial e cleanup nativos,
melhorando as mensagens de recusa. CTX-01 continua necessário para composição/
entrega: compatibilidade com a .30, backup/migração/rollback e os restantes
critérios de integração.

## 2. O que existe e o que falta para autonomia

### 2.1 Plano de backend e posição atual

Os grupos abaixo resumem os 43 requisitos do roadmap; não são percentagens de
conclusão nem uma substituição dos critérios individuais. Voz integral e paridade
de permissões são requisitos explícitos, ainda parcialmente implementados.

| Grupo | Estado comprovado e desenvolvimento seguinte |
| --- | --- |
| Arquitetura, fábrica e adaptadores | Catálogo, identidades e validação de donos/assistentes opcionais integrados e verificados; concluir contratos comportamentais e aceitação de todos os adaptadores, incluindo Gravity e Copilot CLI; preservar as ferramentas/capacidades das sessões originais, distinguindo esse percurso do dispatcher autónomo limitado |
| Sessões e permissões nativas | Contextos/origem/decisões e censo distintos já têm bases; origem comum verificada em Codex gerido/externo, Copilot ACP e gate Gravity; escolhas modernas Codex/ACP verificadas; IO capturado protegido; formulários MCP tipados/adoção externa verificados no subconjunto suportado; vida/resposta de callbacks de operações verificada no broker; concluir aceitação de continuidade/política/cobertura por adaptador e política/sandbox/aprovações do agente original com decisão única entre terminal, chat e voz — PET-02/03, EXE-02 |
| Execução autónoma e verificação — atual | Ciclo limitado de proposta/publicação/aprovação/execução/verificação/replaneamento integrado, com contexto próprio, orçamento comum, feedback assinado e controlador original independente da vista. Suporte limitado Ollama/LM Studio locais e Spotify; preservar aprovação em falhas de ACK e libertar recursos só com drain nativo confirmado. Concluir aceitação Tauri completa, equidade durante aprovação, recuperação sem dono vivo e restantes adaptadores/operadores/verificadores — RUN/EXE |
| Controlo integral por voz | Ampliar as rotas existentes a todas as operações disponíveis do backend, incluindo tarefas/regras/configuração e Sistema autorizado; interrupção de TTS/Stop, decisões nativas, cobertura e áudio real — VOICE-01/02 |
| Memória, documentos e fontes | Anexos/retenção e ICS têm bases integradas no código; fechar aceitação CTX-01, memória estruturada/seletiva, concessões por objetivo/regra, várias fontes e RSS/IMAP — CTX/MEM |
| Rotinas e cooperação | Desenvolver regras/gatilhos/agendamento/deduplicação, preparação diária, notificações e delegação com dono, orçamento e recibos próprios — AUTO/TEAM |
| Sistema e integrações | Desenvolver a matriz pet/subplugin/operação/recurso e ligação ao executor; Rede/Bluetooth numa unidade, Som e restantes componentes com concessões partilháveis; serviços/comunicação com operações distintas — SYS/INT |
| Estado, consumo e recuperação | Ampliar fontes/estados/consumo por objetivo e provider, decisões/eventos entre canais, quotas sem inferir zero, reconciliação após falha e revogação, backups/rotação e controlo de concorrência — OBS, RUN-04/05, DATA-01 |
| Integração e entrega | Resolver preservação/CTX-01, migrações e rollback dos novos schemas, provar aceitação nativa/host/áudio e instalar composição validada — REL-01/02; PostgreSQL permanece opcional e sem integração — DATA-02 |

As verificações de recuperação, consumo, concorrência e preservação acompanham
as unidades, mesmo quando aparecem como grupo próprio. Sistema mantém prioridade
posterior aos pets. A aplicação em execução continua na .30; o código integrado
não demonstra entrega nem paridade integral com os terminais.


### Posição global comunicada ao utilizador

Estimativa global reafirmada em 24/10: **cerca de 40/100** para o objetivo completo,
considerando implementação, integração e aceitação real. É uma avaliação
aproximada do conjunto; os 43 requisitos mantêm os seus critérios próprios.
Há incrementos verificados agora integrados no checkout e funcionalidades antigas
instaladas, com uma parte relevante das melhorias ainda sem entrega. Falta ampliar e completar o ciclo autónomo de
planos/ferramentas/verificação além do primeiro executor Spotify, paridade nativa dos adaptadores e permissões,
memória/fontes, rotinas/cooperação, voz integral, matriz de Sistema e entrega
com migração/backup/rollback/aceitação. Publicação, retoma, backup/restauro legado
e recibos de respostas já têm provas nativas; as fontes foram integradas no
checkout. Seguem a aceitação isolada do ciclo, equidade/recuperação, expansão de ferramentas/adaptadores e entrega.

### 2.2 Bases existentes e lacunas por capacidade

| Unidade | Base existente | Lacuna e ID de continuação |
| --- | --- | --- |
| Identidades e fábrica | `pet_factory` com moldes/herança, retirada separada e metadados de runtime tipados desde a unidade654, integrados na759; lookup nativo de identidade/serviços, classificação/binding e preparação de voz usam o catálogo | PET-01/04/05 e ARC-02: completar contratos comportamentais/adaptadores/controladores, modelos e aceitação; migrações preservam overrides e a fábrica não concede execução |
| Conversas | Biblioteca, arquivos, IDs, origem e tarefas por pet; gravação e cancelamento com dono | PET-02, RUN-01/02: associação de objetivos e conversas de background sem trocar a vista corrente |
| Sessões CLI | Composição742 com origem/decisões/formulários/IO comuns, configuração ACP preservada e retoma Gravity validada; processos e sockets privados verificados | PET-03: completar continuidade e controlo/decisões nas capacidades nativas, sobretudo Gravity; uma nova sessão não prova retoma da TUI original |
| Modelos locais | Loop Ollama Ask, propostas congeladas, aprovação por comando, prazos/limites e recibos | PET-04, RUN-03: contrato comum de objetivo/progresso; modelo sem tools não recebe execução; não assumir a mesma capacidade no LM Studio/API |
| Trabalho persistido | Repositório por objetivo integrado, separado do journal antigo; propostas, revisão/consentimento autenticado, claims/leases, orçamento comum, recibos, execução de receitas, critérios nativos e primeiro ciclo limitado de planeamento/replaneamento com feedback assinado | RUN-01/02/03/04: completar adoção/migração, execução nos restantes adaptadores, expansão de operadores/critérios, cedência durante aprovação, recuperação sem dono vivo e aceitação/entrega; a existência dos ledgers não prova autonomia completa |
| Memória | `assistant_memory` persiste biblioteca/histórico e arquivos | MEM-01/02: memória estruturada, proveniência, âmbito, validade, correção e recuperação seletiva; o nome do módulo não prova estas capacidades |
| Calendários/fontes | Leitor ICS com recorrências/fusos, consultas dia/intervalo e catálogo persistente na .31 | CTX-03/04: gestão/aceitação nativa, concessões de objetivo/regra e vários calendários persistentes; o schema atual inscreve por pet/CID e limita uma fonte ativa por owner |
| Regras/proatividade | Contexto/consultas existentes são bases reutilizáveis | AUTO-01/05: repositório de regras, gatilhos, ocorrências deduplicadas, agendador, orçamento e notificações com política própria |
| Cooperação | IDs/origens e executores individuais são bases | TEAM-01/02: delegação durável com autoridade, orçamento e verificação; não há prova do contrato completo |
| Sistema | Transporte/atores partilhados e operações de rede/Bluetooth/som já desenvolvidos, sete widgets conservados | SYS-01/03: matriz por pet/subplugin/operação e expansão funcional; não tratar uma leitura ou ACK como aceitação do efeito físico |
| Consumo | Ledger local de tentativas e leitores/caches nativos com âmbitos distintos | OBS-01/02: cobertura/rotação/consumo por objetivo e todos os providers; desconhecido não é zero |
| Voz | Identidade/conversa e percursos de chat/ações/calendário | VOICE-01/02: controlo integral por voz das operações de pets/sessões/objetivos/fontes/configuração/Sistema autorizado; paridade das decisões nativas; microfone/Bluetooth/TTS PT/EN/misto reais |
| Dados e entrega | Prefixos versionados, rollback, fontes/artefactos e autostart | DATA/REL: migrações e recuperação do motor novo, aceitação integrada e resolução de bloqueadores antes de ativação |

Não foi identificado no código revisto um motor que satisfaça o contrato
integral de objetivos/regras/agendador definido na arquitetura. Os loops locais,
estados de tarefas e ferramentas existentes devem ser integrados nessa unidade,
conservando as suas proteções e sem anunciar antecipadamente autonomia completa.

## 3. Anexos e retenção — estado exato

### 3.1 Aplicação .30 instalada

A seleção de anexos é singular. O histórico Coucou fica em
`~/.local/share/coucou/assistant/conversations.json`, separado logicamente por
pet/CID. As cópias ficam em `~/.local/share/coucou/inbox/`; ferramentas CLI podem
ter histórico nativo próprio. PostgreSQL não participa nesta persistência.

A associação da seleção/grants está em RAM; não existe nessa versão o manifesto
persistente por CID do protótipo. A ingestão reserva nomes novos, pelo que a
substituição reportada é da seleção no chat, não da cópia anterior no disco.
A limpeza antiga de sete dias pode remover cópias ao voltar a correr; não se
pode apresentar os anexos da .30 como retidos permanentemente.

### 3.2 Protótipo privado de 633 fontes

Implementado e verificado com dados sintéticos:

- Coleção ordenada por pet/CID, IDs estáveis, lote nativo, adição e remoção
  individuais; 32 anexos/100 MiB por conversa, reservas independentes, prazo
  original e quatro vagas físicas de IO até terminar o worker.
- Captura imutável para API/CLI/local/voz; permissões, IDs, cópias e contexto
  originais revalidados. Limites agregados, cobertura parcial e capacidades
  explícitas para ficheiros, pastas, imagens e PDF.
- Vários ICS com seleção explícita, IDs/proveniência próprios, limites comuns e
  coexistência com anexos comuns. Não há inscrição automática de fontes.
- Manifestos privados por CID e referências dos anexos por turno. O journal usa
  v5 quando contém essas referências e mantém leitura de v1–v4; builds antigas
  recusam v5. Não houve migração dos dados instalados.
- Vigilância das pastas por inotify e verificações de árvore/SHA256 nos limites
  de IO/commit; eventos perdidos ou alterações revogam o pedido capturado.
  Não é uma transação atómica do filesystem contra escritores externos.
- Remoção de histórico local por conversa com revisão/consentimento e recuperação,
  conservando o historial nativo do provider e as cópias por essa operação.
- Purga de cópias registadas com catálogo de todos os CIDs, referências/atividade,
  seleção/entregas pendentes, permissões exatas e recusa de autorizações estrangeiras.
- Quarentena no mesmo filesystem, WAL privado, movimentação sem substituição,
  recuperação/retoma e restauro com nome novo quando o anterior foi reutilizado.
  Cópias parcialmente eliminadas não são apresentadas como recuperáveis na íntegra.
- Comandos nativos e controlos funcionais de revisão/confirmação/recuperação;
  tokens de uso único por janela/pet/CID, 120 segundos e cache limitado.
- Proteção conservadora de cópias antigas sem registo, partilhadas, referenciadas
  ou desconhecidas. A antiga limpeza automática foi retirada neste protótipo.

**Pendências observadas na base633 (estado histórico):** os pontos 1–3 abaixo
foram tratados nos incrementos675; os pontos 4–5 continuam sujeitos à aceitação
e aos limites documentados nas secções mais recentes.

1. Persistir um envelope de conclusão separado antes da limpeza final do WAL,
   para recuperar a interrupção entre apagar esse WAL e remover o diretório.
   O intervalo atual pode deixar metadados indisponíveis; ocorre depois de as
   cópias estarem eliminadas ou restauradas.
2. Completar os ensaios do coordenador para restauro parcial com nome entretanto
   ocupado e limpeza recursiva interrompida; há ensaios dessas situações no núcleo.
3. Preservar o motivo específico da recusa no recibo incompleto e remover dois
   helpers sem uso. Cargo check tem 51 avisos, incluindo esses dois além da base49.
4. Aceitar os percursos UI/OS nativos e a composição, com dados sintéticos e
   sem consumir quota Copilot. Verificar assets de som ausentes no snapshot Vite.
5. Backup fresco de biblioteca/inbox/recuperações, revisão v4/v5 e resolução de
   marcadores pendentes antes de rollback. Só depois adotar/empacotar/ativar.

Provas e continuação:

- [Checkpoint de 633 fontes](PETS-VALIDATION.md#evidencia-local).
- [Mapa exato de fontes](PETS-VALIDATION.md#evidencia-local).
- [Contrato da retenção](PETS-VALIDATION.md#evidencia-local).
- [Continuação do trabalho](PETS-VALIDATION.md#evidencia-local).

Os 1725 testes Rust/12 ignorados e 1052 JS, check/TSC/Vite usam os mesmos
288 inputs compilados e 648 inputs de fonte web, sem alterações durante as
verificações. Os percursos sintéticos de oito providers não provam oito pets
configurados nem inferência/aceitação nativa em cada conta. Falhas de publicação
foram simuladas; não foi executado um ensaio real de corte de energia.

## 4. Mapa entre arquitetura e código revisto

Os links deste quadro apontam ao **SDK privado revisto**. Só constituem prova
do comportamento descrito, não de entrega no executável instalado.

| Requisito | Fonte inicial de evidência |
| --- | --- |
| PET-01/04/05 | [Fábrica](PETS-VALIDATION.md#evidencia-local), [configuração](PETS-VALIDATION.md#evidencia-local), [defaults](PETS-VALIDATION.md#evidencia-local) |
| PET-01 — adoção atual | [Registo comum](PETS-VALIDATION.md#evidencia-local), [preparação da voz](PETS-VALIDATION.md#evidencia-local), [metadados da gestão](PETS-VALIDATION.md#evidencia-local) |
| RUN-01 — primeiro kernel | [Modelo e validação](PETS-VALIDATION.md#evidencia-local), [repositório](PETS-VALIDATION.md#evidencia-local), [comandos nativos](PETS-VALIDATION.md#evidencia-local) |
| PET-02/03, RUN-01/04 | [Chat](PETS-VALIDATION.md#evidencia-local), [tarefas existentes](PETS-VALIDATION.md#evidencia-local), [sessões](PETS-VALIDATION.md#evidencia-local) |
| PET-03 | [Codex](PETS-VALIDATION.md#evidencia-local), [Gravity](PETS-VALIDATION.md#evidencia-local), [Copilot](PETS-VALIDATION.md#evidencia-local) |
| RUN-03, EXE-01/03 | [Loop local](PETS-VALIDATION.md#evidencia-local), [tipos/recibos](PETS-VALIDATION.md#evidencia-local), [executor](PETS-VALIDATION.md#evidencia-local) |
| CTX-01/02, DATA-01 | [Documentos](PETS-VALIDATION.md#evidencia-local), [retenção nativa](PETS-VALIDATION.md#evidencia-local), [quarentena](PETS-VALIDATION.md#evidencia-local), [biblioteca](PETS-VALIDATION.md#evidencia-local) |
| CTX-03/04/05 | [Fontes](PETS-VALIDATION.md#evidencia-local), [schema atual pet/CID](PETS-VALIDATION.md#evidencia-local), [avaliação de reutilização](AI-LOCAL-REUSE.md) |
| OBS-01/02 | [Ledger](PETS-VALIDATION.md#evidencia-local), [inventário de processos](PETS-VALIDATION.md#evidencia-local) |
| VOICE-01/02 | [Sessão de voz](PETS-VALIDATION.md#evidencia-local), [tarefas faladas](PETS-VALIDATION.md#evidencia-local) |

## 5. Próxima execução e limites desta revisão

A prioridade continua a arquitetura e os pets. O núcleo PET-01 permite avançar
o contrato de identidade/contexto PET-02 e o repositório/admissão RUN-01/02,
mantendo cobertura/aceitação PET-01 e continuidade PET-03 abertas.
O [contrato de capacidades v1](PET-CAPABILITIES-CONTRACT.md) já está desenhado e
tem agora um primeiro núcleo implementado na composição privada de 650 fontes. Este
núcleo cobre os 25 IDs, distingue suporte/disponibilidade/motivo e publica uma
revisão própria por pet/instância, sem expor endpoints privados. O comando nativo
de leitura e a bridge tipada usam configuração existente e observações dos
pedidos de modelos já explícitos. Não consultam contas ou modelos ao ler o estado.

Os primeiros 19 casos Rust novos cobrem identidade, retirada, modelos/endpoints/perfis,
revisões, privacidade, falta de configuração, datas/expiração, supersessão e
controlo de raciocínio do parser existente. O incremento seguinte acrescenta dez
casos Rust e oito JS: cache de 128 alvos, publicação ordenada, prazo de 45 s,
frescura de 15 s, invalidação e corridas de leitura/ação no chat comum. Codex e
os históricos autenticados de Copilot/Gravity alimentam a projeção; a consulta
passiva não liga a sockets. Os históricos Tauri Codex/Copilot incluem o descritor
do alvo; os consumidores mantêm as verificações nativas de execução.

Os três pets conservam provider, endpoints/credenciais, modelos, sessões,
decisões e memória independentes. A bridge reutiliza código de transporte local;
um teste usa o mesmo ID de sessão com modelos distintos e invalida apenas um pet.
Não foi introduzida delegação entre Gravity e Copilot.

O incremento de voz/gestão acrescenta 14 casos Rust e dez JS. A preparação da
voz confirma o pet/binding ativo e pré-requisitos locais; distingue presença do
executável de autenticação ou modelo pronto e revalida cada novo pedido falado.
Ollama usa o perfil próprio mesmo com outro provider no texto. A rota determinística
Spotify precede o aviso genérico de provider vazio e conserva a permissão musical.
Pedidos de estado/interrupção dirigidos por nome continuam possíveis após
desativação/retirada, mantendo os tokens originais.

A observabilidade da gestão recebe o mesmo descritor, com limites e identidade
validados. Falhas, invalidações e respostas ultrapassadas retiram metadados de
capacidades sem alterar contagens, cobertura ou datas originais dos inventários.

O primeiro kernel RUN-01 acrescenta 24 casos Rust e quatro JS. Guarda ficheiros
por objetivo, conserva intenção até 64 KiB sem truncação e mais de dez tarefas,
com planos/passos propostos e eventos. As gravações usam exclusão de escritores,
revisão original, idempotência e sincronização de ficheiro/diretório/pai; uma
publicação incerta devolve recibo e pode ser reconciliada sem duplicar propostas.
Referências/recibos de tarefas antigas ficam `recoveryRequired`, sem inferir
intenção completa, autorização ou conclusão. Os comandos nativos e bridge tipada
não expõem execução ou injeção de autoridade; `goals.run` permanece indisponível.
O repositório foi testado em diretórios temporários; faltam aceitação/integração,
migração, fila/leases, orçamento e execução/verificação. Nenhum worker arranca ao abrir.

A suite da unidade650 passou 1792 Rust, 1074 JS e check/TSC/Vite com 301 inputs
compilados e 665 de fonte web inalterados, sem novos avisos face à sua base de 51.

A unidade653 prepara conversas privadas por objetivo, dedicadas ou com cópia
explícita e limitada do histórico público. Não troca a conversa visível, nem
copia sessões nativas, tarefas, anexos ou permissões. Um coordenador comum ao
chat manual e aos objetivos conserva a reserva física até terminar o worker,
mesmo após cancelamento/prazo. Os callbacks nativos mantêm a origem original.
Gravações, retries e leitura existente validam dono, revisão, protocolo e locks;
substituir o marcador ou lock revoga escritas, conservando a reserva em curso.
Os registos com contexto ligado usam schema2; o leitor650 recusa-os, pelo que
migração, backups e rollback continuam a exigir aceitação antes da ativação.

A contagem CLI distingue TTYs, processos clientes, foreground e suspensão.
A observação privada do host encontrou três clientes Codex em dois TTYs, um
suspenso; nenhum processo foi alterado. Payloads antigos sem TTY são apresentados
como clientes. Esta correção ainda não foi instalada. A nova leitura do serviço e
`/proc` confirmou a mesma .30/PID4091547 e os mesmos dois TTYs, três clientes,
dois em foreground e um suspenso. Atividade AI/sessões externas exige evidência
própria; o contrato de estados está na arquitetura §15.1.

Esta unidade passou 1811 Rust/12 ignorados, 1078 JS e check/TSC/Vite, com 304 inputs
compilados e 668 de fonte web inalterados. Permanecem 51 avisos Rust: ao adotar
`codex_request_target`, o aviso de função sem uso passou dessa função para a
fachada antiga `codex_request`. Não houve inferência ou quota real. Preparação
de contexto não executa objetivos; autoridade/orçamento, fila durável,
dispatch e verificação permanecem pendentes.

A unidade654 acrescenta a primeira adoção nativa do catálogo tipado. O manifesto
bundled usa schema2; a configuração gravada continua em versão1, com os mesmos
25 defaults. Metadados/rotas explícitos recusam donos ambíguos, adaptadores
incompatíveis e herança de autoridade. Os serviços conservam o seu provider
lógico mesmo partilhando Codex. A classificação, binding e preparação de voz
usam a fábrica; o catálogo não instala um adaptador nem cria um controlador.
Os testes exigem controlador para cada rota de voz declarada. Permanecem tabelas
de controladores/implementações específicas e contratos comportamentais a
generalizar; APIs/LM Studio sem adaptador integrado não passam a executar.

A validação final tem 1818 Rust/12 ignorados (quatro testes em paralelo), 1082 JS,
check/TSC/Vite e os mesmos 51 avisos, sem novos. Os 305 inputs compilados e 669 de
fonte web coincidem entre checks e fontes atuais. Houve uma falha inicial num
helper de rede preexistente (`Unavailable` em vez de `Protocol`); passou isolado
e na suite final, sem alteração do código. Os resultados iniciais foram
preservados; a fiabilidade deste fixture sob concorrência alta fica por avaliar.
As fontes653 permanecem exatas. Nada desta unidade foi instalado.

A unidade657 guarda a revisão/admissão e histórico `queued` no mesmo envelope
schema3. Tokens nativos por janela/pet/objetivo/revisão têm uso único, capacidade
limitada e prazo original; alterações de modelo/política/contexto ou observações
sem prontidão/frescura recusam confirmação. Limites monetários/tokens são
recusados sem medição, e `localOnly` não aceita contas/adaptadores cloud.
Os dados reabertos e recibos incertos não concedem execução. Pausar/cancelar só
encerra propostas sem workers; trabalho com lease requer novos estados e
reconciliação antes de reutilizar esse controlo.

Passou 1833 Rust/12 ignorados, 1084 JS, check/TSC/Vite, com 308 inputs compilados,
672 de fonte web inalterados e 51 avisos sem novos. Os testes cobrem cinco donos
nativos/serviços com observações sintéticas, expiração, políticas, limites,
repetição de tokens, falhas antes/depois da publicação, pausa/retoma e preservação
de 32 decisões sem truncação. O primeiro build encontrou o registo dos macros
Tauri em falta; foi corrigido e os artefactos iniciais foram preservados.
Ainda faltam controlos funcionais de revisão, observações efetivas por adaptador,
proveniência de consentimento recuperável, leases, reserva/enforcement do consumo,
dispatch, execução e aceitação. `goals.run` continua indisponível e nada foi instalado.

A unidade660 acrescenta prova de consentimento em schema4 e ledger privado de
concessões, autenticados com HMAC-SHA256 de domínios separados. Chave/protocolo/
ficheiros têm publicação e validação privadas; perda/corrupção ou namespace
estrangeiro recusam adoção/regeneração. Pausa/cancelamento revoga a concessão
antes de publicar o objetivo. Repetir recibos não renova consentimento nem revoga
uma admissão posterior, e copiar um JSON antigo não desfaz a revogação no ledger.

Passou 1854 Rust/12 ignorados, 1085 JS, check/TSC/Vite, com 311 inputs compilados,
675 de fonte web inalterados e os mesmos 51 avisos, sem novos. Inclui 21 casos Rust
novos de binding criptográfico, cinco donos independentes, chave/protocolo/links/
identidade, falhas de publicação, revogação/replay, prazo e capacidade. O build
inicial encontrou um import de derives serde em falta; foi corrigido e preservado.
A consulta de proveniência é só leitura e mantém `executionAvailable:false`.
Ainda faltam claims/leases, política/prontidão no writer, orçamento aplicado,
execução, integração nativa e aceitação. Os schemas unsigned conservam leitura,
sem upgrade automático para autoridade. Backup/recuperação/rotação da chave,
órfãos, isolamento/exclusão de estado privado das ferramentas e migração/rollback
continuam pendentes. As fontes657 foram preservadas; a unidade não está instalada.

A unidade664 fixa claims em schema5 e ledger de concessão schema2, com um
controlador por provider lógico e contexto próprio. Pending/owner/worker/drain
são valores de backend não construídos por JSON. Publicação incerta não entrega
owner; a reconciliação original conserva o mesmo lease/prazo. Pausa/cancelamento
fica `stopping` até acabar o IO registado, e fim sem verificação conserva
`recoveryRequired`. Expiração não permite a outra instância roubar o slot.

Passou 1872 Rust/12 ignorados, 1087 JS e check/TSC/Vite, com 315 inputs compilados,
679 de fonte web inalterados e 51 avisos sem novos. Inclui 17 casos de lease,
revalidação transitória e cinco donos independentes, mais um caso de exec de
ficheiro ainda aberto para escrita. O fixture de rede preexistente voltou a
falhar noutra ronda (`Unavailable` vs `Protocol`); o resultado final passou e
foram acrescentados diagnósticos de spawn/exit, sem atribuir causa não provada.
Os endpoints de lease são preparação/estado/libertação, sem efeitos/modelos;
`executionAvailable` continua false. Reserva conhecida com zero guards é distinta
de lease externo sem contagem conhecida. Registo interno de IO aguarda produtores
reais. Faltam reserva/enforcement de orçamento por efeito, handoff do hold ao chat,
writer, dispatch, timer/limpeza, startup/reconciliação/Stop sob contenção e aceitação.
As fontes660 permanecem exatas e nada foi instalado.

**PET-01 continua aberto:** observações completas de descoberta/handshakes,
observações efetivas do runtime de áudio/modelo por contexto e adoção pela
admissão de trabalho, cobertura completa de serviços
e bindings e aceitação nativa/IPC. A integração completa da superfície Gravity
e continuidade/decisões continuam em PET-03. As provas atuais são sintéticas,
offline, e não confirmam uma conta ou subscrição atualmente ligadas.
O núcleo não substitui verificações atuais nem
passa a conceder execução. A base633 e o bloqueador CTX-01 foram preservados.
CTX-01 mantém a correção concreta pendente e é pré-condição da ativação da
composição que contém anexos novos. Não volta a ocupar automaticamente toda a
fila de desenvolvimento à frente dos contratos dos pets.

A revisão inicial de arquitetura não alterou implementação; os incrementos PET-01
e RUN-01 posteriores alteraram apenas os SDKs privados e documentos. Dados reais, serviços, modelos,
bases de dados, repositórios AI Local e configurações do anfitrião permanecem
intactos. Não houve commits, instalação ou novos agentes. O goal foi atualizado
com a prioridade arquitetura → pets → Sistema e retomado pela interface nativa;
`get_goal` confirmou texto e estado `active`. O objetivo completo permanece em curso.

A execução seguinte de posse durável está preparada sobre as fontes660 exatas;
ver a [continuação de leases](PETS-VALIDATION.md#evidencia-local).
A preparação dessa unidade ainda não acrescenta execução.

A unidade seguinte de writer/orçamento e handoff parte das fontes664 exatas;
ver a [continuação do writer](PETS-VALIDATION.md#evidencia-local).
A app instalada continua na .30; esta preparação não ativa execução.

### Estados funcionais de terminais e conversas — unidade666

A correção aplica-se ao censo comum de todos os providers declarados e às
conversas carregadas de todos os donos/pets ativos na fábrica. Dois clientes no
mesmo TTY são um terminal; um cliente suspenso não transforma esse terminal
num terceiro terminal ativo. Os estados de processo e de conversa têm fontes
independentes e não comprovam inferência. Uma conversa cancelada conserva
`stopping` até ao fim do guard; fonte ausente/inconsistente é desconhecida.

A apresentação funcional existente mostra esses estados separadamente da
biblioteca e do censo; não expõe PIDs/CWD/IDs nativos nem soma aliases. Passaram
15 regressões Rust e 14 JS novas, no total de 1887 Rust/12 ignorados e 1101 JS,
com check/TSC/Vite, 51 avisos de Rust já existentes e nenhum novo. As provas
fixam os mesmos 315 inputs nativos e 681 inputs web; 666 fontes preservam a
cadeia anterior. [Patch contra a base664](PETS-VALIDATION.md#evidencia-local)
e [checkpoint](PETS-VALIDATION.md#evidencia-local).

A .30 continua instalada; estes estados não foram entregues. O writer/orçamento,
execução/verificação, CTX-01 e os restantes bloqueadores de integração/release
mantêm-se pendentes. A unidade de writer, ainda sem alterações próprias, foi
atualizada para a base666 antes de continuar.

### Permissões nativas e controlo integral por voz — lacunas explícitas

O contrato ampliado não é uma afirmação de paridade concluída. Há percursos
próprios de chat/voz e mecanismos de decisão, mas faltam cobertura integral de
operações faladas e aceitação sincronizada por pedido nativo nos adaptadores.
O backend deve publicar a política observada e distinguir desconhecido de
permitido, incluindo perfil/sandbox/modos de aprovação.

Há um ponto concreto a auditar nessa integração: o modo explícito `full_access`
do chat pode enviar `sandbox=danger-full-access` e `approvalPolicy=never` no
[Codex](PETS-VALIDATION.md#evidencia-local)
ou acrescentar `--dangerously-skip-permissions` no
[Gravity](PETS-VALIDATION.md#evidencia-local).
Isto depende da opção existente; não prova bypass automático. A sua tradução e
migração têm de preservar a escolha do utilizador e ser reconciliadas com a
política/perfil efetivos do agente original e a nova paridade terminal/chat/voz,
sem inferir uma concessão nativa equivalente de uma preferência isolada da app.
Não foram alteradas permissões guardadas, terminais ou a app instalada nesta revisão.

### Writer de resposta e orçamento — unidade671

Passaram 29 regressões Rust novas: contadores/reservas persistentes, confirmação
perdida, duplicação, recibos incertos, fonte alterada, handoff por dono, modelo/
ferramentas, redirecionamentos, limite de conteúdo, protocolo incompleto e Stop
durante HTTP já recebido. No total: 1916 Rust/12 ignorados, 1101 JS e
check/TSC/Vite, 320 inputs nativos e 686 inputs web exatos, 51 avisos existentes
e nenhum novo. [Patch incremental](PETS-VALIDATION.md#evidencia-local)
e [checkpoint](PETS-VALIDATION.md#evidencia-local).

O percurso nativo inicial é apenas texto Ollama local, sem modelos reais em
QA, sem planos/ferramentas e sem concluir critérios; os outros adaptadores
continuam no âmbito e exigem limites nativos comprovados antes desse writer.
O problema de prontidão expirada durante IO longo foi identificado na integração
e é a próxima correção. Voz integral, permissões nativas coerentes entre canais,
aceitação real e os bloqueadores de release permanecem abertos.

### Prontidão antes de efeitos e validade durante IO — unidade673

O teste HTTP expira a prontidão após o servidor receber o pedido e conserva a
resposta autorizada; a mesma configuração passa a recusar nova chamada enquanto
não for renovada. Alterar permissões/política continua a interromper o IO e a
conservar reserva incerta. Metadados locais capturados renovam apenas a cache
do endpoint/modelo originais; falha ou origem remota não publicam prontidão.
Descriptor e fingerprint usam a mesma captura interna de configuração/settings.

Passaram 12 regressões novas, total 1928 Rust/12 ignorados, 1101 JS,
check/TSC/Vite; 322 inputs nativos e 688 inputs web exatos, 51 avisos já
existentes e nenhum novo. [Patch incremental](PETS-VALIDATION.md#evidencia-local)
e [checkpoint](PETS-VALIDATION.md#evidencia-local).
A .30 permanece instalada; não foram consumidas chamadas reais de modelos.
A correção não demonstra a paridade integral de políticas/perfis/sandbox/
aprovações do §9.1, nem controlo integral por voz ou autonomia completa.

### Prova terminal de retenção — unidade675

O envelope privado é sincronizado e publicado antes de remover o WAL final;
contém apenas metadados, dono/CID/ID/fase, bytes/hash originais e identidades dos
nós/namespace. O inventário mantém a recuperação com a revisão original quando
falta o WAL ou a pasta. A revisão nativa correspondente só limpa metadados:
não restaura/apaga anexos nem recria WAL/payload/lock. Envelope, namespace,
nós substituídos, links ou conteúdo extra são recusados e conservados; staging
completo idêntico pode ser reconciliado e staging parcial/estranho fica visível
como indisponível. O checksum não constitui permissão de execução.

Passaram 18 regressões novas, total 1946 Rust/12 ignorados, 1101 JS,
check/TSC/Vite; 324 inputs nativos e 690 inputs web exatos, 51 avisos existentes
e nenhum novo. [Patch incremental](PETS-VALIDATION.md#evidencia-local)
e [checkpoint](PETS-VALIDATION.md#evidencia-local).
A injeção de falha cobre publicação e cada fronteira final de unlink/rmdir/sync
para cópias eliminadas e repostas, namespace/identidade alterados, dados estranhos
e consentimento nativo vinculado. Não é uma simulação de corte físico de energia.

Restam aceitação de restauro parcial/nome reutilizado, limpeza recursiva interrompida,
razões exatas, protocolo de backup/relink dos metadados ligados a inodes e
migração/rollback v4/v5/leitores antigos. A app .30 permanece intacta; CTX-01 e
a entrega não estão concluídos. Voz integral e paridade efetiva das permissões
nativas mantêm os requisitos explícitos e ainda parciais.

### Recuperação parcial nativa e recusa — incremento675 seguinte

A lógica de movimento/restauro existente passou os novos casos reproduzidos;
foram acrescentados pontos de falha para verificar as fronteiras nativas. O percurso nativo foi validado depois do plano durável, cada rename
ou sync, ocupação posterior do destino por ficheiro/pasta/symlink, manifesto
restaurado com WAL ainda pendente e confirmação perdida do WAL final. Mantém
IDs/dono/CID e registos desselecionados/sem concessões nos oito providers.

A limpeza recursiva parcial não permite fingir restauro integral, conserva
nós estranhos e só retoma o que continua verificado. Reivindicação por outro
registo na mesma/outra conversa conserva ambos os registos/dados. Os códigos
nativos de recusa agora produzem mensagens seguras distintas; detalhes internos
não reconhecidos conservam a mensagem genérica, sem expor caminhos.

Passaram 8 regressões novas: 1954 Rust/12 ignorados, 1101 JS, check/TSC/Vite;
324 inputs nativos e 690 web exatos, 51 avisos existentes e nenhum novo.
[Patch incremental](PETS-VALIDATION.md#evidencia-local)
e [checkpoint](PETS-VALIDATION.md#evidencia-local).
As provas finais r4 fixam as fontes compiladas e conservam as verificações anteriores.

A auditoria das 597 fontes .30 confirmou recusa de memória v5 e limpeza de inbox
por idade sem exclusão dos novos namespaces. Seguem canários isolados e política
de backup/migração/rollback; a recusa do JSON antigo não basta para proteger os
anexos. Não houve leitura/mutação de perfis reais, modelos, instalação ou commits.

### Arranque admitido e caminhos — incremento693

A fonte privada liga a admissão nativa ao arranque antes de Settings/Chat/Recap,
separa dados mutáveis de hooks/bin/Models/log e guarda pedidos precoces numa
fila limitada. Os17 testes novos cobrem ordem, posse física, reinício, perfis
de todos os providers e pedidos de voz/preview/atalhos. Validação final:2026
Rust/13 ignorados,1101 JS e check/TSC/Vite; 51 avisos antigos,nenhum novo.
[Checkpoint](PETS-VALIDATION.md#evidencia-local).
A falha intermitente anterior de um fixture de rede não foi reproduzida em
20 repetições do caso exato; a causa específica não está estabelecida. Os
fixtures Python usam agora um interpretador estável, preservando asserções e
o teste de execução direta com imagem aberta para escrita. A suite completa
passou nesta fonte exata. Recuperação parcial/backup/rollback e aceitação GUI
continuam gates; nenhuma alteração foi instalada.

### Retoma nativa da migração — incremento695

A retoma já exige intenção anterior e conteúdo equivalente, valida todo o
payload existente antes de copiar e continua apenas prefixos exatos no mesmo
inode. Recupera meias pastas vazias com identidade original e conserva staging
de metadados limitado, sem o usar como prova de admissão. Divergências, perdas
após conclusão e meias pastas antes ativas continuam recusadas/preservadas.
A ligação ao bootstrap e14 regressões passaram:2040 Rust/13 ignorados,1101 JS
e check/TSC/Vite, sem avisos novos. [Checkpoint](PETS-VALIDATION.md#evidencia-local).

A aceitação seguinte já começou. Um inventário só de metadados da app .30
encontrou9 ficheiros conhecidos,127197 bytes, sem violações de tamanho/tipo/
permissões detetadas e sem truncar o percurso. Não leu conteúdo de conversas
ou anexos e não é snapshot consistente nem prova de schemas/referências.
Seguem referências tipadas, configuração de todos os pets, escritores, backup/
restauro/rollback e aceitação nativa. [Inventário privado](PETS-VALIDATION.md#evidencia-local).
A aplicação instalada não foi alterada; o objetivo integral mantém-se ativo.

### Histórico inativo comum — incremento697

A biblioteca já conserva donos/arquivos fora do catálogo e campos desconhecidos
da raiz num compartimento inativo, com formato6. O Chat partilha a preservação
em gravação, reinício, eliminação e restauro; o contexto dos modelos e os
objetivos isolados não o adotam. Referências não interpretadas impedem a purga
de anexos. A suite passou2054 Rust/13 ignorados,1101 JS e check/TSC/Vite,
sem avisos novos. [Checkpoint](PETS-VALIDATION.md#evidencia-local).
A revisão dos campos internos/normalização das conversas conhecidas e da
configuração continua pendente; este incremento não declara compatibilidade
integral nem fornece adoção/eliminação dos dados inativos.

O perfil .30 atual tem8 conversas,30 mensagens e10 tarefas; os JSON principais
foram apenas analisados localmente para contadores, sem guardar/apresentar
texto ou valores privados. Não se encontraram donos/campos da raiz/IDs de pets
desconhecidos. Os2 cwd de sessões e10 de tarefas são externos à app e não
exigem redirecionamento das pastas internas. A evidência é diagnóstica, sem
backup consistente ou prova de paridade das permissões nativas.
[Classificação privada](PETS-VALIDATION.md#evidencia-local).
Segue aceitação dos codecs nativos numa cópia privada, sem alterar a app.

### Compatibilidade nativa e serviços com AI opcional — incremento698

A cópia privada dos três JSON atuais passou pelos leitores nativos de Settings,
pets, biblioteca e restauro do Chat, em contentor só de leitura e no anfitrião.
Nenhum campo guardado foi removido/alterado; foram apenas acrescentados os cinco
limites de execução ausentes no formato antigo do Ollama, mantendo o modo
original. A cópia permaneceu intacta. Isto não é um backup completo nem prova
de permissões/GUI/áudio. [Aceitação nativa](PETS-VALIDATION.md#evidencia-local).

A prova encontrou e permitiu corrigir um reset silencioso do backend do Spotify.
A fábrica comum declara agora controlador nativo e assistência opcional: mantém
a escolha existente, os modelos/grants e o dono lógico; música/som continuam
com capacidades próprias e voz determinística. Sem backend escolhido não há
ativação implícita de modelos. Rust/TypeScript/capacidades/modelos e os controlos
funcionais existentes partilham o contrato, sem redesenho de UX.
Validação final:2061 Rust/14 ignorados,1103 JS e check/TSC/Vite; 51 avisos antigos,
nenhum novo. O probe privado e o helper de outro processo são executados
explicitamente nos ensaios próprios. [Checkpoint](PETS-VALIDATION.md#evidencia-local).
A app instalada continua na .30. Seguem backup/restauro/rollback consistentes,
escritores/helpers, composição/arranque nativo e entrega; o objetivo completo
permanece ativo.

### Backup legado nativo e restauro independente — incremento698 seguinte

O ensaio sobre o perfil real recusou inicialmente backups históricos e o lock
das métricas. A revisão passou a reconhecer os diretórios datados `pre-mochi…`
como dados opacos, sem adoção; valida o lock privado/vazio e conserva-o no
sítio, sem aquisição/cópia/substituição. Entradas desconhecidas continuam
recusadas. Duas regressões novas passaram na suite:2063 Rust/15 ignorados,
1103 JS e check/TSC/Vite, sem avisos novos.

A captura nativa leu e revalidou as fontes, guardou13 ficheiros e7 pastas
(141048 bytes) num candidato privado durável e reabriu-o noutro processo.
Um restauro independente foi sincronizado e verificado nativamente. Os codecs
de Settings/pets/histórico/Chat aceitaram os JSON restaurados sem remover ou
alterar campos guardados. [Prova privada](PETS-VALIDATION.md#evidencia-local).
Nenhum perfil original, processo da app/CLI, Wi-Fi ou configuração global foi
alterado. A prova cobre os dados mutáveis legados conhecidos, incluindo os
backups antigos; Models/bin/log não fazem parte dela. Não prova drafts em RAM,
rollback do serviço, backup do novo layout protegido ou autoridade nativa.
[Checkpoint](PETS-VALIDATION.md#evidencia-local).
Reconciliação concluída: 180 atualizações aplicadas ao checkout, com pré-imagens
guardadas. Os 22 ficheiros divergentes correspondiam à base .31 preservada;
a composição reconciliada é byte a byte a base698 já validada. Inputs nativos
e web também coincidem, incluindo os gerados. Não houve commit, merge de
branches, instalação ou alteração de perfis originais.
[Integração](PETS-VALIDATION.md#evidencia-local).
Continuam abertos o motor autónomo completo e os restantes 43 requisitos,
incluindo drafts, rollback da aplicação e aceitação de entrega.

### Resultados duráveis do motor — incremento702 integrado

O writer guarda um recibo privado, imutável e autenticado após confirmar a
contabilidade da chamada. O recibo conserva dono lógico, objetivo, decisão,
lease, modelo, pedido, tentativa, texto entregue e bytes aceites. Uma repetição
do pedido recupera e sincroniza esse recibo sem consultar metadados ou inferência.
A consulta nativa explícita não precisa de controlador vivo, funciona após
pausa/reinício e não concede execução nem declara critérios satisfeitos.

Tentativas reservadas, iniciadas, incertas ou concluídas sem recibo têm estados
distintos; não são repetidas automaticamente. Foram verificadas falhas antes e
depois da publicação, ficheiros adulterados/inseguros, troca entre tentativas,
limites e independência de cinco donos lógicos com contabilidade sintética.
Isto não equivale a executar os respetivos providers reais.

Validação final: 2077 Rust/16 ignorados, 1105 JavaScript, check/TSC/Vite e 51 avisos
antigos, nenhum novo. O anfitrião passou 14 ensaios com raízes XDG/temporárias
privadas; um deles executa outro processo para recuperar um objetivo pausado.
O novo helper ignorado é executado explicitamente pelo teste pai. Houve zero
chamadas reais a modelos e zero quota Copilot.
[Checkpoint](PETS-VALIDATION.md#evidencia-local).
[Ensaio nativo privado](PETS-VALIDATION.md#evidencia-local).

As 11 alterações (4 ficheiros novos) estão integradas, com pré-imagens e igualdade
dos 351 inputs nativos/717 web comprovadas. A app instalada continua na .30.
Permanecem pendentes ferramentas/planos/verificação de critérios, limites CLI,
recuperação de efeitos, ciclo de retenção/exportação/eliminação dos resultados,
backup do novo perfil e os restantes requisitos integrais.

### Broker comum das operações interativas — incremento707 integrado

Os percursos existentes que oferecem ferramentas a modelos usam agora o mesmo
validador e dispatcher para pets, snapshot de serviços, Spotify, abrir aplicações
e preparar email. O catálogo/schema e os argumentos tipados são comuns; a
fábrica fornece o dono lógico e as concessões existentes. O parser local de voz
também adota o validador comum. O catálogo não concede execução autónoma.

Um pedido de volume sem valor deixa de poder assumir 50; valores fora de limites
e campos estranhos não chegam ao executor. O teste encontrou ainda campos extra
aceites nas variantes sem parâmetros de Serde; a guarda agora recusa-os.
O dono/configuração e o contexto/cancelamento do chamador são revalidados antes
do despacho e depois do resultado; mudanças noutro pet não emprestam grants.

Os resultados distinguem retorno, ACK, despacho sem confirmação, parcial e
incerto; nenhuma destas respostas declara automaticamente um efeito verificado.
Launchers/mailto deixam de anunciar uma abertura comprovada. A pesquisa Spotify
preserva literalmente Unicode e caracteres como `C++`. O recibo JSON completo
cabe em 16000 bytes, limite partilhado com o contexto local; a truncagem é explícita.
A lógica/10 testes de roster passaram ao módulo comum, com facade nativa.

Validação: 2092 Rust/16 ignorados, 1105 JS, check/TSC/Vite e 51 avisos antigos,
nenhum novo. Os 25 ensaios privados no anfitrião usam executores simulados;
não controlaram som, aplicações ou correio nem consumiram modelos/quota.
[Checkpoint](PETS-VALIDATION.md#evidencia-local).
[Prova nativa privada](PETS-VALIDATION.md#evidencia-local).
As 12 alterações (5 ficheiros novos) estão integradas com igualdade dos 356 inputs
nativos/722 web. A app instalada permanece na .30, sem novos commits.

Este incremento não prova novas capacidades nos CLI reais. Calendário, processos
aprovados e rotas determinísticas de serviços ainda têm de aderir; continuam
pendentes autoridade/ledger/recibos de ferramentas por objetivo, chaves de
idempotência, pinning de recursos/alvos, pós-condições independentes e a matriz
de Sistema. O objetivo integral e os 43 requisitos permanecem ativos.

### Propostas tipadas e revisão vinculada — incremento710 integrado

Os passos guardam opcionalmente ID/versão de operação e argumentos validados
pelo catálogo comum. A proposta permanece dados: não concede execução nem
permissões. O journal de objetivos usa schema6 para propostas tipadas/nova
revisão; admissões novas usam schema2 com hash do conteúdo estático revisto.
Isto é distinto da versão do histórico de conversas.

Tarefas, ligações, descrições, pós-condições e parâmetros ficam vinculados ao
consentimento nativo. Alterar o plano sem mudar a revisão aparente é recusado;
recalcular hashes de dados não substitui a proveniência nativa. Selos antigos
fechados continuam preservados ao retomar um draft com trabalho novo. A prova
de compatibilidade guarda uma fila legada real v3/admissão1 e confirma que abrir
a autoridade não a altera nem lhe acrescenta uma autorização.

O modelo textual recebe intenção/critérios integrais e propostas revistas,
limitados a 256 KiB sem truncagem silenciosa. A pergunta visível na atividade e
no chat mantém a intenção humana, separada do envelope privado de contexto.
A revisão declara `conversationRespond` e `toolExecutionAvailable:false`:
os passos ainda não são executados e `maxToolCalls` não concede ferramentas.

Validação: 2106 Rust/16 ignorados, 1107 JS e check/TSC/Vite; 51 avisos antigos,
nenhum novo. Os 155 ensaios do runtime no anfitrião usam raízes privadas e
HTTP/callbacks simulados; incluem leitura por outro processo após pausa.
Não houve inferência, quota Copilot, controlo de dispositivos ou perfis originais.
[Checkpoint](PETS-VALIDATION.md#evidencia-local).
[Prova nativa privada](PETS-VALIDATION.md#evidencia-local).

As 16 alterações (3 ficheiros novos) estão integradas, com 359 inputs nativos e
725 web exatos. A app instalada permanece na .30. Seguem concessões próprias de
operações, recursos/alvos vinculados, ledger/dispatch/recibos de ferramentas,
verificação independente e os restantes requisitos do roadmap. A validação de
dono do runtime ainda contém regras por provider; rever a adoção da fábrica
para serviços com assistência opcional e restantes adaptadores antes de
anunciar cobertura autónoma geral.

### Execução de passos — checkpoint inicial de transporte, superado pelo717

O transporte Spotify em teste conserva ligação local, bus, owner único e peer
UID/PID/identidade capturados. Os comandos usam o owner único e não iniciam o
leitor, não reconectam nem seguem uma instância substituta. Reprodução, pausa
e volume têm leituras nativas antes/depois, independentes da projeção otimista.
Um erro depois do efeito pode ser reconciliado por leitura, sem repetir o comando.

Nove regressões passaram apenas no transporte isolado, com D-Bus/leitor privados
e nenhum modelo/dispositivo real. O resultado conserva os factos se Stop/prazo
ocorrer durante IO, sem declarar verificação corrente. Não existe ainda concessão
de ferramentas a partir deste checkpoint e não se declara a unidade concluída.
Nessa fase, a raiz permanecia na fonte710. O percurso717 posterior ligou a
autorização e a execução; a composição corrente é718 e a app instalada continua
na .30. O
[protocolo da mesma unidade](PETS-VALIDATION.md#evidencia-local)
define a continuação para autorização, orçamento, recursos, recibos e recuperação.


### Percurso de passos717 e contrato da fábrica718 integrados

O percurso717 acrescentou 28 regressões Rust aprovadas e dois testes JS: revisão
nativa exata por janela, concessão assinada de uso único, orçamento de ferramenta
antes de IO, alvo/recurso MPRIS capturado, observação independente e recibos.
Stop conserva holds até drenar; falhas entre reserva/início/resultado não repetem
a ação. A consulta sobrevive a pausa e processo novo. Aprovação utilizável na app,
restantes providers, critérios, ciclo autónomo e reconciliação explícita continuam
pendentes. [Prova717](PETS-VALIDATION.md#evidencia-local).

O incremento718 removeu as regras específicas de provider da validação de donos,
usando o catálogo e preservando o provider lógico do serviço quando seleciona um
assistente opcional. A criação a partir do descritor nativo rejeita identidades
inativas/retiradas, observadores, rotas sem executor e bindings incompatíveis.
Registos antigos continuam legíveis e não ganham concessões. Mantêm-se os schemas,
os 25 defaults e as preferências. Sete regressões Rust novas; composição2141/17,
1109 JS, check/TSC/Vite e 181 casos nativos privados do runtime, sem novos avisos,
modelos, quota Copilot, alteração de perfis ou instalação. Seis ficheiros aplicados,
367 inputs nativos/733 web exatos. [Prova718](PETS-VALIDATION.md#evidencia-local).

## Registo histórico de estimativa substituída em 25/10

O texto seguinte é preservado como histórico de comunicação; não é medição vigente nem critério de aceitação:

**Estimativa de planeamento comunicada em 24/10 e reconfirmada em 25/10:** aproximadamente **40/100**
para o sistema completo e **150–300 horas de trabalho focado restantes**, com
incerteza elevada. Não resulta da proporção de testes nem de requisitos fechados;
inclui restantes adaptadores, autonomia/recuperação, memória/fontes, voz,
rotinas/cooperação, matriz de Sistema e aceitação/entrega. Uma primeira versão
útil pode preceder o âmbito completo. Rever a estimativa após aceitação integrada
da execução, paridade por provider e voz; não constitui um prazo de entrega.
