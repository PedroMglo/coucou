# Auditoria rastreável — 43 requisitos originais e System Guardian

## Atualização01/11 — fontes1003 e instalação32

Fonte de código assinada25b2a0b;3182 Rust/38 ignorados,1333 web/0 ignorados, TypeScript/Vite, RPM otimizado e nativo284 com sintetizador falso. Instalação32 e admissão nativa observadas; dados originais conhecidos e cópias conservados. Todos43+6critérios integrais permanecem abertos. [Provas e limitações](releases/0.5.1-mochi.32.md).

## 31/10 — Seleção TTS e cancelamento por resposta (C6C2, fontes1000)

Fontes1000 integradas: seleção TTS capturada na ocorrência de Voz e no preview, validade após esperas e cancelamento do renderer por resposta exata. Falha do motor escolhido conserva o texto; só a seleção explícita system pode usar síntese do sistema. [Provas e limites C6C2](../.build/pet-goal-execution-next-r1/VOICE-PLAYBACK-CHECKPOINT-C6C2A3.json).

Matriz JSON atualizada para fontes1000; critérios43+6, proveniência e fontes anteriores mantidos. As secções anteriores são fotografias históricas. Não foi fechado nenhum requisito integral.

## 31/10 — Entradas imutáveis da Voz (C6C1, fontes999)

Fontes999 integradas: entradas de captura/ASR e seleção de inferência imutáveis por ocorrência de Voz, usando os runners e a fábrica existentes. Alterar pet, perfil, endpoint ou preferências durante a espera não redireciona o pedido; o contexto inválido é recusado. O snapshot não renova grants. [Provas e limites C6C1](../.build/pet-goal-execution-next-r1/VOICE-TURN-CHECKPOINT-C6C1A4.json).

Matriz JSON atualizada com fontes/símbolos/provas desta unidade;43 critérios originais e6 adicionais preservados. Os números834/910/992 nas secções anteriores são fotografias históricas, não a composição local atual.

## 31/10 — Novo pedido de atalhos no contexto atual, com histórico preservado (C3D2B3C4A1)

A composição privada SDK995/593 inputs permite um novo pedido humano de atalhos no contexto atual quando o original não pode ser observado. A revisão3 retém a cadeia e as identidades originais, mostra o destino atual e exige quatro confirmações, versão exata e uso único. Reutiliza Controller, Frame, gates, executores e diário; não concede permissões a pets. Sucesso atual conserva recibos e incerteza de scope anteriores. O portal pode partilhar configurações por aplicação: não se afirma independência por novo token, cliente, PID ou nome.

Formato8 impede leitores7 de ignorarem a incerteza histórica de Slots Verified/Reconciled. Controlos cumulativos, migração monotónica, reinício sem registo automático, backup e restauro nativo8 foram verificados.

Provas: 68 dirigidos passaram/2 ignorados; 3138 gerais passaram/38 ignorados; 1329 web/0 omitidos, TypeScript, Vite e build. Nativos274/275 atravessam Tauri/WebKit, quatro confirmações, registo/callback, segunda revisão, leitor7, reinício e restauro8. Portal sintético e X11 virtual; não aceitação humana/hardware/instalação.270/271 e272/273 conservam falhas de fixture (bindings deliberadamente desligados no restauro; profundidade2 indicada por Admission3), corrigidas sem alterar fontes/binário nem remover asserções.

[Checkpoint privado C4](../.build/pet-goal-execution-next-r1/SHORTCUTS-CURRENT-SCOPE-PARTIAL-CHECKPOINT-R43F10B2C3D2B3C4A1.json)

Matriz49 mantém os43 trabalhos/critérios originais e6GUA. A prova fecha esta unidade privada, não o requisito integral ou a entrega. Próximo critério: Lifecycle e adoção exata da composição SDK995: verificar guards após fecho de Settings/chat/janelas, Stop e reinício, sem serviço/autostart persistente; integrar a partir de fontes comprovadas quando os bloqueadores de preservação permitirem. Prosseguir com os controladores Voz/Móvel/Hooks/Credenciais e as dependências integrais.

## R43F10B2C3D2B3C3A2 — comparação nativa original e atual

Privado: 992 fontes/591 entradas nativas. Passaram 55 testes dirigidos, 3134 gerais (38 ignorados), 1326 web/0 ignorados, TypeScript, Vite, build e nativos266/268/269. [Fontes, provas e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-ORIGINAL-OWNER-PARTIAL-CHECKPOINT-R43F10B2C3D2B3C3A2.json).

Matriz49 atualizada com NativeSubject/current_subject, Preview2, decisão versionada e validadores/controlo partilhados.43critérios originais e6GUA preservados. Recuperação num contexto nativo diferente e restante aceitação integral continuam abertos.

## R43F10B2C3D2B3C2A1 — restauro 7 e suspensão de todas as componentes

Composição privada: 992 fontes/591 entradas nativas. Passaram 43 testes do módulo no A1. A execução geral C2 passou 3130 testes e teve duas expectativas antigas corrigidas no reteste dirigido; 38 ignorados. A1 altera apenas cfg(test), sem nova execução geral/build/nativa. Build e ensaio nativo 265 de C2, web1324/TSC/Vite B3 são reutilizados por identidade explícita dos respetivos inputs. [Fontes, provas e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-RESTORE-PARTIAL-CHECKPOINT-R43F10B2C3D2B3C2A1.json).

Matriz de 49 requisitos atualizada com o símbolo partilhado e os dois novos testes. Critérios 43+6 preservados; scope nativo desconhecido, comparação humana e restante aceitação integral continuam explícitos.

## R43F10B2C3D2B3B8 — recuperação dos atalhos e leitores antigos

Composição privada: 992 ficheiros de fonte e 591 entradas nativas; backend e binário B7, uma correção de fixture cfg(test) B8. 52 testes dirigidos B8 passaram; a execução geral B7 teve 3129 testes aprovados, uma falha de fixture e 38 testes ignorados; a falha foi conservada e corrigida com reteste dirigido.Os 1324 testes web, TypeScript e Vite de B3 e os 18 testes D-Bus de B5 reutilizados por identidade explícita; build B7 e ensaios nativos 261 (X11) e 262 (portal) passaram. Não houve nova execução dessas suites no B8. [Fontes, provas e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-RECONCILIATION-PARTIAL-CHECKPOINT-R43F10B2C3D2B3B8.json).

Matriz de 49 requisitos inclui os novos ficheiros/símbolos, dependências e limites;43 critérios originais e6 GUA preservados. Sem aceitação integral; próximo critério é restauro 7 e história do dono/recurso antes da adoção.

## R43F10B2C3D2B3A1 — resultado pendente e publicação

Matriz49 atualizada para privado984/B3A1, com fontes/símbolos, provas e limites novos. Janela de publicação falhada protegida; observação/reconciliação independente e aceitação integral ainda abertas. [Provas e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-PUBLICATION-PARTIAL-CHECKPOINT-R43F10B2C3D2B3A1.json).


## R43F10B2C3D2B2B3 — aprovação dos atalhos e suspensão X11

Matriz49 atualizada para privado983/B2B3; novos ficheiros/símbolos e provas estão no JSON associado. Nenhum critério integral fechado. Falta B3 para incerteza de publicação/limpeza e lifecycle antes de adoção; restantes dependências mantêm-se. [Checkpoint e limites](../.build/pet-goal-execution-next-r1/SHORTCUTS-APPROVAL-PARTIAL-CHECKPOINT-R43F10B2C3D2B2B3.json).


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


Revisão local atualizada em **27/10/2026**, preservando o pedido integral Guardian de25/10. [Dados estruturados](PETS-REQUIREMENTS-AUDIT-2026-10-25.json). Não é publicação, instalação ou declaração de conclusão.

Estado verificado: branch `feat/mochi-cli-accounts`, HEAD `bbe12e21010d28d36619003187d82ba175b6a0ec`, raiz integrada834, SDK934/F10B2B1 e instalação .30. Não foi feito checkout da branch documental, reset, clean, stash ou substituição de árvore. As alterações locais e o índice foram preservados. Só o agente principal está ativo e não há runner próprio pendente após confirmar os resultados terminais (originais deste incremento terminais e consumidos).

Histórico SDK854/R43F2E2: passou2638 Rust/21 ignorados (os mesmos488 inputs nativos em E2a/E2b),1212 JS/TSC/Vite/check/build e Tauri143/144. Revisão manual de dois âmbitos e contexto tipado vinculado reutilizam as autoridades existentes. Janela própria local2/CLI3/Notion tem30 provas de acesso aos controlos e estados finais; contexto trocado/replay/reinício não renovam consentimento. A ativação inline falhou em Xvfb140 e permanece pendente;142 revelou clipping corrigido emE2b, sem tomar streaming como conclusão. As provas anteriores API/CLI/objetivos ficam delimitadas nos respetivos checkpoints. A raiz834 conserva a barreira anterior. Retoma selecionada foi acrescentada em146/147 e proteção de importação antiga emF4:2641 Rust/21 ignorados,check/build. Seguem snapshot/restauro/migração e adoção; não antecipar TEAM ou Guardian. A falha145 de conclusão é preservada e não tem causa exata confirmada.

Revisão posterior local de26/10: SDK858/F5D passou2655Rust/21ignorados,check-tests/build e composição151. O backup149 conserva81ficheiros sintéticos e exige fingerprint exato/dono offline.151 verifica arranque normal/retoma e backup do mesmo perfil. O Busy de conclusão de150 motivouD;23testes dirigidos/1ignorado incluem locks reais, Unknown, Stop e limite sem repetir operações. Restauro completo/adoção ainda abertos. As falhas A/B/148/150 ficam delimitadas no checkpoint.

Mais recente: SDK865/F6D,2669Rust/21ignorados,1219JS/TSC/Vite e nativo154; janela persistida/migrada e CID vazio retido.0modelos/quota. Inline/reconciliação de origens/órfãos/restauro/adoção continuam pendentes. Falhas152/153 delimitadas no checkpointF6.

Mais recente: SDK869/F7A,2674Rust/21ignorados,1230JS/TSC/Vite e nativo158 de cartões autorizados,0modelos/quota. Inline e unbound nativos; leitor2 conserva1 e backend antigo recusa2. Digitação nativa inicial, pre-CID de outras plataformas, órfãos/divergência/restauro/adoção ainda pendentes. Falhas155/156/157 delimitadas no checkpointF7.

Mais recente: SDK874/F8A,2684Rust/21ignorados,1240JS/TSC/Vite e nativo160,0modelos/quota. Listagem/revisão/cópia de rascunhos órfãos conserva original e formato2. Divergência/restauro/adoção/aceitação integral continuam abertos.

Mais recente: SDK882/F8B2,2700Rust/21ignorados,1255JS/TSC/Vite e nativo162 real/scriptado,0modelos/quota. Origens divergentes conservadas em arquivo imutável antes do catálogo3/CAS/recibo; ACK exato transacional, lostACK sem reimportar, recuperação/leitor2/backup. Produção/binarío B1/B2 idênticos, B2 só corrige fixture futura4. Restauro/admissão/autoridade histórica/adoção/aceitação integral continuam abertos.

Mais recente: SDK885/F9A,2713Rust/21ignorados/check506/build e nativo163 real de CLI/IPC,0modelos/quota. Dados restaurados revistos byte exatos/cabeçalhos2/origem/recibos e crash próprio/retoma, mas inativos e recusados pelo bootstrap. Admissão/autoridade histórica/novos trabalhos/continuação/adoção/aceitação integral ainda abertos. Web1255 já validada e fontes não Rust idênticas.

Mais recente: SDK888/F9B1C,2725Rust/21ignorados/12novos/build509 e nativo165 real/HMAC/CLI/IPC,0inferência/quota. Histórico autenticado conservado/quarentenado, novos objetivos independentes/consumo incerto em Rust. Header2 continua inativo: admissão/configuração F9B2, continuação F9B3, adoção/entrega pendentes.

Mais recente: SDK890/F9B2A5,2737Rust/21ignorados/12novos/build511 e nativo168 real/HMAC/CLI/IPC sem inferência/quota. Revisão tipada e recibo configurationInspectionOnly preservam bytes e escolhas; não ativam definições. Header2 permanece inativo; B2B/B3/adoção/entrega pendentes.

Mais recente: SDK893/B2B1B,2754Rust/21ignorados/17novos/build514 e169 real sem inferência/quota. CID/SID/tarefa bloqueados em caminhos comuns com fixtures;169 prova só regressão Root1/B1/B2A. Admission2/índice real/ativação/guard nativo restaurado ainda pendentes.

Mais recente: SDK897/B2B2C1,2764Rust/21ignorados/10novos/build518 e nativo173 real com HTTP/CLI sintéticos/MPRIS privado. Admissão operacional/índice real/novo SID/objetivo/Stop/reinício/ACK provados na primeira via com Authority B1. Backup Root2/épocas/NoAuthority/revalidação voz/mobile/autostart/B3/adoção/entrega pendentes.170 fixture sem workspace e171 leitura de selo são conservados; C1 e173 corrigem os percursos delimitados;172 preserva o diagnóstico do harness.

Mais recente: SDK898/C1A1,2770Rust/21ignorados/6novos/build519 e174 nativo com providers sintéticos. Backup/inspeção Root2 completos dentro dos limites conjuntos; trabalho novo, originais/metadados, fonte offline e recusa por leitor897/restauro1 provados. Publicação/autoridade/ativação da época seguinte continuam pendentes; leitura não é autorização.

Mais recente: SDK898/C2A4,2776Rust/21ignorados/6novos/build519 e177 nativo com providers sintéticos. Dados2 preservados por publicação/retoma comuns em Header3 inativo; copy-worker próprio interrompido/recolhido/retomado, recibo/par/bytes e compatibilidade com recibo anterior898 sem alteração. A3 reteve4falhas SourceChanged no reforço de fsync para diretórios; A4 usa Node/Dir e passa. Reconciliação/ativação3 continuam pendentes; nada de autorização pela admissão arquivada.

Mais recente: SDK901/C3A6,2795Rust/21ignorados/19novos/build522/native179 com providers sintéticos. Mesma Authority preserva cadeia HMAC limitada, nega todas as referências antigas e conserva grants/consumo/incerteza. Originais completos precedem publicação. Prefixos temporários redundantes revistos são recuperados pelo CLI real; o resíduo179 é sintético, não morte nativa durante essa escrita. A4/178 também passou. Header3 ainda inativo, admissão3/revalidações/B3/adoção/entrega pendentes.

Mais recente: SDK901/D1A1,2804Rust/21ignorados/9novos/build522/native180 com providers sintéticos. Configuração Header3 e união de referências anteriores/atuais com recibo2/inspectionOnly e custody completa. Hash composto independentemente no nativo; só recibo acrescentado/ACK/leitor anterior. Sem admissão3/ativação, sem aceitação humana/instalada.

Histórico D2: SDK902/D2A4,2811Rust/21ignorados/7novos/build523/native182 com providers sintéticos. Admissão3/History2 e novos SID/objetivo/decisão/Stop/reinício/ACK provados com dados locais e arquivo externo ausente. Custody/originais/grants antigos conservados. Backup3 ainda recusa e impede adoção/entrega; nenhuma aceitação humana/instalada ou provider real.

Histórico D3: SDK904/D3A5,2822Rust/21ignorados/11novos/build525/native184. Backup3 completo e restauro/admissão seguinte Header3/depth3, providers sintéticos, sem renovar histórico/consumo. NoAuthority/revalidações/B3/adoção/entrega e aceitação humana/instalada continuam abertos.

Histórico D4A: SDK907/D4A5,2830Rust/21ignorados/8novos/build528/native185,0inferência/quota. Preparação independente nativa com intenção antes de RNG e negação histórica, sem publicação/admissão operacional. D4B/proveniência/recuperação incerta/revalidações/B3/adoção/entrega pendentes.

Histórico D4B: SDK909/D4B5,2837Rust/21ignorados/7novos/build530/native188. Mesma chave preparada admitida sob proveniênciaNoOriginalAuthority, novos objetivos/decisões/Stop/reinício/ACK e backup4 com providers sintéticos. Recuperação incerta/migrações/revalidações/B3/adoção/entrega e aceitação humana/instalada continuam abertas.

Histórico D4C: SDK910/D4C2,2841Rust/21ignorados/build531/native191: destino de resultados inválido recusado antes do envio/reserva; limites posteriores ao dispatch preservados. Providers sintéticos,0quota/modelos reais. Recuperação incerta/revalidações/migrações/B3/adoção/entrega abertas.

Histórico D4D1: SDK910/D4D1a,2844Rust/21ignorados/build531/native192: chave inválida exige reconciliação antes de escritas, parser comum e retoma da chave válida conservada,0modelos/quota/CLIagents. Retenção/quarentena explícita e migrações/revalidações/B3/adoção/entrega abertas.

Mais recente: SDK913/D4D2A2,2855Rust/21ignorados/build534/native193. Retenção nativa com intenção durável/sameinode/bytes/retomaRecibo e barreira de futura preparação,0modelos/quota/CLIagents/efeitos. Composição D4D2B/adoção/entrega e aceitação integral abertas.

Mais recente: SDK914/D4D2B1,2864Rust/21ignorados/build535/native195. Autoridade futura distinta após retenção sob outra revisão,Custody2/receipt4/admissão4/OpenedACK/backup2,época seguinte fixture.2HTTP1CLI sintéticos/0quota;adoção/entrega/49integrais abertos.

Mais recente: SDK915/B3A2,2870Rust21ignorados/1256web/TSC/Vite/build536/native196. Consulta autenticada de origem restaurada sem falsozero;B3F2/revalidações/adoção/entrega/49integrais abertos.

Mais recente: SDK923/B3F2A7,2886Rust21ignorados/1262web/TSC/Vite/build542/native199. Revisão nativa de dados13 e novo contexto/consentimento/limites/driver/verificação/Stop/ACK/backup no runtime comum; wire camelCase correto. Providers sintéticos e DBus privado,0quota; época seguinte fixture. Revalidações/adoção/entrega e49integrais abertos.

Adicional B4A1:3casos físicos no fluxo comum passam4096/64MiB e recusas+1/8MiB+1 preservando originais. Apenas testes mudaram; produção923/A7 idêntica, não há nova alegação full/native. Migrações/revalidações/adoção/entrega continuam pendentes.

Histórico: SDK925/F10A4,2892Rust21ignorados/1266web/TSC/Vite/build542/native201. Recusa antes de guardar/publicar,0600 exclusivo,erro de registo eUI readback;2HTTP1CLI sintéticos/0quota. Revalidação não concedida;intenção/CAS/confirmação após publicação/adoção/entrega e49integrais abertos.

Mais recente: SDK928/F10B1A1,2898Rust21ignorados/1268web/TSC/Vite/build544/native202. Intenção porcampo preserva escolha perante janela antiga; gate/no-op/erro retido semreplay;2HTTP1CLI sintéticos/0quota. F10B2/revalidação/adoção/entrega e49integrais abertos.

Mais recente: SDK931/F10B2A3,2905Rust21ignorados/build547/native203;7novasfixtures,9comandos reais comgravação bloqueada e semfalsoACK. Telefones/nomesporvoz/hookflags usam commit confirmado; errotelefonepara serviço.1268web/TSC/Vite de928 reutilizados cominputs/bundle iguais;2HTTP1CLI sintéticos/0quota. Escritor/revalidação/adoção/49integrais abertos.

Mais recente: SDK934/F10B2B1/Linux,12novos/124selecionados1ignorado/2917full21ignorados/check/build550/native204;revisão nativa guardada fora doJSON,edição externa recusada antesdeguardar/autostart,reinício privado/backup e0600. Web1268reutilizado cominputs/bundle iguais,2HTTP1CLI sintéticos/0quota. CAS sóotimista;barreira global/duráveldeincerteza,fsyncdiretório,revalidação/adoção/49integrais abertos.

A branch documental publicada avançou do commit consultado `eeee1b5` para `0abbcfc` apenas para corrigir a contagem do mapa de fontes na cópia documental. A autorização posterior suspende toda a nova publicação. Os números da fotografia não são instrução de rollback.

**Legenda:** documentado é contrato; parcial integrado é código na raiz, ainda sem todos os critérios; privado/fixtures significa unidade isolada. Nativo pode usar IPC real e provider sintético, sem certificar contas reais. Integrado localmente não é entregue. Pendente/indisponível tem motivo na lacuna. Todos os49 requisitos estão abertos nos critérios integrais; contagens de testes não medem conclusão. As bases indicadas para AUTO/TEAM/GUA não são funcionalidades já implementadas.

## Matriz de estado e próxima aceitação

