# Validação da composição dos pets — 25/10/2026

## 01/11 — Fala nativa e candidata mochi.32 (C6C3)

A11 passou3182 testes Rust/38 ignorados e1333 web/0 ignorados, TypeScript e Vite. A10/nativo283 passou janela errada, ID ausente, texto/idioma adulterados, replay, ACK de outra janela, reload e recolha do líder/filho, seguido de novo pedido válido. A11 apenas remove o caminho Dispatcher sem cancelamento próprio e incrementa a versão. O artefacto final passou o ensaio284; nenhum destes testes comprova áudio PT/EN/misto, Bluetooth ou aceitação humana.

[Composição, provas, entrega e limites](releases/0.5.1-mochi.32.md).

A versão.32 está agora instalada e ativa: nome D-Bus, executável empacotado e admissão nativa verificados; três originais de preferências/conversas e cópias nativas byte-exatas. A.30 foi preservada. O ensaio284 do próprio RPM passou com helper sintético; isto não fecha áudio real nem os49 requisitos.

## 31/10 — Seleção TTS e cancelamento por resposta (C6C2, fontes1000)

Fontes1000 integradas: seleção TTS capturada na ocorrência de Voz e no preview, validade após esperas e cancelamento do renderer por resposta exata. Falha do motor escolhido conserva o texto; só a seleção explícita system pode usar síntese do sistema. [Provas e limites C6C2](../.build/pet-goal-execution-next-r1/VOICE-PLAYBACK-CHECKPOINT-C6C2A3.json).

Composição1000: Voice94pass/2ignored; geral3167pass/38ignored;1332web/0skip/TSC/Vite A2 reutilizados em A3 por identidade dos421 inputs nãoRust. A1 teve uma falha web por traduções sem caller após retirar mensagens de fallback; corrigida em A2 e conservada na prova. HTTP de TTS sintético: corpo/modelo/voice_id, revogação antes do POST, redirect, excesso de bytes e alvo malformado. Não é síntese/áudio real nem aceitação instalada.

## 31/10 — Entradas imutáveis da Voz (C6C1, fontes999)

Fontes999 integradas: entradas de captura/ASR e seleção de inferência imutáveis por ocorrência de Voz, usando os runners e a fábrica existentes. Alterar pet, perfil, endpoint ou preferências durante a espera não redireciona o pedido; o contexto inválido é recusado. O snapshot não renova grants. [Provas e limites C6C1](../.build/pet-goal-execution-next-r1/VOICE-TURN-CHECKPOINT-C6C1A4.json).

Composição999: Voice88pass/2ignored; geral3161pass/38ignored; build597 inputs. Fontes congeladas e idênticas às integradas. A2 teve erro de compilação por retirar a API partilhada com Flow; a falha está conservada e a compatibilidade foi reposta antes das provas finais. Não houve microfone/modelo/TTS real ou aceitação instalada.1329web/TSC/Vite anteriores reutilizados por identidade dos421 inputs e71 outputs preservados.

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

Os nativos usam arquivo sintético7 read-only e admissão nativa; bindings são escolhidos de novo via API nativeUser. Comparam histórico apresentado/retido com intents reais validados por hash. Recusam janela errada, versão antiga, quarta confirmação falsa, overrides e replay; verificam efeito só após intenção durável, callback após publicação, história após sucesso e reinício. Restauro8 usa profundidade efetivamente declarada pela admissão e abre o loader real, conservando seis suspensões.

## R43F10B2C3D2B3C3A2 — comparação nativa original e atual

Privado: 992 fontes/591 entradas nativas. Passaram 55 testes dirigidos, 3134 gerais (38 ignorados), 1326 web/0 ignorados, TypeScript, Vite, build e nativos266/268/269. [Fontes, provas e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-ORIGINAL-OWNER-PARTIAL-CHECKPOINT-R43F10B2C3D2B3C3A2.json).

266/269 comparam os dados apresentados com a intenção realmente retida, distinguem sessão Bound de token reservado, recusam decisão sem versão ou versão errada, testam observação/novo consentimento e reinício.268 apresenta as duas identidades/barramentos após restauro e recusa transformar incerteza em ausência.267 falhou antes do listener pós-reinício; causa do timeout não demonstrada, retorno externo não registado.269 mantém asserções e mesmo binário, usando set_collapsed para mapear o controlador antes do JavaScript. Não é prova de correção instalada desse timeout ou de hardware/humano.

## R43F10B2C3D2B3C2A1 — restauro 7 e suspensão de todas as componentes

Composição privada: 992 fontes/591 entradas nativas. Passaram 43 testes do módulo no A1. A execução geral C2 passou 3130 testes e teve duas expectativas antigas corrigidas no reteste dirigido; 38 ignorados. A1 altera apenas cfg(test), sem nova execução geral/build/nativa. Build e ensaio nativo 265 de C2, web1324/TSC/Vite B3 são reutilizados por identidade explícita dos respetivos inputs. [Fontes, provas e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-RESTORE-PARTIAL-CHECKPOINT-R43F10B2C3D2B3C2A1.json).

O código Rust comprova ainda versões 1/2 com revisão global, versões com diário com seis suspensões, recibos verificados/incertos sem mutação e publicação global pendente byte-exata. O ensaio265 usa comandos de restauro/admissão, IPC e UI reais com perfil/broker sintéticos; não fecha aceitação instalada/humana/Wayland físico. As falhas 263/264 estão preservadas e delimitadas.

## R43F10B2C3D2B3B8 — recuperação dos atalhos e leitores antigos

