# Requisitos e critérios de aceitação

Fotografia de **25/10/2026**. Os 43 IDs e as duas colunas de trabalho/aceitação abaixo são conservados do roadmap canónico desta revisão. Todos permanecem **abertos nos critérios integrais**. Um incremento pode satisfazer parte de um ID sem o fechar. A definição de produto completo não se reduz aos testes já positivos.

O estado por composição está em [ESTADO-E-PLANO.md](ESTADO-E-PLANO.md). A [cópia integral do roadmap](fontes/docs/PETS-ROADMAP.md) preserva os relatos de incrementos e a matriz de providers. Existe também [REQUISITOS.json](REQUISITOS.json) para consulta estruturada.


## ARC — Arquitetura e definição do produto

Contrato de revisão: [ARQUITETURA-COMPLETA.md](ARQUITETURA-COMPLETA.md). Prova transversal: Coerência documental, preservação de requisitos e alcance da aceitação.

| ID | Trabalho | Dependência e aceitação integral |
| --- | --- | --- |
| ARC-01 | Separar arquitetura, estado e histórico; atualizar prioridade para arquitetura e pets | Documento normativo, plano com IDs, fontes identificadas e arquivo anterior preservado; ligações coerentes |
| ARC-02 | Rever contratos quando código/decisão revelar incompatibilidade | Registar decisão, motivo, migração, impacto e requisitos afetados antes de implementar um caminho incompatível |
| ARC-03 | Manter definição de sistema completo e cobertura | Cada cenário do dia a dia tem capacidade, implementação, prova e pendência por provider; exceções não desaparecem dos critérios |

## PET — Pets, identidades e adaptadores

Contrato de revisão: [PETS-E-CONTRATOS.md](PETS-E-CONTRATOS.md). Prova transversal: Dono/sessão exatos, política nativa, capacidades reais e ausência de fallback.

| ID | Trabalho | Dependência e aceitação integral |
| --- | --- | --- |
| PET-01 | Implementar `PetDescriptor` e capacidades versionadas a partir de `pet_factory`, `pet_config`, modelos e adaptadores | Matriz por pet distingue suporte, configuração, disponibilidade, frescura e motivo. API/perfil vazio ou hook não aparecem como execução suportada. Nome/modelo não alteram identidade; GitHub entidade continua retirado |
| PET-02 | Associar conversa, sessão nativa, turno, tarefa e decisões com autoridade inequívoca | Trocar de pet/vista/sessão, resultados tardios, sessão ambígua, revogação e Stop preservam dono. Terminal/chat/voz partilham a identidade do pedido nativo e a política da sessão/perfil; uma resposta invalida as outras representações. Descoberta por PID/pasta/título não autoriza controlo |
| PET-03 | Completar continuidade e decisões Gravity/Copilot/Codex segundo capacidades nativas | Continuar a execução original quando suportado, decisões exatas e limpeza dos filhos próprios. Gravity sem canal de aprovação não ganha permissões. Copilot usa fixtures enquanto quota real está vedada. Preservar as capacidades do agente original — ficheiros, comandos, projetos e ferramentas/MCP — conforme a sessão/perfil autorizados; distinguir continuação nativa do dispatcher autónomo ainda sem limites internos garantidos. Verificar paridade de sandbox/política/modos de aprovação, sem flags mais permissivas nem concessões copiadas entre sessões |
| PET-04 | Consolidar identidade, modelos, ferramentas e falhas locais/API | Ollama/LM Studio/API mantêm modelo e contexto escolhidos, recusam ferramentas não suportadas e não fazem fallback silencioso. Completar ligação de pets ainda sem configuração/adaptador resolvido |
| PET-05 | Definir o contrato comum dos pets de serviço e observadores | Spotify permanece determinístico nas capacidades atuais; integrações com executor LLM conservam dono próprio. Só operações efetivamente implementadas ficam disponíveis |

## RUN — Runtime e recuperação

Contrato de revisão: [ARQUITETURA-COMPLETA.md](ARQUITETURA-COMPLETA.md). Prova transversal: Background, limites, crash/retoma, Stop e um dono por efeito.