| ID | Estado observado | Implementação existente | Lacuna / limite da prova | Próximo critério |
| --- | --- | --- | --- | --- |
| ARC-01 | documentado | Arquitetura, plano e histórico separados; revisão Guardian acrescentada sem remover IDs. | Cobertura continua a evoluir com implementação. | Rever ligações,49 IDs e divergências antes de um caminho incompatível. |
| ARC-02 | documentado | Contratos/ADRs precedem mudanças de contexto, autoridade e versões. | Binding de novo dono lógico e lifecycle ainda exigem decisão técnica completa. | Associar cada decisão ao delta de código, migração e prova efetiva. |
| ARC-03 | documentado | Cenários e limites por composição/provider preservados. | Nenhuma prova global fecha os43 ou o Guardian. | Aceitar as matrizes e cenários integrais com exceções resolvidas. |
| PET-01 | parcial integrado | Fábrica e descritores versionados usados por chat/voz/gestão e runtime. Resolução TS de dono pelo canal chat vem da fábrica comum, incluindo APIs sem taskProvider; rascunho unbound não promove capacidade de envio. | Projeção por caller e controlos de memória validados em privado854; separação papel/motor e adoção integral ainda pendentes. | Adotar projeções e testar scopes/callers em todos os percursos sem promover capacidades indisponíveis. |
| PET-02 | parcial integrado + privado | Associação de dono/turno/sessão, decisões e continuidade comuns; R43 acrescenta proveniência de memória no SDK. B2B1B: referências CID/SID/tarefa antes do codec, guardas comuns de reserva/início/retoma/currentness/publicação, histórico legível e novos contextos em fixtures.2754Rust/21ignorados/17novos/build514;169 é regressão Root1/B1/B2A, sem inferência/quota. B2B2C1: plano nativo/recibo B2A/originais integrais antes de transformar a configuração canónica; Admission2 selada/hash histórico e índice real carregado por Storage antes de Chat.2764Rust/21ignorados/10novos/build518;173 IPC real com HTTP/CLI sintéticos e MPRIS privado: SID novo, objetivo/decisão/Stop/reinício/ACK, IDs antigos recusados.0quota/modelos reais.170 fixture sem workspace e171 erro real de leitura de selo corrigidos;172 chegou ao ACK mas faltou esperar o driver no harness;173 passa, com originais conservados. B2C1A1: backup/inspeção Root2 em formato2 com ambos roots completos, trabalho novo e metadados/originais. Validação comum inicial/HMAC/índices/links sem conceder execução;2770Rust/21ignorados/6novos/build519 e174 nativo com HTTP/CLI sintéticos/MPRIS privado: backup após novo objetivo/Stop/reinício, origem offline, tamper e leitor897/restauro1 recusam2 sem publicar payload. B2C2A4: arquivo2 dos roots completos publicado/revisto/retomado num Header3 inativo, dados antigos byte exatos e admissão2 aninhada como dados. Source1/fence/recibo conservados; restauração de dados e ativação executável projetadas separadamente. Entradas completas reutilizadas também fsync antes da completion.2776Rust/21ignorados/6novos/build519 e177 nativo com copy-worker próprio morto/recolhido e mesmo par retomado, ACK, recibo anterior898 inalterado e recusa de controlo3. B2C3A6: mesma Authority/HMAC/ledger com cadeia limitada autenticada/união de negações, raw grants/consumo desconhecido preservados, revisão nativa Header3/originais completos antes do delta e recuperação de prefixos temporários revistos sem renovar consentimento.2795Rust/21ignorados/19novos/build522/native179 com providers sintéticos e resíduo de staging sintético explicitamente rotulado; A4/178 conservados. B2D1A1: revisão nativa Header3 exige custody C3 completa; recibo2/inspectionOnly liga modelos/endpoints/manifest/ready/histórico e união atual+anterior de CID/SID/tarefa/proveniência limitada. Índice1/recibo1 conservados; Admission2 lê só1.2804Rust/21ignorados/9novos/build522/native180 com hash composto independentemente a partir de bytes reais, providers sintéticos e nenhum efeito adicional. B2D2A4: mesma Activation/Storage com manifest/admission3/job próprio/Scope D1 e mesmos originais C3, History2 antes de Chat e paths2, contexto antigo/atual negado;2811Rust/21ignorados/7novos/build523/native182 com novosSID/objetivo/decisão/Stop/reinício/ACK e arquivo externo ausente, providers sintéticos. Grants/consumo antigos conservados. D3A5: backup3/arquivo2 completo e novo Header3/depth3/C3/D1/D2 com quartoSID e objetivo/decisão próprios, histórico/grants/consumo intactos e reinício/ACK/source offline.2822Rust/21ignorados/11novos/build525/native184 privados. Cópia C3 usa revisões imutáveis+Claim por entrada e hashes integrais antes dos efeitos;183 timeout e A3/A4 erros de expectativa das fixtures conservados. D4A5: preparação nativa para ausência real original, mesmo HMAC/negação, intenção antes de RNG e geração única; sem grants/payload/ativação.2830Rust/21ignorados/8novos/build528/native185,0modelos/quota; A3 prova negativa de regeneração corrigida e conservada. D4B5: mesma chave preparada admitida via manifest/admission4/Header2 e recibo3 separado; adição tipada/prefixos exatos, mesmos originais/ledger/negações, Blobs locais após opened, novosgoals não reclassificados.2837Rust/21ignorados/7novos/build530/native188 com CLI/HTTP sintéticos e0quota/modelos reais,novoSID/objetivo/decisão/Stop/reinício/ACK/backup4; nova épocaAdmission4 emfixtures. | NoAuthority normal operacional privado comprovado; geração incerta/seed inválido/pre-manifest/staging anónimo ainda conservados/recusados até reconciliação explícita. Migrações/revalidações voz/mobile/autostart/B3/adoção e aceitação integral continuam abertas. | Fechar recuperação explícita de geração incerta e staging sem regeneração silenciosa; depois revalidações/migrações/B3/adoção antes TEAM/Guardian. |
| PET-03 | parcial integrado + privado | Três adaptadores, continuidade de pasta/ID e política ACP; protocolos CLI sintéticos R43E1. | Quota real Copilot vedada; Gravity/canais e limites de autonomia CLI não são universais. | Provar por protocolo ferramentas/MCP, sandbox, decisões, Stop/recolha e controlo suportado; não ativar autonomia sem limites. |
| PET-04 | parcial integrado + privado | Ollama/LMStudio têm associação e identidade local; API e outros pets dependem da configuração declarada. | API HTTP/rotas têm prova sintética; janela própria manual cobre local2/CLI3/Notion. Inline nativo, contas/modelos reais e binding próprio Guardian não aceites. | Fechar retoma/adoção R43 e separar binding lógico/motor na fábrica com migração e testes. |
| PET-05 | parcial integrado | Serviços/observadores declarados; assistência conserva dono lógico e retirada GitHub. | Catálogo não comprova operações nativas de todos os serviços ou novos especialistas. | Aceitar operação/caller por serviço, com motor indisponível e sem permissões herdadas. |
| RUN-01 | parcial integrado + privado | Repositório de objetivos/planos/passos/eventos; contexto selecionado2/objetivo11 e concessão com memória condicional12 no SDK850. | Migração/aceitação integral e dados de regras/delegações não concluídos. | Conservar IDs/estado ativo/recibos e impedir execução por leitura/migração. |
| RUN-02 | parcial integrado + privado; incremento privado910 | Admissão, reservas físicas, posse e conversa isolada de background. D4C2: preflight comum só de leitura antes de metadata e reserva/inferência, sem reparar namespace nem consumir orçamento na recusa. Falha após dispatch conserva reconciliação sem replay.18dirigidos/2841Rust21ignorados/build531/native191 com providers sintéticos; recusa funcional/ledger intacto/reinício/backup. | Admissão/continuação, conclusão sob concorrência e backup provados em privado; lifecycle integral, restauro/adoção e aceitação instalada pendentes. | Recuperação explícita de geração incerta/seed inválido/pre-manifest/staging; migrações/revalidações/B3/adoção e aceitação integral permanecem. |
| RUN-03 | parcial integrado + privado; incremento privado910 | Planeamento/receitas limitados com critérios e orçamento comum em motores locais. Distingue Unknown/Changed na conclusão e espera até1s apenas por controlo antes da publicação, com prova original e sem efeitos repetidos. D4C2: preflight comum só de leitura antes de metadata e reserva/inferência, sem reparar namespace nem consumir orçamento na recusa. Falha após dispatch conserva reconciliação sem replay.18dirigidos/2841Rust21ignorados/build531/native191 com providers sintéticos; recusa funcional/ledger intacto/reinício/backup. | Native150 mostrou Busy concreto;D tem testes reais de locks/Stop/limite e composição151. Execução geral, CLI/autonomia e critérios de todos os adaptadores ainda abertos. | Recuperação explícita de geração incerta/seed inválido/pre-manifest/staging; migrações/revalidações/B3/adoção e aceitação integral permanecem. |
| RUN-04 | parcial integrado; incremento privado910 | Reconciliação, continuação explícita e recuperação de resultados sem reinferência. Retoma com memória selecionada e cobrança cumulativa provadas por146/147. Stop conserva prioridade durante a espera de conclusão; publicação incerta não é reclassificada como Busy. D4C2: preflight comum só de leitura antes de metadata e reserva/inferência, sem reparar namespace nem consumir orçamento na recusa. Falha após dispatch conserva reconciliação sem replay.18dirigidos/2841Rust21ignorados/build531/native191 com providers sintéticos; recusa funcional/ledger intacto/reinício/backup. D4D1a: parser estrito comum/keyInvalid/manualRequired e recusa de apply antes de escrever, sem regenerar.11dirigidos/2844Rust21ignorados/build531/native192,0modelos/quota/CLIagents; controlo negativo leitorC2,8chaves inválidas,retoma válida e ready/missing recusados. D4D2A2: CLI nativo retém preparação opaca/cadeia8 com intent antes de rename NOREPLACE, mesmo inode/bytes, fresh review/ready/ACK; sem nova chave/grants, pending normal-reader/P1 bloqueados.11dirigidos/2855Rust21ignorados/build534/native193,0modelos/quota/CLIagents/efeitos. Porta conteúdo emfixture;adoçãooperacionalpendente. D4D2B1: Intent2/generation2 sob OUTRA aprovação/Custody2 Binding1/receipt4 por head preserva1/3/4; Activation/Bound/runtime/backup2 comuns,opened ACK offline/época seguinte fixture.7dirigidos/2864Rust21ignorados/build535/native195 com2HTTP+1CLI sintéticos/0quota. Raiz834 e instalação30 preservadas. B3A2: prova negativa de falsozero com grant histórico desligado; histórico2/origemHMAC exata/consumo desconhecido sem autoridade,mesmoStore/Authority/UI.2870Rust21ignorados/1256web/TSC/Vite/build536/native196 sintético,0quota. Trabalho novo mantém resposta1;originais preservados. B3F2A7: novoGoal/Spec13 origem selada só para dados; contexto/admissão/limites próprios,mesmo runtime/driver/Stop/verificação. Histórico concluído é dado,controlo antigo recusado.32dirigidos/2886Rust21ignorados/1262web/build542/native199 UI e HTTP2CLI1/DBus sintéticos,0quota; leitor915 real recusa13 sem destruir. Época13 seguinte é fixture. | Resta lifecycle/entrega e cobertura de todos os efeitos, Guardian e mesh.145 permanece sem causa exata;150 confirmou Busy e foi corrigido emD. | Fronteiras conjuntas4096/64MiB, migrações/revalidações/adoção local de fontes exatas; lifecycle e aceitação integral/provider antes de TEAM/Guardian e entrega. |
| RUN-05 | parcial integrado | Recursos e esperas limitadas existentes; Stop e posse física têm guardas. | Justiça entre rotinas/delegações ainda sem aceitação integral. | Prioridade de Stop/decisões e backpressure sem monopolização ou saldo duplicado. |
| EXE-01 | parcial integrado | Broker de decisões e operações/receitas tipadas em percursos suportados. | Catálogo geral de ferramentas/Sistema e executores adicionais incompletos. | Uma ferramenta descoberta não fica autorizada; alvo/argumentos/versão e efeito exatos. |
| EXE-02 | parcial integrado + privado | Decisões únicas e concessões limitadas; consentimento de dados ligado à concessão/destino assinados e corpo real no SDK850. D4D1a: parser estrito comum/keyInvalid/manualRequired e recusa de apply antes de escrever, sem regenerar.11dirigidos/2844Rust21ignorados/build531/native192,0modelos/quota/CLIagents; controlo negativo leitorC2,8chaves inválidas,retoma válida e ready/missing recusados. D4D2A2: CLI nativo retém preparação opaca/cadeia8 com intent antes de rename NOREPLACE, mesmo inode/bytes, fresh review/ready/ACK; sem nova chave/grants, pending normal-reader/P1 bloqueados.11dirigidos/2855Rust21ignorados/build534/native193,0modelos/quota/CLIagents/efeitos. Porta conteúdo emfixture;adoçãooperacionalpendente. D4D2B1: Intent2/generation2 sob OUTRA aprovação/Custody2 Binding1/receipt4 por head preserva1/3/4; Activation/Bound/runtime/backup2 comuns,opened ACK offline/época seguinte fixture.7dirigidos/2864Rust21ignorados/build535/native195 com2HTTP+1CLI sintéticos/0quota. Raiz834 e instalação30 preservadas. B3A2: prova negativa de falsozero com grant histórico desligado; histórico2/origemHMAC exata/consumo desconhecido sem autoridade,mesmoStore/Authority/UI.2870Rust21ignorados/1256web/TSC/Vite/build536/native196 sintético,0quota. Trabalho novo mantém resposta1;originais preservados. B3F2A7: novoGoal/Spec13 origem selada só para dados; contexto/admissão/limites próprios,mesmo runtime/driver/Stop/verificação. Histórico concluído é dado,controlo antigo recusado.32dirigidos/2886Rust21ignorados/1262web/build542/native199 UI e HTTP2CLI1/DBus sintéticos,0quota; leitor915 real recusa13 sem destruir. Época13 seguinte é fixture. | Controlos Tauri e scopes manuais/objetivo distintos com escolhas automatizadas; inline nativo, adoção/aceitação humana e restantes caminhos ainda abertos. | Fronteiras conjuntas4096/64MiB, migrações/revalidações/adoção local de fontes exatas; lifecycle e aceitação integral/provider antes de TEAM/Guardian e entrega. |
| EXE-03 | parcial integrado; incremento privado910 | Recibos e observação independente das operações Spotify; incerteza e recuperação. D4C2: preflight comum só de leitura antes de metadata e reserva/inferência, sem reparar namespace nem consumir orçamento na recusa. Falha após dispatch conserva reconciliação sem replay.18dirigidos/2841Rust21ignorados/build531/native191 com providers sintéticos; recusa funcional/ledger intacto/reinício/backup. | Outros efeitos/serviços e estação operacional não universalmente aceites. | Recuperação explícita de geração incerta/seed inválido/pre-manifest/staging; migrações/revalidações/B3/adoção e aceitação integral permanecem. |
| EXE-04 | parcial integrado | Rotas nativas preservam ferramentas e instruções para CLI/API/acessibilidade/desktop. | A existência da ferramenta não certifica automação visual ou sessão externa. | Alvo exato, observação antes/depois, sessão bloqueada e falta de suporte explícitas. |
| CTX-01 | privado + base integrada | Protótipo de coleção/retenção/recuperação, com ensaios terminais/nomes reutilizados. | Fecho da limpeza, composição e migração/backup/rollback ainda bloqueiam entrega do conjunto. | Fechar janelas de falha e adotar fontes exatas sem perder cópias/referências antigas. |
| CTX-02 | privado + base integrada | Captura/manifestos e seleção por turno em protótipos e percursos de documentos. | UI/integração da coleção múltipla e cobertura por mensagem não integralmente entregues. | Adicionar cumulativamente e selecionar conjunto imutável sem confundir nome, cópia e leitura. |
| MEM-01 | parcial integrado + privado | Notas/preferências com dono/origem/validade/revisões em834; seleção/revisão humana de dois âmbitos e contexto nativo em854. | Projetos, episódios, derivados, resumos e partilha explícita por implementar. | Concluir R43 e evoluir classes com dependências/invalidação sem memória global. |
| MEM-02 | parcial integrado + privado | CRUD/CAS, pesquisa/captura/revogação e dados por destino; SDK854 acrescenta revisão manual e hash de contexto antes de consumir o token. Retoma selecionada nativa146/147 conserva referências e consumo e exige novo consentimento próprio; duas rondas e revogação em IO com motores sintéticos. Backup atual conserva ficheiros de memória e contexto como dados. | Inline/aceitação humana e contas reais, derivados/índices, restauro/adoção pendentes.150 teve Busy de conclusão corrigido emD;145 não tem diagnóstico causal exato. | Fechar restauro/migração da composição coerente e preservar revisão/destino/autoridade sem renovação. |
| CTX-03 | parcial integrado | Catálogo/consulta ICS e seleção por pet/CID existentes. | Gestão/consentimento e aceitação de todas as superfícies ainda incompletos. | Fonte selecionada e cobertura/frescura explícitas; não anunciar sincronização/escrita. |
| CTX-04 | pendente + base integrada | Fonte inscrita tem âmbito/revisão; base CAS reutilizável. | Coleções multicalendário e principais de objetivo/regra/ocorrência não concluídos. | Migrar sem alargar pet/CID antigo; capturar coleção e revogação por ocorrência. |
| CTX-05 | pendente | Avaliação de reutilização identifica limites TLS/bytes/redirects/IMAP; não é adaptador operacional. | RSS/IMAP limitados ainda por implementar/aceitar. | Limites antes de materializar, pastas/destinos explícitos e nenhuma escrita/envio implícitos. |
| AUTO-01 | pendente | Bases de objetivos/concessões/fontes existem; não constituem repositório completo de regras. | Definição/ativação/revisão de regras e ligação de grants faltam. | Regra autorizada tem dono, fontes, gatilho, validade e limites; texto externo não a cria. |
| AUTO-02 | pendente | Runtime/recursos são fundação a reutilizar, sem segundo scheduler. | Agendamento com ocorrências/DST/offline não aceite ponta a ponta. | Fuso IANA, debounce/deduplicação, skip/coalesce/decisão sem avalanche. |
| AUTO-03 | pendente | Consulta de calendário é base, não rotina diária completa. | Multicalendário, regra/ocorrência e notificação dependem CTX-04/AUTO. | Uma ocorrência com dados novos, falha parcial, origem e uma notificação permitida. |
| AUTO-04 | pendente + base integrada | Critérios/progresso/recuperação individuais existem. | Acompanhamento periódico autorizado e memória de projeto não concluídos. | Detetar bloqueio/progresso e pedir decisão sem replay, spam ou orçamento renovado. |
| AUTO-05 | pendente + base integrada | Estado/avisos têm bases existentes; não prova política integral de notificações de regras. | Silêncio, escalada e deduplicação por ocorrência/canal por completar. | Validade/limites/sensibilidade e ausência de reenvio infinito ou troca de conta/canal. |
| TEAM-01 | pendente | Preparação de texto editável e autoridade individual existem. | Não existe prova de envelope/entrega durável e delegação comum operacional. | Discovery/mensagens diretas com origem nativa, IDs/causa/ACK/prazo e dados autorizados. |
| TEAM-02 | pendente | Leases/orçamento individuais são base, não mesh paralela pronta. | Reserva de orçamento pai/filho, ramos/ciclos/conflitos e ausência de coordenador não aceites. | Ramos independentes, backpressure/cancelamento, saldo repartido e nenhum privilégio herdado. |
| SYS-01 | pendente + bases parciais | Controlos/executores e política nativa de janela existem; não são matriz por pet. | Matriz pet×subplugin×operação×recurso ainda por ligar a executores/pets. | Sistema principal sem grant global; Som Spotify/Ollama com revogação isolada. |
| SYS-02 | privado + pendente | Protótipos de transporte/estado/rede e contratos históricos; grupo Rede/Bluetooth definido. | Paridade dos handlers, alvos, prompts, dispositivos e efeito real não concluída. | Unidade completa com grants separados, ACK distinto de efeito; sem alterar rádios em QA. |
| SYS-03 | parcial + pendente | Som/multimédia e controlos de acesso existentes em percursos declarados. | Streams/perfis/microfone e Ecrã/Energia/Sessão/matriz ainda sem aceitação integral. | Mesmos executores com autoridades distintas, alvos exatos, concorrência e indicadores preservados. |
| INT-01 | parcial integrado | Serviços/assistência e permissões separadas; rascunho não é envio. F10A4: prova negativa200 antes da correção; admissão antes da gravação/publicação no escritor comum,0600 exclusivo,temporário estranho conservado,erro final de registo explícito e controlo com readback sem retry.81dirigidos/2892Rust21ignorados/1266web/build542/native201,2HTTP1CLI sintéticos/0quota;UI recusa/memória/bytes/eventos/gravação/restart/backup. Voice/mobile/autostart continuam bloqueados,sem conceder revalidação. F10B1A1: intenção apenas enabled/Settings nativa, snapshots gerais preservam campo, gate impede sobreposição sem segurar mutexglobal no executor. No-op conserva erro anterior sem replay; recibo distingue confirmação de dados e executor.6dirigidos/2898Rust21ignorados/1268web/build544/native202;duasjanelas reais,registo criado/removido sóHOMEprivado,2HTTP1CLI sintéticos/0quota. F10B2A3: campos do telefone, nomes porvoz e indicadores dehooks só entram no Settings/evento após persistência; erros propagados sem falso rollback. Erro telefone para ligação;Stop chat antes da escrita. Reutiliza transação comum, sem novo formato/autoridade.931/547,2905Rust21ignorados/native203; web1268/TSC/Vite reutilizados sem alteração.9comandos nativos sobfalha,0quota/modelos reais. F10B2B1/Linux934: leitores normal/estrito capturam revisão nativa não serializada;writer comum verifica pai/alvo/alias/inode/stamp antes de substituir. Snapshot de janela não renova a origem.12novos/124selecionados1ignorado/2917full21ignorados/check/build550/native204; edição externa recusada antesdeguardar/autostart,reinício privado ebackup.0quota/modelos reais;web1268reutilizado. | Operações reais de cada serviço/envio/contas não universalmente implementadas/aceites. F10A não implementa revalidação: faltam intenção por campo/CAS e confirmação após publicação/fsync diretório antes de a conceder. Consumidores não fecham escritor: CAS/fsyncdiretório/resultado incerto tipado/reconciliação/reinício pendentes. Listener ainda antes da gravação e chaves com efeitos próprios; não revalidados/entregues. Isto é verificação otimista para writers cooperantes, não CAS atómico/sandbox deoutros processos do mesmoUID. Incerteza pós-publicação tem classe/recusa de nova escrita,mas barreira global deefeitos/reconciliação durável/reinício/fsyncdiretório eparidade Mac/Windows ainda pendentes. | F10B2C: cercar todos novos efeitos com estado de IO do Shared atual, sem invalidar turnos capturados por simples mudança legítima de outro campo;barreira durável/reconciliação sem replay/fsyncdiretório e intenção do listener/chaves;depois revalidação/adoção/aceitação integral. |
| VOICE-01 | parcial integrado | Preparação/roteamento de voz e fronteiras comuns em percursos existentes. F10A4: prova negativa200 antes da correção; admissão antes da gravação/publicação no escritor comum,0600 exclusivo,temporário estranho conservado,erro final de registo explícito e controlo com readback sem retry.81dirigidos/2892Rust21ignorados/1266web/build542/native201,2HTTP1CLI sintéticos/0quota;UI recusa/memória/bytes/eventos/gravação/restart/backup. Voice/mobile/autostart continuam bloqueados,sem conceder revalidação. F10B1A1: intenção apenas enabled/Settings nativa, snapshots gerais preservam campo, gate impede sobreposição sem segurar mutexglobal no executor. No-op conserva erro anterior sem replay; recibo distingue confirmação de dados e executor.6dirigidos/2898Rust21ignorados/1268web/build544/native202;duasjanelas reais,registo criado/removido sóHOMEprivado,2HTTP1CLI sintéticos/0quota. F10B2A3: campos do telefone, nomes porvoz e indicadores dehooks só entram no Settings/evento após persistência; erros propagados sem falso rollback. Erro telefone para ligação;Stop chat antes da escrita. Reutiliza transação comum, sem novo formato/autoridade.931/547,2905Rust21ignorados/native203; web1268/TSC/Vite reutilizados sem alteração.9comandos nativos sobfalha,0quota/modelos reais. F10B2B1/Linux934: leitores normal/estrito capturam revisão nativa não serializada;writer comum verifica pai/alvo/alias/inode/stamp antes de substituir. Snapshot de janela não renova a origem.12novos/124selecionados1ignorado/2917full21ignorados/check/build550/native204; edição externa recusada antesdeguardar/autostart,reinício privado ebackup.0quota/modelos reais;web1268reutilizado. | Cobertura integral das operações e decisões multicanal não demonstrada. F10A não implementa revalidação: faltam intenção por campo/CAS e confirmação após publicação/fsync diretório antes de a conceder. Consumidores não fecham escritor: CAS/fsyncdiretório/resultado incerto tipado/reconciliação/reinício pendentes. Listener ainda antes da gravação e chaves com efeitos próprios; não revalidados/entregues. Isto é verificação otimista para writers cooperantes, não CAS atómico/sandbox deoutros processos do mesmoUID. Incerteza pós-publicação tem classe/recusa de nova escrita,mas barreira global deefeitos/reconciliação durável/reinício/fsyncdiretório eparidade Mac/Windows ainda pendentes. | F10B2C: cercar todos novos efeitos com estado de IO do Shared atual, sem invalidar turnos capturados por simples mudança legítima de outro campo;barreira durável/reconciliação sem replay/fsyncdiretório e intenção do listener/chaves;depois revalidação/adoção/aceitação integral. |
| VOICE-02 | pendente + provas parciais | Modelos/preferências e ensaios de transcrição sintética existentes. | Microfone/Bluetooth/TTS PT/EN/misto/latência/diálogo humanos pendentes. | Provar áudio real no ambiente autorizado; fixtures não certificam qualidade humana. |
| OBS-01 | parcial integrado | Censo separa clientes/TTY/sessões e estado/cobertura; runtime mantém fases. B3A2: prova negativa de falsozero com grant histórico desligado; histórico2/origemHMAC exata/consumo desconhecido sem autoridade,mesmoStore/Authority/UI.2870Rust21ignorados/1256web/TSC/Vite/build536/native196 sintético,0quota. Trabalho novo mantém resposta1;originais preservados. B3F2A7: novoGoal/Spec13 origem selada só para dados; contexto/admissão/limites próprios,mesmo runtime/driver/Stop/verificação. Histórico concluído é dado,controlo antigo recusado.32dirigidos/2886Rust21ignorados/1262web/build542/native199 UI e HTTP2CLI1/DBus sintéticos,0quota; leitor915 real recusa13 sem destruir. Época13 seguinte é fixture. | Todos os providers/estados externos e instalação não têm cobertura completa. | Fronteiras conjuntas4096/64MiB, migrações/revalidações/adoção local de fontes exatas; lifecycle e aceitação integral/provider antes de TEAM/Guardian e entrega. |
| OBS-02 | parcial integrado | Captura/métricas e ledgers em percursos declarados, com lacunas. | Cobertura/rotação por conta/thread/provider e reinícios não integralmente aceite. | Não somar snapshots como deltas, não perder antes do ACK nem autenticar por consulta. |
| DATA-01 | parcial integrado + privado; incremento privado910 | Repositórios privados, CAS, posse e recuperação incremental existentes. F4 recusa memória entregue oculta em formato antigo e qualquer catálogo pet-runtime-* na importação legada; v7 recusado sem escrita. SDK858/F5 acrescenta revisão/confirmação/inspeção CLI de backup completo dos payloads atuais, sob o mesmo dono offline, com proveniência da admissão. Nativo149 prova81ficheiros/adversariais;151 compõe arranque normal, retoma e backup do perfil recém-criado. F6D usa repositório nativo/CAS e migração IndexedDB conhecida para a janela, preserva UTF16/u64/tombstones e retém CID vazio antes de gravar, sem inferência; nativo154 prova migração, conflito, reinício/reset e inclusão no backup. F7A acrescenta produtor inline, slot unbound nativo sem autoridade e cópia explícita conservando origem, controlos comuns, catálogo2 lê1 e leitor antigo recusa2. Nativo158 nos cartões autorizados prova dados de três pets, ponte inline/flutuante, reinício e backup, sem modelos/quota. F8A entrega listagem nativa por dono/revisão, classificação observada, revisão com bytes/destino/prazo/hash e cópia explícita preservando original, sem mensagens/grants. Nativo160 prova controlos inline/flutuante, recusa de replay/conflito/troca de dono, reinício/leitor anterior e backup. F8B2: revisão de ambas as origens/política/revisões/prazo/hash, original completo imutável antes do catálogo3/CAS/recibo; apenas destinos ausentes podem ser juntos. Origens pendentes recuperáveis, reconhecimento transacional/lostACK sem replay, arquivos na UI/backup e leitor2 recusa3. Nativo162 real/scriptado;2700Rust/21ignorados,1255JS/TSC/Vite, produção B1/B2 byte idêntica. F9A: revisão nativa de arquivo/target vazio e retoma, cabeçalhos2/origem/intenção antes dos payloads, subset/prefixo/CAS físico, par/completions/dataRestoredInactive sem .admitted.2713Rust/21ignorados/check/build506;163CLI/IPC real, crash próprio recolhido/retoma, leitores recusam e bytes/originais preservados. Web1255 já validada com fontes idênticas. F9B1C: índice HMAC/protocolo2 sobre o mesmo Authority, negação de decisões/goals históricos, leitura sem permit/owner absence; novo consentimento/claim/budget independente em fixture.2725Rust/21ignorados/12novos/build509 e nativo165 HMAC/CLI/IPC real com0inferência/quota. B2A5: revisão tipada de settings/pets, resolvedor puro comum, diferenças/defaults explícitos e recibo inspectionOnly;2737Rust/21ignorados/12novos/build511/nativo168 real/HMAC sem inferência/quota. B2B1B: referências CID/SID/tarefa antes do codec, guardas comuns de reserva/início/retoma/currentness/publicação, histórico legível e novos contextos em fixtures.2754Rust/21ignorados/17novos/build514;169 é regressão Root1/B1/B2A, sem inferência/quota. B2B2C1: plano nativo/recibo B2A/originais integrais antes de transformar a configuração canónica; Admission2 selada/hash histórico e índice real carregado por Storage antes de Chat.2764Rust/21ignorados/10novos/build518;173 IPC real com HTTP/CLI sintéticos e MPRIS privado: SID novo, objetivo/decisão/Stop/reinício/ACK, IDs antigos recusados.0quota/modelos reais.170 fixture sem workspace e171 erro real de leitura de selo corrigidos;172 chegou ao ACK mas faltou esperar o driver no harness;173 passa, com originais conservados. B2C1A1: backup/inspeção Root2 em formato2 com ambos roots completos, trabalho novo e metadados/originais. Validação comum inicial/HMAC/índices/links sem conceder execução;2770Rust/21ignorados/6novos/build519 e174 nativo com HTTP/CLI sintéticos/MPRIS privado: backup após novo objetivo/Stop/reinício, origem offline, tamper e leitor897/restauro1 recusam2 sem publicar payload. B2C2A4: arquivo2 dos roots completos publicado/revisto/retomado num Header3 inativo, dados antigos byte exatos e admissão2 aninhada como dados. Source1/fence/recibo conservados; restauração de dados e ativação executável projetadas separadamente. Entradas completas reutilizadas também fsync antes da completion.2776Rust/21ignorados/6novos/build519 e177 nativo com copy-worker próprio morto/recolhido e mesmo par retomado, ACK, recibo anterior898 inalterado e recusa de controlo3. B2C3A6: mesma Authority/HMAC/ledger com cadeia limitada autenticada/união de negações, raw grants/consumo desconhecido preservados, revisão nativa Header3/originais completos antes do delta e recuperação de prefixos temporários revistos sem renovar consentimento.2795Rust/21ignorados/19novos/build522/native179 com providers sintéticos e resíduo de staging sintético explicitamente rotulado; A4/178 conservados. B2D1A1: revisão nativa Header3 exige custody C3 completa; recibo2/inspectionOnly liga modelos/endpoints/manifest/ready/histórico e união atual+anterior de CID/SID/tarefa/proveniência limitada. Índice1/recibo1 conservados; Admission2 lê só1.2804Rust/21ignorados/9novos/build522/native180 com hash composto independentemente a partir de bytes reais, providers sintéticos e nenhum efeito adicional. B2D2A4: mesma Activation/Storage com manifest/admission3/job próprio/Scope D1 e mesmos originais C3, History2 antes de Chat e paths2, contexto antigo/atual negado;2811Rust/21ignorados/7novos/build523/native182 com novosSID/objetivo/decisão/Stop/reinício/ACK e arquivo externo ausente, providers sintéticos. Grants/consumo antigos conservados. D3A5: backup3/arquivo2 completo e novo Header3/depth3/C3/D1/D2 com quartoSID e objetivo/decisão próprios, histórico/grants/consumo intactos e reinício/ACK/source offline.2822Rust/21ignorados/11novos/build525/native184 privados. Cópia C3 usa revisões imutáveis+Claim por entrada e hashes integrais antes dos efeitos;183 timeout e A3/A4 erros de expectativa das fixtures conservados. D4A5: preparação nativa para ausência real original, mesmo HMAC/negação, intenção antes de RNG e geração única; sem grants/payload/ativação.2830Rust/21ignorados/8novos/build528/native185,0modelos/quota; A3 prova negativa de regeneração corrigida e conservada. D4B5: mesma chave preparada admitida via manifest/admission4/Header2 e recibo3 separado; adição tipada/prefixos exatos, mesmos originais/ledger/negações, Blobs locais após opened, novosgoals não reclassificados.2837Rust/21ignorados/7novos/build530/native188 com CLI/HTTP sintéticos e0quota/modelos reais,novoSID/objetivo/decisão/Stop/reinício/ACK/backup4; nova épocaAdmission4 emfixtures. D4C2: preflight comum só de leitura antes de metadata e reserva/inferência, sem reparar namespace nem consumir orçamento na recusa. Falha após dispatch conserva reconciliação sem replay.18dirigidos/2841Rust21ignorados/build531/native191 com providers sintéticos; recusa funcional/ledger intacto/reinício/backup. D4D1a: parser estrito comum/keyInvalid/manualRequired e recusa de apply antes de escrever, sem regenerar.11dirigidos/2844Rust21ignorados/build531/native192,0modelos/quota/CLIagents; controlo negativo leitorC2,8chaves inválidas,retoma válida e ready/missing recusados. D4D2A2: CLI nativo retém preparação opaca/cadeia8 com intent antes de rename NOREPLACE, mesmo inode/bytes, fresh review/ready/ACK; sem nova chave/grants, pending normal-reader/P1 bloqueados.11dirigidos/2855Rust21ignorados/build534/native193,0modelos/quota/CLIagents/efeitos. Porta conteúdo emfixture;adoçãooperacionalpendente. D4D2B1: Intent2/generation2 sob OUTRA aprovação/Custody2 Binding1/receipt4 por head preserva1/3/4; Activation/Bound/runtime/backup2 comuns,opened ACK offline/época seguinte fixture.7dirigidos/2864Rust21ignorados/build535/native195 com2HTTP+1CLI sintéticos/0quota. Raiz834 e instalação30 preservadas. B3A2: prova negativa de falsozero com grant histórico desligado; histórico2/origemHMAC exata/consumo desconhecido sem autoridade,mesmoStore/Authority/UI.2870Rust21ignorados/1256web/TSC/Vite/build536/native196 sintético,0quota. Trabalho novo mantém resposta1;originais preservados. B3F2A7: novoGoal/Spec13 origem selada só para dados; contexto/admissão/limites próprios,mesmo runtime/driver/Stop/verificação. Histórico concluído é dado,controlo antigo recusado.32dirigidos/2886Rust21ignorados/1262web/build542/native199 UI e HTTP2CLI1/DBus sintéticos,0quota; leitor915 real recusa13 sem destruir. Época13 seguinte é fixture. B4A1:3dirigidos no RetentionReview real comprovam4096/4097 entradas e64MiB/+1 conjuntos entre originais,8MiB/+1 por ficheiro,bytes/inodes preservados e sem autoridade nova. Só testes mudaram; produção/web iguaisA7 e respetiva prova2886/native199 não repetida. F10A4: prova negativa200 antes da correção; admissão antes da gravação/publicação no escritor comum,0600 exclusivo,temporário estranho conservado,erro final de registo explícito e controlo com readback sem retry.81dirigidos/2892Rust21ignorados/1266web/build542/native201,2HTTP1CLI sintéticos/0quota;UI recusa/memória/bytes/eventos/gravação/restart/backup. Voice/mobile/autostart continuam bloqueados,sem conceder revalidação. F10B1A1: intenção apenas enabled/Settings nativa, snapshots gerais preservam campo, gate impede sobreposição sem segurar mutexglobal no executor. No-op conserva erro anterior sem replay; recibo distingue confirmação de dados e executor.6dirigidos/2898Rust21ignorados/1268web/build544/native202;duasjanelas reais,registo criado/removido sóHOMEprivado,2HTTP1CLI sintéticos/0quota. F10B2A3: campos do telefone, nomes porvoz e indicadores dehooks só entram no Settings/evento após persistência; erros propagados sem falso rollback. Erro telefone para ligação;Stop chat antes da escrita. Reutiliza transação comum, sem novo formato/autoridade.931/547,2905Rust21ignorados/native203; web1268/TSC/Vite reutilizados sem alteração.9comandos nativos sobfalha,0quota/modelos reais. F10B2B1/Linux934: leitores normal/estrito capturam revisão nativa não serializada;writer comum verifica pai/alvo/alias/inode/stamp antes de substituir. Snapshot de janela não renova a origem.12novos/124selecionados1ignorado/2917full21ignorados/check/build550/native204; edição externa recusada antesdeguardar/autostart,reinício privado ebackup.0quota/modelos reais;web1268reutilizado. | NoAuthority normal operacional privado comprovado; geração incerta/seed inválido/pre-manifest/staging anónimo ainda conservados/recusados até reconciliação explícita. Migrações/revalidações voz/mobile/autostart/B3-aceitação-integral/adoção e aceitação integral continuam abertas. F10A não implementa revalidação: faltam intenção por campo/CAS e confirmação após publicação/fsync diretório antes de a conceder. Consumidores não fecham escritor: CAS/fsyncdiretório/resultado incerto tipado/reconciliação/reinício pendentes. Listener ainda antes da gravação e chaves com efeitos próprios; não revalidados/entregues. Isto é verificação otimista para writers cooperantes, não CAS atómico/sandbox deoutros processos do mesmoUID. Incerteza pós-publicação tem classe/recusa de nova escrita,mas barreira global deefeitos/reconciliação durável/reinício/fsyncdiretório eparidade Mac/Windows ainda pendentes. | F10B2C: cercar todos novos efeitos com estado de IO do Shared atual, sem invalidar turnos capturados por simples mudança legítima de outro campo;barreira durável/reconciliação sem replay/fsyncdiretório e intenção do listener/chaves;depois revalidação/adoção/aceitação integral. |
| DATA-02 | pendente opcional | PostgreSQL não integrado; decisão exige benefício, sem acesso a outras bases. | Não há medição que justifique adoção ou migração. | Medir antes de decidir; se adotado, base/role próprios e migração/restauro/rollback provados. |
| REL-01 | parcial integrado + privado; incremento privado910 | Checkpoints, pré-imagens e migrações limitadas; protocolo de preservação documentado. F4 recusa memória entregue oculta em formato antigo e qualquer catálogo pet-runtime-* na importação legada; v7 recusado sem escrita. SDK858/F5 acrescenta revisão/confirmação/inspeção CLI de backup completo dos payloads atuais, sob o mesmo dono offline, com proveniência da admissão. Nativo149 prova81ficheiros/adversariais;151 compõe arranque normal, retoma e backup do perfil recém-criado. F6D usa repositório nativo/CAS e migração IndexedDB conhecida para a janela, preserva UTF16/u64/tombstones e retém CID vazio antes de gravar, sem inferência; nativo154 prova migração, conflito, reinício/reset e inclusão no backup. F7A acrescenta produtor inline, slot unbound nativo sem autoridade e cópia explícita conservando origem, controlos comuns, catálogo2 lê1 e leitor antigo recusa2. Nativo158 nos cartões autorizados prova dados de três pets, ponte inline/flutuante, reinício e backup, sem modelos/quota. F8A entrega listagem nativa por dono/revisão, classificação observada, revisão com bytes/destino/prazo/hash e cópia explícita preservando original, sem mensagens/grants. Nativo160 prova controlos inline/flutuante, recusa de replay/conflito/troca de dono, reinício/leitor anterior e backup. F8B2: revisão de ambas as origens/política/revisões/prazo/hash, original completo imutável antes do catálogo3/CAS/recibo; apenas destinos ausentes podem ser juntos. Origens pendentes recuperáveis, reconhecimento transacional/lostACK sem replay, arquivos na UI/backup e leitor2 recusa3. Nativo162 real/scriptado;2700Rust/21ignorados,1255JS/TSC/Vite, produção B1/B2 byte idêntica. F9A: revisão nativa de arquivo/target vazio e retoma, cabeçalhos2/origem/intenção antes dos payloads, subset/prefixo/CAS físico, par/completions/dataRestoredInactive sem .admitted.2713Rust/21ignorados/check/build506;163CLI/IPC real, crash próprio recolhido/retoma, leitores recusam e bytes/originais preservados. Web1255 já validada com fontes idênticas. F9B1C: índice HMAC/protocolo2 sobre o mesmo Authority, negação de decisões/goals históricos, leitura sem permit/owner absence; novo consentimento/claim/budget independente em fixture.2725Rust/21ignorados/12novos/build509 e nativo165 HMAC/CLI/IPC real com0inferência/quota. B2A5: revisão tipada de settings/pets, resolvedor puro comum, diferenças/defaults explícitos e recibo inspectionOnly;2737Rust/21ignorados/12novos/build511/nativo168 real/HMAC sem inferência/quota. B2B1B: referências CID/SID/tarefa antes do codec, guardas comuns de reserva/início/retoma/currentness/publicação, histórico legível e novos contextos em fixtures.2754Rust/21ignorados/17novos/build514;169 é regressão Root1/B1/B2A, sem inferência/quota. B2B2C1: plano nativo/recibo B2A/originais integrais antes de transformar a configuração canónica; Admission2 selada/hash histórico e índice real carregado por Storage antes de Chat.2764Rust/21ignorados/10novos/build518;173 IPC real com HTTP/CLI sintéticos e MPRIS privado: SID novo, objetivo/decisão/Stop/reinício/ACK, IDs antigos recusados.0quota/modelos reais.170 fixture sem workspace e171 erro real de leitura de selo corrigidos;172 chegou ao ACK mas faltou esperar o driver no harness;173 passa, com originais conservados. B2C1A1: backup/inspeção Root2 em formato2 com ambos roots completos, trabalho novo e metadados/originais. Validação comum inicial/HMAC/índices/links sem conceder execução;2770Rust/21ignorados/6novos/build519 e174 nativo com HTTP/CLI sintéticos/MPRIS privado: backup após novo objetivo/Stop/reinício, origem offline, tamper e leitor897/restauro1 recusam2 sem publicar payload. B2C2A4: arquivo2 dos roots completos publicado/revisto/retomado num Header3 inativo, dados antigos byte exatos e admissão2 aninhada como dados. Source1/fence/recibo conservados; restauração de dados e ativação executável projetadas separadamente. Entradas completas reutilizadas também fsync antes da completion.2776Rust/21ignorados/6novos/build519 e177 nativo com copy-worker próprio morto/recolhido e mesmo par retomado, ACK, recibo anterior898 inalterado e recusa de controlo3. B2C3A6: mesma Authority/HMAC/ledger com cadeia limitada autenticada/união de negações, raw grants/consumo desconhecido preservados, revisão nativa Header3/originais completos antes do delta e recuperação de prefixos temporários revistos sem renovar consentimento.2795Rust/21ignorados/19novos/build522/native179 com providers sintéticos e resíduo de staging sintético explicitamente rotulado; A4/178 conservados. B2D1A1: revisão nativa Header3 exige custody C3 completa; recibo2/inspectionOnly liga modelos/endpoints/manifest/ready/histórico e união atual+anterior de CID/SID/tarefa/proveniência limitada. Índice1/recibo1 conservados; Admission2 lê só1.2804Rust/21ignorados/9novos/build522/native180 com hash composto independentemente a partir de bytes reais, providers sintéticos e nenhum efeito adicional. B2D2A4: mesma Activation/Storage com manifest/admission3/job próprio/Scope D1 e mesmos originais C3, History2 antes de Chat e paths2, contexto antigo/atual negado;2811Rust/21ignorados/7novos/build523/native182 com novosSID/objetivo/decisão/Stop/reinício/ACK e arquivo externo ausente, providers sintéticos. Grants/consumo antigos conservados. D3A5: backup3/arquivo2 completo e novo Header3/depth3/C3/D1/D2 com quartoSID e objetivo/decisão próprios, histórico/grants/consumo intactos e reinício/ACK/source offline.2822Rust/21ignorados/11novos/build525/native184 privados. Cópia C3 usa revisões imutáveis+Claim por entrada e hashes integrais antes dos efeitos;183 timeout e A3/A4 erros de expectativa das fixtures conservados. D4A5: preparação nativa para ausência real original, mesmo HMAC/negação, intenção antes de RNG e geração única; sem grants/payload/ativação.2830Rust/21ignorados/8novos/build528/native185,0modelos/quota; A3 prova negativa de regeneração corrigida e conservada. D4B5: mesma chave preparada admitida via manifest/admission4/Header2 e recibo3 separado; adição tipada/prefixos exatos, mesmos originais/ledger/negações, Blobs locais após opened, novosgoals não reclassificados.2837Rust/21ignorados/7novos/build530/native188 com CLI/HTTP sintéticos e0quota/modelos reais,novoSID/objetivo/decisão/Stop/reinício/ACK/backup4; nova épocaAdmission4 emfixtures. D4C2: preflight comum só de leitura antes de metadata e reserva/inferência, sem reparar namespace nem consumir orçamento na recusa. Falha após dispatch conserva reconciliação sem replay.18dirigidos/2841Rust21ignorados/build531/native191 com providers sintéticos; recusa funcional/ledger intacto/reinício/backup. D4D1a: parser estrito comum/keyInvalid/manualRequired e recusa de apply antes de escrever, sem regenerar.11dirigidos/2844Rust21ignorados/build531/native192,0modelos/quota/CLIagents; controlo negativo leitorC2,8chaves inválidas,retoma válida e ready/missing recusados. D4D2A2: CLI nativo retém preparação opaca/cadeia8 com intent antes de rename NOREPLACE, mesmo inode/bytes, fresh review/ready/ACK; sem nova chave/grants, pending normal-reader/P1 bloqueados.11dirigidos/2855Rust21ignorados/build534/native193,0modelos/quota/CLIagents/efeitos. Porta conteúdo emfixture;adoçãooperacionalpendente. D4D2B1: Intent2/generation2 sob OUTRA aprovação/Custody2 Binding1/receipt4 por head preserva1/3/4; Activation/Bound/runtime/backup2 comuns,opened ACK offline/época seguinte fixture.7dirigidos/2864Rust21ignorados/build535/native195 com2HTTP+1CLI sintéticos/0quota. Raiz834 e instalação30 preservadas. B3A2: prova negativa de falsozero com grant histórico desligado; histórico2/origemHMAC exata/consumo desconhecido sem autoridade,mesmoStore/Authority/UI.2870Rust21ignorados/1256web/TSC/Vite/build536/native196 sintético,0quota. Trabalho novo mantém resposta1;originais preservados. B3F2A7: novoGoal/Spec13 origem selada só para dados; contexto/admissão/limites próprios,mesmo runtime/driver/Stop/verificação. Histórico concluído é dado,controlo antigo recusado.32dirigidos/2886Rust21ignorados/1262web/build542/native199 UI e HTTP2CLI1/DBus sintéticos,0quota; leitor915 real recusa13 sem destruir. Época13 seguinte é fixture. B4A1:3dirigidos no RetentionReview real comprovam4096/4097 entradas e64MiB/+1 conjuntos entre originais,8MiB/+1 por ficheiro,bytes/inodes preservados e sem autoridade nova. Só testes mudaram; produção/web iguaisA7 e respetiva prova2886/native199 não repetida. F10A4: prova negativa200 antes da correção; admissão antes da gravação/publicação no escritor comum,0600 exclusivo,temporário estranho conservado,erro final de registo explícito e controlo com readback sem retry.81dirigidos/2892Rust21ignorados/1266web/build542/native201,2HTTP1CLI sintéticos/0quota;UI recusa/memória/bytes/eventos/gravação/restart/backup. Voice/mobile/autostart continuam bloqueados,sem conceder revalidação. F10B1A1: intenção apenas enabled/Settings nativa, snapshots gerais preservam campo, gate impede sobreposição sem segurar mutexglobal no executor. No-op conserva erro anterior sem replay; recibo distingue confirmação de dados e executor.6dirigidos/2898Rust21ignorados/1268web/build544/native202;duasjanelas reais,registo criado/removido sóHOMEprivado,2HTTP1CLI sintéticos/0quota. F10B2A3: campos do telefone, nomes porvoz e indicadores dehooks só entram no Settings/evento após persistência; erros propagados sem falso rollback. Erro telefone para ligação;Stop chat antes da escrita. Reutiliza transação comum, sem novo formato/autoridade.931/547,2905Rust21ignorados/native203; web1268/TSC/Vite reutilizados sem alteração.9comandos nativos sobfalha,0quota/modelos reais. F10B2B1/Linux934: leitores normal/estrito capturam revisão nativa não serializada;writer comum verifica pai/alvo/alias/inode/stamp antes de substituir. Snapshot de janela não renova a origem.12novos/124selecionados1ignorado/2917full21ignorados/check/build550/native204; edição externa recusada antesdeguardar/autostart,reinício privado ebackup.0quota/modelos reais;web1268reutilizado. | NoAuthority normal operacional privado comprovado; geração incerta/seed inválido/pre-manifest/staging anónimo ainda conservados/recusados até reconciliação explícita. Migrações/revalidações voz/mobile/autostart/B3-aceitação-integral/adoção e aceitação integral continuam abertas. F10A não implementa revalidação: faltam intenção por campo/CAS e confirmação após publicação/fsync diretório antes de a conceder. Consumidores não fecham escritor: CAS/fsyncdiretório/resultado incerto tipado/reconciliação/reinício pendentes. Listener ainda antes da gravação e chaves com efeitos próprios; não revalidados/entregues. Isto é verificação otimista para writers cooperantes, não CAS atómico/sandbox deoutros processos do mesmoUID. Incerteza pós-publicação tem classe/recusa de nova escrita,mas barreira global deefeitos/reconciliação durável/reinício/fsyncdiretório eparidade Mac/Windows ainda pendentes. | F10B2C: cercar todos novos efeitos com estado de IO do Shared atual, sem invalidar turnos capturados por simples mudança legítima de outro campo;barreira durável/reconciliação sem replay/fsyncdiretório e intenção do listener/chaves;depois revalidação/adoção/aceitação integral. |
| REL-02 | pendente de entrega | Instalação operacional .30 preservada; builds privadas posteriores não ativadas. | Autostart/login/hotplug/lifecycle/áudio e aceitação integrada da nova composição pendentes. | Entregar somente no âmbito autorizado, provar comportamento real e preservar legados. |
| GUA-01 | desenhado; pendente de implementação | Sem módulo/identidade Guardian operacional. Fábrica/runtime/brokers existentes são bases a reutilizar, não implementação destes findings ou remediações. | Separar binding lógico/motor/papel e implementar os módulos/cenários deste ID sobre as dependências após R43. | PET-01/04/05, EXE-01/02, OBS-01, SYS-01. Componentes/versões/serviços/processos/exposição/recursos/logs selecionados e estado desejado, com origem/data/cobertura e distinção gerido/observado/externo/privado/dinâmico; percurso determinístico sem LLM e sem varrimento pessoal indiscriminado |
| GUA-02 | desenhado; pendente de implementação | Sem módulo/identidade Guardian operacional. Fábrica/runtime/brokers existentes são bases a reutilizar, não implementação destes findings ou remediações. | Separar binding lógico/motor/papel e implementar os módulos/cenários deste ID sobre as dependências após R43. | GUA-01, CTX-05, EXE-02/03, DATA-01. Componente/versão, evidência/origem/data, severidade/confiança/estado/recomendação; distinguir confirmação/candidata/suspeita, conferir advisories da distribuição e pacote; não inventar CVE, inferir vulnerabilidade só por upstream ou executar exploração/scans intrusivos |
| GUA-03 | desenhado; pendente de implementação | Sem módulo/identidade Guardian operacional. Fábrica/runtime/brokers existentes são bases a reutilizar, não implementação destes findings ou remediações. | Separar binding lógico/motor/papel e implementar os módulos/cenários deste ID sobre as dependências após R43. | GUA-01, OBS-01/02, SYS-03. CPU/memória/IO/GPU suportada/arranque/falhas/conflitos, hipótese/métrica/impacto/custos e validação; fixtures de consumo, dock Thunderbolt e VPN/Tailscale/firewall sem mutações reais, tuning genérico, limpeza destrutiva ou desativação de proteções |
| GUA-04 | desenhado; pendente de implementação | Sem módulo/identidade Guardian operacional. Fábrica/runtime/brokers existentes são bases a reutilizar, não implementação destes findings ou remediações. | Separar binding lógico/motor/papel e implementar os módulos/cenários deste ID sobre as dependências após R43. | GUA-01, EXE-01/02/03, DATA-01, REL-01. Contratos reais de plano/validação/backup/aplicação/recuperação, estado desejado efetivamente gerido e exclusões Coucou/Mochi/projetos preservados; integração de excluído é proposta separada, sem comandos ou fonte de verdade inventados |
| GUA-05 | desenhado; pendente de implementação | Sem módulo/identidade Guardian operacional. Fábrica/runtime/brokers existentes são bases a reutilizar, não implementação destes findings ou remediações. | Separar binding lógico/motor/papel e implementar os módulos/cenários deste ID sobre as dependências após R43. | GUA-01/04, RUN, EXE, SYS-01, REL-01/02. Finding→proposta/diff→impacto/pré-condições→autorização→execução limitada→verificação→resultado; receitas específicas sob grants explícitos e aprovação exata para sensíveis; sem shell/root geral, autoaprovação ou replay; rollback só suportado/provado, compensação/irreversibilidade explícitas |
| GUA-06 | desenhado; pendente de implementação | Sem módulo/identidade Guardian operacional. Fábrica/runtime/brokers existentes são bases a reutilizar, não implementação destes findings ou remediações. | Separar binding lógico/motor/papel e implementar os módulos/cenários deste ID sobre as dependências após R43. | GUA-01/05, AUTO, TEAM, OBS. Gatilhos leves/debounce/deduplicação/limites no scheduler, orçamento e notificações comuns; LLM só quando útil; pedidos diretos/coordenador opcional, delegação autorizada sem ampliar acesso, ausência de motor/coordenador e conteúdo hostil sem alertas/efeitos repetidos |

