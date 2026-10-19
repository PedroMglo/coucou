# Arquitetura dos pets companheiros — Linux

Este é o guia de estado e aceitação do fork. O quadro seguinte prevalece sobre
as notas históricas no fim do documento. Última revisão: 19 de outubro de 2026.

## Estado da entrega

A **0.5.1-mochi.14 está instalada e ativa** no prefixo imutável
`~/.local/opt/coucou/0.5.1-mochi.14`. Launcher, Super e autostart apontam para
esta versão. O arranque automático foi confirmado novamente após reiniciar o
PC em 19/10, sem reinícios por falha. O RPM de sistema .6 e o prefixo .12 foram
preservados para recuperação. A .13 foi rejeitada no ensaio nativo anterior.

O código pertence ao worktree `coucou-codex`; a integração Fedora pertence a
`sysworkspace-coucou-integrated`. Não alterar o outro fork. O relay oficial
confiado conserva o mesmo SHA. As entregas são registadas em commits assinados
separados da app e da integração; isso não equivale a concluir o objetivo geral.

- **766 testes Rust passam, 10 ignorados; 607 JS, TypeScript e Vite passam.**
  As cinco regressões do bridge CLI também passaram após explicitar as
  ferramentas locais de desktop nas instruções comuns dos agentes.
- **8 testes do importador passam.** Migração aplicada com **18 mensagens
  públicas, 8 trabalhos e 7 conversas**. Fontes e QA completa preservadas em
  backup privado; excluiu-se apenas o par sintético Ollama confirmado pela QA.
- O RPM final contém 13 ficheiros permitidos, incluindo os sete do runtime
  Copilot. Prefixo instalado, dados reais e relay ativo foram verificados.
- Nova ilha com **Pets/Sistema**, swipe touch, trackpad horizontal e teclado.
  Entrada geral com os seis pets, incluindo descanso; abrir Codex exige escolha.
  Sessões, tarefas, providers/quotas, modelos e perfis conservam a gestão legacy.
- Aceitação nativa confirmou galeria, navegação, gestão e os cinco painéis reais.
  Após reiniciar o PC, voltou a confirmar calendário/música/tempo centrais,
  notificações e definições rápidas: cerca de 0,45–0,49 s por pedido sequencial.
  O lote anterior sobreposto teve timeout e não foi contado como sucesso.
- Chat: seis donos corretos, aprovação real da sessão exata, flutuante arrastado
  1100 píxeis e escrita num editor exterior passaram no ensaio nativo.
  Ollama devolveu métricas reais: 201 tokens de entrada, 3 de saída e 6,4 s.
- Documentos: PDF na ilha, TXT no pet e pasta no flutuante abriram a escolha
  de ações. Anexo/remoção foram confirmados no flutuante. O Cancel do popup
  direto não foi confirmado antes de ele recolher; não generalizar essa prova.
- Fusão DMS aplicada, com **blocos/ilhas nativos preservados**. A ponte
  **MochiSpace 1.5** usa `noBackground: false`: apenas o canvas entre os blocos
  é transparente. A interpretação anterior de deixar ícones soltos foi rejeitada
  pelo utilizador. Um spacer nativo invisível reserva espaço para o Coucou e os
  indicadores de privacidade, sem desenhar outra ilha ou duplicar o relógio.
  No portátil de 1440×900 píxeis lógicos, o título é truncado pelo layout nativo;
  o mapa real confirmou folgas de 51 e 146 píxeis dos lados do compacto.
  Num ecrã mais largo o mesmo layout permite novamente o título mais longo.
- Parakeet e diarização mantêm os seis ficheiros com os tamanhos esperados;
  o ditado continua ativado. Novas implementações/ensaios de áudio estão adiados.
- O rascunho Ollama de 73 caracteres foi restaurado no compositor da conversa
  original após o login, sem o enviar. Não reiniciar a app sem o preservar.
  A aceitação visual final da barra com blocos e a revisão seguinte do chat
  responsivo são acompanhadas separadamente dos contratos de backend.

## Backend ainda por desenvolver

Estes itens são desenvolvimento real, separados dos ensaios do que já existe.
O objetivo geral permanece aberto enquanto houver requisitos por cumprir.

| Prioridade | Capacidade | Estado e próximo trabalho |
| --- | --- | --- |
| P0 | Componente Sistema completo na ilha | Novo pedido: migrar Wi-Fi, Bluetooth, som, bateria e restantes controlos do bloco superior direito para a página Sistema; descontinuar os acessos duplicados após aceitação. Hoje há cinco painéis, mas esse bloco ainda está na barra. Exigir estados reais, ações nativas, tray e plugins sem perda de funções; só depois retirar os widgets antigos. |
| P0 | Documentos na nova central | A revisão frontend identificou ausência do listener DND em `pet-control.ts`, embora o destino GTK exista. Reproduzir com documento sintético e integrar o handler na revisão seguinte; a aceitação anterior de documentos não certifica esta nova central. |
| P0 | Executor geral local para Ollama | Hoje só há ações limitadas de apps, música e email. Falta execução de tarefas com aprovação explícita, pasta de trabalho, prazo/cancelamento, saída limitada, progresso e journal por conversa, sem trocar de provider. Desenho privado em preparação; não está instalado. |
| P0 | Interagir com sessões Gravity já abertas | Conversa Mochi própria funciona. O cliente normal não expõe ainda um endpoint/CSRF nativo comprovado para a ponte existente; não anunciar chat externo como integrado. |
| P1 | Continuidade Copilot ao fechar a TUI | A extensão atual termina com o cliente que a aloja. Falta um modo nativo independente; modelo e decisões de sessões externas ainda pertencem ao terminal. |
| P1 | Continuidade Codex completa | Sessões geridas com daemon independente já têm API, UI e prova sem IA. Ferramentas dinâmicas/MCP arbitrárias e recuperação de perda de ligação ainda exigem o cliente nativo ou nova integração comprovada. |
| P1 | Ciclo de vida dos acessos a documentos | Completar validade/revogação/limpeza das concessões e explicitar o vínculo às conversas. Seleção de partes de uma pasta é uma melhoria adicional. |
| Adiado | Áudio e notas | Qualidade real Bluetooth PT/EN/mista, latência até Raquel, persistência Notes/áudio e respetiva aceitação; conservar o que já existe. |

## Entidades e capacidades

Só aparecem os pets configurados: Codex, Gravity, Copilot, Ollama, GitHub e
Spotify. Há um pet por provider ou entidade. Modelos Gemini/Claude oferecidos
pelo Gravity são escolhidos dentro desse mesmo pet.

| Pet | Conversa e execução | Dados e integrações |
| --- | --- | --- |
| Codex | Conta CLI nativa, ferramentas/aprovações originais, memória própria Mochi | Sessões externas Codex com alvo exato e histórico público |
| Gravity | Conta CLI nativa e os seus modelos, conversa própria Mochi | A ligação interativa a terminais Gravity existentes continua indisponível |
| Copilot | Conta CLI nativa por ACP, catálogo, esforço, ferramentas e decisões | Ponte na mesma TUI ensaiada; loader global .14 instalado, requer recarga/reabertura das TUIs anteriores |
| Ollama | Modelos locais e ações limitadas implementadas, voz/texto partilhados; executor geral ainda em falta | Documentos e rascunhos de email; manutenção exclusiva de modelos/serviço |
| GitHub | Integração da entidade e conversa própria via Codex, identificada como tal | A sessão da entidade não reutiliza o diálogo do pet Codex |
| Spotify | Abertura/controlo determinístico da app e conversa própria via Codex | Play/pausa, avanço, pesquisa e reprodução por URI disponível |

As três contas CLI recebem também instruções para descobrir `pet-desktop`,
verificar a sessão/janelas e usar alvos exatos, preferindo CLI/API/acessibilidade
e confirmando resultados reais. As ferramentas preservam as decisões nativas
do agente; não garantem automação de interfaces inacessíveis.

Os acessos dos plugins Coucou são independentes das políticas de execução das
CLIs. **Email está ativo apenas no Ollama por defeito.** Nenhuma ação de email
envia mensagens: o cliente abre um rascunho para revisão e envio pelo utilizador.
Os outros pets só apresentam funcionalidades efetivamente implementadas e
autorizadas. Não anunciar capacidades universais que o provider não suporta.

## Configuração e modelos

`~/.config/coucou/pets.json` guarda identidades, aliases, provider, perfis rápido
e raciocínio, esforço e acessos. A UI de gestão edita o mesmo contrato validado,
com controlo de revisão. O ficheiro de origem é
`windows/src-tauri/pets.default.json`; os valores reais podem ser personalizados.
Codex rápido usa gpt-6-luna/low, conforme pedido. A seleção automática dos perfis
usa a complexidade do pedido; a voz usa os perfis do pet.

A seleção Ollama usa `pet_model_select`: valida catálogo/perfis sem executar ou
carregar modelos, confirma gravação e devolve ACK com geração. Só a intenção
mais recente pode abrir o pet. `settings-changed` dessa operação inclui
`ollamaSelectionGeneration`, apenas no evento, para ordenar ACK/eventos de
janelas diferentes; essa geração não é gravada nas preferências. Snapshots
completos antigos não podem repor `chatModels.ollama`. No envio, o modelo
apresentado é capturado; se contradizer a escolha confirmada, o backend rejeita
o pedido antes de executar.

Alterações das preferências, gravação e notificação são serializadas. Falhar a
gravação preserva a RAM e devolve erro visível. A admissão do trabalho precede
a leitura das preferências efetivas; Parar exige a conversa e turno corretos.

A retoma de uma tarefa arquivada não usa a conversa atualmente selecionada.
`reserve_task_resume` revalida ID, fase e origem e restaura/reserva a conversa
original numa transação única. Busy, origem ausente ou ambígua recusam sem
alterar a seleção. Falhas sem sessão nativa iniciam uma conversa limpa e
arquivam o contexto anterior. Validar a pasta precede qualquer troca; a troca
invalida anexos antigos e publica a atualização. Um segundo pedido atrasado não
pode repetir uma tarefa já retomada. O journal mantém os últimos dez trabalhos
por pet, separado dos arquivos de conversas.