| ID | Trabalho | Dependência e aceitação integral |
| --- | --- | --- |
| RUN-01 | Schema e repositório de objetivos, tarefas, planos, passos e eventos; migração do estado atual | PET-01/02. Separar atividade de estado durável; guardar admissão antes de efeitos, preservar IDs e trabalhos ativos além dos dez do journal público |
| RUN-02 | Admissão, fila, leases e associação da conversa de background | RUN-01. Não trocar seleção/rascunhos da UI; uma sessão não recebe dois controladores. Prazos incluem fila, slots físicos aguardam fim do worker, outros pets continuam. Prontidão fresca antes do efeito; validade durante IO não depende apenas da idade da cache e continua vinculada a política/posse/contexto/prazo |
| RUN-03 | Ciclo comum de planeamento, progresso, orçamento e conclusão | RUN-01/02 + EXE-01. Critérios verificáveis, limites de ciclos/tentativas/recursos, falta de progresso detetada; texto do modelo não basta para concluir |
| RUN-04 | Recuperação, pausa, Stop e retoma após falha | RUN-01/02. Reinício não reproduz efeitos; reconciliar processos/sessões/recibos, preservar incerteza e permitir retoma vinculada ao objetivo original |
| RUN-05 | Prioridades e justiça entre trabalho manual e rotinas | RUN-02 + OBS-01. Stop/decisões responsivos, nenhuma rotina monopoliza provider/modelo/dispositivo; limites configurados e observáveis |

## EXE — Ferramentas e efeitos

Contrato de revisão: [ARQUITETURA-COMPLETA.md](ARQUITETURA-COMPLETA.md). Prova transversal: Operação tipada, decisão exata, alvo e pós-condição, incerteza sem replay.

| ID | Trabalho | Dependência e aceitação integral |
| --- | --- | --- |
| EXE-01 | Registo/broker de operações tipadas para ferramentas nativas, locais, MCP e desktop | PET-01/02. Schema, versão, efeitos, alvos, limites, cancelamento, idempotência e verificação; ferramenta descoberta não fica automaticamente autorizada |
| EXE-02 | Concessões por objetivo/regra e receitas limitadas; integrar aprovações existentes | RUN-01 + EXE-01. Uma aprovação não se torna shell genérica; política revogada invalida pedidos pendentes, sem conceder privilégios de outro pet. Chat/voz acompanham a política e os pedidos do agente original, com consumo único, sem duplicar pedidos já satisfeitos nem alargar a decisão nativa |
| EXE-03 | Recibos e verificação de pós-condições | EXE-01. Separar ACK/saída/efeito observado; erro após possível efeito exige reconciliação, sem repetir automaticamente |
| EXE-04 | Uso do computador e capacidades de observação | EXE-01/02. CLI/API/acessibilidade primeiro; alvos exatos, observação antes/depois em controlo visual. Sessão bloqueada ou falta de imagem tem limitação explícita |

## CTX — Contexto, anexos e fontes

Contrato de revisão: [DADOS-MEMORIA-E-FONTES.md](DADOS-MEMORIA-E-FONTES.md). Prova transversal: Vários itens, cobertura/revisões, revogação e preservação de cópias.

| ID | Trabalho | Dependência e aceitação integral |
| --- | --- | --- |
| CTX-01 | Fechar anexos múltiplos, retenção e recuperação do protótipo de 633 fontes | Bloqueador REL-01. Envelope terminal e aceitação nativa de restauro parcial/nome reutilizado/limpeza interrompida/motivos de recusa verificados nos dois incrementos675 privados; completar integração e migração/backup/rollback v4/v5/leitores antigos |
| CTX-02 | Apresentação por mensagem e seleção de anexos para contexto | CTX-01. IDs/proveniência/cobertura visíveis onde necessário; não reautorizar cópias nem inferir leitura integral do modelo |
| CTX-03 | Completar gestão e aceitação do catálogo ICS já implementado | Formulário/consentimento nativos e fonte selecionada; calendário exportado não aparece como sincronização ou capacidade de escrita |
| CTX-04 | Evoluir fontes de pet/CID para objetivos/regras e coleções persistentes | RUN-01 + EXE-02 + CTX-03. Migração sem alargar concessões antigas; seleção explícita de vários calendários, revisão e revogação por ocorrência |
| CTX-05 | Adaptadores RSS/IMAP com leitura limitada e proveniência | EXE-02 + CTX-04. TLS, redirecionamentos/destinos, bytes/prazos e mensagens limitados antes de materializar conteúdo; IMAP sem envio/eliminação implícitos |