## Ficheiros, símbolos, dependências e provas por requisito

### ARC-01 — Separar arquitetura, estado e histórico; atualizar prioridade para arquitetura e pets

**Dependências e critérios preservados:** Documento normativo, plano com IDs, fontes identificadas e arquivo anterior preservado; ligações coerentes

**Responsabilidade observada / bases a reutilizar:**

- [docs/PETS-ARCHITECTURE.md](../docs/PETS-ARCHITECTURE.md) — `Arquitetura dos pets autónomos de AI`, `R43F10B2B1 — revisão carregada antes de substituir preferências`
- [docs/PETS-ROADMAP.md](../docs/PETS-ROADMAP.md) — `Plano de desenvolvimento dos pets autónomos`, `R43F10B2B1 — revisão carregada antes de substituir preferências`

**Prova disponível e alcance:**

- [COUCOU-GUARDIAN-CONTRACT-REGISTRATION-20261025.json](../.build/COUCOU-GUARDIAN-CONTRACT-REGISTRATION-20261025.json) — Preservação documental/IDs/histórico; não execução de produto.

**Limite / pendência:** Cobertura continua a evoluir com implementação.

**Aceitação integral:** aberta. Rever ligações,49 IDs e divergências antes de um caminho incompatível.

### ARC-02 — Rever contratos quando código/decisão revelar incompatibilidade

**Dependências e critérios preservados:** Registar decisão, motivo, migração, impacto e requisitos afetados antes de implementar um caminho incompatível

**Responsabilidade observada / bases a reutilizar:**

- [docs/PET-CAPABILITIES-CONTRACT.md](../docs/PET-CAPABILITIES-CONTRACT.md) — `Contrato de identidade e capacidades dos pets — v1`, `R43F10B2B1 — revisão carregada antes de substituir preferências`
- [docs/SYSTEM-GUARDIAN-ARCHITECTURE.md](../docs/SYSTEM-GUARDIAN-ARCHITECTURE.md) — `System Guardian — especialização do runtime comum`, `1. Identidade, motor, runtime e papel`

**Prova disponível e alcance:**

- [COUCOU-GUARDIAN-CONTRACT-REGISTRATION-20261025.json](../.build/COUCOU-GUARDIAN-CONTRACT-REGISTRATION-20261025.json) — Preservação documental/IDs/histórico; não execução de produto.

**Limite / pendência:** Binding de novo dono lógico e lifecycle ainda exigem decisão técnica completa.

**Aceitação integral:** aberta. Associar cada decisão ao delta de código, migração e prova efetiva.

### ARC-03 — Manter definição de sistema completo e cobertura

**Dependências e critérios preservados:** Cada cenário do dia a dia tem capacidade, implementação, prova e pendência por provider; exceções não desaparecem dos critérios

**Responsabilidade observada / bases a reutilizar:**

- [docs/PETS-ROADMAP.md](../docs/PETS-ROADMAP.md) — `Plano de desenvolvimento dos pets autónomos`, `R43F10B2B1 — revisão carregada antes de substituir preferências`
- [docs/PETS-IMPLEMENTATION-STATUS.md](../docs/PETS-IMPLEMENTATION-STATUS.md) — `Estado de implementação dos pets`, `R43F10B2B1 — revisão carregada antes de substituir preferências`

**Prova disponível e alcance:**

- [COUCOU-GUARDIAN-CONTRACT-REGISTRATION-20261025.json](../.build/COUCOU-GUARDIAN-CONTRACT-REGISTRATION-20261025.json) — Preservação documental/IDs/histórico; não execução de produto.

**Limite / pendência:** Nenhuma prova global fecha os43 ou o Guardian.

**Aceitação integral:** aberta. Aceitar as matrizes e cenários integrais com exceções resolvidas.

### PET-01 — Implementar `PetDescriptor` e capacidades versionadas a partir de `pet_factory`, `pet_config`, modelos e adaptadores

**Dependências e critérios preservados:** Matriz por pet distingue suporte, configuração, disponibilidade, frescura e motivo. API/perfil vazio ou hook não aparecem como execução suportada. Nome/modelo não alteram identidade; GitHub entidade continua retirado

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/pet_factory.rs](../windows/src-tauri/src/pet_factory.rs) — `catalog`
- [windows/src-tauri/src/pet_capabilities/mod.rs](../windows/src-tauri/src/pet_capabilities/mod.rs) — `Revision`, `Registry`, `new`
- [windows/src/core/pet-factory.ts](../windows/src/core/pet-factory.ts) — `invalid`, `merge`, `freeze`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_capabilities/mod.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_capabilities/mod.rs) — `Revision`, `Registry`, `new`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/runtime-goals.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/runtime-goals.ts) — `RuntimeGoalActions`, `musicGoalPlan`, `templateCriteria`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/memory-controls.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/memory-controls.ts) — `buildMemoryControls`, `tell`, `resetEditor`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/memory-review.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/memory-review.ts) — `memoryRecord`, `goalMemoryReview`, `manualMemoryReview`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/runtime-goals.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/runtime-goals.ts) — `buildRuntimeGoals`, `syncMusic`, `syncPets`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_memory/context.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_memory/context.rs) — `Capture`, `catalog_path`, `at`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_memory/manual.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_memory/manual.rs) — `Resolved`, `reviews`, `failure`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/bridge.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/bridge.ts) — `call`, `runtimeBudget`, `runtimeDraftAction`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/session-chat.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/session-chat.ts) — `nativeSessionReady`, `constrainNative`, `SessionChat`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/desktop/pet-chat.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/desktop/pet-chat.ts) — `acceptRect`, `fromLocal`, `report`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/chat.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/chat.ts) — `watchCapabilities`, `capabilityText`, `bubble`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/manual-memory.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/manual-memory.ts) — `buildManualMemory`, `finish`, `discard`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/pet-factory.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/pet-factory.ts) — `invalid`, `merge`, `freeze`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Relatos/checkpoints por composição; as provas parciais não cobrem todos os critérios deste ID.
- [HUMAN-GOAL-MEMORY-PARTIAL-CHECKPOINT-R43F2E1.json](../.build/pet-goal-execution-next-r1/HUMAN-GOAL-MEMORY-PARTIAL-CHECKPOINT-R43F2E1.json) — SDK850:2634 Rust/21 ignorados (487 inputs nativos idênticos A/B),1200 JS/TSC/Vite/check/build e Tauri139. Controlos reais de catálogo/seleção/revisão/consentimento e resposta Ollama/LMStudio, revogação antes da confirmação e leitor antigo. Escolhas automatizadas e motores sintéticos, não aceitação humana/instalada ou revisão manual de chat.
- [MANUAL-MEMORY-PARTIAL-CHECKPOINT-R43F2E2.json](../.build/pet-goal-execution-next-r1/MANUAL-MEMORY-PARTIAL-CHECKPOINT-R43F2E2.json) — SDK854/E2b:2638 Rust/21 ignorados (488 inputs nativos idênticos A/B),1212 JS/TSC/Vite/check/build. Tauri143 janela própria local2/CLI3/Notion com30 hit-tests, contexto vinculado/reinício (3HTTP/4CLI);144 preservação13HTTP/17CLI. Escolhas/protocolos sintéticos; inline nativo falhou ativação em140 e permanece pendente. Sem aceitação humana/instalada.
- [INLINE-DRAFTS-PARTIAL-CHECKPOINT-R43F7.json](../.build/pet-goal-execution-next-r1/INLINE-DRAFTS-PARTIAL-CHECKPOINT-R43F7.json) — SDK869/F7A:2674Rust/21ignorados/check/build,1230JS/types/bundle e nativo158 de cartões autorizados. Inline/unbound/CAS/cópia explícita, leitor antigo/backup;0modelos/quota. Digitação nativa inicial, pre-CID não nativo, órfãos/divergência/restauro/entrega ainda abertos.

**Limite / pendência:** Projeção por caller e controlos de memória validados em privado854; separação papel/motor e adoção integral ainda pendentes.

**Aceitação integral:** aberta. Adotar projeções e testar scopes/callers em todos os percursos sem promover capacidades indisponíveis.

**Incremento939/C1:** origem atual de IO e preservação de Stop/recusa/evidência;14fixtures/2931full21ignorados/1272web/build552/native205 sintético. [Checkpoint](../.build/pet-goal-execution-next-r1/SETTINGS-EFFECT-GUARD-PARTIAL-CHECKPOINT-R43F10B2C1.json). Guardas finais e reconciliação durável pendentes, sem entrega integral.

**Incremento948/C3A4:** intenção/resultado/revisão de preferências duráveis e formato de perfil/backup/restauro; 13dirigidos/2993full21ignorados/build561/native212. [Checkpoint](../.build/pet-goal-execution-next-r1/SETTINGS-DURABLE-PARTIAL-CHECKPOINT-R43F10B2C3A4.json). Reconciliação humana e listener/chaves ainda pendentes; sem adoção/instalação.

- [settings_durable.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings_durable.rs) — `Record`, `Observation`, `Prepared`, `restored_record`
- [settings_revision.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings_revision.rs) — `Bound`, `save_with`, `note_failed`
- [preferences_file.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/preferences_file.rs) — `commit_with`

### PET-02 — Associar conversa, sessão nativa, turno, tarefa e decisões com autoridade inequívoca

**Dependências e critérios preservados:** Trocar de pet/vista/sessão, resultados tardios, sessão ambígua, revogação e Stop preservam dono. Terminal/chat/voz partilham a identidade do pedido nativo e a política da sessão/perfil; uma resposta invalida as outras representações. Descoberta por PID/pasta/título não autoriza controlo

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/cli_chat.rs](../windows/src-tauri/src/cli_chat.rs) — `run_inner`, `backend`
- [windows/src-tauri/src/chat.rs](../windows/src-tauri/src/chat.rs) — `Chat`
- [windows/src-tauri/src/cli_decisions.rs](../windows/src-tauri/src/cli_decisions.rs) — `target`, `live`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_history.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_history.rs) — `capture_next`, `decode_first`, `review`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/chat_restored_history_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/chat_restored_history_tests.rs) — `fixture`, `historical_conversation_is_readable_with_notice_but_cannot_reserve_or_start_a_turn`, `a_fresh_conversation_can_obtain_normal_reservation_while_old_history_is_preserved`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/chat.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/chat.rs) — `Chat`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/bootstrap.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/bootstrap.rs) — `Session`, `prepare`, `paths`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_transaction.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_transaction.rs) — `Activation`, `initialization_delta`, `EpochInitialization`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_projection.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_projection.rs) — `Choices`, `Grant`, `Operation`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_active_restored.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_active_restored.rs) — `Bound`, `verify_initial_payload`, `publish`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_transaction_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_transaction_tests.rs) — `Fixture`, `new`, `with_authority`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restore_cli.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restore_cli.rs) — `activation_sealed`, `execute`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restored_backup.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restored_backup.rs) — `cannot`, `verify`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/current_profile.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/current_profile.rs) — `Review`, `capture`, `fingerprint`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/backup_cli.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/backup_cli.rs) — `Request`, `parse`, `code`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/staging.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/staging.rs) — `Failure`, `Phase`, `Inspection`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore.rs) — `Intent`, `Restored`, `Mode`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_authority.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_authority.rs) — `error`, `AuthorityReview`, `FirstReview`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration.rs) — `FileRef`, `HistoricalGrant`, `GoalRef`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_epochs.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_epochs.rs) — `EpochStep`, `EpochReview`, `prior_staging`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_epoch.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_epoch.rs) — `capture_parts`, `completed`, `verify_current`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_configuration.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_configuration.rs) — `configuration`, `epoch_scope`, `verify_payload`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/revision_fence.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/revision_fence.rs) — `Tree`, `descriptor_budget`, `capture`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_backup_epoch_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_backup_epoch_tests.rs) — `Round`, `new`, `archive`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent.rs) — `preparation_state`, `preview`, `apply_using`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_independent.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_independent.rs) — `IndependentKeyState`, `key_state`, `prepare`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent_operational.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent_operational.rs) — `Custody`, `validate`, `PreparedAuthority`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_independent_transaction_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_independent_transaction_tests.rs) — `prepared`, `old_goal`, `independent_admission4_preserves_absent_original_authority_and_denies_old_ids_before_new_consent`

**Prova disponível e alcance:**

- [APP-NATIVE-NATIVE-MEMORY-R43E1B.json](../.build/pet-goal-execution-next-r1/APP-NATIVE-NATIVE-MEMORY-R43E1B.json) — IPC Tauri nativo com CLI sintéticas; não conta operacional ou áudio.
- [RESTORED-SESSION-GUARDS-PARTIAL-CHECKPOINT-R43F9B2B1.json](../.build/pet-goal-execution-next-r1/RESTORED-SESSION-GUARDS-PARTIAL-CHECKPOINT-R43F9B2B1.json) — SDK893/B2B1B:2754Rust/21ignorados/17novos/build514;169 regressão real Root1/claim/backup/B1/B2A,0inferência/quota. Guards em fixtures; NÃO restauro operacional/admissão/guard nativo ativo ou aceitação humana/instalada.
- [RESTORED-ACTIVATION-PARTIAL-CHECKPOINT-R43F9B2B2.json](../.build/pet-goal-execution-next-r1/RESTORED-ACTIVATION-PARTIAL-CHECKPOINT-R43F9B2B2.json) — 897/B2B2C1 primeira ativação operacional privada;2764Rust21ignorados/10novos/build518/nativo173 com providers sintéticos. Não fecha backup Root2/épocas/revalidações/B3/adoção/aceitação humana ou instalada.
- [PORTABLE-RESTORED-BACKUP-PARTIAL-CHECKPOINT-R43F9B2C.json](../.build/pet-goal-execution-next-r1/PORTABLE-RESTORED-BACKUP-PARTIAL-CHECKPOINT-R43F9B2C.json) — 898/C1A1 backup/inspeção de ambos roots completos;2770Rust21ignorados/6novos/build519/native174 real com providers sintéticos. Não implementa restauro/ativação da época seguinte nem fecha adoção/entrega.
- [NEXT-EPOCH-DATA-RESTORE-PARTIAL-CHECKPOINT-R43F9B2C2.json](../.build/pet-goal-execution-next-r1/NEXT-EPOCH-DATA-RESTORE-PARTIAL-CHECKPOINT-R43F9B2C2.json) — 898/C2A4 dados2/Header3 inativo,2776Rust21ignorados/6novos/build519/nativo177 com providers sintéticos; processo de cópia próprio terminado/recolhido/retomado, compatibilidade anterior898, bytes/recibo/par e controlos recusados. Não é admissão3, falha física de energia ou entrega instalada.
- [MULTI-EPOCH-AUTHORITY-PARTIAL-CHECKPOINT-R43F9B2C3.json](../.build/pet-goal-execution-next-r1/MULTI-EPOCH-AUTHORITY-PARTIAL-CHECKPOINT-R43F9B2C3.json) — 901/C3A6:2795Rust21ignorados/19novos/build522/native179; cadeia/negação/consumo/originais/revisão e prefixo temporário sintético no CLI real. Header3 inativo, sem admissão3/entrega/humana ou provider real.
- [EPOCH-CONFIGURATION-PARTIAL-CHECKPOINT-R43F9B2D1.json](../.build/pet-goal-execution-next-r1/EPOCH-CONFIGURATION-PARTIAL-CHECKPOINT-R43F9B2D1.json) — 901/D1A1:2804Rust/21ignorados/9novos/build522/native180; revisão/recibo2 e hash da união de referências composto independentemente, providers sintéticos. Header3 inativo; não admissão3/entrega/humana ou provider real.
- [EPOCH-ADMISSION-PARTIAL-CHECKPOINT-R43F9B2D2.json](../.build/pet-goal-execution-next-r1/EPOCH-ADMISSION-PARTIAL-CHECKPOINT-R43F9B2D2.json) — 902/D2A4:2811Rust/21ignorados/7novos/build523/native182; admissão3/History2/SID/objetivo/Stop/reinício/ACK/source offline com providers sintéticos. Backup3/adoção/entrega e aceitação humana/real pendentes.
- [RECURSIVE-BACKUP-PARTIAL-CHECKPOINT-R43F9B2D3.json](../.build/pet-goal-execution-next-r1/RECURSIVE-BACKUP-PARTIAL-CHECKPOINT-R43F9B2D3.json) — 904/D3A5:2822Rust/21ignorados/11novos/build525/native184, backup3 e restauro/admissão bounded depth3 com providers sintéticos. NoAuthority/revalidações/B3/adoção/aceitação integral pendentes.
- [INDEPENDENT-AUTHORITY-PREPARATION-PARTIAL-CHECKPOINT-R43F9B2D4A.json](../.build/pet-goal-execution-next-r1/INDEPENDENT-AUTHORITY-PREPARATION-PARTIAL-CHECKPOINT-R43F9B2D4A.json) — 907/A5:2830Rust/21ignorados/8novos/build528/native185,0modelos/quota; apenas preparação de Authority independente, não admissão operacional. Prova negativa de regeneração antes da correção, cortes/tamper/negações/ACK; instalada/humana e D4B pendentes.
- [INDEPENDENT-ADMISSION-PARTIAL-CHECKPOINT-R43F9B2D4B.json](../.build/pet-goal-execution-next-r1/INDEPENDENT-ADMISSION-PARTIAL-CHECKPOINT-R43F9B2D4B.json) — 909/B5:2837Rust/21ignorados/7novos/build530/native188, providers sintéticos; mesmaKey/admission4/recibo3/novoSID+goal+decisão+Stop+restart+ACK+backup4. Novaépoca emfixtures. Geração incerta/revalidações/B3/adoção/aceitação integral pendentes.

**Limite / pendência:** NoAuthority normal operacional privado comprovado; geração incerta/seed inválido/pre-manifest/staging anónimo ainda conservados/recusados até reconciliação explícita. Migrações/revalidações voz/mobile/autostart/B3/adoção e aceitação integral continuam abertas.

**Aceitação integral:** aberta. Fechar recuperação explícita de geração incerta e staging sem regeneração silenciosa; depois revalidações/migrações/B3/adoção antes TEAM/Guardian.

**Incremento940/C2A3:** origem atual na escrita nativa final, controlo/recusa tipados e pre-spawn;21dirigidos/2952full21ignorados/build553/native206 com corte real de4096bytes num peer sintético. [Checkpoint](../.build/pet-goal-execution-next-r1/FINAL-NATIVE-DISPATCH-PARTIAL-CHECKPOINT-R43F10B2C2A3.json). C2B/C3 e aceitação integral continuam abertos.

- [pet_native_write.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_native_write.rs) — `Dispatch`, `Guard`, `frame`
- [pet_native_choices.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_native_choices.rs) — `pure_refusal`, `codex_fixed_refusal`

**Incremento948/C3A4:** intenção/resultado/revisão de preferências duráveis e formato de perfil/backup/restauro; 13dirigidos/2993full21ignorados/build561/native212. [Checkpoint](../.build/pet-goal-execution-next-r1/SETTINGS-DURABLE-PARTIAL-CHECKPOINT-R43F10B2C3A4.json). Reconciliação humana e listener/chaves ainda pendentes; sem adoção/instalação.

- [settings_durable.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings_durable.rs) — `Record`, `Observation`, `Prepared`, `restored_record`
- [settings_revision.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings_revision.rs) — `Bound`, `save_with`, `note_failed`
- [preferences_file.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/preferences_file.rs) — `commit_with`

### PET-03 — Completar continuidade e decisões Gravity/Copilot/Codex segundo capacidades nativas

**Dependências e critérios preservados:** Continuar a execução original quando suportado, decisões exatas e limpeza dos filhos próprios. Gravity sem canal de aprovação não ganha permissões. Copilot usa fixtures enquanto quota real está vedada. Preservar as capacidades do agente original — ficheiros, comandos, projetos e ferramentas/MCP — conforme a sessão/perfil autorizados; distinguir continuação nativa do dispatcher autónomo ainda sem limites internos garantidos. Verificar paridade de sandbox/política/modos de aprovação, sem flags mais permissivas nem concessões copiadas entre sessões

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/cli_chat.rs](../windows/src-tauri/src/cli_chat.rs) — `run_inner`, `backend`
- [windows/src-tauri/src/codex_chat/rpc.rs](../windows/src-tauri/src/codex_chat/rpc.rs) — `in`, `tool_available`, `TurnOptions`
- [windows/src-tauri/src/gravity_chat/rpc.rs](../windows/src-tauri/src/gravity_chat/rpc.rs) — `SessionIdentity`, `native_id`, `new`

**Prova disponível e alcance:**

- [APP-NATIVE-NATIVE-MEMORY-R43E1B.json](../.build/pet-goal-execution-next-r1/APP-NATIVE-NATIVE-MEMORY-R43E1B.json) — IPC Tauri nativo com CLI sintéticas; não conta operacional ou áudio.

**Limite / pendência:** Quota real Copilot vedada; Gravity/canais e limites de autonomia CLI não são universais.

**Aceitação integral:** aberta. Provar por protocolo ferramentas/MCP, sandbox, decisões, Stop/recolha e controlo suportado; não ativar autonomia sem limites.

**Incremento940/C2A3:** origem atual na escrita nativa final, controlo/recusa tipados e pre-spawn;21dirigidos/2952full21ignorados/build553/native206 com corte real de4096bytes num peer sintético. [Checkpoint](../.build/pet-goal-execution-next-r1/FINAL-NATIVE-DISPATCH-PARTIAL-CHECKPOINT-R43F10B2C2A3.json). C2B/C3 e aceitação integral continuam abertos.

- [codex_chat/rpc.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/codex_chat/rpc.rs) — `write_frame_guarded`, `finish_skipped_response`
- [copilot_chat/rpc.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/copilot_chat/rpc.rs) — `write_until_guarded`, `respond_native`
- [gravity_chat/rpc.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/gravity_chat/rpc.rs) — `write_message`
- [session_chat/native_write.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/session_chat/native_write.rs) — `Scope`, `Socket`

