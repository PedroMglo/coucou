# Rotina com notas de projeto fixadas — 17/11/2026

Implementação integrada nas fontes1343/Source4 e validada nativamente em perfil
privado, no binário `fd04c0d69488328088d1948e5eb2f6c7800225416b4340cc0b139c85c77f8c0a`.
A aplicação instalada44 e o RPM49 congelado continuam preservados. A unidade não encerra AUTO04 nem os49 critérios integrais.

A rotina local Ollama reutiliza os mecanismos existentes de regras, scheduler,
Authority, capturas, orçamento por ocorrência, contexto, Driver e resultados.
O utilizador consulta a conversa pública de origem, escolhe projeto/notas e fixa
as revisões no rascunho. A revisão nativa apresenta notas completas, proveniência,
projeto, modelo/destino, horários e orçamento. A recorrência e os dados exigem
caixas de consentimento distintas, inicialmente desmarcadas.

O consentimento permite repetir apenas as mesmas revisões. Cada ocorrência tem
uma captura, uma resposta local e zero ferramentas, dentro da alocação original.
O contexto conserva o CID original e não copia o histórico da conversa. Alterar
projeto, associação, disponibilidade, nota, expiração ou destino bloqueia novos
usos. A atualização passa por edição/revisão/sucessão existentes. Uma revisão de
dados manual ou de objetivo único não autoriza esta recorrência. Notas são dados
não fiáveis, nunca política ou autorização de operações.

A camada de domínio de inferência compõe esta funcionalidade; os pets de serviços
continuam com os controlos que representam. Não se generaliza chat para n8n ou
outros serviços. Avisos/quantidade de calendários não se aplicam a estas notas.

O registo único de captura existente conserva os nomes JSON `calendarRead` e
`calendarReadId`, com hash/domínio de política próprios para projeto. Não foi
criado outro ledger ou worker. Book3, RulePolicy6, RuleGrant11, referência3,
AdmissionSpec31, GoalGrant12 e Record68 distinguem a semântica nova e impedem
leitores antigos de a confundir com autoridade sem dados ou de calendário.
Selection2 e o contexto de memória2 existentes mantêm-se; regras antigas não são
reescritas nem recebem `projectData:null`.

A aceitação delimitada usa a compilação privada final Source4/build2 e o caso
nativo5. Passaram seleção de conversa/projeto/notas, conteúdo completo na revisão,
consentimentos próprios inicialmente desmarcados, duas ocorrências automáticas,
respostas persistidas com critérios ainda por verificar, sucessão6, alteração de
nota bloqueada pela autoridade antiga e revisão7 apenas do saldo restante.
A terceira chamada usou a nota revista; revogar a regra durante o HTTP recusou
a resposta tardia. O objetivo fechou, conservando uma chamada/zero ferramentas,
resultado desconhecido e reserva de65536B. O reinício manteve os bytes do runtime
e não repetiu inferência. As três chamadas do caso aceito foram HTTP sintético.
Não foi necessário confirmar um fecho retido nesse caso; o Driver concluiu-o.

Os dois horários civis são reais: o primeiro já estava atrasado, ainda dentro do
prazo original de24h explicitamente revisto, com política coalesce; o seguinte
venceu no minuto real posterior. O relógio do anfitrião não foi alterado. Isto
verifica a mecânica de duas ocorrências e não certifica utilização durante dias.

Passaram54 testes Rust dirigidos (projeto, Book, autoridade, sucessão/revisão),
105 web dirigidos e TypeScript. O teste de projeto inclui proprietário opaco
original, revogação, Stop/drenagem, recurso novamente disponível e restauro
histórico sem renovar autoridade. Book3 recusa uma representação de projeto como
Book2 e não é reduzido quando se retiram os dados. O restauro desta composição
continua limitado a fixtures; não foi executada uma nova manutenção nativa.

Os casos anteriores ficaram preservados e separados:1 parou numa consulta Busy
antes de inferência;2 passou duas ocorrências/respostas, depois o observador não
voltou a selecionar Ollama ao reabrir a vista;3 passou sucessão/revisão e parou
num fecho retido, sem causa isolada nessa execução;4 conservou o débito incerto,
mas o observador podia aceitar uma listagem ainda sem o terceiro objetivo e
consultar demasiado cedo a resposta. O caso5 espera pelo ID exato desse objetivo,
tolera Busy só nas consultas e não transforma essas falhas em aceitações.
Houve11 chamadas sintéticas no total das tentativas nativas2–5, três no caso final.

Três filtros Rust iniciais corresponderam a zero testes e foram excluídos da
prova; os filtros corrigidos executaram os25 testes de autoridade e14 de
sucessão/revisão incluídos nos54 acima. Os erros de compilação/fixture anteriores
permanecem nos recibos privados, sem recertificação retroativa.

As imagens atuais observadas mostram as duas respostas e os critérios, e uma
vista parcial após reinício. A revisão tem dados completos no DOM nativo; as
capturas de revisão não mostram todo o conteúdo numa só região. Não se declara
aceitação gráfica global, humana, instalada ou de hardware.

[Recibo, fontes, delta, testes, tentativas e preservação](../.build/project-rule-20261117/VERIFIED.json).
[Percurso nativo final](../.build/upstream-parity-20261111/project-rule5/RESULT.json).

Continuam dependências separadas: observação automática dos objetivos/resultados
do projeto, proposta de decisões, notificações aplicáveis ao domínio, utilização
prolongada, restauro desta nova composição e entrega instalada autorizada. Uma
resposta persistida não constitui verificação de critérios ou aceitação humana.