A finalização de trabalho e o cancelamento têm de ter um único vencedor:
paragem aceite não pode gerar um journal «done» sem resposta. A revisão atual
lineariza resposta e estado final sob o mesmo mutex e publica a atividade sem
regravar o journal. Gravação falhada após executar conserva resposta e aviso,
sem apresentar a execução como falhada ou sugerir reenvio automático. O arranque
Codex passa a receber o token de cancelamento antes do initialize. Estas correções passaram no conjunto de 766 testes nativos. O stdout/stdin
Codex usa escrita não bloqueante com prazo e cancelamento, inclusive quando o
processo deixa de ler; o encerramento termina apenas o grupo do processo próprio.
O trabalho guarda `conversationId` separado da sessão nativa e é persistido
antes de iniciar a execução. Uma falha de gravação não executa comandos.

## Voz e ditado

**Prioridade revista pelo utilizador em 19/10:** continuar diálogo escrito,
sessões e tarefas; novas implementações de voz, som, fala e ditado ficam para
uma fase posterior. Preservar funcionalidades, modelos e preferências já
existentes. A aceitação desta fase não depende de novos ensaios de áudio.

Super sozinha abre a escuta neutra. O nome do pet escolhe o interlocutor;
fechar a escuta não cancela automaticamente o trabalho. A escuta termina com
Super novamente ou por inatividade. Os trabalhos e decisões mantêm origem
Mochi/Terminal, provider, sessão/conversa e turno explícitos.

O utilizador descarregou Parakeet v3 e diarização e ativou ditado. Preservar os
ficheiros e preferências no deploy. `voiceAsrEngine` permite auto/Whisper/Parakeet
sem apagar instalações; auto mantém Whisper já configurado. Ditado e conversa
usam o mesmo reconhecedor e disponibilidade. A saída aceite é Raquel (Read
Aloud); diarização distingue intervenientes e não é uma voz TTS.

A comparação PT-PT/EN/mista sintética mostrou erros nos dois reconhecedores.
**Falta validar a fala real Bluetooth e medir a latência até som.** Não afirmar
que a qualidade está resolvida apenas por os modelos estarem disponíveis.
Os campos de aprendizagem de pronúncia no frontend final3 seguem agora os
pets configurados, incluindo Copilot, GitHub e Spotify.

## Conversas, documentos e interface

A ilha central usa os componentes originais Coucou, animação de expansão/fecho,
máscara transparente e sobreposição sem reservar área de tiling. Fecha ao sair
o rato. A antiga casinha deixou de ser o acesso principal. A gestão integra as
funcionalidades legacy: sessões/subagentes/filtros, quotas, modelos por digest,
recursos, retenção, configuração, abrir/retomar/interromper/arquivar e serviço.

O chat flutuante partilha a apresentação do chat do pet, com sidebar recolhível,
pesquisa, histórico e origem inequívoca. Ler um arquivo não muda a conversa
ativa; enviar/restaurar exige alvo e admissão corretos. Fechar a janela não
cancela o trabalho. Após abrir o flutuante com sucesso, recolhe o cartão do
mesmo pet, preservando aprovações. Erros de entrega desconhecida conservam o
pedido para revisão, sem o reenviar automaticamente.

O journal v3 preserva texto público e arquivos por pet; não grava raciocínio
oculto nem áudio. A biblioteca instalada e a privada de QA são distintas.
**Não substituir o journal instalado pelo de QA.** A migração final deve
preservar o Codex real, as novas conversas Codex de QA, todos os trabalhos e a
conversa Ollama original. `linux/migrate-pet-memory.py` prepara um ficheiro v3
privado novo: mantém a conversa instalada ativa e as adicionais distintas como
arquivos, reúne o journal de trabalhos, recusa conflitos ou perdas por retenção
e nunca substitui as fontes. Aplicar apenas com as duas instâncias paradas,
backup e rollback; preparar o ficheiro não equivale a migrar a instalação.
O rascunho Ollama de 73 caracteres foi guardado privadamente e restaurado no
compositor correto após o último reinício, sem enviar. Antes de outro reinício,
preservar novamente os campos ainda não enviados.

O drop nativo captura as conversas e revisões no mouse release, antes de receber
as URI assíncronas. Só cria o token opaco após validar essas URI contra o mesmo
contexto, prazo e geração. Os eventos pertencem exclusivamente à webview de
origem; não são emitidos globalmente para todas as superfícies.
Cópia fora do IPC, ordenação, validade, reset, restauro e cancelamento impedem
que um ficheiro passe para outro diálogo. Os acessos são revalidados antes de
executar ferramentas. PDF/pasta passaram no drop real; o anexo apareceu no
flutuante e Retirar anexo limpou o contexto. O cartão oferece as ações
permitidas; Ações do documento permite recuperá-las no chat compacto. A auditoria
v2 acrescentou handlers no chat/controlo, captura não bloqueante e limpeza da
altura/estado de drag e feedback ao dono original quando a entrega é rejeitada.
Os 31 testes nativos de documentos e cenários JS passaram. A repetição real
aceitou as três superfícies: PDF na ilha, TXT no pet e pasta no flutuante, com
escolha de ação, anexo no contexto e remoção no flutuante verificados. O Cancel
direto não foi confirmado antes de o popup recolher; falta essa aceitação.

As leituras/reset/restauro/cancelamento de chat e leitura/cancelamento do journal
de trabalhos executam fora do main thread
GTK, evitando que o polling espere nesse thread pelo mutex e fsync do journal.
Ao ocultar/recolher, as superfícies libertam a ativação do teclado. O baseline
com um editor GTK exterior não reproduziu captura global; o relato do utilizador
continua a exigir aceitação nativa. O ensaio descobriu também que o keymap mínimo
do `wtype` pode enviar hardware codes incorretos para atalhos em GTK; essa
limitação de automação não deve ser apresentada como defeito corrigido do chat.

`pet_usage` expõe métricas por provider e conversa. Ollama recebe contagens
reais do servidor e duração medida; valores ausentes são `null`, nunca estimados
pelo tamanho do texto. A UI identifica o último turno concluído ou parcial.
Texto mede a duração HTTP; voz usa a duração nativa das chamadas de inferência,
somando chamadas de ferramentas sem incluir o tempo da própria ferramenta.
O cache é limitado e apenas em RAM; outros providers e reinícios começam sem
medição. A confirmação visual com inferência Ollama local passou; não anunciar
contadores de providers que não os devolvem.

A auditoria de Notes fica registada para a fase posterior: confirmar gravação
antes de publicar RAM/sucesso; serializar writers de edição/transcrição/resumo
com temporários exclusivos; conservar áudio quando a persistência falha;
validar IDs e alvos de eliminação e preservar checkboxes durante um resumo.
Não aplicar os protótipos privados nem executar gravações reais nesta fase.
As quatro correções Flow já entregues tratam confirmação de definições, sem
introduzir novas funções de áudio. A respetiva fila frontend conserva intenção,
ordem, erro e repetição explícita, sem repetir ações de escrever noutras apps.

## Fusão autorizada do centro com horas, música e meteorologia

O pedido mais recente autoriza **a fusão funcional nesta entrega**, substituindo
as notas históricas de preparação com a opção desligada. `petSystemCenter` é
ativado explicitamente nesta estação; o valor genérico continua desligado.
A única ilha Coucou conserva animação/máscara original, relógio/data, estado de
música/meteorologia e acesso a todos os pets. A reserva superior passa de 288
para 432 píxeis; a expansão de gestão tem 640×452, sem reservar área de tiling.

`pet_system_panel` aceita exclusivamente calendário, música, meteorologia,
notificações e definições rápidas. Usa `/usr/bin/dms`, valida o ACK e termina o
cliente em caso de timeout. Com DankIsland antigo, `dash openAt` pode não ter
uma barra clássica: só esse erro conhecido autoriza fallback para `dash open`.
O cartão recolhe apenas após ACK; erros permanecem visíveis para recuperação.
Os cinco percursos passaram nativamente antes e depois do cutover, incluindo
o novo arranque após reiniciar o PC.

A ponte MochiSpace 1.1 devolve metadados públicos limitados e datados dos serviços
nativos MPRIS, meteorologia e notificações. Campos ausentes são desconhecidos,
sem estimativas. A referência de meteorologia expira após 20 segundos sem
consumidor. A app recusa versões/frescura/valores inválidos e limita o IPC.

A substituição persistente da barra pertence a sysworkspace. O migrador
`pets-system fusion default true` altera apenas `island`, `centerWidgets` e
`noBackground` e `transparency`, preservando todos os outros campos e widgets laterais. Recusa
layouts imprevistos; rollback simétrico usa `false`. O pedido IPC não prova
durabilidade: verificar o ficheiro gravado e o aspeto real. Instalar e verificar
primeiro o prefixo .14 para que um reinício nunca deixe a estação sem relógio.
Capturar backup privado antes da troca. Não aplicar o template DMS inteiro.

**Correção de apresentação de 19/10:** manter as várias ilhas/blocos originais
da barra, transparentes apenas nos intervalos. `noBackground` fica `false` e o
canvas usa alpha zero. `centerWidgets` contém apenas um spacer nativo invisível
de 612 píxeis: 432 do compacto e folga para os indicadores de privacidade que
seguem o título à esquerda. O DMS adapta a largura do título a este espaço em
cada ecrã, conservando os widgets e a sua ordem. O mapa MochiSpace omite só esse
spacer exato; a reserva real do compacto já é considerada pelo habitat dos pets.
Não trocar uma barra com blocos por ícones soltos nem remover funções no ecrã
mais pequeno. A integração não altera QML do pacote DMS instalado.

## Aceitação e pendências reais

