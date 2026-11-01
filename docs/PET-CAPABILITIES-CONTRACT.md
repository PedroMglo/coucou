# Contrato de identidade e capacidades dos pets — v1

## 01/11 — Fala nativa e candidata mochi.32 (C6C3)

Sucesso do helper nativo significa conclusão do processo, não audição confirmada. O backend Linux admite espeak; spd-say fica indisponível até existir cancelamento da própria mensagem SSIP, sem cancelamento global nem fallback silencioso. A reprodução no browser é um percurso distinto; mantém-se nas plataformas sem binding nativo. Estado disponível não concede permissões nem prova áudio real.

[Composição, provas, entrega e limites](releases/0.5.1-mochi.32.md).

A versão.32 está agora instalada e ativa: nome D-Bus, executável empacotado e admissão nativa verificados; três originais de preferências/conversas e cópias nativas byte-exatas. A.30 foi preservada. O ensaio284 do próprio RPM passou com helper sintético; isto não fecha áudio real nem os49 requisitos.

## 31/10 — Seleção TTS e cancelamento por resposta (C6C2, fontes1000)

Fontes1000 integradas: seleção TTS capturada na ocorrência de Voz e no preview, validade após esperas e cancelamento do renderer por resposta exata. Falha do motor escolhido conserva o texto; só a seleção explícita system pode usar síntese do sistema. [Provas e limites C6C2](../.build/pet-goal-execution-next-r1/VOICE-PLAYBACK-CHECKPOINT-C6C2A3.json).

Plano de saída não é grant. Não anuncia áudio/modelo/device ready por ter produzido um payload; voice_id de ElevenLabs é descoberta dinâmica dentro do serviço/género escolhido. Cancelamento de renderer e cancelamento de processo nativo são provas distintas. Modelos, runtime paths e endpoints não foram acionados realmente em QA.

## 31/10 — Entradas imutáveis da Voz (C6C1, fontes999)

Fontes999 integradas: entradas de captura/ASR e seleção de inferência imutáveis por ocorrência de Voz, usando os runners e a fábrica existentes. Alterar pet, perfil, endpoint ou preferências durante a espera não redireciona o pedido; o contexto inválido é recusado. O snapshot não renova grants. [Provas e limites C6C1](../.build/pet-goal-execution-next-r1/VOICE-TURN-CHECKPOINT-C6C1A4.json).

ASR desconhecido é unsupported/unavailable; falha do motor selecionado não escolhe outro. Cancelamento entre chunks não publica texto parcial. O listener termina quando as entradas deixam de ser válidas; a limpeza de um worker antigo não apaga o seu sucessor. Isto não declara disponibilidade de áudio real nem amplia caller/grants.

## 31/10 — Fontes997: seleção de áudio exata e posse dos workers de captura (C6A/B2)

Integrado na árvore principal, sobre os executores de Voz, ditado e gravação existentes. Um microfone nomeado ausente/ambíguo não abre outro por omissão; monitors/aliases não substituem a seleção explícita. A captura do sistema procura o alvo de loopback por host/ID exatos. O inventário tem limite256. Preferências guardadas, modelos, endpoints e wire schema mantêm-se.

Capture conserva Stop, JoinHandle e holds do Coordinator comum até recolher o worker. Timeout impede play tardio e não liberta um pedido ainda vivo. Os caminhos hotword/manual/sessão recebem o Stop externo; fechar a captura normalmente não cancela o turno de inferência que se segue. Donos tipados de Voz, Ditado, microfone da Gravação e áudio do sistema da Gravação impedem que trocar de alvo fabrique outro dono. As leases de leitura são independentes: Voz e gravação podem partilhar o mesmo endpoint quando o backend permite, sem uma revogar a outra. Não são grants de pet nem exclusividade sobre outras aplicações.

A primeira versão B1 tinha exclusão global demasiado ampla; a auditoria dos callers identificou a quebra que causaria no ditado/gravação enquanto hotword conserva o stream. B2 corrige-a e acrescenta provas de leituras partilhadas e de recusa de retarget do mesmo dono. B1 e os respetivos resultados ficam conservados; não chegou à instalação.

