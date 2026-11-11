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

**Regras — autorização própria validada em privado,10/11/2026:** RuleGrant/reservas partilham a Authority e o writer existentes; revisão funcional, reinício, revogação e restauro de duas gerações passaram sem inferência. A execução das ocorrências ainda não está ligada, a instalação continua43 e AUTO-01/02 permanecem parciais. [Contrato, provas e próxima dependência](PET-RULES.md).

**Regras,10/11/2026:** criação/edição/leitura de rascunhos por pet e prévia IANA estão integradas no binário privado47/Source3. Janela nativa verificou as duas escolhas da hora repetida em Lisboa, revisão/CAS, recusa de ativação forjada, conservação após reinício e zero inferência/grants. Backup/restauro inativo preservou os bytes do novo catálogo. A ativação recorrente, reserva total, ocorrências executáveis, fontes futuras e entrega instalada permanecem pendentes. [Prova](PET-RULES.md).

**Barreira temporal integrada,09/11/2026:** binário privado build46/Source2 conserva e executa a resposta finita após a hora UTC revista; tentativa antecipada, reinício, backup/restauro e leitura histórica passaram. O leitor privado anterior44 recusa Goal66 sem o alterar. Nenhum modelo real/quota paga; instalação normal43 preservada. AUTO-01/02 permanecem parciais: faltam regras, concessões futuras e ocorrências. [Provas e limites](PET-SCHEDULED-EXECUTION.md).

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

**Estado corrente — candidata41; instalada38 (09/11/2026):** o destinatário financiado já usa o planeador comum com consentimento próprio e a reserva original. Admission26/Funding9/Goal61/Grant47 conservam destino, modelos, origem, prazo e saldo; subdelegação16/Payload2 conserva ancestrais autenticados. As provas dirigidas exercitaram dois protocolos locais sintéticos, especialista literal sem modelo, segunda ronda e reconciliação do consumo; recusam ciclo, profundidade, excesso de orçamento e outra admissão concorrente. Cancel durante HTTP guarda consumo incerto e não publica nem repete o efeito. Controlos funcionais, Driver26 e ACK de criação61 estão ligados às superfícies existentes. A família runtime afetada e web/tipos passaram; mesmo RPM39 está preparado. Pela CLI39, backup/restauro dos formatos28/60/61 preservou bytes e seis assinaturas em dois perfis sintéticos; leitor38 recusou sem mutação e runtime39 consultou apenas histórico, sem autoridade antiga. Não comprova recuperação de executor61. A janela abriu em perfil gerado, mas a sessão Hyprland estava bloqueada;0ações gráficas e instância QA recolhida. Faltam janela61, reinício e instalação39; a atualização local já está autorizada.38 continua operacional; sem commits/publicação ou alteração persistente do host neste incremento. [Estado/provas/limites](../.build/pet-goal-execution-next-r1/funded-recipient-planning/CHECKPOINT.json). [Preservação61 e limites](../.build/pet-goal-execution-next-r1/funded-recipient-planning/PRESERVATION61-VERIFIED.json). Recuperação61 validada na raiz posterior à candidata39: fonte financiadora é revalidada sob o guard de autoridade ao capturar a retoma e antes de publicar o novo dono. Perda/recolha real do processo nos dois protocolos sintéticos preservou lease/escrow e enviou só a mensagem pendente, sem HTTP novo; cancelamento após preparar o worker recusou a publicação. Memória selecionada, contexto recebido e retoma base mantiveram as provas afetadas. Não está no RPM39 nem instalada, não abre continuação após fecho controlado e não equivale a aceitação da janela. [Fontes e prova de recuperação61](../.build/pet-goal-execution-next-r1/funded-planning-recovery/VERIFIED.json). Composição41 compilada/empacotada com a recuperação61, painéis nativos de Sistema, ausência de reclamação de widgets, [concessões próprias de Som](PET-SYSTEM-ACCESS-CONTRACT.md) e identidade lógica sem modelo pela fábrica. Backup/restauro nativos41 conservaram concessões de dois pets; projeção operacional suspendeu-as; leitor40 de configurações operacionais recusou-as sem mutação. O backup40 pode preservar dados opacos sem os ativar. A correção de identidade passou nos casos de fábrica, donos e runtime determinístico. Instalada38 permanece inalterada; atualização local autorizada, registo assinado sysworkspace e aceitação funcional/instalada pendentes. Depois da41, a raiz1236 adota estas concessões na voz NativeUser, antes da inferência, com alvo/revisão/sessão originais e recusa de fallback Spotify quando há política própria. Fixtures nativas passaram; captura física de áudio ficou ignorada e aceitação real/instalada pendente. TaskRuntime/ProviderModel, journal de tarefa para operações diretas e migração integral do legado continuam abertos; Guardian operacional continua pendente. [Prova1236](../.build/pet-goal-execution-next-r1/system-sound-voice-adoption/VERIFIED.json). [Fontes/provas/limites41](../.build/pet-goal-execution-next-r1/release41-native/CHECKPOINT.json).

**Última entrega local — memória própria e cooperação (08/11/2026):** implementada na raiz e entregue localmente na mochi.37. Native build14 demonstrou seleção/F2 própria, admissão24/Goal55/Grant45, delegação14, destinatário57 determinístico concluído sem modelo, revisão DATA7698bytes idêntica ao HTTP e entrega13 na segunda ronda. Pai termina pausado com critérios por verificar; ambos os grants fechados e orçamento original preservado. Testes nativos cobrem também destinatário56 com F2 próprio, os dois protocolos locais sintéticos e revogação; regressão de pilha corrigida sem aumentar stacks. Controlos existentes adotados; indicação comum de voz corrigida após observação nativa. Candidata15 compila e passou backup/restauro inativo55/57, recusa do leitor36 sem mutação, consulta histórica sem autoridade e reinício dos estados fechados. A retoma55 após perda/recolha real do processo também passou com as revisões/saldo originais, sem HTTP novo, e recusa por revogação. O controlo nativo56 passou: pedido13, objetivo próprio56/admissão22, binding LM Studio explícito, F2 e saldo próprios, uma resposta finita persistida e worker drenado sem conclusão falsa. Remetente55 também parou/fechou; exatamente duas chamadas HTTP sintéticas no total. O ensaio não guardou corpos HTTP brutos; a exclusão da memória do remetente foi observada na composição revista e comprovada nos testes Rust. Corrigido depois o rótulo pedido/resultado, sem alterar guardas. O RPM37 exato está instalado/ativo no perfil habitual; arranque/versão/Pets e preservação44/45ficheiros confirmados, só o log mudou. Backup nativo verificado,36 preservada; registo Git host pendente da assinatura local já indisponível. Sem commits/push Coucou. [Checkpoint, fontes, testes e limites](../.build/pet-goal-execution-next-r1/iterative-selected-memory/CHECKPOINT.json).

**Estado vigente — admissão iterativa integrada na raiz (08/11/2026):** a revisão inicial explícita,2–8 rondas próprias, saldo comum e capacidades por caller usam o percurso público normal. A build13 percorreu a janela Tauri real com fornecedor sintético: uma delegação, destinatário determinístico concluído sem modelo, revisão DATA própria e segunda inferência; o pai ficou Pausado/limited, com fecho assinado e critérios livres por verificar. O corpo6389bytes revisto corresponde ao hash HTTP enviado; modelo/destino/saldo/prazo originais mantidos. Corrigidas captura passiva após cedência e preparação DATA com readiness envelhecida; efeitos continuam sujeitos a verificação ativa e revogação. Provas97web,19nativas afetadas,12writer e dirigidas reutilizadas conforme as fontes; sem repetir803/0/10. Instalada36 operacional/verificada, com este delta; commit host pendente de desbloqueio da chave de assinatura; atualização local autorizada, commits/push Coucou suspensos. O aviso de listagem foi separado do feedback da operação e desaparece após leitura recuperada;39casos afetados/TSC passaram. É entrega local de desenvolvimento; não é aceitação humana/provider real, Guardian operacional ou fecho dos49 requisitos. [Fontes, provas e limites](../.build/pet-goal-execution-next-r1/iterative-native-admission/CHECKPOINT.json). Os registos seguintes são históricos.