| Área | Prova disponível | Trabalho restante |
| --- | --- | --- |
| Navegação dos seis pets | Abrir, expandir, sidebar e fechar passaram no artefacto .14 instalado e após login | Próximo lote: corrigir destaque de seleção desatualizado no roster |
| Ilha original | Galeria, animação, máscara e fecho ao sair passaram no .14 após login; blocos DMS repostos | Concluir aceitação visual da barra corrigida no portátil; manter adaptação ao ultrawide |
| Chat | Alvos, arquivos, decisão real da sessão exata, drag e escrita exterior passaram nativamente | Próximo lote: falhas de fechar/Escape/resize com erro visível e retry; validar sessões externas suportadas |
| Documentos | PDF/ilha, TXT/pet e pasta/flutuante passaram, com escolha de ação, anexo e remoção no flutuante | Confirmar Cancel direto; completar ciclo de concessões; não enviar documentos reais |
| Persistência das definições | Quatro setters Flow e fila frontend com ordem/erro/repetição integrados; 30 testes filtrados passam | Novos writers Notes/áudio adiados pelo utilizador |
| Email | Digitar assunto/corpo passou; causa do desvio para Codex identificada e corrigida no frontend | Confirmar rascunho/anexo no Betterbird, sem enviar |
| Voz — fase posterior | Modelos locais completos; funcionalidades e preferências existentes preservadas | Adiado pelo utilizador: qualidade Bluetooth PT/EN/mista, resposta Raquel, latência e persistência Notes/áudio |
| Modelos | ACK, perfis, ordenação entre janelas, gravação e admissão têm regressões; carregar/libertar gemma3:4b passou na UI real e API ps voltou vazia | Conservar modelo configurado e explicitar modelos sem métricas |
| Quotas | Cache local e planos recebidos; warning/stale/unknown e deduplicação implementados | Frescura depende do produtor existente/atualização manual; aceitar apresentação |
| Continuidade Codex | API gerida, 44 testes, UI de criar/abrir/libertar e prova sem IA: fechar TUI remota manteve turno, ferramenta e nova aprovação no daemon independente | Aceitar no runtime final; não generalizar a clientes com ferramentas dinâmicas/MCP |
| Gravity externo | LS normal embutido sem endpoint/CSRF herdado comprovado | Bloqueio real de integração; não substituir sessão por outra ou inventar ligação |
| Copilot externo | Envio/histórico reais na própria TUI ensaiados; loader .14 instalado | Requer recarga explícita de TUIs antigas; fechar TUI termina a ponte |
| Entrega | RPM .14 instalado; migração e backups dos dados, autostart/loader e relay verificados após reboot | Continuar os itens de backend e a fusão completa do bloco Sistema descritos no início |

A prova de continuidade não equivale a uma retoma: o trabalho original tem de
manter o mesmo turno e decisões. Codex com daemon independente e ferramentas
nativas foi exercitado com fixture local, sem inferência. A API não certifica
ainda dependências arbitrárias do cliente original. Não fechar terminais reais
como teste nem prometer transferência genérica.

Melhorias posteriores à aceitação incluem seleção de partes de pastas e
retenção dos grants de ficheiros. Não reabrir como implementação em falta
funcionalidades que já têm código e apenas aguardam aceitação nativa.

## Responsabilidade e verificação

A sessão frontend tem TS/CSS/HTML e rato/monitor. A coordenação backend valida
protocolos e capacidades; a integração mantém Rust/runtime, builds, este guia
e as fontes do anfitrião. Vite termina antes de Cargo para garantir os assets
embutidos. Capturas, logs, fixtures, sessões e rascunhos ficam privados em
`.build`; não os versionar. Os handoffs privados registam os contratos e o
runtime atual. Para QA sem servidor de desenvolvimento, compilar com
`cargo build --offline -p coucou --features tauri/custom-protocol`. O build sem
essa opção procura `devUrl` e não valida os assets incorporados; o RPM por
Tauri CLI já ativa esse protocolo.

<details>
<summary>Histórico de pedidos, decisões e ensaios anteriores (não substitui o estado acima)</summary>

# Arquitetura e evolução dos pets companheiros

Atualizado em 18 de outubro de 2026. Este é o guia de implementação e aceitação
do fork. O âmbito de desenvolvimento e aceitação é exclusivamente Linux
(Fedora/Hyprland); `windows/` é o nome herdado das fontes Tauri e não um alvo
desta intervenção. O objetivo é ter assistentes acessíveis desde o arranque que executam e
acompanham tarefas reais, com conversa escrita e falada em português de Portugal
e inglês. Uma implementação só fica concluída depois de passar a validação
adequada; testes simulados não equivalem a um teste na aplicação instalada.

Mochi.12 está instalada desde 18/10/2026 às 18:00. A mochi.13 foi compilada e aplicada às 18:45, mas a aceitação nativa falhou: a superfície e o JavaScript principal não arrancaram corretamente. Foi feito rollback exato para a mochi.12 às 18:48. A mochi.14 está em diagnóstico; não há nova aceitação de voz/documentos. O painel AI da barra foi
desativado depois da verificação nativa das vistas Modelos, Providers e quotas e
Sessões. A aceitação anterior de voz pelo utilizador refere-se à mochi.10; a
transcrição PT/EN nova e a matriz completa de pets continuam em validação. O
manifesto do sysworkspace fixa o artefacto e os backups efetivamente ativos.

## Coordenação ativa das três sessões

A sessão frontend `frontend` assume exclusivamente
TS/CSS/HTML e ensaios visuais. Root assume Rust/runtime, builds, integração do
anfitrião e este guia; a sessão Coordena assume QA/backend e o comando atómico
de seleção do modelo Ollama. O defeito de retorno de foco após aprovação em
modo pets foi entregue à frontend, ainda sem correção no State. Fronteiras,
runtime de QA e dados a preservar estão no handoff privado de coordenação
`.build/TEAM-HANDOFF.md`. Vite deve terminar antes do build nativo.

### Contrato de persistência e seleção Ollama em integração

O backend geral de preferências passa a devolver erro e preservar a RAM se a
gravação falhar; patch, gravação e notificação são serializados pelo mesmo
mutex. O modelo Ollama passa a ser propriedade do comando específico
`pet_model_select`, evitando que um snapshot antigo de outra página desfaça
a escolha confirmada. A frontend tem de encaminhar todas as escolhas de
Ollama para esse comando, incluindo perfis; o backend específico tem de
aceitar os perfis @default/@fast/@reasoning e validar o modelo efetivo por
metadados. Não ativar o próximo artefacto até integrar/testar ambos os lados.

## Correção de aceitação em curso

A mochi.13 passou 501 testes JS, 553 testes Rust (10 ignorados), testes de
browser e 456 testes do sysworkspace, mas falhou no WebKit real após instalar.
Esses testes não validam por si o arranque da app. O rollback repôs os três
ficheiros de `repo-20261018-184545-h07kpbl7` e o hook Gravity voltou à .12 com
backup `hooks.json.bak-20261018-184912`. O relay oficial manteve o mesmo SHA.
O diagnóstico .14 encontrou a causa: o URL inicial real é `tauri://localhost`,
com path vazio, e a allowlist só aceitava `/` ou uma página HTML. A regra foi
corrigida com um teste específico deste URL. O diagnóstico regista apenas erros
limitados de arranque e URLs internos recusados, sem conteúdos de documentos ou voz.

O utilizador rejeitou o teste mochi.11: Super mostrou Codex imediatamente e o
nome Gravity não iniciou a conversa esperada. Não houve um novo trabalho Mochi
no journal nesse ensaio. Foi identificada uma associação visual indevida:
`view.pet ?? agent_codex` atribuía a escuta ainda sem dono ao pet Codex. O dispositivo predefinido foi confirmado como MOMENTUM 4 Bluetooth, suspenso
fora da escuta. A causa exata da transcrição do nome continua por validar; a UI antiga não a deixava clara.

Mochi.12 instalada remove esse fallback, mostra a chamada junto do abrigo,
conserva «Ouvi» após um nome não reconhecido e permite escolher explicitamente um
pet sem reutilizar a conversa de outro. A escolha explícita pede nova fala; não
reenvia automaticamente um texto ambíguo. ASR multilingue mantém-se local.

O utilizador clarificou também que a gestão completa tem de abrir diretamente
no abrigo como um widget, sem depender da janela principal Coucou. A página Pets
instalada é apenas uma parte da fusão. A mochi.12 prepara um popout próprio,
ancorado à barra, com sessões, providers/quotas, modelos, tarefas, perfis e chamada
dos assistentes configurados. Conserva também a retoma de tarefa com uma pasta substituta. Clique no abrigo abre a gestão; botão direito conserva a chamada
dos residentes. A validação .12/.13 encontrou o painel criado mas invisível (alpha 0). A .14 acrescenta a primeira remapagem GTK; o painel de ensaio já foi observado visível. A aceitação do RPM final continua obrigatória.

## Verificação em curso — mochi.14

A navegação inicial usa `tauri://localhost` com caminho vazio. A regra .13
recusava esse endereço; a .14 aceita exclusivamente os documentos internos
conhecidos, incluindo a raiz vazia, e continua a rejeitar PDFs e páginas externas.
O arranque nativo .14 foi verificado com os seis pets configurados. O popout
precisava também da mesma primeira remapagem GTK usada na ilha e no habitat:
foi observado visível, com alpha 1 e conteúdo real, em captura privada revista.
A remoção do fecho por perda de foco, por si só, não resolvia essa falha.

A revisão .14 em desenvolvimento passa a guardar estado público por
`(pet, backend, sessionId)`. Mensagens, pasta, título, passos, diffs e temporizador
pertencem à sessão, mantendo um pet por provider. Um pedido de aprovação captura
a origem e não consulta o último evento recebido. Uma segunda decisão que não
possa ocupar o cartão regressa ao terminal e fica identificada como tal. Os
registos vivos são reconciliados com o inventário após arranque; o widget permite
escolher qual acompanhar, distingue Terminal/Mochi e expõe o ID curto e pasta.
Acompanhar altera a apresentação, nunca o destino das mensagens escritas ou da
voz: estas continuam na conversa própria do Mochi.

