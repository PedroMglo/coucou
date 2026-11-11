<!-- publication-handoff-20261111 -->
**Estado 11/11/2026:** Revisão governada7 implementada na cadeia Authority comum: carryRemaining conserva saldo/limites/validade; renewFiniteAllocation exige decisão própria e nova alocação finita, sem somar saldo antigo. Só slots futuros à aprovação; consumo histórico permanece visível. Core Rust, web1566, fábrica e build58 passaram nos âmbitos descritos. Aceitação Tauri/HTTP, hardware, migração completa7 e entrega instalada pendentes. Publicação autorizada uma vez; parar depois do push. [Passagem integral de contexto](../coucou-migration/HANDOFF-CONTINUIDADE-INTEGRAL-2026-11-11.md). Os estados antigos abaixo conservam data e âmbito; não reativam permissões substituídas.

<!-- rule-authority-succession-v1 -->
**Estado vigente 11/11/2026:** concessão sucessora6 usa apenas saldo/slots ainda não comprometidos da cadeia original, sem reativar decisões antigas ou renovar validade. Backend/controlos implementados, testes dirigidos e preservação CLI verificados; build57 privada. **Janela Tauri e HTTP ponta a ponta ainda pendentes:** sessão bloqueada, sem arranque do app de QA. Instalada43 preservada. [Contrato e provas](PET-RULES.md#sucessão-com-saldo-restante--aceitação-parcial).

**Próxima dependência vigente:** Retomar a janela privada/source2 da unidade rule-authority-succession quando a sessão estiver desbloqueada: revisão própria/Stop, uma resposta HTTP sintética, saldo cumulativo e reinício sem replay. Não repetir Core/CLI concluídos; se o produto mudar, recompor a composição exata antes da aceitação. Em paralelo lógico, avançar nos bloqueadores independentes de backend: alteração/renovação governada de definições, decisões de atraso/DST, dados/fontes futuros, rotinas/notificações. LM Studio adiado; entrega/aceitação instalada e49 critérios globais continuam abertos. As notas abaixo mantêm o seu âmbito histórico.

<!-- revoked-rule-intents-v1 -->
**Estado vigente 11/11/2026:** intenções de regras revogadas consolidadas pelo Authority comum, sem publicar Active, perder reservas ou presumir fim de worker. Revisão humana funcional, falhas/ACK, reinício e backup/restauro validados na privada56; a55 recusa o formato5 sem alterar os dados. Zero pedidos ao modelo; instalada43 preservada. [Contrato e provas](PET-RULES.md#intenções-pendentes-após-revogação--aceitação-parcial).

**Próxima dependência vigente:** Reativação/alteração governada por sucessor com decisão/generation próprias: nunca voltar a pôr Active na mesma concessão revogada. Transferir apenas saldo original não comprometido, sem duplicar orçamentos ou revalidar referências antigas; manter reservas, incertezas e recursos físicos. Depois decisões de atraso/DST, dados/fontes futuros e rotinas/notificações. Aceitação instalada e restantes49 critérios abertos; LM Studio adiado. As notas seguintes conservam o seu âmbito histórico.

<!-- rule-temporal-status-v1 -->
**Estado temporal vigente 10/11/2026:** uma regra fora de `validUntil` mostra `expired`; relógio anterior à revisão autenticada/fora do intervalo mostra `authorityTimeUncertain`. Ambos mantêm `dispatchAvailable:false`; a consulta de configuração não pode apagar a recusa. A privada55 comprovou estado/controlo/recusa nativos com zero POSTs e bytes preservados. [Contrato e provas](PET-RULES.md#estado-temporal-da-autorização--aceitação-parcial). Instalada43 inalterada.

**Próxima dependência vigente:** Reconciliar intenções de metadados preparadas antes de revogar: o corpo pendente Active não pode substituir o canónico Revoked nem perder reservas. Reutilizar Authority/recibos/retirement existentes, provar falhas antes/depois da publicação e ACK/reinício. Só depois reativação/alteração governada com saldo restante. Decisões de atraso/DST, dados/fontes futuros, rotinas/notificações e aceitação instalada continuam abertas; LM Studio adiado. As notas seguintes conservam o seu âmbito histórico.

<!-- rule-probe-recovery-20261110 -->
**Recuperação de prontidão10/11/2026:** RuleGrant4 mantém a identidade nativa da operação e a sondagem gasta. A privada54 comprovou recuperação após crash, revisão própria, continuidade com o prazo/saldo originais, fim de worker no mesmo backend após falha de escrita e recusa da escolha depois de revogar.13 recibos CLI preservaram os dados e a53 recusou ativação incompatível. Modelo HTTP sintético, zero quota real; instalada43 preservada. [Contrato, fontes e limites](PET-RULES.md#recuperação-de-prontidão10112026--aceitação-parcial).

**Próxima dependência vigente:** Corrigir a projeção temporal da regra: depois de validUntil não deve anunciar dispatchAvailable/authorizedScheduled; o dispatcher já recusa a execução. Depois reativação/alteração com saldo restante, decisões de atraso/DST, dados/fontes futuros e rotinas/notificações. Entrega/aceitação instalada e os49 critérios integrais mantêm-se abertos. LM Studio continua adiado pelo utilizador. As notas seguintes conservam a sua composição histórica.

<!-- rule-scheduled-dispatch-20261110 -->
**Estado vigente10/11/2026 — agendamento1297:** rotina automática opt-in ligada ao runtime comum; janela Tauri privada52/HTTP Ollama sintético comprovou horário sem antecipação, Hub fechado, reinício sem renovar saldo/prazo, retry limitado e revogação durante metadados. Backup/restauro52 e recusa segura da51 passaram. A53 confirmou o escritor corrigido com uma única sondagem contada, uma resposta e reinício sem replay. Instalada43 preservada. [Contrato, fontes exatas e limites](PET-RULES.md#agendamento-automático-explícito10112026--aceitação-parcial).

**Próxima dependência vigente:** AUTO-01/02: recuperação exata de sondagens/intenções incertas, sem deduzir fim do worker por expiry, depois reativação/alteração governada com saldo restante e decisões de atraso/DST. Completar dados/fontes futuros, preparação diária e notificações; integrar e entregar após preservação/aceitação próprias. LM Studio adiado por decisão do utilizador; manter o adaptador, sem novo QA ou requisito de entrega nesta fase. Os49 critérios integrais mantêm-se abertos. As notas abaixo conservam os estados históricos das suas composições.

**Ocorrências — runtime comum validado em privado,10/11/2026:** Admission29/Goal67 ligam a ocorrência à reserva original de RuleGrant2. A ação funcional, uma resposta sintética, reinício sem replay, revogação durante HTTP e preservação passaram na privada51. A materialização automática e o segundo protocolo local ainda não estão validados; a instalação permanece43. [Contrato, fontes, falha corrigida e limites](PET-RULES.md#ocorrências-ligadas-ao-runtime10112026--aceitação-parcial).

**Próxima dependência vigente:** AUTO-01/02: completar a materialização automática e limitada de ocorrências no tick de leases existente, com resolução ativa de prontidão autorizada pela regra e limites duráveis de sondagem/retry, sem projetar Unknown como Ready. Conservar a primeira admissão/decisão/reserva/prazo, verificar revogação antes/durante IO e não repetir possíveis efeitos. Validar o segundo protocolo local LM Studio e o percurso de suspensão/atrasos/Stop. Depois reativação governada com saldo restante, dados/fontes futuros, preparação diária e notificações, composição/aceitação instalada;49 critérios integrais mantêm-se abertos.

**Regras — autorização própria validada em privado,10/11/2026:** RuleGrant/reservas partilham a Authority e o writer existentes; revisão funcional, reinício, revogação e restauro de duas gerações passaram sem inferência. A execução das ocorrências ainda não está ligada, a instalação continua43 e AUTO-01/02 permanecem parciais. [Contrato, provas e próxima dependência](PET-RULES.md).

**Próxima unidade:** AUTO-01/02: ligar a reserva única de RuleGrant à admissão e ao objetivo/Attempt normais, sem segundo ledger, orçamento duplicado ou origem peer fictícia. Capturar dono/destino/corpo real antes do hash/reserva; preservar prazo original, revogação durante espera/IO, incerteza e deduplicação/reinício. Só depois abrir dispatch de ocorrências. Reativação/alteração com saldo restante, dados/fontes futuros, preparação diária/notificações e aceitação instalada continuam critérios próprios.

**AUTO-01/02 parcial integrado,10/11/2026:** rascunhos persistentes e prévia IANA nativa com CAS/revisões e escolhas explícitas de mudança de hora/atrasos. Verificados no binário privado47, sem grants ou inferência. Próximo bloqueador: ativação própria, reserva total e admissão durável/deduplicada das ocorrências no runtime comum. Depois fontes futuras, preparação diária e notificações. [Contrato e prova](PET-RULES.md).

**Dependência temporal integrada,09/11/2026:** `notBeforeAt` assinado no dispatcher/admissão/lease/efeito existentes para uma resposta local finita. Revisão UTC, espera sem recurso, reinício e preservação verificados no binário privado46. AUTO-01/02 continuam abertos: implementar agora regras/concessões próprias e ocorrências duráveis, com orçamento único; depois IANA/DST e políticas de atrasos. [Contrato e prova](PET-SCHEDULED-EXECUTION.md).

**Fontes próprias do pet,09/11/2026:** inscrição explícita sem conversa, Source3/Selection2 e objetivo dedicado65/Context11/Grant10 integrados na composição de desenvolvimento build44, com janela Tauri e HTTP sintético. Revisão própria, corpo/hash/reserva e reinício sem repetir inferência passaram; foi corrigida a perda de âmbito na ponte e eliminada a seleção duplicada de formato da concessão. Inscrições antigas mantêm os seus âmbitos/bytes. Backup/restauro inativo, quarentena, ativação sem grants e leitura histórica dos novos formatos passaram em perfil gerado; a build anterior recusou-os sem alterar os dados. Regras/ocorrências e entrega continuam pendentes. A prova anterior de restauro/leitor aplica-se ao Goal64. Instalada normal43 preservada. [Contrato e limites](PET-GOAL-CALENDAR-CONTEXT.md).

**Retenção de fontes1277,09/11/2026:** corrigida a perda de ligação de calendários inscritos numa conversa ainda vazia após reiniciar. O escritor comum fixa o CID duravelmente antes de publicar; reset conserva o arquivo referido. Quatro casos direcionados e a janela Tauri39 com consentimento/CAS/revogação/reinício passaram em perfil gerado, sem modelo. O arranque pedido de inventário Guardian/Ollama também passou na privada36. Instalada43 preservada; scopes autónomos e entrega continuam pendentes. [Contrato](PET-CALENDAR-SOURCES.md), [evidência](../.build/pet-goal-execution-next-r1/calendar-conversation-preservation/VERIFIED.json).

**Coleções persistentes1276,09/11/2026:** catálogo de várias fontes por pet/CID e gestão funcional no chat integrados no percurso de calendários existente. Parser nativo, limites/revogação, formato antigo, restauro e recusa segura da43 passaram em dados gerados; privada38 compilada. Janela nativa, grants de objetivo/regra e rotinas continuam pendentes. Instalada43 e candidata44 preservadas. [Contrato](PET-CALENDAR-SOURCES.md), [provas/limites](../.build/pet-goal-execution-next-r1/persistent-calendar-collections/VERIFIED.json).

**Desenvolvimento local1274,09/11/2026:** arranque opcional de um plano finito de Sistema no Driver comum implementado e validado em componentes nativos privados e controlos funcionais simulados, com zero inferência. O restauro do novo modo passou na CLI; janela nativa, reinício operacional e entrega continuam pendentes. [Preservação e limites](../.build/pet-goal-execution-next-r1/system-requested-preservation/VERIFIED.json). Instalada43 e candidata44 preservadas; esta unidade posterior não integra a44. [Provas, falha corrigida e limites](../.build/pet-goal-execution-next-r1/system-requested-execution/VERIFIED.json).

**Candidata44 preparada,09/11/2026:** pacote normal1272 compilado,13 ficheiros/relay/ELF/arquivo de fontes verificados e11 comandos CLI de preservação passaram sobre perfil gerado35. Medição curta e Inventário manual/TaskRuntime têm provas privadas35; **a janela normal44 ainda não foi ensaiada**, porque a sessão gráfica está bloqueada. A assinatura local e a ativação aguardam o percurso nativo adequado; não falta autorização para atualizar. Instalada43 preservada. As fontes da candidata ficaram congeladas num snapshot próprio para QA enquanto prossegue desenvolvimento independente. Sem commits/push Coucou ou novas alterações do anfitrião. [Estado, fontes, provas e limites](../.build/pet-goal-execution-next-r1/release44-native/VERIFIED-PARTIAL.json).49 critérios integrais continuam abertos.

**Inventário no runtime, privada35,09/11/2026:** Guardian/Ollama completaram dois objetivos finitos de distribuição/kernel com grant Inventory próprio, intenção/reserva antes da recolha e verificação do recibo original. O formulário e a revisão nativos funcionaram sem motor escolhido; reinício, replay recusado, revogação independente, leitor43 e restauro passaram em perfis gerados. A mesma correção elimina recolha antecipada na confirmação da medição curta. Metadados sintéticos cobrem oito rotas de dono, sem validar inferência real. **Instalada43 preservada; medição e inventário novos continuam fora dela.** ProviderModel, geração/delegação e arranque automático destas operações continuam fechados;49 critérios integrais abertos. [Fontes/provas/limites](../.build/pet-goal-execution-next-r1/guardian-inventory-runtime/VERIFIED.json).

**Entrega incremental43, 09/11/2026:** Guardian/Saúde e objetivos finitos sem modelo estão na instalação normal43. O pacote passou migração25/26, dois objetivos Guardian/Ollama pelo runtime/autoridade comuns, reinício, revogação e replay; backup/restauro63/48 preservou dados com autoridade histórica em quarentena. A instalação conservou48/49 ficheiros iguais, só log mudou, nenhum desapareceu; settings.json/modelos/providers/endpoints/autostart e os5 pets selecionados ficaram iguais.26 são entradas do catálogo, não pets ativos nem conclusão funcional. A interface43 e a opção Guardian ainda desligada foram observadas; aprovação/grants continuam próprios. É verificação automatizada, não aceitação humana ou produção integral. Registo assinado sysworkspace pendente do dispositivo local; sem falta de autorização ou bypass. [Fontes exatas, provas e limites](../.build/pet-goal-execution-next-r1/release43-native/VERIFIED.json). Baselines, inventário completo, segurança/remediação/proatividade e49 critérios integrais continuam abertos.

**Entrega nativa42, 09/11/2026:** o pacote com identificador normal passou tarefas de Som Ollama/Spotify, revogação/Stop/replay e preservação dirigida sem inferência. A42 foi instalada e arrancou no perfil habitual; executável/hash/dono D-Bus confirmados, backup nativo preservado e48/49 ficheiros iguais (só log mudou, nenhum perdido). Verificação visual instalada pendente por sessão bloqueada; registo assinado sysworkspace pendente por disponibilidade do assinador, sem falta de autorização. Não fecha os49 requisitos integrais. [Provas e limites](../.build/pet-goal-execution-next-r1/release42-native/VERIFIED.json).

**Tarefas nativas de Som1261,09/11/2026:** transporte público e caller TaskRuntime habilitados só para volume/silêncio com autoridade original; ProviderModel continua fechado. Build25 privada42 passou Tauri completo para Ollama/Spotify sem inferência, conclusão assinada, recusa de replay, revogação independente e Stop durante IO. Tipos/bundle e testes dirigidos passaram. Próximo: composição do pacote42 com identificador normal, preservação e atualização instalada já autorizada; instalada38/pacote41 preservados. Não fecha restantes Sistema/voz/Guardian ou49 requisitos. [Contrato](PET-SYSTEM-ACCESS-CONTRACT.md#adoção-pública-de-tarefas-determinísticas-de-som-09112026-em-integração), [provas e limites](../.build/pet-goal-execution-next-r1/system-task-public-adoption/VERIFIED.json).

**Reserva física de Som1261,09/11/2026:** writer herdado mantém exclusão após morte do backend enquanto o auxiliar vive, incluindo órfão parado numa sessão separada de teste. Reutiliza registry seguro, Session e recolha original; lock não concede autoridade. NativeUser/Som legado adotam a mesma reserva; ficheiros, cancelamento, helpers, efeitos nativos e crashes passaram. Build24 privada42 e Tauri endereçado sobre áudio virtual passaram. Próximo: capacidades por caller e execução de tarefas nativas; gate ainda fechado, instalada38/pacote41 preservados. [Contrato](PET-SYSTEM-ACCESS-CONTRACT.md#reserva-física-de-áudio-entre-processos-09112026-em-implementação), [provas e limites](../.build/pet-goal-execution-next-r1/audio-effect-process-fence/VERIFIED.json).

**Crash de efeito Sistema1260,09/11/2026:** três workers nativos privados foram terminados antes do despacho, após commit com ACK por ler e após observação antes do recibo. Tentativa/orçamento ficaram desconhecidos sem replay, recuperação fictícia ou sucesso; famílias nativas terminaram. Transporte10 e passos/receitas91 passaram. Alterações apenas de teste; build23 conserva equivalência de produção. Próximo: reserva física entre processos enquanto um auxiliar órfão permanece vivo/parado, depois execução Tauri/gate e entrega autorizada. Instalada38/pacote41 preservados. [Contrato](PET-SYSTEM-ACCESS-CONTRACT.md#crash-no-limite-do-efeito-nativo-09112026-em-implementação), [provas e limites](../.build/pet-goal-execution-next-r1/system-effect-crash-recovery/VERIFIED.json).

**Tauri Sistema42/1259,09/11/2026:** gestão/revisão/admissão/Cancel passaram na aplicação nativa real com perfil e áudio virtual privados. SIGKILL do backend terminou o auxiliar readonly em cerca de50ms; reinício preservou objetivo62, grant48 e orçamento e recusou o token antigo. Não despachou efeitos nem prova crash após possível efeito ou aceitação humana. Build23 contém a correção das caches; instalada38/pacote41 preservados, atualização autorizada. Próximo: crash do worker no limite do efeito e recuperação sem replay; depois execução nativa/composição/entrega. [Contrato](PET-SYSTEM-ACCESS-CONTRACT.md#tauri-nativo-revisão-admissão-e-crash-antes-do-efeito-09112026), [provas e limites](../.build/pet-goal-execution-next-r1/system-native-crash-lifecycle/VERIFIED.json).

**Recolha Sistema1259,09/11/2026:** alvos expirados/recusados de passos e receitas passam a ser destruídos após soltar a lock da cache; os quatro percursos de libertação também soltam a lock do serviço antes de limpar. Dois testes nativos privados,91 regressões de passos/receitas e6 de manutenção passaram. Formatos, autoridade e gate público inalterados. Build22 antecede este incremento; próxima unidade é o lifecycle após terminação real do backend, antes de controlo nativo/composição/entrega autorizada. [Contrato](PET-SYSTEM-ACCESS-CONTRACT.md#recolha-de-alvos-de-passos-e-receitas-09112026-em-implementação), [provas e limites](../.build/pet-goal-execution-next-r1/system-task-cache-reaping/VERIFIED.json).

**Preservação nativa System42,09/11/2026:** backup/restauro real de quatro perfis gerados preservou objetivos, grants48/retirada3 e consumo, suspendeu concessões System e recusou replay no runtime. Leitor41 recusou autoridade incompatível sem alterar dados. Build privada42 compilada; não é validação humana/janela, crash de hardware ou entrega instalada. A atualização da instalada está autorizada; instalada38 e candidata41 preservadas. Próxima dependência: recolha dos alvos de passos/receitas fora da lock global e lifecycle dos helpers. [Contrato](PET-SYSTEM-ACCESS-CONTRACT.md#preservação-nativa-system6248--composição-privada42-09112026), [fontes e limites](../.build/pet-goal-execution-next-r1/system-preservation42/VERIFIED.json).

**Preservação System1255,09/11/2026:** grant48 preparado sem admissão pode ser retirado pela transação existente, com referência3 própria. Conserva origem autenticada/domínio, specHash, aprovação e hash System; não recria tarefa, modelo ou saldo. Três cenários privados passaram: três donos, perda/retoma/recusa de publicação e rejeição de admissão/claim omitido. Regressões de retirada/histórico/cache, painel, tipos, bundle e biblioteca passaram. Migração/restauro/leitor antigo, lifecycle e Tauri ainda pendentes; efeitos públicos fechados, candidata41/instalada38 preservadas. [Contrato](PET-SYSTEM-ACCESS-CONTRACT.md#retirada-de-autorização-system-sem-admissão-09112026-em-implementação), [provas e limites](../.build/pet-goal-execution-next-r1/system-orphan-retirement/VERIFIED.json).

**Controlos Sistema1254,09/11/2026:** comando nativo e confirmação System ligados ao painel de objetivos, com escolha de saída autorizada, volume/silêncio, critérios e revisão da concessão/encarnação exata, sem modelo. ACK perdido, revogação e recolha de revisões fora da lock global passaram em serviços nativos privados. Painel/controlador, tipos, bundle privado e biblioteca passaram; Tauri/janela humana, preservação48/62 e efeitos públicos permanecem pendentes. Candidata41/instalada38 preservadas. [Contrato](PET-SYSTEM-ACCESS-CONTRACT.md#controlos-nativos-da-admissão-system-09112026-em-integração), [provas e limites](../.build/pet-goal-execution-next-r1/system-admission-controls/VERIFIED.json).

**Admissão Sistema1249,09/11/2026:** tarefas finitas de Som admitidas sem modelo pela fila/Authority/orçamento existentes, com concessão, encarnação nativa e critérios exatos revistos antecipadamente. Percurso privado de admissão→receita→conclusão passou para sete identidades, com providers sintéticos; reabertura, revisão invalidada e Stop/drain comprovados. Controlos/admissão Tauri e gate público continuam pendentes; candidata41/instalada38 preservadas. [Contrato](PET-SYSTEM-ACCESS-CONTRACT.md#admissão-determinística-de-sistema--contrato-em-implementação-09112026), [provas e limites](../.build/pet-goal-execution-next-r1/deterministic-system-admission/VERIFIED.json).

**Integração nativa1245,09/11/2026:** chaves/grants existentes ligados ao alvo PipeWire e ao journal/Authority/orçamento de passos e receitas comuns. Cinco cenários nativos privados passaram, incluindo revogação durante IO, replay sem efeito, substituição/reinício e critérios finais independentes; identidade no serviço do utilizador validada só em leitura. Biblioteca fora de testes e regressões dirigidas passaram. A admissão determinística, controlos/preservação e autoridade/lifecycle Tauri ainda faltam; entrada pública fechada, candidata41/instalada38 preservadas. [Contrato](PET-SYSTEM-ACCESS-CONTRACT.md#concessões-e-tarefas-nativas--raiz1245), [fontes/provas/limites](../.build/pet-goal-execution-next-r1/native-sound-task-adoption/VERIFIED.json).

**Som nativo1243,09/11/2026:** executor tipado com prepare/commit único, intenção obrigatória, observador independente e recolha antes de libertar o recurso comum. Volume/silêncio, Stop, replay, substituição e reinício passaram num PipeWire privado com saídas virtuais; a leitura do serviço do utilizador e check fora de testes passaram. A integração de tarefas/concessões e admissão sem modelo continuam pendentes; fábrica pública fechada, sem alteração de áudio do utilizador ou nova instalação. [Contrato](PET-SYSTEM-ACCESS-CONTRACT.md#efeito-nativo-tipado--executor1243), [fontes/provas/limites](../.build/pet-goal-execution-next-r1/pipewire-retained-effects/VERIFIED.json).

**Incremento corrente — alvo físico/journal de Som,09/11/2026:** raiz1239 reutiliza os controllers de passos/receitas, Authority, orçamento, recursos e recuperação para ações tipadas de Som com grants próprios. Provas dirigidas e regressões afetadas passaram com helpers sintéticos; falta provar transporte nativo que retenha o objeto físico, migração/restauro e percurso completo. Captura real de tarefa de Som permanece recusada, sem nova capacidade TaskRuntime/ProviderModel. Na raiz1242, o leitor PipeWire pela API Quickshell existente já reteve um objeto em consultas reais, sem alteração de áudio; usa o mesmo supervisor. Efeito/ACK, recursos/grants e remoção/reconexão continuam pendentes; não se recriam definições. [Prova1242](../.build/pet-goal-execution-next-r1/pipewire-retained-reader/VERIFIED.json). Candidata41 e instalada38 não incluem este delta. Atualização local autorizada; nenhuma nova autorização é exigida por esta unidade. [Contrato](PET-SYSTEM-ACCESS-CONTRACT.md#tarefas-e-alvo-físico-comum--raiz1239), [provas/limites](../.build/pet-goal-execution-next-r1/runtime-system-sound-steps/VERIFIED.json).

**Em aceitação — planeamento próprio do destinatário financiado (09/11/2026):** implementado/integrado na raiz e empacotado na candidata mochi.41, reutilizando reserva, F2, ancestrais, Driver e decisões comuns. Provas cobrem subdelegação/segunda ronda nos dois protocolos sintéticos, orçamento único, recusa de ciclos/profundidade/admissão concorrente e Cancel durante IO sem replay. Instalada38 preservada. Próximo critério: sessão gráfica desbloqueada → mesmo RPM39 com revisão/F2 próprios61, subdelegação60 e segunda DATA → reinício61 → ativação local autorizada. Compatibilidade/restauro61 e recusa do leitor38 já passaram no mesmo pacote, em dois perfis sintéticos e sem reativar autoridade. O percurso gráfico e a recuperação de executor61 continuam separados dessa prova. Histórico copiado e continuação financiada após fecho controlado permanecem dependências próprias. Nenhum dos43+6requisitos é retirado ou declarado concluído por este incremento. [Checkpoint](../.build/pet-goal-execution-next-r1/funded-recipient-planning/CHECKPOINT.json). [Preservação61 e limites](../.build/pet-goal-execution-next-r1/funded-recipient-planning/PRESERVATION61-VERIFIED.json). Recuperação61 validada na raiz posterior à candidata39: fonte financiadora é revalidada sob o guard de autoridade ao capturar a retoma e antes de publicar o novo dono. Perda/recolha real do processo nos dois protocolos sintéticos preservou lease/escrow e enviou só a mensagem pendente, sem HTTP novo; cancelamento após preparar o worker recusou a publicação. Memória selecionada, contexto recebido e retoma base mantiveram as provas afetadas. Não está no RPM39 nem instalada, não abre continuação após fecho controlado e não equivale a aceitação da janela. [Fontes e prova de recuperação61](../.build/pet-goal-execution-next-r1/funded-planning-recovery/VERIFIED.json). Composição41 compilada/empacotada com a recuperação61, painéis nativos de Sistema, ausência de reclamação de widgets, [concessões próprias de Som](PET-SYSTEM-ACCESS-CONTRACT.md) e identidade lógica sem modelo pela fábrica. Backup/restauro nativos41 conservaram concessões de dois pets; projeção operacional suspendeu-as; leitor40 de configurações operacionais recusou-as sem mutação. O backup40 pode preservar dados opacos sem os ativar. A correção de identidade passou nos casos de fábrica, donos e runtime determinístico. Instalada38 permanece inalterada; atualização local autorizada, registo assinado sysworkspace e aceitação funcional/instalada pendentes. Depois da41, a raiz1236 adota estas concessões na voz NativeUser, antes da inferência, com alvo/revisão/sessão originais e recusa de fallback Spotify quando há política própria. Fixtures nativas passaram; captura física de áudio ficou ignorada e aceitação real/instalada pendente. TaskRuntime/ProviderModel, journal de tarefa para operações diretas e migração integral do legado continuam abertos; Guardian operacional continua pendente. [Prova1236](../.build/pet-goal-execution-next-r1/system-sound-voice-adoption/VERIFIED.json). [Fontes/provas/limites41](../.build/pet-goal-execution-next-r1/release41-native/CHECKPOINT.json).

**Unidade entregue — contexto recebido na cooperação (09/11/2026):** mochi.38 instalada pelo mesmo RPM nativamente verificado; seleção/transitividade8, F2 próprio, ACK6/7, resposta manual60 e recuperação/preservação59/60 comprovados nos âmbitos registados. Instalação habitual verificou versão/Pets e preservação, sem provider real. Objetivo43+6 aberto. Próxima dependência: planeamento próprio do destinatário de uma delegação financiada, no mesmo runtime e dentro da reserva original; definir orçamento/linhagem/F2 antes de abrir a guarda atual. Histórico copiado é dependência separada. Commits/publicação Coucou suspensos; registo host assinado pendente técnico. [Composição, provas e limites](../.build/pet-goal-execution-next-r1/release38-native/CHECKPOINT.json).

**Unidade implementada, em aceitação — memória própria na delegação iterativa (08/11/2026):** manter o catálogo iterativo explícito e selecionar revisões próprias de memória pelo R43 existente, com consentimento próprio de modelo e revisão separada do texto/destinatário de cada entrega. Linhagem composta conserva todas as revisões selecionadas e a DATA/história de rondas posteriores; não copia memória bruta para destinatários nem transfere consentimento. Formatos condicionais: Admission24/Goal55–57, Snapshot3/Published3, Grant45 e envelopes13/14; origem recebida conserva dependências assinadas para validar também trabalho literal sem modelo; os formatos antigos mantêm os seus bytes/guardas. Usar a mesma Authority, orçamento, fila, Driver e recuperação; sem novo catálogo, provider ou runtime. Contexto recebido/copiado/transitivo continua lacuna explícita. Instalada36 preservada; só promover após percurso completo, testes, UI e preservação. [Contrato, dependências e aceitação](../.build/pet-goal-execution-next-r1/iterative-selected-memory/CONTRACT.json). Prova nativa build14: objetivo55 com memória e destinatário57 literal concluído, envelopes14/13, duas rondas próprias, corpo DATA7698bytes revisto igual ao HTTP, sem conclusão falsa ou orçamento adicional. Destinatário56/F2 e ambos os motores validados por protocolos sintéticos Rust. Novo campo opcional de proveniência fica em Box sem alterar os bytes, evitando aumento da pilha. Hint de voz comum corrigido para respeitar a barreira já existente do backend; não é aceitação de áudio físico. Backup/restauro inativo55/57, recusa do leitor36 sem mutação, histórico sem autoridade e reinício de estados fechados passaram na candidata15. Retoma55 após perda real do processo passou nos dois protocolos sintéticos, conservando revisões/saldo e sem HTTP novo; revogação impede retoma. Controlo nativo56 passou no perfil gerado: F2/destino LM Studio/saldo próprios, uma resposta finita e fecho sem conclusão falsa; fonte55 parada e nenhum envio extra. Rótulo pedido/resultado corrigido depois. Entrega local37 concluída no âmbito de desenvolvimento: mesmo RPM validado ativo, backup e preservação44/45ficheiros confirmados, só o log mudou. Registo assinado host pendente; fonte recebida/copiada/transitiva continua dependência separada.

**Próxima dependência vigente — composição da admissão iterativa (08/11/2026):** a raiz já admite a escolha iterativa explícita pelo percurso público normal, com saldo partilhado e uma ronda antiga inalterada. Native run3 confirmou delegação, filho determinístico, DATA exata da segunda ronda e pai Pausado/fechado sem conclusão falsa. Aviso transitório de listagem corrigido com39casos afetados/TSC; a36 já foi ativada/verificada. Prosseguir pela próxima dependência de cooperação/contexto autorizado. Não repetir suítes gerais sem alteração/risco nem declarar G3 ou49 critérios fechados. Instalada36 inclui este incremento; assinatura do commit host pendente. [Fontes, provas e limites](../.build/pet-goal-execution-next-r1/iterative-native-admission/CHECKPOINT.json).

**Histórico da entrega — após instalação33 (08/11/2026):** atualização local1219 concluída e registada no sysworkspace; fontes ativas de desenvolvimento na própria raiz, sem commits Coucou. Prosseguir pela recuperação do planeamento do pai após perda do Owned: reutilizar fecho, histórico, admissão/continuação, recursos e Authority existentes; preservar encargos e reservas, não repetir efeitos confirmados/incertos nem converter a continuação genérica com saldo adicional numa retoma implícita. A política pública iterativa permanece fechada. Este percurso não abre outro scheduler nem uma nova série de composições privadas paralelas. [Entrega concluída e ponto de retoma](../.build/pet-goal-execution-next-r1/release33-native/FINAL-CHECKPOINT.json).

**Próximo passo vigente — integração1219 (07/11/2026):** a raiz passou a conter a composição comprovada por B3L/B3M e compilou nativamente a partir das suas próprias fontes. As barreiras de restauro assinado/leitor anterior e do controlo Pausar recuperado desta unidade foram verificadas; não repetidas suites integrais. Preparar agora a entrega incremental local com identidade normal, versão/artefacto/fontes exatos e prefixo/ativação/recuperação do sysworkspace; a atualização está autorizada, commits/push Coucou continuam suspensos. Depois continuar as dependências reais do objetivo integral, incluindo continuação do planeamento do pai, lifecycle e Guardian/Sistema/voz, mantendo a política pública ainda fechada onde falta implementação. [Checkpoint1219](../.build/pet-goal-execution-next-r1/integration1219-native-recovery-h3d2/FINAL-CHECKPOINT.json).

**Próxima dependência após B3K — 07/11/2026:** concluídos o percurso nativo de backup/restauro de dados sintéticos B3J e o alinhamento Pausar/Reabrir da vista com o backend B3K (36 casos afetados e build4/1218). Seguem autoridade/reservas/mensagens H3D2 assinadas no restauro do produto e leitores anteriores, mais controlo funcional Tauri do estado recuperado; só depois integrar as fontes exatas e atualizar a app local, já autorizado. Não confundir o restauro de preferências/pets sem grants com prova da autoridade autónoma; não renovar saldo, Owned ou consentimento. [Checkpoint vigente](../.build/pet-goal-execution-next-r1/recovery-controls-b3k/PARTIAL-CHECKPOINT.json).

**Seguimento nativo B3I — 07/11/2026:** composição1218/build3 inclui os29 WAV antes omitidos da árvore de build e respeita hidden nos controles do Hub. Navegação e rascunho entre abas observados nativamente. Próxima dependência: revisão/recuperação Tauri e compatibilidade/leitores/restauro da composição privada, seguida da integração exata e atualização local já autorizada. Áudio físico/voz, iteração pública, continuação do pai e objetivo integral permanecem abertos. [Fontes, build e limites B3I](../.build/pet-goal-execution-next-r1/native-functional-composition-b3i/PARTIAL-CHECKPOINT.json).

**Próxima unidade — 07/11/2026:** B3H privado fecha a prova nativa de inferência do filho originalmente queued e do Stop concorrente após recuperação, com F2/orçamento originais e indicador correto de projeção ainda pendente. Build nativa2 nas mesmas1189 fontes pronta. Seguem controlo funcional Tauri, compatibilidade/restauro/composição dos formatos privados e integração exata antes da atualização local já autorizada. Continuação do planeamento do pai após perda de Owned permanece dependência própria; política pública de iteração fechada. Sistema reutiliza widgets/serviços existentes e Guardian responsável apenas quando designado; critérios integrais continuam abertos. [Fontes, comandos, falhas e limites B3H](../.build/pet-goal-execution-next-r1/peer-iteration-h3d2-b3h-final/PARTIAL-CHECKPOINT.json).

**Plano anterior — 07/11/2026:** B3G6 privado validou perda real de processo, preservação de efeitos incertos/publicações e filho determinístico já vinculado, com Cancel/Pause posteriores governados pelo mesmo ledger. Segue inferência de filho previamente admitido com F2/destino/prazo originais após fecho do pai; depois controlo humano/Tauri, leitores/restauro/composição e integração. Continuação peer do pai não é reconstruída nem se abre a política pública. Sistema reutiliza widgets/serviços existentes e o Guardian pode ser o responsável designado; não há prazo global recalculado sem baseline de esforço. [Fontes, comandos, falhas e limites B3G6](../.build/pet-goal-execution-next-r1/peer-iteration-h3d2-b3g6/PARTIAL-CHECKPOINT.json). Registos seguintes são histórico.

**Estado corrente — 05/11/2026:** B3F4 privado fecha ACK existente e revisão nova só do efeito pendente quando se perde a linhagem do Driver, mantendo Owned original e sem nova inferência/autoridade de modelo. H3D2: completar preservação/recuperação quando se perde o próprio Owned (processo), leitores/restauro dos novos formatos e composição nativa com controlo humano funcional; depois integrar as fontes exatas antes de abrir o catálogo público. Não inferir morte por TTL, reconstruir Approved/Owned de DTO ou repetir efeitos. Instalação32 preservada. [fontes, comandos, falhas e limites B3F4](../.build/pet-goal-execution-next-r1/peer-iteration-h3d2-b3f4/PARTIAL-CHECKPOINT.json). Registos seguintes são histórico.

**Decisão estrutural de 07/11/2026 — Sistema:** integrar as definições/widgets já usados na estação (DMS/Quickshell observado), sem construir definições próprias equivalentes nem substituir `controlCenterButton`. Mantêm-se controlo pelos pets, matriz pet×subplugin×operação×recurso, posse operacional temporária, acesso humano independente e reutilização dos brokers/executores. Navegar para a UI não concede mutação. [Contrato vigente](PET-SYSTEM-INTEGRATION.md). Esta decisão prevalece sobre planos visuais históricos; não altera a instalação nem reabre publicação.

**R9C2C1 integrado localmente1126 (04/11/2026).** R9C2C1 implementado/validado privadamente/integrado1126: seleção/F2 própria,1inferência limitada/0ferramentas, Driver existente com vista fechada, artefacto durável, drenagem e pausa sem critérios verificados. Segue C2C2 continuidade e resultado com orçamento cumulativo e partilha própria. Mesh integral, recursão/ciclos/fan-out/paralelismo, fontes/rotinas/Sistema/voz/Guardian e entrega permanecem no objetivo49. Ver [fontes, comandos, provas e limites R9C2C1](../.build/pet-goal-execution-next-r1/FUNDED-RESPONSE-R9C2C1-FINAL-VERIFICATION-F1.json).

**R9C2B integrado localmente1123 (04/11/2026).** Integração1123: cancelamento/fila/Stop/drenagem e saldo confirmado/incerto do ramo nativo. Compatibilidade e restauro do novo Grant20/reserva3 verificados privadamente. Segue R9C2C inferência/continuação com F2/contexto/autoridade próprios, mantendo retenção para incerteza/staged. Recursão/ciclos/fan-out/paralelismo/resultados, fontes/rotinas/Sistema/voz/Guardian e entrega integral continuam abertos. Ver [fontes, comandos, falhas e limites R9C2B](../.build/pet-goal-execution-next-r1/DELEGATION-BRANCH-R9C2B-FINAL-VERIFICATION-F1.json).

# Plano de desenvolvimento dos pets autónomos

**R9C2A integrado localmente1120 (04/11/2026).** R9C2A integrado: aceitação exata, um único objetivo por origem, admissão própria limitada, vínculo canónico e resposta correlacionada pelo runtime comum. Segue C2B cancelamento/drenagem/reconciliação, incluindo publicação vinculada sem fila. Inferência/continuação/recursão, paralelismo/ciclos e resultados ainda necessários; TEAM integral aberto. Ver [fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/FUNDED-DELEGATION-R9C2A-FINAL-VERIFICATION-F1.json).


**R9C1 integrado localmente1116 (04/11/2026).** R9C1 integrado: pedido tipado, reserva/débito/recibo atómicos, pool nativo, observação e retirada exata do ramo não vinculado. R9C2 segue: vínculo único e admissão/autoridade próprias do destinatário, consumo acumulado e cancelamento/drenagem; depois paralelismo/ciclos/resultados. TEAM-01/02 permanecem abertos. Ver [fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/DELEGATION-R9C1-FINAL-VERIFICATION-F1.json).


**R9B3B2 integrado localmente1110 (04/11/2026).** B2 integrado: um relatório futuro finito com consentimento próprio, fila/Driver comuns, reinício e restauro histórico comprovados. A próxima dependência é R9C1, delegação com reservas pai-filho e orçamento cumulativo, critérios, contexto mínimo, profundidade/fan-out/ciclos e cancelamento por ramo. A ausência destas bases mantém TEAM-01/02 abertos. Ver [fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/WORK-DISCLOSURE-R9B3B2-FINAL-VERIFICATION-F1.json).


**R9B3B1 integrado localmente1107 (04/11/2026).** R9B3B1 integrado: relatório revisto de estado, envio canónico e restauro histórico. Segue R9B3B2: política explícita de divulgação persistente e consulta TaskRuntime; depois delegação com reservas comuns, fan-out/ciclos/cancelamento por ramo. O relatório manual não fecha autonomia/cooperacão integral. Ver [provas, fontes e limites](../.build/pet-goal-execution-next-r1/WORK-REPORT-R9B3B1-FINAL-VERIFICATION-F1.json).


**Checkpoint R9B3A (04/11/2026):** acompanhamento nativo do objetivo próprio ligado ao pedido integrado1102, com origem/revisão autenticadas, fila/reserva/tentativa/resposta/conclusão distintas e histórico com contagens desconhecidas. Store/reconciliação comuns, NativeUser apenas, sem novos efeitos ou inferência na consulta. Native415/403 validaram a interface privada e restauro com provider sintético; instalação32 preservada. [Contrato](PET-MESH-CONTRACT.md) e [evidência/limites](../.build/pet-goal-execution-next-r1/RECEIVED-WORK-R9B3A-FINAL-VERIFICATION-F1.json). Publicação durável de estados e delegação continuam pendentes;49requisitos integrais abertos.


**Checkpoint R9B2B (04/11/2026):** seleção e revisão própria do pedido recebido+notas para contexto autónomo local, corpo/hash/reserva comuns e leitura nativa separada de histórico implementados/integrados1097. Native391/381 validaram em privado o percurso com HTTP sintético, recuperação, leitor antigo e restauro; zero provider real/quota, instalação32 preservada. [Contrato](PET-MESH-CONTRACT.md) e [evidência/limites](../.build/pet-goal-execution-next-r1/RECEIVED-CONTEXT-R9B2B-FINAL-VERIFICATION-F1.json). Seguem estados/delegação;49requisitos integrais abertos.


**Checkpoint R9B2A (04/11/2026):** preparação do plano/contexto do objetivo recebido disponível pela interface, com escolha imutável e retoma de gravações parciais pelo runtime normal. Native371 comprovou as decisões separadas até uma resposta local, zero modelos; plano genérico também prepara dados sem inferência. Fontes1092 integradas; backend/formato1090 preservados. [Contrato](PET-MESH-CONTRACT.md) e [provas/limites](../.build/pet-goal-execution-next-r1/RECEIVED-PLAN-R9B2A-FINAL-VERIFICATION-F1.json). Segue contexto R43 da origem, estados/delegação; instalação32 e49requisitos integrais ainda não entregues.

**Checkpoint R9B1 (03/11/2026):** pedido recebido pode originar objetivo próprio revisto, com proveniência assinada e Goal24; criação não concede contexto de modelo, orçamento ou execução. A janela nativa privada comprovou decisões separadas, reinício, leitor anterior e restauro histórico. [Contrato](PET-MESH-CONTRACT.md) e [provas/limites](../.build/pet-goal-execution-next-r1/RECEIVED-TASK-R9B1-FINAL-VERIFICATION-F1.json). Fontes1090 integradas; instalada32 inalterada. Falta controlo de tarefa/plano do rascunho recebido, contexto R43 e restantes estados/delegação. Os49requisitos integrais continuam abertos.

**Checkpoint R9A (03/11/2026):** respostas locais correlacionadas integradas nas fontes1080. Cada pet responde com objetivo, sessão, orçamento e aprovação próprios; não recebe permissões do remetente. Janela nativa automatizada, reinício, leitor anterior e restauro passaram; o histórico restaurado não concede resposta/execução. [Contrato](PET-MESH-CONTRACT.md) e [provas/limites](../.build/pet-goal-execution-next-r1/CORRELATED-REPLY-R9A-FINAL-VERIFICATION-F1.json). Seguem aceitação autorizada/contexto próprio e delegação; os49critérios integrais continuam abertos. Instalada32 inalterada, sem publicação/host/quota.

**Checkpoint R8 C3B (03/11/2026):** revisão nativa de passos pendentes integrada localmente1074: preserva recibos, exige renovação explícita de prazo e separa aprovação dos dados da continuação/orçamento. Janela nativa automatizada, reinício, leitor anterior e restauro passaram; o restauro só conserva histórico. [Contrato](PET-DETERMINISTIC-RUNTIME.md) e [provas/limites](../.build/pet-goal-execution-next-r1/NATIVE-PLAN-REVISION-R8C3B-FINAL-VERIFICATION-F1.json). Seguem respostas/admissão própria do destinatário; inferência legada, delegação, Guardian/Sistema/voz e49critérios integrais continuam abertos. Instalada32 inalterada; sem commits/push/host/quota. Entradas anteriores são histórico.

**R8 C2 F1 — fecho e continuação no mesmo backend integrados localmente1062 (03/11):** o DrainProof original sela o fecho no mesmo Grant13 antes de publicar o Goal19. ACK apenas publica o alvo exato já assinado. Reference4/Spec7/Goal20 permitem nova revisão sem modelo após fecho confirmado, preservando recibos e encargos; Closed sozinho, Uncertain, cancelamento e JSON não concedem continuação. Native343 comprovou dois fechos/três admissões no mesmo processo, UI, reinício/ACK e leitor C1 anterior; native344 comprovou restauro/ativação com o mesmo binário, apenas como dados históricos. [Contrato](PET-DETERMINISTIC-RUNTIME.md), [composição](../.build/pet-goal-execution-next-r1/integration1062-native-controlled-continuation-r8c2/FINAL-CHECKPOINT.json) e [provas/limites](../.build/pet-goal-execution-next-r1/NATIVE-CONTROLLED-CONTINUATION-R8C2-FINAL-VERIFICATION-F1.json). Segue C3: revisão explícita de planos/passos expirados sem reenviar os cobertos, por mecanismos comuns.43+6requisitos integrais preservados/abertos; instalação32 inalterada, sem commits/push/merge/host/quota. As composições anteriores abaixo são histórico.

**R8 C1 F1 — continuação após perda do executor integrada localmente1057 (03/11):** a revisão explícita sem inferência conserva recibos e encargos originais; os passos cobertos não são enviados novamente. Reference3/NativeCarry, Spec6/Goal18 e Report2/Grant12 reutilizam Recovery/Store/Authority/Owned, sem outro ledger, modelo, saldo renovado ou autoridade restaurada. Provas privadas: duas perdas reais de processo, três admissões do mesmo objetivo, revisão/recusa pela UI, leitor B anterior e restauro do produto. O restauro só conserva dados históricos. [Contrato](PET-DETERMINISTIC-RUNTIME.md), [composição](../.build/pet-goal-execution-next-r1/integration1057-native-continuation-r8c/FINAL-CHECKPOINT.json) e [provas/limites](../.build/pet-goal-execution-next-r1/NATIVE-CONTINUATION-R8C-FINAL-VERIFICATION-F1.json). Segue C2: retoma após fecho controlado com prova do drain e revisão explícita dos passos expirados, sem ModelAttempt fictício. Critérios livres, respostas/admissão do destinatário e delegação mantêm-se abertos;43+6requisitos integrais preservados. Instalação32 inalterada; sem commits/push/merge/host/quota real. As composições anteriores abaixo são histórico.

**R8 B F1 — conclusão nativa integrada localmente1056 (03/11):** a revisão original inclui os predicados nativos suportados antes de certificar a admissão. Spec5/Goal17 selam Binding2; Grant11 guarda o relatório imutável no mesmo ledger de autoridade. O mesmo `Store.complete_certified` verifica e publica a conclusão, com origem explícita em recibos locais e sem recipeId fictício, novo modelo, orçamento ou aprovação no fim. Critérios livres continuam Unknown; persistência local não verifica trabalho do destinatário. Native339 demonstrou quatro conclusões automáticas do remetente, parciais/critério livre/Stop/reinício/ACK e revisão funcional pela UI automatizada; native340 verificou leitor A4 anterior e backup/inspeção/restauro/ativação reais do produto, conservando os dados históricos e recusando a autoridade antiga. [Contrato](PET-DETERMINISTIC-RUNTIME.md), [composição](../.build/pet-goal-execution-next-r1/integration1056-native-completion-r8b/FINAL-CHECKPOINT.json) e [provas/limites](../.build/pet-goal-execution-next-r1/NATIVE-COMPLETION-R8B-FINAL-VERIFICATION-F1.json). Segue R8 C: recuperação/continuação explícita do modo nativo, sem renovar autoridade ou saldo.43+6requisitos integrais preservados/abertos; instalação32 inalterada, sem commits/push/merge/host/quota real. As composições abaixo são histórico.

**R8 A4 — bases integradas localmente1051 (03/11):** o mesmo runtime admite trabalho literal de `pet.message` sem modelo, readiness, autenticação ou reserva de inferência. Reutiliza Store/Review/Approved/Authority, fila, Owned, recursos, orçamento e recuperação. Spec4/Goal16, Proof3 `goalDeterministic`, Lease2 e Grant10 selam `runtimeOnly` com0chamadas de modelo; os formatos legados conservam as suas guardas. [Contrato](PET-DETERMINISTIC-RUNTIME.md), [composição](../.build/pet-goal-execution-next-r1/integration1051-deterministic-runtime-r8a4/FINAL-CHECKPOINT.json) e [provas/limites](../.build/pet-goal-execution-next-r1/DETERMINISTIC-RUNTIME-R8-A4-VERIFICATION.json). Native338 comprovou API/controlo funcional/reinício em Ollama, LM Studio, OpenAI e Spotify sem pedidos de metadata/inferência; native336 comprovou leitor R7 anterior e backup/inspeção/restauro/ativação com recibo histórico inativo. Não é aceitação humana, hardware, inferência real ou entrega instalada. Faltam verificação nativa dos critérios e continuação explícita deste modo; receber uma mensagem não conclui o objetivo. Respostas, admissão do destinatário e delegação continuam abertas.43+6requisitos preservados; instalação32 inalterada; sem commits/push/merge/host. As entradas de composições anteriores abaixo conservam o histórico; o próximo passo vigente é R8 B, verificação/recuperação determinísticas.

R7 F1 está integrado nas fontes locais1048: controlo nativo de mensagens, recibos/inbox e consulta de histórico restaurado sem autoridade de execução. [Composição e provas](../.build/pet-goal-execution-next-r1/integration1048-mesh-r7-f1/CHECKPOINT.json). Não promove chamadas de modelo, respostas ou delegação. Próxima dependência: separar admissão/reserva de trabalho determinístico da disponibilidade de um motor, mantendo os grants/budgets/leases e a identidade do mesmo runtime. A instalação32 não foi alterada.

R7/TEAM-01 evolui pelo [contrato da mesh lógica](PET-MESH-CONTRACT.md): ingresso durável com origem nativa, mensagem/recibo/orçamento no Authority comum e sem autoridade implícita para o destinatário. C4 privado acrescenta revisão/IO nativos, UI funcional, inbox/ACK após reinício e retirada exata de intenção, com [provas e limites](../.build/pet-goal-execution-next-r1/mesh-r7-native-c4/VERIFICATION.json). Próxima dependência: compatibilidade/restauro e integração; depois respostas, admissão própria do destinatário e TEAM-02. Não anuncia delegação autónoma.

A separação de dono lógico e motor evolui pelo [contrato de binding por pet](PET-INFERENCE-BINDING.md), integrado localmente no grupo R6 F1, com instalação .32 inalterada. Preserva identidades, escolhas legadas, autoridade e recursos comuns; não ativa o Guardian ou modelos por consulta de estado.

## R5C1A4 — Esperas expiradas sem bloquear trabalho independente

R5C1A4 integrado: limpeza de esperas sem depender do catálogo, preservação de workers/Pending, resposta tardia recusada e recuperação da capacidade após64registos expirados. A entrega funcional é esta unidade do runtime individual;13dirigidos e native318 privados cobrem o delta final. Full3326/50ignorados A2 é base anterior identificada, web1353/TSC/Vite reutilizados por identidade. Instalada32 inalterada; lifecycle/protocolos/execução geral/TEAM/Voz/Sistema/Guardian e49critérios mantêm as dependências vigentes. Agrupar próximos desenvolvimentos por funcionalidade, sem repetir a suite geral em cada correção.

[Composição, provas e limites](../.build/pet-goal-execution-next-r1/QUEUE-HOUSEKEEPING-CHECKPOINT-R5C1A4.json).

## 02/11 — Aprovações externas sem interferência no desenvolvimento (R5D1A3)

Desvio funcional pedido pelo utilizador: R5D1A3 integrado1027 evita que o encaminhamento de hooks externos para o Coucou adie outros desenvolvimentos. Na build de desenvolvimento o modo é terminal por defeito; escolha temporária explícita em Settings, retirada/anti-replay e restart comprovados em privado. Instalação32 permanece1003; aceitação física/instalada pendente. R5C conserva os3 rascunhos próprios não testados no checkpoint da unidade e volta a ser a próxima dependência; não foi integrado como se estivesse concluído. Todos49 critérios, G2 e Guardian operacional permanecem abertos.

[Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/EXTERNAL-APPROVAL-PARTIAL-CHECKPOINT-R5D1A3.json).

## 02/11 — Pedido durável de arranque e fila inicial (R5B1A2)

R5B1A2 integrado1023 fecha este incremento de pedido explícito e dispatch inicial no percurso limitado já operacional. Native310 comprova fila ocupada, vistas ocultas, Stop por ramo, retomada só de pedido não reclamado e recusa do14 pelo leitor1021; native311 comprova backup/restauro com pedido antigo inerte e rascunho futuro sem grant. Próxima unidade R5C: housekeeping de cancelamento/deadline antes de Store/list, para catálogo indisponível não conservar um hold de espera além do necessário; worker/Pending vivos mantêm exclusão até recolha/proof. Preferências/justiça integrais, restantes protocolos/lifecycle, serviços/TEAM e49 requisitos continuam abertos.

[Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/REQUESTED-DISPATCH-PARTIAL-CHECKPOINT-R5B1A2.json).

## 02/11 — Fila de recursos e cancelamento independente (R5A5)

R5A5 integrado1021: arbitragem usada pelo percurso real de yield/decisão/reaquisição, posição observável e cancelamento de um ramo sem afetar os restantes. Native309 comprova dois objetivos, bloqueio por terceiro dono e conclusão do segundo após cancelar o primeiro. Segue R5B: intenção explícita de execução persistida antes do efeito, dispatch limitado e recuperação no runtime existente; não iniciar filas antigas/restauros silenciosamente. Configuração de prioridades, restante lifecycle/protocolos e todos49 critérios integrais continuam abertos.

[Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/RUNTIME-ARBITRATION-PARTIAL-CHECKPOINT-R5A5.json).

## 01/11 — Objetivo com vistas fechadas e Stop durante IO (R4A1)

R4A1 exercita o lifecycle já existente na composição1020: objetivo completa com chat/gestão ocultos e cancelamento manual separado; outro para durante IO, conserva encargo desconhecido e permite nova aquisição do recurso. Não houve alteração de produção nem repetição de suites iguais. Próxima auditoria/implementação G2: fila, prioridades/justiça e passagem de admissão para execução autorizada, incluindo recursos indisponíveis, reinício e Stop. Reutilizar Store/Authority/dispatcher/broker, sem segundo scheduler ou ativação silenciosa de trabalho antigo. Todos49 critérios integrais, demais lifecycle/protocolos e entrega continuam abertos.

[Composição, comando, prova e limites](../.build/pet-goal-execution-next-r1/RUNTIME-LIFECYCLE-PARTIAL-CHECKPOINT-R4A1.json).

## 01/11 — Retirada de autorização com origem nativa (R3A1)

R3A1 integrado1020 fecha o recorte de implementação, validação automatizada e integração local das três fronteiras R1/R2/R3. O grupo G2 integral e a entrega instalada permanecem abertos. Próxima unidade R4: lifecycle de objetivos com chat/Settings ocultos, cancelamento manual com âmbito separado, Stop durante IO, workers/holds e retoma sem renovar autoridade. Reutilizar o backend e a solução mais simples; nenhum serviço persistente será instalado. Finalidade/painel e restantes protocolos/aceitações mantêm-se no plano.

[Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/RUNTIME-RETIREMENT-REVIEW-PARTIAL-CHECKPOINT-R3A1.json).

## 01/11 — Grupos de entrega integral

[Dez grupos,49 requisitos e critérios de aceitação](PETS-DELIVERY-GROUPS.md). Trabalho atual em G2 (autonomia individual); a subfase de origem nativa das revisões tem R1/R2/R3 integrados:3/3 unidades,100% da implementação/prova automatizada/integração local delimitadas, por contagem de unidades de peso igual. Este número não mede G2 inteiro nem o sistema integral; a aceitação instalada permanece pendente. Segue lifecycle R4 sem alterar a ordem/dependências do objetivo completo.

## 01/11 — Consentimento nativo de admissão e continuação (R2A3)

R2A3 integrado1019, backend e controlo funcional:36 dirigidos,3291 gerais/50 ignorados,1341 web/0ignorados/TypeScript/Vite,build616 e nativos299/300. Próxima unidade PET/RUN/EXE é a revisão de retirada de autorização não utilizada, ainda ligada apenas à label em retirement_api.rs; reutilizar a mesma origem nativa e preservar escolha/ACK/efeitos incertos. Depois seguem finalidade/painel e demais dependências reais do runtime individual, antes de TEAM. A cooperação, restantes integrações, Voz/Sistema e Guardian mantêm os seus critérios integrais; não há nova entrega por estas provas.

[Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/RUNTIME-ADMISSION-REVIEW-PARTIAL-CHECKPOINT-R2A3.json).

## 01/11 — Contexto nativo da recuperação de objetivos (R1A1)

R1A1 integrado1018: revisão nativa da recuperação e reconciliação da escolha já consumida, com testes dirigidos, suite geral e nativo294 aprovados. Próxima unidade PET/RUN/EXE: aplicar a mesma identidade ao consentimento de admissão/continuação e às respetivas esperas, preservando grants já assinados e ACKs; conservar a aceitação nativa das três superfícies e distinguir janela/documento de finalidade/painel. TEAM segue as dependências de autonomia individual. Os bloqueadores físicos do clipboard e Voz permanecem no seu percurso; os49 critérios e Guardian continuam abertos. Nativo296 acrescenta Companions (ocultar/reabrir) e PetControl (reload/reabertura explícita pela navegação nativa),9.17s: revisão nova nas três superfícies Linux existentes, token anterior/caller errado recusados e objetivo/ficheiros inalterados,0inferência/quota.295 foi conservado: o reload recolhe PetControl e a nova revisão permanece recusada até à abertura explícita; não se alargaram ACLs nem se alteraram fontes para fazer passar o teste.

[Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/RUNTIME-RECOVERY-REVIEW-PARTIAL-CHECKPOINT-R1A1-N296.json).

## 01/11 — Sequência global após R3: runtime individual

A unidade de IO do clipboard C6C4B2B1R3A6 está integrada e comprovada no seu âmbito. A sequência global volta agora a PET/RUN/EXE: vincular a revisão humana de recuperação ao contexto nativo que a apresentou, reutilizando primitivas de identidade existentes e preservando Approved/ACK/efeitos incertos. A auditoria observou comparação por label no cache; ainda falta o caso nativo de fecho/reabertura para provar o comportamento. Fechar essa revisão não deve cancelar objetivos de background nem renovar concessões. Seleção/revisão/frescura/alvo físicos e Close sem ACK continuam pendentes no percurso Voz/Sistema, sem levantar a barreira de ativação. Estas dependências não impedem o desenvolvimento independente do runtime individual, anterior a TEAM. [Contrato e critérios da próxima unidade](../.build/pet-goal-execution-next-r1/RUNTIME-RECOVERY-REVIEW-CONTRACT-R1.md).

## 01/11 — Escrita Linux com driver próprio e confirmação (C6C4B2B1R3A6)

C6C4B2B1R3A6 integrado1017: init/write Linux isoladas em processo próprio, troca limitada não bloqueante, Prepare/Commit/Abort e posse/recolha mecânicas.29 dirigidos leitura/escrita,3281 gerais/50 ignorados, build614 e nativo291 A4 reutilizados por485 dependências do compilador e diretório capabilities idênticos;23 helpers A5 e28 Copilot A6 aprovados e7 testes de residente A2 estão também no full final. Web1339/TSC/Vite A4 reutilizados por igualdade explícita das fontes/configuração e bundle. Próximo: recibo/posse/revisão/frescura da seleção e alvo nativos, incluindo substituição por bytes iguais, antes de teclas/reposição. Depois reconciliação/admissão durável de Close sem ACK, Notes/contexto antes do áudio, captura/runtime/modelo físicos, ativação governada e restantes dependências integrais. Nenhuma autorização nova de instalação/publicação.

[Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/CLIPBOARD-WRITER-PARTIAL-CHECKPOINT-C6C4B2B1R3A6.json).

## 01/11 — Preservação e recuperação manual do texto (C6C4B2B1R2A4)

C6C4B2B1R2A4 integrado1013: segunda observação limitada, estados de preservação, cópia manual do último texto e invalidação por origem/revisão. Autoridade do supervisor comum volta a verificar cancelamento/prazo após callback, antes de preflight/IO. Final:3259 Rust/50 ignorados,1339 web/0 ignorados, TypeScript/Vite, build610 e nativo privado287. Os originais interrompidos de build/full foram preservados; recovery1 usa os mesmos inputs após comprovar ausência de workers. Próximo: escrita/init nativas limitadas, recibo de seleção/revisão/frescura/alvo e reconciliação durável de Close sem ACK. Não tratar bytes iguais como posse. Depois Notes/contexto antes do áudio, identidade de captura/runtime/modelo, ativação governada e restantes dependências integrais; autonomia/TEAM/Guardian continuam abertos.

[Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/CLIPBOARD-RESTORE-PARTIAL-CHECKPOINT-C6C4B2B1R2A4.json).

## 01/11 — Leitura anterior do clipboard em helper próprio (C6C4B2B1R1A2)

C6C4B2B1R1A2 integra a leitura anterior isolada e limitada, com14 dirigidos,3246 gerais/50 ignorados, build610 e ensaio nativo privado285 aprovados. Próximo: retirar também a leitura bloqueante da reposição e distinguir no controlo funcional os estados anteriores vazio/indisponível/excessivo; depois prender seleção/revisão/frescura e alvo nativos antes de teclas/reposição, sem tratar texto igual como recibo. Watcher público já permite observar/cancelar, mas não expõe uma barreira síncrona fresca. Falta ainda reconciliação durável de Close sem ACK. Seguem Notes/contexto antes do áudio, identidade de captura/runtime/modelo e ativação governada; as dependências integrais mantêm-se.

[Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/CLIPBOARD-READ-PARTIAL-CHECKPOINT-C6C4B2B1R1A2.json).

## 01/11 — Ditado: guards internos e resultados de colagem (C6C4B2A5)

C6C4B2A5 integra guards internos e resultados parciais de clipboard/portal. Próxima dependência: seleção/revisão nativa do clipboard e frescura antes das teclas, alvo exato da janela/recurso e cancelamento dos IOs que ainda podem bloquear dentro das bibliotecas. Também falta reconciliação durável quando Close não tem ACK. Não se considera a comparação por texto uma prova de posse; a reposição antiga permanece best effort enquanto se implementa o recibo nativo. Seguem identidade física de captura/runtime/modelo, Notes/contexto antes do áudio e ativação governada de Voz após restauro; depois restantes componentes e dependências integrais.

[Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/VOICE-PASTE-PARTIAL-CHECKPOINT-C6C4B2A5.json).

## 01/11 — Whisper: validade durante IO e família própria (C6C4B1A3)

C6C4B1A3 conclui este incremento de IO/ownership do Whisper em Linux:21 dirigidos,3209 gerais/38 ignorados e build606 passaram. Próxima unidade: validade dentro das esperas de clipboard/portal do ditado, antes dos efeitos, com resultado parcial/incerto explícito e sem repetir a colagem. Permanecem identidade física da captura/runtime/modelo antes de esperar, Notes com timestamps, conversa/anexos antes do áudio e ativação governada de Voz após restauro. Depois seguem as componentes Móvel/Hooks/Credenciais e as dependências integrais de autonomia, TEAM e Guardian.

[Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/VOICE-WHISPER-IO-CHECKPOINT-C6C4B1A3.json).

## 01/11 — Contexto ASR e ditado antes do áudio (C6C4A4)

C6C4A4 integra a captura antecipada do contexto ASR e das opções de ditado, com cancelamento por ocorrência e exclusão no worker existente. Continuam abertos o IO e recolha de família do Whisper, os guards internos de clipboard/portal, identidade física do alvo antes de esperar e Notes com timestamps/diarização. A próxima dependência é o Whisper: um guard nos limites da chamada não interrompe write_all bloqueado nem recolhe filhos. Seguem alvo físico/Notes e ativação governada de Voz após restauro; as suspensões existentes não foram levantadas.

[Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/VOICE-CAPTURE-CHECKPOINT-C6C4A4.json).

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

Os 43 IDs originais mantêm todos os critérios. Estes seis IDs são adicionais; a implementação completa e as recusas estão no [contrato Guardian](SYSTEM-GUARDIAN-ARCHITECTURE.md). Estado atual09/11: observação limitada e tarefas finitas de Saúde entregues na43; medição curta integrada/validada em privada32. Inventário completo, segurança, sysworkspace, remediação e proatividade continuam parciais/pendentes. Os49 critérios integrais mantêm-se.

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


### R9C2C2A integrada1127 — continuação finita sem saldo adicional

Após integração1126, reutilizar o histórico autenticado/Authority/F2/Driver para uma ronda local adicional, só depois do fecho original confirmado e de nova revisão humana. Reserva Bound original, CAS único, consumo cumulativo e prazo conservados; Settled não reabre. Formatos/provas em `PET-MESH-CONTRACT.md`, secção R9C2C2A. Primeiro fechar esta unidade com testes dirigidos, UI nativa, reinício/leitor antigo/restauro; depois R9C2C2B partilha explícita de derivados e verificação/publicação do resultado. Não representa fecho TEAM, R9 ou qualquer um dos49 requisitos.


## R9C2C2A — continuação finita integrada1127 (2026-11-04T07:01:18.507246+00:00)

O mesmo objetivo delegado pode receber outra resposta local finita após fecho original confirmado, nova revisão humana e consentimento F2 próprio. Usa Reference6/Anchor e Authority/fila/Driver existentes; Funding2 autentica o prefixo e o binding muda por CAS. Consumo anterior + novos limites cabem na reserva original e no mesmo prazo. Não existe saldo adicional, continuação automática, reabertura de Settled ou partilha implícita de derivados. Resultados desconhecidos bloqueiam nova ronda.

Formatos condicionais Admission13/Goal31/Funding2/Reference6/Grant22; Reservation4/Grant23 no pai. Reconcile agrega prefixo e atual, preservando consumo/certeza. Metadados podem ser consultados explicitamente para um objetivo financiado pausado, sem criar execução ou contornar F2. A referência da cache que mantinha o escritor da conversa é retirada só após ACK do fecho original, recolha dos workers e libertação do recurso; conserva uma projeção limitada do estado sem executor. Permite rondas sucessivas sem reiniciar. Stop/inatividade/expiração não são prova suficiente de libertação. Capacidades continuam por caller; a extensão não promove taskRuntime/providerModel.

Provas: `.build/pet-goal-execution-next-r1/FUNDED-CONTINUATION-R9C2C2A-FINAL-VERIFICATION-F1.json`; cinco casos finais de continuação, fecho7 e regressões afetadas9+4+6 por identidade dos ficheiros cobertos, web62/tipos/Vite e build698. Native521 e restauro525 validam duas rondas/duas escolhas F2, hashes do corpo real, orçamento cumulativo, terceira ronda recusada, fecho original, reinício, leitor anterior e backup/restauro do produto. São provas privadas com provider sintético, sem aceitação humana/instalada ou quota real. Falhas e filtros vazios estão preservados e excluídos da prova.

Os49 requisitos integrais continuam abertos. Seguinte: R9C2C2B, partilha autorizada de resultados derivados e verificação/publicação; ferramentas, delegação recursiva/paralela, restantes integrações, Sistema/voz/Guardian e entrega permanecem dependências posteriores. Sem commits/push/merge ou instalação.


### R9C2C2B em execução — resultado autenticado até partilha verificada

B1 reutiliza a revisão de planos DATA para capturar um excerto autenticado, a sua origem e o destinatário original, sem execução. B2 acrescenta concessão de divulgação própria, sucessor determinístico, contabilização cumulativa e publicação/verificação do resultado. A fronteira é explícita: ModelResult e F2 não concedem partilha. O contrato B1 está em `PET-MESH-CONTRACT.md`; o objetivo mantém os49 requisitos integrais.


## R9C2C2B1 — proposta de resultado derivado integrada1129 (2026-11-04T07:41:18.146714+00:00)

O recibo próprio ModelResult1 pode fornecer um excerto exato UTF-8 a uma revisão de plano DATA, através do mesmo NativePlanReview/publicação/ACK/retirada existente. Referências vinculam dono, objetivo, decisão, pedido, tentativa, hash do recibo, intervalo e cobertura; destino e correlação são a origem da delegação. Corpo/texto/recibo/offsets/destino trocados e origem cancelada são recusados. Sob writer, a validação autenticada do recibo não readquire flock. Não copia o histórico nem altera a identidade ou critérios.

Formatos condicionais Reference2/Seal2/Goal32/Grant24; ModelResult1 preservado e View acrescenta receiptHash. A validação dos antigos admissions de resposta consulta o plano original autenticado, mantendo os fences de novos efeitos sobre o plano corrente. `pet.resultReply` v1 tem apenas validação DATA, sem executor, Call ou ferramenta de modelo. A capacidade é NativeUser-only para pets LLM; não promove observadores/serviços.

A interface permite selecionar o intervalo, rever texto/proveniência/destino e guardar o plano. A confirmação não cria admissão, orçamento, envio ou consentimento de divulgação. Depois da proposta guardada, a execução fica bloqueada até B2; F2 e WorkDisclosure continuam com âmbitos separados. Não certifica critérios pelo conteúdo do modelo.

Provas: `.build/pet-goal-execution-next-r1/DERIVED-RESULT-PLAN-R9C2C2B1-FINAL-VERIFICATION-F1.json`; core4, revisão de planos16, resultados14 (+helper filho executado pelo teste pai), armazenamento4, capacidades46, web71/tipos/Vite, build700. Native527/531 validam UI real sobre provider sintético, duas rondas sem reiniciar, proposta32/24 e ACK/inventário DATA-only, orçamento conservado, ausência de envio, reinício, leitor anterior e backup/restauro do produto sem renovar autoridade. Falhas/filtro vazio estão preservados e não sustentam aceitação.

Próximo: R9C2C2B2, concessão de divulgação própria e sucessor determinístico com orçamento cumulativo, entrega e verificação. Os49 requisitos integrais permanecem abertos. Sem commits/push/merge, instalação ou aceitação humana/física/online.


#### R9C2C2B2D1 em execução — 04/11/2026

Sobre integrated1129: capturar origem autenticada, destino local exato e dependências atuais para a revisão humana do resultado, pela superfície nativa de plano existente. Sem concessão ou envio. Depois D2: concessão exata e sucessor determinístico financiado, com revalidação antes do efeito e recibo local; não reabrir Settled nem usar F2 como autorização de partilha. Os 43+6 requisitos conservam os critérios e o estado aberto.


### R9C2C2B2D1 — revisão exata da origem e dependências integrada1133 (2026-11-04T08:11:33.669573+00:00)

A superfície nativa de revisão de resultado apresenta agora motor/adaptador/modelo e destino original autenticados, input hash, tempos, excerto UTF-8 e cobertura, destinatário/configuração e referências exatas das fontes. Capturas próprias da memória, delegação e destinatário são invalidadas de forma permanente ao observarem alterações; há revalidação no writer de publicação. Reutiliza NativePlanReview/Authority/Target/Capture; sem formato persistente ou ledger novos. A confirmação continua DATA-only: divulgação e execução dependem de D2, ainda aberto. Evidência detalhada em `docs/PETS-VALIDATION.md` e `.build/pet-goal-execution-next-r1/RESULT-DISCLOSURE-R9C2C2B2D1-FINAL-VERIFICATION-F1.json`. Os43+6 critérios integrais continuam abertos. Sem commits, publicação ou instalação.


#### R9C2C2B2D2 iniciado — 04/11/2026

Depois de integrated1133: concessão exata de resultado e sucessor determinístico Admission14 sobre plano2/Ref7/Funding3; depois Envelope6, entrega no mesmo Driver, verificações e aceitação nativa sintética. Zero inferência adicional para enviar e nenhuma reserva adicional. Os49 requisitos integrais continuam abertos.


### R9C2C2B2D2 — entrega autorizada integrada1138 (2026-11-04T09:52:11.346927+00:00)

D2 integrado em desenvolvimento após testes dirigidos e percurso Tauri privado. O envio usa consentimento próprio e reserva original, sem modelo adicional. A retoma após Busy verifica a sessão pendente exata; a consulta da resposta original sobrevive ao sucessor nativo. Segue C3: consumo autorizado do resultado recebido; depois ferramentas/ciclos/fan-out/paralelismo. TEAM-01/02 e o objetivo integral continuam abertos. Evidência e limites: `.build/pet-goal-execution-next-r1/RESULT-DELIVERY-R9C2C2B2D2-FINAL-VERIFICATION-F1.json`.


#### R9C2C3 — próxima unidade após integração1138

Consumo explícito do resultado derivado recebido. Reutilizar a adoção de dados/objetivos e R43, sem um runtime ou ledger novo. Primeiro fechar o contrato de origem/contexto e invalidar dependências sem partilhar memória bruta; depois ligar controlo funcional e execução própria autorizada com provas dirigidas e nativas. Envelope6 só entregue continua DATA-only. Inventário: `.build/pet-goal-execution-next-r1/RECEIVED-RESULT-R9C2C3-DEPENDENCIES.json`; implementação ainda não iniciada.


R9C2C3 iniciado: ligação completa de resultado6 a objetivo/contexto próprios e resposta finita com orçamento próprio/F2. Contrato e formatos condicionais em PET-MESH-CONTRACT; adoção DATA-only não é conclusão deste percurso. Sem quota real, instalação ou nova infraestrutura.


### R9C2C3 — resultado recebido e resposta própria integrada1142 (2026-11-04T11:11:42.447422+00:00)

C3 integrado em desenvolvimento: consentimento separado, uma resposta própria, janela fechada, reinício e restauro sem replay/renovação. A correção do leitor histórico preserva resultados utilizáveis apenas como DATA. R9C2D: inventariar a continuação de ferramentas no mesmo runtime/orçamento/autorização; fechar a próxima lacuna executável sem duplicar scheduler/ledger. Depois recursão/ciclos/fan-out/paralelismo/arbitragem e restantes49 requisitos, Sistema/voz/Guardian e produção. Prova/limites: `.build/pet-goal-execution-next-r1/RECEIVED-RESULT-R9C2C3-FINAL-VERIFICATION-F1.json`.


### R9C2D — continuação de ferramentas determinísticas financiadas

Estender o mesmo percurso nativo de mensagens já executável para continuar passos em falta após drenagem/recolha confirmada do dono original. Reutilizar NativeCarry/recibos, revisão de plano, Authority, fila, recursos, ledger e executores; uma lease expirada não prova fecho. Âmbito inicial: operações determinísticas entre pets já suportadas, sem inferência própria, delegação recursiva, WorkDisclosure ou ResultDisclosure. Não equivale a habilitar shell/Sistema arbitrários.

Formatos condicionais: Continuation8 com carry nativo/prefixo11 ou16, Funding4, Admission16, Goal36, Grant26 para filho com esse prefixo ou revisão de plano financiada e Reservation5/Grant27 no pai. A referência mantém additionalAllowance=false; consumo anterior + todos os novos limites cabem na mesma reserva e no prazo original. Troca de binding é CAS exato; saldo novo, reabertura de Settled e repetição de efeitos incertos ficam recusados. Formatos antigos preservam bytes e validações.

A revisão própria mostra passos/recibos anteriores, recursos e orçamento; somente os passos não cobertos podem executar. Alterar um passo já entregue é recusado. Uma revisão de plano conserva a fonte/carry mas não concede execução. Revalidar cancelamento/fonte/destinatário antes de novos efeitos; recuperação/ACK mantêm a mesma decisão, sem novo orçamento. Critérios nativos verificam recibos locais, não êxito semântico do destinatário. Provas integrais e integração desta unidade ainda pendentes. Inventário: `.build/pet-goal-execution-next-r1/FUNDED-TOOLS-R9C2D-DEPENDENCIES.json`.


### R9C2D — continuação de ferramentas financiadas integrada1143 (2026-11-04T11:31:16.354805+00:00)

D integrado localmente após testes dirigidos e Tauri privado: retoma de passos pendentes na mesma reserva, preservação e reconciliação. R9C2E: inventariar recursão/ciclos/fan-out e paralelismo/arbitragem sobre as reservas comuns; implementar a próxima dependência real sem nova fila ou ledger. Inferência com ferramentas gerais, restantes integrações/Sistema/voz/Guardian e produção continuam abertos. Evidência/limites: `.build/pet-goal-execution-next-r1/FUNDED-TOOLS-R9C2D-FINAL-VERIFICATION-F1.json`.


### R9C2E — próxima dependência: recursão, ciclos e orçamento de descendentes

O runtime atual executa delegação de uma aresta e continuação própria financiada; `maxDepth`/catálogo não provam recursão. Antes de a habilitar, autenticar a linhagem a partir das concessões/recibos canónicos, impor profundidade/fan-out e ciclos por pet/objetivo, repartir o mesmo saldo e verificar todos os ancestrais antes de efeitos. Reconciliar consumo próprio e de descendentes sem libertar saldo/recursos enquanto houver resultados ou workers desconhecidos. Cancelar um ramo não cancela os irmãos. Não alargar formatos antigos para representar autoridade nova.

Reutilizar Authority/Source/Origin/admissão/fila/reserva/drenagem existentes. Novos formatos ainda não alocados; implementação desta dependência não iniciada. Inventário de fontes, lacunas, decisões e critérios: `.build/pet-goal-execution-next-r1/RECURSIVE-DELEGATION-R9C2E-DEPENDENCIES.json`. Os49 requisitos integrais continuam abertos.


### R9C2E — contrato de recursão finita no runtime comum

Cada aresta autentica origem/destino, recibo e reserva canónicos; a lista de ancestrais é derivada pelo backend. A raiz fixa profundidade máxima (até8), cada nível só pode reduzi-la; recusam-se pets/objetivos repetidos, alvos ancestrais e fan-out acima de16 por pool. O saldo global é limitado pela reserva original e não aumenta ao criar níveis. O mesmo runtime admite e executa operações; o coordenador continua facultativo.

Alocação condicional: Admission17/Goal37 para um pool próprio financiado recebido da raiz; Envelope7/Ledger7/Payload2 para nova delegação com linhagem; Origin4/Goal38 para a aceitação DATA de uma delegação recursiva; Goal39 para a sua admissão17 (pool) ou18 (operações determinísticas de folha). Funding5 (pool) e6 (folha), sem prefixo/continuação nesta primeira rota; Grant28 (pool financiado), Reservation6/Grant29 (ancestral que vincula a nova classe), Grant30 (folha). Formatos anteriores e respetivas validações permanecem estáveis. Reply5 pode responder à nova origem autenticada; receber ou delegar não concede inferência, memória, partilha ou ferramenta extra.

Admission17 reserva o próprio envio e os recursos dos filhos dentro da concessão do pai. Admission18 usa apenas operações determinísticas já suportadas e aprovações próprias exatas. Modelo, ferramentas de Sistema/shell arbitrárias e continuação recursiva não são habilitados pela nova origem: dependem dos contratos completos próprios. Este é um incremento da recursão, não a conclusão dos49 requisitos nem uma substituição do objetivo integral.

A validação comum percorre os ancestrais com limite fixo antes de admissão/claim/espera/efeito; cancelamento ou expiração em qualquer aresta impede novos efeitos. Stop por subárvore persiste intenção antes de sinalizar executores e preserva irmãos. Uma reserva ancestral só é liquidada quando o próprio dono foi recolhido e as reservas descendentes foram reconciliadas/retiradas; soma os consumos próprios e os conhecidos/desconhecidos dos descendentes uma única vez. Fecho de janela, timeout/lease ou conclusão local de entrega não provam fim da subárvore. A recuperação conserva referências/decisões/saldo e nunca repete efeitos desconhecidos.

Reutiliza Source/Origin/revisão nativa, Authority/HMAC, Store/fila, recursos, ledger e drenagem. Testar dois níveis com orçamentos repartidos, revogação da raiz, um ramo cancelado e outro preservado, ciclos/forja/replay/overbudget, publicação parcial, reinício/restauro e leitor incompatível antes da integração. A capacidade recursiva só será projetada como implementada quando o percurso funcional e as provas estiverem fechados.


R9C2E — correlação das respostas recursivas: a inspeção do codec confirmou que Reply5/6 exigem correlação direta com a mensagem raiz e origem sem `replyTo`. Para preservar integralmente esse contrato, a resposta à delegação7 usa Envelope8/Ledger8, mantendo a correlação da raiz e a referência exata à aresta recebida. Reply5/6 permanecem para as fontes anteriores; não são alargados silenciosamente para aceitar uma cadeia. A delegação7 transporta `replyTo` da origem canónica e a mesma correlação da raiz. Esta extensão é parte da mesma unidade, ainda sem promoção/integração.


### R9C2E — delegação recursiva determinística integrada1147 (2026-11-04T12:46:54.560631+00:00)

R9C2E integrado após backend/controlos, Tauri privado e preservação/restauro. R9C2F: inventariar e fechar o consentimento R43 próprio para resposta local finita a uma delegação recursiva, sobre fontes/revisões/destino/modelo/corpo exatos, mesma reserva e runtime; não remover a recusa atual até contrato e provas completos. Autonomia/planeamento/ferramentas gerais, cooperação paralela integral, CLI, rotinas/fontes, Sistema/voz/Guardian e produção permanecem abertos. Evidência e limites: `.build/pet-goal-execution-next-r1/RECURSIVE-DELEGATION-R9C2E-FINAL-VERIFICATION-F1.json`.


### R9C2F — dados e resposta própria de delegação recursiva (contrato antes de código, 04/11)

Estende R43/ReceivedContext e o Driver finito existentes a Origin4/Envelope7. A seleção4 conserva hash da origem certificada e fonte exata; Context6, Goal40 antes da admissão/41 com resposta, Preview/GoalDataGrant5, Admission19/Funding7, Grant31, Reservation7 e parentGrant32 são condicionais. Formatos antigos preservam a resolução anterior e recusam os novos formatos incompatíveis. Origin4/Envelope7 não entra no antigo selection2/Envelope4.

A admissão19 exige proprietário com motor local Ollama/LM Studio explicitamente configurado e suportado, seleção4 no contexto6, revisão e consentimento próprios do objetivo com referências/revisões/destino/modelo e mensagem composta real, uma chamada ao modelo/zero ferramentas, sem continuação, plano de ferramentas ou divulgação de resultados. Captura do corpo precede hash/reserva; a mesma reserva de origem mantém prazo e limites, sem prefixo ou novo saldo. A revisão de turno manual/CLI não substitui este consentimento.

O witness revalida a linhagem canónica inteira e a reserva Pending ou Bound ao próprio objetivo, incluindo revogação/expiração durante esperas/IO. Materializa apenas dados selecionados mínimos, nunca leases/controllers/históricos da origem ou autoridade de texto. Cancelamento ancestral bloqueia novos efeitos; se dados já foram entregues, conserva a incerteza e proveniência em vez de repetir ou publicar sucesso. Recebidos de pets continuam não fiáveis.

Reutiliza Authority/Store/binding CAS, recursos, driver finito, fecho original e reconciliação. Uma resposta persistida fica por verificar; não declara critérios arbitrários cumpridos, não responde automaticamente a outro pet e não recebe permissão de Sistema/shell. Apenas nativeUser prepara a revisão; taskRuntime executa só a admissão original já autorizada e providerModel não ganha ferramenta de admissão. Controlos mantêm recusa de Origin4 até o percurso completo estar comprovado. Provas: ambos os protocolos locais sintéticos, consentimento ausente e referências/destino/modelo trocados, soma da mesma reserva, cancelamento ancestral durante HTTP, reinício/restauro e leitor E anterior no âmbito correto. Sem quota real, instalação ou alterações do anfitrião.


### R9C2F — resposta própria a delegação recursiva integrada1148 (2026-11-04T13:17:37.986677+00:00)

Resposta local finita com origem recursiva integrada. R9C2G: divulgação autorizada de um resultado próprio obtido de delegação recursiva, reutilizando ResultDisclosure/planos/revisão/admissão/binding/recibos existentes. Primeiro inventariar fontes/formatos e registar o contrato; só depois implementar o excerto e proveniência exatos, destinatário original/correlação da raiz e consumo cumulativo da mesma reserva. Não transformar resposta persistida em partilha ou sucesso implícitos. Restantes critérios TEAM/autonomia/CLI/rotinas/fontes/Sistema/voz/Guardian/produção mantêm-se abertos. Evidência e limites: `.build/pet-goal-execution-next-r1/RECURSIVE-INFERENCE-R9C2F-FINAL-VERIFICATION-F1.json`.


### R9C2G — resultado derivado de origem recursiva: entrega e utilização (04/11, contrato antes do código)

G1 estende o percurso existente ResultReference1/NativePlanReview/ResultDisclosure ao resultado próprio de Admission19/Context6/Selection4. Reutiliza `pet.resultReply` v1: os argumentos continuam a significar o excerto autenticado do resultado próprio para o remetente da referência selecionada; Policy2 vincula explicitamente origem recursiva7, contexto6 e destinatário/configuração. Não aceita uma origem7 no antigo Policy1/Envelope6.

Formatos condicionais G1: NativePlanRevision/Seal3, Continuation9 (fecho original da resposta19 para revisão DATA) e10 (proposta3 para divulgação); Goal42 da proposta/43 da admissão20; Grant33 (resposta com revisão3),34 (entrega),35 (pool pai com reserva8); Funding8/Reservation8; ResultDisclosure Preview/Policy/Published2, Envelope/Ledger9. ResultReference1/ModelResult1 mantêm bytes e hashes. A revisão DATA não cria orçamento, worker ou divulgação. Uma segunda revisão/consentimento BoundedResultDisclosure, distinta do consentimento R43, admite exatamente uma entrega/zero modelos.

A assinatura vincula excerto/offsets/hash/cobertura e proveniência do resultado, causa estável, origem7 e remetente original/configuração, raiz de correlação, prazo e dependências selecionadas. O writer revalida tudo imediatamente antes do efeito e conserva a invalidação observada; reativar uma fonte/pet não revive uma revisão retirada. Memória bruta, históricos e autoridade não são enviados. Dados recebidos/modelo nunca aprovam a política.

O consumo conhecido/incerto da resposta19 + a nova ferramenta/saída cabem na mesma reserva/prazo. Funding8 autentica o prefixo e troca o binding original por CAS; reserva sem ferramenta restante é recusada. Não reabre Settled/CancelRequested nem repete entrega após ACK perdido. Usa o mesmo Authority, fila, recursos, Driver, fecho original e reconciliação; cancelamento ancestral chega ao sucessor e aos restantes descendentes sem libertar workers vivos.

O destinatário pode inspecionar o resultado9 e a proveniência como dados não fiáveis; não obtém execução nem uso por modelos automaticamente. G2 completará a adoção própria e o consentimento R43 do resultado9 pelo destinatário, sobre o mecanismo comum de resultados recebidos, sem copiar memória bruta ou saldo do remetente. Esta sequência não elimina os critérios integrais de cooperação/autonomia. Frontend apenas para seleção, revisões, consentimento, estado e observação funcional.

Aceitação G1: protocolo sintético e Tauri privado com resposta19, recusa da divulgação sem consentimento próprio, duas fases de revisão, uma entrega9 ao remetente exato/correlação da raiz, zero nova inferência, soma de saldo, revogação/troca de destino e revisão/UTF8/argumentos indevidos recusados, reinício, ACK, leitor anterior e backup/restauro. Não é aceitação humana/instalada ou provider real. Sem novos agentes, quota real, commits/publicação/instalação ou host.


### R9C2G1 — resultado recursivo revisto e entregue integrado1149 (2026-11-04T13:48:38.598891+00:00)

G1 integrado: proposta e entrega autorizada do resultado próprio recursivo pelo mesmo runtime. R9C2G2: adoção e utilização própria pelo destinatário do resultado recursivo9, com proveniência/revisões exatas e novo consentimento R43 para o destino/modelo/corpo reais. Reutilizar o percurso recebido Result6, Authority/Store/contexto/Driver e orçamento próprios; resultado9 não concede uso por modelos nem renova o saldo do remetente. Depois restantes TEAM/autonomia/providers/rotinas/fontes/Sistema/voz/Guardian e produção. Não confundir G1 de divulgação com conclusão de G ou do objetivo global. Evidência, comandos, fontes e limites: `.build/pet-goal-execution-next-r1/RECURSIVE-RESULT-R9C2G1-FINAL-VERIFICATION-F1.json`.


### R9C2G2 — utilização própria do resultado recursivo (04/11, contrato antes do código)

Estende a adoção DATA e o R43 existentes ao resultado9/Policy2/Grant34, sem outro scheduler, ledger de autoridade ou armazenamento de resultados. O endpoint de resultados recebidos mantém-se: o backend identifica a família pela referência/envelope canónicos; o ACK resolve primeiro a criação original durável, inclusive depois de revogação/expiração. Origem/Proposta5 com propósito receivedRecursiveResultOwnGoalDataOnly, Selection5, Context7, Goal44 antes de seleção/45 com dados, GoalDataPreview/Grant6 e Admission21 são condicionais. Mantém o domínio de request de resultado existente, que já inclui a referência imutável e o clientRequest; não permite trocar intent/critério/caller ou família. Os formatos3/selection3/result6 e seus leitores conservam a validação anterior.

A proveniência autentica Policy2, excerto/recibo/UTF8, entrega9, remetente/destinatário, raiz de correlação, fonte7 e todos os ancestrais. O mesmo percurso limitado de linhagem ganha um modo exclusivamente DATA que aceita Bound ou Settled sem cancellation com binding exato e donos/fechos canónicos; o modo de execução/financiamento mantém a recusa de Settled. Ler um resultado após liquidação não reabre a reserva nem devolve saldo. Cancelamento/retirada ancestral ou das referências originais impede novos usos, inclusive durante IO. Revogação observada é definitiva para a captura; não apaga dados já entregues.

O destinatário define objetivo/critérios/plano e contexto próprios, escolhe explicitamente o resultado mínimo e recebe revisão/consentimento R43 próprios para referências/revisões/destino/modelo/corpo real antes do hash/reserva. Admission21 admite uma resposta finita no motor local configurado suportado, uma chamada/zero ferramentas, sem continuação, redistribuição ou conclusão sem prova. Usa orçamento próprio, nunca Funding do remetente; prazo não ultrapassa a fonte. Contexto persistido conserva seleção/proveniência, sem cópia de histórico, memória bruta, leases/controllers ou credenciais da origem. Nenhum fallback de provider/modelo/endpoint/conta.

Inbox7 projeta resultSource9 apenas pelo novo percurso autenticado, sem criar tarefa nem conceder uso por modelos. Revisões são NativeUser; TaskRuntime só executa a admissão própria original; ProviderModel não prepara/admite. O frontend serve seleção/revisões/consentimento/estado e conserva a escolha original durante esperas/ACK. A fronteira existente de resultado recebido6 permanece funcional.

Aceitação: ambos os protocolos locais sintéticos, depois de liquidação e com ancestrais ainda vinculados; consentimento ausente/destino/modelo/corpo trocados, forja de dono/revisão/família, fonte/ancestral revogado antes/durante HTTP, consumo incerto sem replay, orçamento próprio sem mutação dos saldos do remetente; Tauri privado com contexto/revisões/execução e janela fechada, reinício, ACK de dados, leitor G1 anterior e backup/restauro histórico. Não é aceitação instalada/humana, provider/quota real ou conclusão dos49 requisitos integrais. Sem novos agentes, commits/publicação/instalação ou alterações do anfitrião.


### R9C2G2 — utilização própria do resultado recursivo integrada1150 (2026-11-04T14:25:40.450511+00:00)

G1+G2 integrados: resposta recursiva, divulgação própria e análise própria do destinatário com R43 e saldo independentes. R9C2H: inventariar e executar a próxima lacuna de cooperação autónoma/paralelismo sobre o scheduler, reservas, recursos e autoridade comuns. Confirmar o que já funciona em fan-out/irmãos e o que ainda requer decisões manuais; fechar o percurso utilizável de paralelismo limitado/arbitragem e cancelamento por ramo sem criar outro runtime nem conceder permissão por delegação. A seguir ferramentas/planeamento de cooperação e restantes49 critérios de providers, rotinas/fontes, Sistema/voz/Guardian e produção. G1+G2 fecham o percurso delimitado de resultado recursivo, não TEAM nem o objetivo global. Evidência, comandos, fontes e limites: `.build/pet-goal-execution-next-r1/RECURSIVE-RESULT-USE-R9C2G2-FINAL-VERIFICATION-F1.json`.


### R9C2H1 — arranque explícito de trabalho finito na fila comum (04/11, contrato antes de código)

A escolha opcional executionStart integra a AdmissionSpec assinada existente e a sua revisão humana; não cria outro ledger, scheduler, orçamento ou admissão. ExecutionStart1 contém mode=whenAvailable e scope inferido nativamente: finiteResponse ou resultDisclosure. O caller apenas pede o modo; não fornece uma política pronta nem a família. Goal46 é obrigatório quando qualquer admissão contém a nova escolha. Campos ausentes mantêm bytes, formatos e comportamento antigos; leitores anteriores recusam o campo desconhecido/Goal46 sem o destruir. O consentimento manual de turno/CLI ou uma admissão antiga não autoriza este modo.

finiteResponse exige motor local explícito suportado, uma chamada/zero ferramentas, plano próprio de resposta sem operações e os dados/R43 próprios quando selecionados. Abrange a resposta local simples2 e as respostas próprias12/13/15/19/21, sem alterar os limites e critérios dessas famílias. resultDisclosure exige admissão14/20 e o consentimento separado exato BoundedResultDisclosure, uma entrega/zero inferência. A primeira fase não promove os pools/delegações literais17/18 nem planeamento/ferramentas gerais: falta-lhes o percurso de Driver/decisão próprio, a completar depois. Planeamento3 e divulgação de estado9 conservam a política de arranque anterior.

O dispatcher comum ACKa a escolha original Issued/neverClaimedZero, estabiliza a publicação e captura PetId/GoalId/contexto/destino/modelo/SpecHash/proof antes de esperar. Usa a mesma fila de recursos, Waiting, claim original, Driver e drenagem. Decisões/Stop conservam prioridade, não aguardam a fila normal. O recomeço pode retomar apenas a intenção original ainda não reclamada e válida; lease, controller ou IO incerto não são reconstruídos. Trocar janela/pet/modelo não redireciona trabalho; não renovar prazo, saldo, consentimento ou grant. Fontes/destino/modelo são revalidados antes do efeito; sem fallback.

A revisão apresenta claramente o arranque pedido; a escolha fica capturada antes de readiness/esperas. Busy/ACK incerto preserva exatamente a aprovação original, incluindo executionStart. Nenhum campo de confirmação pode acrescentar ou retirar esse modo. A interface distingue fila, espera, execução, fecho e resultado por verificar. ProviderModel não prepara/admite; TaskRuntime só executa a escolha nativa assinada.

Aceitação: manual antigo não arranca; JSON coerente/forjado não promove autoridade; dois trabalhos próprios independentes e conflito de recurso mantêm dono e orçamento; Stop/expiração/revogação e resultados tardios não reabrem waits/holds; reinício válido antes do claim e nenhuma repetição depois de claim/efeito incerto. Controlos Tauri privados revistos, janela fechada, corpo/hash e Driver originais, leitor anterior e restauro histórico sem renovar execução. Testes só dirigidos, protocolos sintéticos gratuitos; sem host/autostart/serviços reais, agentes adicionais, commits/push/merge/instalação. O backend comum tem de estar disponível; não se promete trabalho com a máquina desligada. Mantêm-se43+6 critérios integrais abertos.


H1 — arbitragem antes do arranque: a leitura final identificou que tick aguardava a resolução de readiness de um objetivo antes de avançar o seguinte. Um endpoint lento podia atrasar trabalho independente. A mesma Waiting passa a conservar a ocupação mecânica advancing, limitada a8 tarefas de avanço; não é autoridade nem outra fila. O avanço retém o pedido/hold original até terminar a observação, mesmo após Stop. Atualizações tardias só podem tocar o mesmo claimRequest + admissão assinada. Nenhuma decisão/Stop espera pelo IO desses avanços. Provar limite, late-result fencing e independência antes da integração.


### R9C2H1 — checkpoint privado, integração pendente (2026-11-04T15:21:59.552857+00:00)

Raiz1150 e instalação32 preservadas. H1 acrescenta a escolha assinada de arranque finito e avanço limitado na fila existente. Corrigidos o descarte de start na bridge, o bloqueio de consentimento por recurso ocupado e a observação de Busy durante IO, mantendo Stop, dados, destino e orçamento originais. Provas web/Rust dirigidas passaram; a prova Tauri657 é parcial e falhou antes da revisão do terceiro trabalho. Reinício, leitor anterior e restauro deste novo modo ainda não foram aceites; H1 não foi integrado. Fontes, comandos, resultados, falhas e próxima unidade: .build/pet-goal-execution-next-r1/requested-execution-r9c2h1-a6/PARTIAL-CHECKPOINT.json. Os43+6 critérios integrais mantêm-se abertos.


### R9C2H1 — arranque finito pedido integrado1151 (2026-11-04T15:51:28.798761+00:00)

Escolha inicial assinada executionStart/Goal46, fila comum com até8 avanços independentes, dono/destino/modelo/prazo originais e Stop sem libertação fictícia do worker. Corrigidos Bridge, confirmação sob Busy, observação limitada do IO e resolução finita após reinício. Entrega direta/recursiva mantém consentimento próprio e zero inferência. Native663 privado passou: ambos protocolos sintéticos, espera/cancelamento, vista fechada, hash real, reinício após recuperação governada do titular antigo, leitor anterior e backup/restauro histórico. Nenhum claim antigo foi saltado ou repetido; nenhuma aceitação humana/instalada/hardware é afirmada. Fontes/provas/limites: [verificação](../.build/pet-goal-execution-next-r1/REQUESTED-EXECUTION-R9C2H1-FINAL-VERIFICATION-F1.json). Instalação32 preservada;43+6 integrais abertos.

Próxima unidade: R9C2H2: inventariar e implementar o Driver comum de planos nativos/determinísticos e delegações que ainda exigem arranque manual. Reutilizar Authority/Store, receitas/decisões, recursos, orçamento, recibos e fecho existentes; nenhum segundo scheduler/ledger, promoção de concessões antigas ou permissões por delegação. Confirmar famílias4/6/10/11/16/17/18, continuação e pools realmente suportados, capturar plano/dono/destino antes de esperar e registar o contrato antes do código. Demonstrar conflito, progresso independente e cancelamento por ramo com saldo original. Reconciliação autónoma de claims anteriores permanece dependência própria; não saltar a recuperação governada para forçar paralelismo. Depois planeamento/ferramentas gerais e restantes49 critérios de providers, fontes/rotinas, Sistema/voz/Guardian e entrega.


### R9C2H2 — contrato inicial do Driver de planos nativos (antes do código)

A inspeção de1151 confirma que planos de mensagens/respostas/delegações nativas ainda são manuais: Driver::new não os aceita. Reutilizar deterministic::plan, message_controller (review/send/ACK), reservas existentes, verificação nativa e fecho do Driver. Uma proposta ou um orçamento não autoriza os dados da mensagem. Acrescentar uma política finita própria na admissão assinada, com revisão dos passos/argumentos/destinatários/fontes exatos e consentimento de dados separado do arranque. O modelo não pode preparar ou aprovar esta autoridade.

O modo solicitado deriva nativamente da política; ausência mantém o comportamento manual antigo. Usar formato Goal distinto do46 para esta extensão e provar recusa pelo leitor anterior sem destruição. A execução não invoca inferência; reservas de modelos de um pool são apenas dos descendentes. Cada destinatário mantém a admissão própria. WorkDisclosure/resultados mantêm os seus contratos específicos; relatórios dinâmicos exigem captura limitada explicitamente revista. Continuação só executa os passos restantes autenticados; ACK incerto conserva a aprovação original e não repete a entrega. Claims antigos mantêm recuperação governada, com incerteza e filhos preservados.

Inventário/decisões e próxima leitura: .build/pet-goal-execution-next-r1/NATIVE-DISPATCH-R9C2H2-DEPENDENCIES.json. Estado: desenhado/inventariado; implementação ainda por iniciar. Esta unidade não fecha TEAM, autonomia geral ou os43+6 requisitos integrais.

H2 — representação decidida antes da implementação: nativePeerPlan1 opcional na AdmissionSpec, com dono/contexto/plano/revisão originais, passos restantes tipados e destinatários/configurações/fontes exatos; Goal47 quando presente. BoundedNativePeerPlan é um âmbito próprio no campo de consentimento existente e não é intercambiável com memória, estado futuro ou resultado. executionStart.scope=nativePeerPlan exige essa política. Inclui mensagens, respostas e delegações nativas já suportadas e continuação autenticada, sem inferência do pai. WorkReport1 fixa estado/revisão/dados e admite apenas a hora da nova observação; WorkDisclosure2 e ResultReply conservam as suas políticas específicas. Capturas de destinatário/fonte retiradas não são reativadas; Busy de leitura não é autorização. O Driver retém um envelope original por passo e consulta o ACK antes de qualquer tentativa, ignora apenas passos cobertos por recibos/carry autenticados e usa a verificação/fecho existentes. Critérios arbitrários ficam por verificar; não se permite sucesso por texto do modelo.


### R9C2H2 — planos nativos revistos integrados1156 (2026-11-04T16:34:11.217358+00:00)

O Driver comum executa planos peer exatos com escolha inicial assinada e consentimento DATA boundedNativePeerPlan separado. Goal47 vincula argumentos, destinatários/configuração, fontes/revisões e prazo; nenhum destinatário recebe execução ou uso de modelos por delegação. Mesmos orçamento, recursos, recibos, retoma e fecho, com pedido estável distinto por passo. Fixtures exercem identidades, financiamento, recursão, ACK e carry; Native665 privado comprova revisão funcional, progresso independente, espera/reinício após recuperação governada, leitor anterior e backup/restauro histórico, sem chamadas a modelos. [Fontes, provas e limites](../.build/pet-goal-execution-next-r1/NATIVE-PEER-PLAN-R9C2H2-FINAL-VERIFICATION-F1.json). H2 não fecha planeamento/cooperação/autonomia integral nem aceitação instalada;43+6 permanecem abertos e mochi32 preservada.

Próxima unidade: R9C2H3: inventariar a ligação do planeamento aos planos nativos de cooperação e à verificação dos resultados dos destinatários. Confirmar o catálogo e parser realmente utilizados, as revisões de dados por destino e os limites do executor; escolher a próxima lacuna implementável sem outro scheduler, aprovação por texto ou partilha implícita de dados. H2 automatiza apenas planos exatos previamente revistos. Autonomia geral com ferramentas, coordenação configurável, providers nativos, fontes/rotinas, Sistema/voz/Guardian e aceitação de produção continuam dependências abertas.


### R9C2H3A — decisão nativa de plano peer no transporte comum (contrato antes do código)

A inspeção1156 encontrou que Source/Decision e a cedência de recursos de planeamento dependem do executor Spotify. Antes de acrescentar propostas de IA ao catálogo, ligar uma revisão peer exata ao transporte nativo de decisões já existente. O arranque manual explícito de uma admissão determinística pode preparar uma única decisão para todos os passos pendentes do plano original. Isso não promove arranques antigos nem cria outra admissão: sem escolha humana exata não há envio. A revisão inclui causas, argumentos, envelopes, fontes, snapshots, destinatários/configuração, orçamento e prazos. Aprovar esse pedido autoriza só essas entregas locais; não concede uso por modelos ou execução aos destinatários.

A origem tipada goalPeerPlan separa a decisão peer da receita Spotify, mantendo o mesmo broker, alvo original, consumo único, Stop e superfícies chat/voz/terminal. Retêm-se os envelopes e aprovações exatos após a decisão; ACK incerto não gera outros IDs nem repete efeitos. Recibos cobertos são factos, não novo saldo. Esta fase usa a reserva nativa existente, sem inferência nem espera de recurso físico de modelo, e não ativa automaticamente admissões manuais. O Driver usa os mesmos send_message, verificação e fecho. Critérios livres permanecem por verificar.

Esta unidade entrega um percurso utilizável de revisão de sequência nativa; a extensão posterior do catálogo de planeamento tem de reutilizá-lo e os outputs/proveniência existentes. Não alargar o parser para obter efeitos antes dessa ligação e das revisões de dados R43.


### R9C2H3A — revisão de sequência peer integrada1159 (2026-11-04T17:22:30.520238+00:00)

Arranque manual explícito de planos nativos pode usar uma decisão exata para todos os passos pendentes no broker comum. Origem transitória goalPeerPlan, lease/dono e envelopes originais vinculados; sem novo formato persistente, inferência, execução do destinatário ou promoção automática de admissões antigas. Source/Decision, Driver, recibos e fecho são reutilizados. A confirmação final dos recibos trata Busy conhecido com a prova imutável original, sem reenvio. Native673 privado passou alvo/replay, recusa/Stop e progresso independente; leitor H2 preserva compatibilidade. [Provas, fontes e limites](../.build/pet-goal-execution-next-r1/PEER-SEQUENCE-R9C2H3A-FINAL-VERIFICATION-F1.json). Instalação32 e49 requisitos integrais continuam separados.

Próxima unidade: R9C2H3B: ligar propostas de cooperação geradas por IA ao catálogo/outputs/journal existentes e à decisão peer tipada H3A. Primeiro confirmar o seed, generated_candidate/ACK, orçamento/feedback e a verificação nativa de um plano gerado, incluindo critérios livres. Preservar seleção de dados/R43 para o motor explícito, revisão própria dos destinatários/dados antes de mensagens e autoridade própria dos destinatários. Não alargar apenas o parser, reconstruir approvals de JSON ou usar nova fila/ledger. Depois restantes critérios de ferramentas/autonomia/coordenação/providers/fontes/rotinas/Sistema/voz/Guardian e produção.


### R9C2H3B1 — catálogo de correspondência gerada, contrato antes do código

Adicionar PeerCorrespondenceV1 ao planeamento comum, selecionado explicitamente por nativeUser, mantendo SpotifyTransportV1 e os seus bytes legados. A primeira composição do catálogo suporta uma ronda de proposta, até aos passos/bytes/orçamento originais, para pet.message, pet.reply v1 e pet.workReport v1. Não inclui redistribuição de orçamento por pet.delegate: essa extensão posterior tem de contabilizar inferência própria e escrow dos descendentes no mesmo saldo. O objetivo integral de delegação/cooperação não fica reduzido a esta unidade.

A proposta continua no output/journal/publicação+ACK assinados existentes. A decisão H3A revê apenas o plano gerado autenticado, envelopes, fontes, destinatários/configuração e proveniência do modelo/contexto selecionado. Consentimento R43 do motor não é consentimento de envio: exige a decisão própria e verifica revogação dos dados derivados até à publicação. Ceder/retomar o recurso físico enquanto aguarda decisão usa a prova zero-worker e o ledger comuns. Sem outro scheduler, motor implícito, permissões de música ou autoridade concedida pelo texto do modelo.

Formatos condicionais previstos: Goal48 apenas para o catálogo novo; Grant36 com marcador peerPlanning derivado da admissão; Scope.generatedPlan apenas em correspondência gerada; Binding3/Report3 para critérios nativos do plano gerado. Referências de publicação e resultados continuam as existentes. Leitores anteriores recusam o catálogo/formato incompatível sem o destruir. Critérios livres mantêm Unknown; só o critério literal nativo de persistência local pode ficar Met pelos recibos, nunca pela resposta do modelo nem pelo alegado sucesso do destinatário. O corpo real da inferência e a reserva continuam no percurso R43 vigente.


B1 — barreira de proveniência antes de inferência: o percurso de correspondência gerada começa com contexto dedicado sem memórias/fontes selecionadas ou histórico copiado. Foi identificada uma dependência concreta: mensagens simples recebidas não conservam ainda a linhagem de revogação de memória que os envelopes de resultado6/9 já conservam. Não enviar dados selecionados como mensagens simples nem descartar silenciosamente a seleção. Recusar essa composição antes da chamada ao modelo até a extensão de linhagem e utilização pelo destinatário reutilizar Dependencies/R43. Os percursos R43 e de resultado existentes mantêm-se; completar essa dependência continua obrigatório para o objetivo integral.


### R9C2H3B1 — planeamento de correspondência integrado1162 (2026-11-04T18:04:49.035404+00:00)

O catálogo explícito peerCorrespondenceV1 liga uma proposta local de IA à publicação assinada e à revisão nativa exata dos dados/destinatários, usando Driver, orçamento, recibos e cedência/retoma de recursos comuns. Goal48/Grant36 e Report3 distinguem recibo local de conclusão do destinatário: critérios livres continuam por verificar. O percurso funcional foi validado em Tauri privado com Ollama/LM Studio sintéticos; não constitui aceitação humana/instalada. Até existir proveniência/revogação nos envelopes e R43 no destinatário, memória/contexto recebido/histórico selecionados são recusados antes da inferência. Delegação gerada, rondas adicionais e backup/restauro do novo formato continuam pendentes. [Fontes, provas e limites](../.build/pet-goal-execution-next-r1/PLANNED-PEER-R9C2H3B1-FINAL-VERIFICATION-F1.json). Instalada32 preservada;43+6 critérios integrais abertos.

Próxima unidade: R9C2H3B2: completar a proveniência/revogação de dados selecionados nas mensagens peer e a revisão R43 própria do destinatário, reutilizando Dependencies/Context/Authority existentes. Até aí, peerCorrespondenceV1 recusa memória/contexto recebido/histórico copiado antes da inferência. Depois fechar orçamento híbrido de inferência própria e descendentes antes de pet.delegate gerado, feedback/replaneamento limitado, coordenação e restantes integrações/Sistema/voz/Guardian/produção. Novo formato48 também precisa de prova dirigida de backup/restauro do produto; reinício/leitor antigo já verificados, sem os confundir.


### H3B1 — prova adicional de preservação48 (2026-11-04T18:20:29.393937+00:00)

O arquivo do produto criado em685 foi restaurado com revisões separadas de dados, autoridade, configuração e ativação histórica. A prova nativa689 confirma os três objetivos48, grants36 e relatório3 com bytes, admissões e orçamentos originais; classificação histórica e recusa de claim/status/Driver, sem modelos repetidos. As tentativas anteriores de fixture permanecem falhadas; não foram reduzidas as proteções de caminho, mount ou autoridade. [Composição da prova](../.build/pet-goal-execution-next-r1/planned-peer-restore-r9c2h3b1-s1/CHECKPOINT.json). Não cobre crash em execução ou aceitação instalada.


### R9C2H3B2 — memória selecionada em correspondência planeada (contrato antes de código)

A extensão mantém peerCorrespondenceV1 e os seus leitores/dados. Um catálogo explícito peerCorrespondenceMemoryV1 admite apenas contexto dedicado com memória selecionada do próprio objetivo, sem histórico copiado nem contexto recebido transitivo nesta unidade. Reutiliza R43 para o motor/modelo/endpoint originais, seguido de uma decisão nativa distinta sobre o texto realmente proposto e cada destinatário. Todas as referências fornecidas ao modelo são dependências conservadoras dos textos derivados; o modelo não decide que fontes podem ser omitidas.

O runtime sela no grant um snapshot tipado de pet, CID, destino original, hash do contexto e revisões/referências da memória. O envelope versionado liga esse snapshot à publicação assinada/ACK e à tentativa real de planeamento. Mensagens/request locais conservam identidade e orçamento; publicar não concede execução nem uso de modelos no destinatário. A revisão nativa retém a captura de dados antes de esperar; revogação, edição, expiração e cancelamento impedem novos envios mesmo após uma resposta do modelo. Um ACK já canónico continua a ser observado antes de qualquer novo efeito.

O destinatário usa o mesmo percurso de criação de objetivo e R43, com origem/seleção tipadas para DATA derivada. Autentica o recibo e grant canónicos do remetente e verifica as dependências originais antes de materializar conteúdo e imediatamente antes do efeito. Recebe apenas o texto derivado selecionado; registos brutos e controlos de execução do remetente não são copiados para o seu prompt. O seu destino/modelo/endpoint, orçamento e consentimento são próprios. Resposta recebida continua a ser dado não fiável.

Formatos condicionais planeados: envelope10, grant37, objetivo49 para planeamento com memória; origem/seleção recebida6, contexto8, GoalGrant de dados7 e admissão22/objetivo50 para uma resposta local própria com zero ferramentas. Os campos opcionais ausentes e bytes dos formatos anteriores conservam-se; leitores incompatíveis recusam dados novos sem os destruir. Esta numeração é de formatos, não altera os43+6 requisitos. O catálogo novo só será integrado após o percurso completo, controlo funcional e provas; as barreiras atuais não serão removidas por existir um tipo ou parser. Delegação gerada, contexto transitivo e rondas adicionais seguem dependências próprias.


### R9C2H3B2 — memória em correspondência integrada1166 (2026-11-04T19:05:46.675334+00:00)

Catálogo explícito com memória própria selecionada, proveniência/revogações no recibo comum e decisão nativa distinta sobre o texto/destino. O destinatário prepara objetivo/contexto e consente uma resposta local finita com orçamento próprio; não recebe memória bruta, consentimento ou saldo do remetente. Reutiliza R43, Driver, Store, Authority e Dependencies. Native697 e restauro699 passaram com providers sintéticos; não é entrega instalada/humana. [Composição, testes e limites](../.build/pet-goal-execution-next-r1/PEER-MEMORY-R9C2H3B2-FINAL-VERIFICATION-F1.json). Contexto transitivo, delegação gerada e restantes critérios integrais continuam abertos.

Próxima unidade: H3C: verificar e completar orçamento híbrido de inferência própria e filhos no mesmo ledger antes de pet.delegate gerado; manter barreira do catálogo até haver reservas, plano gerado assinado, admissão própria do destinatário, revogação/recuperação e controlo funcional. Depois feedback/replaneamento limitado, coordenação opcional, protocolos de providers, fontes/rotinas, Sistema/voz, Guardian e produção. Contexto recebido transitivo e histórico copiado continuam recusados neste catálogo de correspondência com memória.


H3C em desenvolvimento privado: [contrato de delegação proposta e orçamento híbrido](PET-MESH-CONTRACT.md#r9c2h3c--delegação-proposta-com-orçamento-híbrido-contrato-antes-de-código). Nenhum catálogo adicional integrado/instalado; reutilizar saldo comum antes de admitir inferência própria e reservas dos filhos.


### H3C — delegação proposta integrada1167 (2026-11-04T19:29:13.085603+00:00)

O catálogo peerDelegationV1 junta uma ronda própria de IA às reservas dos filhos no mesmo orçamento/Grant. Verifica o saldo de todo o plano antes do primeiro envio e volta a validá-lo no escritor comum. O destinatário conserva admissão e R43 próprios. Native703 comprovou cancelamento/reconciliação de um ramo enquanto o irmão aguardava inferência, sem alterar a autorização desse irmão; restauro705 conservou apenas autoridade histórica. [Fontes, testes e limites](../.build/pet-goal-execution-next-r1/HYBRID-PLANNED-DELEGATION-H3C-FINAL-VERIFICATION-F1.json). Contexto selecionado/transitivo, colaboração iterativa e aceitação integral continuam abertos. Instalação32 preservada.

Próxima unidade: H3D: completar colaboração iterativa a partir dos resultados próprios/correlacionados dos filhos e feedback verificável, com progresso/limites/revisões no Driver e journal comuns. Inventariar consentimento prévio de tarefas delegadas, proveniência de memória/contexto em delegações e novas rondas sem renovar saldo ou usar respostas como aprovações. Não promover para inferência/delegação os contextos selecionados que os catálogos ainda recusam. Depois coordenação opcional, protocolos de providers, fontes/rotinas, Sistema/voz, Guardian operacional e produção.


H3D1: [evidência correlacionada de filhos](PET-MESH-CONTRACT.md#h3d--colaboração-iterativa-d1-evidência-correlacionada-antes-de-inferência) antes da continuação por IA. Implementar consulta funcional e provas sobre os verificadores existentes; D2 conserva o objetivo de iteração, dados autorizados, orçamento e ausência de progresso.


### H3D1 — evidência correlacionada integrada1170 (2026-11-04T19:53:31.264169+00:00)

Consulta nativa funcional dos filhos exatos de cada reserva, com revisões/hashes e recibos de modelo verificados pela reconciliação comum. Não copia respostas/memória nem concede execução ou uso por modelos; recibo disponível não verifica critérios. Native707 passou sem inferência ou escritas de runtime. [Fontes, testes e limites](../.build/pet-goal-execution-next-r1/PEER-FEEDBACK-H3D1-FINAL-VERIFICATION-F1.json). É base para H3D2; iteração autónoma continua por implementar. Instalação32 e49 critérios integrais preservados.

Próxima unidade: H3D2: ligar evidência correlacionada a uma continuação limitada no mesmo Driver/journal e orçamento. Registar âmbito explícito de uso dos metadados/dados pelo motor, capturar revisões/fontes e corpo real antes do hash/reserva e revalidar antes de IO/publicação. Separar feedback peer do verificador de receitas; não converter recibo/modelo em critérios cumpridos. Descontar reservas de filhos em modelos/ferramentas/bytes, ceder recursos na espera e limitar repetição/ausência de nova informação. Divulgação de texto/memória e admissão pré-autorizada de filhos mantêm contratos próprios; nada de habilitar mais rondas apenas mudando o limite. Depois coordenação, providers, fontes/rotinas, Sistema/voz, Guardian e produção.


H3D2: [captura exata de dados, informação nova e saldo comum](PET-MESH-CONTRACT.md#h3d2--captura-de-dados-e-saldo-antes-da-continuação). Implementação privada; manter catálogos de uma ronda até completar decisão de dados/corpo/journal/espera e provas.

## Coleções persistentes de calendários — contrato09/11/2026

Evoluir o catálogo nativo existente para seleção explícita de várias fontes ICS pelo mesmo pet/CID, reutilizando CAS/consentimento e o percurso multicalendário já funcional. Formato1 conserva seleção única e bytes antigos; formato2 explícito suporta até32 fontes persistentes por dono e recusa downgrade. Limite total33 com anexos,8 consultas/128KiB partilhados por turno; nomes/UIDs iguais não fundem fontes. Gestão funcional no chat revê dono/fontes/âmbito antes de guardar e revoga só a associação escolhida. Não cria regras, sincronização de conta, escrita ou autoridade para objetivos/modelos adicionais. [Contrato e aceitação da unidade](../.build/pet-goal-execution-next-r1/persistent-calendar-collections/CONTRACT.json).