## MEM — Memória e recuperação seletiva

Contrato de revisão: [DADOS-MEMORIA-E-FONTES.md](DADOS-MEMORIA-E-FONTES.md). Prova transversal: Origem/âmbito, seleção/destino, correção e invalidação de derivados.

| ID | Trabalho | Dependência e aceitação integral |
| --- | --- | --- |
| MEM-01 | Schema de memória: preferências, contexto de projeto, episódios, derivados e resumos | PET-01 + RUN-01. Dono/âmbito/proveniência/validade/revisão; dados derivados não viram instrução nem autorização |
| MEM-02 | Recuperação seletiva, correção, exclusão, revogação e retenção | MEM-01 + CTX-01. Pesquisa estruturada/lexical inicial, isolamento entre pets/projetos e invalidação de derivados/índices; embeddings externos só com concessão própria |

## AUTO — Rotinas e proatividade

Contrato de revisão: [ARQUITETURA-COMPLETA.md](ARQUITETURA-COMPLETA.md). Prova transversal: Regra autorizada, fuso/DST/offline, ocorrência única e silêncio.

| ID | Trabalho | Dependência e aceitação integral |
| --- | --- | --- |
| AUTO-01 | Repositório de regras, gatilhos e concessões; ativar/desativar por utilizador | RUN-01/02 + EXE-02 + CTX-04. Regra tem dono, revisão, fontes, limites e validade; conteúdo externo não inscreve regras |
| AUTO-02 | Agendador de horários/eventos e deduplicação de ocorrências | AUTO-01. Fuso IANA, mudança de hora, suspensão/offline e políticas skip/coalesce/decisão; nenhum efeito duplicado por evento repetido |
| AUTO-03 | Primeira rotina: preparação diária a partir de calendários escolhidos | AUTO-01/02 + EXE-03. Dados sintéticos desconhecidos pelo modelo, várias fontes, frescura/cobertura, falha parcial e uma única notificação no horário permitido |
| AUTO-04 | Rotina de acompanhamento de objetivo/projeto e pedido de decisão | RUN-03/04 + MEM-02. Progresso verificável, interrupção/retoma sem repetir efeitos, orçamento repartido e comunicação sem spam |
| AUTO-05 | Notificações, silêncio e escalada de bloqueios | OBS-01 + AUTO-01. Conteúdo sensível limitado, deduplicação e validade; indisponibilidade não causa reenvio infinito ou mudança de canal/conta |

## TEAM — Comunicação e cooperação

Contrato de revisão: [ARQUITETURA-COMPLETA.md](ARQUITETURA-COMPLETA.md). Prova transversal: Ramos paralelos, sem coordenador, ciclos/conflitos, ACK perdido e revogação.

| ID | Trabalho | Dependência e aceitação integral |
| --- | --- | --- |
| TEAM-01 | Protocolo comum de comunicação direta, descoberta e delegação | PET-01/02 + RUN-01 + EXE-02. Mensagens/pedidos/respostas/eventos/resultados com origem/destino, correlação/causa, tarefa/objetivo, idempotência, artefactos, autoridade, orçamento e prazo explícitos; reutilizar mecanismos existentes. Utilizador e pets chamam cada pet diretamente; nenhuma delegação amplia permissões |
| TEAM-02 | Execução paralela/assíncrona, verificação, cancelamento e coordenação resiliente | TEAM-01 + RUN-03/04 + EXE-03. Mesmo controlo para tarefas diretas/delegadas; um dono por efeito, entrega deduplicada, limites de profundidade/concorrência/custo, ciclos/conflitos recusados e falhas isoladas. Coordenador Ollama configurável, com visão autorizada dos trabalhos, sem intermediação obrigatória; restantes pets operacionais sem coordenador/motor indisponível. Resultados com proveniência, sem copiar o histórico inteiro |