A abertura de terminal tem uma ação `focus` própria: não faz retoma implícita.
O helper exige uma janela identificada de forma única quando várias partilham o
PID WezTerm. Backup da aplicação deste helper:
`repo-20261018-191214-yj9s1zev`. Há 505 testes JavaScript e 458 testes do
sysworkspace aprovados nesta revisão; os 554 testes nativos também passaram. Os ensaios de interface verificaram a origem imutável das aprovações, os controlos visíveis e a gestão. O ensaio de arrastar real, com rato reservado pelo utilizador, confirmou a posição exata mas não entregou o documento: esta parte não foi aceite. A .14 substitui a negociação de ficheiros WebKit/Wry por um destino GTK explícito COPY, com grant apenas após drop e URI local validada; O segundo e terceiro ensaios reais também falharam para PDF e pasta (erro GTK 5, genérico); não houve cópias na inbox. O diagnóstico seguinte substitui apenas os handlers DND internos WebKit que podem consumir o sinal antes do destino COPY; exige nova aceitação real.
A cópia de anexos saiu da thread da interface; cancelamento de cópia lenta e
limites temporais de todos os comandos do portal ainda precisam de revisão.

O runtime de ensaio é uma unidade transitória `coucou-pets-qa14.service` com
assets incorporados. O wrapper/autostart persistentes continuam na .12 enquanto
não houver RPM .14 aceite. Não confundir este ensaio com a instalação final.

Aceitação adicional da janela flutuante: a sessão Ollama real de ensaio voltou
a abrir com as duas mensagens públicas preservadas, verificadas por comparação
em memória sem publicar o conteúdo. A lista lateral, a identidade Mochi e o
modelo local aparecem na janela nativa. Um arrasto físico de +180/+190 píxeis
moveu a superfície exatamente +180/+190; hover posterior não a deslocou. Fechar
a janela não interrompeu tarefas. As capturas são privadas. O rascunho anterior
de 73 caracteres está preservado numa cópia privada e ainda precisa de ser
reposto no compositor correto, sem o enviar.

O primeiro arranque desta revisão ficou transparente e sem habitat/controlo;
o reinício recuperou as superfícies. A causa dessa ocorrência ainda não foi
confirmada: não considerar robustez de arranque aceite. O ensaio real também
confirmou que WebKitGTK perde `pointerleave` da ilha ao sair da área Wayland.
Foi acrescentado um observador nativo de foco do ponteiro, só enquanto expandida,
para iniciar a curva de recolha ao sair (intervalo 33 ms). O ensaio físico no
binário novo confirmou expansão e recolha concluída após sair, sem reservar
espaço nas janelas. O botão de um residente inativo foi corrigido para chamar o pet
na ilha principal, em vez de depender de já existir no habitat. Drop no cabeçalho
original volta a abrir o percurso de documentos desse pet.

Checkpoint atual: 522 testes JS e 586 testes nativos aprovados (10 ignorados).
Os ensaios isolados verificam a UI original, sidebar, alvos exatos, aprovações,
consulta de arquivos e ausência de movimento por hover. Isto não substitui a
aceitação Linux de documentos, voz, todos os pets e continuidade de terminais.

Copilot externo: o adapter encaminha apenas o par `agent_copilot/copilot`,
exclui sessões da biblioteca Mochi e usa a extensão na própria sessão TUI.
O ensaio isolado do dono confirmou envio e histórico público na mesma sessão;
não é um executor ACP novo. Observadores são limitados a 16, expiram após
90 segundos sem consulta e reconectam sem repetir mutações. Modelo e aprovações
permanecem no terminal; cancelamento exato não é anunciado. O RPM Linux passa
a incluir o runtime da extensão, e o loader global está preparado em sysworkspace,
**ainda não aplicado**. A extensão requer as funcionalidades experimentais
nativas; sessões já abertas precisam de recarga/reabertura explícita do dono.
Não executar `/clear` nem alterar políticas para forçar essa recarga. A ponte
externa Gravity continua indisponível enquanto não existir prova do endpoint
nativo herdado; conversa Mochi própria Gravity é um percurso separado.

## Organização da barra — pedido confirmado em 18/10

O utilizador confirmou que «UI nativa» significa o aspeto original da ilha
**Coucou**, e não o antigo widget IAStatus do DMS. O centro de controlo dos pets
volta ao centro superior: superfície preta com os cantos e Mochis animados
originais, estados reais e acesso direto às sessões/providers/modelos/tarefas.
Os pets móveis e os balões continuam como estão; as configurações já existentes
mantêm o seu percurso. A ilha de horas/música/meteorologia do sistema passa para
a direita. A eventual fusão de horas e Coucou fica para uma fase posterior.

Clarificação adicional do utilizador: recuperar também o encolher/expandir e a
animação da ilha original, sem uma barra fixa equivalente. Em desenvolvimento:
`IslandStateMachine`, `Tracked`/spring e curva de fecho originais, com forma
compacta de 288×32, superfície expandida para a gestão e área transparente sem
input fora da forma visível. A expansão sobrepõe as janelas; não reserva espaço
nem altera o tiling. A área de input GTK acompanha a animação. Fecho/Esc recolhe,
hover e temporizadores seguem a ilha original; decisões nativas continuam no
cartão próprio. O centro tem de oferecer todas as funcionalidades já feitas.

A revisão atual reutiliza os componentes originais `buildHeader` e
`buildOverview`, com os dois cartões e Mochi animado na vista de 640×198.
O ícone de definições e o seletor de sessões dão acesso à gestão completa
760×700 dentro da mesma ilha. O cabeçalho recebe contagens reais por pet e a
vista observa o estado público da ilha principal, sem duplicar os controladores
de agentes ou aprovações. Ensaio visual isolado passou; falta validar a nova
apresentação no runtime Linux e o comportamento com as decisões reais.

Clarificações posteriores: a ilha central recolhe logo quando o rato sai da
sua forma visível, iniciando de imediato a animação de fecho. Quando está ativa,
o widget antigo da casinha não aparece nem reserva espaço; os pets continuam
acessíveis na ilha. Estes dois ajustes estão no código e aguardam ensaio nativo.

A implementação em curso separa `pet-control` (apresentação pública e ações
explícitas de abertura) da ilha de conversa/aprovação. A barra não cria uma
segunda instância dos agentes nem toma decisões, não usa um provider implícito
e não movimenta os cartões de aprovação quando o utilizador percorre os pets.
O habitat reserva apenas a forma compacta central; a própria ilha incorpora a
gestão na expansão. O popout separado mantém-se como fallback quando a barra
está desativada.
A geometria da ilha DMS fica versionada em sysworkspace. Verificar barra real,
seis identidades configuradas, mudança de estado, clique/gestão, horas à direita,
sem colisões com satellites e comportamento em ecrãs de dimensão diferente.
As horas já foram deslocadas à direita. A primeira revisão animada foi observada
no Linux e não mudou a geometria das janelas. A revisão posterior que reutiliza
os cartões originais ainda precisa de instalação e aceitação real.

## Revisão integral solicitada em 18/10 — critérios de entrega

Esta secção prevalece sobre contagens e descrições históricas abaixo. O objetivo
continua aberto: não foi declarada uma entrega completa nem uma garantia geral
baseada apenas em testes simulados. Cada requisito tem de distinguir código,
instalação e validação no ambiente real.

| Pedido do utilizador | Estado real / trabalho obrigatório |
| --- | --- |
| Um pet por conta/provider ou entidade útil | Em desenvolvimento na mochi.13: Gravity concentra todos os modelos; Gemini/Claude deixam de ser clones Gravity. IDs e histórico antigos preservados. Abrigo e perfis limitados às integrações explicitamente selecionadas na app. Rever todos os caminhos de chamada e modelo. |
| Email apenas no Ollama por defeito | Default corrigido no código da mochi.13; restantes pets desativados. Falta aplicar e verificar o rascunho real no Betterbird. |
| Plugins e acessos por pet | Em desenvolvimento: documentos, rascunhos, música e abrir apps, com validação nativa. São concessões das integrações Mochi; não constituem uma sandbox das ferramentas CLI. Permissões nativas Codex/Gravity continuam obrigatórias. Falta uma política completa por app para ferramentas nativas, sem prometer isolamento que não existe. |
| Entrega de documentos e pastas | Bug real: WebKit navegou para o PDF dentro de companions.html, apagando os pets. Recuperação temporária por restart só quando não há trabalhos próprios ativos. Mochi.13 acrescenta veto de navegação, drop nativo com dono exato, cópias privadas e ações por capacidade. Pastas: snapshot limitado a 256 elementos, 12 níveis e 100 MiB, sem symlinks; rejeição explícita quando não suportadas. Email de pasta ainda não implementado; não deve ser oferecido. |
| Análise efetiva de anexos | Código mochi.13 entrega o caminho ao CLI; modelos locais recebem texto/PDF extraído localmente ou erro honesto. Rever PDF com texto, PDF digitalizado/protegido, TXT/CSV, pastas, tipo incompatível, limites, cancelamento, dois pets e aprovações simultâneas. Documentos sintéticos, sem enviar o PDF privado do utilizador. |
| Barra central original Coucou | Pedido confirmado: interface da ilha original no centro, pets atuais preservados, horas/DMS à direita. Superfície separada em implementação; falta aceitação visual e persistente. |
| Fusão TOTAL do centro AI legacy | Página e bridge já existem; a fusão ainda não está aceite. Comparar sistematicamente IAStatus e ia-barra com o abrigo: sessões, subagentes, providers, quotas/frescura, filtros, detalhes, abrir/retomar, configurações, arquivo, interrupção, modelos/digests, memória/RAM/VRAM/disco, retenção, serviço local e alertas. Validar cada operação no popout, sem depender da janela principal. |
| Múltiplas sessões do mesmo provider | P0 acrescentado pelo utilizador: com dois terminais Codex, o pet não identifica a origem de mensagens/decisões. Guardar estado por sessão, mostrar título/pasta/origem em cada fala e cartão, listar sessões ativas no próprio pet e permitir abrir a escolhida. Uma sessão não pode substituir o estado de outra. A conversa Mochi é um canal próprio e explícito. Validar com dois Codex simultâneos e repetir nos restantes providers. |
| Chat expandido por sessão | P0: Expandir no cabeçalho e compositor, janela flutuante com o mesmo chat, transparência/tamanho, seleção/pesquisa de conversas Mochi e terminais, resposta ao ID selecionado e nova sessão. UI e guardas passaram testes isolados. Codex externo tem ligação ao daemon com histórico público paginado e eventos em direto, ensaiada numa sessão sintética própria. Falta aceitação integrada Linux, adapters externos Gravity/Copilot e continuidade após fechar o terminal. A vista antiga de metadados foi rejeitada e substituída no código. |
| Sessões em segundo plano sem terminal | P0 novo: transferência explícita para gestão pelo pet; confirmar continuidade real antes de fechar o terminal. Implementar por capacidade do provider, mantendo execução, contexto, ferramentas e aprovações. Não matar o terminal e apresentar uma retoma como se fosse a mesma execução. Sessões sem suporte devem explicar o limite e continuar no terminal. Acompanhar estado não autoriza enviar texto para uma sessão externa. |
| Conversas completas por pet | Biblioteca v2 com IDs, arquivo antes de Novo chat, seleção/restauro, leitura sem alterar a conversa ativa, pesquisa e persistência local/Ollama implementada e testada. O limite herdado de 80 mensagens não satisfaz histórico completo: revisão v3 em desenvolvimento separa arquivo público completo e contexto limitado do modelo. Falta aceitação integrada e migração preservando conversas reais. |
| Voz fluida e PT/EN real | Super neutra implementada na .12; último teste real de .11 falhou. Repetir Gravity/Codex/Spotify/Ollama com Bluetooth real, observar «Ouvi», isolamento, aprovações, latência e Raquel. Sem ativar o microfone automaticamente pelo agente de desenvolvimento. |
| Execução e controlo das apps | Ferramentas nativas e pet-desktop implementadas; ainda falta matriz real de todas as entidades configuradas, Spotify fechado/seleção de faixa, tarefa GUI verificável e aprovação Gravity humana. Não confundir um controlo enviado com o efeito confirmado. |
| Arranque, autonomia e recuperação | Falta ensaio após login/reinício, tarefa controlada interrompida, recuperação explícita e disponibilidade no abrigo sem app principal aberta. |
| Entrega reproduzível | Testes e inspeção do artefacto final, instalação com rollback, arquitetura atualizada e commits assinados de projeto + sysworkspace continuam obrigatórios no fim. Nenhum commit desta fase foi feito. |