Provas B2:19 dirigidos passaram/2 ignorados;3153 gerais passaram/38 ignorados; build595 inputs/100 avisos existentes. Inventários e streams são sintéticos, exercitando os mesmos seletores/donos de produção. Não foram abertos dispositivos ou modelos reais. Os421 inputs não-Rust e71 outputs web da raiz continuam exatos;1329 testes web/TSC/Vite anteriores são reutilizados por identidade, sem nova execução.

[Checkpoint C6A/B2](../.build/pet-goal-execution-next-r1/VOICE-AUDIO-PARTIAL-CHECKPOINT-R43F10B2C3D2C6B2.json) · [Integração997 B2](../.build/pet-goal-execution-next-r1/integration997-r43f10b2c3d2c6b2/CHECKPOINT.json) · [Contrato de ativação e dependências](../.build/pet-goal-execution-next-r1/VOICE-ACTIVATION-CONTRACT-R43F10B2C3D2C.md). Instalação, HEAD/índice Git e10 ficheiros externos gravity_managed preservados; sem commits/publicação.

Pendente: Revisão de ativação Voz utilizável, com alvo nativo capturado e validado antes de esperar, estado/recibo de captura reais, autoridade distinta para captura/ASR/inferência/TTS/operação e bindings da fábrica; sem alterar modelos escolhidos. Defaults/aliases CPAL não são pinning físico. Depois Móvel/Hooks/Credenciais e restantes critérios integrais. Esta correção não demonstra pinning físico, áudio real/humano ou ativação governada completa. Todos os49 critérios integrais permanecem abertos.

## 31/10 — Novo pedido de atalhos no contexto atual, com histórico preservado (C3D2B3C4A1)

O endpoint shortcuts_review_prepare_current_scope é exclusivo das Definições nativas visíveis. Preview3/fingerprint capturam contexto e cadeia antes da aprovação. A decisão exige previewSchemaVersion3 e acknowledgeRetainedUncertainty além das três confirmações existentes. O renderer não fornece histórico, destinatário, caller ou políticas. nativeUser não concede taskRuntime/providerModel.

A composição privada SDK995/593 inputs permite um novo pedido humano de atalhos no contexto atual quando o original não pode ser observado. A revisão3 retém a cadeia e as identidades originais, mostra o destino atual e exige quatro confirmações, versão exata e uso único. Reutiliza Controller, Frame, gates, executores e diário; não concede permissões a pets. Sucesso atual conserva recibos e incerteza de scope anteriores. O portal pode partilhar configurações por aplicação: não se afirma independência por novo token, cliente, PID ou nome.

Formato8 impede leitores7 de ignorarem a incerteza histórica de Slots Verified/Reconciled. Controlos cumulativos, migração monotónica, reinício sem registo automático, backup e restauro nativo8 foram verificados.

Provas: 68 dirigidos passaram/2 ignorados; 3138 gerais passaram/38 ignorados; 1329 web/0 omitidos, TypeScript, Vite e build. Nativos274/275 atravessam Tauri/WebKit, quatro confirmações, registo/callback, segunda revisão, leitor7, reinício e restauro8. Portal sintético e X11 virtual; não aceitação humana/hardware/instalação.270/271 e272/273 conservam falhas de fixture (bindings deliberadamente desligados no restauro; profundidade2 indicada por Admission3), corrigidas sem alterar fontes/binário nem remover asserções.

[Checkpoint privado C4](../.build/pet-goal-execution-next-r1/SHORTCUTS-CURRENT-SCOPE-PARTIAL-CHECKPOINT-R43F10B2C3D2B3C4A1.json)

## R43F10B2C3D2B3C3A2 — comparação nativa original e atual

Preview2 expõe original/current a partir de fontes nativas validadas; o renderer só devolve chave, versão e confirmações. O backend exige previewSchemaVersion2 antes de observar/gravar. Dados originais ausentes ficam explícitos e não permitem Ready. Mesmos validadores estritos preservam u64 como decimais exatos e recusam processo/ambiente/portal malformados. Exibir sessões conhecidas não concede registo ou prova de fecho.

Privado: 992 fontes/591 entradas nativas. Passaram 55 testes dirigidos, 3134 gerais (38 ignorados), 1326 web/0 ignorados, TypeScript, Vite, build e nativos266/268/269. [Fontes, provas e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-ORIGINAL-OWNER-PARTIAL-CHECKPOINT-R43F10B2C3D2B3C3A2.json).