## SYS — Sistema e permissões

Contrato de revisão: [SISTEMA-E-PERMISSOES.md](SISTEMA-E-PERMISSOES.md). Prova transversal: Partilha Som Spotify/Ollama, recusa entre componentes e efeito nativo.

| ID | Trabalho | Dependência e aceitação integral |
| --- | --- | --- |
| SYS-01 | Matriz pet × subplugin × operação × recurso; autoridade dos pets ligada ao executor comum | PET-01 + EXE-01/02. Plugin Sistema não concede tudo; Som pode ser partilhado por Spotify/Ollama; revogação de um pet preserva o outro |
| SYS-02 | Rede e Bluetooth como uma unidade de execução | SYS-01 + EXE-03. Estado/scan/alvos/prompts/perfis e operações suportadas; distinguir implementação/ACK/efeito. Sem desligar Wi-Fi nem alterar dispositivos em QA sem autorização específica |
| SYS-03 | Som/multimédia; ecrã/energia/sessão em subunidades completas | SYS-01 + EXE-03. Streams/dispositivos/perfis e restante âmbito da arquitetura; partilha/concorrência verificadas e indicadores preservados |

## INT — Integrações e serviços

Contrato de revisão: [PETS-E-CONTRATOS.md](PETS-E-CONTRATOS.md). Prova transversal: Rascunho separado de envio, dono próprio e operação real autorizada.

| ID | Trabalho | Dependência e aceitação integral |
| --- | --- | --- |
| INT-01 | Integrações de serviço e comunicação externa com operações distintas | PET-05 + EXE-02/03. Abrir rascunho não envia; envio futuro exige concessão própria. Autenticação/tokens e identidades nativas preservados |

## VOICE — Voz integral

Contrato de revisão: [ARQUITETURA-COMPLETA.md](ARQUITETURA-COMPLETA.md). Prova transversal: Mesmos contratos de chat, decisão única, Stop e áudio real PT/EN/misto.

| ID | Trabalho | Dependência e aceitação integral |
| --- | --- | --- |
| VOICE-01 | Controlo e interação integral do sistema de pets por voz | PET-02/03 + EXE-01/02 + CTX-04 + SYS-01. Cobertura por operação de pets/sessões/conversas/objetivos/tarefas/regras/fontes/memória/configuração/modelos e Sistema autorizado; mesmos dono, permissões e pedidos nativos que chat/terminal. Decisão única por pedido, clarificação falada de ambiguidade, Stop prioritário e sem reenvio após reconexão |
| VOICE-02 | Aceitação real de microfone/Bluetooth/TTS PT/EN/misto | VOICE-01. Latência/pronúncia/alternância/falhas, conversa contínua, interrupção de TTS e reconexão com áudio real; matriz de cobertura funcional por voz. Uma transcrição escrita não fecha este requisito |

## OBS — Estado e consumo

Contrato de revisão: [ARQUITETURA-COMPLETA.md](ARQUITETURA-COMPLETA.md). Prova transversal: Terminais/clientes/sessões separados, cobertura e ausência distinta de zero.

| ID | Trabalho | Dependência e aceitação integral |
| --- | --- | --- |
| OBS-01 | Estado, decisões, prazos, recursos e consumo por tarefa/pet | Distinguir terminais/TTYs, clientes e sessões, com atividade/espera/suspensão/inatividade confirmadas ou desconhecido; lease preparado não é inferência. Fontes/cobertura explícitas, sem segredos/documentos privados em logs |
| OBS-02 | Cobertura/rotação dos ledgers e captura de consumo nativo | Separar tentativas/thread/conta/quota; não perder notificação antes do ACK, não somar snapshots como deltas e não autenticar por abrir painel |

## DATA — Persistência e bases de dados

Contrato de revisão: [DADOS-MEMORIA-E-FONTES.md](DADOS-MEMORIA-E-FONTES.md). Prova transversal: Publicação concorrente/parcial, restauro e migração sem alargar acesso.