### PET-04 — Consolidar identidade, modelos, ferramentas e falhas locais/API

**Dependências e critérios preservados:** Ollama/LM Studio/API mantêm modelo e contexto escolhidos, recusam ferramentas não suportadas e não fazem fallback silencioso. Completar ligação de pets ainda sem configuração/adaptador resolvido

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/local_chat.rs](../windows/src-tauri/src/local_chat.rs) — `server`, `send_runtime_once`
- [windows/src-tauri/src/lmstudio.rs](../windows/src-tauri/src/lmstudio.rs) — `Binding`, `id`, `digest`
- [windows/src-tauri/src/pet_factory_runtime.rs](../windows/src-tauri/src/pet_factory_runtime.rs) — `parse`, `recognizes_owner`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/claude.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/claude.rs) — `send`, `send_resolved`, `call_with_timeout`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/openai_compat.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/openai_compat.rs) — `send`, `send_resolved`, `call`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Relatos/checkpoints por composição; as provas parciais não cobrem todos os critérios deste ID.
- [MANUAL-MEMORY-PARTIAL-CHECKPOINT-R43F2E2.json](../.build/pet-goal-execution-next-r1/MANUAL-MEMORY-PARTIAL-CHECKPOINT-R43F2E2.json) — SDK854/E2b:2638 Rust/21 ignorados (488 inputs nativos idênticos A/B),1212 JS/TSC/Vite/check/build. Tauri143 janela própria local2/CLI3/Notion com30 hit-tests, contexto vinculado/reinício (3HTTP/4CLI);144 preservação13HTTP/17CLI. Escolhas/protocolos sintéticos; inline nativo falhou ativação em140 e permanece pendente. Sem aceitação humana/instalada.

**Limite / pendência:** API HTTP/rotas têm prova sintética; janela própria manual cobre local2/CLI3/Notion. Inline nativo, contas/modelos reais e binding próprio Guardian não aceites.

**Aceitação integral:** aberta. Fechar retoma/adoção R43 e separar binding lógico/motor na fábrica com migração e testes.

**Incremento939/C1:** origem atual de IO e preservação de Stop/recusa/evidência;14fixtures/2931full21ignorados/1272web/build552/native205 sintético. [Checkpoint](../.build/pet-goal-execution-next-r1/SETTINGS-EFFECT-GUARD-PARTIAL-CHECKPOINT-R43F10B2C1.json). Guardas finais e reconciliação durável pendentes, sem entrega integral.

**Incremento940/C2A3:** origem atual na escrita nativa final, controlo/recusa tipados e pre-spawn;21dirigidos/2952full21ignorados/build553/native206 com corte real de4096bytes num peer sintético. [Checkpoint](../.build/pet-goal-execution-next-r1/FINAL-NATIVE-DISPATCH-PARTIAL-CHECKPOINT-R43F10B2C2A3.json). C2B/C3 e aceitação integral continuam abertos.

- [codex_chat/rpc.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/codex_chat/rpc.rs) — `start_inner_timed`
- [copilot_chat/rpc.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/copilot_chat/rpc.rs) — `start_with_cancel`
- [gravity_chat/rpc.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/gravity_chat/rpc.rs) — `start`

**Incremento948/C3A4:** intenção/resultado/revisão de preferências duráveis e formato de perfil/backup/restauro; 13dirigidos/2993full21ignorados/build561/native212. [Checkpoint](../.build/pet-goal-execution-next-r1/SETTINGS-DURABLE-PARTIAL-CHECKPOINT-R43F10B2C3A4.json). Reconciliação humana e listener/chaves ainda pendentes; sem adoção/instalação.

- [settings_durable.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings_durable.rs) — `Record`, `Observation`, `Prepared`, `restored_record`
- [settings_revision.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings_revision.rs) — `Bound`, `save_with`, `note_failed`
- [preferences_file.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/preferences_file.rs) — `commit_with`

### PET-05 — Definir o contrato comum dos pets de serviço e observadores

**Dependências e critérios preservados:** Spotify permanece determinístico nas capacidades atuais; integrações com executor LLM conservam dono próprio. Só operações efetivamente implementadas ficam disponíveis

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/pet_factory_runtime.rs](../windows/src-tauri/src/pet_factory_runtime.rs) — `parse`, `recognizes_owner`
- [windows/src-tauri/src/pet_config.rs](../windows/src-tauri/src/pet_config.rs) — `select`, `retired`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Relatos/checkpoints por composição; as provas parciais não cobrem todos os critérios deste ID.

**Limite / pendência:** Catálogo não comprova operações nativas de todos os serviços ou novos especialistas.

**Aceitação integral:** aberta. Aceitar operação/caller por serviço, com motor indisponível e sem permissões herdadas.

**Incremento942/C2B1A3:** IO nas fronteiras existentes de Rede, Som e DMS;15dirigidos/2967full21ignorados/build555/native208 com16recusas tipadas. [Checkpoint](../.build/pet-goal-execution-next-r1/SYSTEM-IO-PARTIAL-CHECKPOINT-R43F10B2C2B1A3.json). C2B2/C3 e aceitação integral continuam abertos.

- [pet_system.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_system.rs) — `DmsUse`, `dms`

**Incremento945/C2B2A2:** worker mantém Hold até recolha do grupo; controlo de Som→readback e DMS tipado no Tauri com peers privados.13dirigidos/2980full21ignorados/build558/native210. [Checkpoint](../.build/pet-goal-execution-next-r1/SYSTEM-HELPER-PARTIAL-CHECKPOINT-R43F10B2C2B2A2.json). C3/lifecycle/aceitação integral permanecem.

- [pet_helper.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_helper.rs) — `Lane`, `Session`, `supervise`
- [pet_process_group.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_process_group.rs) — `OwnedChild`, `finish_ready`, `group_has_live_members`

### RUN-01 — Schema e repositório de objetivos, tarefas, planos, passos e eventos; migração do estado atual

**Dependências e critérios preservados:** PET-01/02. Separar atividade de estado durável; guardar admissão antes de efeitos, preservar IDs e trabalhos ativos além dos dez do journal público

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/pet_runtime/model.rs](../windows/src-tauri/src/pet_runtime/model.rs) — `Record`, `Owner`, `validate`
- [windows/src-tauri/src/pet_runtime/store.rs](../windows/src-tauri/src/pet_runtime/store.rs) — `Store`, `read`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/model.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/model.rs) — `Record`, `Owner`, `validate`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Relatos/checkpoints por composição; as provas parciais não cobrem todos os critérios deste ID.

**Limite / pendência:** Migração/aceitação integral e dados de regras/delegações não concluídos.

**Aceitação integral:** aberta. Conservar IDs/estado ativo/recibos e impedir execução por leitura/migração.

### RUN-02 — Admissão, fila, leases e associação da conversa de background

**Dependências e critérios preservados:** RUN-01. Não trocar seleção/rascunhos da UI; uma sessão não recebe dois controladores. Prazos incluem fila, slots físicos aguardam fim do worker, outros pets continuam. Prontidão fresca antes do efeito; validade durante IO não depende apenas da idade da cache e continua vinculada a política/posse/contexto/prazo

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/pet_runtime/leases.rs](../windows/src-tauri/src/pet_runtime/leases.rs) — `Lease`, `basic`, `valid`
- [windows/src-tauri/src/pet_runtime/lease_controller.rs](../windows/src-tauri/src/pet_runtime/lease_controller.rs) — `Validity`, `Validation`, `eq`
- [windows/src-tauri/src/pet_resources.rs](../windows/src-tauri/src/pet_resources.rs) — `Failure`, `Coordinator`, `Hold`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/admission_confirmation.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/admission_confirmation.rs) — `transient`, `temporary`, `authority_blocked`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/writer.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/writer.rs) — `model_once`, `authorize_body`, `record_result`, `before_send`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_memory/context_model.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_memory/context_model.rs) — `Selected`, `Selection`, `valid`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_capabilities/mod.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_capabilities/mod.rs) — `Revision`, `Registry`, `new`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/runtime-goals.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/runtime-goals.ts) — `RuntimeGoalActions`, `musicGoalPlan`, `templateCriteria`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/memory-controls.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/memory-controls.ts) — `buildMemoryControls`, `tell`, `resetEditor`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/memory-review.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/memory-review.ts) — `memoryRecord`, `goalMemoryReview`, `manualMemoryReview`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/runtime-goals.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/runtime-goals.ts) — `buildRuntimeGoals`, `syncMusic`, `syncPets`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_memory/context.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_memory/context.rs) — `Capture`, `catalog_path`, `at`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_memory/manual.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_memory/manual.rs) — `Resolved`, `reviews`, `failure`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/bridge.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/bridge.ts) — `call`, `runtimeBudget`, `runtimeDraftAction`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/session-chat.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/session-chat.ts) — `nativeSessionReady`, `constrainNative`, `SessionChat`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/desktop/pet-chat.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/desktop/pet-chat.ts) — `acceptRect`, `fromLocal`, `report`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/chat.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/chat.ts) — `watchCapabilities`, `capabilityText`, `bubble`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/manual-memory.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/manual-memory.ts) — `buildManualMemory`, `finish`, `discard`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/completion_commit.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/completion_commit.rs) — `complete_retained`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/lease_controller.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/lease_controller.rs) — `Validity`, `Validation`, `eq`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/store.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/store.rs) — `Store`, `read`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/results.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/results.rs) — `preflight`, `lookup`, `publish`, `recover`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/result_sink_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/result_sink_tests.rs) — `bytes`, `visit`, `incompatible`

**Prova disponível e alcance:**

- [SCOPED-MEMORY-INTEGRATED-CHECKPOINT-R42.json](../.build/pet-goal-execution-next-r1/SCOPED-MEMORY-INTEGRATED-CHECKPOINT-R42.json) — Gestão humana834 e bases integradas; não envio autónomo.
- [API-MEMORY-WIRE-PARTIAL-CHECKPOINT-R43F2D.json](../.build/pet-goal-execution-next-r1/API-MEMORY-WIRE-PARTIAL-CHECKPOINT-R43F2D.json) — SDK846/R43F2D:2632 Rust/21 ignorados/check/build,5 casos API e Tauri136 com13 HTTP/17 prompts CLI sintéticos; concessão/corpo real/revogação/retoma nativa e destinos, sem interface humana/adoção/entrega.
- [HUMAN-GOAL-MEMORY-PARTIAL-CHECKPOINT-R43F2E1.json](../.build/pet-goal-execution-next-r1/HUMAN-GOAL-MEMORY-PARTIAL-CHECKPOINT-R43F2E1.json) — SDK850:2634 Rust/21 ignorados (487 inputs nativos idênticos A/B),1200 JS/TSC/Vite/check/build e Tauri139. Controlos reais de catálogo/seleção/revisão/consentimento e resposta Ollama/LMStudio, revogação antes da confirmação e leitor antigo. Escolhas automatizadas e motores sintéticos, não aceitação humana/instalada ou revisão manual de chat.
- [MANUAL-MEMORY-PARTIAL-CHECKPOINT-R43F2E2.json](../.build/pet-goal-execution-next-r1/MANUAL-MEMORY-PARTIAL-CHECKPOINT-R43F2E2.json) — SDK854/E2b:2638 Rust/21 ignorados (488 inputs nativos idênticos A/B),1212 JS/TSC/Vite/check/build. Tauri143 janela própria local2/CLI3/Notion com30 hit-tests, contexto vinculado/reinício (3HTTP/4CLI);144 preservação13HTTP/17CLI. Escolhas/protocolos sintéticos; inline nativo falhou ativação em140 e permanece pendente. Sem aceitação humana/instalada.
- [MEMORY-MIGRATION-PARTIAL-CHECKPOINT-R43F4.json](../.build/pet-goal-execution-next-r1/MEMORY-MIGRATION-PARTIAL-CHECKPOINT-R43F4.json) — SDK854/F4:3 fontes nativas alteradas; baseline antes da correção1passa/2falham; depois2641 Rust/21 ignorados,check/build e Tauri147. Native146/147 prova retoma selecionada,2rondas com2aprovações,7HTTP sintéticos,revogação/reinício e recusa do leitor844. Não certifica backup completo, integração/instalação ou aceitação humana. Timeout145 preservado com causa desconhecida.
- [CURRENT-PROFILE-BACKUP-PARTIAL-CHECKPOINT-R43F5.json](../.build/pet-goal-execution-next-r1/CURRENT-PROFILE-BACKUP-PARTIAL-CHECKPOINT-R43F5.json) — SDK858/F5D:2655Rust/21ignorados,23dirigidos/1ignorado,check-tests/build e composição151.149 prova CLI81ficheiros/integridade/owner/consentimento/recusas;151 arranque normal/continuação7HTTP sintéticos e backup do mesmo perfil.150 Busy anterior retido;D repete só controlo anterior à escrita, nunca IO físico/modelo, com Stop e limite. Sem restauro/instalação/aceitação humana ou quota real.
- [MODEL-RESULT-SINK-PREFLIGHT-PARTIAL-CHECKPOINT-R43F9B2D4C.json](../.build/pet-goal-execution-next-r1/MODEL-RESULT-SINK-PREFLIGHT-PARTIAL-CHECKPOINT-R43F9B2D4C.json) — 910/C2:18dirigidos/2841Rust21ignorados/build531/native191; preflight sem envio/reserva, alteração durante metadata e falha posterior com reconciliação. Providers sintéticos,0quota/modelos reais, sem entrega/aceitação humana.

**Limite / pendência:** Admissão/continuação, conclusão sob concorrência e backup provados em privado; lifecycle integral, restauro/adoção e aceitação instalada pendentes.

**Aceitação integral:** aberta. Recuperação explícita de geração incerta/seed inválido/pre-manifest/staging; migrações/revalidações/B3/adoção e aceitação integral permanecem.

**Incremento939/C1:** origem atual de IO e preservação de Stop/recusa/evidência;14fixtures/2931full21ignorados/1272web/build552/native205 sintético. [Checkpoint](../.build/pet-goal-execution-next-r1/SETTINGS-EFFECT-GUARD-PARTIAL-CHECKPOINT-R43F10B2C1.json). Guardas finais e reconciliação durável pendentes, sem entrega integral.

**Incremento940/C2A3:** origem atual na escrita nativa final, controlo/recusa tipados e pre-spawn;21dirigidos/2952full21ignorados/build553/native206 com corte real de4096bytes num peer sintético. [Checkpoint](../.build/pet-goal-execution-next-r1/FINAL-NATIVE-DISPATCH-PARTIAL-CHECKPOINT-R43F10B2C2A3.json). C2B/C3 e aceitação integral continuam abertos.

- [pet_native_write.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_native_write.rs) — `write_reason`, `frame`

### RUN-03 — Ciclo comum de planeamento, progresso, orçamento e conclusão

**Dependências e critérios preservados:** RUN-01/02 + EXE-01. Critérios verificáveis, limites de ciclos/tentativas/recursos, falta de progresso detetada; texto do modelo não basta para concluir

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/pet_runtime/planning.rs](../windows/src-tauri/src/pet_runtime/planning.rs) — `Limits`, `Catalog`, `Policy`
- [windows/src-tauri/src/pet_runtime/criteria.rs](../windows/src-tauri/src/pet_runtime/criteria.rs) — `Playback`, `Predicate`, `valid`
- [windows/src-tauri/src/pet_runtime/writer.rs](../windows/src-tauri/src/pet_runtime/writer.rs) — `model_once`, `record_result`, `before_send`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/completion_commit.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/completion_commit.rs) — `complete_retained`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/lease_controller.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/lease_controller.rs) — `Validity`, `Validation`, `eq`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/store.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/store.rs) — `Store`, `read`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/writer.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/writer.rs) — `model_once`, `authorize_body`, `record_result`, `before_send`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/results.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/results.rs) — `preflight`, `lookup`, `publish`, `recover`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/result_sink_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/result_sink_tests.rs) — `bytes`, `visit`, `incompatible`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Relatos/checkpoints por composição; as provas parciais não cobrem todos os critérios deste ID.
- [CURRENT-PROFILE-BACKUP-PARTIAL-CHECKPOINT-R43F5.json](../.build/pet-goal-execution-next-r1/CURRENT-PROFILE-BACKUP-PARTIAL-CHECKPOINT-R43F5.json) — SDK858/F5D:2655Rust/21ignorados,23dirigidos/1ignorado,check-tests/build e composição151.149 prova CLI81ficheiros/integridade/owner/consentimento/recusas;151 arranque normal/continuação7HTTP sintéticos e backup do mesmo perfil.150 Busy anterior retido;D repete só controlo anterior à escrita, nunca IO físico/modelo, com Stop e limite. Sem restauro/instalação/aceitação humana ou quota real.
- [MODEL-RESULT-SINK-PREFLIGHT-PARTIAL-CHECKPOINT-R43F9B2D4C.json](../.build/pet-goal-execution-next-r1/MODEL-RESULT-SINK-PREFLIGHT-PARTIAL-CHECKPOINT-R43F9B2D4C.json) — 910/C2:18dirigidos/2841Rust21ignorados/build531/native191; preflight sem envio/reserva, alteração durante metadata e falha posterior com reconciliação. Providers sintéticos,0quota/modelos reais, sem entrega/aceitação humana.

**Limite / pendência:** Native150 mostrou Busy concreto;D tem testes reais de locks/Stop/limite e composição151. Execução geral, CLI/autonomia e critérios de todos os adaptadores ainda abertos.

**Aceitação integral:** aberta. Recuperação explícita de geração incerta/seed inválido/pre-manifest/staging; migrações/revalidações/B3/adoção e aceitação integral permanecem.

### RUN-04 — Recuperação, pausa, Stop e retoma após falha

**Dependências e critérios preservados:** RUN-01/02. Reinício não reproduz efeitos; reconciliar processos/sessões/recibos, preservar incerteza e permitir retoma vinculada ao objetivo original

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/pet_runtime/continuation.rs](../windows/src-tauri/src/pet_runtime/continuation.rs) — `Reference`, `source_matches`
- [windows/src-tauri/src/pet_runtime/recovery.rs](../windows/src-tauri/src/pet_runtime/recovery.rs) — `Reference`, `valid`, `source_matches`
- [windows/src-tauri/src/pet_runtime/results.rs](../windows/src-tauri/src/pet_runtime/results.rs) — `lookup`, `publish`, `recover`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/completion_commit.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/completion_commit.rs) — `complete_retained`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/lease_controller.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/lease_controller.rs) — `Validity`, `Validation`, `eq`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/store.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/store.rs) — `Store`, `read`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/writer.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/writer.rs) — `model_once`, `authorize_body`, `record_result`, `before_send`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/results.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/results.rs) — `preflight`, `lookup`, `publish`, `recover`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/result_sink_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/result_sink_tests.rs) — `bytes`, `visit`, `incompatible`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_store.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_store.rs) — `KeyFile`, `parse`, `restoration_base`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_independent.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_independent.rs) — `IndependentKeyState`, `key_state`, `prepare`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent.rs) — `preparation_state`, `preview`, `apply_using`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent_tests.rs) — `Fixture`, `tree`, `visit`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_retention.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_retention.rs) — `RetentionReview`, `scan`, `current`, `apply_using`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_retention_inventory.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_retention_inventory.rs) — `Opaque`, `Budget`, `inspect`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_retention_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_retention_tests.rs) — `review`, `job`, `opaque`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/staging_fs.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/staging_fs.rs) — `transfer_dir`, `publish_dir`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restore_cli.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restore_cli.rs) — `activation_sealed`, `execute`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent_operational.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent_operational.rs) — `Custody`, `validate`, `PreparedAuthority`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_configuration.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_configuration.rs) — `configuration`, `epoch_scope`, `verify_payload`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_retained_independent_transaction_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_retained_independent_transaction_tests.rs) — `data`, `retain`, `prepare`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_history.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_history.rs) — `goal_snapshot`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/history.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/history.rs) — `RestoredSource`, `Snapshot`, `empty`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_history.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_history.rs) — `phase`, `history_inventory`, `history`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/runtime-goals.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/runtime-goals.ts) — `buildRuntimeGoals`, `syncMusic`, `syncPets`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/restored_future.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/restored_future.rs) — `Review`, `Origin`, `valid_for`, `request_id`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restored_future.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restored_future.rs) — `Witness`, `capture`, `current`, `verify_restored_origin`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/store_restored_future.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/store_restored_future.rs) — `existing_restored_future`, `create_restored_future_clock`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/restored_future_api.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/restored_future_api.rs) — `pet_runtime_prepare_restored_future`, `pet_runtime_confirm_restored_future`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/restored-future-review.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/restored-future-review.ts) — `buildRestoredFutureReview`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Relatos/checkpoints por composição; as provas parciais não cobrem todos os critérios deste ID.
- [MEMORY-MIGRATION-PARTIAL-CHECKPOINT-R43F4.json](../.build/pet-goal-execution-next-r1/MEMORY-MIGRATION-PARTIAL-CHECKPOINT-R43F4.json) — SDK854/F4:3 fontes nativas alteradas; baseline antes da correção1passa/2falham; depois2641 Rust/21 ignorados,check/build e Tauri147. Native146/147 prova retoma selecionada,2rondas com2aprovações,7HTTP sintéticos,revogação/reinício e recusa do leitor844. Não certifica backup completo, integração/instalação ou aceitação humana. Timeout145 preservado com causa desconhecida.
- [CURRENT-PROFILE-BACKUP-PARTIAL-CHECKPOINT-R43F5.json](../.build/pet-goal-execution-next-r1/CURRENT-PROFILE-BACKUP-PARTIAL-CHECKPOINT-R43F5.json) — SDK858/F5D:2655Rust/21ignorados,23dirigidos/1ignorado,check-tests/build e composição151.149 prova CLI81ficheiros/integridade/owner/consentimento/recusas;151 arranque normal/continuação7HTTP sintéticos e backup do mesmo perfil.150 Busy anterior retido;D repete só controlo anterior à escrita, nunca IO físico/modelo, com Stop e limite. Sem restauro/instalação/aceitação humana ou quota real.
- [MODEL-RESULT-SINK-PREFLIGHT-PARTIAL-CHECKPOINT-R43F9B2D4C.json](../.build/pet-goal-execution-next-r1/MODEL-RESULT-SINK-PREFLIGHT-PARTIAL-CHECKPOINT-R43F9B2D4C.json) — 910/C2:18dirigidos/2841Rust21ignorados/build531/native191; preflight sem envio/reserva, alteração durante metadata e falha posterior com reconciliação. Providers sintéticos,0quota/modelos reais, sem entrega/aceitação humana.
- [INDEPENDENT-RECOVERY-REVIEW-PARTIAL-CHECKPOINT-R43F9B2D4D1.json](../.build/pet-goal-execution-next-r1/INDEPENDENT-RECOVERY-REVIEW-PARTIAL-CHECKPOINT-R43F9B2D4D1.json) — 910/D4D1a:11dirigidos/2844Rust21ignorados/build531/native192,0modelos/quota/CLIagents. Parser comum/keyInvalid/manual;8casos semescrita,retomamesmachave,missing/readyrecusados. Não resolve retenção/quarentena/abandono,adoção/entrega.
- [INDEPENDENT-RETENTION-PARTIAL-CHECKPOINT-R43F9B2D4D2A.json](../.build/pet-goal-execution-next-r1/INDEPENDENT-RETENTION-PARTIAL-CHECKPOINT-R43F9B2D4D2A.json) — 913/D4D2A2:11dirigidos/2855Rust21ignorados/build534/native193,0modelos/quota/CLIagents/efeitos. Retenção nativa/sameinode/bytes/cadeia/retomaRecibo,blockedP1. Não é recuperação operacional/backup/adoção/entrega; D4D2B e limites exatos conjuntos pendentes.
- [RETAINED-ADMISSION-PARTIAL-CHECKPOINT-R43F9B2D4D2B.json](../.build/pet-goal-execution-next-r1/RETAINED-ADMISSION-PARTIAL-CHECKPOINT-R43F9B2D4D2B.json) — 914/D4D2B1:7dirigidos/2864Rust21ignorados/build535/native195;2HTTP1CLI sintéticos/0quota. Nova preparação independente,receipt4/Custody2/Admission4/OpenedACK/backup2,época seguinte fixture. Não é adoção/entrega/aceitação humana;49integrais abertos.
- [RESTORED-HISTORY-PARTIAL-CHECKPOINT-R43F9B3A.json](../.build/pet-goal-execution-next-r1/RESTORED-HISTORY-PARTIAL-CHECKPOINT-R43F9B3A.json) — 915/B3A2: prova negativa,2870Rust21ignorados/1256web/TSC/Vite/build536/native196,2HTTP1CLI sintéticos/0quota. História2/origemexata/totaldesconhecido/read-only antesIO,novoGoalhistória1. B3F2/adoção/entrega/humana/49integrais abertos.
- [RESTORED-FUTURE-PARTIAL-CHECKPOINT-R43F9B3F2A7.json](../.build/pet-goal-execution-next-r1/RESTORED-FUTURE-PARTIAL-CHECKPOINT-R43F9B3F2A7.json) — 923/A7:32dirigidos/2886Rust21ignorados/1262web/build542/native199; origem13 só dados + runtime comum/consentimento/Stop/reinício/backup; UI automatizada e2HTTP1CLI sintéticos/0quota,leitor915 real preserva13. Época seguinte fixture;49integrais/adoção/entrega abertos.

**Limite / pendência:** Resta lifecycle/entrega e cobertura de todos os efeitos, Guardian e mesh.145 permanece sem causa exata;150 confirmou Busy e foi corrigido emD.

**Aceitação integral:** aberta. Fronteiras conjuntas4096/64MiB, migrações/revalidações/adoção local de fontes exatas; lifecycle e aceitação integral/provider antes de TEAM/Guardian e entrega.

**Incremento939/C1:** origem atual de IO e preservação de Stop/recusa/evidência;14fixtures/2931full21ignorados/1272web/build552/native205 sintético. [Checkpoint](../.build/pet-goal-execution-next-r1/SETTINGS-EFFECT-GUARD-PARTIAL-CHECKPOINT-R43F10B2C1.json). Guardas finais e reconciliação durável pendentes, sem entrega integral.

**Incremento940/C2A3:** origem atual na escrita nativa final, controlo/recusa tipados e pre-spawn;21dirigidos/2952full21ignorados/build553/native206 com corte real de4096bytes num peer sintético. [Checkpoint](../.build/pet-goal-execution-next-r1/FINAL-NATIVE-DISPATCH-PARTIAL-CHECKPOINT-R43F10B2C2A3.json). C2B/C3 e aceitação integral continuam abertos.

- [local_tasks/executor.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/local_tasks/executor.rs) — `run`, `linux`
- [session_chat/rpc.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/session_chat/rpc.rs) — `write_native_guarded`, `respond_native`

**Incremento945/C2B2A2:** worker mantém Hold até recolha do grupo; controlo de Som→readback e DMS tipado no Tauri com peers privados.13dirigidos/2980full21ignorados/build558/native210. [Checkpoint](../.build/pet-goal-execution-next-r1/SYSTEM-HELPER-PARTIAL-CHECKPOINT-R43F10B2C2B2A2.json). C3/lifecycle/aceitação integral permanecem.

- [pet_helper.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_helper.rs) — `Lane`, `Session`, `supervise`
- [pet_process_group.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_process_group.rs) — `OwnedChild`, `finish_ready`, `group_has_live_members`
- [local_tasks/executor.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/local_tasks/executor.rs) — `run`, `linux`

**Incremento948/C3A4:** intenção/resultado/revisão de preferências duráveis e formato de perfil/backup/restauro; 13dirigidos/2993full21ignorados/build561/native212. [Checkpoint](../.build/pet-goal-execution-next-r1/SETTINGS-DURABLE-PARTIAL-CHECKPOINT-R43F10B2C3A4.json). Reconciliação humana e listener/chaves ainda pendentes; sem adoção/instalação.

- [settings_durable.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings_durable.rs) — `Record`, `Observation`, `Prepared`, `restored_record`
- [settings_revision.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings_revision.rs) — `Bound`, `save_with`, `note_failed`
- [preferences_file.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/preferences_file.rs) — `commit_with`

### RUN-05 — Prioridades e justiça entre trabalho manual e rotinas

**Dependências e critérios preservados:** RUN-02 + OBS-01. Stop/decisões responsivos, nenhuma rotina monopoliza provider/modelo/dispositivo; limites configurados e observáveis

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/pet_resources.rs](../windows/src-tauri/src/pet_resources.rs) — `Failure`, `Coordinator`, `Hold`
- [windows/src-tauri/src/pet_runtime/resource_wait.rs](../windows/src-tauri/src/pet_runtime/resource_wait.rs) — `Entry`, `valid`, `same_wait`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Relatos/checkpoints por composição; as provas parciais não cobrem todos os critérios deste ID.

**Limite / pendência:** Justiça entre rotinas/delegações ainda sem aceitação integral.

**Aceitação integral:** aberta. Prioridade de Stop/decisões e backpressure sem monopolização ou saldo duplicado.

### EXE-01 — Registo/broker de operações tipadas para ferramentas nativas, locais, MCP e desktop

**Dependências e critérios preservados:** PET-01/02. Schema, versão, efeitos, alvos, limites, cancelamento, idempotência e verificação; ferramenta descoberta não fica automaticamente autorizada

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/cli_decisions.rs](../windows/src-tauri/src/cli_decisions.rs) — `target`, `live`
- [windows/src-tauri/src/pet_runtime/recipe_decisions.rs](../windows/src-tauri/src/pet_runtime/recipe_decisions.rs) — `Target`, `Origin`, `valid`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Relatos/checkpoints por composição; as provas parciais não cobrem todos os critérios deste ID.

**Limite / pendência:** Catálogo geral de ferramentas/Sistema e executores adicionais incompletos.

**Aceitação integral:** aberta. Uma ferramenta descoberta não fica autorizada; alvo/argumentos/versão e efeito exatos.

### EXE-02 — Concessões por objetivo/regra e receitas limitadas; integrar aprovações existentes

**Dependências e critérios preservados:** RUN-01 + EXE-01. Uma aprovação não se torna shell genérica; política revogada invalida pedidos pendentes, sem conceder privilégios de outro pet. Chat/voz acompanham a política e os pedidos do agente original, com consumo único, sem duplicar pedidos já satisfeitos nem alargar a decisão nativa

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/cli_decisions.rs](../windows/src-tauri/src/cli_decisions.rs) — `target`, `live`
- [windows/src-tauri/src/pet_runtime/admission.rs](../windows/src-tauri/src/pet_runtime/admission.rs) — `Denial`, `Payment`, `Budget`
- [windows/src-tauri/src/pet_runtime/authority.rs](../windows/src-tauri/src/pet_runtime/authority.rs) — `Failure`, `control_error`, `Operation`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/admission.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/admission.rs) — `Denial`, `Payment`, `Budget`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_capabilities/mod.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_capabilities/mod.rs) — `Revision`, `Registry`, `new`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/runtime-goals.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/runtime-goals.ts) — `RuntimeGoalActions`, `musicGoalPlan`, `templateCriteria`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/memory-controls.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/memory-controls.ts) — `buildMemoryControls`, `tell`, `resetEditor`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/memory-review.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/memory-review.ts) — `memoryRecord`, `goalMemoryReview`, `manualMemoryReview`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/runtime-goals.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/runtime-goals.ts) — `buildRuntimeGoals`, `syncMusic`, `syncPets`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_memory/context.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_memory/context.rs) — `Capture`, `catalog_path`, `at`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_memory/manual.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_memory/manual.rs) — `Resolved`, `reviews`, `failure`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/bridge.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/bridge.ts) — `call`, `runtimeBudget`, `runtimeDraftAction`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/session-chat.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/session-chat.ts) — `nativeSessionReady`, `constrainNative`, `SessionChat`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/desktop/pet-chat.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/desktop/pet-chat.ts) — `acceptRect`, `fromLocal`, `report`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/chat.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/chat.ts) — `watchCapabilities`, `capabilityText`, `bubble`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/manual-memory.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/manual-memory.ts) — `buildManualMemory`, `finish`, `discard`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_store.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_store.rs) — `KeyFile`, `parse`, `restoration_base`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_independent.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_independent.rs) — `IndependentKeyState`, `key_state`, `prepare`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent.rs) — `preparation_state`, `preview`, `apply_using`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent_tests.rs) — `Fixture`, `tree`, `visit`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_retention.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_retention.rs) — `RetentionReview`, `scan`, `current`, `apply_using`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_retention_inventory.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_retention_inventory.rs) — `Opaque`, `Budget`, `inspect`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_retention_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_retention_tests.rs) — `review`, `job`, `opaque`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/staging_fs.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/staging_fs.rs) — `transfer_dir`, `publish_dir`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restore_cli.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restore_cli.rs) — `activation_sealed`, `execute`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent_operational.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent_operational.rs) — `Custody`, `validate`, `PreparedAuthority`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_configuration.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_configuration.rs) — `configuration`, `epoch_scope`, `verify_payload`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_retained_independent_transaction_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_retained_independent_transaction_tests.rs) — `data`, `retain`, `prepare`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_history.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_history.rs) — `goal_snapshot`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/history.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/history.rs) — `RestoredSource`, `Snapshot`, `empty`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_history.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_history.rs) — `phase`, `history_inventory`, `history`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/restored_future.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/restored_future.rs) — `Review`, `Origin`, `valid_for`, `request_id`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restored_future.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restored_future.rs) — `Witness`, `capture`, `current`, `verify_restored_origin`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/store_restored_future.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/store_restored_future.rs) — `existing_restored_future`, `create_restored_future_clock`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/restored_future_api.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/restored_future_api.rs) — `pet_runtime_prepare_restored_future`, `pet_runtime_confirm_restored_future`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/restored-future-review.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/restored-future-review.ts) — `buildRestoredFutureReview`