Composição privada: 992 ficheiros de fonte e 591 entradas nativas; backend e binário B7, uma correção de fixture cfg(test) B8. 52 testes dirigidos B8 passaram; a execução geral B7 teve 3129 testes aprovados, uma falha de fixture e 38 testes ignorados; a falha foi conservada e corrigida com reteste dirigido.Os 1324 testes web, TypeScript e Vite de B3 e os 18 testes D-Bus de B5 reutilizados por identidade explícita; build B7 e ensaios nativos 261 (X11) e 262 (portal) passaram. Não houve nova execução dessas suites no B8. [Fontes, provas e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-RECONCILIATION-PARTIAL-CHECKPOINT-R43F10B2C3D2B3B8.json).

261 usa plugin X11 em Xvfb com callbacks reais no servidor virtual;262 usa portal sintético e Tauri/WebKit reais. Ambos cobrem três confirmações, recusas de janela/caller/ACK/token, recuperação atual/restart, nova aprovação distinta, leitor 6 e backup 7. Corte pendente deriva de artefactos nativos, mas é sintético.260 conserva timeout JS na terceira instância, sem causa atribuída. Sem teclado/compositor físicos, áudio, Bluetooth, aceitação humana/instalada ou restauro 7 nativo.

## R43F10B2C3D2B3A1 — resultado pendente e publicação

51dirigidos/0ignorados,16D-Bus/0ignorados,3118Rust/36ignorados,1318web/0skip/TSC/Vite e build586/100avisos.257portal e259X11 verificam o executável efetivo e a sequência pendente→publicação→final.259 inclui estado pendente sintético derivado de artefactos nativos reais, preservação e recusa old983/current; não prova crash/powerloss físico.254/255montaram binário errado;256parou no probe de uma webview auxiliar. Diagnósticos retidos, produto sem alterações nesses retestes. [Provas e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-PUBLICATION-PARTIAL-CHECKPOINT-R43F10B2C3D2B3A1.json).


## R43F10B2C3D2B2B3 — aprovação dos atalhos e suspensão X11

45dirigidos,15D-Bus privados,3112Rust/35ignorados e build585/100avisos passaram.1317web/0skip/TSC/Vite B2 reutilizados em B3 por identidade não-Rust/bundle. Nativos252X11 e253portal passaram; falhas248/249das fixtures e251do produto estão preservadas e explicadas. Não são provas de hardware/áudio/humano/instalação; B3 está aberto. [Checkpoint e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-APPROVAL-PARTIAL-CHECKPOINT-R43F10B2C3D2B2B3.json).


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

27/10/2026:970fontes privadas/577inputs. `Target` identifica o slot pelos dev/inode/birth-time da base retida e nomes relativos, com domínio2, sem fallback por caminho. Substituição da base muda a identidade; rename/alterações normais mantêm-na. O plano vivo continua a prender alvo/launcher/revisões. Origem legada1 é desconhecida: revisão específica/4ACK observa só o presente, conserva raw Unknown e não afirma equivalência física/sucesso passado nem reativa. Elegibilidade requer recurso físico, perfil e revisão de dados exatos. A intenção/diário e o controlo funcional reutilizam o percurso existente. Settings6 protege a nova semântica e é conservado em migração/backup/restauro.

45dirigidos/3082Rust21ignorados/build577 passaram. E4 executou24web/TSC/Vite;1302testes web E3 são reutilizados (o TSC E3 falhou; não houve nova execução completa E4). Nativo239 passou140.45s: arquivo5 real sintético readonly, D5 anterior abre5 e recusa6 sem alterar roots, decisão legada própria, freshSet/reinício, troca física conservando caminho/perfil, nova revisão e efeito só na base nova, backup6 byte-exato e restauro/custody/config/admissão depth4 com6componentes suspensas e observações antigas inelegíveis. E1 confirmou a falha real; E2/E3 conservam falhas intermédias de guardas/fixture/tipagem corrigidas. [Prova, fontes e limites](../.build/pet-goal-execution-next-r1/SETTINGS-AUTOSTART-PHYSICAL-PARTIAL-CHECKPOINT-R43F10B2C3D1B2E4.json).

Segue revisão própria dos atalhos sobre `plan/apply`/portal/registry e o mesmo diário; Voz/Móvel/Hooks/Credenciais continuam suspensos. Root834/instalação .30 intactos. Sem aceitação humana/login/hardware/powerloss, publicação ou fecho dos49requisitos.

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

<a id="memoria-chat-contexto-r43f2e2"></a>

## R43F2E2 — revisão manual e contexto vinculado, SDK854

Fonte [854/E2b](../.build/pet-goal-execution-next-r1/source-manual-memory854-r43f2e2b.json), snapshot imutável `manual-memory-r43f2e2b-source`, [decisão](../.build/pet-goal-execution-next-r1/HUMAN-MEMORY-SURFACE-DECISION-R43F2E.json) e [checkpoint](../.build/pet-goal-execution-next-r1/MANUAL-MEMORY-PARTIAL-CHECKPOINT-R43F2E2.json). Reutiliza catálogo/CAS, captura/revisão opaca, transporte/proveniência nativos, sessões e renderização de registos E1; não cria runtime ou autoridade paralelos.

Ambos os compositores têm seleção de recordações do pet e da conversa própria, até16 no conjunto, com pergunta/modelo/anexos capturados antes de esperar. O rascunho fica por enviar até à revisão completa e à escolha explícita do âmbito. Mudanças de dono/modelo/pedido/anexos invalidam a revisão; uma resposta tardia não substitui o pet atual. A sessão externa de terminal não recebe este percurso de memória. Um hash nativo vincula as referências tipadas de contexto e `documentSequence` antes de consumir o token; não lê ficheiros para construir o hash. A projeção de destinos API usa a mesma função de escolha de rota do envio, com limites de observação explícitos.

