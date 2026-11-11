# Aproveitamento do AI Local nos pets

Avaliação inicial por leitura de fontes em 21/10/2026, autorizada expressamente
pelo utilizador. O AI Local é tratado como descontinuado. A consulta não altera
os seus repositórios, inicia serviços, contacta contas ou migra dados. O objetivo
continua a ser concluir os pets, com backend prioritário. Desde 24/10, commits
assinados e atualização do remoto estão autorizados após validação.

Prioridade revista em 23/10/2026: arquitetura primeiro e desenvolvimento dos pets
a seguir. A [arquitetura atual](PETS-ARCHITECTURE.md) e o
[plano com dependências](PETS-ROADMAP.md) integram os candidatos deste relatório;
os relatos de versões abaixo são evidência histórica. O sistema AI Local
continua descontinuado e não é uma dependência obrigatória dos pets.

Este relatório está no checkout `coucou-codex`, branch
`feat/mochi-cli-accounts`, em `docs/AI-LOCAL-REUSE.md`. A composição validada e
a documentação são agora versionadas nessa branch; o checkout `coucou` na branch
`fix/linux-desktop-voice` continua separado.

## Resultado

**Há trabalho aproveitável**, sobretudo contratos, casos de teste e partes
isoladas. A arquitetura completa de 19 repositórios e os seus serviços têm um
custo de integração que não acelera automaticamente uma aplicação desktop.
Reaproveitar uma peça exige confrontá-la com o que os pets já implementam.

### Quarto incremento — backend fechado, candidata .31 empacotada

O catálogo local de fontes ICS está implementado e adotado no checkout de
desenvolvimento. O fecho privado tem 600 fontes imutáveis, 1566 testes Rust sem
falhas/12 ignorados, 20 testes específicos do catálogo e cargo check aprovados.
A inscrição exige consentimento explícito distinto de entregar um anexo;
seleção, permissões e revogação estão vinculadas ao pet/conversa/turno originais.
O armazenamento tem revisões, gravação atómica e exclusão de escritores
concorrentes; os adaptadores mantêm documentos e imagens independentes.

Recibo: `.build/personal-source-catalog-next-r1/FINAL-PERSONAL-SOURCES.json`.
A candidata `.31` passou as nove verificações de composição. O RPM foi construído
e verificado (`db9a934f4008…`), com as 600 fontes inalteradas, sem instalação.
A aplicação ativa continua na `.30`. Não foi inscrita uma fonte pessoal real,
consultada uma conta ou usada inferência neste incremento. A gestão visual e
aceitação nativa do consentimento continuam pendentes, assim como RSS/IMAP,
proatividade e eventual PostgreSQL. Os repositórios AI Local não foram alterados.

### Terceiro incremento integrado na .30 — 22/10/2026

A unidade privada de voz/calendário foi fechada com 597 fontes imutáveis,
1541 testes Rust sem falhas, 24 testes de regressão ICS e 12 de datas/fusos.
`calendar_query_day` calcula o dia completo no backend com a mesma base IANA
do leitor ICS, incluindo mudanças de hora; conserva a consulta por intervalo.
O contrato está ligado aos adaptadores de API, Codex, helper CLI, Ollama Ask e
voz. As cinco ações de voz e as aprovações nativas mantêm-se. O novo percurso
de voz usa os controlos de raciocínio exatos anunciados pelo modelo Ollama.

Dois ensaios com o modelo local gratuito `qwen3:0.6b` consultaram o dia completo
pedido e devolveram o evento sintético desconhecido pelo modelo, com três
pedidos de inferência por ensaio e nenhuma ação. Isto valida o backend com uma
transcrição escrita; microfone/TTS e consultas reais em CLIs/cloud continuam
sem aceitação neste incremento. Uma falha transitória num teste existente de
limpeza de processos de rede passou nas repetições isolada e completa, sem
alterar o código de rede.

Recibo: `.build/calendar-voice-next-r1/FINAL-CALENDAR-VOICE-DAY.json`.
A `.30` está instalada e ativa, processo `4091547`, RPM `94febbfe61d1…`.
As nove verificações da composição passaram e as 597 fontes ficaram iguais
durante os 337,03 s de construção. Definições, 25 perfis e histórico mantiveram
os bytes. Os quatro rascunhos foram capturados antes da troca; a leitura após
fechar/reabrir confirmou Ollama (76 caracteres) e Codex (vazio). Gravity/Copilot
ainda aguardam essa leitura nativa: o painel recolheu e o ponteiro estava em
movimento. A aceitação visual parcial não foi registada como completa.
Catálogo persistente de fontes, IMAP/RSS, proatividade e PostgreSQL continuam
por implementar. Os repositórios AI Local permanecem intactos.

### Segundo incremento integrado na .29 — 22/10/2026