**Prova disponível e alcance:**

- [SCOPED-MEMORY-INTEGRATED-CHECKPOINT-R42.json](../.build/pet-goal-execution-next-r1/SCOPED-MEMORY-INTEGRATED-CHECKPOINT-R42.json) — Gestão humana834 e bases integradas; não envio autónomo.
- [API-MEMORY-WIRE-PARTIAL-CHECKPOINT-R43F2D.json](../.build/pet-goal-execution-next-r1/API-MEMORY-WIRE-PARTIAL-CHECKPOINT-R43F2D.json) — SDK846/R43F2D:2632 Rust/21 ignorados/check/build,5 casos API e Tauri136 com13 HTTP/17 prompts CLI sintéticos; concessão/corpo real/revogação/retoma nativa e destinos, sem interface humana/adoção/entrega.
- [HUMAN-GOAL-MEMORY-PARTIAL-CHECKPOINT-R43F2E1.json](../.build/pet-goal-execution-next-r1/HUMAN-GOAL-MEMORY-PARTIAL-CHECKPOINT-R43F2E1.json) — SDK850:2634 Rust/21 ignorados (487 inputs nativos idênticos A/B),1200 JS/TSC/Vite/check/build e Tauri139. Controlos reais de catálogo/seleção/revisão/consentimento e resposta Ollama/LMStudio, revogação antes da confirmação e leitor antigo. Escolhas automatizadas e motores sintéticos, não aceitação humana/instalada ou revisão manual de chat.
- [MANUAL-MEMORY-PARTIAL-CHECKPOINT-R43F2E2.json](../.build/pet-goal-execution-next-r1/MANUAL-MEMORY-PARTIAL-CHECKPOINT-R43F2E2.json) — SDK854/E2b:2638 Rust/21 ignorados (488 inputs nativos idênticos A/B),1212 JS/TSC/Vite/check/build. Tauri143 janela própria local2/CLI3/Notion com30 hit-tests, contexto vinculado/reinício (3HTTP/4CLI);144 preservação13HTTP/17CLI. Escolhas/protocolos sintéticos; inline nativo falhou ativação em140 e permanece pendente. Sem aceitação humana/instalada.
- [INDEPENDENT-RECOVERY-REVIEW-PARTIAL-CHECKPOINT-R43F9B2D4D1.json](../.build/pet-goal-execution-next-r1/INDEPENDENT-RECOVERY-REVIEW-PARTIAL-CHECKPOINT-R43F9B2D4D1.json) — 910/D4D1a:11dirigidos/2844Rust21ignorados/build531/native192,0modelos/quota/CLIagents. Parser comum/keyInvalid/manual;8casos semescrita,retomamesmachave,missing/readyrecusados. Não resolve retenção/quarentena/abandono,adoção/entrega.
- [INDEPENDENT-RETENTION-PARTIAL-CHECKPOINT-R43F9B2D4D2A.json](../.build/pet-goal-execution-next-r1/INDEPENDENT-RETENTION-PARTIAL-CHECKPOINT-R43F9B2D4D2A.json) — 913/D4D2A2:11dirigidos/2855Rust21ignorados/build534/native193,0modelos/quota/CLIagents/efeitos. Retenção nativa/sameinode/bytes/cadeia/retomaRecibo,blockedP1. Não é recuperação operacional/backup/adoção/entrega; D4D2B e limites exatos conjuntos pendentes.
- [RETAINED-ADMISSION-PARTIAL-CHECKPOINT-R43F9B2D4D2B.json](../.build/pet-goal-execution-next-r1/RETAINED-ADMISSION-PARTIAL-CHECKPOINT-R43F9B2D4D2B.json) — 914/D4D2B1:7dirigidos/2864Rust21ignorados/build535/native195;2HTTP1CLI sintéticos/0quota. Nova preparação independente,receipt4/Custody2/Admission4/OpenedACK/backup2,época seguinte fixture. Não é adoção/entrega/aceitação humana;49integrais abertos.
- [RESTORED-HISTORY-PARTIAL-CHECKPOINT-R43F9B3A.json](../.build/pet-goal-execution-next-r1/RESTORED-HISTORY-PARTIAL-CHECKPOINT-R43F9B3A.json) — 915/B3A2: prova negativa,2870Rust21ignorados/1256web/TSC/Vite/build536/native196,2HTTP1CLI sintéticos/0quota. História2/origemexata/totaldesconhecido/read-only antesIO,novoGoalhistória1. B3F2/adoção/entrega/humana/49integrais abertos.
- [RESTORED-FUTURE-PARTIAL-CHECKPOINT-R43F9B3F2A7.json](../.build/pet-goal-execution-next-r1/RESTORED-FUTURE-PARTIAL-CHECKPOINT-R43F9B3F2A7.json) — 923/A7:32dirigidos/2886Rust21ignorados/1262web/build542/native199; origem13 só dados + runtime comum/consentimento/Stop/reinício/backup; UI automatizada e2HTTP1CLI sintéticos/0quota,leitor915 real preserva13. Época seguinte fixture;49integrais/adoção/entrega abertos.

**Limite / pendência:** Controlos Tauri e scopes manuais/objetivo distintos com escolhas automatizadas; inline nativo, adoção/aceitação humana e restantes caminhos ainda abertos.

**Aceitação integral:** aberta. Fronteiras conjuntas4096/64MiB, migrações/revalidações/adoção local de fontes exatas; lifecycle e aceitação integral/provider antes de TEAM/Guardian e entrega.

### EXE-03 — Recibos e verificação de pós-condições

**Dependências e critérios preservados:** EXE-01. Separar ACK/saída/efeito observado; erro após possível efeito exige reconciliação, sem repetir automaticamente

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/pet_runtime/steps.rs](../windows/src-tauri/src/pet_runtime/steps.rs) — `Scope`, `slot`, `valid`
- [windows/src-tauri/src/pet_runtime/recipes.rs](../windows/src-tauri/src/pet_runtime/recipes.rs) — `Scope`, `slot`, `valid`
- [windows/src-tauri/src/pet_runtime/results.rs](../windows/src-tauri/src/pet_runtime/results.rs) — `lookup`, `publish`, `recover`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/writer.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/writer.rs) — `model_once`, `authorize_body`, `record_result`, `before_send`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/results.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/results.rs) — `preflight`, `lookup`, `publish`, `recover`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/result_sink_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/result_sink_tests.rs) — `bytes`, `visit`, `incompatible`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Relatos/checkpoints por composição; as provas parciais não cobrem todos os critérios deste ID.
- [MODEL-RESULT-SINK-PREFLIGHT-PARTIAL-CHECKPOINT-R43F9B2D4C.json](../.build/pet-goal-execution-next-r1/MODEL-RESULT-SINK-PREFLIGHT-PARTIAL-CHECKPOINT-R43F9B2D4C.json) — 910/C2:18dirigidos/2841Rust21ignorados/build531/native191; preflight sem envio/reserva, alteração durante metadata e falha posterior com reconciliação. Providers sintéticos,0quota/modelos reais, sem entrega/aceitação humana.

**Limite / pendência:** Outros efeitos/serviços e estação operacional não universalmente aceites.

**Aceitação integral:** aberta. Recuperação explícita de geração incerta/seed inválido/pre-manifest/staging; migrações/revalidações/B3/adoção e aceitação integral permanecem.

### EXE-04 — Uso do computador e capacidades de observação

**Dependências e critérios preservados:** EXE-01/02. CLI/API/acessibilidade primeiro; alvos exatos, observação antes/depois em controlo visual. Sessão bloqueada ou falta de imagem tem limitação explícita

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/cli_chat.rs](../windows/src-tauri/src/cli_chat.rs) — `run_inner`, `backend`
- [windows/src-tauri/src/pet_config.rs](../windows/src-tauri/src/pet_config.rs) — `select`, `retired`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Relatos/checkpoints por composição; as provas parciais não cobrem todos os critérios deste ID.

**Limite / pendência:** A existência da ferramenta não certifica automação visual ou sessão externa.

**Aceitação integral:** aberta. Alvo exato, observação antes/depois, sessão bloqueada e falta de suporte explícitas.

### CTX-01 — Fechar anexos múltiplos, retenção e recuperação do protótipo de 633 fontes

**Dependências e critérios preservados:** Bloqueador REL-01. Envelope terminal e aceitação nativa de restauro parcial/nome reutilizado/limpeza interrompida/motivos de recusa verificados nos dois incrementos675 privados; completar integração e migração/backup/rollback v4/v5/leitores antigos

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/pet_documents.rs](../windows/src-tauri/src/pet_documents.rs) — `Attachment`, `context_file`, `Ticket`
- [windows/src-tauri/src/pet_documents/purge_terminal.rs](../windows/src-tauri/src/pet_documents/purge_terminal.rs) — `name`, `id`, `staging`
- [windows/src-tauri/src/pet_documents/copy_retention.rs](../windows/src-tauri/src/pet_documents/copy_retention.rs) — `Initial`, `Payload`, `NativeReview`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Relatos/checkpoints por composição; as provas parciais não cobrem todos os critérios deste ID.

**Limite / pendência:** Fecho da limpeza, composição e migração/backup/rollback ainda bloqueiam entrega do conjunto.

**Aceitação integral:** aberta. Fechar janelas de falha e adotar fontes exatas sem perder cópias/referências antigas.

### CTX-02 — Apresentação por mensagem e seleção de anexos para contexto

**Dependências e critérios preservados:** CTX-01. IDs/proveniência/cobertura visíveis onde necessário; não reautorizar cópias nem inferir leitura integral do modelo

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/pet_documents/request.rs](../windows/src-tauri/src/pet_documents/request.rs) — `PendingCheck`, `drop`, `Stamp`
- [windows/src-tauri/src/pet_files.rs](../windows/src-tauri/src/pet_files.rs) — `ingest`, `compose`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Relatos/checkpoints por composição; as provas parciais não cobrem todos os critérios deste ID.

**Limite / pendência:** UI/integração da coleção múltipla e cobertura por mensagem não integralmente entregues.

**Aceitação integral:** aberta. Adicionar cumulativamente e selecionar conjunto imutável sem confundir nome, cópia e leitura.

### MEM-01 — Schema de memória: preferências, contexto de projeto, episódios, derivados e resumos

**Dependências e critérios preservados:** PET-01 + RUN-01. Dono/âmbito/proveniência/validade/revisão; dados derivados não viram instrução nem autorização

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/pet_memory/model.rs](../windows/src-tauri/src/pet_memory/model.rs) — `Owner`, `validate`
- [windows/src-tauri/src/pet_memory/mod.rs](../windows/src-tauri/src/pet_memory/mod.rs) — `allowed`, `message`, `validate_owner`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_capabilities/mod.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_capabilities/mod.rs) — `Revision`, `Registry`, `new`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/runtime-goals.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/runtime-goals.ts) — `RuntimeGoalActions`, `musicGoalPlan`, `templateCriteria`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/memory-controls.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/memory-controls.ts) — `buildMemoryControls`, `tell`, `resetEditor`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/memory-review.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/memory-review.ts) — `memoryRecord`, `goalMemoryReview`, `manualMemoryReview`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/runtime-goals.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/runtime-goals.ts) — `buildRuntimeGoals`, `syncMusic`, `syncPets`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_memory/context.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_memory/context.rs) — `Capture`, `catalog_path`, `at`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_memory/manual.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_memory/manual.rs) — `Resolved`, `reviews`, `failure`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/bridge.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/bridge.ts) — `call`, `runtimeBudget`, `runtimeDraftAction`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/session-chat.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/session-chat.ts) — `nativeSessionReady`, `constrainNative`, `SessionChat`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/desktop/pet-chat.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/desktop/pet-chat.ts) — `acceptRect`, `fromLocal`, `report`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/chat.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/chat.ts) — `watchCapabilities`, `capabilityText`, `bubble`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/manual-memory.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/manual-memory.ts) — `buildManualMemory`, `finish`, `discard`

**Prova disponível e alcance:**

- [SCOPED-MEMORY-INTEGRATED-CHECKPOINT-R42.json](../.build/pet-goal-execution-next-r1/SCOPED-MEMORY-INTEGRATED-CHECKPOINT-R42.json) — Gestão humana834 e bases integradas; não envio autónomo.
- [API-MEMORY-WIRE-PARTIAL-CHECKPOINT-R43F2D.json](../.build/pet-goal-execution-next-r1/API-MEMORY-WIRE-PARTIAL-CHECKPOINT-R43F2D.json) — SDK846/R43F2D:2632 Rust/21 ignorados/check/build,5 casos API e Tauri136 com13 HTTP/17 prompts CLI sintéticos; concessão/corpo real/revogação/retoma nativa e destinos, sem interface humana/adoção/entrega.
- [HUMAN-GOAL-MEMORY-PARTIAL-CHECKPOINT-R43F2E1.json](../.build/pet-goal-execution-next-r1/HUMAN-GOAL-MEMORY-PARTIAL-CHECKPOINT-R43F2E1.json) — SDK850:2634 Rust/21 ignorados (487 inputs nativos idênticos A/B),1200 JS/TSC/Vite/check/build e Tauri139. Controlos reais de catálogo/seleção/revisão/consentimento e resposta Ollama/LMStudio, revogação antes da confirmação e leitor antigo. Escolhas automatizadas e motores sintéticos, não aceitação humana/instalada ou revisão manual de chat.
- [MANUAL-MEMORY-PARTIAL-CHECKPOINT-R43F2E2.json](../.build/pet-goal-execution-next-r1/MANUAL-MEMORY-PARTIAL-CHECKPOINT-R43F2E2.json) — SDK854/E2b:2638 Rust/21 ignorados (488 inputs nativos idênticos A/B),1212 JS/TSC/Vite/check/build. Tauri143 janela própria local2/CLI3/Notion com30 hit-tests, contexto vinculado/reinício (3HTTP/4CLI);144 preservação13HTTP/17CLI. Escolhas/protocolos sintéticos; inline nativo falhou ativação em140 e permanece pendente. Sem aceitação humana/instalada.

**Limite / pendência:** Projetos, episódios, derivados, resumos e partilha explícita por implementar.

**Aceitação integral:** aberta. Concluir R43 e evoluir classes com dependências/invalidação sem memória global.

### MEM-02 — Recuperação seletiva, correção, exclusão, revogação e retenção

**Dependências e critérios preservados:** MEM-01 + CTX-01. Pesquisa estruturada/lexical inicial, isolamento entre pets/projetos e invalidação de derivados/índices; embeddings externos só com concessão própria

**Responsabilidade observada / bases a reutilizar:**

- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_memory/context.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_memory/context.rs) — `Capture`, `catalog_path`, `at`
- [windows/src-tauri/src/private_registry.rs](../windows/src-tauri/src/private_registry.rs) — `load_at`, `save_guarded`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/memory_review.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/memory_review.rs) — `Captured`, `confirm`, `Preview`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/claude.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/claude.rs) — `send`, `send_resolved`, `call_with_timeout`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/openai_compat.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/openai_compat.rs) — `send`, `send_resolved`, `call`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/net.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/net.rs) — `exact_destination_client`, `while_current`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/local_chat.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/local_chat.rs) — `server`, `send_runtime_once`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/local_tasks/protocol.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/local_tasks/protocol.rs) — `round_calendar_captured`, `capabilities`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_capabilities/mod.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_capabilities/mod.rs) — `Revision`, `Registry`, `new`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/runtime-goals.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/runtime-goals.ts) — `RuntimeGoalActions`, `musicGoalPlan`, `templateCriteria`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/memory-controls.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/memory-controls.ts) — `buildMemoryControls`, `tell`, `resetEditor`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/memory-review.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/memory-review.ts) — `memoryRecord`, `goalMemoryReview`, `manualMemoryReview`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/runtime-goals.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/runtime-goals.ts) — `buildRuntimeGoals`, `syncMusic`, `syncPets`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_memory/context.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_memory/context.rs) — `Capture`, `catalog_path`, `at`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_memory/manual.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_memory/manual.rs) — `Resolved`, `reviews`, `failure`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/bridge.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/bridge.ts) — `call`, `runtimeBudget`, `runtimeDraftAction`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/session-chat.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/session-chat.ts) — `nativeSessionReady`, `constrainNative`, `SessionChat`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/desktop/pet-chat.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/desktop/pet-chat.ts) — `acceptRect`, `fromLocal`, `report`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/chat.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/chat.ts) — `watchCapabilities`, `capabilityText`, `bubble`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/manual-memory.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/manual-memory.ts) — `buildManualMemory`, `finish`, `discard`

**Prova disponível e alcance:**

- [SCOPED-MEMORY-INTEGRATED-CHECKPOINT-R42.json](../.build/pet-goal-execution-next-r1/SCOPED-MEMORY-INTEGRATED-CHECKPOINT-R42.json) — Gestão humana834 e bases integradas; não envio autónomo.
- [API-MEMORY-WIRE-PARTIAL-CHECKPOINT-R43F2D.json](../.build/pet-goal-execution-next-r1/API-MEMORY-WIRE-PARTIAL-CHECKPOINT-R43F2D.json) — SDK846/R43F2D:2632 Rust/21 ignorados/check/build,5 casos API e Tauri136 com13 HTTP/17 prompts CLI sintéticos; concessão/corpo real/revogação/retoma nativa e destinos, sem interface humana/adoção/entrega.
- [HUMAN-GOAL-MEMORY-PARTIAL-CHECKPOINT-R43F2E1.json](../.build/pet-goal-execution-next-r1/HUMAN-GOAL-MEMORY-PARTIAL-CHECKPOINT-R43F2E1.json) — SDK850:2634 Rust/21 ignorados (487 inputs nativos idênticos A/B),1200 JS/TSC/Vite/check/build e Tauri139. Controlos reais de catálogo/seleção/revisão/consentimento e resposta Ollama/LMStudio, revogação antes da confirmação e leitor antigo. Escolhas automatizadas e motores sintéticos, não aceitação humana/instalada ou revisão manual de chat.
- [MANUAL-MEMORY-PARTIAL-CHECKPOINT-R43F2E2.json](../.build/pet-goal-execution-next-r1/MANUAL-MEMORY-PARTIAL-CHECKPOINT-R43F2E2.json) — SDK854/E2b:2638 Rust/21 ignorados (488 inputs nativos idênticos A/B),1212 JS/TSC/Vite/check/build. Tauri143 janela própria local2/CLI3/Notion com30 hit-tests, contexto vinculado/reinício (3HTTP/4CLI);144 preservação13HTTP/17CLI. Escolhas/protocolos sintéticos; inline nativo falhou ativação em140 e permanece pendente. Sem aceitação humana/instalada.
- [MEMORY-MIGRATION-PARTIAL-CHECKPOINT-R43F4.json](../.build/pet-goal-execution-next-r1/MEMORY-MIGRATION-PARTIAL-CHECKPOINT-R43F4.json) — SDK854/F4:3 fontes nativas alteradas; baseline antes da correção1passa/2falham; depois2641 Rust/21 ignorados,check/build e Tauri147. Native146/147 prova retoma selecionada,2rondas com2aprovações,7HTTP sintéticos,revogação/reinício e recusa do leitor844. Não certifica backup completo, integração/instalação ou aceitação humana. Timeout145 preservado com causa desconhecida.
- [CURRENT-PROFILE-BACKUP-PARTIAL-CHECKPOINT-R43F5.json](../.build/pet-goal-execution-next-r1/CURRENT-PROFILE-BACKUP-PARTIAL-CHECKPOINT-R43F5.json) — SDK858/F5D:2655Rust/21ignorados,23dirigidos/1ignorado,check-tests/build e composição151.149 prova CLI81ficheiros/integridade/owner/consentimento/recusas;151 arranque normal/continuação7HTTP sintéticos e backup do mesmo perfil.150 Busy anterior retido;D repete só controlo anterior à escrita, nunca IO físico/modelo, com Stop e limite. Sem restauro/instalação/aceitação humana ou quota real.

**Limite / pendência:** Inline/aceitação humana e contas reais, derivados/índices, restauro/adoção pendentes.150 teve Busy de conclusão corrigido emD;145 não tem diagnóstico causal exato.

**Aceitação integral:** aberta. Fechar restauro/migração da composição coerente e preservar revisão/destino/autoridade sem renovação.

### CTX-03 — Completar gestão e aceitação do catálogo ICS já implementado

**Dependências e critérios preservados:** Formulário/consentimento nativos e fonte selecionada; calendário exportado não aparece como sincronização ou capacidade de escrita

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/pet_calendar/mod.rs](../windows/src-tauri/src/pet_calendar/mod.rs) — `Note`, `query`, `fmt`
- [windows/src-tauri/src/pet_sources/mod.rs](../windows/src-tauri/src/pet_sources/mod.rs) — `Consent`, `Document`, `path`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Relatos/checkpoints por composição; as provas parciais não cobrem todos os critérios deste ID.

**Limite / pendência:** Gestão/consentimento e aceitação de todas as superfícies ainda incompletos.

**Aceitação integral:** aberta. Fonte selecionada e cobertura/frescura explícitas; não anunciar sincronização/escrita.

### CTX-04 — Evoluir fontes de pet/CID para objetivos/regras e coleções persistentes

**Dependências e critérios preservados:** RUN-01 + EXE-02 + CTX-03. Migração sem alargar concessões antigas; seleção explícita de vários calendários, revisão e revogação por ocorrência

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/pet_sources/model.rs](../windows/src-tauri/src/pet_sources/model.rs) — `Owner`, `validate`
- [windows/src-tauri/src/pet_sources/mod.rs](../windows/src-tauri/src/pet_sources/mod.rs) — `Consent`, `Document`, `path`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Relatos/checkpoints por composição; as provas parciais não cobrem todos os critérios deste ID.

**Limite / pendência:** Coleções multicalendário e principais de objetivo/regra/ocorrência não concluídos.

**Aceitação integral:** aberta. Migrar sem alargar pet/CID antigo; capturar coleção e revogação por ocorrência.

### CTX-05 — Adaptadores RSS/IMAP com leitura limitada e proveniência

**Dependências e critérios preservados:** EXE-02 + CTX-04. TLS, redirecionamentos/destinos, bytes/prazos e mensagens limitados antes de materializar conteúdo; IMAP sem envio/eliminação implícitos

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/pet_sources/model.rs](../windows/src-tauri/src/pet_sources/model.rs) — `Owner`, `validate`
- [docs/AI-LOCAL-REUSE.md](../docs/AI-LOCAL-REUSE.md) — `Aproveitamento do AI Local nos pets`, `Resultado`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Apenas bases/histórico; não há aceitação integral específica deste requisito.

**Limite / pendência:** RSS/IMAP limitados ainda por implementar/aceitar.

**Aceitação integral:** aberta. Limites antes de materializar, pastas/destinos explícitos e nenhuma escrita/envio implícitos.

### AUTO-01 — Repositório de regras, gatilhos e concessões; ativar/desativar por utilizador

**Dependências e critérios preservados:** RUN-01/02 + EXE-02 + CTX-04. Regra tem dono, revisão, fontes, limites e validade; conteúdo externo não inscreve regras

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/pet_runtime/model.rs](../windows/src-tauri/src/pet_runtime/model.rs) — `Record`, `Owner`, `validate`
- [windows/src-tauri/src/pet_sources/model.rs](../windows/src-tauri/src/pet_sources/model.rs) — `Owner`, `validate`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Apenas bases/histórico; não há aceitação integral específica deste requisito.

**Limite / pendência:** Definição/ativação/revisão de regras e ligação de grants faltam.

**Aceitação integral:** aberta. Regra autorizada tem dono, fontes, gatilho, validade e limites; texto externo não a cria.

### AUTO-02 — Agendador de horários/eventos e deduplicação de ocorrências

**Dependências e critérios preservados:** AUTO-01. Fuso IANA, mudança de hora, suspensão/offline e políticas skip/coalesce/decisão; nenhum efeito duplicado por evento repetido

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/pet_runtime/resource_wait.rs](../windows/src-tauri/src/pet_runtime/resource_wait.rs) — `Entry`, `valid`, `same_wait`
- [windows/src-tauri/src/pet_runtime/model.rs](../windows/src-tauri/src/pet_runtime/model.rs) — `Record`, `Owner`, `validate`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Apenas bases/histórico; não há aceitação integral específica deste requisito.

**Limite / pendência:** Agendamento com ocorrências/DST/offline não aceite ponta a ponta.

**Aceitação integral:** aberta. Fuso IANA, debounce/deduplicação, skip/coalesce/decisão sem avalanche.

### AUTO-03 — Primeira rotina: preparação diária a partir de calendários escolhidos

**Dependências e critérios preservados:** AUTO-01/02 + EXE-03. Dados sintéticos desconhecidos pelo modelo, várias fontes, frescura/cobertura, falha parcial e uma única notificação no horário permitido

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/pet_calendar/mod.rs](../windows/src-tauri/src/pet_calendar/mod.rs) — `Note`, `query`, `fmt`
- [windows/src-tauri/src/pet_sources/mod.rs](../windows/src-tauri/src/pet_sources/mod.rs) — `Consent`, `Document`, `path`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Apenas bases/histórico; não há aceitação integral específica deste requisito.

**Limite / pendência:** Multicalendário, regra/ocorrência e notificação dependem CTX-04/AUTO.

**Aceitação integral:** aberta. Uma ocorrência com dados novos, falha parcial, origem e uma notificação permitida.

### AUTO-04 — Rotina de acompanhamento de objetivo/projeto e pedido de decisão

**Dependências e critérios preservados:** RUN-03/04 + MEM-02. Progresso verificável, interrupção/retoma sem repetir efeitos, orçamento repartido e comunicação sem spam

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/pet_runtime/criteria.rs](../windows/src-tauri/src/pet_runtime/criteria.rs) — `Playback`, `Predicate`, `valid`
- [windows/src-tauri/src/pet_runtime/continuation.rs](../windows/src-tauri/src/pet_runtime/continuation.rs) — `Reference`, `source_matches`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Apenas bases/histórico; não há aceitação integral específica deste requisito.

**Limite / pendência:** Acompanhamento periódico autorizado e memória de projeto não concluídos.

**Aceitação integral:** aberta. Detetar bloqueio/progresso e pedir decisão sem replay, spam ou orçamento renovado.

### AUTO-05 — Notificações, silêncio e escalada de bloqueios

**Dependências e critérios preservados:** OBS-01 + AUTO-01. Conteúdo sensível limitado, deduplicação e validade; indisponibilidade não causa reenvio infinito ou mudança de canal/conta

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/pet_runtime/model.rs](../windows/src-tauri/src/pet_runtime/model.rs) — `Record`, `Owner`, `validate`
- [windows/src-tauri/src/pet_observability/mod.rs](../windows/src-tauri/src/pet_observability/mod.rs) — 

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Apenas bases/histórico; não há aceitação integral específica deste requisito.

**Limite / pendência:** Silêncio, escalada e deduplicação por ocorrência/canal por completar.

**Aceitação integral:** aberta. Validade/limites/sensibilidade e ausência de reenvio infinito ou troca de conta/canal.

### TEAM-01 — Protocolo comum de comunicação direta, descoberta e delegação

**Dependências e critérios preservados:** PET-01/02 + RUN-01 + EXE-02. Mensagens/pedidos/respostas/eventos/resultados com origem/destino, correlação/causa, tarefa/objetivo, idempotência, artefactos, autoridade, orçamento e prazo explícitos; reutilizar mecanismos existentes. Utilizador e pets chamam cada pet diretamente; nenhuma delegação amplia permissões

**Responsabilidade observada / bases a reutilizar:**

- [windows/src/mochi/delegation.ts](../windows/src/mochi/delegation.ts) — `delegatedRequest`
- [windows/src-tauri/src/pet_runtime/authority.rs](../windows/src-tauri/src/pet_runtime/authority.rs) — `Failure`, `control_error`, `Operation`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Apenas bases/histórico; não há aceitação integral específica deste requisito.

**Limite / pendência:** Não existe prova de envelope/entrega durável e delegação comum operacional.

**Aceitação integral:** aberta. Discovery/mensagens diretas com origem nativa, IDs/causa/ACK/prazo e dados autorizados.

### TEAM-02 — Execução paralela/assíncrona, verificação, cancelamento e coordenação resiliente

**Dependências e critérios preservados:** TEAM-01 + RUN-03/04 + EXE-03. Mesmo controlo para tarefas diretas/delegadas; um dono por efeito, entrega deduplicada, limites de profundidade/concorrência/custo, ciclos/conflitos recusados e falhas isoladas. Coordenador Ollama configurável, com visão autorizada dos trabalhos, sem intermediação obrigatória; restantes pets operacionais sem coordenador/motor indisponível. Resultados com proveniência, sem copiar o histórico inteiro

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/pet_runtime/leases.rs](../windows/src-tauri/src/pet_runtime/leases.rs) — `Lease`, `basic`, `valid`
- [windows/src-tauri/src/pet_runtime/effect_budget.rs](../windows/src-tauri/src/pet_runtime/effect_budget.rs) — `Kind`, `Phase`, `Attempt`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Apenas bases/histórico; não há aceitação integral específica deste requisito.

**Limite / pendência:** Reserva de orçamento pai/filho, ramos/ciclos/conflitos e ausência de coordenador não aceites.

**Aceitação integral:** aberta. Ramos independentes, backpressure/cancelamento, saldo repartido e nenhum privilégio herdado.

### SYS-01 — Matriz pet × subplugin × operação × recurso; autoridade dos pets ligada ao executor comum

**Dependências e critérios preservados:** PET-01 + EXE-01/02. Plugin Sistema não concede tudo; Som pode ser partilhado por Spotify/Ollama; revogação de um pet preserva o outro

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/pet_system.rs](../windows/src-tauri/src/pet_system.rs) — `authorize`
- [windows/src-tauri/src/pet_audio/system_policy.rs](../windows/src-tauri/src/pet_audio/system_policy.rs) — `allowed`, `SystemSnapshot`, `SystemReceipt`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Relatos/checkpoints por composição; as provas parciais não cobrem todos os critérios deste ID.

**Limite / pendência:** Matriz pet×subplugin×operação×recurso ainda por ligar a executores/pets.

**Aceitação integral:** aberta. Sistema principal sem grant global; Som Spotify/Ollama com revogação isolada.

**Incremento942/C2B1A3:** IO nas fronteiras existentes de Rede, Som e DMS;15dirigidos/2967full21ignorados/build555/native208 com16recusas tipadas. [Checkpoint](../.build/pet-goal-execution-next-r1/SYSTEM-IO-PARTIAL-CHECKPOINT-R43F10B2C2B1A3.json). C2B2/C3 e aceitação integral continuam abertos.

- [pet_native_network_core/executor.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_native_network_core/executor.rs) — `ActorCommand`, `publish`
- [pet_audio/linux.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_audio/linux.rs) — `validate_dispatch`, `control_for`
- [pet_system.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_system.rs) — `DmsUse`, `dms`

**Incremento945/C2B2A2:** worker mantém Hold até recolha do grupo; controlo de Som→readback e DMS tipado no Tauri com peers privados.13dirigidos/2980full21ignorados/build558/native210. [Checkpoint](../.build/pet-goal-execution-next-r1/SYSTEM-HELPER-PARTIAL-CHECKPOINT-R43F10B2C2B2A2.json). C3/lifecycle/aceitação integral permanecem.

- [pet_helper.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_helper.rs) — `Lane`, `Session`, `supervise`
- [pet_process_group.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_process_group.rs) — `OwnedChild`, `finish_ready`, `group_has_live_members`

### SYS-02 — Rede e Bluetooth como uma unidade de execução