| Prova | Resultado / alcance |
| --- | --- |
| Rust completo/check E2a |2638 passaram/21 ignorados; todos os488 inputs nativos são byte a byte os mesmos em E2b. Só o painel de revisão mudou para resolver a visibilidade. [Full](../.build/pet-goal-execution-next-r1/manual-memory-full-r43f2e2a-RESULT.json), [check](../.build/pet-goal-execution-next-r1/manual-memory-check-r43f2e2a-RESULT.json) |
| Web E2b |1212 JS, TSC e Vite passaram: seleção de dois âmbitos, rascunho, destinos, cancelamento, dados hostis, modelo/anexos/dono imutáveis e recusa de empréstimo por sessão externa. [Resultado](../.build/pet-goal-execution-next-r1/manual-memory-web-r43f2e2b-RESULT.json) |
| App E2b |Build com488 inputs nativos inalterados e bundle novo exato; avisos de compilação preservados. [Resultado](../.build/pet-goal-execution-next-r1/manual-memory-app-build-r43f2e2b-RESULT.json) |
| Tauri143 |Seis percursos reais da janela de conversa: Ollama, LMStudio, Codex, Copilot, Gravity e Notion com assistência Codex. Dois âmbitos/revisões, caixa inicialmente desmarcada,30 verificações de acesso por posição/hit-test, scroll e capturas observadas. Texto em streaming não conta como conclusão: espera estado inativo e recibo nativo. Contexto/janela/revisão trocados recusados antes de envio; original usado uma vez; reinício não renova token.3 HTTP/4 prompts CLI sintéticos. [Prova](../.build/pet-goal-execution-next-r1/APP-NATIVE-MANUAL-MEMORY-R43F2E2C.json) |
| Tauri144 |Na mesma fonte/binary/bundle E2b: memória local/CLI/serviço, decisões e recolha, contexto/corpo/concessão de objetivos, revogação, redirecionamentos, reinício e leitor844.13 HTTP/17 prompts CLI sintéticos; zero pedidos ao destino alternativo. [Prova](../.build/pet-goal-execution-next-r1/APP-NATIVE-MANUAL-MEMORY-REGRESSION-R43F2E2B.json) |

Falhas preservadas: [140](../.build/pet-goal-execution-next-r1/MANUAL-MEMORY-NATIVE140-SURFACE-FINDING.json) não ativou o inline em Xvfb e terminou antes de enviar; não certifica funcionamento nem falha na instalação. [142](../.build/pet-goal-execution-next-r1/MANUAL-MEMORY-NATIVE142-FUNCTIONAL-FINDING.json) revelou revisão fora da área visível e um teste que tomava streaming como conclusão. E2b limita o painel e oferece scroll, escondendo catálogos durante a revisão;143 exige controlos alcançáveis e estado final. As capturas de dados sintéticos ficam privadas em `.build/`, sem publicação.

A interface inline tem provas de integração de controller/unidade, não aceitação nativa por este ensaio. IPC/DOM Tauri são reais, mas cliques/consentimentos são automatizados e todos os motores/protocolos são sintéticos. Não se verificaram cofre/contas/modelos reais, quota, áudio, hardware ou entrega instalada. API/HTTP sintéticos não são prova de credenciais reais. Anexos múltiplos e sua captura são provados em controllers/contextos tipados, sem fechar a aceitação instalada do seletor de ficheiros. Retoma de objetivos com memória, backup/migração/downgrade/adoção/entrega e os critérios integrais dos49 requisitos continuam por concluir. Não houve commits, agentes auxiliares ou alterações ao anfitrião/instalação.

<a id="memoria-controlos-objetivos-r43f2e1"></a>

## R43F2E1 — controlos de memória e consentimento de objetivos, SDK850

Fonte: [mapa850](../.build/pet-goal-execution-next-r1/source-human-goal-memory850-r43f2e1b.json), snapshot imutável `human-goal-memory-r43f2e1b-source`, [decisão](../.build/pet-goal-execution-next-r1/HUMAN-MEMORY-SURFACE-DECISION-R43F2E.json) e [checkpoint](../.build/pet-goal-execution-next-r1/HUMAN-GOAL-MEMORY-PARTIAL-CHECKPOINT-R43F2E1.json). Não substitui as provas de transportes anteriores nem a instalação.

O mesmo catálogo humano apresenta conteúdo integral como texto, dono/âmbito, origem, sensibilidade, validade e revisão. Consulta explícita, seleção até16 por IDs/revisões, paginação vinculada à revisão/pesquisa, edição CAS e eliminação em dois passos; ACK perdido não causa repetição. Alterar/atualizar o catálogo não substitui silenciosamente a revisão selecionada. A criação captura seleção, dono e plano antes da primeira espera.

A revisão de admissão mostra os registos completos, destino/modelo e âmbito limitado do objetivo. Uma caixa inicialmente desmarcada distingue `boundedGoalContext` de consentimento manual/CLI e da execução. O controller recusa âmbito ausente/errado antes de metadados/admissão; uma confirmação retida conserva token, scope, limites e dono originais. O botão de resposta usa a operação existente para Ollama e LMStudio, sem promover outros adaptadores.

