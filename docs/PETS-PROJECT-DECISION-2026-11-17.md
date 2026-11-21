# Pedido de decisão como dados de projeto — 17/11/2026

Prova inicial integrada e validada na composição privada Source3/build2/
native2, binário `bbac7a16471dad0295e0635c571c50876ad0b7bca9de417b7c7edef149e4b4f6`.
Instalada44 e RPM49 permanecem intactos. Sem commits, pushes, merges, nova
instalação ou aceitação humana. O objetivo integral49 continua aberto.

| Estado | Prova e limite |
|---|---|
| Desenhado/implementado | Pergunta limitada, origem nativa, revisão própria e resposta numa nota do projeto. |
| Fixtures iniciais | 5 Rust dirigidos,79 web e TypeScript na fonte inicial; atualização abaixo. |
| Nativo privado | Tauri/WebKit/IPC/HTTP reais, pergunta/revisão/nota, recusas, uso revisto num novo objetivo e reinício. |
| Integrado | Fontes atuais e compilação privada própria; Source4 de fecho só altera documentação. |
| Instalado/humano | Pendente; não pertence à instalada44 nem ao RPM49 arquivado. |

## Contrato e domínio

A rotina de acompanhamento pode escolher explicitamente “Permitir um pedido de
decisão no acompanhamento”. A opção começa desmarcada e compõe um contrato de
formatação na intenção original revista/assinada. Editar conserva a opção e o
texto original. O backend exige esse contrato exato no objetivo autenticado;
uma resposta que apenas parece JSON não promove um objetivo antigo.

O relatório contém um resumo e, opcionalmente, uma pergunta com2..4 opções.
Resumo≤512 bytes, pergunta≤1024, cada opção≤80; relatório≤8192. O modelo pode
indicar que não é necessária uma decisão. Campos desconhecidos, formato
incompleto/malformado e excesso recusam a interpretação, conservando a resposta;
não há reparação, nova inferência ou fallback. Todos os textos são dados e são
renderizados como texto, incluindo a fixture com `<script>`.

Âmbito implementado: domínio local Ollama com Admission32/Context12 de projeto.
A leitura usa o recibo real do modelo, GoalID, DecisionID, hash, projeto/revisão e
datas originais da observação/resposta. Não inventa received_from ou uma origem
CLI. Serviços/n8n não recebem chat ou esta superfície. O broker de receitas em
execução continua a autorizar receitas; não é usado para chamar “aprovação” a
uma resposta informativa do projeto.

## Responder e preservar

A supervisão permite escolher uma opção, escrever outra resposta≤1024 bytes ou
recusar decidir. Não há opção preselecionada. Uma revisão nativa nova mostra
projeto, pergunta, resposta e nota integral. A gravação tem checkbox própria,
inicialmente desmarcada, e consentimento `saveProjectDecisionData`; consentimentos
de memória/inferência/recorrência/receita não o substituem. A revisão dura120s e
está ligada à janela nativa/documento de gestão. Fecho, navegação, cancelamento,
expiração, mudança de origem, revisão ou dados impedem a gravação.

Projeto/pergunta/resposta são legíveis. A nota integral e as referências ficam
recolhidas, acessíveis à revisão. A gravação e a leitura após reinício mostram
a resposta guardada; notas editadas são apresentadas como dados do utilizador.
A proveniência ExplicitUser é a semântica existente de gravação explícita de
memória, não certificação de presença humana ou autorização de execução.

Reutiliza Registry2 e as notas sensíveis de projeto existentes. O conteúdo da
nota guarda um objeto de dados com origem, relatório, resposta e data da revisão;
não acrescenta um formato do catálogo, GoalRecord, Grant, scheduler ou ledger.
O ID da nota é determinístico pela origem/recibo. Confirmação idêntica pode
reconhecer uma publicação incerta; outra resposta ou uma nota editada não é
sobrescrita. Eliminar deliberadamente a nota pelos controlos normais permite
registar de novo após nova revisão; não se criou um histórico paralelo de tombstones.