**Dependências e critérios preservados:** SYS-01 + EXE-03. Estado/scan/alvos/prompts/perfis e operações suportadas; distinguir implementação/ACK/efeito. Sem desligar Wi-Fi nem alterar dispositivos em QA sem autorização específica

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/pet_system.rs](../windows/src-tauri/src/pet_system.rs) — `authorize`
- [docs/PETS-ARCHITECTURE-HISTORY-2026-10-23.md](../docs/PETS-ARCHITECTURE-HISTORY-2026-10-23.md) — `Arquitetura dos pets companheiros — Linux`, `Prioridade de desenvolvimento atual`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Relatos/checkpoints por composição; as provas parciais não cobrem todos os critérios deste ID.

**Limite / pendência:** Paridade dos handlers, alvos, prompts, dispositivos e efeito real não concluída.

**Aceitação integral:** aberta. Unidade completa com grants separados, ACK distinto de efeito; sem alterar rádios em QA.

**Incremento942/C2B1A3:** IO nas fronteiras existentes de Rede, Som e DMS;15dirigidos/2967full21ignorados/build555/native208 com16recusas tipadas. [Checkpoint](../.build/pet-goal-execution-next-r1/SYSTEM-IO-PARTIAL-CHECKPOINT-R43F10B2C2B1A3.json). C2B2/C3 e aceitação integral continuam abertos.

- [pet_native_network_core/transport.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_native_network_core/transport.rs) — `effect_ready`, `write_frame_checked`
- [pet_native_network_core/executor.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_native_network_core/executor.rs) — `ActorCommand`
- [pet_bluetooth/mod.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_bluetooth/mod.rs) — `PreferencesUnconfirmed`

**Incremento945/C2B2A2:** worker mantém Hold até recolha do grupo; controlo de Som→readback e DMS tipado no Tauri com peers privados.13dirigidos/2980full21ignorados/build558/native210. [Checkpoint](../.build/pet-goal-execution-next-r1/SYSTEM-HELPER-PARTIAL-CHECKPOINT-R43F10B2C2B2A2.json). C3/lifecycle/aceitação integral permanecem.

- [pet_helper.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_helper.rs) — `Lane`, `Session`, `supervise`
- [pet_process_group.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_process_group.rs) — `OwnedChild`, `finish_ready`, `group_has_live_members`

### SYS-03 — Som/multimédia; ecrã/energia/sessão em subunidades completas

**Dependências e critérios preservados:** SYS-01 + EXE-03. Streams/dispositivos/perfis e restante âmbito da arquitetura; partilha/concorrência verificadas e indicadores preservados

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/pet_audio/mod.rs](../windows/src-tauri/src/pet_audio/mod.rs) — `authorize`, `pet_audio_snapshot`, `pet_audio_control`
- [windows/src-tauri/src/pet_audio/linux.rs](../windows/src-tauri/src/pet_audio/linux.rs) — `Cache`, `cache`, `now`
- [windows/src-tauri/src/pet_system.rs](../windows/src-tauri/src/pet_system.rs) — `authorize`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Relatos/checkpoints por composição; as provas parciais não cobrem todos os critérios deste ID.

**Limite / pendência:** Streams/perfis/microfone e Ecrã/Energia/Sessão/matriz ainda sem aceitação integral.

**Aceitação integral:** aberta. Mesmos executores com autoridades distintas, alvos exatos, concorrência e indicadores preservados.

**Incremento942/C2B1A3:** IO nas fronteiras existentes de Rede, Som e DMS;15dirigidos/2967full21ignorados/build555/native208 com16recusas tipadas. [Checkpoint](../.build/pet-goal-execution-next-r1/SYSTEM-IO-PARTIAL-CHECKPOINT-R43F10B2C2B1A3.json). C2B2/C3 e aceitação integral continuam abertos.

- [pet_audio/linux.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_audio/linux.rs) — `run_checked`, `validate_dispatch`, `control_for`
- [pet_audio/mod.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_audio/mod.rs) — `authorize_owner`
- [pet_system.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_system.rs) — `DmsUse`, `dms`

**Incremento945/C2B2A2:** worker mantém Hold até recolha do grupo; controlo de Som→readback e DMS tipado no Tauri com peers privados.13dirigidos/2980full21ignorados/build558/native210. [Checkpoint](../.build/pet-goal-execution-next-r1/SYSTEM-HELPER-PARTIAL-CHECKPOINT-R43F10B2C2B2A2.json). C3/lifecycle/aceitação integral permanecem.

- [pet_helper.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_helper.rs) — `Lane`, `Session`, `supervise`
- [pet_process_group.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_process_group.rs) — `OwnedChild`, `finish_ready`, `group_has_live_members`
- [pet_audio/linux.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_audio/linux.rs) — `control_for`, `validate_dispatch`
- [pet_system.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_system.rs) — `dms`, `dms_caller`, `dms_executable`

### INT-01 — Integrações de serviço e comunicação externa com operações distintas

**Dependências e critérios preservados:** PET-05 + EXE-02/03. Abrir rascunho não envia; envio futuro exige concessão própria. Autenticação/tokens e identidades nativas preservados

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/pets.factory.json](../windows/src-tauri/pets.factory.json) — `definição declarativa; sem inferência de prontidão`
- [windows/src-tauri/src/pet_config.rs](../windows/src-tauri/src/pet_config.rs) — `select`, `retired`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings.rs) — `update_checked_with`, `save_to`, `read_initialized`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_autostart.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_autostart.rs) — `admit`, `set`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/lib.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/lib.rs) — `pet_draft_mode`, `Shared`, `BootInfo`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/settings/autostart-setting.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/settings/autostart-setting.ts) — `buildAutostartSetting`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_autostart_settings.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_autostart_settings.rs) — `Patch`, `Controller`, `Receipt`, `preserve_incoming`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/phone_link/mod.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/phone_link/mod.rs) — `commit_preferences`, `Keystore`, `get`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/phone_link/preferences.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/phone_link/preferences.rs) — `Change`, `commit_with`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/voice/session.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/voice/session.rs) — `commit_alias_with`, `active`, `suspend_hotword`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/private_registry.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/private_registry.rs) — `load_at`, `save_guarded`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/preferences_file.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/preferences_file.rs) — `Witness`, `read`, `current`, `commit`, `unconfirmed`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings_revision.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings_revision.rs) — `Revision`, `load_from`, `bind_strict`, `save_to`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Relatos/checkpoints por composição; as provas parciais não cobrem todos os critérios deste ID.
- [RESTORED-FEATURE-PARTIAL-CHECKPOINT-R43F10A4.json](../.build/pet-goal-execution-next-r1/RESTORED-FEATURE-PARTIAL-CHECKPOINT-R43F10A4.json) — 925/A4:200 negativo/201 positivo;81dirigidos/2892Rust21ignorados/1266web/build542. Recusa anteswrite/eventos,0600/restart/backup/erro executor eUI readback;HTTP2CLI1/DBus/HOME sintéticos/0quota;nenhuma revalidação concedida ou aceitação humana/instalada.
- [AUTOSTART-INTENT-PARTIAL-CHECKPOINT-R43F10B1A1.json](../.build/pet-goal-execution-next-r1/AUTOSTART-INTENT-PARTIAL-CHECKPOINT-R43F10B1A1.json) — 928/A1:6dirigidos/2898Rust21ignorados/1268web/build544/native202; intenção porcampo,duasjanelas e no-op/erro semreplay. HOME/HTTP/CLI/DBus privados;0quota. F10B2/revalidação/adoção/49integrais abertos.
- [SETTINGS-CONSUMERS-PARTIAL-CHECKPOINT-R43F10B2A3.json](../.build/pet-goal-execution-next-r1/SETTINGS-CONSUMERS-PARTIAL-CHECKPOINT-R43F10B2A3.json) — 931/A3:2905Rust21ignorados/build547/native203. Novecomandos Tauri sobfalha privada sem alteração em memória/evento;7novasfixtures deconsumidores/nomes. Web1268/TSC/Vite anterior reutilizado com inputs exatos.0quota. CAS/durabilidade/reconciliação/efeito listener e aceitação integral abertos.
- [SETTINGS-REVISION-PARTIAL-CHECKPOINT-R43F10B2B1.json](../.build/pet-goal-execution-next-r1/SETTINGS-REVISION-PARTIAL-CHECKPOINT-R43F10B2B1.json) — 934/B1/Linux:12novos/124selecionados1ignorado/2917full21ignorados/check/build550/native204. Revisão carregada,edição externa recusada antesdegravar/efeito,reinício privado/0600/backup eJSON preservado. HTTP/CLI/DBus sintéticos/0quota;web1268reutilizado. Não prova CAS atómico,barreira global/duráveldeincerteza ou entrega.

**Limite / pendência:** Operações reais de cada serviço/envio/contas não universalmente implementadas/aceites. F10A não implementa revalidação: faltam intenção por campo/CAS e confirmação após publicação/fsync diretório antes de a conceder. Consumidores não fecham escritor: CAS/fsyncdiretório/resultado incerto tipado/reconciliação/reinício pendentes. Listener ainda antes da gravação e chaves com efeitos próprios; não revalidados/entregues. Isto é verificação otimista para writers cooperantes, não CAS atómico/sandbox deoutros processos do mesmoUID. Incerteza pós-publicação tem classe/recusa de nova escrita,mas barreira global deefeitos/reconciliação durável/reinício/fsyncdiretório eparidade Mac/Windows ainda pendentes.

**Aceitação integral:** aberta. F10B2C: cercar todos novos efeitos com estado de IO do Shared atual, sem invalidar turnos capturados por simples mudança legítima de outro campo;barreira durável/reconciliação sem replay/fsyncdiretório e intenção do listener/chaves;depois revalidação/adoção/aceitação integral.

**Incremento948/C3A4:** intenção/resultado/revisão de preferências duráveis e formato de perfil/backup/restauro; 13dirigidos/2993full21ignorados/build561/native212. [Checkpoint](../.build/pet-goal-execution-next-r1/SETTINGS-DURABLE-PARTIAL-CHECKPOINT-R43F10B2C3A4.json). Reconciliação humana e listener/chaves ainda pendentes; sem adoção/instalação.

- [settings_durable.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings_durable.rs) — `Record`, `Observation`, `Prepared`, `restored_record`
- [settings_revision.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings_revision.rs) — `Bound`, `save_with`, `note_failed`
- [preferences_file.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/preferences_file.rs) — `commit_with`

### VOICE-01 — Controlo e interação integral do sistema de pets por voz

**Dependências e critérios preservados:** PET-02/03 + EXE-01/02 + CTX-04 + SYS-01. Cobertura por operação de pets/sessões/conversas/objetivos/tarefas/regras/fontes/memória/configuração/modelos e Sistema autorizado; mesmos dono, permissões e pedidos nativos que chat/terminal. Decisão única por pedido, clarificação falada de ambiguidade, Stop prioritário e sem reenvio após reconexão

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/voice/admission.rs](../windows/src-tauri/src/voice/admission.rs) — `uses_pet_response`, `known_voice_address`, `Route`
- [windows/src-tauri/src/voice/mod.rs](../windows/src-tauri/src/voice/mod.rs) — `VoiceView`, `Running`, `capture_lease`
- [windows/src-tauri/src/cli_decisions.rs](../windows/src-tauri/src/cli_decisions.rs) — `target`, `live`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings.rs) — `update_checked_with`, `save_to`, `read_initialized`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_autostart.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_autostart.rs) — `admit`, `set`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/lib.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/lib.rs) — `pet_draft_mode`, `Shared`, `BootInfo`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/settings/autostart-setting.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/settings/autostart-setting.ts) — `buildAutostartSetting`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_autostart_settings.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_autostart_settings.rs) — `Patch`, `Controller`, `Receipt`, `preserve_incoming`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/phone_link/mod.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/phone_link/mod.rs) — `commit_preferences`, `Keystore`, `get`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/phone_link/preferences.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/phone_link/preferences.rs) — `Change`, `commit_with`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/voice/session.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/voice/session.rs) — `commit_alias_with`, `active`, `suspend_hotword`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/private_registry.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/private_registry.rs) — `load_at`, `save_guarded`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/preferences_file.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/preferences_file.rs) — `Witness`, `read`, `current`, `commit`, `unconfirmed`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings_revision.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings_revision.rs) — `Revision`, `load_from`, `bind_strict`, `save_to`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Relatos/checkpoints por composição; as provas parciais não cobrem todos os critérios deste ID.
- [RESTORED-FEATURE-PARTIAL-CHECKPOINT-R43F10A4.json](../.build/pet-goal-execution-next-r1/RESTORED-FEATURE-PARTIAL-CHECKPOINT-R43F10A4.json) — 925/A4:200 negativo/201 positivo;81dirigidos/2892Rust21ignorados/1266web/build542. Recusa anteswrite/eventos,0600/restart/backup/erro executor eUI readback;HTTP2CLI1/DBus/HOME sintéticos/0quota;nenhuma revalidação concedida ou aceitação humana/instalada.
- [AUTOSTART-INTENT-PARTIAL-CHECKPOINT-R43F10B1A1.json](../.build/pet-goal-execution-next-r1/AUTOSTART-INTENT-PARTIAL-CHECKPOINT-R43F10B1A1.json) — 928/A1:6dirigidos/2898Rust21ignorados/1268web/build544/native202; intenção porcampo,duasjanelas e no-op/erro semreplay. HOME/HTTP/CLI/DBus privados;0quota. F10B2/revalidação/adoção/49integrais abertos.
- [SETTINGS-CONSUMERS-PARTIAL-CHECKPOINT-R43F10B2A3.json](../.build/pet-goal-execution-next-r1/SETTINGS-CONSUMERS-PARTIAL-CHECKPOINT-R43F10B2A3.json) — 931/A3:2905Rust21ignorados/build547/native203. Novecomandos Tauri sobfalha privada sem alteração em memória/evento;7novasfixtures deconsumidores/nomes. Web1268/TSC/Vite anterior reutilizado com inputs exatos.0quota. CAS/durabilidade/reconciliação/efeito listener e aceitação integral abertos.
- [SETTINGS-REVISION-PARTIAL-CHECKPOINT-R43F10B2B1.json](../.build/pet-goal-execution-next-r1/SETTINGS-REVISION-PARTIAL-CHECKPOINT-R43F10B2B1.json) — 934/B1/Linux:12novos/124selecionados1ignorado/2917full21ignorados/check/build550/native204. Revisão carregada,edição externa recusada antesdegravar/efeito,reinício privado/0600/backup eJSON preservado. HTTP/CLI/DBus sintéticos/0quota;web1268reutilizado. Não prova CAS atómico,barreira global/duráveldeincerteza ou entrega.

**Limite / pendência:** Cobertura integral das operações e decisões multicanal não demonstrada. F10A não implementa revalidação: faltam intenção por campo/CAS e confirmação após publicação/fsync diretório antes de a conceder. Consumidores não fecham escritor: CAS/fsyncdiretório/resultado incerto tipado/reconciliação/reinício pendentes. Listener ainda antes da gravação e chaves com efeitos próprios; não revalidados/entregues. Isto é verificação otimista para writers cooperantes, não CAS atómico/sandbox deoutros processos do mesmoUID. Incerteza pós-publicação tem classe/recusa de nova escrita,mas barreira global deefeitos/reconciliação durável/reinício/fsyncdiretório eparidade Mac/Windows ainda pendentes.

**Aceitação integral:** aberta. F10B2C: cercar todos novos efeitos com estado de IO do Shared atual, sem invalidar turnos capturados por simples mudança legítima de outro campo;barreira durável/reconciliação sem replay/fsyncdiretório e intenção do listener/chaves;depois revalidação/adoção/aceitação integral.

**Incremento939/C1:** origem atual de IO e preservação de Stop/recusa/evidência;14fixtures/2931full21ignorados/1272web/build552/native205 sintético. [Checkpoint](../.build/pet-goal-execution-next-r1/SETTINGS-EFFECT-GUARD-PARTIAL-CHECKPOINT-R43F10B2C1.json). Guardas finais e reconciliação durável pendentes, sem entrega integral.

**Incremento940/C2A3:** origem atual na escrita nativa final, controlo/recusa tipados e pre-spawn;21dirigidos/2952full21ignorados/build553/native206 com corte real de4096bytes num peer sintético. [Checkpoint](../.build/pet-goal-execution-next-r1/FINAL-NATIVE-DISPATCH-PARTIAL-CHECKPOINT-R43F10B2C2A3.json). C2B/C3 e aceitação integral continuam abertos.

- [pet_native_write.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_native_write.rs) — `Guard`, `frame`
- [pet_native_choices.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_native_choices.rs) — `pure_refusal`

**Incremento942/C2B1A3:** IO nas fronteiras existentes de Rede, Som e DMS;15dirigidos/2967full21ignorados/build555/native208 com16recusas tipadas. [Checkpoint](../.build/pet-goal-execution-next-r1/SYSTEM-IO-PARTIAL-CHECKPOINT-R43F10B2C2B1A3.json). C2B2/C3 e aceitação integral continuam abertos.

- [pet_audio/mod.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_audio/mod.rs) — `spoken`
- [voice/services.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/voice/services.rs) — `spoken`

**Incremento945/C2B2A2:** worker mantém Hold até recolha do grupo; controlo de Som→readback e DMS tipado no Tauri com peers privados.13dirigidos/2980full21ignorados/build558/native210. [Checkpoint](../.build/pet-goal-execution-next-r1/SYSTEM-HELPER-PARTIAL-CHECKPOINT-R43F10B2C2B2A2.json). C3/lifecycle/aceitação integral permanecem.

- [pet_helper.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_helper.rs) — `Lane`, `Session`, `supervise`
- [pet_process_group.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_process_group.rs) — `OwnedChild`, `finish_ready`, `group_has_live_members`

**Incremento948/C3A4:** intenção/resultado/revisão de preferências duráveis e formato de perfil/backup/restauro; 13dirigidos/2993full21ignorados/build561/native212. [Checkpoint](../.build/pet-goal-execution-next-r1/SETTINGS-DURABLE-PARTIAL-CHECKPOINT-R43F10B2C3A4.json). Reconciliação humana e listener/chaves ainda pendentes; sem adoção/instalação.

- [settings_durable.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings_durable.rs) — `Record`, `Observation`, `Prepared`, `restored_record`
- [settings_revision.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings_revision.rs) — `Bound`, `save_with`, `note_failed`
- [preferences_file.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/preferences_file.rs) — `commit_with`

### VOICE-02 — Aceitação real de microfone/Bluetooth/TTS PT/EN/misto

**Dependências e critérios preservados:** VOICE-01. Latência/pronúncia/alternância/falhas, conversa contínua, interrupção de TTS e reconexão com áudio real; matriz de cobertura funcional por voz. Uma transcrição escrita não fecha este requisito

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/voice/mod.rs](../windows/src-tauri/src/voice/mod.rs) — `VoiceView`, `Running`, `capture_lease`
- [linux/whisper-model.json](../linux/whisper-model.json) — `definição declarativa; sem inferência de prontidão`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Relatos/checkpoints por composição; as provas parciais não cobrem todos os critérios deste ID.

**Limite / pendência:** Microfone/Bluetooth/TTS PT/EN/misto/latência/diálogo humanos pendentes.

**Aceitação integral:** aberta. Provar áudio real no ambiente autorizado; fixtures não certificam qualidade humana.

### OBS-01 — Estado, decisões, prazos, recursos e consumo por tarefa/pet

**Dependências e critérios preservados:** Distinguir terminais/TTYs, clientes e sessões, com atividade/espera/suspensão/inatividade confirmadas ou desconhecido; lease preparado não é inferência. Fontes/cobertura explícitas, sem segredos/documentos privados em logs

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/pet_observability/process_census.rs](../windows/src-tauri/src/pet_observability/process_census.rs) — `summarize`, `Census`
- [windows/src-tauri/src/pet_runtime/model.rs](../windows/src-tauri/src/pet_runtime/model.rs) — `Record`, `Owner`, `validate`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_history.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_history.rs) — `goal_snapshot`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/history.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/history.rs) — `RestoredSource`, `Snapshot`, `empty`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_history.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_history.rs) — `phase`, `history_inventory`, `history`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/runtime-goals.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/runtime-goals.ts) — `buildRuntimeGoals`, `syncMusic`, `syncPets`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/restored_future.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/restored_future.rs) — `Review`, `Origin`, `valid_for`, `request_id`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restored_future.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restored_future.rs) — `Witness`, `capture`, `current`, `verify_restored_origin`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/store_restored_future.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/store_restored_future.rs) — `existing_restored_future`, `create_restored_future_clock`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/restored_future_api.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/restored_future_api.rs) — `pet_runtime_prepare_restored_future`, `pet_runtime_confirm_restored_future`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/restored-future-review.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/restored-future-review.ts) — `buildRestoredFutureReview`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Relatos/checkpoints por composição; as provas parciais não cobrem todos os critérios deste ID.
- [RESTORED-HISTORY-PARTIAL-CHECKPOINT-R43F9B3A.json](../.build/pet-goal-execution-next-r1/RESTORED-HISTORY-PARTIAL-CHECKPOINT-R43F9B3A.json) — 915/B3A2: prova negativa,2870Rust21ignorados/1256web/TSC/Vite/build536/native196,2HTTP1CLI sintéticos/0quota. História2/origemexata/totaldesconhecido/read-only antesIO,novoGoalhistória1. B3F2/adoção/entrega/humana/49integrais abertos.
- [RESTORED-FUTURE-PARTIAL-CHECKPOINT-R43F9B3F2A7.json](../.build/pet-goal-execution-next-r1/RESTORED-FUTURE-PARTIAL-CHECKPOINT-R43F9B3F2A7.json) — 923/A7:32dirigidos/2886Rust21ignorados/1262web/build542/native199; origem13 só dados + runtime comum/consentimento/Stop/reinício/backup; UI automatizada e2HTTP1CLI sintéticos/0quota,leitor915 real preserva13. Época seguinte fixture;49integrais/adoção/entrega abertos.

**Limite / pendência:** Todos os providers/estados externos e instalação não têm cobertura completa.

**Aceitação integral:** aberta. Fronteiras conjuntas4096/64MiB, migrações/revalidações/adoção local de fontes exatas; lifecycle e aceitação integral/provider antes de TEAM/Guardian e entrega.

### OBS-02 — Cobertura/rotação dos ledgers e captura de consumo nativo

**Dependências e critérios preservados:** Separar tentativas/thread/conta/quota; não perder notificação antes do ACK, não somar snapshots como deltas e não autenticar por abrir painel

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/pet_usage.rs](../windows/src-tauri/src/pet_usage.rs) — `record`
- [windows/src-tauri/src/pet_observability/usage.rs](../windows/src-tauri/src/pet_observability/usage.rs) — `now_seconds`, `seconds_from_millis`, `LedgerError`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Relatos/checkpoints por composição; as provas parciais não cobrem todos os critérios deste ID.

**Limite / pendência:** Cobertura/rotação por conta/thread/provider e reinícios não integralmente aceite.

**Aceitação integral:** aberta. Não somar snapshots como deltas, não perder antes do ACK nem autenticar por consulta.

### DATA-01 — Repositórios, migrações, retenção e backups

