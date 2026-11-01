# Objetivo persistente dos pets

## 01/11 — Fala nativa e candidata mochi.32 (C6C3)

O objetivo integral e os 43 requisitos originais + 6 Guardian permanecem abertos. A instrução do utilizador de 01/11 autoriza um ciclo seguro de commits, atualização do remoto e instalação da aplicação. Depois desse ciclo, commits e publicações voltam a ficar suspensos e o desenvolvimento prossegue. A composição está integrada; o artefacto e o arranque ativo foram verificados; a aceitação integral continua pendente.

[Composição, provas, entrega e limites](releases/0.5.1-mochi.32.md).

A versão.32 está agora instalada e ativa: nome D-Bus, executável empacotado e admissão nativa verificados; três originais de preferências/conversas e cópias nativas byte-exatas. A.30 foi preservada. O ensaio284 do próprio RPM passou com helper sintético; isto não fecha áudio real nem os49 requisitos.

## 31/10 — Seleção TTS e cancelamento por resposta (C6C2, fontes1000)

Fontes1000 integradas: seleção TTS capturada na ocorrência de Voz e no preview, validade após esperas e cancelamento do renderer por resposta exata. Falha do motor escolhido conserva o texto; só a seleção explícita system pode usar síntese do sistema. [Provas e limites C6C2](../.build/pet-goal-execution-next-r1/VOICE-PLAYBACK-CHECKPOINT-C6C2A3.json).

Progresso de implementação e integração. Todos49 requisitos integrais mantêm-se abertos. C6C3: vincular também voice_say_native à resposta/superfície/validade e ao worker próprios; concluir alvo nativo exato de captura e revisão/estado/revogação/ativação de Voz após restauro. Sem áudio real para QA. Depois Móvel/Hooks/Credenciais e restantes dependências integrais.

## 31/10 — Entradas imutáveis da Voz (C6C1, fontes999)

Fontes999 integradas: entradas de captura/ASR e seleção de inferência imutáveis por ocorrência de Voz, usando os runners e a fábrica existentes. Alterar pet, perfil, endpoint ou preferências durante a espera não redireciona o pedido; o contexto inválido é recusado. O snapshot não renova grants. [Provas e limites C6C1](../.build/pet-goal-execution-next-r1/VOICE-TURN-CHECKPOINT-C6C1A4.json).

Progresso de implementação e integração; os49 requisitos integrais continuam abertos. C6C2: fixar também o destino de TTS e o alvo nativo de captura; concluir revisão humana/estado/revogação e ativação da Voz após restauro com os gates existentes, sem abrir dispositivos/modelos reais para QA. Depois Móvel/Hooks/Credenciais e restantes dependências integrais.

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

Objetivo integral ativo, único agente, sem commits/publicação ou substituição da instalação. Turno = progresso: recuperação utilizável no contexto atual, com autoridade própria e preservação de incerteza. Root834 e instalação mantidos.

A composição privada SDK995/593 inputs permite um novo pedido humano de atalhos no contexto atual quando o original não pode ser observado. A revisão3 retém a cadeia e as identidades originais, mostra o destino atual e exige quatro confirmações, versão exata e uso único. Reutiliza Controller, Frame, gates, executores e diário; não concede permissões a pets. Sucesso atual conserva recibos e incerteza de scope anteriores. O portal pode partilhar configurações por aplicação: não se afirma independência por novo token, cliente, PID ou nome.

Formato8 impede leitores7 de ignorarem a incerteza histórica de Slots Verified/Reconciled. Controlos cumulativos, migração monotónica, reinício sem registo automático, backup e restauro nativo8 foram verificados.

Provas: 68 dirigidos passaram/2 ignorados; 3138 gerais passaram/38 ignorados; 1329 web/0 omitidos, TypeScript, Vite e build. Nativos274/275 atravessam Tauri/WebKit, quatro confirmações, registo/callback, segunda revisão, leitor7, reinício e restauro8. Portal sintético e X11 virtual; não aceitação humana/hardware/instalação.270/271 e272/273 conservam falhas de fixture (bindings deliberadamente desligados no restauro; profundidade2 indicada por Admission3), corrigidas sem alterar fontes/binário nem remover asserções.

[Checkpoint privado C4](../.build/pet-goal-execution-next-r1/SHORTCUTS-CURRENT-SCOPE-PARTIAL-CHECKPOINT-R43F10B2C3D2B3C4A1.json)

Segue Lifecycle e adoção exata da composição SDK995: verificar guards após fecho de Settings/chat/janelas, Stop e reinício, sem serviço/autostart persistente; integrar a partir de fontes comprovadas quando os bloqueadores de preservação permitirem. Prosseguir com os controladores Voz/Móvel/Hooks/Credenciais e as dependências integrais. Todos os49 critérios integrais continuam abertos.

## R43F10B2C3D2B3C3A2 — comparação nativa original e atual