**Entrega local de desenvolvimento — mochi.34 (08/11/2026):** RPM nativo da raiz1224 ativado pelo instalador restrito e gestor HOME existentes, após Quit normal da33 e backup nativo verificado. Confirmados executável/hash, dono D-Bus, versão visível e página Pets no perfil habitual; prefixo/artefactos33 preservados. Dos45 ficheiros acompanhados,44 mantiveram-se iguais e apenas o log mudou; nenhum desapareceu (modelos grandes comparados por metadados). A nova iteração permanece recusada na admissão pública. As duas falhas intermitentes de handoff e uma posterior de continuação finita continuam abertas; os ensaios isolados passaram, sem causa demonstrada. Entrega autorizada de desenvolvimento não equivale a aceitação privada/global. Hooks sysworkspace passaram, mas o commit focado ficou pendente por falha real de autenticação/assinatura YubiKey; fontes e índice mantidos, sem bypass. Commits/push/merge Coucou suspensos. [Instalação e limites](../.build/pet-goal-execution-next-r1/release34-native/INSTALLED34-VERIFIED.json) · [Registo pendente](../.build/pet-goal-execution-next-r1/release34-native/HOST-REGISTRATION-PENDING.json).

**Recuperação do planeamento verificada na janela nativa — 08/11/2026:** a raiz implementa a retoma revista pela mesma admissão/lease/Authority, Pending e manutenção. Na build8, com perfil gerado fiel à política, modelo/destino e inodes originais, foram observados revisão/recusa/confirmação, arranque separado, nova aprovação da receita e entrega só da mensagem pendente. Nenhuma chamada ao modelo foi repetida; saldo, journal, recibos e reservas originais conservaram-se. O objetivo parou para verificação, sem conclusão falsa; reinício e Cancel pela mesma UI preservaram consumo e cancelaram o filho. Backup/restauro do produto preservou Goal54/Grant44; o leitor33 recusou sem alterar dados e o runtime restaurado consultou2 objetivos/3 mensagens apenas como histórico, recusando a autoridade antiga sem escrever. Reutilizadas as provas dirigidas anteriores; apenas4 ficheiros cfg(test) mudaram face à build8. Não é aceitação humana/provider real, entrega instalada ou fecho dos49 critérios. A candidata34 compilou, mas a regressão encontrou um estouro da pilha padrão no percurso de dados derivados; a correção de4MiB por executor passou pelo worker nativo, e o RPM34 corrigido abriu a página Pets com identidade normal. A regressão obteve800 casos passados,2 falhados e10 ignorados; os5 casos de retoma passam isoladamente, pelo que a interação no processo alargado continua por diagnosticar. Esta fotografia precede a entrega34 registada no parágrafo seguinte; a investigação intermitente e a separação da readiness continuam abertas, sem abertura da política pública. [Fontes, provas, limites e próxima ação](../.build/pet-goal-execution-next-r1/parent-planning-recovery/CHECKPOINT.json).

**Entrega local concluída — mochi.33 (08/11/2026):** a composição1219 foi empacotada nativamente e o mesmo RPM validado está ativo no perfil habitual. Confirmados executável, dono D-Bus, versão visível e página Pets; a32 e os seus artefactos permanecem preservados. O backup nativo Coucou ficou verificado; dos45 ficheiros existentes observados,44 mantiveram bytes, nenhum desapareceu e apenas o log mudou. A instalação ficou registada no commit assinado `11c2e90e2829a8da3578297b9e20bfd54bf81244` de sysworkspace, com hooks aprovados; commits/push/merge Coucou continuam suspensos. As provas nativas anteriores de recuperação/Pausar/restauro foram reutilizadas no âmbito das fontes exatas. Isto não comprova aceitação humana/áudio, Guardian operacional ou conclusão dos49 critérios. Segue a continuação governada do planeamento após perda do executor. [Composição, instalação e limites](../.build/pet-goal-execution-next-r1/release33-native/FINAL-CHECKPOINT.json).

**Integração local1219 — 07/11/2026:** promovidos81 ficheiros exatos (61 atualizações com backup por ficheiro,20 adições); preservados os10 ficheiros Gravity alheios ao mapa. Compilação nativa da própria raiz passou em TSC/Vite/Rust. B3L comprovou backup/restauro do produto em6 perfis/12 objetivos assinados, recusa do leitor antigo para Goal52 e revisão histórica dos formatos28/30 conhecidos, sem destruição ou reativação; runtime restaurado recusa a autoridade antiga e consulta mensagens só como histórico explícito. B3M acionou Pausar pelo Tauri/AT-SPI real: objetivo pausado, Stop assinado e fecho/lease/consumo/reservas originais intactos; Reabrir rascunho ausente. A preparação dessa fixture UI não é um restauro operacional. Instalada32 ainda preservada; segue preparação da atualização local já autorizada pelo procedimento sysworkspace. Admissão pública da nova iteração permanece fechada; continuação do planeamento após perda do Owned, Guardian, Sistema/voz e49 critérios integrais não estão concluídos. [Composição, provas e limites1219](../.build/pet-goal-execution-next-r1/integration1219-native-recovery-h3d2/FINAL-CHECKPOINT.json).

**Incremento funcional B3J/B3K — 07/11/2026:** o binário nativo build3 preservou byte a byte o payload do perfil sintético criado pela app em backup/restauro inativo; hash errado e destino ocupado foram recusados, ACK repetido não reescreveu o par. Este perfil não contém autoridade H3D2 assinada. Corrigida a vista: Pausar fica disponível para a execução recuperada com lease original e sem legado; Reabrir rascunho deixa de aparecer quando existe lease ainda em recuperação. Reutilizada a mutação nativa, sem alterar autoridade, saldo ou formato. Os36 testes afetados da vista e TSC/Vite/Rust da build4/1218 passaram. Raiz1170/instalada32 preservadas; atualização local está autorizada, pendem compatibilidade/restauro H3D2 assinado, leitores anteriores e controlo Tauri nesse estado. [Fontes, provas e limites](../.build/pet-goal-execution-next-r1/recovery-controls-b3k/PARTIAL-CHECKPOINT.json).

**Composição nativa B3I — 07/11/2026:** build3/1218 corrige hidden no Hub e inclui os29 sons partilhados com hashes iguais à fonte atual. Observados no desktop: Tarefas mostra objetivos, Nomes/perfis oculta o editor inativo, identidade/configuração fica acessível e o rascunho não guardado permanece no regresso à aba. O aviso HTML como áudio desapareceu; não é aceitação de áudio físico/voz. Backend Rust é o mesmo validado em B3H (3+9+69), sem repetir suites por CSS/recursos. Instalada32 e raiz1170 preservadas; integração/compatibilidade/restauro e revisão funcional completa ainda pendentes. [Fontes, build e limites B3I](../.build/pet-goal-execution-next-r1/native-functional-composition-b3i/PARTIAL-CHECKPOINT.json).

**Estado corrente — 07/11/2026:** B3H privado validou nativamente a inferência de filho já admitido após perda/recolha do processo pai, nos dois protocolos locais sintéticos, mantendo F2/destino/prazo/admissão e saldo originais. Corrigidos tempos concorrentes na conclusão de Stop já assinado e a indicação de projeção pendente após Cancel/Pause de pool recuperado. Cancel durante IO conserva consumo desconhecido, sem resultado aceite ou repetição. Reutilizados Authority/Store/Owned/Driver e reservas/Stop comuns;3 casos novos+9 recuperação+69 funding passaram nas mesmas1189 fontes e build nativa2 compila. Não é sobrevivência de worker ao desaparecimento do backend, retoma do planeamento do pai, aceitação Tauri/humana/áudio, compatibilidade/restauro integral ou entrega. Raiz1170/instalada32 preservadas; atualização local autorizada quando validada; commits/push suspensos. [Fontes, comandos, falhas e limites B3H](../.build/pet-goal-execution-next-r1/peer-iteration-h3d2-b3h-final/PARTIAL-CHECKPOINT.json).

**Checkpoint anterior — 07/11/2026:** B3G6 privado preserva publicações parciais/IO incerto após perda real do processo em perfis privados e nos dois protocolos HTTP sintéticos. Filho determinístico já admitido executa com a reserva original; Pending continua bloqueado. Corrigidos Cancel/Pause posteriores ao fecho do pool, incluindo replay, esquema peer e bloqueio de ResumeDraft bruto; fecho original e crédito vinculado preservados. Reutilizados Authority/Store/Owned/Driver, reservas e Stop de subárvore. 6 casos novos +9 recuperação +69 funding passaram;1 helper de subprocesso explicitamente ignorado no filtro pai. Não é prova de inferência do filho após perda do pai, UI/humano, leitor anterior/restauro, integração ou entrega. Raiz1170/instalada32 preservadas; Guardian não operacional e critérios integrais abertos. [Fontes, comandos, falhas e limites B3G6](../.build/pet-goal-execution-next-r1/peer-iteration-h3d2-b3g6/PARTIAL-CHECKPOINT.json). Registos seguintes são histórico.