**Dependências e critérios preservados:** Falhas antes/depois de publicação, escritor concorrente, reinício e restauro; nenhum dado ativo desaparece por rotação

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/private_registry.rs](../windows/src-tauri/src/private_registry.rs) — `load_at`, `save_guarded`
- [windows/src-tauri/src/pet_data/startup.rs](../windows/src-tauri/src/pet_data/startup.rs) — `Owner`, `name`, `local_address`
- [windows/src-tauri/src/pet_runtime/store.rs](../windows/src-tauri/src/pet_runtime/store.rs) — `Store`, `read`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/inventory.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/inventory.rs) — `legacy_history_backup`, `Area`, `Rule`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/tests.rs) — `Temp`, `new`, `layout`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/legacy_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/legacy_tests.rs) — `run`, `aged`, `sha`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/current_profile.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/current_profile.rs) — `Review`, `capture`, `fingerprint`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/backup_cli.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/backup_cli.rs) — `Request`, `parse`, `code`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/staging.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/staging.rs) — `Failure`, `Phase`, `Inspection`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication.rs) — `Header`, `Complete`, `Admission`
- [windows/src/core/session-drafts.ts](../windows/src/core/session-drafts.ts) — `draftTarget`, `targetFromKey`, `parseStoredDraft`
- [windows/src/desktop/pet-chat.ts](../windows/src/desktop/pet-chat.ts) — `acceptRect`, `fromLocal`, `report`
- [windows/src/views/chat.ts](../windows/src/views/chat.ts) — `watchCapabilities`, `capabilityText`, `bubble`
- [docs/PETS-DATA-BACKUP.md](../docs/PETS-DATA-BACKUP.md) — `Backup nativo — uso e cobertura`, `R43F10B2B1 — revisão carregada antes de substituir preferências`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/mod.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/mod.rs) — `allowed`, `path`, `current`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/model.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/model.rs) — `Record`, `validate`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/store.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/store.rs) — `read`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/tests.rs) — `Files`, `new`, `path`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/native-drafts.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/native-drafts.ts) — `text`, `draftWireTarget`, `nativeDraftTarget`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/draft-migration.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/draft-migration.ts) — `draftImportReceipt`, `canonicalLegacyBatch`, `migration`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/inline-drafts.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/inline-drafts.ts) — `InlineDrafts`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/draft-status.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/draft-status.ts) — `draftStatusControl`, `sync`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/chat.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/chat.ts) — `watchCapabilities`, `capabilityText`, `bubble`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/pet-factory.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/pet-factory.ts) — `invalid`, `merge`, `freeze`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/fixtures/reader_v1.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/fixtures/reader_v1.rs) — `Target`, `Origin`, `id`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/recovery.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/recovery.rs) — `Owner`, `validate`, `Availability`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/recovery_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/recovery_tests.rs) — `Files`, `new`, `path`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/draft-recovery.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/draft-recovery.ts) — `DraftRecoveryFailure`, `invalid`, `archive`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/draft-recovery.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/draft-recovery.ts) — `draftRecoveryControl`, `sync`, `capture`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/archives.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/archives.rs) — `Archive`, `empty`, `validate`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/reconcile.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/reconcile.rs) — `Policy`, `Summary`, `Review`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/reconcile_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/reconcile_tests.rs) — `Files`, `new`, `path`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/fixtures/reader_v2.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/fixtures/reader_v2.rs) — `Target`, `Origin`, `id`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/draft-reconcile.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/draft-reconcile.ts) — `DraftReconcileFailure`, `invalid`, `wire`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/draft-reconcile.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/draft-reconcile.ts) — `draftReconcileControl`, `sync`, `capture`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore.rs) — `Intent`, `Restored`, `Mode`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_tests.rs) — `Fixture`, `new`, `layout`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restore_cli.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restore_cli.rs) — `activation_sealed`, `execute`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/bootstrap.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/bootstrap.rs) — `Session`, `prepare`, `paths`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration.rs) — `FileRef`, `HistoricalGrant`, `GoalRef`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_tests.rs) — `clone_dir`, `copy_fixture`, `review`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_authority.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_authority.rs) — `error`, `AuthorityReview`, `FirstReview`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_configuration.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_configuration.rs) — `configuration`, `epoch_scope`, `verify_payload`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_configuration_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_configuration_tests.rs) — `settings`, `pets`, `Fixture`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_config.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_config.rs) — `select`, `retired`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_history.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_history.rs) — `capture_next`, `decode_first`, `review`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/chat_restored_history_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/chat_restored_history_tests.rs) — `fixture`, `historical_conversation_is_readable_with_notice_but_cannot_reserve_or_start_a_turn`, `a_fresh_conversation_can_obtain_normal_reservation_while_old_history_is_preserved`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/chat.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/chat.rs) — `Chat`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_transaction.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_transaction.rs) — `Activation`, `initialization_delta`, `EpochInitialization`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_projection.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_projection.rs) — `Choices`, `Grant`, `Operation`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_active_restored.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_active_restored.rs) — `Bound`, `verify_initial_payload`, `publish`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_transaction_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_transaction_tests.rs) — `Fixture`, `new`, `with_authority`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restored_backup.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restored_backup.rs) — `cannot`, `verify`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_epochs.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_epochs.rs) — `EpochStep`, `EpochReview`, `prior_staging`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_epoch.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_epoch.rs) — `capture_parts`, `completed`, `verify_current`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/revision_fence.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/revision_fence.rs) — `Tree`, `descriptor_budget`, `capture`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_backup_epoch_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_backup_epoch_tests.rs) — `Round`, `new`, `archive`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent.rs) — `preparation_state`, `preview`, `apply_using`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_independent.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_independent.rs) — `IndependentKeyState`, `key_state`, `prepare`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent_operational.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent_operational.rs) — `Custody`, `validate`, `PreparedAuthority`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_independent_transaction_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_independent_transaction_tests.rs) — `prepared`, `old_goal`, `independent_admission4_preserves_absent_original_authority_and_denies_old_ids_before_new_consent`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/writer.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/writer.rs) — `model_once`, `authorize_body`, `record_result`, `before_send`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/results.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/results.rs) — `preflight`, `lookup`, `publish`, `recover`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/result_sink_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/result_sink_tests.rs) — `bytes`, `visit`, `incompatible`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_store.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_store.rs) — `KeyFile`, `parse`, `restoration_base`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent_tests.rs) — `Fixture`, `tree`, `visit`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_retention.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_retention.rs) — `RetentionReview`, `scan`, `current`, `apply_using`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_retention_inventory.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_retention_inventory.rs) — `Opaque`, `Budget`, `inspect`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_retention_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_retention_tests.rs) — `review`, `job`, `opaque`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/staging_fs.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/staging_fs.rs) — `transfer_dir`, `publish_dir`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_retained_independent_transaction_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_retained_independent_transaction_tests.rs) — `data`, `retain`, `prepare`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_history.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_history.rs) — `goal_snapshot`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/history.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/history.rs) — `RestoredSource`, `Snapshot`, `empty`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_history.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_history.rs) — `phase`, `history_inventory`, `history`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/runtime-goals.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/runtime-goals.ts) — `buildRuntimeGoals`, `syncMusic`, `syncPets`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/restored_future.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/restored_future.rs) — `Review`, `Origin`, `valid_for`, `request_id`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restored_future.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restored_future.rs) — `Witness`, `capture`, `current`, `verify_restored_origin`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/store_restored_future.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/store_restored_future.rs) — `existing_restored_future`, `create_restored_future_clock`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/restored_future_api.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/restored_future_api.rs) — `pet_runtime_prepare_restored_future`, `pet_runtime_confirm_restored_future`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/restored-future-review.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/restored-future-review.ts) — `buildRestoredFutureReview`
- [.build/pet-goal-execution-next-r1/retention-boundary-r43f9b4a1-source/windows/src-tauri/src/pet_data/publication_restore_retention_tests.rs](../.build/pet-goal-execution-next-r1/retention-boundary-r43f9b4a1-source/windows/src-tauri/src/pet_data/publication_restore_retention_tests.rs) — `review`, `job`, `opaque`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings.rs) — `update_checked_with`, `save_to`, `read_initialized`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_autostart.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_autostart.rs) — `admit`, `set`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/lib.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/lib.rs) — `pet_draft_mode`, `Shared`, `BootInfo`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/settings/autostart-setting.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/settings/autostart-setting.ts) — `buildAutostartSetting`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_autostart_settings.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_autostart_settings.rs) — `Patch`, `Controller`, `Receipt`, `preserve_incoming`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/phone_link/mod.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/phone_link/mod.rs) — `commit_preferences`, `Keystore`, `get`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/phone_link/preferences.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/phone_link/preferences.rs) — `Change`, `commit_with`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/voice/session.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/voice/session.rs) — `commit_alias_with`, `active`, `suspend_hotword`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/private_registry.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/private_registry.rs) — `load_at`, `save_guarded`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/preferences_file.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/preferences_file.rs) — `Witness`, `read`, `current`, `commit`, `unconfirmed`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings_revision.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings_revision.rs) — `Revision`, `load_from`, `bind_strict`, `save_to`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Relatos/checkpoints por composição; as provas parciais não cobrem todos os critérios deste ID.
- [MEMORY-MIGRATION-PARTIAL-CHECKPOINT-R43F4.json](../.build/pet-goal-execution-next-r1/MEMORY-MIGRATION-PARTIAL-CHECKPOINT-R43F4.json) — SDK854/F4:3 fontes nativas alteradas; baseline antes da correção1passa/2falham; depois2641 Rust/21 ignorados,check/build e Tauri147. Native146/147 prova retoma selecionada,2rondas com2aprovações,7HTTP sintéticos,revogação/reinício e recusa do leitor844. Não certifica backup completo, integração/instalação ou aceitação humana. Timeout145 preservado com causa desconhecida.
- [CURRENT-PROFILE-BACKUP-PARTIAL-CHECKPOINT-R43F5.json](../.build/pet-goal-execution-next-r1/CURRENT-PROFILE-BACKUP-PARTIAL-CHECKPOINT-R43F5.json) — SDK858/F5D:2655Rust/21ignorados,23dirigidos/1ignorado,check-tests/build e composição151.149 prova CLI81ficheiros/integridade/owner/consentimento/recusas;151 arranque normal/continuação7HTTP sintéticos e backup do mesmo perfil.150 Busy anterior retido;D repete só controlo anterior à escrita, nunca IO físico/modelo, com Stop e limite. Sem restauro/instalação/aceitação humana ou quota real.
- [NATIVE-DRAFTS-PARTIAL-CHECKPOINT-R43F6.json](../.build/pet-goal-execution-next-r1/NATIVE-DRAFTS-PARTIAL-CHECKPOINT-R43F6.json) — SDK865/F6D: native draft repository/UTF16/u64/CAS, known IndexedDB migration and retained empty CID; full Rust,1219JS/types/build and native154 with0models/quota. Inline/divergence/orphan UI/restore/adoption/human/installed acceptance remain open.
- [INLINE-DRAFTS-PARTIAL-CHECKPOINT-R43F7.json](../.build/pet-goal-execution-next-r1/INLINE-DRAFTS-PARTIAL-CHECKPOINT-R43F7.json) — SDK869/F7A:2674Rust/21ignorados/check/build,1230JS/types/bundle e nativo158 de cartões autorizados. Inline/unbound/CAS/cópia explícita, leitor antigo/backup;0modelos/quota. Digitação nativa inicial, pre-CID não nativo, órfãos/divergência/restauro/entrega ainda abertos.
- [ORPHAN-DRAFT-RECOVERY-PARTIAL-CHECKPOINT-R43F8.json](../.build/pet-goal-execution-next-r1/ORPHAN-DRAFT-RECOVERY-PARTIAL-CHECKPOINT-R43F8.json) — SDK874/F8A:2684Rust/21ignorados/check/build,1240JS/types/bundle e nativo160 com0modelos/quota. Órfãos/cópia humana scriptada/origem conservada, inline/flutuante/replay/conflito/leitor2/backup. Não reconcilia IndexedDB divergente nem fecha restauro/adoção/aceitação integral.
- [DIVERGENT-DRAFT-RECONCILIATION-PARTIAL-CHECKPOINT-R43F8B.json](../.build/pet-goal-execution-next-r1/DIVERGENT-DRAFT-RECONCILIATION-PARTIAL-CHECKPOINT-R43F8B.json) — SDK882/F8B2:2700Rust/21ignorados/check503/build;1255JS/TSC/Vite emB1 idêntico na produção/B2 e binário byte idêntico. Nativo162 real/IndexedDB/scriptado,0modelos/quota: reconciliação, arquivo/CAS/ACK/lostACK sem replay, recuperação/cópia, restart/leitor2/backup. Sem restauro/adoção/timing de mutação durante IO nativo ou aceitação humana/instalada.
- [CURRENT-PROFILE-RESTORE-PARTIAL-CHECKPOINT-R43F9.json](../.build/pet-goal-execution-next-r1/CURRENT-PROFILE-RESTORE-PARTIAL-CHECKPOINT-R43F9.json) — SDK885/F9A:2713Rust/21ignorados/check506/build;web1255/TSC/Vite reutilizada com bytes não Rust idênticos.163CLI/IPC real/private0models/quota: publicação revista/dataRestoredInactive, par/origem/bytes exatos, crash próprio/retoma/subset divergente/ACK idempotente e leitores recusam. Não é admissão/autoridade executável, semântica de assinatura/contabilidade/continuação ou aceitação humana/instalada.
- [RESTORED-AUTHORITY-PARTIAL-CHECKPOINT-R43F9B1.json](../.build/pet-goal-execution-next-r1/RESTORED-AUTHORITY-PARTIAL-CHECKPOINT-R43F9B1.json) — SDK888/F9B1C:2725Rust/21ignorados/12novos/build509;web1255 fontes idênticas;165HMAC/CLI/IPC real sem inferência/quota. Quarentena histórica/assinaturas/bytes/ACK/tamper/leitores; novo claim e consumo incerto em Rust. Header2 inativo, F9B2/F9B3/adoção/aceitação integral pendentes.
- [RESTORED-CONFIGURATION-PARTIAL-CHECKPOINT-R43F9B2A.json](../.build/pet-goal-execution-next-r1/RESTORED-CONFIGURATION-PARTIAL-CHECKPOINT-R43F9B2A.json) — SDK890/B2A5:2737Rust/21ignorados/12novos/build511/nativo168 real/CLI/IPC/HMAC sem inferência/quota; revisão tipada/inspectionOnly/bytes/ACK e leitores. Não é ativação, admissão operacional, continuação ou aceitação humana/instalada.
- [RESTORED-SESSION-GUARDS-PARTIAL-CHECKPOINT-R43F9B2B1.json](../.build/pet-goal-execution-next-r1/RESTORED-SESSION-GUARDS-PARTIAL-CHECKPOINT-R43F9B2B1.json) — SDK893/B2B1B:2754Rust/21ignorados/17novos/build514;169 regressão real Root1/claim/backup/B1/B2A,0inferência/quota. Guards em fixtures; NÃO restauro operacional/admissão/guard nativo ativo ou aceitação humana/instalada.
- [RESTORED-ACTIVATION-PARTIAL-CHECKPOINT-R43F9B2B2.json](../.build/pet-goal-execution-next-r1/RESTORED-ACTIVATION-PARTIAL-CHECKPOINT-R43F9B2B2.json) — 897/B2B2C1 primeira ativação operacional privada;2764Rust21ignorados/10novos/build518/nativo173 com providers sintéticos. Não fecha backup Root2/épocas/revalidações/B3/adoção/aceitação humana ou instalada.
- [PORTABLE-RESTORED-BACKUP-PARTIAL-CHECKPOINT-R43F9B2C.json](../.build/pet-goal-execution-next-r1/PORTABLE-RESTORED-BACKUP-PARTIAL-CHECKPOINT-R43F9B2C.json) — 898/C1A1 backup/inspeção de ambos roots completos;2770Rust21ignorados/6novos/build519/native174 real com providers sintéticos. Não implementa restauro/ativação da época seguinte nem fecha adoção/entrega.
- [NEXT-EPOCH-DATA-RESTORE-PARTIAL-CHECKPOINT-R43F9B2C2.json](../.build/pet-goal-execution-next-r1/NEXT-EPOCH-DATA-RESTORE-PARTIAL-CHECKPOINT-R43F9B2C2.json) — 898/C2A4 dados2/Header3 inativo,2776Rust21ignorados/6novos/build519/nativo177 com providers sintéticos; processo de cópia próprio terminado/recolhido/retomado, compatibilidade anterior898, bytes/recibo/par e controlos recusados. Não é admissão3, falha física de energia ou entrega instalada.
- [MULTI-EPOCH-AUTHORITY-PARTIAL-CHECKPOINT-R43F9B2C3.json](../.build/pet-goal-execution-next-r1/MULTI-EPOCH-AUTHORITY-PARTIAL-CHECKPOINT-R43F9B2C3.json) — 901/C3A6:2795Rust21ignorados/19novos/build522/native179; cadeia/negação/consumo/originais/revisão e prefixo temporário sintético no CLI real. Header3 inativo, sem admissão3/entrega/humana ou provider real.
- [EPOCH-CONFIGURATION-PARTIAL-CHECKPOINT-R43F9B2D1.json](../.build/pet-goal-execution-next-r1/EPOCH-CONFIGURATION-PARTIAL-CHECKPOINT-R43F9B2D1.json) — 901/D1A1:2804Rust/21ignorados/9novos/build522/native180; revisão/recibo2 e hash da união de referências composto independentemente, providers sintéticos. Header3 inativo; não admissão3/entrega/humana ou provider real.
- [EPOCH-ADMISSION-PARTIAL-CHECKPOINT-R43F9B2D2.json](../.build/pet-goal-execution-next-r1/EPOCH-ADMISSION-PARTIAL-CHECKPOINT-R43F9B2D2.json) — 902/D2A4:2811Rust/21ignorados/7novos/build523/native182; admissão3/History2/SID/objetivo/Stop/reinício/ACK/source offline com providers sintéticos. Backup3/adoção/entrega e aceitação humana/real pendentes.
- [RECURSIVE-BACKUP-PARTIAL-CHECKPOINT-R43F9B2D3.json](../.build/pet-goal-execution-next-r1/RECURSIVE-BACKUP-PARTIAL-CHECKPOINT-R43F9B2D3.json) — 904/D3A5:2822Rust/21ignorados/11novos/build525/native184, backup3 e restauro/admissão bounded depth3 com providers sintéticos. NoAuthority/revalidações/B3/adoção/aceitação integral pendentes.
- [INDEPENDENT-AUTHORITY-PREPARATION-PARTIAL-CHECKPOINT-R43F9B2D4A.json](../.build/pet-goal-execution-next-r1/INDEPENDENT-AUTHORITY-PREPARATION-PARTIAL-CHECKPOINT-R43F9B2D4A.json) — 907/A5:2830Rust/21ignorados/8novos/build528/native185,0modelos/quota; apenas preparação de Authority independente, não admissão operacional. Prova negativa de regeneração antes da correção, cortes/tamper/negações/ACK; instalada/humana e D4B pendentes.
- [INDEPENDENT-ADMISSION-PARTIAL-CHECKPOINT-R43F9B2D4B.json](../.build/pet-goal-execution-next-r1/INDEPENDENT-ADMISSION-PARTIAL-CHECKPOINT-R43F9B2D4B.json) — 909/B5:2837Rust/21ignorados/7novos/build530/native188, providers sintéticos; mesmaKey/admission4/recibo3/novoSID+goal+decisão+Stop+restart+ACK+backup4. Novaépoca emfixtures. Geração incerta/revalidações/B3/adoção/aceitação integral pendentes.
- [MODEL-RESULT-SINK-PREFLIGHT-PARTIAL-CHECKPOINT-R43F9B2D4C.json](../.build/pet-goal-execution-next-r1/MODEL-RESULT-SINK-PREFLIGHT-PARTIAL-CHECKPOINT-R43F9B2D4C.json) — 910/C2:18dirigidos/2841Rust21ignorados/build531/native191; preflight sem envio/reserva, alteração durante metadata e falha posterior com reconciliação. Providers sintéticos,0quota/modelos reais, sem entrega/aceitação humana.
- [INDEPENDENT-RECOVERY-REVIEW-PARTIAL-CHECKPOINT-R43F9B2D4D1.json](../.build/pet-goal-execution-next-r1/INDEPENDENT-RECOVERY-REVIEW-PARTIAL-CHECKPOINT-R43F9B2D4D1.json) — 910/D4D1a:11dirigidos/2844Rust21ignorados/build531/native192,0modelos/quota/CLIagents. Parser comum/keyInvalid/manual;8casos semescrita,retomamesmachave,missing/readyrecusados. Não resolve retenção/quarentena/abandono,adoção/entrega.
- [INDEPENDENT-RETENTION-PARTIAL-CHECKPOINT-R43F9B2D4D2A.json](../.build/pet-goal-execution-next-r1/INDEPENDENT-RETENTION-PARTIAL-CHECKPOINT-R43F9B2D4D2A.json) — 913/D4D2A2:11dirigidos/2855Rust21ignorados/build534/native193,0modelos/quota/CLIagents/efeitos. Retenção nativa/sameinode/bytes/cadeia/retomaRecibo,blockedP1. Não é recuperação operacional/backup/adoção/entrega; D4D2B e limites exatos conjuntos pendentes.
- [RETAINED-ADMISSION-PARTIAL-CHECKPOINT-R43F9B2D4D2B.json](../.build/pet-goal-execution-next-r1/RETAINED-ADMISSION-PARTIAL-CHECKPOINT-R43F9B2D4D2B.json) — 914/D4D2B1:7dirigidos/2864Rust21ignorados/build535/native195;2HTTP1CLI sintéticos/0quota. Nova preparação independente,receipt4/Custody2/Admission4/OpenedACK/backup2,época seguinte fixture. Não é adoção/entrega/aceitação humana;49integrais abertos.
- [RESTORED-HISTORY-PARTIAL-CHECKPOINT-R43F9B3A.json](../.build/pet-goal-execution-next-r1/RESTORED-HISTORY-PARTIAL-CHECKPOINT-R43F9B3A.json) — 915/B3A2: prova negativa,2870Rust21ignorados/1256web/TSC/Vite/build536/native196,2HTTP1CLI sintéticos/0quota. História2/origemexata/totaldesconhecido/read-only antesIO,novoGoalhistória1. B3F2/adoção/entrega/humana/49integrais abertos.
- [RESTORED-FUTURE-PARTIAL-CHECKPOINT-R43F9B3F2A7.json](../.build/pet-goal-execution-next-r1/RESTORED-FUTURE-PARTIAL-CHECKPOINT-R43F9B3F2A7.json) — 923/A7:32dirigidos/2886Rust21ignorados/1262web/build542/native199; origem13 só dados + runtime comum/consentimento/Stop/reinício/backup; UI automatizada e2HTTP1CLI sintéticos/0quota,leitor915 real preserva13. Época seguinte fixture;49integrais/adoção/entrega abertos.
- [RETENTION-BOUNDARY-CHECKPOINT-R43F9B4A1.json](../.build/pet-goal-execution-next-r1/RETENTION-BOUNDARY-CHECKPOINT-R43F9B4A1.json) — 3casos/0ignorados,542inputs B4A1; limites conjuntos4096/64MiB aceites e+1 recusado,preservação de originais e sem autoridade. Somente testes mudaram; produçãoA7 preservada, sem nova alegação full/native/humana/instalada.
- [RESTORED-FEATURE-PARTIAL-CHECKPOINT-R43F10A4.json](../.build/pet-goal-execution-next-r1/RESTORED-FEATURE-PARTIAL-CHECKPOINT-R43F10A4.json) — 925/A4:200 negativo/201 positivo;81dirigidos/2892Rust21ignorados/1266web/build542. Recusa anteswrite/eventos,0600/restart/backup/erro executor eUI readback;HTTP2CLI1/DBus/HOME sintéticos/0quota;nenhuma revalidação concedida ou aceitação humana/instalada.
- [AUTOSTART-INTENT-PARTIAL-CHECKPOINT-R43F10B1A1.json](../.build/pet-goal-execution-next-r1/AUTOSTART-INTENT-PARTIAL-CHECKPOINT-R43F10B1A1.json) — 928/A1:6dirigidos/2898Rust21ignorados/1268web/build544/native202; intenção porcampo,duasjanelas e no-op/erro semreplay. HOME/HTTP/CLI/DBus privados;0quota. F10B2/revalidação/adoção/49integrais abertos.
- [SETTINGS-CONSUMERS-PARTIAL-CHECKPOINT-R43F10B2A3.json](../.build/pet-goal-execution-next-r1/SETTINGS-CONSUMERS-PARTIAL-CHECKPOINT-R43F10B2A3.json) — 931/A3:2905Rust21ignorados/build547/native203. Novecomandos Tauri sobfalha privada sem alteração em memória/evento;7novasfixtures deconsumidores/nomes. Web1268/TSC/Vite anterior reutilizado com inputs exatos.0quota. CAS/durabilidade/reconciliação/efeito listener e aceitação integral abertos.
- [SETTINGS-REVISION-PARTIAL-CHECKPOINT-R43F10B2B1.json](../.build/pet-goal-execution-next-r1/SETTINGS-REVISION-PARTIAL-CHECKPOINT-R43F10B2B1.json) — 934/B1/Linux:12novos/124selecionados1ignorado/2917full21ignorados/check/build550/native204. Revisão carregada,edição externa recusada antesdegravar/efeito,reinício privado/0600/backup eJSON preservado. HTTP/CLI/DBus sintéticos/0quota;web1268reutilizado. Não prova CAS atómico,barreira global/duráveldeincerteza ou entrega.

**Limite / pendência:** NoAuthority normal operacional privado comprovado; geração incerta/seed inválido/pre-manifest/staging anónimo ainda conservados/recusados até reconciliação explícita. Migrações/revalidações voz/mobile/autostart/B3-aceitação-integral/adoção e aceitação integral continuam abertas. F10A não implementa revalidação: faltam intenção por campo/CAS e confirmação após publicação/fsync diretório antes de a conceder. Consumidores não fecham escritor: CAS/fsyncdiretório/resultado incerto tipado/reconciliação/reinício pendentes. Listener ainda antes da gravação e chaves com efeitos próprios; não revalidados/entregues. Isto é verificação otimista para writers cooperantes, não CAS atómico/sandbox deoutros processos do mesmoUID. Incerteza pós-publicação tem classe/recusa de nova escrita,mas barreira global deefeitos/reconciliação durável/reinício/fsyncdiretório eparidade Mac/Windows ainda pendentes.

**Aceitação integral:** aberta. F10B2C: cercar todos novos efeitos com estado de IO do Shared atual, sem invalidar turnos capturados por simples mudança legítima de outro campo;barreira durável/reconciliação sem replay/fsyncdiretório e intenção do listener/chaves;depois revalidação/adoção/aceitação integral.

**Incremento939/C1:** origem atual de IO e preservação de Stop/recusa/evidência;14fixtures/2931full21ignorados/1272web/build552/native205 sintético. [Checkpoint](../.build/pet-goal-execution-next-r1/SETTINGS-EFFECT-GUARD-PARTIAL-CHECKPOINT-R43F10B2C1.json). Guardas finais e reconciliação durável pendentes, sem entrega integral.

**Incremento940/C2A3:** origem atual na escrita nativa final, controlo/recusa tipados e pre-spawn;21dirigidos/2952full21ignorados/build553/native206 com corte real de4096bytes num peer sintético. [Checkpoint](../.build/pet-goal-execution-next-r1/FINAL-NATIVE-DISPATCH-PARTIAL-CHECKPOINT-R43F10B2C2A3.json). C2B/C3 e aceitação integral continuam abertos.

- [local_tasks/executor.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/local_tasks/executor.rs) — `linux`
- [pet_native_write.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_native_write.rs) — `Outcome`, `Fault`

**Incremento942/C2B1A3:** IO nas fronteiras existentes de Rede, Som e DMS;15dirigidos/2967full21ignorados/build555/native208 com16recusas tipadas. [Checkpoint](../.build/pet-goal-execution-next-r1/SYSTEM-IO-PARTIAL-CHECKPOINT-R43F10B2C2B1A3.json). C2B2/C3 e aceitação integral continuam abertos.

- [pet_native_network_core/transport.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_native_network_core/transport.rs) — `WriteState`, `write_frame_checked`

**Incremento945/C2B2A2:** worker mantém Hold até recolha do grupo; controlo de Som→readback e DMS tipado no Tauri com peers privados.13dirigidos/2980full21ignorados/build558/native210. [Checkpoint](../.build/pet-goal-execution-next-r1/SYSTEM-HELPER-PARTIAL-CHECKPOINT-R43F10B2C2B2A2.json). C3/lifecycle/aceitação integral permanecem.

- [pet_helper.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_helper.rs) — `Lane`, `Session`, `supervise`
- [pet_process_group.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_process_group.rs) — `OwnedChild`, `finish_ready`, `group_has_live_members`

**Incremento948/C3A4:** intenção/resultado/revisão de preferências duráveis e formato de perfil/backup/restauro; 13dirigidos/2993full21ignorados/build561/native212. [Checkpoint](../.build/pet-goal-execution-next-r1/SETTINGS-DURABLE-PARTIAL-CHECKPOINT-R43F10B2C3A4.json). Reconciliação humana e listener/chaves ainda pendentes; sem adoção/instalação.

- [settings_durable.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings_durable.rs) — `Record`, `Observation`, `Prepared`, `restored_record`
- [settings_revision.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings_revision.rs) — `Bound`, `save_with`, `note_failed`
- [preferences_file.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/preferences_file.rs) — `commit_with`
- [pet_data/settings_io.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/settings_io.rs) — `Format`, `settings_io_protocol`
- [pet_data/publication.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication.rs) — `BackupOrigin`, `controls`
- [pet_data/restored_transaction.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_transaction.rs) — `capture`, `proposed`
- [private_registry.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/private_registry.rs) — `operation_lock`, `save_checked`

### DATA-02 — Decisão e eventual backend PostgreSQL

**Dependências e critérios preservados:** Benefício medido e esquema/base/utilizador próprios; IDs e concessões preservados, migração/restauro comprovados; estado atual continua sem integração

**Responsabilidade observada / bases a reutilizar:**

- [docs/AI-LOCAL-REUSE.md](../docs/AI-LOCAL-REUSE.md) — `Aproveitamento do AI Local nos pets`, `Resultado`
- [docs/PETS-ARCHITECTURE.md](../docs/PETS-ARCHITECTURE.md) — `Arquitetura dos pets autónomos de AI`, `R43F10B2B1 — revisão carregada antes de substituir preferências`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Apenas bases/histórico; não há aceitação integral específica deste requisito.

**Limite / pendência:** Não há medição que justifique adoção ou migração.

**Aceitação integral:** aberta. Medir antes de decidir; se adotado, base/role próprios e migração/restauro/rollback provados.

### REL-01 — Composição, adoção das fontes e migração/rollback

**Dependências e critérios preservados:** Fontes/hash exatos, preservação de dados/configuração/rascunhos e resolução de bloqueadores; journal v5 e marcadores pendentes tratados antes de regressar a leitores antigos

**Responsabilidade observada / bases a reutilizar:**