A pequena publicação mantém os leitores existentes de GoalStore/Authority e o
CAS da memória. Revalida o contexto, associação/projeto e notas originais que
sustentaram os dados derivados. Retirar a nota original durante a revisão impede
publicar a resposta; repor o seu texto com outra revisão não revive a captura.
Histórico restaurado é consultável, sem capacidade de registar nova resposta a
partir dessa origem. Esta última apresentação tem fixtures e contrato; o restauro
nativo não estava coberto pela prova inicial; ver a atualização abaixo.

Guardar não executa a opção, completa critérios, retoma um objetivo, renova saldo,
encaminha a nota ou a inclui automaticamente em contexto. O pedido é consultado
na supervisão existente; não se acrescentou transporte de perguntas por voz,
telefone ou notificações. As notificações de resultado já existentes conservam
o seu consentimento/limites; não recebem o conteúdo destas perguntas.

<a id="decisao-projeto-correspondencia-2111"></a>

## 21/11 — proposta de correspondência a partir de uma resposta

No RPM 59, após guardar a resposta como nota e selecioná-la num novo
objetivo do mesmo projeto, a interface bloqueava o planeamento. A
validação de admissão também exigia uma resposta própria sem ferramentas.
Isso deixava a correspondência planeada com memória fora deste percurso.
[Observação no RPM 59](../.build/upstream-parity-20261111/normal59-project-action1/RESULT.json).

Na fonte atual, um objetivo com notas de projeto continua com uma
resposta local como opção inicial. Se o utilizador escolher planear,
o único catálogo disponível é a correspondência de uma ronda com memória
selecionada. A revisão do novo objetivo vincula a nota e o orçamento.
O texto e cada destinatário têm outra revisão antes de qualquer envio.
A receção conserva o percurso de adoção pelo destinatário com a sua
própria revisão e os seus próprios limites.

A correspondência usa o recibo existente e inclui a impressão digital
do projeto, além das referências das notas. A conversa de origem
reabre a seleção. A sessão privada de execução identifica o efeito
no recibo. A revalidação usa a associação e a revisão exatas e
compara a impressão digital. O envelope omite os registos brutos e
o nome do projeto.
As referências sem projeto mantêm os bytes anteriores. Leitores
anteriores recusam a nova forma e o uso na instalação fica sujeito a
verificação da preservação e reversão. Os restantes catálogos continuam
fora do âmbito para notas de projeto.

O binário privado que inclui essa alteração passou o percurso de uma
resposta escolhida até um pedido ao pet Codex. A primeira revisão
autorizou os dados do novo objetivo e o planeamento local. Até à
segunda revisão, o destinatário continuou sem mensagens. O envio
registou um único recibo para o texto e destinatário revistos. O Codex
adotou o pedido como rascunho próprio, sem admissão de execução ou de
modelo. O reinício conservou os dois objetivos e o recibo, com zero
chamadas adicionais. Depois de alterar a revisão do projeto, a
confirmação pendente de outro uso e uma preparação nova foram recusadas.
[Ensaio nativo](../.build/upstream-parity-20261111/normal59-project-action7/RESULT.json).