**Estado corrente — 05/11/2026:** raiz1170 e instalação32 preservadas; G3 em desenvolvimento. B3F4 privado recupera publicação/ACK já canónicos após perda do estado de um Driver sobre o mesmo Owned vivo, sem reconstruir Approved, publicar plano ausente ou repetir inferência. Nova revisão abrange só efeitos pendentes; staging ambíguo recusa nova revisão. Prazos originais/monotónicos, orçamento e revogação conservados. 46 casos dirigidos/afetados passaram nas mesmas1188 fontes. Falta perda do próprio Owned/processo, UI/crash/leitores/restauro/composição e integração; política pública fechada, Guardian não operacional e49 critérios integrais abertos. [fontes, comandos, falhas e limites B3F4](../.build/pet-goal-execution-next-r1/peer-iteration-h3d2-b3f4/PARTIAL-CHECKPOINT.json). Registos seguintes são histórico.

**Decisão estrutural de 07/11/2026 — Sistema:** integrar as definições/widgets já usados na estação (DMS/Quickshell observado), sem construir definições próprias equivalentes nem substituir `controlCenterButton`. Mantêm-se controlo pelos pets, matriz pet×subplugin×operação×recurso, posse operacional temporária, acesso humano independente e reutilização dos brokers/executores. Navegar para a UI não concede mutação. [Contrato vigente](PET-SYSTEM-INTEGRATION.md). Esta decisão prevalece sobre planos visuais históricos; não altera a instalação nem reabre publicação.

**R9C2C1 integrado localmente1126 (04/11/2026).** Integração1126: Goal30/Selection2/Context4/GoalData3/Admission12/Grant21, modelo/destino e consentimento próprios. Core verifica os dois protocolos locais sintéticos e cancelamento HTTP com incerteza. Native487 comprova controles F2, Driver com vista fechada,1pedido sintético/hash real, resultado/reinício/ACK/settlement/leitor anterior/backup.489 restaura pelo produto o artefacto e contexto legíveis, sem reativar autoridade. Não instalado; não autonomia integral. Ver [fontes, comandos, provas e limites R9C2C1](../.build/pet-goal-execution-next-r1/FUNDED-RESPONSE-R9C2C1-FINAL-VERIFICATION-F1.json).

**R9C2B integrado localmente1123 (04/11/2026).** Implementado e integrado1123: novo controlo nativo de ramo, grant20/reserva3/vista2; worker ativo/lease expirado não libertam saldo; orphan Bound sem fila pode ser revogado e só então reconciliado como nunca reclamado. Native469 comprova UI/Stop/drenagem/hold/reinício. C480+479 comprovam backup/restauro real de quatro objetivos e recusa conservadora do leitor anterior. As falhas parciais463/477 são preservadas e não reclassificadas como verdes. Não instalado. Ver [fontes, comandos, falhas e limites R9C2B](../.build/pet-goal-execution-next-r1/DELEGATION-BRANCH-R9C2B-FINAL-VERIFICATION-F1.json).

# Estado de implementação dos pets

**R9C2A integrado localmente1120 (04/11/2026).** Implementado/integrado1120: origem2/Goal29/Admission11/Grant19/Bound2/reply5/inbox4, controlos utilizáveis e provas privadas455/457. O saldo do pai não duplica e não é devolvido por mero fecho/TTL. Rota nativa manualmente revista; não é a delegação autónoma completa. Instalação32 preservada. Ver [fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/FUNDED-DELEGATION-R9C2A-FINAL-VERIFICATION-F1.json).


**R9C1 integrado localmente1116 (04/11/2026).** Implementado/integrado1116: reserva no Grant18 e envio Envelope4, Admission10/Goal28, controlo mínimo utilizável e histórico sem autoridade. Validado por7casos atuais e native439/445; aceitação/execução financiadas do destinatário indisponíveis, instalação32 preservada. Ver [fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/DELEGATION-R9C1-FINAL-VERIFICATION-F1.json).


**R9B3B2 integrado localmente1110 (04/11/2026).** Implementado/integrado1110: tarefa de divulgação futura finita, consentimento próprio, espera antes do recurso, execução determinística pelo Driver comum e publicação/fecho únicos. Validado em fixtures e aplicação privada428/429; instalação32 anterior preservada. Não é delegação, subscrição contínua ou aceitação operacional de todos os providers. Ver [fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/WORK-DISCLOSURE-R9B3B2-FINAL-VERIFICATION-F1.json).


**R9B3B1 integrado localmente1107 (04/11/2026).** Implementado/integrado1107: relatório mínimo nativo de trabalho recebido, UI de criação/revisão e envio sem modelo. Validado com fixtures/backend e aplicação nativa privada422/423. Instalação32 permanece anterior; operacional/humano/hardware e divulgação autónoma pendentes. Ver [provas, fontes e limites](../.build/pet-goal-execution-next-r1/WORK-REPORT-R9B3B1-FINAL-VERIFICATION-F1.json).


**Checkpoint R9B3A (04/11/2026):** acompanhamento nativo do objetivo próprio ligado ao pedido integrado1102, com origem/revisão autenticadas, fila/reserva/tentativa/resposta/conclusão distintas e histórico com contagens desconhecidas. Store/reconciliação comuns, NativeUser apenas, sem novos efeitos ou inferência na consulta. Native415/403 validaram a interface privada e restauro com provider sintético; instalação32 preservada. [Contrato](PET-MESH-CONTRACT.md) e [evidência/limites](../.build/pet-goal-execution-next-r1/RECEIVED-WORK-R9B3A-FINAL-VERIFICATION-F1.json). Publicação durável de estados e delegação continuam pendentes;49requisitos integrais abertos.


**Checkpoint R9B2B (04/11/2026):** seleção e revisão própria do pedido recebido+notas para contexto autónomo local, corpo/hash/reserva comuns e leitura nativa separada de histórico implementados/integrados1097. Native391/381 validaram em privado o percurso com HTTP sintético, recuperação, leitor antigo e restauro; zero provider real/quota, instalação32 preservada. [Contrato](PET-MESH-CONTRACT.md) e [evidência/limites](../.build/pet-goal-execution-next-r1/RECEIVED-CONTEXT-R9B2B-FINAL-VERIFICATION-F1.json). Seguem estados/delegação;49requisitos integrais abertos.


**Checkpoint R9B2A (04/11/2026):** preparação do plano/contexto do objetivo recebido disponível pela interface, com escolha imutável e retoma de gravações parciais pelo runtime normal. Native371 comprovou as decisões separadas até uma resposta local, zero modelos; plano genérico também prepara dados sem inferência. Fontes1092 integradas; backend/formato1090 preservados. [Contrato](PET-MESH-CONTRACT.md) e [provas/limites](../.build/pet-goal-execution-next-r1/RECEIVED-PLAN-R9B2A-FINAL-VERIFICATION-F1.json). Segue contexto R43 da origem, estados/delegação; instalação32 e49requisitos integrais ainda não entregues.

**Checkpoint R9B1 (03/11/2026):** pedido recebido pode originar objetivo próprio revisto, com proveniência assinada e Goal24; criação não concede contexto de modelo, orçamento ou execução. A janela nativa privada comprovou decisões separadas, reinício, leitor anterior e restauro histórico. [Contrato](PET-MESH-CONTRACT.md) e [provas/limites](../.build/pet-goal-execution-next-r1/RECEIVED-TASK-R9B1-FINAL-VERIFICATION-F1.json). Fontes1090 integradas; instalada32 inalterada. Falta controlo de tarefa/plano do rascunho recebido, contexto R43 e restantes estados/delegação. Os49requisitos integrais continuam abertos.

**Checkpoint R9A (03/11/2026):** respostas locais correlacionadas integradas nas fontes1080. Cada pet responde com objetivo, sessão, orçamento e aprovação próprios; não recebe permissões do remetente. Janela nativa automatizada, reinício, leitor anterior e restauro passaram; o histórico restaurado não concede resposta/execução. [Contrato](PET-MESH-CONTRACT.md) e [provas/limites](../.build/pet-goal-execution-next-r1/CORRELATED-REPLY-R9A-FINAL-VERIFICATION-F1.json). Seguem aceitação autorizada/contexto próprio e delegação; os49critérios integrais continuam abertos. Instalada32 inalterada, sem publicação/host/quota.