## R43F10B2C3D2B3C2A1 — restauro 7 e suspensão de todas as componentes

Ausência de um recibo não revalida a respetiva componente após copiar o perfil. O restauro suspende explicitamente as seis operações; formatos legados sem diário equivalente exigem revisão global. Preservar as concessões/orçamentos/recibos como dados não os torna autorização no destino. Confirmações humanas não fazem um processo de outro namespace parecer ausente.

Composição privada: 992 fontes/591 entradas nativas. Passaram 43 testes do módulo no A1. A execução geral C2 passou 3130 testes e teve duas expectativas antigas corrigidas no reteste dirigido; 38 ignorados. A1 altera apenas cfg(test), sem nova execução geral/build/nativa. Build e ensaio nativo 265 de C2, web1324/TSC/Vite B3 são reutilizados por identidade explícita dos respetivos inputs. [Fontes, provas e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-RESTORE-PARTIAL-CHECKPOINT-R43F10B2C3D2B3C2A1.json).

## R43F10B2C3D2B3B8 — recuperação dos atalhos e leitores antigos

Prepare/inspect/apply de reconciliação são exclusivamente nativeUser, com três confirmações e token/hash/Frame próprios; renderer não escolhe dono, PID, bus ou alvo. Resultado de observação não é capacidade de registar. Rejeitar caller/argumentos desconhecidos, outra janela, ACKs incompletos, token substituído, origem/nativo mudados e replay. Mesma instância inclui processo/ambiente/transporte; Stop só conserva ACK completo quando a cobertura do efeito publicado está comprovada. Planos legados sem identidade suficiente ficam recusados com motivo.

Composição privada: 992 ficheiros de fonte e 591 entradas nativas; backend e binário B7, uma correção de fixture cfg(test) B8. 52 testes dirigidos B8 passaram; a execução geral B7 teve 3129 testes aprovados, uma falha de fixture e 38 testes ignorados; a falha foi conservada e corrigida com reteste dirigido.Os 1324 testes web, TypeScript e Vite de B3 e os 18 testes D-Bus de B5 reutilizados por identidade explícita; build B7 e ensaios nativos 261 (X11) e 262 (portal) passaram. Não houve nova execução dessas suites no B8. [Fontes, provas e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-RECONCILIATION-PARTIAL-CHECKPOINT-R43F10B2C3D2B3B8.json).

## R43F10B2C3D2B3A1 — resultado pendente e publicação

Registo observado, publicação e limpeza são factos distintos. A resposta transporta cleanup tipado; nativeUser não recebe sucesso global por haver observação nativa verificada. Unconfirmed pendente bloqueia outra revisão, incluindo após reinício; só o controlador vivo finaliza o mesmo resultado exato. Sem grants novos para pets/modelos. Reconciliação própria ainda pendente. [Provas e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-PUBLICATION-PARTIAL-CHECKPOINT-R43F10B2C3D2B3A1.json).


## R43F10B2C3D2B2B3 — aprovação dos atalhos e suspensão X11

Atalhos têm prepare/inspect/cancel/apply/status de âmbito nativeUser; token, hash, Frame, fonte, destino e três confirmações vinculam a decisão única. Consulta não regista teclas; providerModel/taskRuntime não recebem esta autoridade. Verified é facto de sessão e conserva a barreira persistida. B3 ainda tem de fechar publicação/limpeza incerta e lifecycle. [Checkpoint e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-APPROVAL-PARTIAL-CHECKPOINT-R43F10B2C3D2B2B3.json).


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


## Contrato em implementação — R43F6A, rascunhos (26/10/2026)

`pet_draft_read/write/import` são IPC do utilizador nas superfícies autorizadas; não são ferramentas de `providerModel` ou `taskRuntime`. O backend valida a admissão dos dados antes de ler e imediatamente antes de publicar. Rascunho não concede acesso ao conteúdo ao modelo, não partilha memória e não envia mensagem. O armazenamento nativo conserva identidades UTF16 exatas, revisões u64 e tombstones, limita tamanho e faz CAS por destino. `pet_draft_mode` declara explicitamente o backend suportado; erro nativo não seleciona outro armazenamento. A API IndexedDB antiga fica retida/fenced, não apagada. [Contrato privado](../.build/pet-goal-execution-next-r1/NATIVE-DRAFTS-CONTRACT-R43F6A.md). Estado: em implementação, provas pendentes, inline/reconciliação/restauro ainda abertos.