## Expansão e gestão de sessões — pedido adicional de 18/10

O utilizador confirmou uma resposta Ollama bem gerada e boa interface compacta.
Pediu expansão para ler e responder melhor, flutuante e parcialmente transparente,
para todas as sessões, incluindo as criadas em terminal. Pediu também fechar o
terminal mantendo a sessão em segundo plano sob gestão do pet.

A janela expandida precisa de receber um identificador de sessão completo,
independente do provider/foco global, e apresentar origem, título, pasta, modelo,
estado e decisões dessa sessão. A memória e o cursor de envio devem pertencer à
mesma sessão. A janela compacta e a expandida são vistas do mesmo canal, evitando
mensagens duplicadas e respostas dirigidas ao último pet selecionado. Cada envio
nativo deve validar o destino e o estado/ocupação da sessão. A escolha de uma
sessão externa pelo utilizador é explícita; a voz neutra nunca a deve assumir.

A transferência para segundo plano exige um supervisor ou uma API nativa que
mantenha a execução após fechar a interface do terminal. Primeiro verificar a
capacidade real de Codex daemon, Gravity e Copilot ACP; para processos ligados a
PTY usar mecanismo de detach revisto. Antes da aceitação, ensaiar uma sessão
sintética: iniciar em terminal, transferir, confirmar execução/decisões, fechar
só o terminal e continuar a mesma sessão pelo pet. Nunca testar este percurso
fechando os terminais do utilizador em trabalho. Não substituir continuidade por
replay automático, nem contornar políticas nativas de ferramentas/permissões.

A conversa Ollama testada enquanto o ensaio gráfico usa dados privados temporários
precisa de ser preservada ao sair desse ensaio. Não declarar a biblioteca local
pronta: a persistência de múltiplas conversas continua por implementar.

## Prioridade de sessões na janela flutuante — clarificação do utilizador

A janela maior tem de ser o ponto de interação com **todas** as sessões de cada
pet com chat: origem Mochi ou terminal, seleção por título/ID/pasta, conversa
completa em tempo real, resposta na sessão exata e criação de sessões novas.
Uma expansão iniciada no painel de gestão tem de abrir essa mesma UI de chat.
O utilizador rejeitou a vista que só apresenta metadados/atividade e não mostra
a conversa corrente do terminal. Essa vista não satisfaz o pedido nem é aceite.

Alvo explícito e imutável: `(pet, backend, origem, sessionId/conversationId)` em
leitura, stream, envio, cancelamento e aprovação. Nenhum fallback para a conversa
Mochi atual do provider ao abrir uma sessão externa. A seleção visual não muda
o destino de um envio já em curso; mensagens tardias ficam no respetivo alvo.
O frontend reutiliza os componentes nativos de chat, modelos, mensagens e ações.
O backend deve consultar/interagir com a sessão nativa por APIs reais, manter os
pedidos humanos de permissão e distinguir sessão ocupada, espera e ligação perdida.
Não apresentar uma cópia ou replay noutra sessão como se fosse continuidade.
Criar sessão nova é uma ação explícita com identidade própria e biblioteca.

Implementação em falta: adaptadores de sessões externas e histórico/live por
capacidade (Codex daemon, Gravity, Copilot ACP), selector unificado e leases por
alvo. O fecho de terminal e continuidade supervisionada continuam a exigir ensaio
sintético próprio. A sessão adicional será coordenada para estes adaptadores após
o módulo ACP; a sessão principal mantém UI, DND, aceitação e integração Linux.

## Janela flutuante e biblioteca em desenvolvimento — mochi.14

A expansão deve estar também na zona de mensagem/compositor de todos os pets,
não apenas no cabeçalho. O botão da conversa abre a mesma conversa Mochi. A
janela flutuante reutiliza o componente de chat e o seu aspeto, sem criar uma
interface paralela. Este acesso foi acrescentado ao código; falta aceitação real.

O utilizador observou a janela Ollama anterior ainda aberta durante o QA e
rejeitou as diferenças de cor e o arrasto instável. A revisão atual preserva a
paleta/cartão/balões/compositor original e apresenta sessões numa lateral
recolhível com pesquisa e nova sessão. «Todas as sessões» no compositor compacto
abre a mesma lateral. O arrasto deixou de somar deltas relativos da janela:
enquanto o botão esquerdo está premido no cabeçalho, consulta a posição absoluta
do compositor Hyprland e calcula a deslocação a partir do ponto inicial. Hover,
cancelamento, blur e perda de captura não enviam reposicionamentos. A posição
é preservada entre aberturas e a remapagem GTK não volta a mostrar uma janela
entretanto fechada. Ensaio isolado passou; falta confirmar a revisão nativa.

A janela `pet-chat` é separada do painel e do cartão de decisões. A conversa
Mochi reutiliza `buildPrompt` e a biblioteca nativa, com identidade da conversa
no envio; não cria um segundo agente ou contexto. Tem superfície translúcida
ajustável (55–100%, texto opaco), deslocação pelo cabeçalho e dois tamanhos.
No Linux usa layer-shell sem zona exclusiva, fora do tiling. Fechar só oculta
a janela, sem cancelar a execução. A aceitação nativa ainda está por fazer.

O utilizador rejeitou a vista de atividade/metadados como substituto da conversa.
O novo seletor da janela flutuante junta conversas Mochi e sessões de terminal,
com origem/ID, pesquisa, seleção explícita e nova sessão. Usa o mesmo construtor,
bolhas Markdown e compositor do chat dos pets. O painel de gestão abre esta mesma
janela através de «Abrir conversa». O seletor e controlador estão no código;
ligação nativa às sessões externas e aceitação real continuam P0.

O adapter Codex externo já está registado no código, via WebSocket Unix do
daemon da própria conta. Pagina todo o histórico público, mantém os IDs das
threads/subagentes e só envia quando `canAcceptDirectInput` é explicitamente
verdadeiro. Não reconstitui uma conversa num executor diferente nem escreve num
terminal. O ensaio sintético confirmou envio, resposta final e três amostras de
streaming no mesmo ID. Sessões temporárias cujo daemon não disponibiliza
histórico dão uma limitação explícita. Ao ligar a meio de uma mensagem, o trecho
inicial pode só aparecer quando a mensagem conclui; a UI explica esse caso.
Reconexões têm geração própria, invalidam aprovações antigas e limpam a cache;
escritas que ficaram na fila após timeout/cancelamento são descartadas antes de
iniciar o envio. Falta aceitação integrada no runtime e adapters equivalentes
para Gravity/Copilot externos.

O controlador fixa `(petId, provider, origin, sessionId, conversationId?)` em cada
leitura/envio/interrupção. Leituras atrasadas e erros de outra seleção são
descartados, notificações são agrupadas sem atrasar indefinidamente o streaming,
e sessões sem capacidade de escrita não enviam para uma conversa alternativa.
A biblioteca ganhou leitura de arquivo sem restauro: consultar não altera a
conversa ativa nem interrompe uma tarefa; responder restaura o ID selecionado,
com verificação de ocupação e nova verificação do ID ao reservar o envio.

As aprovações nativas têm snapshot dos pedidos pendentes e um destino imutável,
para abrir o chat durante uma decisão e responder uma única vez. A janela
flutuante filtra por sessão; o broker confirma o destino recebido. A ativação
Copilot via ACP foi integrada no código após catálogo/retoma/troca de modelo
sintéticos reais, mantendo permissões humanas. Falta aceitação no runtime da app.
Continuidade em segundo plano após fechar um terminal externo permanece P0.