**Checkpoint R8 C3B (03/11/2026):** revisão nativa de passos pendentes integrada localmente1074: preserva recibos, exige renovação explícita de prazo e separa aprovação dos dados da continuação/orçamento. Janela nativa automatizada, reinício, leitor anterior e restauro passaram; o restauro só conserva histórico. [Contrato](PET-DETERMINISTIC-RUNTIME.md) e [provas/limites](../.build/pet-goal-execution-next-r1/NATIVE-PLAN-REVISION-R8C3B-FINAL-VERIFICATION-F1.json). Seguem respostas/admissão própria do destinatário; inferência legada, delegação, Guardian/Sistema/voz e49critérios integrais continuam abertos. Instalada32 inalterada; sem commits/push/host/quota. Entradas anteriores são histórico.

**Checkpoint R8 C3A (03/11):** transação canónica de publicação partilhada adotada no percurso Spec3;43provas dirigidas/afetadas sem ignorados. Não altera formatos, não entrega ainda revisão humana de passos/TTL e não foi instalada. [Contrato/estado](PET-DETERMINISTIC-RUNTIME.md) e [evidência](../.build/pet-goal-execution-next-r1/SHARED-PLAN-PUBLICATION-R8C3A-VERIFICATION.json). Segue C3B;49critérios integrais mantêm-se abertos.

**R8 C2 F1 — fecho e continuação no mesmo backend integrados localmente1062 (03/11):** o DrainProof original sela o fecho no mesmo Grant13 antes de publicar o Goal19. ACK apenas publica o alvo exato já assinado. Reference4/Spec7/Goal20 permitem nova revisão sem modelo após fecho confirmado, preservando recibos e encargos; Closed sozinho, Uncertain, cancelamento e JSON não concedem continuação. Native343 comprovou dois fechos/três admissões no mesmo processo, UI, reinício/ACK e leitor C1 anterior; native344 comprovou restauro/ativação com o mesmo binário, apenas como dados históricos. [Contrato](PET-DETERMINISTIC-RUNTIME.md), [composição](../.build/pet-goal-execution-next-r1/integration1062-native-controlled-continuation-r8c2/FINAL-CHECKPOINT.json) e [provas/limites](../.build/pet-goal-execution-next-r1/NATIVE-CONTROLLED-CONTINUATION-R8C2-FINAL-VERIFICATION-F1.json). Segue C3: revisão explícita de planos/passos expirados sem reenviar os cobertos, por mecanismos comuns.43+6requisitos integrais preservados/abertos; instalação32 inalterada, sem commits/push/merge/host/quota. As composições anteriores abaixo são histórico.

**R8 C1 F1 — continuação após perda do executor integrada localmente1057 (03/11):** a revisão explícita sem inferência conserva recibos e encargos originais; os passos cobertos não são enviados novamente. Reference3/NativeCarry, Spec6/Goal18 e Report2/Grant12 reutilizam Recovery/Store/Authority/Owned, sem outro ledger, modelo, saldo renovado ou autoridade restaurada. Provas privadas: duas perdas reais de processo, três admissões do mesmo objetivo, revisão/recusa pela UI, leitor B anterior e restauro do produto. O restauro só conserva dados históricos. [Contrato](PET-DETERMINISTIC-RUNTIME.md), [composição](../.build/pet-goal-execution-next-r1/integration1057-native-continuation-r8c/FINAL-CHECKPOINT.json) e [provas/limites](../.build/pet-goal-execution-next-r1/NATIVE-CONTINUATION-R8C-FINAL-VERIFICATION-F1.json). Segue C2: retoma após fecho controlado com prova do drain e revisão explícita dos passos expirados, sem ModelAttempt fictício. Critérios livres, respostas/admissão do destinatário e delegação mantêm-se abertos;43+6requisitos integrais preservados. Instalação32 inalterada; sem commits/push/merge/host/quota real. As composições anteriores abaixo são histórico.

**R8 B F1 — conclusão nativa integrada localmente1056 (03/11):** a revisão original inclui os predicados nativos suportados antes de certificar a admissão. Spec5/Goal17 selam Binding2; Grant11 guarda o relatório imutável no mesmo ledger de autoridade. O mesmo `Store.complete_certified` verifica e publica a conclusão, com origem explícita em recibos locais e sem recipeId fictício, novo modelo, orçamento ou aprovação no fim. Critérios livres continuam Unknown; persistência local não verifica trabalho do destinatário. Native339 demonstrou quatro conclusões automáticas do remetente, parciais/critério livre/Stop/reinício/ACK e revisão funcional pela UI automatizada; native340 verificou leitor A4 anterior e backup/inspeção/restauro/ativação reais do produto, conservando os dados históricos e recusando a autoridade antiga. [Contrato](PET-DETERMINISTIC-RUNTIME.md), [composição](../.build/pet-goal-execution-next-r1/integration1056-native-completion-r8b/FINAL-CHECKPOINT.json) e [provas/limites](../.build/pet-goal-execution-next-r1/NATIVE-COMPLETION-R8B-FINAL-VERIFICATION-F1.json). Segue R8 C: recuperação/continuação explícita do modo nativo, sem renovar autoridade ou saldo.43+6requisitos integrais preservados/abertos; instalação32 inalterada, sem commits/push/merge/host/quota real. As composições abaixo são histórico.

**R8 A4 — bases integradas localmente1051 (03/11):** o mesmo runtime admite trabalho literal de `pet.message` sem modelo, readiness, autenticação ou reserva de inferência. Reutiliza Store/Review/Approved/Authority, fila, Owned, recursos, orçamento e recuperação. Spec4/Goal16, Proof3 `goalDeterministic`, Lease2 e Grant10 selam `runtimeOnly` com0chamadas de modelo; os formatos legados conservam as suas guardas. [Contrato](PET-DETERMINISTIC-RUNTIME.md), [composição](../.build/pet-goal-execution-next-r1/integration1051-deterministic-runtime-r8a4/FINAL-CHECKPOINT.json) e [provas/limites](../.build/pet-goal-execution-next-r1/DETERMINISTIC-RUNTIME-R8-A4-VERIFICATION.json). Native338 comprovou API/controlo funcional/reinício em Ollama, LM Studio, OpenAI e Spotify sem pedidos de metadata/inferência; native336 comprovou leitor R7 anterior e backup/inspeção/restauro/ativação com recibo histórico inativo. Não é aceitação humana, hardware, inferência real ou entrega instalada. Faltam verificação nativa dos critérios e continuação explícita deste modo; receber uma mensagem não conclui o objetivo. Respostas, admissão do destinatário e delegação continuam abertas.43+6requisitos preservados; instalação32 inalterada; sem commits/push/merge/host. As entradas de composições anteriores abaixo conservam o histórico; o próximo passo vigente é R8 B, verificação/recuperação determinísticas.

**R7 F1 — integração local1048:** mensagens locais com origem nativa, revisão exata, recibo/débito no Grant comum, inbox e recuperação de intenção não publicada. Inclui leitura explícita de histórico restaurado, verificada contra o índice/hash/assinatura original e sem reativar pedidos.29ficheiros integrados,14novos;22dirigidos,3374gerais/50ignorados,93web/TSC/Vite e build636inputs. Native325 cobre2direções/Frame/reinício;326 verifica o leitor R6 anterior;329 realiza backup/inspeção/restauro/ativação e consulta histórica no produto com metadata sintética e0inferência. [Composição](../.build/pet-goal-execution-next-r1/integration1048-mesh-r7-f1/CHECKPOINT.json) e [provas/limites](../.build/pet-goal-execution-next-r1/mesh-r7-history-c5/VERIFICATION.json). Instalação .32 inalterada. Recebido continua apenas dados: respostas, admissão do destinatário e delegação não estão fechadas. A admissão inicial de tarefas determinísticas sem evidência/reserva de modelo é a próxima dependência arquitetural;49requisitos integrais continuam abertos.

**R6 F1 — integração local1034:** o [binding por pet](PET-INFERENCE-BINDING.md) separa dono lógico de motor Ollama/LM Studio no mesmo runtime, incluindo objetivos, memória consentida, orçamento e recuperação. [Fontes/provas](../.build/pet-goal-execution-next-r1/integration1034-inference-binding-r6f1/CHECKPOINT.json). Instalada .32 inalterada;49 requisitos integrais continuam abertos.

## R5C1A4 — Esperas expiradas sem bloquear trabalho independente