**R43F5D validado em privado — 26/10:** SDK858 acrescenta revisão/confirmação/inspeção de backup dos dois payloads nativos admitidos e corrige a conclusão sob locks transitórios, conservando prova/aprovação e prioridade de Stop. Passaram2655 Rust/21 ignorados,23 dirigidos/1 ignorado,check-tests/build e composição nativa151;149 cobre as recusas e integridade do arquivo. São perfis, motores e escolhas sintéticos. **Rascunhos IndexedDB/inline ainda ficam fora desta cobertura**: segue persistência nativa/migração dos rascunhos, depois restauro/adoção. Raiz834 e instalação .30 preservadas; os49 critérios integrais continuam abertos. [Provas e limites](PETS-VALIDATION.md#backup-payload-conclusao-r43f5) · [Uso e cobertura](PETS-DATA-BACKUP.md).

**R43F3/F4 validado em privado — 25/10:** a composição854/F4 conserva a memória selecionada ao continuar objetivos com novo consentimento e orçamento adicional explícitos. Tauri146/147 provam duas rondas, aprovações separadas, revogação durante IO e preservação por reinício/leitor antigo, com motores sintéticos. A importação legada passa a recusar proveniência nova escondida em formato antigo e catálogos de runtime que seriam omitidos. Passaram2641 Rust/21 ignorados,check/build e a composição nativa147. O timeout145 permanece sem causa exata confirmada; não é uma regressão dada como corrigida. [Provas e limites](PETS-VALIDATION.md#retoma-memoria-importacao-r43f4). Seguem backup/restauro coerente do perfil atual e adoção; raiz834 e instalação .30 preservadas. Sem novos commits ou aceitação integral dos49 requisitos.

**R43F2E2 validado em privado — 25/10:** SDK854 liga a seleção/revisão de memória aos chats manuais, vincula anexos/contexto ao token e apresenta destinos API conhecidos sem prometer contenção do fornecedor. Passaram2638 Rust/21 ignorados,1212 JS/TSC/Vite/check/build e Tauri143/144. A janela de conversa passou nos dois motores locais, três CLI e Notion/Codex; o inline tem provas de controller/unidade, com aceitação nativa pendente após falha de ativação em Xvfb. São escolhas automatizadas e providers sintéticos. [Alcance e limites](PETS-VALIDATION.md#memoria-chat-contexto-r43f2e2). Raiz834/instalação .30 preservadas; seguem retoma com memória e migração/adoção.

O consentimento manual distingue `thisRequest` (o turno/pedido lógico, que pode conter rondas do protocolo suportado) de `nativeSessionContext` (dados que a sessão nativa pode conservar). Nenhum equivale a `boundedGoalContext` ou a uma concessão de ferramentas/Sistema. A preparação nativa vincula pergunta, dono/CID, destino/modelo/credencial, referências/revisões e hash do contexto tipado/`documentSequence`; trocar esse contexto não consome a revisão original. O conjunto de endpoints API é projetado pela mesma escolha de rota usada no despacho, incluindo Responses OpenAI para calendário. A base não é apresentada como prova de um único POST, e o routing/conta/ferramentas internos não observados continuam explícitos. A descoberta de um terminal externo não autoriza a ligação de memória da conversa Mochi.


**Histórico R43F2E1 — 25/10:** SDK850 acrescenta catálogo/seleção por revisão e revisão completa com consentimento próprio nos controlos reais de objetivos Ollama/LMStudio, além da projeção de memória por caller. Reutiliza CRUD/CAS, admissão, concessão e writer existentes. Passaram2634 Rust/21 ignorados (487 inputs nativos iguais entre E1a/E1b),1200 JS/TSC/Vite/check/build e Tauri139. Os cliques/consentimentos foram automatizados e os motores sintéticos: não é aceitação humana ou instalada. Seguem revisão manual de chat, retoma com memória e adoção/migração/entrega; os49 requisitos continuam abertos nos critérios integrais. [Provas e limites](PETS-VALIDATION.md#memoria-controlos-objetivos-r43f2e1). Raiz834 e instalação .30 preservadas.

Projeção R43 no SDK850: `memory.manage` permite apenas gestão por `nativeUser` (incluindo pet inativo configurado); não é ferramenta de modelo. `memory.contextReview` admite preparação humana nas rotas nativas realmente configuradas e permanece desconhecida até resolver o alvo. `memory.goalContext` admite `nativeUser`/`taskRuntime` somente nos dois motores locais associados, com revisão, concessão e writer originais. `memory.retrieve` continua não suportado para `providerModel`. A consulta passiva não resolve credenciais, autentica contas, carrega modelos ou faz inferência. Suporte não implica disponibilidade nem autoridade; a melhoria comum não concede acesso novo.


**Histórico R43F2D — 25/10:** SDK846 prova os envios de memória de Anthropic/OpenAI/Google através da implementação nativa comum, com HTTP loopback sintético, dados separados da pergunta e recusa de destino/modelo/credencial/pergunta alterados. Foi retirado o opt-in de fallback automático Anthropic; APIs e percursos locais usam o destino exato sem seguir redirecionamentos. Passaram2632 Rust/21 ignorados, check/build e Tauri136:13 pedidos HTTP e17 prompts CLI sintéticos, incluindo memória/decisões/recolha/retoma dos três CLI e serviço separado, objetivos locais e dois redirecionamentos recusados sem pedidos ao destino alternativo. Não certifica contas reais, routing interno do fornecedor, UI humana ou instalação. Raiz834 e instalação .30 preservadas. Seguem interface humana/callers, retoma de objetivos com memória e adoção/migração/entrega; R43 e os49 requisitos continuam abertos.

**Revisão posterior de 25/10 — Guardian e execução integral:** os 43 requisitos originais conservam os seus critérios; acrescentam-se GUA-01..06 no [contrato System Guardian](SYSTEM-GUARDIAN-ARCHITECTURE.md) e na [auditoria rastreável](PETS-REQUIREMENTS-AUDIT-2026-10-25.md). Pet, runtime comum, motor de inferência e papel funcional são entidades distintas. `ai_ollama` continua coordenador preferencial configurável, sem ser router obrigatório ou concedente de permissões. Guardian é especialista da mesma fábrica, sem autoridade própria adicional. Não há um pet Guardian operacional por esta atualização documental.

**Estado verificado nesta continuação:** raiz integrada834 e instalação .30 preservadas. SDK843/R43F2A passou 21 testes dirigidos de contexto, 2617 Rust/21 ignorados e cargo check em 483 inputs nativos imutáveis. O novo contrato de revisão mostra dados completos e destino nativo, separa consentimento de objetivo e verifica âmbito/credencial/revogação. É fundação privada: ainda falta a ligação à admissão, concessão assinada e corpo real anterior ao hash/reserva. Não foi acrescentada prova Tauri ou build instalada por este incremento. A barreira F1 permanece fechada; não declarar R43 concluído.

**Restrições vigentes:** um único agente, sem novos commits/pushes/merges de qualquer pasta; a publicação anterior do dossier foi uma exceção histórica já executada. Sem alteração da instalação/autostart/serviços, Wi-Fi/VPN/firewall/dispositivos ou dados pessoais. Sem quota real Copilot/GitHub, segredos em modelos/logs/Git ou bases de outros projetos. Alterações persistentes do anfitrião seguem sysworkspace e autorização específica; o desenvolvimento atual fica no projeto. Estados/estimativas antigos são históricos e não medem conclusão.

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


Correção em curso R43F6C: os rascunhos Mochi novos retêm o CID existente pelo mecanismo de `Chat`, depois de validar fábrica/destino/CAS e antes de publicar. O arquivo posterior consulta referências de rascunhos sem obter locks de escrita; desconhecido conserva. Não cria conversa, mensagem ou consentimento. Rascunhos legados órfãos permanecem dados e exigem recuperação explícita ainda pendente. O limite físico é64MiB para representar os limites lógicos anteriores, sem aumentar estes últimos. Provas finais em execução.

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

## Extensão de papéis e Guardian — contrato posterior de 25/10

Distinguir identidade de pet, runtime comum, binding de inferência e papel funcional. A evolução do descritor/configuração deve representar o papel de coordenador/especialista/serviço/observador sem o inferir do provider, sem conferir privilégios e sem alterar os IDs existentes. Estes campos novos são desenho; não se anuncia um schema implementado por este parágrafo.

Reservar `agent_system_guardian` e manter configuração de motor/modelo/servidor explícita e própria. O primeiro motor alvo é Ollama; falta evoluir a associação de dono lógico sem emprestar o PetId/provider/contexto de `ai_ollama`. Operações determinísticas, inferência, gestão humana, runtime e chamadas do modelo têm capacidades/callers distintos. Unknown não é ready; uma resolução ativa explicitamente pedida e autorizada pode obter observações sem confundir a consulta passiva com autenticação, carregamento ou consumo.

GUA-01..06 seguem o [contrato Guardian](SYSTEM-GUARDIAN-ARCHITECTURE.md), a matriz Sistema por pet/subplugin/operação/recurso e os brokers nativos. Nenhuma operação é pronta por existir no catálogo. A fábrica mantém base/molde/extensão/override e `pets.default.json` permanece projeção. Isolamento lógico não é sandbox para plugins não fiáveis. O [mapa auditado](PETS-REQUIREMENTS-AUDIT-2026-10-25.md) identifica as lacunas de implementação e prova.

### Preservação na continuação e importação — R43F3/F4

A consulta ou confirmação de encerramento anterior pode devolver o estado atual, mas não concede nova execução. Continuar um objetivo com memória requer a seleção original ainda válida, nova decisão `boundedGoalContext`, destino/modelo exatos e concessão própria; a receita nativa mantém a sua decisão separada. A prova privada146/147 inclui revogação após o envio: a resposta tardia não se publica e os dados já entregues não são retraídos.

O catálogo não anuncia backup/restauro completo por existir uma recusa de formato: R43F4 apenas impede a importação legada de estado moderno que seria omitido ou lido sob uma versão errada. Posse física e autoridade não são transferidas por cópia de ficheiros.

### Backup do perfil atual e restauro — decisão R43F5

O backup do perfil atual reutiliza o inventário limitado e a cópia com verificação existentes, mas tem protocolo/namespace próprio, distinto da importação .30. Captura integralmente os dois payloads admitidos, incluindo memória, anexos, objetivos, concessões, contextos, resultados e recibos incertos. Executa em manutenção offline, antes de iniciar Chat/workers, sob o mesmo nome D-Bus e lock físico de startup; uma aplicação ativa causa recusa, sem tomada de posse. A revisão nativa mostra origem/destino/resumo e exige confirmação do fingerprint exato antes da cópia. Os ficheiros da aplicação podem conter informação e chaves privadas: o arquivo é local/privado e não se publica em Git ou em modelos.

Um arquivo verificado contém dados, não execução autorizada. Não transfere locks, identidade física do dono ou saldo renovado, nem é prova de autenticidade contra código do mesmo UID. O restauro completo continua obrigatório e exige transação própria pelo gate de publicação/admissão, com recuperação explícita da autoridade histórica e efeitos incertos; não fica concluído por este incremento de backup. Não alterar a instalação ou executar este modo sobre o perfil pessoal sem a autorização específica aplicável.

### Cobertura dos rascunhos antes do restauro — descoberta R43F6

A inspeção do produtor real confirmou `IndexedDraftStorage`/`coucou.sessionDrafts.v1` na janela e Map em memória no prompt inline. O IndexedDB observado no próprio QA143 fica em `data/fr.louisraille.coucou/databases/indexeddb`, fora do payload nativo. Portanto, backup F5 significa cópia íntegra dos dois payloads admitidos; não fecha a preservação integral de rascunhos. Antes de restauro/adoção, mover estes produtores para armazenamento nativo revisto, reutilizando dono/CAS e a API de drafts existente, com migração dos registos conhecidos, conflitos, revisões, reinício e ACK incerto. Preservar a base antiga até à confirmação; não copiar perfis WebKit/autenticados nem enviar o rascunho automaticamente. Esta dependência foi descoberta no código e não elimina DATA-01/REL-01 ou critérios originais.
