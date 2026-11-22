# Acompanhamento limitado de objetivos de projeto — 17/11/2026

Atualização posterior: o [pedido e resposta de decisão como dados do projeto](PETS-PROJECT-DECISION-2026-11-17.md)
tem prova própria Source3/build2/native2. Esta página conserva o âmbito da
observação Source17; a nova prova não a substitui nem concede execução de ações.

Entrega técnica delimitada integrada na árvore e validada na composição privada
Source17/build5/native9, binário `140364077f2c1cb8489768f69b0b53eb3e2d8907c28c5afb7d5f86c6a60078a3`.
Instalada44 e RPM49 preservados. Sem commits, pushes, merges ou substituição.
O objetivo integral49 e a aceitação humana/instalada permanecem abertos.

| Estado | Alcance comprovado |
|---|---|
| Desenhado e implementado | Contrato próprio, backend, controlo funcional e integração no runtime existente. |
| Fixtures | 71 Rust dirigidos,58 web e TypeScript; restauro69/32/12 conservador em fixtures. |
| Nativo privado | Tauri/WebKit/HTTP, respostas legíveis, duas ocorrências reais, sucessão/revisão, revogação e reinício. |
| Integrado | Fonte de desenvolvimento atual; artefacto privado próprio com hashes. |
| Entrega instalada/humana | Pendente; esta funcionalidade não está na instalada44 ou no RPM49 arquivado. |

Compor o runtime existente com observações de1..8 objetivos escolhidos do mesmo
pet/projeto, fixando GoalID+SpecHash. Cada ocorrência captura estado, critérios,
última admissão, contabilidade nullable/incertezas e, por escolha inicialmente
não selecionada, última resposta finita elegível. Projeto/CID original e notas
opcionais explicitamente selecionados; não copiar histórico de conversa nem
exigir notas artificiais. Respostas são dados, não conclusão ou autorização.

Âmbito atual: domínio de inferência local Ollama e gestão nativa. Não acrescenta
chat a serviços/n8n, não muda coordenação, modelo, conta, destino ou privilégios
do Guardian. O runtime comum mantém autoridade, scheduler, recursos, pedidos,
reserva, execução e recuperação; não existe outro índice/ledger/serviço.

Autoridade: consentimento novo boundedRuleProjectGoalProgressData distinto de
manual, um objetivo e notas fixadas; BoundedRuleRecurrenceWithProjectGoalProgress.
Política fixa as definições, projeto e opções; a revisão mostra a fotografia
inicial completa e consente futuras observações desses mesmos objetivos.
1 leitura/10s,1 modelo/0 ferramentas por ocorrência,128KiB máximo da fotografia e
256KiB no pedido; excesso recusa sem corte. As alocações/prazos permanecem os
originais em sucessão/revisão. Nenhum fallback de destino/modelo.

Proveniência: progresso normal não muda uma fotografia as-of. Mudança de projeto,
associação/definição/indisponibilidade bloqueia o âmbito. Uma resposta continua
a depender das notas/projeto da admissão ORIGINAL que a produziu, mesmo que o
objetivo tenha depois outra admissão. Dados revogados não são lavados por texto
derivado: captura nova indica sourceDataUnavailable; captura em uso mantém
revogação observada e rejeita resultado tardio. Respostas de classes transitivas,
pedidos recebidos/calendários/progresso composto exigem outro contrato de
encaminhamento; não são silenciosamente promovidas. Histórico/restauro conserva
dados sem autoridade ou saldo atuais.

Persistência: Book4, Policy7, RuleGrant12/HMAC12, Reference4, ContextBinding12,
Admission32, GoalDataGrant13/Preview14, Record69. O nome legado calendarRead
continua no registo único, com domínio hash específico; None omite o novo campo
para conservar bytes antigos. Isto não é NativeContext CLI3: a forma CLI2 antiga
não foi substituída nesta unidade.

Correções concretas encontradas na integração:
- Reentrância do catálogo/Authority ao validar fontes dentro de publicação:
  reutiliza Store::source_snapshot atómico e as provas/fences existentes. A
  captura nova continua a usar leitores nativos verificados e contabilidade.
- Mistura de relógios na revisão de resposta derivada: reabrir usa o mesmo now
  nativo do chamador. rust15 reproduziu a recusa; rust16/17 passaram sem enfraquecer
  memoryClockChanged ou refrescar observações/TTL.