R5C1A4 integrado localmente em4ficheiros, sem novo módulo ou outputs web. A fila liberta a espera expirada mesmo com catálogo ocupado e permite conversa independente; não liberta Hold usado por worker/Pending. A capacidade não fica consumida por64estados cujo prazo assinado já terminou. Prova final:13testes dirigidos e native318; build621. Geral3326pass/50ignorados foi executado em A2, antes do delta final de estado/capacidade, e não se apresenta como full A4. Web1353/TSC/Vite anteriores são byte-exatos. Instalada32/fontes1003 intacta; nenhum critério integral declarado completo.

[Composição, provas e limites](../.build/pet-goal-execution-next-r1/QUEUE-HOUSEKEEPING-CHECKPOINT-R5C1A4.json).

## 02/11 — Aprovações externas sem interferência no desenvolvimento (R5D1A3)

Integradas1027 fontes em10 caminhos/4novos,42 outputs web e29 sons preservados. Final Rust3321pass/50ignorados, incluindo44 casos pipe; TypeScript/Vite e1353web A2 reutilizados em A3 por frontend/config/bundle exatos. Build621 com107avisos anteriores e native313/3.83s: dois hooks externos sintéticos devolvidos imediatamente, escolha real na gestão, retirada de espera, modo/revisão/caller inválidos e aprovação tardia recusados, preferências intactas e restart com defeito de desenvolvimento.0inferência/quota/efeitos de leitor. A1 conserva2 falhas de preparação DOM corrigidas em A2; A2 já passava mas A3 acrescenta o epoch que cobre respostas retiradas de Pending. Instalada32/fontes1003 intacta; não é aceitação Hyprland/humana/instalada nem fecho do sistema.

[Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/EXTERNAL-APPROVAL-PARTIAL-CHECKPOINT-R5D1A3.json).

## 02/11 — Pedido durável de arranque e fila inicial (R5B1A2)

Fontes1023 integradas em27 caminhos, dois novos módulos;42 outputs web e29 sons preservados. Passaram7 testes do pedido,14 do broker,2 no filtro de admissão, full3313/50ignorados, web1346/0ignorados, TypeScript, Vite e build620/107avisos preexistentes. Native310/39.63s: admissão ocupada, fila1/2, pausa de um ramo, arranque com vistas ocultas, receita explícita, pós-condições, reinício só não reclamado e leitor1021 a recusar14 sem escrever. Native311/11.53s: produto exporta/restaura14 byte a byte, autoridade antiga histórica, prazo ainda válido, claim recusado e futuro13 sem admissions/contexto.0quota/modelos reais. Instalada32 permanece1003. Segue R5C; todos49 requisitos e Guardian operacional continuam abertos.

[Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/REQUESTED-DISPATCH-PARTIAL-CHECKPOINT-R5B1A2.json).

## 02/11 — Fila de recursos e cancelamento independente (R5A5)

Fontes1021 integradas em12 caminhos, incluindo um novo módulo de testes. Dirigidos9/0ignorados, full3303/50ignorados, web1342/0ignorados, TypeScript, Vite e build618 passaram. Native309 passou em25.23s com2pedidos de inferência sintéticos e2efeitos MPRIS privados: posições1/2, cancelamento do primeiro promove só o segundo, terceiro dono preservado, conclusão por pós-condições e mesmos contextos/orçamentos. Instalada32 continua1003. Arbitragem está integrada; dispatch durável, G2 integral e Guardian operacional continuam pendentes.

[Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/RUNTIME-ARBITRATION-PARTIAL-CHECKPOINT-R5A5.json).

## 01/11 — Objetivo com vistas fechadas e Stop durante IO (R4A1)

A composição integrada1020 passou o ensaio lifecycle304 em9.74s, sem mudança de código. Dois pedidos de modelo sintético: o primeiro gera receita revista e completa só após pós-condições nativas no leitor privado, com chat/Settings ocultos e cancelamento manual separado. O segundo é parado em0.131s: pausa, orçamento original, tentativa desconhecida ainda cobrada, dono fechado e recurso reutilizado por novo objetivo sem inferência.0modelos/quota/efeitos reais. A instalação32 continua1003. Fila/justiça/dispatch, demais motores e lifecycle/aceitação integrais permanecem por fechar.

[Composição, comando, prova e limites](../.build/pet-goal-execution-next-r1/RUNTIME-LIFECYCLE-PARTIAL-CHECKPOINT-R4A1.json).

## 01/11 — Retirada de autorização com origem nativa (R3A1)

Fontes1020 integradas em4 caminhos, um novo módulo de testes.16 dirigidos/0ignorados,3295 gerais/50ignorados, build617 e nativo302 passaram;107 avisos de build preexistentes. Web1341/TypeScript/Vite reutilizados por igualdade de frontend/config/bundle;42 outputs e29 sons preservados. Native302 usa grant realmente emitido, reinício privado, fecho/reabertura/reload, contenção real após decisão, alias original, retirada única e ACK após reinício sem escrita.0inferência/quota/efeitos. Instalada32/fontes1003 intacta. Guardian e49 requisitos integrais continuam abertos.

[Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/RUNTIME-RETIREMENT-REVIEW-PARTIAL-CHECKPOINT-R3A1.json).

## 01/11 — Grupos de entrega integral

[Dez grupos,49 requisitos e critérios de aceitação](PETS-DELIVERY-GROUPS.md). Trabalho atual em G2 (autonomia individual); a subfase de origem nativa das revisões tem R1/R2/R3 integrados:3/3 unidades,100% da implementação/prova automatizada/integração local delimitadas, por contagem de unidades de peso igual. Este número não mede G2 inteiro nem o sistema integral; a aceitação instalada permanece pendente. Segue lifecycle R4 sem alterar a ordem/dependências do objetivo completo.

## 01/11 — Consentimento nativo de admissão e continuação (R2A3)

Fontes1019 integradas em15 caminhos, um novo módulo de testes, mais42 outputs web;29 sons preservados e4 outputs compilados obsoletos retirados só após cópia de recuperação.36 dirigidos e3291 Rust/50 ignorados passaram, além de1341 web/TypeScript/Vite e build616 com107 avisos preexistentes. Tauri299 cobre admissão/planeamento/continuação, token recusado após fecho/reload e assinatura antes de publicação bloqueada;300 cobre unsafePath após assinatura e reconciliação da mesma escolha.0inferência/quota/efeitos de ferramenta. App32/fontes1003, HEAD/índice e10fontes externas intactos. Guardian ainda não operacional;49 critérios integrais abertos.

[Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/RUNTIME-ADMISSION-REVIEW-PARTIAL-CHECKPOINT-R2A3.json).

## 01/11 — Contexto nativo da recuperação de objetivos (R1A1)

Fontes1018 integradas em7 caminhos, um módulo novo.9 testes de recuperação e13 de revisão/configuração passaram; suite geral3287 aprovados/50 ignorados e build615inputs exatos. Nativo294: Tauri/GTK/WebKit reais, lease nativa, crash/recolha do pai privado, token antigo recusado após fecho/reabertura/reload, objetivo inalterado; falha de escrita após closure assinada, reabertura cleanup-only, mesma aprovação/orçamento e ACK sem duplicação. Descoberta HTTP sintética, zero inferência/quota/efeitos de leitor. App32/fontes1003 preservada. Settings294, Companions/PetControl296 validados nativamente nos cenários indicados; quatro labels em fixtures, PetPanel Linux retirado. Os49 requisitos integrais e Guardian continuam incompletos. Nativo296 acrescenta Companions (ocultar/reabrir) e PetControl (reload/reabertura explícita pela navegação nativa),9.17s: revisão nova nas três superfícies Linux existentes, token anterior/caller errado recusados e objetivo/ficheiros inalterados,0inferência/quota.295 foi conservado: o reload recolhe PetControl e a nova revisão permanece recusada até à abertura explícita; não se alargaram ACLs nem se alteraram fontes para fazer passar o teste.

[Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/RUNTIME-RECOVERY-REVIEW-PARTIAL-CHECKPOINT-R1A1-N296.json).

## 01/11 — Escrita Linux com driver próprio e confirmação (C6C4B2B1R3A6)