O RPM 60 repetiu o percurso com o próprio executável. O ensaio de
restauro abriu o recibo em histórico após novo reinício. O rascunho
do destinatário mantém a sua própria admissão por fazer.
[Resultado no pacote](PETS-VALIDATION.md#rpm60-correspondencia-2111).

O primeiro binário chegou à proposta, mas a conversa pública de origem
não coincidia com a sessão privada do recibo. A comparação passou com
as duas referências distintas. Nas tentativas seguintes, consultas do
observador encontraram estados de atualização e `busy`. A tentativa
final aguardou as consultas sem repetir uma confirmação de envio.
Usaram-se perfis novos e três respostas sintéticas no percurso final.
Falta preparar pacote normal e validar a gestão dos dados novos numa
atualização e reversão. A versão operacional continua na 44.
[Recibo conjunto](../.build/project-decision-action-20261121-r2/VERIFIED.json).

Passaram os 39 testes Rust filtrados por projeto, os cinco testes de
estrutura da proveniência e a verificação TypeScript com quatro suites
web dirigidas. A suite Rust mais ampla parou numa fixture recursiva por
excesso de pilha. A fixture passou isoladamente no mesmo executável com
16 MiB de pilha na thread de teste. O restante dessa suite mantém o
estado de execução parcial.

## Prova na composição atual

Native2 usou perfil, HOME/XDG, X11, D-Bus e cofre privados, imagem local fixa e
providers HTTP sintéticos. IPC e assinaturas reais; nenhuma assinatura ou decisão
humana foi fabricada. Quatro chamadas sintéticas: uma resposta fonte, dois
relatórios independentes e uma nova proposta com a nota explicitamente escolhida.
Zero inferência real/paga, Copilot/GitHub, alteração instalada ou configuração
persistente do anfitrião.

- A seleção de projeto/objetivo, contrato de pergunta e consentimentos da rotina
  atravessaram a UI nativa. Notas da rotina vazias são válidas; o recibo fonte é
  autenticado e a nota bruta de origem não foi encaminhada.
- A pergunta veio do recibo guardado. Dono/revisão errados, campos de autoridade
  acrescentados, consentimento de outra classe e revisão cancelada recusaram.
- Escolha/revisão/checkbox própria→nota ordinária exata passaram. Uma segunda
  revisão com resposta diferente não a sobrescreveu. A gravação deixou os hashes
  do runtime e os saldos dos objetivos inalterados.
- A nota só foi enviada ao modelo depois de criar outro objetivo, selecionar a
  revisão exata dessa nota e confirmar a sua própria revisão BoundedGoalContext.
  A chamada seguinte continha a nota exata numa mensagem USER, excluindo a nota
  bruta original; uma resposta JSON nesse objetivo ordinário não o promoveu para
  um novo pedido de decisão. A proposta continua com critérios livres por verificar.
- No segundo relatório, revogar a nota original da resposta fonte durante a
  revisão bloqueou a gravação. Repor o conteúdo com revisão nova não a reviveu;
  nenhuma nota dessa resposta foi publicada.
- Reinício conservou notas e todos os ficheiros do runtime; não repetiu inferência
  ou gravação. A resposta guardada continuou legível e não apareceu como pendente.

As três imagens X11 atuais REVIEW/SAVED/RESTART foram observadas. Mostram projeto,
pergunta, resposta, consentimento desmarcado/botão bloqueado e resposta persistida,
com referências recolhidas. São recortes privados com scroll, não aceitação
humana, gráfica global, de hardware ou instalada.

Recibos: `.build/project-decision-20261117/{VERIFIED,PRESERVATION}.json`,
`rust4/RESULT.json`, `web4/RESULT.json`, `app-build2/RESULT.json` e
`.build/upstream-parity-20261111/project-decision2/{RESULT,NATIVE-QUESTION,SAVED-PROJECT-DECISION,REVIEWED-NOTE-REUSE,REVOKED-REVIEW,RESTART}.json`.

Diagnósticos preservados: erro inicial de sintaxe do observador não iniciou
processos; rust1 expôs erros de compilação e rust2 um ID de projeto inválido na
fixture, corrigido reutilizando a criação nativa. Rust3 passou a gravação.
Native1 passou o percurso anterior, mas as imagens mostraram a resposta escondida
entre referências técnicas. A apresentação foi corrigida e o percurso completo
foi repetido em native2 na composição atual; não se usa native1 como aceitação da
nova UI. Não se repetiu a suite integral de milhares de testes.

## Prazo e restauro da composição atual

Atualização posterior à prova inicial: Source2/build2 de
`.build/project-decision-recovery-20261117`, binário privado
`b321a7005a543073ef359277ace70345abd70ce9c63b2e146530a312ef45c668`.
A entrega inicial bbac7a16 acima mantém o seu âmbito; não é usada como aceitação
executável desta atualização. Source3 de fecho altera só documentação.

Confirmou-se uma falha concreta na revisão da resposta: o guarda anterior só
verificava a expiração UTC. Aceitava confirmação anterior à preparação e, com
UTC parado, depois dos120s monotónicos. Duas fixtures vermelhas reproduziram
publicação indevida na nota Registry2 real, incluindo depois de preparar o
ficheiro temporário. O guarda exige agora UTC desde a preparação e antes da
expiração, mais o prazo monotónico original; volta a verificar a validade antes
da publicação atómica. A revisão/frame/origem nativos continuam obrigatórios.
Não se alterou o relógio do anfitrião, nenhum formato persistido ou consentimento.
7 testes Rust dirigidos passaram. Os79 testes web/tipos anteriores são reutilizados
por hashes de todos os seus inputs e ausência de qualquer delta frontend.
O terceiro ficheiro Rust acrescenta apenas diagnóstico em builds de desenvolvimento.

O percurso nativo atual produziu seis respostas/tentativas sintéticas: resposta
fonte, dois relatórios, uso explicitamente revisto da nota num objetivo separado,
pergunta sem notas nem resposta fonte selecionada e uma chamada cuja associação
ao projeto foi retirada durante HTTP. A escolha e a recusa de decidir ficaram em
duas notas ordinárias; o relatório sem resposta humana permaneceu sem nota. A
última chamada conservou1 resultado desconhecido/65536B, sem publicar uma resposta.

O perfil exato atravessou22 comandos CLI de manutenção existentes: arquivo,
inspeção, restauro inativo, revisão/quarentena da autoridade, revisão de
configuração e ativação explícita de dados com grants[]. Repetiu-se o percurso
sobre o perfil já restaurado, conservando duas gerações de recuperação. Dez
assinaturas foram verificadas em cada revisão. Em ambas as gerações, os oito
objetivos, quatro contextos12, projeto/revisão, notas e recibos ficaram exatos.
Os formatos Registry2/Book4/Goal69/Admission32/Context12 foram preservados.

Os leitores e a UI Tauri/WebKit reais mostraram escolha, recusa e pergunta não
respondida como histórico restaurado. Nova gravação a partir dessa origem, token
antigo e alteração/admissão de um rascunho sem projeto foram recusados. O
rascunho de controlo tinha alcançado a revisão própria antes do arquivo e essa
revisão foi recusada de propósito; o seu bloqueio posterior não se explica por
estar pausado ou depender de um projeto revogado. As quatro regras ficaram
snapshotOnly, sem dispatch. Contabilidade global histórica continua incompleta;
o subtotal e a tentativa desconhecida são conservados, não certificados como
saldo de execução atual. Reinício dos dois perfis manteve os ficheiros protegidos
exatos e produziu zero HTTP/inferência/replay. Foram observadas imagens X11 de
escolha/recusa, pergunta não respondida e escolha após o segundo reinício.

A cadeia de prova distingue as tentativas: seed4 terminou os seis percursos,
mas o observador falhou depois ao tratar um campo opcional vazio. O recibo
falhado permanece intacto; finalize2 consultou/reiniciou o mesmo perfil e binário,
sem nova inferência. Só essa continuação e restored1/restored2 têm recibos0.
As primeiras tentativas encontraram recusa antes do envio; seed3 confirmou
metadados locais fora da validade15s. O ensaio independente atualizou metadados
explicitamente antes de autorizar, sem alargar esse prazo nem repetir um efeito.
A interrupção143 da manutenção1 ocorreu antes de escrever intenção/processo da
quarentena; os bytes exatos e ausência de dispatch foram verificados antes de
continuar só a fase ainda não iniciada. A primeira manutenção2 não iniciou a
app: o sandbox recusou o socket D-Bus. A tentativa ficou preservada;2b usou apenas
o D-Bus privado autorizado.

Permanece um diagnóstico de fiabilidade: numa tentativa separada seed5, a quarta
regra criou o objetivo sequência1, mas não avançou para tarefa/contexto/admissão
nos90s observados; nenhuma sexta chamada foi enviada. A causa ainda não está
estabelecida. Os restauros aceites não certificam esse percurso de preparação
nem uso prolongado. O próximo trabalho é reproduzir esse ponto com o erro de
dispatch observado e corrigir apenas a causa confirmada, antes de evoluir mais
capacidades. Sem novos ledgers, formatos ou inferência repetida.

Recibos privados: `.build/project-decision-recovery-20261117/{VERIFIED,PRESERVATION,WEB-REUSE,FOLLOWUP-NATIVE-SEED5}.json`,
`rust-red/RESULT.json`, `rust1/RESULT.json`, `app-build2/RESULT.json`,
`maintenance1/RESULT.json`, `maintenance2b/RESULT.json`; e
`.build/upstream-parity-20261111/decision-recovery-seed4-finalize2/RESULT.json`,
`decision-recovery-restored1/RESULT.json`, `decision-recovery-restored2/RESULT.json`.

## Consultas e publicação da captura sob contenção

A investigação do bloqueio seed5 confirmou duas regressões de contenção do fork.
Na listagem, o leitor mantinha o bloqueio partilhado do catálogo enquanto
verificava a proveniência de todos os registos. Um escritor legítimo podia
esgotar a sua espera sem criar tarefa/contexto. Duas fixtures com o leitor e o
escritor nativos reais reproduziram Busy, incluindo a criação de outro objetivo.
A reprodução nativa no perfil QA original observou esse bloqueio; quando a
observação terminou, o mesmo processo avançou. Não se promove esse diagnóstico
a uma causa única de todos os eventuais bloqueios do produto.

O leitor autentica agora os registos fora da secção partilhada longa. Os nomes
são capturados sob um bloqueio breve e os stamps de todos os ficheiros são
verificados sob outro bloqueio breve antes de devolver as sínteses. Um registo
alterado/retirado não é apresentado como atual. Se o catálogo mudar, complete é
false, incluindo quando os contadores de ficheiros indisponíveis são zero.
A consulta é informação; selecionar/admitir/executar conserva as revisões,
assinaturas, origem e autoridade próprias. Listing1 e os restantes formatos
persistidos/transportados não mudaram.

O primeiro ensaio com esse leitor passou tarefa/plano/Context12 e revelou o
segundo ponto: Busy antes de publicar o resultado da captura deixava a leitura
pendente. A fixture com um leitor real retido180ms reproduziu a falha. O selo
espera agora apenas por aquisição anterior à publicação, com o mesmo dono,
resultado, partição e prazo originais. A espera local entre aquisições é
limitada500ms e os limites UTC originais são revalidados antes da publicação.
Não chama novamente o leitor de dados, preparador de contexto, reserva, admissão
ou modelo. Uncertain e as restantes falhas não são repetidas; expiração e
revogação continuam a recusar. O limite global de espera da Authority e os
prazos de captura/ocorrência não foram aumentados. A falha de selo tem agora
identificação de etapa no log já existente.

Composição executada: Source5/build2, binário privado
`0ba38c5ed053c2c3b83103bcfdbacffb0b73eda53854122f89d65a0bc16d08e7`.
Source6 de fecho só altera documentação. Quatro ficheiros Rust, nenhum novo
ledger, formato, provider, UI duplicada ou permissão.91 testes dirigidos passaram:
25 de modelo/store,25 de autoridade,29 de regras e12 de histórico/futuros
restaurados. Incluem as cinco novas verificações de concorrência, publicação
incerta e prazo.79 web/tipos são reutilizados por hashes e ausência de delta
frontend; não se repetiu a suite integral.

Native fixed3 usou nova revisão/decisão explícita da rotina no perfil QA original,
sem reativar ocorrências antigas. Com consultas concorrentes, criou tarefa/plano,
Context12, Admission32, executou o Driver/HTTP próprios e guardou uma resposta;
1 chamada sintética/0 ferramentas/0 resultados desconhecidos. A reserva/captura
originais foram seladas uma vez. Todos os registos e definições anteriores
permaneceram exatos. O reinício conservou objetivo, resposta e memória e não
produziu novas chamadas HTTP/inferência. Critérios não foram dados como verificados.

O mesmo binário leu o perfil histórico de duas gerações de restauro, incluindo
as duas notas e a tentativa desconhecida65536. Dados e recibos ficaram iguais;
nova resposta, token antigo e alteração/admissão de rascunho histórico foram
recusados. As quatro regras continuaram snapshotOnly. Reinício com zero HTTP e
zero autoridade reconstituída passou. Os22 comandos de arquivo/restauro da
entrega anterior não foram repetidos: aqui foi exercitado o leitor atual sobre
os dados existentes.

Limite gráfico explícito: a imagem física X11 da escolha antes do reinício foi
observada e é legível. Após reinício, DOM e captura renderizada do WebKit mostram
a resposta, mas duas capturas físicas do X11 virtual mostraram o painel em branco,
mesmo aguardando pinturas. Não se certifica a pintura pós-reinício no compositor
real a partir dessas imagens. Confirmá-la numa sessão de desenvolvimento isolada
em Wayland é o próximo gate, antes de alargar a aceitação gráfica/instalada.

As tentativas intermédias continuam distintas: falhas de prontidão/Busy antes de
atingir o ponto observado, erro de sintaxe do observador, Busy na escolha original
conservada e Busy no selo. Uma falha do observador também interrompeu a limpeza;
os processos/contentor QA foram recuperados por identidades exatas e terminados,
com os registos originais preservados. A primeira compilação da fixture de selo
falhou num caminho de tipo; só red2 demonstra a regressão, com1 falha/2 sucessos.
Não se recertificam essas tentativas como aceitação da composição final.

Recibos privados: `.build/project-rule-preparation-20261117/{VERIFIED,PRESERVATION,WEB-REUSE}.json`,
`rust-red/RESULT.json`, `rust-seal-red2/RESULT.json`, `rust3/RESULT.json`,
`app-build2/RESULT.json`; `.build/upstream-parity-20261111/rule-preparation-existing2/{PREPARATION-DIAGNOSTIC,RECOVERED-CLEANUP}.json`,
`rule-preparation-fixed3/{RESULT,CURRENT-RESULT,RESTART}.json` e
`rule-reader-restored2/{RESULT,HISTORICAL-ONLY,RECOVERED-DATA-NATIVE}.json`.
Uso prolongado, semântica útil, entrega instalada/humana e49 integrais continuam
abertos. Instalada44/RPM49 e as regras de domínio/fábrica anteriores permanecem.

## Dependências restantes

Backup/restauro nativo destes dados passou no âmbito acima. O diagnóstico de preparação foi tratado no âmbito acima; pintura no compositor
real, uso prolongado, qualidade semântica com trabalho útil e consumo/revisão de
respostas em rotinas futuras, autorização própria de ações e restantes critérios
AUTO04/49, pacote/instalação e aceitação humana permanecem abertos. A escolha de
uma opção é informação: não é uma autorização de ferramentas ou do Guardian.

## Verificação no pacote normal50

O próprio RPM50 leu as decisões restauradas, as duas notas e os oito objetivos,
conservando os quatro contextos12 e a reserva desconhecida65536. Foram recusados
nova resposta/token antigo e mutação/admissão do rascunho histórico. O perfil foi
preparado pelo protocolo nativo50; a cópia direta recusada não foi contornada.
Na rotina atual, a revisão própria UI produziu uma única resposta sintética;
a recusa Busy do observador não originou nova execução. O fecho nativo foi por
leitura da mesma resposta/contabilização, UI e reinício sem HTTP. As capturas X11
virtuais reais de resposta atual e decisão histórica ficaram legíveis após
reinício nesta composição; a causa dos frames debug anteriores não foi provada.
91 Rust/79 web anteriores mantêm o âmbito, com delta de produto apenas nos cinco
metadados49→50 desde o último fecho privado. Wayland, instalada, humano e objetivo
integral continuam abertos. [Artefacto, tentativas e provas50](PETS-INSTALLED-DELIVERY-50.md).
