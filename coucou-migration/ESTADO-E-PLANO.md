# Estado observado e plano de desenvolvimento

Fotografia de **25/10/2026**. O destino é a [arquitetura completa](ARQUITETURA-COMPLETA.md); este ficheiro descreve implementação e pendências. Os números de testes são evidência do âmbito indicado, sem equivalência a percentagem de produto concluído.

## 1. Três composições distintas

| Camada | Estado nesta fotografia |
| --- | --- |
| Aplicação instalada | `0.5.1-mochi.30`. Não recebeu automaticamente os incrementos seguintes. Não foi substituída para criar este dossier. |
| Árvore de trabalho integrada | Raiz `coucou-codex`, branch `feat/mochi-cli-accounts`, 834 fontes no último checkpoint de código aceite. Alterações locais sem novo commit. |
| Incremento privado corrente | SDK841/R43F1A: memória selecionada na preparação de objetivos, ainda com admissão autónoma deliberadamente bloqueada até ao consentimento/envio F2. |

HEAD e remoto observado continuam em `bbe12e21010d28d36619003187d82ba175b6a0ec`. Este commit não contém todas as alterações locais descritas. Commits, pushes e merges de código estão suspensos por instrução do utilizador. A exceção posterior autoriza apenas o commit/push da pasta `coucou-migration` numa branch de documentação, sem merge. O dossier inclui alterações documentais posteriores ao checkpoint834 sem promover o SDK841 a código integrado.

## 2. Bases implementadas e integradas localmente

| Área | O que existe | Limite de conclusão |
| --- | --- | --- |
| Fábrica e capacidades | Definições comuns, resolvedores, configuração independente e núcleo de descritores com adoção inicial. | Cobertura de todos os callers/operações e aceitação completa por pet continuam pendentes. |
| Conversa e CLI | Rotas Codex, Gravity, Copilot, locais/API e serviços conforme configuração; sessões e decisões nas operações suportadas. | Paridade real de terminais externos, contas/protocolos, ferramentas e áudio não está universalmente aceite. |
| Continuidade CLI | Associação exata; recusa de pasta/intenção incompatível antes do prompt; Nova conversa conserva arquivo; política ACP preservada. | Não equivale a adotar qualquer terminal descoberto ou admitir autonomia interna ilimitada. |
| Ollama / LM Studio | Modelo/instância e ligação explícita, contexto/consentimento, runtime local limitado, receitas, Stop e recuperação comuns. | Motor/cofre operacional e restantes capacidades têm aceitação própria; voz/shell não são herdados. |
| Runtime durável | Objetivos, contexto isolado, admissão, autoridade, reservas, leases, tentativas, resultados, planeamento/receitas limitados e recibos. | Não é execução geral de qualquer tarefa por todos os providers. |
| Recuperação | Reconciliação, continuação explícita com histórico/consumo original, resultados sem repetir inferência e tratamento de efeitos desconhecidos. | Executores adicionais e todos os percursos de falha/entrega continuam abertos. |
| Ferramentas | Broker/decisões comuns e primeiro executor Spotify com play/pause/volume, orçamento e pós-condições. | Não cobre o catálogo inteiro de Sistema nem todos os efeitos de CLI/desktop. |
| Memória humana | Notas/preferências por pet/conversa, proveniência, validade, CRUD/pesquisa paginada, CAS e arquivo de CID vazio. | Não injeta automaticamente memória nos modelos da raiz834; projetos/derivados/partilha continuam pendentes. |
| Observabilidade | Repositórios de atividade, descoberta/associação, consumo e cobertura em vários percursos. | Estado global de todos os terminais e medição completa por conta/provider permanecem por aceitar. |

O checkpoint834 passou 2582 testes Rust com 21 ignorados, check/build e 751 ensaios privados no anfitrião. Os 1186 testes JavaScript, TypeScript e Vite estão vinculados ao bundle web de 42 ficheiros. Treze percursos Tauri privados passaram na composição aceite. Estes ensaios preservaram o perfil e a app instalada.

## 3. Trabalho validado apenas em privado

O incremento R43 trata seleção explícita de memória, destino/consentimento e revogação. As camadas são cumulativas e a aceitação da unidade continua aberta.