Fontes1017 integradas:11 caminhos,4 novos, sem dependência nova. O Linux já não inicializa/escreve arboard no processo principal; o driver residente recebe pedidos em duas fases e conserva o hold até recolher a família. Passaram29 dirigidos A4,23 helpers A5 e28 Copilot A6 e3281 gerais/50 ignorados; build614 A4 reutilizada por dependências compiladas exatas, com 107 avisos;1339 web/TSC/Vite anteriores reutilizados por inputs frontend/bundle exatos. Nativo291 atravessa Tauri/IPC reais, reutiliza o mesmo residente apenas em novos pedidos autorizados, mantém as leituras/reposição, preserva dono sem texto, verifica Stop/clear durante leitura e ausência de residente vivo após quit. Prova em Xvfb sintético; não hardware, Wayland/Windows, revisão/posse física ou áudio. App32/fontes1003 intacta.43+6 requisitos integrais continuam abertos; Guardian ainda não operacional.

[Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/CLIPBOARD-WRITER-PARTIAL-CHECKPOINT-C6C4B2B1R3A6.json).

## 01/11 — Preservação e recuperação manual do texto (C6C4B2B1R2A4)

Fontes1013 integradas:15 caminhos, um novo teste web. A leitura da reposição já está no helper limitado; clipboard sem texto preservável/indisponível/excessivo bloqueia colagem automática. Botão funcional Copiar último texto nas páginas existentes, com nova ocorrência e sem expor o texto no estado passivo. Clear/reload/substituição invalidam a origem capturada. Passaram3259 Rust/50 ignorados,1339 web/0 ignorados, TypeScript/Vite e build610/107 avisos anteriores. Nativo287 comprovou ambas as observações, reposição, controlo de cópia, dono sem texto preservado, vazio X11 observado, Stop/clear durante leitura e recolha de helpers próprios. Xvfb/Bus/perfil sintéticos, sem aceitação física ou ASR real. A instalada32/fontes1003 está intacta; sem commits, publicação ou instalação. Guardian continua desenhado/reservado, não operacional; os49 requisitos integrais continuam abertos.

[Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/CLIPBOARD-RESTORE-PARTIAL-CHECKPOINT-C6C4B2B1R2A4.json).

## 01/11 — Leitura anterior do clipboard em helper próprio (C6C4B2B1R1A2)

Fontes1012 integradas: seis caminhos, dois novos, sem nova dependência. Leitura anterior real usa o helper limitado e de uso único; o supervisor comum drena IO sem dormir após blocos que progrediram, mantendo guard/limites por iteração. Passaram14 testes de leitura,3246 gerais/50 ignorados e build610, com107 avisos anteriores preservados. Nativo285 passou em3.77s: Tauri/IPC reais, clipboard X11 sintético em Xvfb, leitura pelo pai autenticado, reposição do original e recusa70 por outro executável, sem conteúdo no log. Isto não valida clipboard do anfitrião, revisão física, Wayland, Windows, modelos ou áudio. App instalada continua32/fontes1003; sem commits/publicação/instalação.

[Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/CLIPBOARD-READ-PARTIAL-CHECKPOINT-C6C4B2B1R1A2.json).

## 01/11 — Ditado: guards internos e resultados de colagem (C6C4B2A5)

Fontes1010/C6C4B2A5 integradas: nove caminhos, dois novos de testes, sobre os mesmos executores. Passaram12 ensaios de protocolo em D-Bus privado e3232 testes gerais;50 ficaram ignorados na suite geral, incluindo esses12 executados separadamente. A4 passou61 testes Flow/12 ignorados; o full final cobre o código A5. A build608 passou. Web1333/TSC/Vite A11 reutilizados por identidade das fontes/configuração frontend. Nenhum clipboard, portal do anfitrião, áudio, modelo ou quota reais foram usados. App instalada continua32/fontes1003; commits/publicação/instalação suspensos. O incremento é parcial: posse/alvo nativos, IO de biblioteca e fecho incerto ainda precisam de trabalho.

[Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/VOICE-PASTE-PARTIAL-CHECKPOINT-C6C4B2A5.json).

## 01/11 — Whisper: validade durante IO e família própria (C6C4B1A3)

Fontes1008/C6C4B1A3 integradas: sete caminhos, um novo. Reutilizados RecognitionScope, cache Whisper, OwnedChild e primitivas de IO. Passaram21 testes dirigidos,3209 gerais/38 ignorados e build privada com606 inputs exatos. Os1333 testes web/TSC/Vite A11 são reutilizados por identidade das fontes frontend/configuração de build; o helper Python mudou como input nativo. A build tem106 avisos: baseline105 e o wrapper de compatibilidade transcribe_with_vocabulary agora sem caller de produção. A app instalada continua32/fontes1003. Sem modelos, áudio, quota, alterações do anfitrião, commits ou publicação neste incremento.

[Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/VOICE-WHISPER-IO-CHECKPOINT-C6C4B1A3.json).

## 01/11 — Contexto ASR e ditado antes do áudio (C6C4A4)

Fontes 1007/C6C4A4 integradas localmente: nove caminhos, quatro novos, reutilizando o ASR existente em Voice e ditado. Passaram seis testes dirigidos do scope e oito do ditado; a execução geral passou 3196 testes, com 38 ignorados. A build privada passou com 605 entradas exatas. Os 1333 testes web/TSC/Vite A11 são reutilizados por identidade dos inputs não Rust. A app instalada continua mochi.32, baseada nas fontes1003/C3. Não houve commits, pushes, instalação, modelos, quota ou áudio reais neste incremento.

[Fontes, comandos, provas e limites](../.build/pet-goal-execution-next-r1/VOICE-CAPTURE-CHECKPOINT-C6C4A4.json).

## 01/11 — Continuação após entrega32: seleção comum de captura

A build32 permanece instalada e o remoto confirma0a1ba81. O ciclo autorizado terminou; commits, publicações e novas instalações voltaram a ficar suspensos. No SDK privado, C6C4A1 passou os seis testes do seletor comum: nomes de microfone com controlos são recusados antes da enumeração/default, também nos percursos de ditado e reuniões. Esta correção ainda não foi integrada nem instalada. Não prova identidade física do dispositivo ou áudio real. Seguem captura do contexto/ASR antes do áudio em Flow/Notes, identidade do recurso e ativação governada após restauro.

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


## R9C2C2A — continuação finita integrada1127 (2026-11-04T07:01:18.507246+00:00)

O mesmo objetivo delegado pode receber outra resposta local finita após fecho original confirmado, nova revisão humana e consentimento F2 próprio. Usa Reference6/Anchor e Authority/fila/Driver existentes; Funding2 autentica o prefixo e o binding muda por CAS. Consumo anterior + novos limites cabem na reserva original e no mesmo prazo. Não existe saldo adicional, continuação automática, reabertura de Settled ou partilha implícita de derivados. Resultados desconhecidos bloqueiam nova ronda.

Formatos condicionais Admission13/Goal31/Funding2/Reference6/Grant22; Reservation4/Grant23 no pai. Reconcile agrega prefixo e atual, preservando consumo/certeza. Metadados podem ser consultados explicitamente para um objetivo financiado pausado, sem criar execução ou contornar F2. A referência da cache que mantinha o escritor da conversa é retirada só após ACK do fecho original, recolha dos workers e libertação do recurso; conserva uma projeção limitada do estado sem executor. Permite rondas sucessivas sem reiniciar. Stop/inatividade/expiração não são prova suficiente de libertação. Capacidades continuam por caller; a extensão não promove taskRuntime/providerModel.

Provas: `.build/pet-goal-execution-next-r1/FUNDED-CONTINUATION-R9C2C2A-FINAL-VERIFICATION-F1.json`; cinco casos finais de continuação, fecho7 e regressões afetadas9+4+6 por identidade dos ficheiros cobertos, web62/tipos/Vite e build698. Native521 e restauro525 validam duas rondas/duas escolhas F2, hashes do corpo real, orçamento cumulativo, terceira ronda recusada, fecho original, reinício, leitor anterior e backup/restauro do produto. São provas privadas com provider sintético, sem aceitação humana/instalada ou quota real. Falhas e filtros vazios estão preservados e excluídos da prova.

Os49 requisitos integrais continuam abertos. Seguinte: R9C2C2B, partilha autorizada de resultados derivados e verificação/publicação; ferramentas, delegação recursiva/paralela, restantes integrações, Sistema/voz/Guardian e entrega permanecem dependências posteriores. Sem commits/push/merge ou instalação.


## R9C2C2B1 — proposta de resultado derivado integrada1129 (2026-11-04T07:41:18.146714+00:00)

O recibo próprio ModelResult1 pode fornecer um excerto exato UTF-8 a uma revisão de plano DATA, através do mesmo NativePlanReview/publicação/ACK/retirada existente. Referências vinculam dono, objetivo, decisão, pedido, tentativa, hash do recibo, intervalo e cobertura; destino e correlação são a origem da delegação. Corpo/texto/recibo/offsets/destino trocados e origem cancelada são recusados. Sob writer, a validação autenticada do recibo não readquire flock. Não copia o histórico nem altera a identidade ou critérios.