- Verificação estrutural de horários calculava SHA do runtime1.5MiB e descartava
  esse valor. Conserva revisão capturada sem esse cálculo; uso atual ainda compara
  o resolvedor. Não é uma medição de paridade global de recursos.
- Preparação de metadados podia perder repetidamente a corrida com leitores:
  apenas Busy anterior à publicação tem espera≤500ms pelos mesmos pedidos
  determinísticos, antes de gastar leitura. Revogação/prazo/ambiguidade param;
  nenhuma inferência/leitura é repetida. Fixture de ambiguidade comprova o limite.

Provas dirigidas atuais: rust17 71 passados/3 ignorados preexistentes, entradas
nativas exatas. Incluem duas ocorrências assinadas, dados presentes/desconhecidos,
recusa de auto-observação/projeto/pet/SpecHash, isolamento/limites, revogação
persistente, restauro69/32/12 histórico sem execução, relógio e contenção.
58 web mais TypeScript: web10, com27 dos percursos alterados e tipos repetidos em
web12; os restantes ficheiros web não mudaram desde a prova reutilizada.
Build5 Source17/1348 entradas e829 nativas, binário privado14036407…; não é RPM.

Tentativas nativas1..8 preservadas e NÃO promovidas a aceitação integral:
1 sem prontidão,0 inferência;2 recusa de assinatura por reentrância;3/4 despacho
não observado (4 incluiu ptrace e é diagnóstico);5 captura explícita recusou
Busy;6 não completou primeira;7 capturou contexto mas recusou evidência pelo
relógio;8 passou duas ocorrências reais/as-of/sucessão6/revisão7 mas a terceira
parou Busy antes da leitura. Só providers HTTP sintéticos; nenhum Copilot/GitHub,
chamada paga, alteração instalada, assinatura/commit global ou consentimento
humano fabricado. Decisões de QA só no perfil gerado, pelas APIs nativas.

Native9 passou na composição atual, sem captura/admissão manuais suplementares:
- Uma resposta fonte HTTP sintética foi assinada pelo runtime. Seleção explícita
  de um objetivo próprio, com outro do mesmo projeto excluído, sem notas
  artificiais; as duas escolhas de recorrência/dados começam desmarcadas.
- Duas ocorrências civis efetivas: a primeira atrasada do dia anterior dentro do
  prazo24h/coalesce revisto; a segunda no minuto real do dia. O cancelamento do
  objetivo fonte entre elas entrou apenas na segunda; a primeira fotografia
  ficou igual. Não é prova de vários dias de uso.
- Os dois recibos foram consultados pela API nativa e pela UI, com os critérios
  ainda por verificar. As imagens X11 atuais RESPONSE0/1 foram observadas;
  mostram as respostas legíveis. A revisão inicial foi observada parcialmente
  no viewport; não se afirma aceite gráfico integral de todas as vistas.
- Sucessão6 com a política original e revisão7 carryRemaining deixaram só uma
  ocorrência no saldo vigente. A terceira usou o mesmo caminho e atingiu HTTP.
- Revogar a nota ORIGINAL da resposta fonte durante o HTTP bloqueou a resposta
  tardia. O objetivo parou pausado;1 chamada desconhecida e reserva65536B
  permaneceram na contabilidade. Captura nova devolveu sourceDataUnavailable,
  sem texto revogado. A regra foi depois revogada pelos controlos existentes.
- Reinício conservou os hashes de todos os ficheiros de runtime/grants/contextos,
  sem replay.4 chamadas sintéticas no cenário completo, zero reais/pagas.

Recibos privados: `.build/project-followup-20261117/VERIFIED.json`,
`rust17/RESULT.json`, `web10/RESULT.json`, `web12/RESULT.json`,
`app-build5/RESULT.json` e
`.build/upstream-parity-20261111/project-followup9/{RESULT,TWO-NATIVE-RESULTS,SUCCESSION6,REVISION7,SOURCE-REVOCATION,RESTART}.json`.
As capturas e perfis de QA permanecem privados. O Source18 de fecho regista a documentação e remove duas linhas vazias no EOF de
`pet_runtime/mod.rs`; o código e a composição executável validados no Source17
permanecem os mesmos. Não se repetiu inferência por essa alteração de formatação.

Pedido de decisão governado a partir deste relatório, uso
prolongado, restauro desta composição na app, pacote normal, entrega instalada,
aceitação humana e objetivo integral49 permanecem dependências próprias. A
instalada44 e RPM49 têm hashes preservados; autorização49 não foi inferida.