| Prova | Resultado e limite |
| --- | --- |
| Rust completo E1a |2634 passaram/21 ignorados; os487 inputs nativos são byte a byte os mesmos em E1b. A diferença B limita-se à vista e ao teste funcional LMStudio. [Resultado](../.build/pet-goal-execution-next-r1/human-goal-memory-full-r43f2e1a-RESULT.json) |
| TypeScript E1b |1200 JS, TSC e Vite passaram; troca de dono durante espera, revisões imutáveis, texto hostil, ACK perdido, consentimento e paridade local. [Resultado](../.build/pet-goal-execution-next-r1/human-goal-memory-web-r43f2e1b-RESULT.json) |
| Check/build E1b |487 inputs nativos inalterados, bundle novo exato; avisos registados nos logs. [Check](../.build/pet-goal-execution-next-r1/human-goal-memory-check-r43f2e1b-RESULT.json), [build](../.build/pet-goal-execution-next-r1/human-goal-memory-app-build-r43f2e1b-RESULT.json) |
| Tauri privado139 |Catálogo/criação/seleção/revisão/caixa própria/fila/reserva/resposta/libertação através dos controlos reais para Ollama e LMStudio. Eliminar antes de confirmar recusa admissão/inferência; corpo/hash/revogação durante HTTP/reinício/leitor844 voltaram a passar. Cinco inferências sintéticas no conjunto. [Prova](../.build/pet-goal-execution-next-r1/APP-NATIVE-HUMAN-GOAL-MEMORY-R43F2E1C.json) |

IPC e DOM Tauri reais em WebKit/Xvfb isolado; cliques e consentimentos de fixtures automatizados, sem aceitação humana, contas reais, quota, áudio, hardware ou aplicação instalada. Os21 testes ignorados continuam discriminados no checkpoint. Não há shell livre ou modelo remoto autorizado por estes controlos. Revisão manual inline/sessão, cobertura exata dos destinos API na revisão humana, retoma com memória, backup/migração/downgrade/adoção e entrega continuam por concluir. A raiz834 conserva a barreira anterior; não transportar esta prova para um caller/provider não suportado.

<a id="api-memoria-destino-exato-r43f2d"></a>

## APIs de memória e destino exato dos cinco transportes — R43F2D privado

**Atualização R43F2D validada em privado — 25/10:** SDK846 prova os envios de memória de Anthropic/OpenAI/Google através da implementação nativa comum, com HTTP loopback sintético, dados separados da pergunta e recusa de destino/modelo/credencial/pergunta alterados. Foi retirado o opt-in de fallback automático Anthropic; APIs e percursos locais usam o destino exato sem seguir redirecionamentos. Passaram2632 Rust/21 ignorados, check/build e Tauri136:13 pedidos HTTP e17 prompts CLI sintéticos, incluindo memória/decisões/recolha/retoma dos três CLI e serviço separado, objetivos locais e dois redirecionamentos recusados sem pedidos ao destino alternativo. Não certifica contas reais, routing interno do fornecedor, UI humana ou instalação. Raiz834 e instalação .30 preservadas. Seguem interface humana/callers, retoma de objetivos com memória e adoção/migração/entrega; R43 e os49 requisitos continuam abertos.

Fontes: `source-api-memory-wire846-r43f2d2.json`, snapshot `api-memory-wire-r43f2d2-source`,846 fontes/486 inputs nativos; sete deltas face845/C6. [Checkpoint](../.build/pet-goal-execution-next-r1/API-MEMORY-WIRE-PARTIAL-CHECKPOINT-R43F2D.json) e [Tauri136](../.build/pet-goal-execution-next-r1/APP-NATIVE-API-MEMORY-WIRE-REGRESSION-R43F2D.json). Reutiliza Capture/Prepared/Reviews, canal de memória de uma utilização, preparação de documentos/calendários, builders, parsing/commit e cliente HTTP. As duas funções públicas API continuam a resolver a credencial/destino nativos; helpers cfg(test) exercitam a mesma implementação após resolução com parâmetros sintéticos, sem novo IPC/configuração de endpoint ou consulta do cofre.

Foram identificadas e corrigidas duas divergências no SDK. O builder Anthropic solicitava `fallbacks:default` e o cliente adicionava um beta de fallback de servidor; ambos foram removidos, sem substituir o modelo pedido ou tratar recusa como sucesso. O cliente genérico segue redirects por omissão; um307 podia reenviar o corpo para outro endereço após a revisão. `net::exact_destination_client` partilha o builder normal mas recusa redirects nas APIs/listagens e nos envios/listagens/calendário/protocolo/metadados locais. O cliente genérico dos outros consumidores e o local_client autónomo conservam os seus contratos. Não houve mudança de conta, modelo escolhido, preferência, hook ou instalação.

Os cinco casos API novos passaram primeiro em D1 e depois na suite coerente D2: três APIs nativas com chave/modelo/caminho exatos e dados como mensagem user separada; pergunta/história públicas sem cópia automática de notas; uso único e envio normal posterior conservados; doze alterações de destino/modelo/chave/pergunta recusadas antes de HTTP; revogação em headers e corpo nos três clientes;307 sem encaminhamento; recusa sem fallback/reentrega. São18 pedidos HTTP sintéticos nesses casos, além dos restantes testes da suite. Os2632 Rust/21 ignorados, comandos, hashes e nomes dos ignorados constam do checkpoint. O campo modelCalls dos wrappers de compilação declara ausência de modelos reais, não conta tráfego sintético.