Formatos condicionais Reference2/Seal2/Goal32/Grant24; ModelResult1 preservado e View acrescenta receiptHash. A validação dos antigos admissions de resposta consulta o plano original autenticado, mantendo os fences de novos efeitos sobre o plano corrente. `pet.resultReply` v1 tem apenas validação DATA, sem executor, Call ou ferramenta de modelo. A capacidade é NativeUser-only para pets LLM; não promove observadores/serviços.

A interface permite selecionar o intervalo, rever texto/proveniência/destino e guardar o plano. A confirmação não cria admissão, orçamento, envio ou consentimento de divulgação. Depois da proposta guardada, a execução fica bloqueada até B2; F2 e WorkDisclosure continuam com âmbitos separados. Não certifica critérios pelo conteúdo do modelo.

Provas: `.build/pet-goal-execution-next-r1/DERIVED-RESULT-PLAN-R9C2C2B1-FINAL-VERIFICATION-F1.json`; core4, revisão de planos16, resultados14 (+helper filho executado pelo teste pai), armazenamento4, capacidades46, web71/tipos/Vite, build700. Native527/531 validam UI real sobre provider sintético, duas rondas sem reiniciar, proposta32/24 e ACK/inventário DATA-only, orçamento conservado, ausência de envio, reinício, leitor anterior e backup/restauro do produto sem renovar autoridade. Falhas/filtro vazio estão preservados e não sustentam aceitação.

Próximo: R9C2C2B2, concessão de divulgação própria e sucessor determinístico com orçamento cumulativo, entrega e verificação. Os49 requisitos integrais permanecem abertos. Sem commits/push/merge, instalação ou aceitação humana/física/online.


### R9C2C2B2D1 — revisão exata da origem e dependências integrada1133 (2026-11-04T08:11:33.669573+00:00)

A superfície nativa de revisão de resultado apresenta agora motor/adaptador/modelo e destino original autenticados, input hash, tempos, excerto UTF-8 e cobertura, destinatário/configuração e referências exatas das fontes. Capturas próprias da memória, delegação e destinatário são invalidadas de forma permanente ao observarem alterações; há revalidação no writer de publicação. Reutiliza NativePlanReview/Authority/Target/Capture; sem formato persistente ou ledger novos. A confirmação continua DATA-only: divulgação e execução dependem de D2, ainda aberto. Evidência detalhada em `docs/PETS-VALIDATION.md` e `.build/pet-goal-execution-next-r1/RESULT-DISCLOSURE-R9C2C2B2D1-FINAL-VERIFICATION-F1.json`. Os43+6 critérios integrais continuam abertos. Sem commits, publicação ou instalação.


### R9C2C2B2D2 — entrega autorizada integrada1138 (2026-11-04T09:52:11.346927+00:00)

Raiz1138/SDK1138: envio local de resultado derivado funcional em Tauri privado com modelo sintético, retoma da sessão original após Busy e preservação da resposta completa. Reinício, reconciliação, backup/restauro e recusa do leitor anterior verificados no perfil privado. Não instalado: mochi32 preservada; 49 requisitos integrais abertos. Evidência e limites: `.build/pet-goal-execution-next-r1/RESULT-DELIVERY-R9C2C2B2D2-FINAL-VERIFICATION-F1.json`.


### R9C2C3 — resultado recebido e resposta própria integrada1142 (2026-11-04T11:11:42.447422+00:00)

Raiz/SDK1142,710 inputs nativos: consumo explícito de resultado e uma resposta própria funcionais em Tauri privado com provider sintético. Reinício, leitor anterior, backup/restauro verificados. Mochi32 preservada; sem instalação, publicação ou aceitação humana. Os49 requisitos integrais permanecem abertos. Prova/limites: `.build/pet-goal-execution-next-r1/RECEIVED-RESULT-R9C2C3-FINAL-VERIFICATION-F1.json`.


### R9C2D — continuação de ferramentas financiadas integrada1143 (2026-11-04T11:31:16.354805+00:00)

Raiz/SDK1143/native711. Continuação de ferramentas determinísticas entre pets validada em fixtures para sete identidades e no Tauri privado: primeira entrega, Stop/drenagem, novo consentimento dentro da reserva, segunda entrega, reinício e restauro sem replay/renovação. Mochi32 permanece instalada;49 requisitos integrais abertos. Evidência/limites: `.build/pet-goal-execution-next-r1/FUNDED-TOOLS-R9C2D-FINAL-VERIFICATION-F1.json`.


### R9C2E — núcleo recursivo privado, integração pendente (2026-11-04T12:16:01.742507+00:00)

A raiz aceite permanece1143. O SDK privado1146/native714 implementa o percurso de dois níveis, autoridade própria, linhagem canónica, mesma reserva, correlação da raiz e cancelamento por ramo/subárvore. A3: três cenários de backend passam em fixtures (incluem sete identidades e worker original/ramo irmão); compatibilidade afetada44+6 passa,0ignorados, sem modelo/quota. Isto não é prova da interface Tauri, provider real, instalação ou conclusão de TEAM. Antes de integrar faltam Stop/Cancel/Pause genéricos do pool, controlos/bindings/codecs/capacidades, provas adversariais adicionais e percurso nativo com preservação/restauro/leitor anterior. Instalação mochi32 intacta; sem commits/publicação. Estado recuperável e fontes exatas: `.build/pet-goal-execution-next-r1/RECURSIVE-DELEGATION-R9C2E-WIP.json`. Os49 requisitos integrais continuam abertos.


### R9C2E — delegação recursiva determinística integrada1147 (2026-11-04T12:46:54.560631+00:00)

Raiz/SDK1147/native714. Native605 percorreu três pets e os controlos reais de adoção/plano/admissão/claim/revisão/envio, resposta8, reconciliação cumulativa, reinício, leitor anterior e backup. Native609 restaurou os três objetivos/contextos/recibos como histórico sem autoridade/saldo atuais. Stop ancestral e worker original validados em fixtures. Mochi32 preservada,49 requisitos integrais abertos. Evidência e limites: `.build/pet-goal-execution-next-r1/RECURSIVE-DELEGATION-R9C2E-FINAL-VERIFICATION-F1.json`.


### R9C2F — resposta própria a delegação recursiva integrada1148 (2026-11-04T13:17:37.986677+00:00)

Raiz/SDK1148/native715. Native613 validou controlos reais R43, uma resposta sintética com janela fechada, corpo/hash exatos, mesma reserva, reinício e leitor E anterior. Native615 restaurou os três objetivos/contextos e o resultado como dados históricos, sem renovar consentimento/execução/saldo. Sem instalação;49 requisitos integrais abertos. Evidência e limites: `.build/pet-goal-execution-next-r1/RECURSIVE-INFERENCE-R9C2F-FINAL-VERIFICATION-F1.json`.


### R9C2G1 — resultado recursivo revisto e entregue integrado1149 (2026-11-04T13:48:38.598891+00:00)

Raiz/SDK1149/native716. Testes dirigidos do backend/interface, Tauri617 e restauro619 passaram. Uma resposta sintética é revista e enviada exatamente uma vez com consentimento próprio, sem outra inferência; mesmo orçamento, reinício, ACK, leitor F e backup/restauro. Instalação32 preservada;49 requisitos integrais abertos. Segue G2 de adoção/uso próprio do resultado9. Evidência, comandos, fontes e limites: `.build/pet-goal-execution-next-r1/RECURSIVE-RESULT-R9C2G1-FINAL-VERIFICATION-F1.json`.


### R9C2G2 — utilização própria do resultado recursivo integrada1150 (2026-11-04T14:25:40.450511+00:00)

Raiz/SDK1150/native717. Web/Rust dirigidos, Percurso Tauri comprovado por prefixo633 + seguimento645 + auditoria canónica; restauro647 passou. Destinatário usa resultado recursivo depois da liquidação, com nova seleção/revisão/consentimento R43, uma resposta própria/zero ferramentas e saldo independente. Revogação ancestral durante HTTP conserva incerteza e impede sucesso/replay. A instalação32 e49 critérios integrais permanecem separados e abertos. Evidência, comandos, fontes e limites: `.build/pet-goal-execution-next-r1/RECURSIVE-RESULT-USE-R9C2G2-FINAL-VERIFICATION-F1.json`.