Trabalho retomado pelo pedido explícito «continue» após pausa. O gestor ainda reportava paused; não foi marcado completo nem alterado por este incremento. Objetivo integral e restrições mantidos. Root834/instalação preservados, sem commits/publicação.

Privado: 992 fontes/591 entradas nativas. Passaram 55 testes dirigidos, 3134 gerais (38 ignorados), 1326 web/0 ignorados, TypeScript, Vite, build e nativos266/268/269. [Fontes, provas e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-ORIGINAL-OWNER-PARTIAL-CHECKPOINT-R43F10B2C3D2B3C3A2.json).

Segue C4, recuperação de contexto nativo com autoridade própria e incerteza anterior conservada, depois lifecycle/adoção e restantes componentes. A comparação atual não fecha essa recuperação nem os49critérios integrais.

## R43F10B2C3D2B3C2A1 — restauro 7 e suspensão de todas as componentes

Goal integral ativo e turno classificado como progresso: correção real no restauro partilhado, não mudança do objetivo. Fonte privada992; composição integrada 834/instalação .30 intactos, sem commits/publicação/host/modelos.

Composição privada: 992 fontes/591 entradas nativas. Passaram 43 testes do módulo no A1. A execução geral C2 passou 3130 testes e teve duas expectativas antigas corrigidas no reteste dirigido; 38 ignorados. A1 altera apenas cfg(test), sem nova execução geral/build/nativa. Build e ensaio nativo 265 de C2, web1324/TSC/Vite B3 são reutilizados por identidade explícita dos respetivos inputs. [Fontes, provas e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-RESTORE-PARTIAL-CHECKPOINT-R43F10B2C3D2B3C2A1.json).

Seguem comparação original/atual no controlo humano, história e recuperação de scope nativo, lifecycle/adoção e restantes quatro componentes. Os 49critérios integrais continuam abertos.

## R43F10B2C3D2B3B8 — recuperação dos atalhos e leitores antigos

Goal integral ativo; único agente, sem commits/publicação/instalação. Recuperação utilizável e barreira de leitor 7 verificadas em privado. O ensaio260 falhou numa consulta de metadados da janela oculta; causa não estabelecida, prova preservada.262 evita repetir a mesma prova de janela errada, mantendo os três percursos e o mesmo binário.

Composição privada: 992 ficheiros de fonte e 591 entradas nativas; backend e binário B7, uma correção de fixture cfg(test) B8. 52 testes dirigidos B8 passaram; a execução geral B7 teve 3129 testes aprovados, uma falha de fixture e 38 testes ignorados; a falha foi conservada e corrigida com reteste dirigido.Os 1324 testes web, TypeScript e Vite de B3 e os 18 testes D-Bus de B5 reutilizados por identidade explícita; build B7 e ensaios nativos 261 (X11) e 262 (portal) passaram. Não houve nova execução dessas suites no B8. [Fontes, provas e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-RECONCILIATION-PARTIAL-CHECKPOINT-R43F10B2C3D2B3B8.json).

Continuar C3D2B3C (restauro 7/história/novo perfil), lifecycle/adoção e quatro componentes. Root834/.30 intactos e 49 critérios integrais abertos.

## R43F10B2C3D2B3A1 — resultado pendente e publicação

Goal integral ativo; único agente e sem publicação/instalação. O privado984 preserva a incerteza entre registo/publicação/limpeza e recusa nova revisão após falha. A compatibilidade com leitor983 foi ensaiada; a interrupção é fixture sintética, não crash/powerloss real. Continuar B3B funcional antes de adoção. Root834/.30 e49critérios integrais abertos. [Provas e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-PUBLICATION-PARTIAL-CHECKPOINT-R43F10B2C3D2B3A1.json).


## R43F10B2C3D2B2B3 — aprovação dos atalhos e suspensão X11

Goal integral ativo, único agente e sem commits/publicação/instalação. O privado983 avançou o percurso autorizado dos atalhos e corrigiu o bloqueio real de suspensão X11. Segue B3, incluindo a falha após Verified que não pode ser tratada como reconciliação. Root834/.30 permanecem intactos; os49critérios integrais continuam abertos. [Checkpoint e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-APPROVAL-PARTIAL-CHECKPOINT-R43F10B2C3D2B2B3.json).


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


## R43F9B2D1 — configuração Header3 revista, ainda sem admissão operacional

901/D1A1 acrescenta revisão/recibo2 nativo de configuração e índice2 que une referências anteriores seladas às atuais; não ativa definições nem trabalho. Corpo1 e recibo1 mantêm bytes; leitores Admission2 exigem índice1.2804Rust/21ignorados/9novos/build522/native180 com providers sintéticos, incluindo hash do índice composto independentemente e ACK sem escrita. [Provas e limites](../.build/pet-goal-execution-next-r1/EPOCH-CONFIGURATION-PARTIAL-CHECKPOINT-R43F9B2D1.json). D2/admissão3, restantes dependências e49critérios integrais abertos; raiz834/instalação.30 intactas,0quota/modelos reais,sem publicação.


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