A biblioteca v3 migra os ficheiros v1/v2 e guarda IDs, arquivos e mensagens
públicas completas. O limite de 80 mensagens/256 KiB é apenas o contexto enviado
ao modelo; a UI e o arquivo não cortam por esse limite. A biblioteca admite
128 MiB e 100 arquivos por pet, com escrita atómica, modos privados e erro
explícito. Uma falha de gravação mantém a resposta completa em RAM e mostra um
aviso; a execução já concluída não é reportada como pedido não enviado. Novo
envio/reset/restauro tenta primeiro guardar o estado pendente, sem o descartar.
Mensagens que versões antigas já descartaram não podem ser recuperadas por esta
migração. Testes de 90 mensagens longas, migração com 122 mensagens e falha/retoma
de gravação passaram, além do streaming partilhado. Falta aceitação no runtime.

## Trabalho em equipa e pesquisa opcional

A sessão adicional convocada pelo utilizador assume exclusivamente biblioteca
multiconversa, persistência local/Ollama texto e voz, com lease partilhado,
restauro/arquivo e testes. A sessão coordenadora mantém barra, documentos,
metadados de sessões externas, aceitação, instalação e integração da estação.
O documento de arquitetura tem um único escritor para evitar estados contraditórios.
Commits assinados apenas no fecho; nenhuma publicação está autorizada.

PostgreSQL é uma extensão opcional para pesquisa e memória partilhada por
permissões, não uma dependência da conversa. Antes de qualquer implementação,
prever BD e role próprios, isolamento por pet/conversa, consentimento de partilha
e recuperação local. Não consultar nem reutilizar bases de outros projetos.
O serviço existente não foi alterado e esta integração não está implementada.

## Componentes e propriedade

| Componente | Responsabilidade | Fontes principais |
| --- | --- | --- |
| Habitat e abrigo | Movimento, expressões, balões, recolha e chamada dos pets | `windows/src/mochi/`, `windows/src/desktop/companions.ts` |
| Ilha e cartões | Conversa, contexto, seleção de modelo e decisões nativas | `windows/src/island/`, `windows/src/views/chat.ts` |
| Gestão dos pets | Tarefas, sessões externas, subagentes, quotas, modelos locais e preferências | `windows/src/hub/pages/pets.ts` |
| Identidades e modelos | IDs estáveis, nomes, aliases, modo e dois perfis por pet | `windows/src-tauri/pets.default.json`, `src/pet_config.rs` |
| Execução e memória | Um trabalho por pet, contexto próprio, progresso público e recuperação explícita | `chat.rs`, `cli_chat.rs`, `assistant_memory.rs`, `assistant_tasks.rs` |
| Contas nativas | Codex app-server e Gravity stream-json com as suas ferramentas e políticas | `codex_chat/`, `gravity_chat/`, `gravity_hook.rs` |
| Voz | Captura, transcrição local, seleção do pet, controlos locais e reprodução | `speech/`, `voice/` |
| Estação | Arranque, Super, geometria da barra, ferramentas gráficas e inventário | repositório `sysworkspace` |

Os projetos e ambientes Python/modelos permanecem no projeto. O sysworkspace
versiona apenas a integração persistente do anfitrião. Credenciais, sessões,
conversas, capturas, áudio e caches privados não entram no Git. O fork de outra
sessão e o checkout canónico permanecem preservados.

## Da fala à resposta

1. Super sozinha inicia a escuta explícita. A interface distingue ligação ao
   microfone, escuta, transcrição e receção. Super novamente fecha áudio e
   reprodução; não cancela trabalho CLI já autorizado.
2. Whisper Large v3 Turbo transcreve localmente com deteção de idioma e suporte
   multilingue. O texto não é traduzido deliberadamente para português. Não se
   fornecem nomes dos pets como hotwords, evitando enviesar «olá» para «Ollama».
3. O nome ou alias seleciona uma identidade estável. O pedido é mostrado em
   «Ouvi». Comandos de estado e interrupção pertencem apenas ao pet selecionado.
4. O perfil escolhe modelo e esforço. Codex e Gravity usam a
   conta CLI respetiva; os templates de serviços configurados usam sessões Codex próprias e
   identificadas como tal. Ollama mantém o backend local.
5. O pet anima o estado e apresenta apenas mensagens públicas. Aprovações e
   perguntas mantêm os cartões e as decisões nativas. A aprovação de uma sessão
   externa não deve ocultar que uma fala Mochi foi recebida.
6. A resposta final chega em texto e é reproduzida pela Raquel. O microfone
   reabre depois, até terminar a conversa ou decorrer o período de inatividade.

Raquel usa o serviço Read Aloud através de edge-tts, sem chave API paga. Requer
rede e envia o texto falado à Microsoft. Não representa uma API de ChatGPT Voice
ou Gemini Live incluída na subscrição. ASR e áudio não são guardados por defeito.

## Ficheiro de identidades e modelos

O ficheiro ativo é `~/.config/coucou/pets.json`. Os defaults distribuídos estão
em `windows/src-tauri/pets.default.json`; a estação também guarda uma cópia
revista em sysworkspace. A interface está em Gestão dos pets, Nomes e perfis.

| Campo | Efeito |
| --- | --- |
| `pets.<id>.name` | Nome apresentado; o ID e o histórico mantêm a identidade |
| `aliases` | Variantes de nomes reconhecidas; aprendizagem guarda texto, não biometria |
| `provider` | Conta/família implementada; não permite trocar credenciais ou permissões |
| `mode` | `auto`, `fast` ou `reasoning` |
| `fast.model`, `fast.effort` | Modelo e esforço do perfil rápido |
| `reasoning.model`, `reasoning.effort` | Modelo e esforço do perfil de raciocínio |
| `reasoningTerms`, `reasoningLength` | Heurística local, configurável, de escolha automática |

| Pet ou grupo | Rápido | Raciocínio |
| --- | --- | --- |
| Codex | gpt-6-luna, low | gpt-6.1-sol, high |
| Gravity | gemini-3.8-flash-low | claude-opus-4-6-thinking |
| Templates de serviços via Codex | gpt-6-luna, low | gpt-6.1-sol, high |
| Ollama | gemma3:4b | qwen3:8b |

Voz usa o perfil do pet. No chat, «Perfil definido para o pet» segue o ficheiro;
«Automático», «Rápido» e «Raciocínio elevado» são escolhas explícitas. Um modelo
real selecionado manualmente prevalece. Gemini e Claude da conta Gravity são modelos do mesmo pet Gravity; os IDs antigos
ficam apenas para preservar dados/monitorização dos clientes independentes, sem
criar novos clones de conversa Gravity. «Usar este perfil no chat» repõe apenas
a preferência desse pet. «Modo rápido …» e «modo raciocínio …» permitem forçar o
modo no pedido. Para modelos Codex, o esforço é enviado em cada turno; não se
altera o esforço global do CLI do utilizador.