### R9C2H1 — checkpoint privado, integração pendente (2026-11-04T15:21:59.552857+00:00)

Raiz1150 e instalação32 preservadas. H1 acrescenta a escolha assinada de arranque finito e avanço limitado na fila existente. Corrigidos o descarte de start na bridge, o bloqueio de consentimento por recurso ocupado e a observação de Busy durante IO, mantendo Stop, dados, destino e orçamento originais. Provas web/Rust dirigidas passaram; a prova Tauri657 é parcial e falhou antes da revisão do terceiro trabalho. Reinício, leitor anterior e restauro deste novo modo ainda não foram aceites; H1 não foi integrado. Fontes, comandos, resultados, falhas e próxima unidade: .build/pet-goal-execution-next-r1/requested-execution-r9c2h1-a6/PARTIAL-CHECKPOINT.json. Os43+6 critérios integrais mantêm-se abertos.


### R9C2H1 — arranque finito pedido integrado1151 (2026-11-04T15:51:28.798761+00:00)

Escolha inicial assinada executionStart/Goal46, fila comum com até8 avanços independentes, dono/destino/modelo/prazo originais e Stop sem libertação fictícia do worker. Corrigidos Bridge, confirmação sob Busy, observação limitada do IO e resolução finita após reinício. Entrega direta/recursiva mantém consentimento próprio e zero inferência. Native663 privado passou: ambos protocolos sintéticos, espera/cancelamento, vista fechada, hash real, reinício após recuperação governada do titular antigo, leitor anterior e backup/restauro histórico. Nenhum claim antigo foi saltado ou repetido; nenhuma aceitação humana/instalada/hardware é afirmada. Fontes/provas/limites: [verificação](../.build/pet-goal-execution-next-r1/REQUESTED-EXECUTION-R9C2H1-FINAL-VERIFICATION-F1.json). Instalação32 preservada;43+6 integrais abertos.


### R9C2H2 — planos nativos revistos integrados1156 (2026-11-04T16:34:11.217358+00:00)

O Driver comum executa planos peer exatos com escolha inicial assinada e consentimento DATA boundedNativePeerPlan separado. Goal47 vincula argumentos, destinatários/configuração, fontes/revisões e prazo; nenhum destinatário recebe execução ou uso de modelos por delegação. Mesmos orçamento, recursos, recibos, retoma e fecho, com pedido estável distinto por passo. Fixtures exercem identidades, financiamento, recursão, ACK e carry; Native665 privado comprova revisão funcional, progresso independente, espera/reinício após recuperação governada, leitor anterior e backup/restauro histórico, sem chamadas a modelos. [Fontes, provas e limites](../.build/pet-goal-execution-next-r1/NATIVE-PEER-PLAN-R9C2H2-FINAL-VERIFICATION-F1.json). H2 não fecha planeamento/cooperação/autonomia integral nem aceitação instalada;43+6 permanecem abertos e mochi32 preservada.


### R9C2H3A — revisão de sequência peer integrada1159 (2026-11-04T17:22:30.520238+00:00)

Arranque manual explícito de planos nativos pode usar uma decisão exata para todos os passos pendentes no broker comum. Origem transitória goalPeerPlan, lease/dono e envelopes originais vinculados; sem novo formato persistente, inferência, execução do destinatário ou promoção automática de admissões antigas. Source/Decision, Driver, recibos e fecho são reutilizados. A confirmação final dos recibos trata Busy conhecido com a prova imutável original, sem reenvio. Native673 privado passou alvo/replay, recusa/Stop e progresso independente; leitor H2 preserva compatibilidade. [Provas, fontes e limites](../.build/pet-goal-execution-next-r1/PEER-SEQUENCE-R9C2H3A-FINAL-VERIFICATION-F1.json). Instalação32 e49 requisitos integrais continuam separados.


### R9C2H3B1 — planeamento de correspondência integrado1162 (2026-11-04T18:04:49.035404+00:00)

O catálogo explícito peerCorrespondenceV1 liga uma proposta local de IA à publicação assinada e à revisão nativa exata dos dados/destinatários, usando Driver, orçamento, recibos e cedência/retoma de recursos comuns. Goal48/Grant36 e Report3 distinguem recibo local de conclusão do destinatário: critérios livres continuam por verificar. O percurso funcional foi validado em Tauri privado com Ollama/LM Studio sintéticos; não constitui aceitação humana/instalada. Até existir proveniência/revogação nos envelopes e R43 no destinatário, memória/contexto recebido/histórico selecionados são recusados antes da inferência. Delegação gerada, rondas adicionais e backup/restauro do novo formato continuam pendentes. [Fontes, provas e limites](../.build/pet-goal-execution-next-r1/PLANNED-PEER-R9C2H3B1-FINAL-VERIFICATION-F1.json). Instalada32 preservada;43+6 critérios integrais abertos.


### H3B1 — prova adicional de preservação48 (2026-11-04T18:20:29.393937+00:00)

O arquivo do produto criado em685 foi restaurado com revisões separadas de dados, autoridade, configuração e ativação histórica. A prova nativa689 confirma os três objetivos48, grants36 e relatório3 com bytes, admissões e orçamentos originais; classificação histórica e recusa de claim/status/Driver, sem modelos repetidos. As tentativas anteriores de fixture permanecem falhadas; não foram reduzidas as proteções de caminho, mount ou autoridade. [Composição da prova](../.build/pet-goal-execution-next-r1/planned-peer-restore-r9c2h3b1-s1/CHECKPOINT.json). Não cobre crash em execução ou aceitação instalada.


### H3B2 — linhagem privada em implementação (2026-11-04T18:33:15.387465+00:00)

Implementação privada1165/A2 conserva todas as referências/revisões selecionadas e a publicação original na proveniência da mensagem; o destinatário reutiliza a captura de dependências e a preparação R43 existentes. Provas dirigidas de dados, falsificação, privacidade e compatibilidade passaram. O catálogo com memória continua bloqueado: faltam admissão22 do destinatário, controlos funcionais e aceitação de ponta a ponta antes de qualquer integração. Raiz1162 e instalada32 preservadas. [Checkpoint recuperável](../.build/pet-goal-execution-next-r1/peer-data-r9c2h3b2-a2/PARTIAL-CHECKPOINT.json).


### R9C2H3B2 — memória em correspondência integrada1166 (2026-11-04T19:05:46.675334+00:00)

Catálogo explícito com memória própria selecionada, proveniência/revogações no recibo comum e decisão nativa distinta sobre o texto/destino. O destinatário prepara objetivo/contexto e consente uma resposta local finita com orçamento próprio; não recebe memória bruta, consentimento ou saldo do remetente. Reutiliza R43, Driver, Store, Authority e Dependencies. Native697 e restauro699 passaram com providers sintéticos; não é entrega instalada/humana. [Composição, testes e limites](../.build/pet-goal-execution-next-r1/PEER-MEMORY-R9C2H3B2-FINAL-VERIFICATION-F1.json). Contexto transitivo, delegação gerada e restantes critérios integrais continuam abertos.


### H3C — delegação proposta integrada1167 (2026-11-04T19:29:13.085603+00:00)

O catálogo peerDelegationV1 junta uma ronda própria de IA às reservas dos filhos no mesmo orçamento/Grant. Verifica o saldo de todo o plano antes do primeiro envio e volta a validá-lo no escritor comum. O destinatário conserva admissão e R43 próprios. Native703 comprovou cancelamento/reconciliação de um ramo enquanto o irmão aguardava inferência, sem alterar a autorização desse irmão; restauro705 conservou apenas autoridade histórica. [Fontes, testes e limites](../.build/pet-goal-execution-next-r1/HYBRID-PLANNED-DELEGATION-H3C-FINAL-VERIFICATION-F1.json). Contexto selecionado/transitivo, colaboração iterativa e aceitação integral continuam abertos. Instalação32 preservada.


### H3D1 — evidência correlacionada integrada1170 (2026-11-04T19:53:31.264169+00:00)

Consulta nativa funcional dos filhos exatos de cada reserva, com revisões/hashes e recibos de modelo verificados pela reconciliação comum. Não copia respostas/memória nem concede execução ou uso por modelos; recibo disponível não verifica critérios. Native707 passou sem inferência ou escritas de runtime. [Fontes, testes e limites](../.build/pet-goal-execution-next-r1/PEER-FEEDBACK-H3D1-FINAL-VERIFICATION-F1.json). É base para H3D2; iteração autónoma continua por implementar. Instalação32 e49 critérios integrais preservados.