A janela já é consultável nos percursos de chat com ferramentas e ICS autorizado:
`calendar_query` aceita instantes/fuso explícitos, até 128 eventos e oito consultas
por turno, com resposta e cobertura limitadas. A revisão do ficheiro, pet, CID,
turno e autorização originais mantêm-se durante a consulta. Codex usa ferramenta
dinâmica; Gravity/Copilot usam helper temporário através da execução e permissões
nativas. API/local usam tool calls, com o modelo escolhido; o caminho Ollama Ask
conserva as aprovações dos comandos. O OpenAI segue o contrato de continuação
sem estado de [Responses](https://developers.openai.com/api/docs/guides/migrate-to-responses).

A .29 passou 1512 testes Rust e as oito verificações de composição, sem modelos
ou quota. A entrada nativa do helper com endpoint inválido foi verificada sem GUI.
Na entrega .29, consulta real com modelo/helper, voz normal Ollama, catálogo de
fontes e proatividade ainda exigiam trabalho. Os adaptadores de API de fornecedores sem
pet/anexo configurável não criam por si uma nova via de autorização. PostgreSQL
continua sem integração; o Stack AI Local não foi iniciado ou modificado.

### Primeiro incremento integrado na .27

O normalizador ICS com recorrências, exceções e fusos horários e o adaptador
limitado ao documento entregue ao pet/CID/turno estão no código instalado da
`.27`. As dependências seguem no executável, sem instalação global. Passaram
24 testes sintéticos com o pacote isolado e a composição completa com 1468
Rust, incluindo cancelamento dos consumidores e compatibilidade do arranque DMS.
Os caminhos comuns contemplam pets CLI, Ollama/LM Studio e fornecedores de API.
A aceitação com calendário real e inferência ainda não foi feita.

O âmbito inicial é um export selecionado, até 32 eventos nos próximos 14 dias,
com proveniência e cobertura explícitas. Nessa primeira fase, fontes persistentes, janela consultável,
IMAP/RSS e proatividade continuavam por implementar. Não foram consultadas contas
ou calendários reais e PostgreSQL continua sem integração nesta unidade.

| Componente revisto | Benefício para os pets | Decisão inicial |
| --- | --- | --- |
| Personal Context | Calendário ICS, leitura IMAP e RSS para contexto do dia a dia | Candidato funcional prioritário; exige fontes explicitamente escolhidas, permissões por pet, proveniência e limites. Os adaptadores atuais precisam de correção antes de integrar. |
| Reasoning Loop | Estados de tarefa, orçamento, prazo, falta de progresso e terminação tipada | Reutilizar desenho e fixtures ao completar tarefas em background. Preservar dono/CID, cancelamento, recibos e aprovações existentes. Uma confiança do modelo não confirma execução. |
| Document Processing | OCR, extração e documentos que o extrator atual dos pets não cobre | Integração opcional futura, após identificar uma lacuna concreta. O serviço completo depende de PostgreSQL e de outros contratos; Vision acrescenta um servidor/modelo. |
| RAG | Pesquisa com proveniência em documentos e memória de longa duração | Útil quando houver uma necessidade de pesquisa que o arquivo local não satisfaça. Avaliar primeiro fontes, escopo por pet e recuperação; não substituir o journal. |
| Execution Policy | Avaliação determinística de comandos e motivos de decisão | Aproveitar casos de teste e explicações de risco. Não substituir a autorização nativa nem transformar uma classificação em permissão de execução. |
| Workspace Execution | Workspaces descartáveis e execução isolada | Desenho útil para um executor opcional. A implementação existente exige runner autenticado, imagens, PostgreSQL/storage e lifecycle próprios; não é uma ligação imediata ao executor atual. |
| Resource Governor | Admissão, leases e telemetria para recursos locais | Usar os contratos como referência para concorrência/modelos. Evitar introduzir todo o plano de controlo para resolver um limite local pequeno. |

## Constatações do código

- `personal_context/calendar.py` usa `cfg.calendar.ics_paths`, enquanto
  `CalendarSettings` e o carregador atuais expõem `paths`. Com o calendário
  habilitado, esse acesso fica fora do bloco de tratamento por ficheiro.
  Também lê o ficheiro completo e procura recursivamente sem um limite de
  ficheiros/bytes nesse adaptador. A implementação observada não expande RRULE.
- `personal_context/rss.py` usa XML protegido por `defusedxml`; a configuração
  aceita apenas URLs HTTPS sem credenciais. O cliente segue redirecionamentos e
  materializa a resposta antes de limitar os itens. Antes de integrar, são
  necessários limite de bytes/prazo total e revisão dos destinos de redirect.
- O adaptador IMAP seleciona pastas com `readonly=True` e limita o número de
  mensagens, mas faz `RFC822` completo. A fatia de 200 caracteres ocorre depois
  da leitura; não constitui limite de tamanho da mensagem. A configuração
  observada admite `imap_tls=False`.
- `reasoning_loop/policy.py` tem decisões explícitas para cancelamento,
  orçamento, prazo, repetição/oscilação e falta de evidência. É uma boa referência
  para o controlo da progressão. Não prova efeitos reais de ferramentas.
- RAG separa a base PostgreSQL/pgvector do runtime e declara fontes e schema
  próprios. Workspace Execution separa gestor, sandbox e autoridade de runner.
  São fronteiras úteis; os runtimes não foram ensaiados nesta avaliação.

Estas constatações são de leitura, não uma auditoria completa ou prova de
funcionamento atual. Não foram usados modelos, credenciais ou dados pessoais.

## Integração proposta

1. Fechar os incrementos em curso: proteção da lista nativa de pets e entrada
   explícita de tarefa local pela voz Ollama. A voz normal conserva música,
   gestão de pets, dados dos serviços, abrir aplicações e preparar email.
2. Definir uma unidade de contexto pessoal de leitura: fontes explícitas,
   habilitação por capacidade/pet, origem/data de cada resultado, ausência
   distinguida de lista vazia e limites antes de ler ficheiros/respostas.
   Reaproveitar os contratos e fixtures adequados do Personal Context; corrigir
   as incompatibilidades no componente adotado. Não inferir contas instaladas.
3. Avaliar recuperação documental/RAG quando o contrato de memória exigir
   pesquisa semântica. Manter histórico, identidade e recibos no dono atual.
4. Usar os contratos de progressão, recursos e sandbox ao desenvolver as
   respetivas lacunas dos pets, sem duplicar os mecanismos já implementados.

O PostgreSQL local **continua sem integração com os pets**. Pode tornar-se útil
para um índice RAG/pgvector ou dados estruturados partilhados entre capacidades.
Essa decisão requer schema e role próprios, migração/backup e uma necessidade
concreta; não justifica transportar o lifecycle inteiro do Stack para os pets.

### Primeira unidade candidata: contexto pessoal de leitura

Proposta original de backend. O ICS entregue ao chat já foi integrado nas
entregas anteriores; o catálogo persistente está no quarto incremento acima,
**ainda sem instalação**. Os requisitos seguintes mantêm-se para as restantes fontes:

- Um catálogo de fontes explicitamente inscritas. Cada fonte tem identidade,
  tipo, revisão, habilitação e pets autorizados. Ausência de configuração
  significa desativado, sem descoberta de calendários ou contas pessoais.
- Primeiro percurso com calendário ICS escolhido pelo utilizador; depois RSS
  e IMAP como capacidades distintas. Uma permissão de preparar rascunhos de
  email não concede leitura da caixa de correio.
- Pedido associado ao pet/CID e a IDs de fontes autorizados. Resposta com
  estado `disabled`, `ready`, `partial` ou `unavailable`, instante da consulta,
  origem dos itens e limites aplicados. Lista vazia confirmada distingue-se de
  erro, dependência em falta e origem que ainda não foi consultada.
- Leitura limitada antes de carregar o conteúdo: ficheiros, bytes, itens,
  paginação e prazo total. No ICS, recorrência/fuso não suportados produzem
  cobertura parcial explícita; não inventar ocorrências nem resultados completos.
- Um resultado só entra no contexto do modelo depois de revalidar o dono e a
  revisão da concessão. Dados do calendário, RSS ou email são dados externos,
  não instruções que concedem ferramentas, permissões ou novos destinos.
- Transporte e segredos têm configuração própria e escopo estreito. Consultas
  externas e tarefas proativas não são ativadas pela simples disponibilidade
  do código. Não há envio de email, eventos ou mensagens nessa unidade.
- Fixtures locais para fonte desativada, ficheiro excessivo, erro de parser,
  recorrência/fuso incompletos, concessão revogada e resultado tardio de outro
  pet/CID. Nenhuma conta real ou modelo é necessário nesses testes.

Esta unidade pode começar sem PostgreSQL. A adoção do serviço AI Local completo
ou de um adaptador menor deve ser decidida depois de fechar o contrato e medir
as dependências; não está decidida por esta avaliação inicial.

## Fontes locais e revisões

Workspace: `/home/pmglo/_projects/ai-local`. Foram lidos os contextos de ownership,
READMEs e os módulos citados. Não foram copiados segredos, configuração de
máquina, modelos ou dados de utilizador.

| Repositório | HEAD observado | Estado inicial |
| --- | --- | --- |
| ai-local-stack | `27451c27353c6e914d582805c46f06b138e2fcfb` | Alterações locais existentes; preservadas |
| ai-local-personal-context | `0f4728b9afbeb17f3194a7c04ff2755014f66fb3` | Limpo |
| ai-local-reasoning-loop | `fb7f15ae3f1fd575ff840ae01a9b9569368b2ded` | Limpo |
| ai-local-execution-policy | `8a53441b1d6a568b46fbf4c68d9e81375303b872` | Limpo |
| ai-local-document-processing | `50e254a3edbb06a23824f5692472d7ce3845250b` | Limpo |
| ai-local-rag | `011c084112cd15242ffc5565d67dcac9387d54f7` | Limpo |
| ai-local-resource-governor | `a183a059710e4d19bfd1ba0573c124bae0dea176` | Limpo |
| ai-local-workspace-execution | `0a20e7c018b05e66c1355e454353800a04e2c2f4` | Limpo |