| ID | Trabalho | Dependência e aceitação integral |
| --- | --- | --- |
| DATA-01 | Repositórios, migrações, retenção e backups | Falhas antes/depois de publicação, escritor concorrente, reinício e restauro; nenhum dado ativo desaparece por rotação |
| DATA-02 | Decisão e eventual backend PostgreSQL | Benefício medido e esquema/base/utilizador próprios; IDs e concessões preservados, migração/restauro comprovados; estado atual continua sem integração |

## REL — Adoção e entrega

Contrato de revisão: [MIGRACAO-E-ENTREGA.md](MIGRACAO-E-ENTREGA.md). Prova transversal: Composição exata, perfil/writers, backup, compatibilidade e aceitação instalada.

| ID | Trabalho | Dependência e aceitação integral |
| --- | --- | --- |
| REL-01 | Composição, adoção das fontes e migração/rollback | Fontes/hash exatos, preservação de dados/configuração/rascunhos e resolução de bloqueadores; journal v5 e marcadores pendentes tratados antes de regressar a leitores antigos |
| REL-02 | Entrega e aceitação integrada por capacidades | Executável/assets/helper/autostart/login/reinício, hotplug e trabalhos interrompidos; paridade funcional dos centros legados antes de descontinuar capacidades, mantendo os indicadores. Rádio/dispositivos/contas respeitam o âmbito autorizado |

## Cenários integrados que o utilizador deve conseguir aceitar

| Cenário | IDs envolvidos | O que distingue aceitação de demonstração parcial |
| --- | --- | --- |
| Preparação diária com vários calendários | PET, RUN, EXE-02/03, CTX-03/04, AUTO-01/02/03/05, OBS-01 | Dados novos/sintéticos com origem, timezone, fonte indisponível e uma ocorrência/notificação única. |
| Objetivo com a conversa fechada | PET-02/03, RUN-01/02/03/04/05, EXE-01/02/03, OBS-01, REL | Mesmo dono e contexto, decisões acessíveis, Stop e recuperação com critérios preservados. |
| Vários ficheiros e memória | CTX-01/02, MEM-01/02, DATA-01, REL-01 | Adição cumulativa, seleção por turno, limites, correção/revogação e restauro sem perder referências. |
| Crash com possível efeito | RUN-04, EXE-03, DATA-01, REL | Não repetir chamada/comando, consumo/incerteza conservados, consulta/retoma autenticadas. |
| Aprovação no chat/voz/terminal | PET-02/03, EXE-02, VOICE-01 | Uma única decisão sobre o pedido original; as outras representações deixam de o poder resolver. |
| Pets em paralelo sem coordenador | TEAM-01/02, PET, RUN, EXE-02/03 | Mensagens diretas, ramos independentes, limites/ciclos/conflitos e nenhuma herança de autoridade. |
| Som partilhado | SYS-01/03, PET-05, EXE-02/03, VOICE-01 | Spotify e Ollama usam só o Som concedido, revogação isolada e Rede/Energia recusadas. |
| Assistente por voz | VOICE-01/02 e IDs das operações invocadas | Cobertura integral das operações, clarificação e Stop; microfone/Bluetooth/TTS reais PT/EN/misto. |
| Upgrade e rollback | CTX-01, DATA-01, REL-01/02 | Rascunhos, arquivos, perfil, autoridade e cópias preservados; leitor antigo recusa o que não entende. |

## Níveis de prova e cobertura

Cada célula de pet/capacidade identifica: desenhado, implementado, fixture, nativo, integrado, entregue, indisponível ou pendente. Um teste nativo pode usar o IPC real e um provider sintético; isso não é uma conta real certificada. A ausência de quota/dispositivo é uma pendência identificada, sem a contornar declarando conclusão.

As recusas são parte do contrato: dono errado, sessão ambígua, grant revogado, prazo ultrapassado, dados alterados, replay, worker ainda ativo, destino/modelo mudado e falha de persistência. A prova conserva os resultados negativos e os limites conhecidos.

Nenhuma ocorrência histórica de «passou» numa fonte substitui os critérios desta matriz. Alterações futuras devem conservar estes 43 IDs e associar os pedidos posteriores aos IDs aplicáveis, revendo o contrato antes de apagar ou reduzir âmbito.