Tauri136 usa IPC real em WebKit/Xvfb privado: oito pedidos manuais locais (seleção/revogação/decisões/comando próprio),17 prompts para Codex/Copilot/Gravity e serviço Notion sobre transporte Codex, três pedidos de objetivo local e duas recusas307 manuais. O destino alternativo recebeu zero pedidos. Foram conservados ID/owner/âmbito antes do prompt, retoma sem reenviar notas brutas, isolamento do serviço, recolha dos quatro CLI após revogação, incerteza de ID Gravity, recusa de prompt quando a gravação do ID falha, recibo sem inferência repetida, memória revogada durante HTTP e leitura antiga844 sem reescrever Record12. Protocolos/modelos são sintéticos, zero quota real. Não exercita rádio, áudio, cofre/conta operacional ou UI humana.

O modelo **pedido** e o endpoint permanecem os revistos, sem opt-in de fallback pelo Coucou. O servidor pode resolver aliases e executar os seus serviços internos; esta prova não certifica o modelo interno nem contenção dos dados dentro de um fornecedor. Web search Anthropic e calendário existentes não foram eliminados. A interface de revisão ainda tem de apresentar a cobertura real de uso, sem prometer apagar dados já recebidos. Nenhuma nova capacidade autónoma online/CLI ou acesso a memória por modelos foi concedida por estes testes.

Seguem seleção/revisão/confirmação humanas comuns, incluindo o âmbito próprio de objetivo e origem/destino/limites, projeções nativeUser/taskRuntime/providerModel, retoma com memória e migração/backup/downgrade/adoção. A matriz dos49 requisitos permanece aberta na aceitação integral; não atualizar a instalação ou publicar por esta prova privada.

<a id="consentimento-objetivo-corpo-real-r43f2c"></a>

## Consentimento do objetivo, concessão e corpo real — R43F2C privado

**Atualização R43F2C validada em privado — 25/10:** SDK845 liga o consentimento próprio do objetivo à concessão assinada existente. A proposta expõe dados, origem, destino/modelo e âmbito para revisão; a mensagem efémera entra no corpo real antes do hash e da reserva de orçamento. Passaram30 casos de contexto dentro dos2627 Rust/21 ignorados, check/build e Tauri privado135 com Ollama/LM Studio sintéticos. A revogação durante HTTP impede novo uso e publicação tardia sem repetir inferência; conserva a tentativa e os limites dos dados já enviados. A concessão com memória exige Record12; reinício e recusa do leitor844 preservaram os bytes. A raiz834 e a instalação .30 continuam preservadas. Faltam três rotas HTTP API, interface humana/callers, retoma com memória e adoção/migração/entrega coerentes. R43 e os49 requisitos permanecem abertos.

Fontes exatas: `source-goal-memory-signed845-r43f2c6.json`, snapshot `goal-memory-signed-r43f2c6-source`,845 fontes/485 inputs nativos. Prova recuperável: [checkpoint R43F2C](../.build/pet-goal-execution-next-r1/GOAL-MEMORY-SIGNED-PARTIAL-CHECKPOINT-R43F2C.json); [Tauri135](../.build/pet-goal-execution-next-r1/APP-NATIVE-GOAL-MEMORY-SIGNED-R43F2C2.json). São18 deltas Rust face844, incluindo as fixtures. Reutilizam-se Review/Approved/Authority/ACK, ContextBinding/Capture/Fence, posse física, writer, orçamento, tentativas, resultados e recuperação; não há segundo ledger ou scheduler.

`ConfirmInput.memoryConsent=boundedGoalContext` é distinto de um turno manual/sessão CLI. Falta de âmbito recusa antes de observar/criar autoridade ou consumir a revisão; âmbitos manuais/sessão e consentimento acrescentado a revisão sem memória são recusados. Só o objeto nativo original permite confirmar. `AdmissionSpec.memoryData` assina destino/modelo e teto256KiB; as referências/revisões/dono/origem ficam no contexto imutável. Record12 é condicional à concessão com memória; preparação conserva Context2/Record11 e ausência de seleção conserva formatos/fingerprints anteriores. Leitura de ACK já publicado não renova autorização.

O writer confirma destino antes de metadados, compõe uma mensagem de dados `role=user` separada da pergunta, volta a validar destino/conteúdo/limite e só depois calcula o hash do corpo completo e reserva/inicia a tentativa. O teste de histórico perto256KiB demonstra que história+dados+envelope não passam para inferência ou orçamento se o corpo exceder o limite. A injeção não copia automaticamente notas para a pergunta, journal ou autoridade; isto não impede que uma resposta autorizada cite dados. Metadados/HTTP não concedem ferramentas e texto do modelo não verifica critérios nem conclui o objetivo.

Os30 casos de contexto e a suite2627/21 ignorados passaram na mesma fonte congelada; lista dos21 ignorados e comandos/resultados estão no checkpoint/log. Tauri135 usa IPC real e WebKit/Xvfb privado: duas revisões/concessões e pedidos locais, mesmo hash observado no HTTP e no ledger, recibos sem reinferência, terceiro pedido revogado durante HTTP sem resposta tardia gravada/replay, recolha pelo dono original, reinícios e leitor844 a recusar sem reescrever. São três inferências sintéticas, zero modelos/quotas reais. Não prova UI humana, áudio, contas operacionais ou instalação. A libertação pública conserva `recoveryRequired`, não inventa conclusão ou desfaz efeitos.

