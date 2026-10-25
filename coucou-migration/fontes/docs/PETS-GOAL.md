# Objetivo persistente dos pets

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