- [windows/src-tauri/src/pet_data/startup.rs](../windows/src-tauri/src/pet_data/startup.rs) — `Owner`, `name`, `local_address`
- [linux/migrate-pet-memory.py](../linux/migrate-pet-memory.py) — 
- [coucou-migration/MIGRACAO-E-ENTREGA.md](../coucou-migration/MIGRACAO-E-ENTREGA.md) — `Migração, compatibilidade e entrega`, `1. Evolução do fork e fronteiras`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/inventory.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/inventory.rs) — `legacy_history_backup`, `Area`, `Rule`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/tests.rs) — `Temp`, `new`, `layout`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/legacy_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/legacy_tests.rs) — `run`, `aged`, `sha`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/current_profile.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/current_profile.rs) — `Review`, `capture`, `fingerprint`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/backup_cli.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/backup_cli.rs) — `Request`, `parse`, `code`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/staging.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/staging.rs) — `Failure`, `Phase`, `Inspection`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication.rs) — `Header`, `Complete`, `Admission`
- [windows/src/core/session-drafts.ts](../windows/src/core/session-drafts.ts) — `draftTarget`, `targetFromKey`, `parseStoredDraft`
- [windows/src/desktop/pet-chat.ts](../windows/src/desktop/pet-chat.ts) — `acceptRect`, `fromLocal`, `report`
- [windows/src/views/chat.ts](../windows/src/views/chat.ts) — `watchCapabilities`, `capabilityText`, `bubble`
- [docs/PETS-DATA-BACKUP.md](../docs/PETS-DATA-BACKUP.md) — `Backup nativo — uso e cobertura`, `R43F10B2B1 — revisão carregada antes de substituir preferências`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/mod.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/mod.rs) — `allowed`, `path`, `current`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/model.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/model.rs) — `Record`, `validate`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/store.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/store.rs) — `read`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/tests.rs) — `Files`, `new`, `path`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/native-drafts.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/native-drafts.ts) — `text`, `draftWireTarget`, `nativeDraftTarget`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/draft-migration.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/draft-migration.ts) — `draftImportReceipt`, `canonicalLegacyBatch`, `migration`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/inline-drafts.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/inline-drafts.ts) — `InlineDrafts`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/draft-status.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/draft-status.ts) — `draftStatusControl`, `sync`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/chat.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/chat.ts) — `watchCapabilities`, `capabilityText`, `bubble`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/pet-factory.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/pet-factory.ts) — `invalid`, `merge`, `freeze`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/fixtures/reader_v1.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/fixtures/reader_v1.rs) — `Target`, `Origin`, `id`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/recovery.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/recovery.rs) — `Owner`, `validate`, `Availability`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/recovery_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/recovery_tests.rs) — `Files`, `new`, `path`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/draft-recovery.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/draft-recovery.ts) — `DraftRecoveryFailure`, `invalid`, `archive`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/draft-recovery.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/draft-recovery.ts) — `draftRecoveryControl`, `sync`, `capture`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/archives.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/archives.rs) — `Archive`, `empty`, `validate`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/reconcile.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/reconcile.rs) — `Policy`, `Summary`, `Review`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/reconcile_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/reconcile_tests.rs) — `Files`, `new`, `path`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/fixtures/reader_v2.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_drafts/fixtures/reader_v2.rs) — `Target`, `Origin`, `id`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/draft-reconcile.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/core/draft-reconcile.ts) — `DraftReconcileFailure`, `invalid`, `wire`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/draft-reconcile.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/draft-reconcile.ts) — `draftReconcileControl`, `sync`, `capture`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore.rs) — `Intent`, `Restored`, `Mode`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_tests.rs) — `Fixture`, `new`, `layout`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restore_cli.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restore_cli.rs) — `activation_sealed`, `execute`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/bootstrap.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/bootstrap.rs) — `Session`, `prepare`, `paths`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration.rs) — `FileRef`, `HistoricalGrant`, `GoalRef`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_tests.rs) — `clone_dir`, `copy_fixture`, `review`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_authority.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_authority.rs) — `error`, `AuthorityReview`, `FirstReview`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_configuration.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_configuration.rs) — `configuration`, `epoch_scope`, `verify_payload`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_configuration_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_configuration_tests.rs) — `settings`, `pets`, `Fixture`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_config.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_config.rs) — `select`, `retired`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_history.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_history.rs) — `capture_next`, `decode_first`, `review`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/chat_restored_history_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/chat_restored_history_tests.rs) — `fixture`, `historical_conversation_is_readable_with_notice_but_cannot_reserve_or_start_a_turn`, `a_fresh_conversation_can_obtain_normal_reservation_while_old_history_is_preserved`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/chat.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/chat.rs) — `Chat`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_transaction.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_transaction.rs) — `Activation`, `initialization_delta`, `EpochInitialization`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_projection.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_projection.rs) — `Choices`, `Grant`, `Operation`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_active_restored.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_active_restored.rs) — `Bound`, `verify_initial_payload`, `publish`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_transaction_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_transaction_tests.rs) — `Fixture`, `new`, `with_authority`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restored_backup.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restored_backup.rs) — `cannot`, `verify`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_epochs.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_epochs.rs) — `EpochStep`, `EpochReview`, `prior_staging`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_epoch.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_epoch.rs) — `capture_parts`, `completed`, `verify_current`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/revision_fence.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/revision_fence.rs) — `Tree`, `descriptor_budget`, `capture`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_backup_epoch_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_backup_epoch_tests.rs) — `Round`, `new`, `archive`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent.rs) — `preparation_state`, `preview`, `apply_using`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_independent.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_independent.rs) — `IndependentKeyState`, `key_state`, `prepare`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent_operational.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent_operational.rs) — `Custody`, `validate`, `PreparedAuthority`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_independent_transaction_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_independent_transaction_tests.rs) — `prepared`, `old_goal`, `independent_admission4_preserves_absent_original_authority_and_denies_old_ids_before_new_consent`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/writer.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/writer.rs) — `model_once`, `authorize_body`, `record_result`, `before_send`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/results.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/results.rs) — `preflight`, `lookup`, `publish`, `recover`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/result_sink_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/result_sink_tests.rs) — `bytes`, `visit`, `incompatible`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_store.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_store.rs) — `KeyFile`, `parse`, `restoration_base`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_independent_tests.rs) — `Fixture`, `tree`, `visit`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_retention.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_retention.rs) — `RetentionReview`, `scan`, `current`, `apply_using`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_retention_inventory.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_retention_inventory.rs) — `Opaque`, `Budget`, `inspect`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_retention_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/publication_restore_retention_tests.rs) — `review`, `job`, `opaque`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/staging_fs.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/staging_fs.rs) — `transfer_dir`, `publish_dir`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_retained_independent_transaction_tests.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/restored_retained_independent_transaction_tests.rs) — `data`, `retain`, `prepare`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_history.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restoration_history.rs) — `goal_snapshot`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/history.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/history.rs) — `RestoredSource`, `Snapshot`, `empty`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_history.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_history.rs) — `phase`, `history_inventory`, `history`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/runtime-goals.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/runtime-goals.ts) — `buildRuntimeGoals`, `syncMusic`, `syncPets`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/restored_future.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/restored_future.rs) — `Review`, `Origin`, `valid_for`, `request_id`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restored_future.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/authority_restored_future.rs) — `Witness`, `capture`, `current`, `verify_restored_origin`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/store_restored_future.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/store_restored_future.rs) — `existing_restored_future`, `create_restored_future_clock`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/restored_future_api.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_runtime/restored_future_api.rs) — `pet_runtime_prepare_restored_future`, `pet_runtime_confirm_restored_future`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/restored-future-review.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/views/restored-future-review.ts) — `buildRestoredFutureReview`
- [.build/pet-goal-execution-next-r1/retention-boundary-r43f9b4a1-source/windows/src-tauri/src/pet_data/publication_restore_retention_tests.rs](../.build/pet-goal-execution-next-r1/retention-boundary-r43f9b4a1-source/windows/src-tauri/src/pet_data/publication_restore_retention_tests.rs) — `review`, `job`, `opaque`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings.rs) — `update_checked_with`, `save_to`, `read_initialized`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_autostart.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_autostart.rs) — `admit`, `set`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/lib.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/lib.rs) — `pet_draft_mode`, `Shared`, `BootInfo`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/settings/autostart-setting.ts](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src/settings/autostart-setting.ts) — `buildAutostartSetting`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_autostart_settings.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_autostart_settings.rs) — `Patch`, `Controller`, `Receipt`, `preserve_incoming`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/phone_link/mod.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/phone_link/mod.rs) — `commit_preferences`, `Keystore`, `get`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/phone_link/preferences.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/phone_link/preferences.rs) — `Change`, `commit_with`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/voice/session.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/voice/session.rs) — `commit_alias_with`, `active`, `suspend_hotword`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/private_registry.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/private_registry.rs) — `load_at`, `save_guarded`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/preferences_file.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/preferences_file.rs) — `Witness`, `read`, `current`, `commit`, `unconfirmed`
- [.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings_revision.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings_revision.rs) — `Revision`, `load_from`, `bind_strict`, `save_to`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Relatos/checkpoints por composição; as provas parciais não cobrem todos os critérios deste ID.
- [MEMORY-MIGRATION-PARTIAL-CHECKPOINT-R43F4.json](../.build/pet-goal-execution-next-r1/MEMORY-MIGRATION-PARTIAL-CHECKPOINT-R43F4.json) — SDK854/F4:3 fontes nativas alteradas; baseline antes da correção1passa/2falham; depois2641 Rust/21 ignorados,check/build e Tauri147. Native146/147 prova retoma selecionada,2rondas com2aprovações,7HTTP sintéticos,revogação/reinício e recusa do leitor844. Não certifica backup completo, integração/instalação ou aceitação humana. Timeout145 preservado com causa desconhecida.
- [CURRENT-PROFILE-BACKUP-PARTIAL-CHECKPOINT-R43F5.json](../.build/pet-goal-execution-next-r1/CURRENT-PROFILE-BACKUP-PARTIAL-CHECKPOINT-R43F5.json) — SDK858/F5D:2655Rust/21ignorados,23dirigidos/1ignorado,check-tests/build e composição151.149 prova CLI81ficheiros/integridade/owner/consentimento/recusas;151 arranque normal/continuação7HTTP sintéticos e backup do mesmo perfil.150 Busy anterior retido;D repete só controlo anterior à escrita, nunca IO físico/modelo, com Stop e limite. Sem restauro/instalação/aceitação humana ou quota real.
- [NATIVE-DRAFTS-PARTIAL-CHECKPOINT-R43F6.json](../.build/pet-goal-execution-next-r1/NATIVE-DRAFTS-PARTIAL-CHECKPOINT-R43F6.json) — SDK865/F6D: native draft repository/UTF16/u64/CAS, known IndexedDB migration and retained empty CID; full Rust,1219JS/types/build and native154 with0models/quota. Inline/divergence/orphan UI/restore/adoption/human/installed acceptance remain open.
- [INLINE-DRAFTS-PARTIAL-CHECKPOINT-R43F7.json](../.build/pet-goal-execution-next-r1/INLINE-DRAFTS-PARTIAL-CHECKPOINT-R43F7.json) — SDK869/F7A:2674Rust/21ignorados/check/build,1230JS/types/bundle e nativo158 de cartões autorizados. Inline/unbound/CAS/cópia explícita, leitor antigo/backup;0modelos/quota. Digitação nativa inicial, pre-CID não nativo, órfãos/divergência/restauro/entrega ainda abertos.
- [ORPHAN-DRAFT-RECOVERY-PARTIAL-CHECKPOINT-R43F8.json](../.build/pet-goal-execution-next-r1/ORPHAN-DRAFT-RECOVERY-PARTIAL-CHECKPOINT-R43F8.json) — SDK874/F8A:2684Rust/21ignorados/check/build,1240JS/types/bundle e nativo160 com0modelos/quota. Órfãos/cópia humana scriptada/origem conservada, inline/flutuante/replay/conflito/leitor2/backup. Não reconcilia IndexedDB divergente nem fecha restauro/adoção/aceitação integral.
- [DIVERGENT-DRAFT-RECONCILIATION-PARTIAL-CHECKPOINT-R43F8B.json](../.build/pet-goal-execution-next-r1/DIVERGENT-DRAFT-RECONCILIATION-PARTIAL-CHECKPOINT-R43F8B.json) — SDK882/F8B2:2700Rust/21ignorados/check503/build;1255JS/TSC/Vite emB1 idêntico na produção/B2 e binário byte idêntico. Nativo162 real/IndexedDB/scriptado,0modelos/quota: reconciliação, arquivo/CAS/ACK/lostACK sem replay, recuperação/cópia, restart/leitor2/backup. Sem restauro/adoção/timing de mutação durante IO nativo ou aceitação humana/instalada.
- [CURRENT-PROFILE-RESTORE-PARTIAL-CHECKPOINT-R43F9.json](../.build/pet-goal-execution-next-r1/CURRENT-PROFILE-RESTORE-PARTIAL-CHECKPOINT-R43F9.json) — SDK885/F9A:2713Rust/21ignorados/check506/build;web1255/TSC/Vite reutilizada com bytes não Rust idênticos.163CLI/IPC real/private0models/quota: publicação revista/dataRestoredInactive, par/origem/bytes exatos, crash próprio/retoma/subset divergente/ACK idempotente e leitores recusam. Não é admissão/autoridade executável, semântica de assinatura/contabilidade/continuação ou aceitação humana/instalada.
- [RESTORED-AUTHORITY-PARTIAL-CHECKPOINT-R43F9B1.json](../.build/pet-goal-execution-next-r1/RESTORED-AUTHORITY-PARTIAL-CHECKPOINT-R43F9B1.json) — SDK888/F9B1C:2725Rust/21ignorados/12novos/build509;web1255 fontes idênticas;165HMAC/CLI/IPC real sem inferência/quota. Quarentena histórica/assinaturas/bytes/ACK/tamper/leitores; novo claim e consumo incerto em Rust. Header2 inativo, F9B2/F9B3/adoção/aceitação integral pendentes.
- [RESTORED-CONFIGURATION-PARTIAL-CHECKPOINT-R43F9B2A.json](../.build/pet-goal-execution-next-r1/RESTORED-CONFIGURATION-PARTIAL-CHECKPOINT-R43F9B2A.json) — SDK890/B2A5:2737Rust/21ignorados/12novos/build511/nativo168 real/CLI/IPC/HMAC sem inferência/quota; revisão tipada/inspectionOnly/bytes/ACK e leitores. Não é ativação, admissão operacional, continuação ou aceitação humana/instalada.
- [RESTORED-SESSION-GUARDS-PARTIAL-CHECKPOINT-R43F9B2B1.json](../.build/pet-goal-execution-next-r1/RESTORED-SESSION-GUARDS-PARTIAL-CHECKPOINT-R43F9B2B1.json) — SDK893/B2B1B:2754Rust/21ignorados/17novos/build514;169 regressão real Root1/claim/backup/B1/B2A,0inferência/quota. Guards em fixtures; NÃO restauro operacional/admissão/guard nativo ativo ou aceitação humana/instalada.
- [RESTORED-ACTIVATION-PARTIAL-CHECKPOINT-R43F9B2B2.json](../.build/pet-goal-execution-next-r1/RESTORED-ACTIVATION-PARTIAL-CHECKPOINT-R43F9B2B2.json) — 897/B2B2C1 primeira ativação operacional privada;2764Rust21ignorados/10novos/build518/nativo173 com providers sintéticos. Não fecha backup Root2/épocas/revalidações/B3/adoção/aceitação humana ou instalada.
- [PORTABLE-RESTORED-BACKUP-PARTIAL-CHECKPOINT-R43F9B2C.json](../.build/pet-goal-execution-next-r1/PORTABLE-RESTORED-BACKUP-PARTIAL-CHECKPOINT-R43F9B2C.json) — 898/C1A1 backup/inspeção de ambos roots completos;2770Rust21ignorados/6novos/build519/native174 real com providers sintéticos. Não implementa restauro/ativação da época seguinte nem fecha adoção/entrega.
- [NEXT-EPOCH-DATA-RESTORE-PARTIAL-CHECKPOINT-R43F9B2C2.json](../.build/pet-goal-execution-next-r1/NEXT-EPOCH-DATA-RESTORE-PARTIAL-CHECKPOINT-R43F9B2C2.json) — 898/C2A4 dados2/Header3 inativo,2776Rust21ignorados/6novos/build519/nativo177 com providers sintéticos; processo de cópia próprio terminado/recolhido/retomado, compatibilidade anterior898, bytes/recibo/par e controlos recusados. Não é admissão3, falha física de energia ou entrega instalada.
- [MULTI-EPOCH-AUTHORITY-PARTIAL-CHECKPOINT-R43F9B2C3.json](../.build/pet-goal-execution-next-r1/MULTI-EPOCH-AUTHORITY-PARTIAL-CHECKPOINT-R43F9B2C3.json) — 901/C3A6:2795Rust21ignorados/19novos/build522/native179; cadeia/negação/consumo/originais/revisão e prefixo temporário sintético no CLI real. Header3 inativo, sem admissão3/entrega/humana ou provider real.
- [EPOCH-CONFIGURATION-PARTIAL-CHECKPOINT-R43F9B2D1.json](../.build/pet-goal-execution-next-r1/EPOCH-CONFIGURATION-PARTIAL-CHECKPOINT-R43F9B2D1.json) — 901/D1A1:2804Rust/21ignorados/9novos/build522/native180; revisão/recibo2 e hash da união de referências composto independentemente, providers sintéticos. Header3 inativo; não admissão3/entrega/humana ou provider real.
- [EPOCH-ADMISSION-PARTIAL-CHECKPOINT-R43F9B2D2.json](../.build/pet-goal-execution-next-r1/EPOCH-ADMISSION-PARTIAL-CHECKPOINT-R43F9B2D2.json) — 902/D2A4:2811Rust/21ignorados/7novos/build523/native182; admissão3/History2/SID/objetivo/Stop/reinício/ACK/source offline com providers sintéticos. Backup3/adoção/entrega e aceitação humana/real pendentes.
- [RECURSIVE-BACKUP-PARTIAL-CHECKPOINT-R43F9B2D3.json](../.build/pet-goal-execution-next-r1/RECURSIVE-BACKUP-PARTIAL-CHECKPOINT-R43F9B2D3.json) — 904/D3A5:2822Rust/21ignorados/11novos/build525/native184, backup3 e restauro/admissão bounded depth3 com providers sintéticos. NoAuthority/revalidações/B3/adoção/aceitação integral pendentes.
- [INDEPENDENT-AUTHORITY-PREPARATION-PARTIAL-CHECKPOINT-R43F9B2D4A.json](../.build/pet-goal-execution-next-r1/INDEPENDENT-AUTHORITY-PREPARATION-PARTIAL-CHECKPOINT-R43F9B2D4A.json) — 907/A5:2830Rust/21ignorados/8novos/build528/native185,0modelos/quota; apenas preparação de Authority independente, não admissão operacional. Prova negativa de regeneração antes da correção, cortes/tamper/negações/ACK; instalada/humana e D4B pendentes.
- [INDEPENDENT-ADMISSION-PARTIAL-CHECKPOINT-R43F9B2D4B.json](../.build/pet-goal-execution-next-r1/INDEPENDENT-ADMISSION-PARTIAL-CHECKPOINT-R43F9B2D4B.json) — 909/B5:2837Rust/21ignorados/7novos/build530/native188, providers sintéticos; mesmaKey/admission4/recibo3/novoSID+goal+decisão+Stop+restart+ACK+backup4. Novaépoca emfixtures. Geração incerta/revalidações/B3/adoção/aceitação integral pendentes.
- [MODEL-RESULT-SINK-PREFLIGHT-PARTIAL-CHECKPOINT-R43F9B2D4C.json](../.build/pet-goal-execution-next-r1/MODEL-RESULT-SINK-PREFLIGHT-PARTIAL-CHECKPOINT-R43F9B2D4C.json) — 910/C2:18dirigidos/2841Rust21ignorados/build531/native191; preflight sem envio/reserva, alteração durante metadata e falha posterior com reconciliação. Providers sintéticos,0quota/modelos reais, sem entrega/aceitação humana.
- [INDEPENDENT-RECOVERY-REVIEW-PARTIAL-CHECKPOINT-R43F9B2D4D1.json](../.build/pet-goal-execution-next-r1/INDEPENDENT-RECOVERY-REVIEW-PARTIAL-CHECKPOINT-R43F9B2D4D1.json) — 910/D4D1a:11dirigidos/2844Rust21ignorados/build531/native192,0modelos/quota/CLIagents. Parser comum/keyInvalid/manual;8casos semescrita,retomamesmachave,missing/readyrecusados. Não resolve retenção/quarentena/abandono,adoção/entrega.
- [INDEPENDENT-RETENTION-PARTIAL-CHECKPOINT-R43F9B2D4D2A.json](../.build/pet-goal-execution-next-r1/INDEPENDENT-RETENTION-PARTIAL-CHECKPOINT-R43F9B2D4D2A.json) — 913/D4D2A2:11dirigidos/2855Rust21ignorados/build534/native193,0modelos/quota/CLIagents/efeitos. Retenção nativa/sameinode/bytes/cadeia/retomaRecibo,blockedP1. Não é recuperação operacional/backup/adoção/entrega; D4D2B e limites exatos conjuntos pendentes.
- [RETAINED-ADMISSION-PARTIAL-CHECKPOINT-R43F9B2D4D2B.json](../.build/pet-goal-execution-next-r1/RETAINED-ADMISSION-PARTIAL-CHECKPOINT-R43F9B2D4D2B.json) — 914/D4D2B1:7dirigidos/2864Rust21ignorados/build535/native195;2HTTP1CLI sintéticos/0quota. Nova preparação independente,receipt4/Custody2/Admission4/OpenedACK/backup2,época seguinte fixture. Não é adoção/entrega/aceitação humana;49integrais abertos.
- [RESTORED-HISTORY-PARTIAL-CHECKPOINT-R43F9B3A.json](../.build/pet-goal-execution-next-r1/RESTORED-HISTORY-PARTIAL-CHECKPOINT-R43F9B3A.json) — 915/B3A2: prova negativa,2870Rust21ignorados/1256web/TSC/Vite/build536/native196,2HTTP1CLI sintéticos/0quota. História2/origemexata/totaldesconhecido/read-only antesIO,novoGoalhistória1. B3F2/adoção/entrega/humana/49integrais abertos.
- [RESTORED-FUTURE-PARTIAL-CHECKPOINT-R43F9B3F2A7.json](../.build/pet-goal-execution-next-r1/RESTORED-FUTURE-PARTIAL-CHECKPOINT-R43F9B3F2A7.json) — 923/A7:32dirigidos/2886Rust21ignorados/1262web/build542/native199; origem13 só dados + runtime comum/consentimento/Stop/reinício/backup; UI automatizada e2HTTP1CLI sintéticos/0quota,leitor915 real preserva13. Época seguinte fixture;49integrais/adoção/entrega abertos.
- [RETENTION-BOUNDARY-CHECKPOINT-R43F9B4A1.json](../.build/pet-goal-execution-next-r1/RETENTION-BOUNDARY-CHECKPOINT-R43F9B4A1.json) — 3casos/0ignorados,542inputs B4A1; limites conjuntos4096/64MiB aceites e+1 recusado,preservação de originais e sem autoridade. Somente testes mudaram; produçãoA7 preservada, sem nova alegação full/native/humana/instalada.
- [RESTORED-FEATURE-PARTIAL-CHECKPOINT-R43F10A4.json](../.build/pet-goal-execution-next-r1/RESTORED-FEATURE-PARTIAL-CHECKPOINT-R43F10A4.json) — 925/A4:200 negativo/201 positivo;81dirigidos/2892Rust21ignorados/1266web/build542. Recusa anteswrite/eventos,0600/restart/backup/erro executor eUI readback;HTTP2CLI1/DBus/HOME sintéticos/0quota;nenhuma revalidação concedida ou aceitação humana/instalada.
- [AUTOSTART-INTENT-PARTIAL-CHECKPOINT-R43F10B1A1.json](../.build/pet-goal-execution-next-r1/AUTOSTART-INTENT-PARTIAL-CHECKPOINT-R43F10B1A1.json) — 928/A1:6dirigidos/2898Rust21ignorados/1268web/build544/native202; intenção porcampo,duasjanelas e no-op/erro semreplay. HOME/HTTP/CLI/DBus privados;0quota. F10B2/revalidação/adoção/49integrais abertos.
- [SETTINGS-CONSUMERS-PARTIAL-CHECKPOINT-R43F10B2A3.json](../.build/pet-goal-execution-next-r1/SETTINGS-CONSUMERS-PARTIAL-CHECKPOINT-R43F10B2A3.json) — 931/A3:2905Rust21ignorados/build547/native203. Novecomandos Tauri sobfalha privada sem alteração em memória/evento;7novasfixtures deconsumidores/nomes. Web1268/TSC/Vite anterior reutilizado com inputs exatos.0quota. CAS/durabilidade/reconciliação/efeito listener e aceitação integral abertos.
- [SETTINGS-REVISION-PARTIAL-CHECKPOINT-R43F10B2B1.json](../.build/pet-goal-execution-next-r1/SETTINGS-REVISION-PARTIAL-CHECKPOINT-R43F10B2B1.json) — 934/B1/Linux:12novos/124selecionados1ignorado/2917full21ignorados/check/build550/native204. Revisão carregada,edição externa recusada antesdegravar/efeito,reinício privado/0600/backup eJSON preservado. HTTP/CLI/DBus sintéticos/0quota;web1268reutilizado. Não prova CAS atómico,barreira global/duráveldeincerteza ou entrega.

**Limite / pendência:** NoAuthority normal operacional privado comprovado; geração incerta/seed inválido/pre-manifest/staging anónimo ainda conservados/recusados até reconciliação explícita. Migrações/revalidações voz/mobile/autostart/B3-aceitação-integral/adoção e aceitação integral continuam abertas. F10A não implementa revalidação: faltam intenção por campo/CAS e confirmação após publicação/fsync diretório antes de a conceder. Consumidores não fecham escritor: CAS/fsyncdiretório/resultado incerto tipado/reconciliação/reinício pendentes. Listener ainda antes da gravação e chaves com efeitos próprios; não revalidados/entregues. Isto é verificação otimista para writers cooperantes, não CAS atómico/sandbox deoutros processos do mesmoUID. Incerteza pós-publicação tem classe/recusa de nova escrita,mas barreira global deefeitos/reconciliação durável/reinício/fsyncdiretório eparidade Mac/Windows ainda pendentes.

**Aceitação integral:** aberta. F10B2C: cercar todos novos efeitos com estado de IO do Shared atual, sem invalidar turnos capturados por simples mudança legítima de outro campo;barreira durável/reconciliação sem replay/fsyncdiretório e intenção do listener/chaves;depois revalidação/adoção/aceitação integral.

**Incremento939/C1:** origem atual de IO e preservação de Stop/recusa/evidência;14fixtures/2931full21ignorados/1272web/build552/native205 sintético. [Checkpoint](../.build/pet-goal-execution-next-r1/SETTINGS-EFFECT-GUARD-PARTIAL-CHECKPOINT-R43F10B2C1.json). Guardas finais e reconciliação durável pendentes, sem entrega integral.

**Incremento940/C2A3:** origem atual na escrita nativa final, controlo/recusa tipados e pre-spawn;21dirigidos/2952full21ignorados/build553/native206 com corte real de4096bytes num peer sintético. [Checkpoint](../.build/pet-goal-execution-next-r1/FINAL-NATIVE-DISPATCH-PARTIAL-CHECKPOINT-R43F10B2C2A3.json). C2B/C3 e aceitação integral continuam abertos.

- [pet_native_write.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_native_write.rs) — `Guard`, `frame`

**Incremento942/C2B1A3:** IO nas fronteiras existentes de Rede, Som e DMS;15dirigidos/2967full21ignorados/build555/native208 com16recusas tipadas. [Checkpoint](../.build/pet-goal-execution-next-r1/SYSTEM-IO-PARTIAL-CHECKPOINT-R43F10B2C2B1A3.json). C2B2/C3 e aceitação integral continuam abertos.

- [pet_audio/linux.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_audio/linux.rs) — `run_checked`
- [pet_system.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_system.rs) — `dms`

**Incremento945/C2B2A2:** worker mantém Hold até recolha do grupo; controlo de Som→readback e DMS tipado no Tauri com peers privados.13dirigidos/2980full21ignorados/build558/native210. [Checkpoint](../.build/pet-goal-execution-next-r1/SYSTEM-HELPER-PARTIAL-CHECKPOINT-R43F10B2C2B2A2.json). C3/lifecycle/aceitação integral permanecem.

- [pet_helper.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_helper.rs) — `Lane`, `Session`, `supervise`
- [pet_process_group.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_process_group.rs) — `OwnedChild`, `finish_ready`, `group_has_live_members`

**Incremento948/C3A4:** intenção/resultado/revisão de preferências duráveis e formato de perfil/backup/restauro; 13dirigidos/2993full21ignorados/build561/native212. [Checkpoint](../.build/pet-goal-execution-next-r1/SETTINGS-DURABLE-PARTIAL-CHECKPOINT-R43F10B2C3A4.json). Reconciliação humana e listener/chaves ainda pendentes; sem adoção/instalação.

- [settings_durable.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings_durable.rs) — `Record`, `Observation`, `Prepared`, `restored_record`
- [settings_revision.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/settings_revision.rs) — `Bound`, `save_with`, `note_failed`
- [preferences_file.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/preferences_file.rs) — `commit_with`
- [pet_data/bootstrap.rs](../.build/pet-goal-execution-next-r1/settings-review-r43f10b2c3b1b4-source/windows/src-tauri/src/pet_data/bootstrap.rs) — `settings_io_observation`, `install`

### REL-02 — Entrega e aceitação integrada por capacidades

**Dependências e critérios preservados:** Executável/assets/helper/autostart/login/reinício, hotplug e trabalhos interrompidos; paridade funcional dos centros legados antes de descontinuar capacidades, mantendo os indicadores. Rádio/dispositivos/contas respeitam o âmbito autorizado

**Responsabilidade observada / bases a reutilizar:**

- [linux/tauri.fork.conf.json](../linux/tauri.fork.conf.json) — `definição declarativa; sem inferência de prontidão`
- [docs/PETS-IMPLEMENTATION-STATUS.md](../docs/PETS-IMPLEMENTATION-STATUS.md) — `Estado de implementação dos pets`, `R43F10B2B1 — revisão carregada antes de substituir preferências`

**Prova disponível e alcance:**

- [PETS-VALIDATION.md](../docs/PETS-VALIDATION.md) — Relatos/checkpoints por composição; as provas parciais não cobrem todos os critérios deste ID.

**Limite / pendência:** Autostart/login/hotplug/lifecycle/áudio e aceitação integrada da nova composição pendentes.

**Aceitação integral:** aberta. Entregar somente no âmbito autorizado, provar comportamento real e preservar legados.

### GUA-01 — Inventário limitado, evidência e classificação de gestão

**Dependências e critérios preservados:** PET-01/04/05, EXE-01/02, OBS-01, SYS-01. Componentes/versões/serviços/processos/exposição/recursos/logs selecionados e estado desejado, com origem/data/cobertura e distinção gerido/observado/externo/privado/dinâmico; percurso determinístico sem LLM e sem varrimento pessoal indiscriminado

**Responsabilidade observada / bases a reutilizar:**

- [docs/SYSTEM-GUARDIAN-ARCHITECTURE.md](../docs/SYSTEM-GUARDIAN-ARCHITECTURE.md) — `System Guardian — especialização do runtime comum`, `1. Identidade, motor, runtime e papel`
- [windows/src-tauri/src/pet_factory_runtime.rs](../windows/src-tauri/src/pet_factory_runtime.rs) — `parse`, `recognizes_owner`
- [windows/src-tauri/src/pet_runtime/model.rs](../windows/src-tauri/src/pet_runtime/model.rs) — `Record`, `Owner`, `validate`

**Prova disponível e alcance:**

- [COUCOU-GUARDIAN-CONTRACT-REGISTRATION-20261025.json](../.build/COUCOU-GUARDIAN-CONTRACT-REGISTRATION-20261025.json) — Contrato, IDs e ausência de ativação; nenhuma prova de execução Guardian.

**Limite / pendência:** Ainda não há operações, UI funcional, fixtures de diagnóstico ou aceitação Guardian; nenhum acesso/mutação do anfitrião foi concedido.

**Aceitação integral:** aberta. PET-01/04/05, EXE-01/02, OBS-01, SYS-01. Componentes/versões/serviços/processos/exposição/recursos/logs selecionados e estado desejado, com origem/data/cobertura e distinção gerido/observado/externo/privado/dinâmico; percurso determinístico sem LLM e sem varrimento pessoal indiscriminado

### GUA-02 — Findings de segurança e vulnerabilidades

**Dependências e critérios preservados:** GUA-01, CTX-05, EXE-02/03, DATA-01. Componente/versão, evidência/origem/data, severidade/confiança/estado/recomendação; distinguir confirmação/candidata/suspeita, conferir advisories da distribuição e pacote; não inventar CVE, inferir vulnerabilidade só por upstream ou executar exploração/scans intrusivos

**Responsabilidade observada / bases a reutilizar:**

- [docs/SYSTEM-GUARDIAN-ARCHITECTURE.md](../docs/SYSTEM-GUARDIAN-ARCHITECTURE.md) — `System Guardian — especialização do runtime comum`, `1. Identidade, motor, runtime e papel`
- [windows/src-tauri/src/pet_factory_runtime.rs](../windows/src-tauri/src/pet_factory_runtime.rs) — `parse`, `recognizes_owner`
- [windows/src-tauri/src/pet_runtime/model.rs](../windows/src-tauri/src/pet_runtime/model.rs) — `Record`, `Owner`, `validate`

**Prova disponível e alcance:**

- [COUCOU-GUARDIAN-CONTRACT-REGISTRATION-20261025.json](../.build/COUCOU-GUARDIAN-CONTRACT-REGISTRATION-20261025.json) — Contrato, IDs e ausência de ativação; nenhuma prova de execução Guardian.

**Limite / pendência:** Ainda não há operações, UI funcional, fixtures de diagnóstico ou aceitação Guardian; nenhum acesso/mutação do anfitrião foi concedido.

**Aceitação integral:** aberta. GUA-01, CTX-05, EXE-02/03, DATA-01. Componente/versão, evidência/origem/data, severidade/confiança/estado/recomendação; distinguir confirmação/candidata/suspeita, conferir advisories da distribuição e pacote; não inventar CVE, inferir vulnerabilidade só por upstream ou executar exploração/scans intrusivos

### GUA-03 — Saúde/desempenho com baseline e resultado

**Dependências e critérios preservados:** GUA-01, OBS-01/02, SYS-03. CPU/memória/IO/GPU suportada/arranque/falhas/conflitos, hipótese/métrica/impacto/custos e validação; fixtures de consumo, dock Thunderbolt e VPN/Tailscale/firewall sem mutações reais, tuning genérico, limpeza destrutiva ou desativação de proteções

**Responsabilidade observada / bases a reutilizar:**

- [docs/SYSTEM-GUARDIAN-ARCHITECTURE.md](../docs/SYSTEM-GUARDIAN-ARCHITECTURE.md) — `System Guardian — especialização do runtime comum`, `1. Identidade, motor, runtime e papel`
- [windows/src-tauri/src/pet_factory_runtime.rs](../windows/src-tauri/src/pet_factory_runtime.rs) — `parse`, `recognizes_owner`
- [windows/src-tauri/src/pet_runtime/model.rs](../windows/src-tauri/src/pet_runtime/model.rs) — `Record`, `Owner`, `validate`

**Prova disponível e alcance:**

- [COUCOU-GUARDIAN-CONTRACT-REGISTRATION-20261025.json](../.build/COUCOU-GUARDIAN-CONTRACT-REGISTRATION-20261025.json) — Contrato, IDs e ausência de ativação; nenhuma prova de execução Guardian.

**Limite / pendência:** Ainda não há operações, UI funcional, fixtures de diagnóstico ou aceitação Guardian; nenhum acesso/mutação do anfitrião foi concedido.

**Aceitação integral:** aberta. GUA-01, OBS-01/02, SYS-03. CPU/memória/IO/GPU suportada/arranque/falhas/conflitos, hipótese/métrica/impacto/custos e validação; fixtures de consumo, dock Thunderbolt e VPN/Tailscale/firewall sem mutações reais, tuning genérico, limpeza destrutiva ou desativação de proteções

### GUA-04 — Adaptador governado ao sysworkspace

**Dependências e critérios preservados:** GUA-01, EXE-01/02/03, DATA-01, REL-01. Contratos reais de plano/validação/backup/aplicação/recuperação, estado desejado efetivamente gerido e exclusões Coucou/Mochi/projetos preservados; integração de excluído é proposta separada, sem comandos ou fonte de verdade inventados

**Responsabilidade observada / bases a reutilizar:**

- [docs/SYSTEM-GUARDIAN-ARCHITECTURE.md](../docs/SYSTEM-GUARDIAN-ARCHITECTURE.md) — `System Guardian — especialização do runtime comum`, `1. Identidade, motor, runtime e papel`
- [windows/src-tauri/src/pet_factory_runtime.rs](../windows/src-tauri/src/pet_factory_runtime.rs) — `parse`, `recognizes_owner`
- [windows/src-tauri/src/pet_runtime/model.rs](../windows/src-tauri/src/pet_runtime/model.rs) — `Record`, `Owner`, `validate`

**Prova disponível e alcance:**

- [COUCOU-GUARDIAN-CONTRACT-REGISTRATION-20261025.json](../.build/COUCOU-GUARDIAN-CONTRACT-REGISTRATION-20261025.json) — Contrato, IDs e ausência de ativação; nenhuma prova de execução Guardian.

**Limite / pendência:** Ainda não há operações, UI funcional, fixtures de diagnóstico ou aceitação Guardian; nenhum acesso/mutação do anfitrião foi concedido.

**Aceitação integral:** aberta. GUA-01, EXE-01/02/03, DATA-01, REL-01. Contratos reais de plano/validação/backup/aplicação/recuperação, estado desejado efetivamente gerido e exclusões Coucou/Mochi/projetos preservados; integração de excluído é proposta separada, sem comandos ou fonte de verdade inventados

### GUA-05 — Remediação autorizada completa

**Dependências e critérios preservados:** GUA-01/04, RUN, EXE, SYS-01, REL-01/02. Finding→proposta/diff→impacto/pré-condições→autorização→execução limitada→verificação→resultado; receitas específicas sob grants explícitos e aprovação exata para sensíveis; sem shell/root geral, autoaprovação ou replay; rollback só suportado/provado, compensação/irreversibilidade explícitas

**Responsabilidade observada / bases a reutilizar:**

- [docs/SYSTEM-GUARDIAN-ARCHITECTURE.md](../docs/SYSTEM-GUARDIAN-ARCHITECTURE.md) — `System Guardian — especialização do runtime comum`, `1. Identidade, motor, runtime e papel`
- [windows/src-tauri/src/pet_factory_runtime.rs](../windows/src-tauri/src/pet_factory_runtime.rs) — `parse`, `recognizes_owner`
- [windows/src-tauri/src/pet_runtime/model.rs](../windows/src-tauri/src/pet_runtime/model.rs) — `Record`, `Owner`, `validate`

**Prova disponível e alcance:**

- [COUCOU-GUARDIAN-CONTRACT-REGISTRATION-20261025.json](../.build/COUCOU-GUARDIAN-CONTRACT-REGISTRATION-20261025.json) — Contrato, IDs e ausência de ativação; nenhuma prova de execução Guardian.

**Limite / pendência:** Ainda não há operações, UI funcional, fixtures de diagnóstico ou aceitação Guardian; nenhum acesso/mutação do anfitrião foi concedido.

**Aceitação integral:** aberta. GUA-01/04, RUN, EXE, SYS-01, REL-01/02. Finding→proposta/diff→impacto/pré-condições→autorização→execução limitada→verificação→resultado; receitas específicas sob grants explícitos e aprovação exata para sensíveis; sem shell/root geral, autoaprovação ou replay; rollback só suportado/provado, compensação/irreversibilidade explícitas

### GUA-06 — Proatividade e cooperação sem runtime paralelo

**Dependências e critérios preservados:** GUA-01/05, AUTO, TEAM, OBS. Gatilhos leves/debounce/deduplicação/limites no scheduler, orçamento e notificações comuns; LLM só quando útil; pedidos diretos/coordenador opcional, delegação autorizada sem ampliar acesso, ausência de motor/coordenador e conteúdo hostil sem alertas/efeitos repetidos

**Responsabilidade observada / bases a reutilizar:**

- [docs/SYSTEM-GUARDIAN-ARCHITECTURE.md](../docs/SYSTEM-GUARDIAN-ARCHITECTURE.md) — `System Guardian — especialização do runtime comum`, `1. Identidade, motor, runtime e papel`
- [windows/src-tauri/src/pet_factory_runtime.rs](../windows/src-tauri/src/pet_factory_runtime.rs) — `parse`, `recognizes_owner`
- [windows/src-tauri/src/pet_runtime/model.rs](../windows/src-tauri/src/pet_runtime/model.rs) — `Record`, `Owner`, `validate`

**Prova disponível e alcance:**

- [COUCOU-GUARDIAN-CONTRACT-REGISTRATION-20261025.json](../.build/COUCOU-GUARDIAN-CONTRACT-REGISTRATION-20261025.json) — Contrato, IDs e ausência de ativação; nenhuma prova de execução Guardian.

**Limite / pendência:** Ainda não há operações, UI funcional, fixtures de diagnóstico ou aceitação Guardian; nenhum acesso/mutação do anfitrião foi concedido.

**Aceitação integral:** aberta. GUA-01/05, AUTO, TEAM, OBS. Gatilhos leves/debounce/deduplicação/limites no scheduler, orçamento e notificações comuns; LLM só quando útil; pedidos diretos/coordenador opcional, delegação autorizada sem ampliar acesso, ausência de motor/coordenador e conteúdo hostil sem alertas/efeitos repetidos

## Divergências que orientam a execução

Os hashes identificam o instante auditado. Referências privadas de SDK934/F10B2B1 apontam ao snapshot congelado, para não confundir código em edição posterior com a fonte comprovada.

A seleção/revisão manual, os controlos de objetivos e o consentimento/concessão/writer estão validados apenas no SDK854, enquanto a raiz834 ainda é gestão humana. O refinamento F2 exige âmbito próprio, destino e referências exatas; não remove as guardas nativas. Não copiar uma prova dos transportes manuais para taskRuntime. O binding de pet/provider ainda tem pressupostos a evoluir para um Guardian com motor local próprio; a reserva de ID não os resolve.

O protocolo atual de delegação por texto não é mesh durável. Autonomia limitada local não prova limites internos das CLI. Uma política de janela de Sistema não é matriz por pet. Voz preparada não é cobertura de todas as operações ou áudio real. Protótipos de anexos/centro não foram entregues por esta revisão.

O sysworkspace foi consultado em leitura: AGENTS, contratos de gestão e políticas de preservação de Coucou/Mochi. Não se alterou esse repositório ou o anfitrião, nem se inventou um comando de remediação. A etapa Guardian descobrirá os contratos exatos dos gestores quando implementar o adaptador.

Os caminhos privados de prova são referências locais e não estão incluídos no dossier remoto. Nenhuma fonte/campo/documento isolado substitui a aceitação de todos os critérios, plataformas e autorizações aplicáveis.