Falhas preservadas e corrigidas com novas fontes/tentativas: C1 teve quatro fixtures editadas antes de o runner terminar e não é prova de entradas imutáveis; C2 abriu o catálogo nativo em vez do catálogo da fixture; C3/C4 usaram rótulos curtos no histórico público em vez dos textos grandes pretendidos. C5 expôs um `Busy` numa leitura concorrente de uma fixture antiga de Stop; C6 usa espera limitada apenas para o mesmo `Busy`, mantendo prova, IDs, argumentos e verificações de incerteza. A build C4 ainda em fila foi cancelada antes de criar container. Tauri134 tentou reabrir contexto sob a posse ativa original e recebeu `Busy`;135 só o observa depois da libertação. Nenhuma guarda de produção foi afrouxada para fazer passar esses testes. Ver findings e resultados negativos no checkpoint.

A barreira F1 da raiz não foi removida. No SDK, o refinamento F2 só admite os dados com consentimento tipado, concessão e todas as guardas nativas; UI/callers e composição restante ainda bloqueiam adoção. Seguem três APIs HTTP sintéticas, regressões locais/CLI, seleção/revisão/confirmação humanas, projeções nativeUser/taskRuntime/providerModel, retoma e migração/backup/downgrade/adoção. Sem atualização da instalação, commits, pushes, agentes adicionais ou mudanças do anfitrião.

As secções seguintes registam composições anteriores e os respetivos limites históricos.

<a id="revisao-nativa-dados-r43f2b"></a>

## Revisão nativa de dados no fluxo de admissão — R43F2B privado

**Atualização R43F2B validada — 25/10:** SDK844 liga a captura nativa de memória à preparação da revisão de admissão existente. A proposta mostra os registos completos, origem, destino/modelo e consentimento exigido; compara privadamente credenciais, conserva a revisão original e revalida os dados. Passaram24 testes dirigidos,2620 Rust/21 ignorados,check/build e Tauri privado133 com Ollama/LM Studio sintéticos. A aprovação antiga continua recusada antes de consumir a revisão/criar autoridade; recusa explícita, reinício e leitor840 foram verificados. A raiz834 e a instalação .30 permanecem preservadas. Faltam consentimento tipado na concessão assinada, corpo real anterior ao hash/reserva, três APIs HTTP, superfície humana/callers e adoção/entrega. Não há fecho de R43 ou dos49 requisitos.

Mapa `source-native-goal-data-review844-r43f2b.json`,844 fontes/484 inputs nativos e oito deltas face843: DTO portátil, material nativo e produtores/caches de revisão ligados ao percurso existente. O fingerprint sem dados mantém o formato anterior. O fingerprint com dados e a comparação semântica incluem o preview, mantendo a credencial só no objeto nativo. Bare `Evidence::observe`, aprovação antiga e formato de admissão continuam recusados para memória; nenhum consentimento autónomo é reconstruído de JSON.

Provas: `.build/pet-goal-execution-next-r1/NATIVE-GOAL-DATA-REVIEW-PARTIAL-CHECKPOINT-R43F2B.json` e `APP-NATIVE-GOAL-DATA-REVIEW-R43F2B3.json`. Tauri133: dados completos e destino efetivo para os dois motores, duas recusas de aprovação antiga sem autoridade/consumo, repetição devolve token/prazo originais e Decline termina o pedido. Os esquemas11/2 selecionados e2/1 sem memória, âmbitos/CIDs, ausência de notas brutas em goal/header/journal, reinício, exclusão e recusa do leitor840 conservaram os bytes. IPC real em WebKit/Xvfb privado; providers/metadados são sintéticos e não houve inferência/quota/áudio ou perfil/display do anfitrião.

Falhas preservadas:131 parou no preflight porque o PID histórico da instalação já não existia; não iniciou o container.132 encontrou a fixture `/api/show` sem `control/mode.json`: o serviço falhou e o descritor corretamente recusou com providerUnavailable/notReady.133 inicializou só esse modo sintético e passou com o mesmo código, script e binário. Não repetir suites Rust nem corrigir o backend para ocultar essa falha de fixture. A baseline instalada passou a ser capturada dinamicamente; no ensaio positivo o executável .30 e o conjunto PID/startTicks mantiveram-se. A observação separada de uma entrada Podman antiga Running com PID ausente não foi tratada como estado deste runner e não causou remoção ou restart.

Não existe ainda confirmação tipada de objetivo na concessão nem mensagem de memória no writer autónomo. A UI humana/callers e três APIs HTTP continuam pendentes. Segue F2C com autoridade/ACK/orçamento/contexto originais e corpo antes de hash/reserva, depois aceitação/adopção coerente de R43, sem substituir a instalação. O novo Guardian é contrato, sem pet operacional ou permissões ativas.

<a id="revisao-dados-objetivo-r43f2a"></a>

## Contrato nativo de revisão de dados de objetivo — R43F2A privado

SDK843, mapa `source-goal-memory-review843-r43f2a.json`: 843 fontes, sete deltas face841 e 483 inputs nativos. Novos módulos `pet_runtime/memory_review.rs` e `memory_review_tests.rs`; captura/referências e serialização comuns de `pet_memory`, sem novo endpoint ou autoridade paralela. O objeto nativo de revisão não é Clone/Deserialize nem um permit de execução. O preview expõe registos completos/destino e mantém a credencial privada; o consentimento tipado `boundedGoalContext` distingue turno/sessão manual.

Passaram 21 testes dirigidos de contexto (cinco novos), 2617 Rust/21 ignorados e cargo check. Os runners originais59931/56954/88108 foram consumidos; inputs imutáveis. Fixtures de ambos os motores locais cobrem origem pública exata, ausência de origem inventada no modo dedicado, credencial/modelo/endpoint alterados, intenção original alterada, exclusão/edição/expiração, mudança não relacionada do catálogo, bytes restaurados após revogação e nil sem leitura de catálogo ou resolução de motor. Notas brutas não entraram em goal/header/journal. Zero chamadas a modelos/quota ou alterações da instalação.