O modo automático é uma heurística por termos e tamanho, não um classificador
semântico. No Ollama de voz, o rápido envia `think:false`; raciocínio pede
`think:true` com orçamento maior. O controlo depende das capacidades do modelo,
conforme a [API Ollama](https://docs.ollama.com/capabilities/thinking).
Falta expor mais opções de geração/limites e aferir a escolha automática com
pedidos reais. Alterar o ficheiro durante um pedido não muda um trabalho em curso.

## Matriz dos pets

| Pets | Capacidade implementada em mochi.11 | Validação ainda necessária |
| --- | --- | --- |
| Codex | Chat/voz, ferramentas CLI, tarefas próprias, memória pública e decisões | Repetir no artefacto final os perfis rápido/elevado e decisões |
| Gravity | Um único pet, chat/voz e catálogo completo de modelos da conta | Aceitação da consolidação .13, troca de modelos e aprovação humana de shell pelo Mochi |
| Ollama | Chat/voz local, perfis, ações limitadas e gestão de modelos | Fala real mista e qualidade de cada perfil; libertação pela UI verificada com Qwen 0.6B |
| Spotify | Chat/voz próprios via Codex; iniciar app, play, pause, next, previous, volume, pesquisa e URI | App fechada, sequência de voz e seleção de música confirmada no player |
| GitHub, Notion, n8n, Vercel, Resend, Cal.com, Stripe | Conversa/voz próprias via Codex e ferramentas nativas disponíveis à conta | Percurso escrito/falado e tarefa verificável por integração autenticada |
| Copilot, Cursor, Muse, OpenCode, Amp, Hermes | Monitorização e cartões dos clientes/hooks implementados | Rever cada cliente instalado; não existe ponte universal de chat/voz |
| Anthropic, OpenAI, Google AI | Chat opcional com chave própria | Fora do percurso sem API paga; nunca usados como fallback |
| LM Studio | Chat no servidor local configurado | Verificar servidor real; sem execução geral nem voz integrada |
| Claude Desktop, Apple Music | Plataformas específicas | Não oferecidos como assistentes Linux |

Os pets de serviços têm IDs, conversas, tarefas e permissões separados do pet
Codex. A identidade visual do Spotify não transforma o modelo num modelo Spotify:
a assistência geral usa a conta Codex, com a origem indicada. Os controlos rápidos
MPRIS não precisam de uma inferência. «Pesquisar» abre uma pesquisa; não prova que
uma música começou. Links Spotify só são anunciados como reproduzidos após
confirmação do player. Ferramentas de conta exigem que a integração esteja ligada.

## O que está implementado e o que falta

| Área | Implementado | Por implementar ou rever | Prioridade |
| --- | --- | --- | --- |
| Acesso contínuo | Autostart, recuperação de falha, reconciliação de clientes, ilha e chamada dos seis pets configurados | Ensaio após novo login/reinício e com monitores diferentes | P1 |
| Estado e UX | Expressões, animações, balões acoplados, receção visível, decisões separadas | Rever o percurso instalado de todos os pets, navegação e acessibilidade | P0 |
| Modelos | Ficheiro validado, revisão concorrente, perfis, nomes completos e esforço visível | Confirmar legibilidade no artefacto instalado; acrescentar pesquisa de modelos | P0 |
| Conversa atual | Memória pública por pet, chat escrito/voz partilhados, biblioteca v3 e sidebar de sessões | Aceitar na app todos os owners, sessões nativas e trocas durante eventos concorrentes | P0 |
| Histórico | Journal v3 de texto público completo, arquivos locais, pesquisa/títulos, leitura sem restauro e Novo com arquivo | Migrar o journal instalado preservando o Codex real e a conversa Ollama de QA; aceitar retenção e restauro na app | P0 |
| Gestão AI | Página dos pets com sessões/subagentes, quotas, detalhes, modelos, retenção e ações | Página instalada e IAStatus desativado; popout .14 visível em ensaio nativo; falta aceitação do RPM e alertas de quota no pet | P0 |
| Ollama | Inventário, aliases por digest, RAM/VRAM/disco, load/unload com confirmação e reserva exclusiva durante manutenção | Aceitar visualmente o bloqueio de manutenção; gestão persistente de thinking por modelo e limites | P1 |
| Voz PT/EN | Deteção e transcrição multilingue sem hotwords | Teste Bluetooth real com alternância de línguas, nomes técnicos, pontuação e ruído | P0 |
| Voz de saída | Raquel aceite; reprodução começa por streaming; desligar áudio isolado do trabalho | Medir fim do modelo até primeiro som; seleção de voz por idioma/pet e tratamento de falhas | P1 |
| Spotify | Abertura e controlos determinísticos, pesquisa, URI e assistência Codex própria | Confirmar app fechada e música efetivamente selecionada; pesquisa autenticada de catálogo, quando disponível | P0 |
| Execução no PC | Ferramentas CLI nativas e pet-desktop com janela exata e acessibilidade | Tarefas reais em apps representativas; limites de apps sem acessibilidade; aprovação Gravity | P0 |
| Recuperação | Estado interrompido após restart, retoma explícita, pasta substituta | Ensaio de reinício durante tarefa controlada e confirmação de efeitos antes da retoma | P1 |
| Entrega | RPM por prefixo, relay oficial, planos e rollback | Migração ativa verificada; aceitação completa e commits assinados pendentes | P0 |

O cartão normal de aprovação é preservado. Não se introduz execução universal
sem decisões: «poder fazer tudo» significa acesso às ferramentas disponíveis com
as políticas nativas, às aplicações autenticadas e ao estado real verificável.

## Plano de aceitação

Para cada pet com chat/voz implementados, verificar: chamada pelo nome e alias;
transcrição mista; confirmação de receção; modelo/conta e esforço efetivos; resposta
em texto/voz; uma tarefa apropriada à sua função; estados de trabalho/aprovação;
interrupção do áudio sem cancelar trabalho; consulta da conversa; isolamento de
outras sessões; comportamento após recolha e novo arranque. Registar resultado
individual, versão e condições. Para monitores, testar o cliente/hook e mostrar a
capacidade real em vez de oferecer um chat não implementado.

A validação já observada inclui 498 testes JavaScript, 549 testes Rust da app,
34 do relay e 456 do sysworkspace; dez testes de plataforma foram ignorados.
TypeScript/Vite e ensaios de browser cobrem os perfis, revisão concorrente,
progresso/decisões separados, limites do painel, sessões com ancestrais, quotas e
confirmação de memória com alvo exato. O teste adicional de thinking local passou
após correção da chamada de teste. Estes números são checkpoints; consultar os
logs do artefacto final antes da entrega.

Codex e Gemini via Gravity leram canários aleatórios pela conta nativa. Um
processo Codex retomou um contexto próprio depois de reiniciar o transporte;
Luna low respondeu ao ensaio curto em 4,8 s. O Gemma local produziu JSON válido
no ensaio sintético, incluindo cold start em 8,3 s. Os ensaios de ASR reconheceram
PT e EN separados; a frase mista ainda juntou/trocou palavras. A confirmação real
da mochi.10 pelo utilizador não valida automaticamente a mochi.11. Na aplicação
mochi.11, o Qwen 0.6B foi carregado sem prompt e libertado pela confirmação da
gestão dos pets; `/api/ps` confirmou zero modelos residentes no fim. Nenhuma
conversa de terminal foi fechada no ensaio.

## Auditoria do legacy e desenho pendente

A comparação do QML IAStatus e do helper ia-barra confirma que os quatro grupos
de filtros de sessões e modelos, árvore de subagentes, modelos por digest,
retenção, ações de configuração, abrir/retomar, arquivo/interrupção e
load/unload têm representação na página Pets. A ação legacy `chat` é substituída
pela conversa do pet Ollama com o modelo selecionado. Ainda não basta: falta
validar a seleção atómica desse modelo, a exibição nativa do popout, detalhes de
clientes por modelo, alertas permanentes de quota (o legacy avisava a partir de
85% ou bloqueio), estado desconhecido/stale e a proteção de pedidos locais
ativos contra unload/stop. A lista e a libertação de um modelo de ensaio foram exercitadas;
operações que interrompem sessões reais não devem ser executadas como teste
sem um alvo de ensaio próprio e uma confirmação específica.

A biblioteca v3 já tem IDs próprios, pesquisa, arquivo antes de Novo chat,
restauro explícito protegido por trabalho ativo e migração de formatos v1/v2.
O journal de tarefas é independente. Guarda apenas texto público; a conversa
Ollama falada partilha a memória persistente e o bloqueio do chat escrito.
A integração está coberta por regressões, mas a migração dos dados privados
instalados e a aceitação gráfica de todos os owners continuam pendentes.

Copilot selecionado já tem conversa própria pelo ACP nativo, catálogo, esforço,
ferramentas e aprovações, mantendo as políticas da CLI e sem allow-all. A ponte
externa usa uma extensão na sessão TUI existente: envio/histórico passaram num
alvo próprio isolado, o runtime está preparado no RPM, mas o loader global e a
aceitação gráfica não estão concluídos. Modelos/aprovações externas continuam
no terminal. Não atribuir tarefas Copilot a outro provider.

A cópia de documentos já decorre fora do IPC, com token nativo, cancelamento,
ordenação de entregas e validade por conversa. Falta aceitar todo o percurso
real, permitir seleção de partes de uma pasta e completar a política de retenção
dos grants de ficheiros independentes do contexto do chat. A interface não deve
limpar o anexo ou conversa anterior quando uma nova entrega é rejeitada. O
rascunho de email deve preservar campos por pet e nunca enviar automaticamente.

## Próximas entregas

1. Concluir a aceitação da revisão .14 em QA, ainda não instalada como default. A escolha de modelo está legível
   e a consulta de histórico dos serviços foi corrigida, com cobertura dos oito
   owners; falta testar os percursos reais de todos os pets.
2. Aceitar a biblioteca v3, o restauro e o envio exato em cada pet, migrando os
   dados privados instalados sem misturar sessões externas nem substituir tarefas.
3. Executar a matriz real de todos os pets disponíveis, incluindo Spotify
   fechado e aprovação Gravity, e corrigir os resultados encontrados.
4. Afinar a transcrição mista com a fala real e medir a latência da voz.
5. Atualizar este guia e o registo da estação; concluir com commits assinados,
   mantendo o relay, hooks e assinatura ativos.

## Última verificação nativa de documentos e sessão

O quarto ensaio real do PDF sintético continuou a falhar (GTK DRAG_FAILED 5),
mesmo com habitat em overlay e o pet correto confirmado. Não foi aceite. O
destino foi transferido do widget WebKit para a sua janela GTK, mantendo a
máscara de input e o grant de caminho exclusivamente após um drop completo.
Esta alternativa está compilada e aguarda novo ensaio.

O runtime de QA passou a usar `/run/user/1000` com uma única instância, mantendo
`XDG_DATA_HOME` privado para preservar a biblioteca instalada. O runtime isolado
anterior interferia com a descoberta PipeWire e excedia o comprimento Unix dos
sockets Copilot; não certificava áudio nem descoberta de sessões nativas. O
wrapper/autostart persistentes continuam na .12, sem nova instalação aprovada.

### Resultado real e problemas adicionais encontrados (22:00)

O PDF e a pasta sintéticos concluíram o drop nativo no Ollama (`DRAG_ENDED`,
sem `DRAG_FAILED`) e os ficheiros foram confirmados na caixa de entrada privada.
O ensaio passou a incluir pequenos movimentos finais sobre o pet, em vez de
terminar exatamente na primeira entrada na superfície: o código Hyprland 0.56.2
[envia inicialmente o centro da superfície no evento enter](https://github.com/hyprwm/Hyprland/blob/v0.56.2/src/protocols/core/DataDevice.cpp#L654);
o movimento seguinte fornece as coordenadas reais. A rapidez de um drop sem
novo movimento ainda precisa de revisão; não atribuir a falha exclusivamente
a permissões do plugin. O destino GTK permanece na janela que contém WebKit.

A inspeção do percurso completo encontrou lacunas adicionais: o chat compacto
só enviava o anexo na primeira mensagem, e o flutuante enviava sempre contexto
vazio. A correção em curso guarda contexto de documento por provider/conversa,
invalida cópias anteriores após Novo/restauro e permite consulta/consumo pelas
duas superfícies. Também foram encontrados e estão em correção: admissão queued
cancelável antes de executar ferramentas, Parar com turno exato, abertura A/B
concorrente do flutuante, e recuperação visível de texto após entrega desconhecida
sem repor nem reenviar automaticamente o pedido. Estas correções exigem testes
próprios e nova aceitação integrada.

A opção de email foi observada no Ollama, mas o ensaio de preencher o formulário
parou porque o controlo WebKit anunciava edição sem fornecer EditableText. Não
foi criado nem enviado email. O formulário também não pedia foco de teclado na
camada Linux; isso está corrigido nas fontes, ainda por testar no binário novo.
O helper `pet-desktop` agora explica a ausência dessa interface em JSON em vez
de produzir traceback (16 testes; backup `repo-20261018-220637-mx7gccjw`).

Foi detetado e revertido outro efeito do ensaio: o plugin de autostart escrevia o
executável debug na entrada de login. A fonte e o ficheiro ativo voltaram à .12
(backup `repo-20261018-215242-_nc1x8b7`). A versão debug Linux passa a não
reescrever o autostart no arranque; mudanças explícitas de preferências mantêm o
seu percurso. Esta proteção está compilada, pendente do próximo reinício QA.

### Reconhecimento multilingue e escolha explícita (18-10, revisão seguinte)

O utilizador descarregou os modelos pela UI e ativou o ditado. Confirmados
Parakeet TDT 0.6B v3 int8 e os dois ficheiros de diarização completos; as
preferências mantêm ditado ativo, limpeza por API desativada, resumo desligado
e saída Raquel. O modelo de diarização distingue intervenientes, não sintetiza
a voz dos pets. Não foram alteradas estas escolhas nem os modelos instalados.

A fonte passa a expor `voiceAsrEngine` (`auto`, `whisper`, `parakeet`) e o estado
real do reconhecedor no painel. Automático preserva o Whisper já configurado;
caso contrário usa o Parakeet. Ditado, Super e conversa usam o mesmo motor e
a mesma verificação de disponibilidade. Uma seleção explícita sem modelo
não muda silenciosamente para outro motor. Alternar preserva os caminhos do
Whisper e os ficheiros descarregados. Estas alterações estão em compilação,
ainda não certificadas na instância gráfica.

Comparação local de quatro áudios sintéticos PT-PT/EN/mistos: ambos reconhecem
as duas línguas, mas ambos erram partes das frases mistas. Whisper GPU demorou
0,55–1,25 s por frase (modelo carregado); o ensaio Parakeet demorou 2,98–3,38 s
incluindo arranque, carga e diarização, pelo que os tempos não comparam apenas
a inferência. A qualidade em fala real com Bluetooth continua pendente.
Não foram usados microfone, API paga nem sessões LLM neste ensaio.

Integração de documentos reforçada nas fontes: token opaco nasce no drop
nativo e captura as conversas nesse instante; reset/restauro/cancelamento
revogam a entrega exata e não afetam uma entrega mais recente. O anexo é
visível e pode ser retirado no chat flutuante; acompanha também mensagens
a meio da conversa compacta e pedidos por voz. É revalidado antes de iniciar
as ferramentas nativas, incluindo depois de esperar por modelos/fila.
14 regressões Store e o getter leve de ID passaram no target isolado; a
aceitação gráfica desta integração depende da nova compilação completa.

### Checkpoint de validação

617 testes nativos passaram (10 ensaios de hardware/modelos ignorados),
525 testes JS passaram, TypeScript e Vite concluíram. Ensaios browser da ilha
original e chat flutuante passaram: sidebar, arquivo sem mudar conversa ativa,
alvos de envio, histórico público externo, eventos ao vivo, decisões com origem,
transparência, arrasto e ausência de movimento por hover. Compilação debug nova
em curso; não confundir estes resultados com aceitação da app instalada.
A manutenção Ollama passa a reservar o mesmo canal de trabalho do chat/voz
antes da fila, e a reserva só é libertada após terminar o helper.

### Matriz gráfica e anfitrião após reinício QA

A nova instância debug iniciou e manteve o autostart persistente na .12; a
proteção contra registar binário debug ficou verificada. Nenhum rascunho novo
por enviar foi encontrado antes deste reinício. A cópia privada de preferências
QA tinha a voz de sistema de fábrica e resumo Claude; passou a respeitar Raquel
e resumo desligado já escolhidos na instalação, mantendo ditado/microfone/tecla
e modelos descarregados pelo utilizador. Não se alterou a preferência instalada.

Os seis pets (Ollama, Codex, Gravity, Copilot, GitHub e Spotify) abriram
o chat compacto com provider/modelo correto e expandiram para a conversa com
sidebar e Nova sessão; fechar o flutuante funcionou nos seis, sem enviar
pedidos nem cancelar trabalhos. É aceitação de navegação real, não uma
certificação de ferramentas/inferência de todos os providers.
O sysworkspace passa 461 testes e o checker de 283 ficheiros.

### Anexos e edição de email na UI real

PDF e pasta voltaram a completar o drop após integrar os tokens nativos.
O PDF apresentou as opções Analisar/conversar, Preparar email e Cancelar no
Ollama. Foi corrigida nas fontes uma corrida entre a chamada pendente do pet
e a entrega do ficheiro: a entrega passa a substituir a apresentação anterior
com a vista de ações do documento. O chat compacto ganha Ações do documento
para recuperar essas opções depois de fechar o cartão, preservando decisões
pendentes.

O ensaio de preencher email por acessibilidade foi interrompido: o campo
continuava a anunciar foco, mas o cartão fechara pelo temporizador e o teclado
passava para o browser. Nenhum rascunho foi criado ou email enviado. Corrigidos
nas fontes o foco ao reabrir a mesma vista e a retenção do cartão enquanto
um input tiver foco real; esta retenção é independente de pins/aprovações e
é libertada quando o utilizador trabalha noutra aplicação. A ilha central
continua com o fecho imediato ao sair o rato. Faltam repetir o ensaio nativo
com estas fontes e confirmar o anexo no Betterbird sem enviar mensagem.

O anexo PDF também foi observado no chat flutuante real; Retirar anexo eliminou
o contexto canónico e o botão desapareceu sem enviar qualquer pedido.
As fontes acrescentam pesquisa de modelos por nome/ID, teclado e estado vazio;
os testes de picker (11) e chat existentes (28) passaram isoladamente.
O rascunho original de 73 caracteres continua guardado privadamente e não foi
reenviado; a reposição no compositor correto fica para o último reinício.

### Revisão integrada após quota e edição

626 testes Rust passam (10 ignorados), 558 JS passam, TypeScript/Vite e ambos
os ensaios browser passam. O relay oficial mantém SHA-256
`92bf5488ce4ff290d4573ceca24ad3d3e63520d1c16cd30b22073bffe14b8b67`.
A fonte de quota lê apenas o cache local sanitizado e usa planos já recebidos;
não acrescenta pedidos de rede. Sinaliza >=85%, só anuncia bloqueio explícito,
distingue informação desconhecida/desatualizada, deduplica por conta e preserva
a prioridade de decisões. O pet apresenta um indicador persistente de aviso
ou bloqueio e a utilização no tooltip acessível, sem substituir a animação
de trabalho. A frescura das fontes de quota continua dependente do produtor
existente ou de atualização manual; não inventar dados Copilot/Gravity.

### Resultado do ensaio de escrita e causa da interrupção

Com o binário integrado, os dois campos sintéticos do email foram escritos e
confirmados por acessibilidade. O rascunho continuou sem aparecer; a inspeção
mostrou o provider Codex no cartão que fora aberto para o Ollama. O percurso de
aprovação muda o foco e `State.endApproval` só restaura o anterior no modo
singleton. O botão de email consulta o provider global e faz no-op se o novo
owner não tiver ficheiro. A sessão frontend assume a correção de propriedade
e retorno de foco; não foi criado nem enviado email. A hipótese anterior de
falha exclusivamente GTK não explica este desvio e não deve orientar mudanças
nativas de teclado sem nova prova.

Auditoria de paridade: as funcionalidades legacy têm implementação nas fontes,
incluindo pesquisa, quotas e manutenção exclusiva Ollama. Permanecem P0 de
implementação a seleção atómica do modelo Ollama (backend ACK em execução por
Coordena, UI pela frontend) e a continuidade/ligação de terminais onde o CLI
nativo a suporte. Gravity normal 1.4.2 embute o LS sem token/endpoint herdado;
ler argv não resolve o caso ensaiado e a ponte externa mantém-se indisponível.
O percurso Mochi próprio continua separado. Copilot externo depende da TUI;
fechar a TUI encerra a ponte e não equivale a uma transferência em background.

### Persistência e admissão do modelo — backend em teste

`save_settings` e a gravação da posição do pet passam a serializar alteração,
ficheiro e publicação com o mesmo mutex. A RAM só muda após gravação válida;
uma falha devolve erro em vez de confirmar uma preferência perdida no restart.
O modelo Ollama é atualizado apenas por `pet_model_select` com ACK/generation;
snapshots completos antigos deixam de o repor. O comando foi registado no
backend e valida catálogo/perfis sem executar ou carregar um modelo.

A admissão de chat escrito passou a ocorrer antes de copiar as preferências
efetivas; a voz também relê a seleção depois de reservar o turno. Assim uma
escolha confirmada não é seguida por um envio que ainda usa o snapshot
anterior. A escolha explícita por sessão mantém o âmbito dessa sessão.
Os testes de rollback/ordenação de persistência e as novas regressões de
seleção estão a compilar; a integração frontend/ACK ainda requer aceitação.

A integração de seleção/persistência passou 641 testes nativos (10 ignorados),
incluindo 13 casos do comando e dois de rollback/ordenação das preferências.
Uma salvaguarda adicional, ainda por testar no próximo conjunto, rejeita
`selected_model` Ollama que contradiga a última escolha confirmada, evitando
que um picker antigo volte a sobrepor o ACK. Outros providers conservam
a escolha explícita por sessão. A frontend reportou correções de retorno de
foco/vista após aprovação, captura do owner do email e falhas de gravação
visíveis; a aceitação nativa aguarda os seus assets finais.

644 testes nativos passam com a salvaguarda do picker desatualizado; os assets
frontend recebidos passaram 566 JS/TypeScript e ensaios browser de UX. O novo
binário debug está em compilação para a aceitação visual da frontend.
Lacuna adicional identificada no painel de voz: os campos de aprendizagem de
nomes ainda usam cinco providers fixos e omitem Copilot/GitHub/Spotify, apesar
de o backend aceitar esses owners. A frontend deve derivar os campos dos pets
configurados, com os nomes atuais; não mostrar providers desativados.

A frontend confirmou nativamente perfis e abertura da sessão Ollama/histórico
corretos no novo flutuante. Encontrou sobreposição com o cartão compacto,
corrigida nas fontes para recolher o cartão do mesmo pet após ACK da janela,
com preservação de aprovação pendente. Também conserva o pet inicialmente
escolhido se a carga das preferências falhar e o utilizador tentar Recarregar.
Os assets final2 estão a ser incorporados no debug; só houve mudanças frontend,
pelo que os 644 testes nativos anteriores não são repetidos neste build.

</details>