| Incremento | Prova e comportamento | O que não prova |
| --- | --- | --- |
| R43D/838 | Captura/revisão comum de um turno, uso temporário de dados, limites e revogação durante IO, decisão e comando. Ollama/LM Studio com HTTP sintético e IPC Tauri nativo. | Os três envios API reais do protocolo ainda precisam de ensaio HTTP sintético próprio. |
| R43E1/840 | 2606 Rust/21 ignorados, check/build; 17 prompts de protocolo sintético em Codex, Copilot, Gravity e um serviço. Proveniência antes do prompt, ID exato, reinício, revogação/recolha e falha de adoção. Oito pedidos locais anteriores passaram como regressão. | Contas reais, quota, terminais externos e áudio não foram certificados. |
| R43F1A/841 | 2612 Rust/21 ignorados, 16 testes dirigidos de contexto, check/build e percurso Tauri130. Referências duráveis por pet/CID; contexto2/objetivo11 condicionais; leitor840 recusa sem alterar bytes; reinício e exclusão verificados. | Ainda não envia memória num objetivo nem aceita o consentimento autónomo correspondente. A barreira temporária é intencional. |

As chamadas de modelos/CLI de prova são sintéticas e não consumiram quota real. Os protocolos usam o backend e IPC nativos dentro do ambiente privado; não são confirmação da configuração pessoal da estação. Falhas anteriores, correções de fixtures e limites de cobertura continuam registados nas [provas canónicas fotografadas](fontes/docs/PETS-VALIDATION.md).

## 4. O que falta, concretamente

1. Concluir R43: revisão de dados/destino e consentimento próprio de objetivos, corpo real antes do hash/reserva, revogação/retoma, três percursos HTTP API sintéticos, superfície humana funcional e capacidades corretas por caller; integrar a unidade apenas depois.
2. Completar paridade dos pets: configuração/adaptadores pendentes, decisões e continuidade nativas, ferramentas/MCP, consumo, limites e recuperação conforme cada provider; promover observadores apenas com execução efetivamente implementada.
3. Fechar anexos múltiplos: recuperação/limpeza terminal, seleção por mensagem, migração de dados e adoção do protótipo com backup/rollback.
4. Evoluir memória: projeto, episódios/derivados/resumos, dependências/invalidação, pesquisa e partilha explícita entre pets e projetos.
5. Completar fontes: gestão/consentimento ICS, coleções multicalendário, âmbitos de objetivo/regra, leitura RSS/IMAP limitada.
6. Construir rotinas e proatividade integrais: regras, agendamento, ocorrências perdidas, preparação diária, acompanhamento e notificações com silêncio/deduplicação.
7. Implementar comunicação durável e cooperação: mensagens diretas, delegação paralela/assíncrona, causalidade, orçamento por ramo, conflitos/ciclos e coordenação resiliente opcional por Ollama.
8. Completar Sistema: matriz por pet/subplugin/operação/recurso, executores das unidades, prompts, partilha/revogação e paridade dos controlos legados.
9. Completar voz: cobertura de todas as operações do backend, decisões/Stop comuns e aceitação com microfone/Bluetooth/TTS PT/EN/misto reais.
10. Concluir dados, observabilidade e entrega: retenção, backup/restauro, perfis/leitores antigos, executável/helpers/autostart, reinício/hotplug e aceitação instalada. Decidir PostgreSQL apenas se houver benefício medido.

Os [43 IDs](REQUISITOS-E-ACEITACAO.md) conservam os critérios individuais. Nenhum deles é apagado por este agrupamento de trabalho.

## 5. Próxima execução e sequência

Após a produção deste dossier, retomar **R43F2**: rever os dados e o destino no fluxo nativo de admissão, exigir consentimento específico do objetivo e incluir a mensagem no corpo antes do hash canónico/reserva. Reutilizar as fronteiras de contexto, autoridade, validade e recursos existentes; não criar um segundo controlador.

Seguem a prova HTTP sintética das APIs, o controlo humano funcional e descritores. Depois da integração de R43, a unidade seguinte é escolhida pelas dependências do roadmap e pelos bloqueadores de preservação/execução individual. TEAM vem após a base individual necessária; Sistema e expansão visual não deslocam essas prioridades.

O backend mantém desenvolvimento incremental com snapshots/provas exatos. Não se inicia trabalho nas mesmas features de outra sessão: a instrução atual mantém um único agente de desenvolvimento. Isto não impede desenvolver colaboração entre pets do produto.

## 6. Estimativa e limites

A estimativa anteriormente comunicada é cerca de **40/100** e **150–300 horas de trabalho focado restantes**, com incerteza elevada. É uma estimativa de planeamento, não uma medição de conclusão, prazo de calendário ou compromisso de entrega. A quantidade de testes não produz essa percentagem.

Uma primeira versão útil pode preceder todo o âmbito. Reduzir âmbito exige decisão explícita; não se declara o sistema completo por atingir essa versão. A revisão deste dossier pode alterar decisões e esforço. A fotografia permanece identificada por data mesmo quando o desenvolvimento avança.