Provas: `.build/pet-goal-execution-next-r1/GOAL-MEMORY-REVIEW-PARTIAL-CHECKPOINT-R43F2A.json` e resultados `goal-memory-review-{targeted,full,check}-r43f2a-RESULT.json`. AST limitado de 24 ficheiros:1354 nós/4876 relações, sem extração semântica/modelos. A análise sintática não prova integração de fluxo.

Não há build/app Tauri nova ou aceitação humana por este incremento. A admissão com memória continua recusada em `Evidence::observe` e `AdmissionSpec::binding_format`. Seguem integração de revisão/consentimento na transação existente, destino assinado e mensagem antes do hash/reserva canónicos; APIs HTTP, superfície humana, projeção de callers, adoção/migração/entrega continuam pendentes. Guardian tem contrato, sem implementação/prova operacional. Os43 requisitos e GUA-01..06 não fecham.

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

<a id="retoma-memoria-importacao-r43f4"></a>
## Retoma com memória e proteção da importação antiga — R43F3/F4

Checkpoint recuperável: [MEMORY-MIGRATION-PARTIAL-CHECKPOINT-R43F4.json](../.build/pet-goal-execution-next-r1/MEMORY-MIGRATION-PARTIAL-CHECKPOINT-R43F4.json). Fontes exatas: [mapa854/F4](../.build/pet-goal-execution-next-r1/source-memory-migration854-r43f4a.json) e snapshot `memory-migration-r43f4a-source`. A raiz834, o índice e a instalação .30 permanecem iguais; nenhum commit/push/merge ou alteração do anfitrião foi feito.

| Prova / comando | Resultado e limite |
| --- | --- |
| [Tauri146](../.build/pet-goal-execution-next-r1/APP-NATIVE-SELECTED-MEMORY-CONTINUATION-R43F3A.json), `run-selected-memory-continuation-r43f3a.py r43f2e2b` | Fonte/binário E2b exatos;45,18s;7HTTP sintéticos. Objetivos Ollama e LMStudio, memória de pet/conversa, dedicado/snapshot, recuperação após terminar o dono original, novo consentimento e orçamento adicional; dados/revisões e cobranças anteriores conservados. |
| Planeamento146/147 | Duas rondas e duas decisões nativas de receita distintas; pausa/volume18 na primeira deixa um critério por cumprir, a segunda verifica volume17. Hashes das tentativas correspondem ao corpo HTTP real com memória. Leitor MPRIS privado; nenhum efeito no som da estação. |
| Revogação146/147 | Eliminação durante HTTP retido recusa a publicação tardia e o replay, conservando a tentativa e consumo. Não promete apagar dados já recebidos pelo destino. Reinícios/leitor844 recusado não alteram os ficheiros. |
| [Baseline antes da correção](../.build/pet-goal-execution-next-r1/memory-migration-negative-r43f4-RESULT.json), `cargo test … legacy_import_` |1passa/2falham: a admissão aceitava memória entregue sob versão antiga e excluía catálogos recentes de runtime. Fontes/testes anteriores congelados. |
| [Rust completo F4](../.build/pet-goal-execution-next-r1/memory-migration-full-r43f4a-RESULT.json), `cargo test --locked --offline --manifest-path src-tauri/Cargo.toml` |2641passam/0falham/21ignorados;146,76s;488 inputs nativos imutáveis. Inclui16 combinações versão1..4/campo/ativo-arquivado,5 catálogos atuais/futuros e recusa v7 sem escrita; controlos legados positivos existentes também passam. Não é migração do perfil pessoal. |
| [check](../.build/pet-goal-execution-next-r1/memory-migration-check-r43f4a-RESULT.json) e [app build](../.build/pet-goal-execution-next-r1/memory-migration-app-build-r43f4a-RESULT.json) | Exit0 e mesmos488 inputs exatos; avisos preservados nos logs. Os1212JS/TSC/Vite E2b não foram repetidos: fontes web e bundle42 inalterados. |
| [Tauri147](../.build/pet-goal-execution-next-r1/APP-NATIVE-MEMORY-MIGRATION-COMPOSITION-R43F4A.json), `run-memory-migration-composition-r43f4a.py r43f4a` | Nova composição854/F4,45,98s,7HTTP sintéticos; repete a aceitação de continuação porque mudou a admissão de startup. IPC nativo real, escolhas automatizadas e motor sintético;0quota/modelos reais e sem montar perfil/display do utilizador. |

**Falha preservada:** [Tauri145](../.build/pet-goal-execution-next-r1/APP-NATIVE-SELECTED-MEMORY-CONTINUATION-R43F3.json) chegou a dois planos/quatro operações e verificação final assinada verdadeira, mas o objetivo ficou leased até ao timeout. Não capturou estado/razão do driver.146 acrescentou diagnóstico e passou, assim como147; não foi feita alteração de produto para essa falha, e a causa exata continua desconhecida. [Observação](../.build/pet-goal-execution-next-r1/SELECTED-MEMORY-CONTINUATION-NATIVE145-FINDING.json).

Reutilizados recuperação/continuação/admissão/revisão, Authority/histórico/orçamento, Owned/writer/driver/decisões e LegacyReview/startup/publicação. O delta F4 é apenas `pet_data/inventory.rs`, `tests.rs` e `legacy_tests.rs`; não altera schemas nem cria outro runtime. Não fecha backup/restauro completo, inline nativo, aceitação humana/provider/hardware/instalada ou os49 critérios integrais. [Próxima dependência e limites físicos](../.build/pet-goal-execution-next-r1/COHERENT-RESTORE-NEXT-SOURCE-AUDIT-R43F4.json).