Revisão normativa: **24/10/2026**, a partir do objetivo atualizado fornecido pelo
utilizador e lido integralmente nesta sessão. A referência atual do goal aponta
para esse anexo; este documento conserva o conteúdo completo para a execução no
repositório. Guardar este ficheiro não altera os controlos de Goals da aplicação.

## Texto do objetivo

Concluir o sistema completo de Pets do fork Coucou como assistentes digitais de IA persistentes, autónomos, cooperativos e sempre disponíveis enquanto a infraestrutura permitir, capazes de trabalhar de forma proativa e autorizada no dia a dia.

PRIORIDADE PRINCIPAL: concluir a arquitetura coerente, robusta, verificável e extensível já em desenvolvimento. Preservar os 43 requisitos originais, todos os pedidos posteriores, as decisões existentes e o trabalho realizado. Seguir docs/PETS-ARCHITECTURE.md, docs/PETS-ROADMAP.md, docs/PET-CAPABILITIES-CONTRACT.md e a restante documentação vigente. Não reiniciar, substituir ou reestruturar componentes funcionais sem necessidade demonstrada.

ARQUITETURA MULTIAGENTE: todos os Pets existentes e futuros devem ser entidades autónomas de primeira classe, incluindo Codex, Gravity, Copilot CLI, Ollama e outros adaptadores configuráveis. Cada Pet deve poder ser chamado diretamente pelo utilizador, manter identidade, capacidades, contexto, sessões, memória e objetivos, receber e executar tarefas, desenvolver iniciativas autorizadas e colaborar com outros Pets, respeitando as capacidades reais do respetivo fornecedor.

COMUNICAÇÃO E COOPERAÇÃO: preservar e evoluir a comunicação direta entre Pets, sem obrigar todas as interações a passar por um coordenador. Implementar ou consolidar um protocolo comum para descoberta de capacidades, mensagens, pedidos, respostas, delegação, eventos, resultados e acompanhamento de tarefas. Suportar colaboração paralela e assíncrona, correlação de mensagens, rastreabilidade, controlo de concorrência, prevenção de ciclos, conflitos, duplicações e delegações ilimitadas. Preferir reutilizar mecanismos existentes em vez de introduzir infraestrutura redundante.

COORDENAÇÃO: permitir que o Pet Ollama assuma preferencialmente o papel de coordenador global, com visão dos objetivos, prioridades, estados e capacidades dos restantes Pets, sem se tornar intermediário obrigatório, dependência central ou ponto único de falha. A coordenação deve ser uma capacidade funcional configurável, separada do motor de inferência e da infraestrutura de controlo. Outros Pets devem continuar operacionais quando o coordenador, Ollama ou qualquer fornecedor estiver indisponível.

RUNTIME AUTÓNOMO: separar conceptualmente a identidade/interface visual de cada Pet, a sua execução persistente e os respetivos motores de IA. Permitir que os Pets permaneçam disponíveis para receber eventos, gerir tarefas, comunicar e executar operações determinísticas sem manter modelos locais carregados ou processos de inferência permanentemente ativos. Invocar Ollama, Codex, Copilot ou outros motores apenas quando necessário e suportado. Preservar tarefas e estados através de encerramentos da interface, reinícios e falhas, dentro das limitações reais do ambiente e dos fornecedores.

AUTONOMIA CONTROLADA: completar objetivos/tarefas/filas duráveis, agendamento, eventos, proatividade configurável, execução e verificação, recuperação de falhas, memória individual e partilhada, documentos, fontes pessoais, sessões e decisões nativas, regras, utilização, voz e notificações. Garantir que tarefas iniciadas diretamente num Pet e tarefas delegadas são acompanhadas pelo mesmo sistema de controlo, sem retirar independência aos agentes.

SEGURANÇA E GOVERNANÇA: manter uma matriz de permissões de Sistema por Pet/subplugin/operação, aprovações nativas, isolamento adequado, auditoria, limites de recursos, custos, concorrência e profundidade de delegação. Nenhum Pet pode obter permissões adicionais apenas por receber uma delegação de outro. Operações destrutivas, alterações sensíveis, envios externos e ações irreversíveis devem respeitar as autorizações explícitas aplicáveis. Preservar dados, preferências, credenciais, permissões e Wi-Fi.

IMPLEMENTAÇÃO: avaliar primeiro a arquitetura real e os componentes existentes, distinguindo capacidades implementadas de funcionalidades apenas documentadas. Preferir integração incremental, modular, testável e compatível com o sistema atual. A arquitetura deve permitir acrescentar novos Pets, motores, ferramentas e integrações sem exigir alterações extensivas aos restantes. PostgreSQL permanece opcional e exige benefício demonstrado, sem acesso às bases de dados de outros projetos.

