# Estado de implementação dos pets

Observação documental: **24/10/2026**. A
[arquitetura](PETS-ARCHITECTURE.md) define requisitos e prioridade; o
[roadmap](PETS-ROADMAP.md) define unidades e aceitação. Este documento distingue
código, provas e entrega. Não há percentagem global de conclusão comprovada.

**Estado corrente:** 759 fontes integradas e versionadas. Os 400
inputs nativos e 774 web são os validados: 2373 Rust/20 ignorados, 1153 JavaScript,
cargo check, TypeScript e Vite. Passaram 277 ensaios privados no anfitrião de
runtime/metadados/leitor/operações e decisões; cinco helpers são invocados pelos
pais. Mantêm-se 48 avisos no check e 41 nos testes, sem novos. A app instalada
continua na **.30**, sem alterações ao perfil original ou inferência/quota real.
[Integração](PETS-VALIDATION.md#evidencia-local).

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
| Checkout de trabalho | `/home/pmglo/Projects/coucou-codex`, branch `feat/mochi-cli-accounts`; composição de código assinada e documentação em publicação |
| Composição atual integrada | 759 fontes; entrada funcional em Tarefas e confirmação que tolera renovação da observação, mantendo o âmbito original; 2373 Rust/20 ignorados, 1153 JS e 277 ensaios privados; [validação versionada](PETS-VALIDATION.md) |
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
| Arquitetura, fábrica e adaptadores | Catálogo, identidades e validação de donos/assistentes opcionais integrados e verificados; concluir contratos comportamentais e aceitação de todos os adaptadores, incluindo Gravity e Copilot CLI |
| Sessões e permissões nativas | Contextos/origem/decisões e censo distintos já têm bases; origem comum verificada em Codex gerido/externo, Copilot ACP e gate Gravity; escolhas modernas Codex/ACP verificadas; IO capturado protegido; formulários MCP tipados/adoção externa verificados no subconjunto suportado; vida/resposta de callbacks de operações verificada no broker; concluir aceitação de continuidade/política/cobertura por adaptador e política/sandbox/aprovações do agente original com decisão única entre terminal, chat e voz — PET-02/03, EXE-02 |
| Execução autónoma e verificação — atual | Fila, consentimento, leases, contexto, writer textual e recibos duráveis integrados. Broker interativo comum e propostas de operações nos planos; nova admissão sela o trabalho exato e continua só textual. A receita Spotify revista já tem execução limitada, critérios tipados e conclusão autenticada; há primeira aprovação partilhada em chat/voz determinística. Há entrada funcional em Tarefas e inspeção local explícita, com decisões de gestão limitadas a objetivos. Continuar planeamento/replaneamento, despacho nativo, restantes executores/verificadores/adaptadores, aceitação real, prioridades e reconciliação explícita de efeitos — RUN/EXE |
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
checkout. Seguem ferramentas/planos/verificação e a aceitação/entrega.

### 2.2 Bases existentes e lacunas por capacidade

| Unidade | Base existente | Lacuna e ID de continuação |
| --- | --- | --- |
| Identidades e fábrica | `pet_factory` com moldes/herança, retirada separada e metadados de runtime tipados desde a unidade654, integrados na759; lookup nativo de identidade/serviços, classificação/binding e preparação de voz usam o catálogo | PET-01/04/05 e ARC-02: completar contratos comportamentais/adaptadores/controladores, modelos e aceitação; migrações preservam overrides e a fábrica não concede execução |
| Conversas | Biblioteca, arquivos, IDs, origem e tarefas por pet; gravação e cancelamento com dono | PET-02, RUN-01/02: associação de objetivos e conversas de background sem trocar a vista corrente |
| Sessões CLI | Composição742 com origem/decisões/formulários/IO comuns, configuração ACP preservada e retoma Gravity validada; processos e sockets privados verificados | PET-03: completar continuidade e controlo/decisões nas capacidades nativas, sobretudo Gravity; uma nova sessão não prova retoma da TUI original |
| Modelos locais | Loop Ollama Ask, propostas congeladas, aprovação por comando, prazos/limites e recibos | PET-04, RUN-03: contrato comum de objetivo/progresso; modelo sem tools não recebe execução; não assumir a mesma capacidade no LM Studio/API |
| Trabalho persistido | Repositório por objetivo integrado, separado do journal antigo; propostas, revisão/consentimento autenticado, claims/leases, orçamento comum, recibos textuais, passos Spotify e primeiro executor de receita revista com concessões próprias | RUN-01/02/03/04: completar adoção/migração, execução nos restantes adaptadores, ciclo de planeamento/critérios, reconciliação sem prova nativa e aceitação/entrega; a existência dos ledgers não prova autonomia completa |
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