<a id="backup-payload-conclusao-r43f5"></a>
## Backup de payloads e conclusão sob concorrência — R43F5

[Checkpoint exato858/F5D](../.build/pet-goal-execution-next-r1/CURRENT-PROFILE-BACKUP-PARTIAL-CHECKPOINT-R43F5.json) · [mapa de fontes](../.build/pet-goal-execution-next-r1/source-current-backup858-r43f5d.json) · snapshot `current-backup-r43f5d-source`. Reutilizados inventário/cópia/manifesto/readback, startup/Active/dono físico e a transação Authority/Store de conclusão. Não há novo scheduler, ledger de execução ou fonte paralela de pets.

| Prova | Resultado / âmbito |
| --- | --- |
| [check-tests D](../.build/pet-goal-execution-next-r1/current-backup-check-tests-r43f5d-RESULT.json) | Exit0;492 inputs nativos imutáveis;15,09s. |
| [Critérios dirigidos D](../.build/pet-goal-execution-next-r1/current-backup-completion-target-r43f5d-RESULT.json) |23passam/1ignorado. Locks reais do catálogo/autoridade, Unknown transitório, Stop com gate livre entre tentativas e Busy persistente limitado; mesma prova e dois efeitos físicos, sem nova inferência. Inclui os ensaios anteriores de falha antes/depois de publicação e worker ainda ativo. |
| [Rust completo D](../.build/pet-goal-execution-next-r1/current-backup-full-r43f5d-RESULT.json) |2655passam/0falham/21ignorados;126,43s incluindo compilação/espera. Os ignorados e avisos continuam registados; não são aceitação de modelos/hardware reais. |
| [Build D](../.build/pet-goal-execution-next-r1/current-backup-app-build-r43f5d-RESULT.json) | Exit0,mesmos492 inputs;225,79s de runner incluindo espera. Web/TS sem alterações: bundle42 e1212JS/TSC/Vite E2b reutilizados com hashes exatos. |
| [Nativo149](../.build/pet-goal-execution-next-r1/APP-NATIVE-CURRENT-PROFILE-BACKUP-R43F5B2.json) | CLI Rust real/DBus privado,3,47s.81ficheiros sintéticos, perfis/locks ocupados, hash errado, origem/destino trocados, proveniência, modos privados, anexos com nomes repetidos/não UTF-8, arquivo parcial e adulterado.0modelos/quota. A build C é [byte-idêntica à B](../.build/pet-goal-execution-next-r1/CURRENT-BACKUP-NATIVE-BINARY-EQUIVALENCE-R43F5C.json); o nativo D é separado abaixo. |
| [Nativo151](../.build/pet-goal-execution-next-r1/APP-NATIVE-CURRENT-BACKUP-COMPOSITION-R43F5D.json) | Nova composição D,45,46s. Arranque normal e importação legada, IPC real, retoma com memória/duas rondas/duas aprovações de receita,7HTTP sintéticos e exatamente4efeitos MPRIS privados; revogação/reinício/recusa do leitor844. Depois termina/recolhe o backend próprio e faz revisão/confirmação/inspeção de backup do mesmo perfil:78ficheiros byte-exatos, originais preservados e0chamadas adicionais a modelos. |

**Falhas preservadas e causa delimitada:** A check-tests falhou por falta de import `Layout` num teste legado após generalizar o helper; corrigido antes deB.148 falhou no setup (pasta inbox inexistente) antes de qualquer chamada nativa;149 usa novo perfil/setup. Full B:2648passam/2falham/21ignorados em arranque de peers Gravity; o errno original não foi capturado. C reproduz ETXTBSY com um writer real, faz apenas preflight limitado do executável sintético e passa2651/21; não é correção do protocolo/conta Gravity em produção. [Registo B](../.build/pet-goal-execution-next-r1/CURRENT-BACKUP-FULL-B-FINDING.json).

**Falha de produto150:** o driver terminou `recoveryRequired/Busy`, com aprovação retida, antes do backup. [Prova negativa](../.build/pet-goal-execution-next-r1/APP-NATIVE-CURRENT-BACKUP-COMPOSITION-R43F5C.json). D repete somente autenticação/publicação de controlo anterior à escrita, até1s, mantendo prova, lease, orçamento, prazo e alvo. `Unknown` devolve Busy, alteração/revogação recusa; liberta o gate entre tentativas. Resultado incerto/ACK indisponível não é Busy e não causa outra operação. [Decisão](../.build/pet-goal-execution-next-r1/COMPLETION-BUSY-DECISION-R43F5D.json). O timeout145 continua sem diagnóstico causal exato;150 não prova a sua causa específica.

**Limites de cobertura:** backup integral apenas dos payloads admitidos. Os rascunhos IndexedDB e inline ainda não estão incluídos; [próxima unidade nativa](../.build/pet-goal-execution-next-r1/NATIVE-DRAFT-COVERAGE-NEXT-AUDIT-R43F6.json). Restauro, autoridade após cópia, adoção da raiz, inline nativo, aceitação humana/provider/hardware/instalada e os49 critérios integrais permanecem abertos. [Uso](PETS-DATA-BACKUP.md). Raiz834/índice/instalação .30 preservados; sem novos agentes/commits/push/merge ou alterações do anfitrião.