ÂMBITO E RESTRIÇÕES: descontinuar temporariamente apenas o Pet GitHub enquanto entidade, conservando os seus dados; Gravity e Copilot CLI permanecem no âmbito. Trabalhar como único agente de desenvolvimento desta intervenção, sem criar novos agentes de desenvolvimento auxiliares e sem novos commits, pushes ou merges. Esta restrição não impede implementar ou testar a colaboração entre Pets do produto. Frontend/UX limitado a bloqueadores funcionais, aprovações, preservação e entrega. Não consumir quota real de GitHub CLI/Copilot em testes; utilizar alternativas gratuitas adequadas ou, quando estritamente necessário e autorizado, a opção mais económica. Respeitar as instruções da estação e registar alterações persistentes do anfitrião no sysworkspace.

ORDEM DE EXECUÇÃO: primeiro consolidar a arquitetura e contratos; depois concluir identidade, capacidades, sessões, persistência, autonomia e execução individual dos Pets; em seguida garantir comunicação, cooperação, delegação e coordenação resilientes; depois completar as integrações e capacidades funcionais pendentes. Ajustar esta sequência às dependências reais e ao estado já implementado, sem refazer trabalho concluído.

CONTINUIDADE E CONCLUSÃO: continuar autonomamente após cada incremento, atualizar documentação, arquitetura e evidências, identificar a próxima unidade útil e executá-la dentro das permissões disponíveis. Não declarar o objetivo concluído até todos os requisitos aplicáveis estarem implementados, integrados, testados, verificados no âmbito correto e entregues, com exceções e pendências explicitamente resolvidas. Testes verdes isolados, protótipos privados ou documentação sem implementação não bastam para declarar o sistema completo.

## Execução atual

Atualização R43: incremento privado840/E1 validado com 2606 Rust/21 ignorados,
35 testes de memória, check/build e provas Tauri para as três contas CLI e um
serviço independente: 17 pedidos sintéticos, reinício na mesma sessão, revogação,
recolha dos CLI e preservação de proveniência incerta. A regressão dos dois
motores locais também passou na mesma fonte, sem quota real. A raiz aceite
continua834. Seguem memória na admissão/hash/recibos dos objetivos e os três
caminhos API com HTTP sintético, depois confirmação humana e descritores.
[Provas e pendências](PETS-VALIDATION.md#contexto-memoria-r43).

A prioridade é consolidar a arquitetura, completar a autonomia e execução
individual dos pets, depois comunicação/cooperação/coordenação resilientes e as
integrações pendentes, segundo as dependências reais. Preservar os 43 requisitos,
identidades, dados e componentes existentes. Ollama é coordenador opcional;
os restantes pets continuam diretamente acessíveis sem esse motor/coordenador.

O estado aceite é [local834](PETS-VALIDATION.md#memoria-local834):2582 Rust/21
ignorados, check/build, 751 ensaios privados no anfitrião e 1186 JS/TSC/Vite.
Memória explícita por pet/conversa reutiliza armazenamento privado, com pesquisa,
revisões, validade e exclusão. O percurso nativo cobre 24 pets não retirados,
conversas vazias persistentes/arquivadas sem mensagens fabricadas e reinícios.
Os12 percursos anteriores passaram na mesma fonte/build; três após corrigir
apenas a ativação LM Studio nas fixtures. São 13 percursos positivos e um controlo
anterior que reproduziu a perda de CID. As falhas ficam preservadas.

A continuidade CLI e o runtime local mantêm os contratos anteriores. Memória não
é incluída automaticamente nos modelos. Segue R43: seleção/contexto comuns por
transporte, revalidação, destino autorizado, revogação e vínculo à admissão dos
objetivos; o contexto já entregue a uma sessão nativa exige tratamento próprio.
Restantes providers, autonomia integral, fontes/projetos/derivados/partilha,
cooperação, voz, matriz de Sistema e migração/backup/rollback/entrega mantêm os
respetivos critérios. O sistema completo e os 43 requisitos continuam ativos.

A app instalada continua .30. Trabalhar como único agente de desenvolvimento;
commits, pushes e merges continuam suspensos. Cooperação dos pets do produto
continua no âmbito. Não consumir quota real Copilot/GitHub CLI nem modificar
perfis pessoais, Wi-Fi ou a instalação nos ensaios privados. Consultar a
[arquitetura](PETS-ARCHITECTURE.md), o [roadmap](PETS-ROADMAP.md) e o
[estado](PETS-IMPLEMENTATION-STATUS.md).

**Atualização de estado R43F1A — 25/10/2026:** no SDK privado841 passou a preparação durável de memória selecionada para objetivos: referências exatas por pet/conversa de origem, contexto versão2 e objetivo versão11 apenas quando há seleção, sem copiar os dados brutos para o diálogo inicial. Passaram 2612 testes Rust/21 ignorados, check/build e o percurso Tauri privado130, incluindo reinício, revogação e recusa do leitor840 sem alterar os ficheiros. Não houve chamadas a modelos ou quota. A admissão autónoma com memória permanece deliberadamente recusada até R43F2 ligar revisão humana dos dados/destino, consentimento próprio do objetivo e mensagem anterior ao hash/reserva canónicos. A raiz aceite permanece834; a app instalada permanece .30. R43 e os 43 requisitos continuam abertos.

**Revisão posterior de 25/10 — Guardian e execução integral:** os 43 requisitos originais conservam os seus critérios; acrescentam-se GUA-01..06 no [contrato System Guardian](SYSTEM-GUARDIAN-ARCHITECTURE.md) e na [auditoria rastreável](PETS-REQUIREMENTS-AUDIT-2026-10-25.md). Pet, runtime comum, motor de inferência e papel funcional são entidades distintas. `ai_ollama` continua coordenador preferencial configurável, sem ser router obrigatório ou concedente de permissões. Guardian é especialista da mesma fábrica, sem autoridade própria adicional. Não há um pet Guardian operacional por esta atualização documental.

**Estado verificado nesta continuação:** raiz integrada834 e instalação .30 preservadas. SDK843/R43F2A passou 21 testes dirigidos de contexto, 2617 Rust/21 ignorados e cargo check em 483 inputs nativos imutáveis. O novo contrato de revisão mostra dados completos e destino nativo, separa consentimento de objetivo e verifica âmbito/credencial/revogação. É fundação privada: ainda falta a ligação à admissão, concessão assinada e corpo real anterior ao hash/reserva. Não foi acrescentada prova Tauri ou build instalada por este incremento. A barreira F1 permanece fechada; não declarar R43 concluído.

**Restrições vigentes:** um único agente, sem novos commits/pushes/merges de qualquer pasta; a publicação anterior do dossier foi uma exceção histórica já executada. Sem alteração da instalação/autostart/serviços, Wi-Fi/VPN/firewall/dispositivos ou dados pessoais. Sem quota real Copilot/GitHub, segredos em modelos/logs/Git ou bases de outros projetos. Alterações persistentes do anfitrião seguem sysworkspace e autorização específica; o desenvolvimento atual fica no projeto. Estados/estimativas antigos são históricos e não medem conclusão.

## Extensão integral System Guardian e responsabilidade — 25/10

O pedido posterior acrescenta GUA-01..06: inventário/evidência, segurança/vulnerabilidades, saúde/desempenho, integração sysworkspace, remediação governada e proatividade/cooperacão. O [contrato completo](SYSTEM-GUARDIAN-ARCHITECTURE.md) e a [auditoria dos 43+6 requisitos](PETS-REQUIREMENTS-AUDIT-2026-10-25.md) preservam os critérios. Nenhum substitui os 43 IDs ou reduz o produto a um chatbot/MVP.

Separar pet, runtime comum, motor e papel. `ai_ollama` é coordenador preferencial configurável, sem router obrigatório ou poder de conceder acessos. `agent_system_guardian` é especialista separado, ainda por implementar. Não criar outro runtime, scheduler, ledger de autoridade, stack, banco ou modelo residente por pet sem necessidade demonstrada. Desenho de lifecycle distingue chat/janelas/saída/crash/suspensão/logout/reboot e não promete execução com máquina desligada.

Concluir R43 antes de novas especializações: consentimento próprio de objetivo, dados/destino exatos, corpo real antes de hash/reserva, revogação/retoma, três APIs HTTP sintéticas, regressões locais/CLI, controlo humano e capacidades por caller; integrar fontes comprovadas sem substituir a instalação. Prosseguir por autonomia/preservação, mesh, memória/fontes/rotinas, Sistema/voz, Guardian e entrega conforme dependências. Não repetir incrementos comprovados ou suites sem alteração/risco; manter checkpoints sem commits.

Preservar ferramentas/MCP/sandbox/decisões originais, orçamento repartido sem duplicação, um dono por efeito, limpeza física antes de libertar recurso e incerteza sem replay. Policies são determinísticas: conteúdo hostil de fontes/pets/logs não concede acesso; código não fiável exige contenção real. Não inventar exactly-once de efeitos externos, rollback, vulnerabilidades, validação humana ou percentagens/horas sem base mensurável.

**Histórico R43F2D — 25/10:** SDK846 prova os envios de memória de Anthropic/OpenAI/Google através da implementação nativa comum, com HTTP loopback sintético, dados separados da pergunta e recusa de destino/modelo/credencial/pergunta alterados. Foi retirado o opt-in de fallback automático Anthropic; APIs e percursos locais usam o destino exato sem seguir redirecionamentos. Passaram2632 Rust/21 ignorados, check/build e Tauri136:13 pedidos HTTP e17 prompts CLI sintéticos, incluindo memória/decisões/recolha/retoma dos três CLI e serviço separado, objetivos locais e dois redirecionamentos recusados sem pedidos ao destino alternativo. Não certifica contas reais, routing interno do fornecedor, UI humana ou instalação. Raiz834 e instalação .30 preservadas. Seguem interface humana/callers, retoma de objetivos com memória e adoção/migração/entrega; R43 e os49 requisitos continuam abertos.

## Histórico de execução anterior a esta revisão

O conteúdo seguinte conserva evidência e decisões de incrementos anteriores.
Datas, números de fontes e próximos passos antigos têm valor histórico; o
objetivo, a prioridade e o estado correntes são os apresentados acima e nos
documentos normativos ligados.

Estado corrente em 24/10: **759 fontes integradas e versionadas na raiz do projeto**. A área de Tarefas já tem chamadas de produto às APIs de objetivos:
criação/contexto/admissão/claim/resposta/receita/estado/Stop. A gestão responde
apenas a pedidos de objetivos no broker comum, sem alargar permissões de
terminais. A inspeção de metadados locais é explícita, limitada e sem inferência.
São 400 inputs nativos/774 web exatos, 2373 Rust/20 ignorados, 1153 JS, check/TSC/
Vite e 277 ensaios privados no anfitrião. Vinte ficheiros aplicados, 15 pré-imagens,
cinco fontes novas, sem novos avisos/modelos/quota. A app instalada continua .30.
[Checkpoint](PETS-VALIDATION.md#evidencia-local).
[Aceitação e limites](PETS-VALIDATION.md#evidencia-local).

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

Guardar ou atualizar a vista não inicia trabalho. Uma gravação incerta interrompe
a sequência; a autorização é revista pelo utilizador, e um controlador nativo
original é necessário para executar. Preservam-se os contextos privados, origem
própria de receitas, consumo único em chat/voz, critérios tipados e conclusão
schema7 autenticada. Os controlos e o controlador têm fixtures; a inspeção e o
broker têm IO privado. A aceitação da aplicação gráfica completa, microfone/TTS,
inferência real e entrega permanece pendente. O writer é ainda Ollama local;
o primeiro executor/verificador físico mantém as operações Spotify suportadas.

Continuar a **MESMA unidade completa** com planeamento/replaneamento limitado,
despacho nativo, restantes operações/verificadores/adaptadores e aceitação real.
As revisões v2 selam o plano: não as enfraquecer para admitir mudanças; a evolução
exige contrato de proposta/ack nativo, orçamento comum e transação autenticada.
Não remover a guarda de pedidos internos das CLI sem limites comprovados. Nem
JSON, prova antes da primeira publicação ou lease guardado criam controlador.
Migração/backup/rollback do schema7 e aceitação de entrega continuam obrigatórios.
[Contrato seguinte](PETS-VALIDATION.md#evidencia-local).
[Continuação](PETS-VALIDATION.md#evidencia-local).

Todos os 43 critérios permanecem no âmbito: memória/fontes, regras, cooperação,
voz integral, matriz de Sistema e preservação/entrega. A estimativa global de
40/100 é aproximada, sem substituir a aceitação por requisito. O objetivo
persistente continua ativo; os parágrafos seguintes conservam o histórico.

O primeiro incremento da melhoria da fábrica está validado em 654 fontes
privadas: identidades/serviços, classificação/binding e preparação de voz usam
metadados tipados comuns. Passou 1818 Rust/12 ignorados, 1082 JS e check/TSC/Vite;
mantém os 25 defaults e a versão1 das preferências. Completar os contratos
comportamentais por adaptador e a aceitação continua no âmbito. O catálogo não
concede execução; a unidade não está instalada.

Prosseguir RUN-01/02: integração/migração, proveniência de consentimento e posse durável dos workers, orçamento/autoridade,
fila/leases e associação de contexto de background sem trocar a conversa da UI;
depois execução/verificação e recuperação. O primeiro repositório de objetivos,
derivado da unidade de voz/gestão, passou 1792 Rust/12 ignorados, 1074 JS e
check/TSC/Vite em 650 fontes privadas. Guarda intenção, critérios, dono/contexto,
tarefas/planos/passos propostos e eventos em ficheiros por objetivo, com revisão,
idempotência e tratamento de gravação incerta. Não executa trabalho ao abrir.
PET-01/02/03/04/05 e os restantes requisitos permanecem no âmbito; observações
efetivas por contexto/modelo, cobertura de providers e aceitação nativa continuam
pendentes. O primeiro kernel não representa o motor autónomo completo.
O incremento seguinte tem 653 fontes privadas, 1811 Rust/12 ignorados, 1078 JS e
check/TSC/Vite aprovados: contextos próprios por objetivo, reservas físicas
partilhadas, origem original dos callbacks e correção do censo CLI. Está por
integrar/instalar e não executa objetivos. O goal foi novamente confirmado
`active`; o progresso detalhado está no
[checkpoint de contextos](PETS-VALIDATION.md#evidencia-local)
e no [checkpoint atual da fábrica](PETS-VALIDATION.md#evidencia-local).
A unidade657 seguinte passou 1833 Rust/12 ignorados, 1084 JS e check/TSC/Vite:
revisão nativa exata e entradas de fila/histórico em schema3, sem dispatch ou
permit de execução após reabertura. A continuação é proveniência autenticada de
consentimento, leases/recuperação, integração dos produtores de prontidão e
writer com orçamento aplicado. Ver o
[checkpoint de admissão](PETS-VALIDATION.md#evidencia-local).
A unidade660 passou 1854 Rust/12 ignorados, 1085 JS e check/TSC/Vite: prova de
consentimento nativo e ledger independente de revogação. O endpoint de proveniência
continua sem permitir execução. A continuação imediata é claim/posse durável de
workers com condições frescas e reconciliação, seguida de orçamento no writer e
dispatch. Ver o
[checkpoint de proveniência](PETS-VALIDATION.md#evidencia-local).
A unidade664 passou 1872 Rust/12 ignorados, 1087 JS e check/TSC/Vite: claim/
controlador autenticado, slots físicos partilhados, `stopping` e drenagem sem
conclusão indevida. Continua sem dispatcher ou permit de efeito. O passo seguinte
é reserva/enforcement do orçamento no writer e handoff do hold ao chat privado,
seguido de integração/aceitação nativa e recuperação. Ver o
[checkpoint de leases](PETS-VALIDATION.md#evidencia-local).
O bloqueador CTX-01 permanece pendente antes da ativação da composição. Nenhuma
destas provas privadas fecha RUN-01/PET-01 ou reduz o âmbito integral acima.

A correção funcional OBS-01 pedida no relato dos três terminais foi ampliada
na unidade666: distinguir TTY, cliente/suspensão e conversa carregada do pet,
sem inferir atividade de IA. Passaram 1887 Rust/12 ignorados, 1101 JS e
check/TSC/Vite; [checkpoint](PETS-VALIDATION.md#evidencia-local).
A app instalada continua na .30. O writer/orçamento e handoff do hold prosseguem
sobre esta base666; os 43 requisitos e a entrega continuam por concluir.

A unidade671 acrescenta orçamento persistente e permit de uso único na fronteira
HTTP real, handoff do mesmo recurso ao chat privado e primeira resposta textual
Ollama ao intento original. Passaram 1916 Rust/12 ignorados, 1101 JS e
check/TSC/Vite, incluindo cancelamento de stream já recebido, sem chamadas de
modelos reais. [Checkpoint](PETS-VALIDATION.md#evidencia-local).
A continuação imediata separa prontidão antes do efeito de validade durante IO
longo e integra produtores/controlo nativos; planos/ferramentas/verificação,
recuperação e entrega permanecem pendentes.

Requisitos do utilizador explicitados: controlo/interação de todo o backend de
pets por voz (VOICE-01/02) e permissões/pedidos do agente original coerentes entre
terminal, chat e voz, com política/sandbox/perfil preservados e uma decisão nativa
vencedora (PET-02/03, EXE-02). O plano de backend inclui também consumo por
objetivo, concorrência, reconciliação e migração/rollback, sem reduzir os 43
requisitos nem anunciar estas capacidades como completas. Arquitetura §9.1/11.1
e estado §2.1 são as referências atuais.

A unidade673 corrigiu a validade das chamadas longas: 1928 Rust/12 ignorados,
1101 JS e check/TSC/Vite, com HTTP simulado para prontidão envelhecida versus
política revogada e novas chamadas bloqueadas sem frescura. Descriptor e revisão
de política usam uma captura coerente de configuração; metadados nativos
capturados renovam a cache exata. [Checkpoint](PETS-VALIDATION.md#evidencia-local).
A continuação é o bloqueador CTX-01 de preservação/recuperação da conclusão e
limpeza terminal antes de integrar/entregar a composição. Os 43 requisitos,
voz integral, permissões nativas, restantes executores e aceitação permanecem
ativos; sem instalação, commits, agentes ou modelos reais.

A unidade675 resolveu o bloqueador específico de perda da prova de conclusão
após apagar o WAL final: envelope durável fora da pasta, descoberta/reap mesmo
sem WAL/pasta e revisão nativa só de metadados, preservando ID/dono/CID/revisão.
Passaram 1946 Rust/12 ignorados, 1101 JS e check/TSC/Vite, incluindo 18 regressões
novas de falhas/namespace/identidades e consentimento nativo, sem modelos reais.
[Checkpoint](PETS-VALIDATION.md#evidencia-local).
A continuação reproduz e completa aceitação de restauro parcial/nomes reutilizados,
limpeza interrompida, razões exatas e backup/migração/rollback antes de integração.
O objetivo integral, os 43 requisitos com voz/permissões nativas e a entrega
continuam ativos; a .30 instalada e os dados/permissões/Wi-Fi estão preservados.

O incremento de recuperação nativa mantém 675 fontes e passou 1954 Rust/12
ignorados, 1101 JS e check/TSC/Vite, incluindo 8 regressões novas. Restauro
multi-item interrompido/destinos ocupados/manifesto publicado e cleanup parcial
preservam dados/IDs/ausência de concessões; recusas agora distinguem motivos
verificados sem detalhes internos. [Checkpoint](PETS-VALIDATION.md#evidencia-local).
A unidade seguinte,683 fontes, passou 1966 Rust/12 ignorados, 1101 JS e
check/TSC/Vite sem avisos novos. Os trechos exatos do leitor/sweep .30
confirmaram em dados sintéticos que recusar JSON v5 não protege a quarentena
na inbox antiga; a pasta irmã protegida sobrevive. A revisão nativa de migração
é só de leitura, limitada a dados conhecidos, com identidade/hash/revalidação,
e recusa estado novo/autoridade/provas que exigem outro protocolo.
[Checkpoint](PETS-VALIDATION.md#evidencia-local).
A cópia privada seguinte,686 fontes, passou 1980 Rust/12 ignorados, 1101 JS e
check/TSC/Vite sem avisos novos. São 14 regressões novas: copiar apenas os dados
admitidos, conservar bytes/nomes/pastas, revalidar origem e destino antes de
concluir, conservar cópias parciais e recusar adulterações/aliases de montagem.
Inspecionar uma cópia após reinício não reconstrói a preparação nativa nem
concede publicação/execução. [Checkpoint](PETS-VALIDATION.md#evidencia-local).
O componente de posse nativa seguinte,688 fontes, passou 1992 Rust/13 ignorados
(12 de host e1 helper executado pelo teste de outro processo), 1101 JS e
check/TSC/Vite sem avisos novos. Verifica nome/dono único/PID/utilizador no bus
privado real e mantém flock/identidade corrente do perfil, sem conceder
execução aos pets. [Checkpoint](PETS-VALIDATION.md#evidencia-local).
A publicação privada dos dois destinos,690 fontes, passou 2009 Rust/13
ignorados, 1101 JS e check/TSC/Vite sem avisos novos. Os17 novos ensaios
comprovam cópia/admissão, falhas nos dois destinos, recuperação de dados
completamente copiados, payload mutável após reinício e dois filesystems reais.
[Checkpoint](PETS-VALIDATION.md#evidencia-local).
A ligação efetiva ao arranque/caminhos da fonte privada,693 fontes, passou
2026 Rust/13 ignorados, 1101 JS e check/TSC/Vite, sem avisos novos. Settings,
Chat e Recap são construídos após admissão; dados mutáveis usam o payload,
assets estáveis conservam o caminho e pedidos precoces usam uma fila limitada.
[Checkpoint](PETS-VALIDATION.md#evidencia-local).
A retoma nativa seguinte,695 fontes, passou2040 Rust/13 ignorados,1101 JS e
check/TSC/Vite, sem avisos novos. A intenção anterior e a revisão atual
condicionam a cópia, que preserva prefixos/inodes e staging de metadados
limitado. Meias pastas apenas vazias podem completar; dados divergentes ou
anteriormente admitidos não são reconstruídos da origem antiga.
[Checkpoint](PETS-VALIDATION.md#evidencia-local).
A aceitação de perfis já começou com inventário só de metadados da .30, sem
modificar os dados nem ler conversas/anexos. Referências tipadas, configuração,
escritores, backup/restauro/rollback, sons/composição e aceitação nativa
continuam por resolver.
Os caminhos de produção não foram alterados. CTX-01, entrega e os 43 requisitos
integrais, incluindo voz/permissões nativas, continuam ativos/incompletos.

A biblioteca comum697 passou2054 Rust/13 ignorados,1101 JS e check/TSC/Vite:
conserva histórico de donos indisponíveis/campos da raiz em formato6 inativo,
sem adoção por modelos/objetivos e sem eliminar anexos com referências incertas.
[Checkpoint](PETS-VALIDATION.md#evidencia-local).
A aceitação seguinte verifica normalização dos leitores nativos numa cópia
privada dos JSON atuais; referências externas conservam o caminho e o perfil
original permanece intacto. Backup/rollback, aceitação nativa e o objetivo
integral continuam por concluir.

A aceitação nativa698 passou2061 Rust/14 ignorados,1103 JS e check/TSC/Vite,
sem avisos novos. Os leitores reais preservaram todos os campos guardados
dos três JSON copiados. O reset encontrado no Spotify foi corrigido através
da fábrica: controlos nativos coexistem com assistência de AI explicitamente
configurada, sem alterar modelos/grants ou adotar contexto de outro pet.
[Checkpoint](PETS-VALIDATION.md#evidencia-local).
Segue cópia consistente de todos os dados mutáveis conhecidos e restauro
privado verificado, reutilizando LegacyReview/staging. Instalação, rollback
de aplicação, aceitação nativa e os43 requisitos continuam por concluir.

O incremento698 de backup passou2063 Rust/15 ignorados,1103 JS e check/TSC/Vite:
a captura/restauro nativos verificaram13 ficheiros e7 pastas do perfil legado
real, sem modificar originais nem adotar autoridade de metadados. Backups
históricos são conservados e o lock ativo das métricas permanece no sítio.
[Checkpoint](PETS-VALIDATION.md#evidencia-local).
Continua para reconciliação da composição com o checkout e preparação da
entrega, mantendo drafts/rollback/host/GUI e os43 requisitos integrais ativos.
