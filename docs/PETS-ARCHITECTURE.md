# Arquitetura dos pets companheiros — Linux

Este é o guia de estado e aceitação do fork. O quadro seguinte prevalece sobre
as notas históricas no fim do documento. Última revisão: 21 de outubro de 2026.

## Prioridade de desenvolvimento atual

Concluir a integração e validação da build corrente. Depois, por orientação
expressa do utilizador, priorizar funcionalidades, backend e sistema; frontend,
UI e UX deixam de ser prioridade até nova indicação. Isso não adia correções
bloqueantes necessárias à entrega nem elimina os requisitos já registados.
Não iniciar novas melhorias visuais por iniciativa. Os incrementos funcionais
já preparados conservam a sua revisão e o seu critério de aceitação.

### Responsável único e organização por unidades

Por pedido explícito mais recente do utilizador, a sessão
`01a1505c-e9e3-7bd2-87c0-fd9424be0538` assume sozinha o objetivo integral:
**Sistema** (rede/Wi-Fi, Bluetooth, som, energia/sessão), **pets LLM** (sessões,
execução, conversa, memória e utilização), arquitetura, integração, release e
registo do anfitrião em sysworkspace. A sessão integradora anterior parou o seu
trabalho. Não criar novos agentes. As unidades completas continuam a organizar
as implementações; a divisão anterior entre dois executores terminou.

O objetivo ativo é «quero transformar os pets em asisstentes digitais sempre
prontos e poderosos já que temos a possibilidade de usar AI». Terminar um
incremento ou passar uma suite não conclui este objetivo. Os pacotes fechados
mantêm-se imutáveis; a composição .24 parte da fonte exata .23; os incrementos seguintes partem da fonte congelada .24.

O incremento Sistema integrado na .24 reúne a leitura Bluetooth própria e a
operação explícita de desativação Wi-Fi. A leitura Bluetooth passou 1368 testes
Rust, dez ignorados e cargo check; o conjunto com a desativação passou **1400
testes Rust, dez ignorados e cargo check**, com 569 inputs inalterados. O delta
fechado contém 12 ficheiros existentes e sete novos, adotados na .24.
A leitura usa o contexto original de Sistema e o transporte autenticado existente,
com limites de dados/tempo e uma consulta de cada vez; não publica endereços,
caminhos nativos, tokens de emparelhamento ou alvos de comandos. Desativar Wi-Fi
reutiliza o mesmo ator, autoridade e slot físico da rede, com scope e ACK próprios.
Os dois incrementos foram integrados na .24, sem serem usados para alterar a rede.

A correlação obrigatória do `turn.id` aninhado no evento final do Codex está
fechada e revista num pacote que foi integrado na .24: seis casos novos, 1362 testes Rust,
dez ignorados e cargo check. A composição .24 reúne essa correção com o
incremento Sistema e a desativação temporária do pet GitHub. A identidade
`integration_github`, os seus perfis, conversas e tarefas antigas permanecem;
ficam bloqueados novos pedidos e consultas do pet. O pet Copilot CLI permanece.
Não realizar ensaios reais GitHub CLI/Copilot que consumam quota. Os restantes
pets também fazem parte do desenvolvimento e da aceitação.

Desativar Wi-Fi é um controlo explícito que o utilizador poderá acionar no
Sistema. Não é uma mudança na configuração desejada da estação nem uma ação
automática: não foi desligada a ligação e não houve testes de escrita na rede.
O ACK de software continua sem provar o efeito físico no rádio.

## Estado da entrega

A **0.5.1-mochi.24 está instalada e ativa permanentemente**, com Bluetooth de leitura, o controlo
explícito de desativação Wi-Fi, a correlação do turno Codex e a desativação
temporária do pet GitHub. Passaram **1413 testes Rust, zero falhas e dez
ignorados**, cargo check, **1010 JavaScript**, TypeScript, Vite e quatro Python,
com 571 fontes iguais durante verificação e empacotamento. RPM `04ea788d9da2…`,
prefixo `~/.local/opt/coucou/0.5.1-mochi.24`; executável, socket, XDG e DMS reais
confirmados. Não houve chamadas
a modelos, testes de quota ou ações reais de rede/Bluetooth neste incremento.

A política de desativação é explícita por instância na fábrica, separada dos
25 perfis guardados. Mantém a classificação da conta e as rotas de identidade
necessárias para ler o histórico antigo. Bloqueia admissão, seleção de modelos,
retoma de tarefas, voz, ingestão de documentos e polling do pet GitHub; não
concede a outro pet os seus pedidos ou dados. O plano do anfitrião retira apenas
`integration_github` da lista de pets ativos, preservando todos os outros campos.
A migração foi aplicada com backup e comparação dos outros campos: os 25
perfis ficaram byte a byte iguais, assim como o journal. O Ollama recuperou
automaticamente os 73 caracteres na CID original, apresentou o estado de
rascunho recuperado e passou fechar/reabrir sem escrita ou envio. Codex abriu
a conversa, mas a retoma do seletor pela acessibilidade não encontrou a linha
para reabrir; Copilot e esse percurso Codex conservam aceitação gráfica pendente,
sem regressão demonstrada. Os quatro rascunhos dos pets anteriormente ativos
ficaram guardados privadamente. A configuração concorrente que deixou Gravity
inativo foi preservada, sem reativação. O rascunho GitHub
foi guardado privadamente antes da troca; não foi reaberto depois da retirada,
e não há código novo de eliminação/migração dos seus dados IndexedDB. A conversa
e as tarefas antigas GitHub permanecem no journal com o mesmo dono. A .23 fica
disponível para recuperação. Apresentação/hotplug e os sete widgets direitos
conservam os limites anteriores; `ownUiComplete=false`.

A **0.5.1-mochi.23 foi a versão ativa anterior**, RPM
`62d2bd67e39c…`, com 562 fontes congeladas e prefixo
`~/.local/opt/coucou/0.5.1-mochi.23`. Passaram **1356 testes Rust, zero falhas e
dez ignorados**, cargo check, **1007 JavaScript**, TypeScript, Vite e quatro
Python. Os 562 inputs permaneceram iguais durante testes e empacotamento; os
13 ficheiros do pacote, incluindo os sete da ponte Copilot e o relay original,
foram verificados. Executável, socket, XDG e ambiente DMS do processo real foram
confirmados. A .22 permanece para recuperação.

A troca preservou 26 mensagens, oito conversas, tarefas, seleção e preferências.
Os cinco rascunhos atuais passaram abrir/fechar/reabrir com dono/CID e texto
exatos: Ollama com 73 caracteres, restantes quatro vazios. Depois de reiniciar
o mesmo executável .23, o Ollama **recuperou automaticamente os 73 caracteres**
na conversa correta, sem qualquer escrita após o reinício, apresentando o estado
«Rascunho recuperado neste dispositivo». Journal e preferências ficaram iguais.
Não foram enviadas mensagens nem respondidas aprovações nesses ensaios.

A .23 integra os 50 caminhos funcionais revistos sobre a fonte exata da .22:
vocabulário PT/EN por pedido, rascunhos persistentes por sessão, medições internas
de latência desligadas por defeito, fábrica de pets e ativação Wi-Fi explícita.
Os 25 defaults de pets foram confirmados sem reescrever preferências do utilizador.
A qualidade humana com Bluetooth, a latência audível da Raquel e o ensaio real
de ativação Wi-Fi ainda não estão aceites. O novo backend Bluetooth, a desativação
Wi-Fi e a correção de turnos Codex ficam fora deste pacote congelado.

A **.22 foi a versão ativa anterior**, RPM `121e538672f6…`, fonte544,
1300 Rust (dez ignorados), 953 JS/TypeScript/Vite. As suas correções de geometria
e projeção/encaminhamento de pedidos continuam preservadas na .23. A aceitação
de apresentação/pedidos e mudança real de ecrã permanece parcial; o primeiro
hover .22 foi inválido porque o cursor terminou fora do pet em movimento. Estes
ensaios pendentes não são apresentados como concluídos nem como regressões
provadas e acompanham a entrega funcional seguinte.

A **.19 foi a versão ativa anterior**, RPM
`0880f34f74ff…`, com 507 ficheiros de fonte congelados e prefixo imutável
`~/.local/opt/coucou/0.5.1-mochi.19`. Executável real, dono do socket, XDG,
launcher e autostart foram verificados. A .18 permanece para recuperação.
Passaram **1079 testes Rust, zero falhas e 10 ignorados**, **900 JavaScript**,
TypeScript e Vite na composição exata que foi empacotada. O relay original e os
sete ficheiros da ponte Copilot foram preservados. A integração do anfitrião
passou a verificação de 285 ficheiros e os 491 testes.

Na composição da **.20**, os 187 ficheiros Rust/build e os inputs
embebidos/configuração ficaram iguais depois da integração JS. As 544 fontes
estão congeladas; o RPM `26ab1e35ac7b…` compilou e os 13 ficheiros de payload,
incluindo sete da ponte Copilot e o relay original, foram verificados. Inclui a
ligação nativa R2 revista, rede própria de leitura/pesquisa, definições FE49-r2
e atividade da conta FE50, preservando Recap da .19. A revisão da atividade da
conta registou o P2 defensivo de enums indicado abaixo e adiado. A sessão foi desbloqueada e
a troca protegida está concluída. A aceitação gráfica continua em curso; não
atribuir os ensaios da .19 aos novos controlos.

A .18 acrescenta **Som dentro da própria ilha**: saídas, microfones, portas,
perfis, volume, balanço e silêncio, com leitura/ações tipadas e confirmação do
estado. O percurso Sistema → Som, o regresso e a mudança Pets/Sistema foram
observados no executável permanente. Recolher volta à galeria; selecionar
Sistema recupera a página de som. O ensaio executou duas ações reais apenas
numa saída virtual própria: silenciar e repor, confirmadas independentemente
por pactl e pela UI. A saída temporária foi removida e a UI deixou de a listar;
a saída e o microfone predefinidos ficaram iguais. Não houve reprodução sonora.
Volume, balanço, mudança de microfone/perfil/porta e regressão nativa completa
do Spoty conservam a sua aceitação específica. Não atribuir o ensaio Spoty de
uma versão anterior aos novos controlos de Sistema.

O som de Sistema tem autorização independente da música do Spoty. Leituras
antigas deixam de atualizar a página oculta; fechar não cancela uma operação
nativa já despachada. A UI explica que alterar o microfone predefinido ou o
perfil pode mover captações e interromper Bluetooth. O backend valida a janela
nativa, a sua visibilidade e a definição global; isto não atesta o tab DOM.
Rede, Bluetooth, energia e os restantes controlos próprios continuam em falta:
**`ownUiComplete=false`**, sete widgets direitos mantidos, sem corte do centro
antigo. A captura da barra confirmou blocos, intervalos transparentes, tray,
periféricos/componentes, clipboard, notificações e bateria.

A .18 inclui FE41: navegação e atualizações automáticas das quotas usam apenas
cache; atualizar a conta exige o botão explícito. Protege a seleção durante a
carga dos perfis, confirma o descarte das alterações e impede que a aplicação
tardia de um perfil afete outro pet depois de navegar. A suite e os ensaios
simulados passaram. No executável real, o contador Codex abriu o gestor filtrado
para Codex, com 1748 entradas nesta consulta datada. A página de perfis abriu;
o ensaio nativo de editar/manter/descartar não foi concluído, porque o campo
WebKit não expõe EditableText. Tentativas de navegação também foram interrompidas
por recolhas/mudanças do ponteiro. Não houve gravação de perfis pela UI. Antes
da troca seguinte, Gravity abriu o filtro com inventário indisponível e Copilot
abriu o filtro com 33 registos, ambos observados. Restam outros filtros/rotas
do gestor e a matriz nativa completa das definições. Limites não contam como PASS.

A troca preservou **24 mensagens públicas em oito conversas**, tarefas,
seleções e definições. O rascunho atual de **73 caracteres** foi guardado antes
da troca, reposto na mesma conversa e confirmado ao fechar/reabrir na .19,
**sem envio**. O journal permaneceu byte a byte igual no fim do ensaio.
Nenhuma inferência, atualização da conta ou decisão de aprovação foi acionada.

A .19 acrescenta a correção de persistência do resumo semanal e FE46, sem os
módulos privados de rede. A revisão independente confirmou a correção da corrida
de privacidade. Arranque e reposição do rascunho foram aceites no executável real;
os mutadores do resumo semanal passaram com dados sintéticos, sem teste de
apagar/ocultar/desativar no histórico real. As preferências do resumo e todos os
85 turnos e 387 decisões anteriores foram conservados; durante o trabalho foi
registada mais uma decisão normal. Não representar o ficheiro inteiro como
imutável enquanto os hooks continuam a recolher atividade. A troca esperou
pelos pedidos nativos terminarem, sem tarefas próprias ou captação de áudio.

Na .19, as cinco linhas explícitas da biblioteca abriram as conversas próprias
de Codex, Gravity, Copilot, Ollama e GitHub com dono/CID corretos. As cinco
capturas foram observadas; fechar conservou o journal byte a byte e o rascunho
Ollama de 73 caracteres. Spoty abriu o seu cartão de música, sem chat LLM e sem
acionar reprodução ou som. Isto aceita navegação/apresentação, não inferência
ou execução por todos os modelos. A sessão externa Codex também mostrou as
mensagens atuais do terminal, separada da conversa própria e sem envio quando
o turno era ambíguo. Melhoria UX pendente: «Conversar» acompanha a sessão externa
ativa nesse contexto; distinguir essa ação de abrir a conversa própria do pet.

Evidência anterior, da .17: os cinco cartões LLM e galeria/residentes passaram
as contagens com âmbito explícito. Codex mostrou quatro clientes TTY, ≥1747
UUIDs conhecidos e ≥1001 arquivados; Copilot mostrou 33 registos com arquivo
desconhecido. Ollama mostrou biblioteca Coucou 2/1 e Gravity 1/0. GitHub indicou
a fonte física Codex. Os contadores locais Ollama/Gravity abriram as conversas
existentes com dono/CID corretos, sem criar sessões. Números datados não são
constantes, totais globais ou prova de sessões controláveis.

O observador de definições nativas continua passivo. Não ativa Always Allow.
O transporte de rede R3 e o lifecycle R2 foram revistos e passaram 107 testes
isolados inclusivos (24 novos, 19 de leitura, 48 de lifecycle e 16 de projeção).
Permanecem privados. O executor R1 passou 139 testes inclusivos, incluindo os
107 anteriores; faltam ligação à autoridade nativa/Tauri, UI e aceitação no
sistema. O ACK de pesquisa Wi-Fi nunca prova conclusão física.
Uma inconsistência documental de :Z no lifecycle R2 foi registada para corrigir
na adoção; o runner efetivo usa o lock externo e :Z coordenado. Não editar os
recibos fechados nem atribuir estes testes ao executável .19.

A configuração efetiva antiga do Ollama não tinha `localExecution` e o backend
conservava corretamente «Só conversa». Em 20/10 foi aplicada apenas a preferência
revista `localExecution: {mode: "ask"}`, já presente na fonte do anfitrião, com
backup privado e confirmação de que os outros campos ficaram iguais. Não
aprova nem executa comandos automaticamente. O `/api/show` local confirmou
ferramentas no `qwen3:8b` e ausência delas no `gemma3:4b`; não gerou respostas.
A escolha atual de modelo foi preservada. O teste completo modelo → proposta
→ decisão humana → execução → resposta continua pendente. O teste de aprovação
sintética anterior não certifica esse percurso completo.
Os CLIs Codex/Gravity continuam com as suas próprias capacidades e decisões.

Continuam pendentes o vínculo inequívoco processo/sessão controlável, tokens
CLI, as matrizes de execução/áudio/monitores, a qualidade PT/EN/Raquel e os
commits assinados no fim pedido. As auditorias 43/44 das definições estão no
quadro adicional no fim; só a parte Recap foi corrigida na .19.

### Correções prioritárias de interação — aceitação reaberta

Na .20, após passar do portátil (escala 2) para o monitor externo (escala 1),
o hover do Codex reposicionou o cartão e expôs os seus controlos à acessibilidade,
mas a superfície `coucou` manteve alfa zero no compositor e não apareceu na
captura. Dois movimentos reais do ponteiro foram confirmados; neste ensaio o
hover chegou ao percurso de abertura. Os módulos centrais desse percurso são
iguais aos da .19, pelo que não está demonstrada uma regressão introduzida pela
.20. O reinício controlado do mesmo executável no monitor externo recuperou
alfa um; conversas e preferências ficaram iguais e o hover voltou a abrir um
cartão legível, confirmado visualmente com o cursor dentro do pet no fim do
ensaio. Isso apoia investigar o ciclo de apresentação/hotplug, sem certificar
ainda a causa ou a correção definitiva.
Não houve decisão automática, alteração de permissões ou pedido a modelos.

Há dois defeitos frontend independentes, reproduzidos com transporte fictício:

- Um pedido pendente bloqueia o hover de todos os pets, incluindo o dono.
  Além disso, um marcador agregado de terminal B pode ter prioridade sobre o
  pedido exato da sessão A. O candidato 52-r2 corrige ambos num único módulo,
  com prioridade do destino exato e sem fallback para B em caso de recusa/erro.
  Passaram 953 testes JS, TypeScript, Vite e as matrizes de browser; a revisão
  complementar não encontrou bloqueador. Está fechado e integrado na .22;
  52-r1 foi substituído e não deve ser integrado.
- Receber apenas o evento de aprovação/pergunta pode não projetar imediatamente
  o aviso no pet que já está a trabalhar. Repetir o mesmo snapshot faz o aviso
  aparecer. O candidato separado 57 atualiza a projeção visual no evento; os testes
  e a revisão independente passaram. Não cria destinos, não aprova e não resolve o alfa.

A composição **.22 instalada** reúne 52-r2 e 57 sobre as fontes .20, além da
correção nativa R2. Passou 953 testes JavaScript, TypeScript e Vite. A revisão
independente de 57 incluiu nove provas adicionais da ordem de eventos com os
handlers e modelo reais, sem GUI ou IPC nativo. A parte nativa recalcula as
dimensões lógicas de WebKit/GTK e recusa callbacks de geometria/ecrã antigos.
A revisão da primeira candidata encontrou uma janela defensiva entre ler a
política de input e aplicar a região; a contraprova usa o helper real com
leitura simulada, sem demonstrar alcançabilidade em GTK. R2 volta a validar
depois da leitura e passou revisão independente, nove fixtures e a composição
final de 1300 testes Rust, com dez ignorados. R1 foi substituída e não deve ser
instalada. A .21 privada de voz, rascunhos e medições nunca foi instalada como
um todo; os seus deltas compõem a .23 separada.

A primeira tentativa de hover na .22 não terminou com o cursor dentro da
posição final do pet e não expôs os controlos do cartão: é um ensaio inválido,
não uma aceitação nem prova de regressão. Os cinco chats próprios passaram a
abertura e reabertura nativa. Falta aceitar a apresentação de pedidos no
percurso nativo e a mudança real de ecrã/escala; alfa um isoladamente não
certifica um cartão visível.

A recusa silenciosa ao abrir o pedido exato e a exigência de movimento adicional
continuam lacunas distintas. Concluir a apresentação nativa, a sinalização do
pedido e o encaminhamento correto antes de retomar a aceitação geral. As
aprovações conservam os checks nativos de dono, sessão, pedido e prazo.

### Quadro de conclusão atual

| Área | Estado e próximo critério de conclusão |
| --- | --- |
| Barra e ilha | .24 ativa; blocos e sete widgets direitos preservados. Só retirar controlCenterButton após paridade própria completa. |
| Som próprio | Leitura/navegação e silenciar/repor saída de teste aceites. Faltam volume/balanço/portas/perfis/microfone e regressão Spoty no executável novo. |
| Rede, Bluetooth e energia | Ligação GTK/Tauri R2 revista e integrada na .20 ativa, com UI de leitura/pesquisa Wi-Fi; aceitação nativa em curso. A .23 inclui ativação Wi-Fi explícita, ainda sem ensaio real da ação. A .24 inclui leitura Bluetooth própria e o backend de desativação Wi-Fi explícita, sem aceitação de ação real; energia/ligações e restantes controlos próprios precisam de implementação e aceitação. |
| Sessões e consumo | Contagens com fontes reais disponíveis. .24 inclui atividade da conta Codex, revista e compilada; falta completar a leitura/aceitação pela UI. Falta identidade inequívoca para controlo das sessões CLI, ledger de tokens e matriz completa de retoma/interação em background. |
| Execução pelos pets | CLIs conservam capacidades/permissões nativas. Ollama recebeu política de propostas com aprovação por ação, preservando o modelo atual; falta escolher modelo com ferramentas e aceitar o percurso completo. |
| Definições | .19 inclui FE41 e backend Recap + FE46; arranque/rascunho aceites, mutadores Recap testados apenas em dados sintéticos. Reconciliar definições gerais/Chat44 e completar ensaios nativos de perfis. |
| Voz e documentos | Qualidade PT/EN com o microfone efetivo, pronúncia e latência Raquel ainda pendentes. Preservar email só Ollama e repetir regressões de documentos por capacidade ao fechar a entrega. |
| Interação prioritária | As correções 52-r2/57 estão integradas desde a .22 e preservadas na .24; falta aceitar o percurso real após hotplug. Nenhuma delas concede autorização. |
| Fábrica de pets | Integrada na .23, com moldes LLM/aplicação e extensão de serviço; os 25 defaults ficaram iguais. Identidade, provider, grants e execução local não são herdáveis. Novos pets e mudanças de permissões continuam explícitos. |
| Aceitação final | Completar monitores/hotplug/foco/tarefas por provider e alterações de permissões; rever fontes e realizar os commits assinados no fim pedido. |

Os nomes, aliases, permissões e perfis `fast`/`reasoning` já pertencem ao ficheiro
editável `~/.config/coucou/pets.json`, com base revista em
[`pets.default.json`](../windows/src-tauri/pets.default.json). O modo `auto` serve
voz e chat; o default revisto do Codex continua `gpt-6-luna` com esforço `low`
no perfil rápido. A escolha efetiva ainda tem de respeitar os modelos disponíveis
na conta e as seleções explícitas do utilizador. O plugin de email continua
habilitado por default apenas no Ollama. Estes pontos já implementados não devem
ser confundidos com os ensaios nativos de perfis/permissões ainda pendentes.

Pendências pequenas, adiadas conforme a preferência explícita do utilizador:

| Caso | Limite conhecido e prova |
| --- | --- |
| Feedback ao abrir terminal | No ensaio simulado, pedir terminal para A, mudar para B e receber a recusa de A mostra esse erro em B. O rascunho e a navegação ficam preservados; não houve envio ou abertura de terminal real. Falta cercar o callback pelo destino/intenção original. |
| Validação defensiva da atividade da conta | O parser FE50 converte status/reason com String e aceita arrays JSON equivalentes. O backend tipado atual não produz esses arrays e só consultas nativas aplicam snapshots; os eventos não aplicam estado. Falta rejeição estrita de tipos, sem caminho de produção demonstrado nesta prova. |
| Cartões que abrem sem pedido | Conservar como pendência os casos menores já reportados; não confundir com autorização ou envio de mensagens. |
| Retenção do cartão de outro pet | Ensaio fictício reproduziu um pedido Codex a manter aberto o overview manual, não fixado, do Ollama depois de ambos os ponteiros saírem. Falta limitar a retenção ao dono, preservando fixação, documentos e decisões. Correção UX adiada, fora do hotfix. |
| Rascunhos e metadados incoerentes | No candidato privado de persistência, um contador de revisões inferior ao registo existente pode reciclar uma revisão antes de o cliente rejeitar o ACK. O escritor transacional normal não produz esse estado; falta recusar a incoerência antes da escrita. Defesa adiada, sem ocorrência demonstrada no perfil real. |

### Próximo incremento preparado, ainda fora da aplicação instalada

O vocabulário de voz passa a receber os nomes canónicos dos pets configurados
e cinco expressões técnicas, por pedido e sem fixar a língua. A revisão e os
testes isolados passaram. Sete amostras sintéticas pelo helper efetivo conservaram
PT/EN e termos como «Docker Compose» e «pull requests»; «Spoty» ainda foi
transcrito como «Spotty». Não é aceitação da pronúncia humana, do Bluetooth ou
de todos os comandos de chamada. Não há substituição automática de palavras.

O candidato de rascunhos guarda apenas texto no IndexedDB, pela identidade
completa da conversa, com revisões transacionais, conflitos visíveis e limites
explícitos. Passaram 977 testes JavaScript, TypeScript, Vite e 21 casos de browser
com dados fictícios; a revisão não encontrou um caminho normal bloqueante e
registou a defesa acima. Abrange o chat expandido, não o compositor inline antigo.
A persistência no WebKit nativo, incluindo fechar/reabrir a aplicação e trocar
de versão, continua por aceitar. Até lá, conservar as cópias privadas antes das
atualizações. Um envio e a limpeza do rascunho não são uma transação única: não
há garantia de recuperação de um envio incerto nem reenvio automático.
Os dois incrementos e as medições opcionais já foram compostos numa área privada
da futura .21, com **1317 testes Rust, zero falhas e dez ignorados**, **977
JavaScript**, **quatro Python**, cargo check, TypeScript e Vite aprovados.
As 551 fontes estão congeladas. As fontes partilhadas da .20 permanecem
congeladas; ainda não existe RPM ou instalação .21.

As medições de demora continuam separadas da otimização. Em sessões Codex
próprias e efémeras, `gpt-6-luna`/`low` respondeu ao texto sintético em 16,36 s
por `exec`; outro ensaio app-server demorou 6,94 s. Num par separado com o mesmo
processo/thread de teste, os turnos demoraram 3,89 e 1,05 s. São percursos
reduzidos, sem ferramentas, UI ou voz; não provam o ganho causal de um pool nem
justificam reutilizar sessões de terminal. A instrumentação opcional foi revista
e integrada apenas na .21 privada, desligada por default, com etapas fixas e tempos, sem conteúdo
de mensagens, identificadores pessoais ou credenciais.

Dois ensaios sintéticos com o helper Raquel inalterado entregaram os primeiros
bytes em 2,73 e 3,84 s. O GStreamer produziu o primeiro áudio descodificado cerca
de 3 ms depois, usando uma saída de teste sem dispositivo de som. Isto mede a
síntese/transporte e o descodificador; não certifica a latência audível, a saída
Bluetooth ou o percurso completo do Mochi. Não foi aplicada uma mudança de voz.

### Fábrica comum de pets — candidata privada revista

O catálogo declarativo separa moldes LLM/aplicação, uma extensão de serviço e
instâncias. A regra de desenvolvimento fica também em `CONTRIBUTING.md`:
após adoção, novos pets e melhorias comuns usam base/molde/extensão; diferenças
específicas ficam na instância ou adapter. Evoluir moldes e migrar duplicações
quando isso trouxer benefício, mantendo identidade e autoridade fora da herança. Modelos, perfis e estrutura podem ser partilhados; nome, aliases,
provider, permissões e política de execução local têm de ser explícitos por pet
e não podem vir da herança. Owner lógico e provider físico são conceitos
separados; uma rota ausente ou ambígua é recusada. Os 25 defaults calculados
permanecem iguais, incluindo email apenas Ollama e Spoty sem chat LLM.

O pacote privado de 13 caminhos passou 1320 testes Rust (dez ignorados), 987
JavaScript, cargo check, TypeScript e Vite na sua composição sobre a .21 privada.
A revisão independente de fontes, hashes, replay e equivalência não encontrou
bloqueador. Falta compor depois do hotfix e aceitar a UI nativa. Harmonizar o
teto de 64 entradas no resolver frontend/gerador antes de ampliar o catálogo;
o catálogo atual tem 25. Não há migração de permissões nem adoção de configurações
do utilizador por este candidato.

### Paridade que falta para retirar o centro de sistema antigo

A revisão das fontes instaladas identificou 17 grupos funcionais. É um catálogo,
não prova de que todos estejam configurados ou suportados nesta máquina. Antes
do corte, inventariar a lista ativa e aceitar os grupos presentes pela UI própria
da ilha. A existência de uma chamada num cliente não prova o respetivo handler,
formato de resposta ou capacidade viva do serviço.

| Grupo | Trabalho restante |
| --- | --- |
| Wi-Fi | Aceitação nativa da leitura/pesquisa .20; ligar/desligar rádio, ligar/desligar redes, redes guardadas e autoconnect com alvos inequívocos e resultado observado. |
| Ethernet, rede móvel e VPN | Ações próprias sobre ligações/perfis disponíveis e respetivos fluxos de configuração. As listas observadas não dão ainda esses controlos. |
| Hotspot | Configurar/iniciar/parar, disponibilidade real e política de exposição de segredos. |
| Bluetooth | Adaptadores/dispositivos, descoberta, ligação, emparelhar/confiar/remover e prompts; codecs apenas onde suportados. |
| Som | Completar a aceitação dos dispositivos/perfis/portas/balanço/microfone e regressão Spoty; o plano de ligação ao novo contexto nativo continua separado. |
| Ecrã | Luminosidade por dispositivo/monitor, luz noturna, temperatura e agenda, respeitando a disponibilidade do compositor. |
| Preferências da sessão | Tema, não incomodar e inibição de suspensão, com estado efetivo e persistência correspondente. |
| Energia e sessão | Perfis disponíveis, detalhes da bateria e ações explícitas de bloquear/suspender/hibernar/reiniciar/desligar/terminar sessão. Nenhuma destas ações foi ensaiada nesta revisão. |
| Disco/componentes e plugins | Preservar periféricos/CPU/componentes; migrar apenas os grupos efetivamente presentes no centro, incluindo plugins opcionais e respetivas configurações. |
| Ferramentas e apresentação | Color picker, edição/layout/definições e ações de scroll/contexto que existam no botão antigo. Abrir o painel antigo não aceita paridade. |

Só `controlCenterButton` é candidato a retirada futura. `gravacao`, `systemTray`,
`udiskieMenu`, `clipboard`, `notificationButton` e `battery` permanecem, incluindo
indicadores personalizados e o acesso aos componentes. O próximo incremento
funcional de rede proposto é habilitar Wi-Fi, depois de confirmar o handler e o
ACK reais; não inventar um setter nem reutilizar o ACK da pesquisa. Este plano
não implementa os controlos nem altera a lista de widgets.

A prova posterior do handler oficial confirmou `network.wifi.enable`, sem
parâmetros, com ACK `result.enabled=true`. A implementação privada em curso
limita-se inicialmente ao NetworkManager observado e suportado, com âmbito
global do gestor, autorização nativa e leitura posterior separada. A UI terá
uma ação explícita de ativação, sem substituir o widget antigo nesta fase.
O ACK não confirma
rádio físico nem ligação. Corrige-se também uma limitação da documentação
anterior: a API 34 aceita pesquisa com adaptador opcional; é o cliente atual
que usa apenas a forma global. Os recibos fechados e a .20 não foram alterados.
O contrato prevê uma única leitura automática após ACK, quando o trabalho
nativo correspondente já terminou. «Cancelado» ou «desconhecido» podem surgir
antes disso e não desencadeiam essa leitura. Se outro trabalho ocupar o serviço
ou a leitura falhar, mostrar «por confirmar» e atualização manual; nunca repetir
a ativação, pesquisar redes ou enviar uma desativação como compensação.

### Voz PT/EN — evidência e próximo incremento

O ensaio sintético de seis frases com o helper Whisper atual, sem microfone ou
ações nos pets, reconheceu corretamente português e inglês separados. Deformou
termos em algumas frases mistas com a Raquel a pronunciar inglês em português,
incluindo Copilot, unit tests e pull requests. Não é aceitação de qualidade humana.
Na RTX 4060 Laptop de 8 GB observada, a primeira resposta demorou 5,705 segundos
incluindo carga; as seguintes cerca de 0,44–0,47 segundos de reconhecimento. São
tempos do ASR sintético, não latência de diálogo, VAD, inferência ou voz de resposta.

Uma comparação privada de 15 casos forneceu pistas de vocabulário ao modelo:
nomes dos pets e cinco termos técnicos melhoraram essas palavras; o controlo
«Olá, podes ajudar-me? Obrigado, até logo» manteve-se correto. Não prova ausência
geral de alucinações nem qualidade com Bluetooth. Está implementado e revisto num
pacote privado separado para .21 um vocabulário limitado por pedido, com nomes canónicos definidos
no ficheiro dos pets e sem alimentar aliases fonéticos aprendidos, que podem
conter erros. Preservar deteção livre PT/EN, sem tradução ou substituição posterior
do texto. Passaram 1304 testes Rust (13 novos e a base de 1291) e quatro casos
Python com modelo simulado. Um ensaio CLI posterior usou o helper final sem
alterações e os 11 termos dos pets configurados: preservou as expressões técnicas,
português/inglês separados e «Olá» nas sete amostras sintéticas. «Spoty» ainda
apareceu como «Spotty», variante que falta tratar/validar no reconhecimento do nome.
Sem microfone, ações, novos downloads ou alterações à configuração. Faltam
composição, teste humano e a latência completa da Raquel.
Nenhum modelo novo foi descarregado, nenhuma captação foi iniciada e a aplicação
instalada não recebeu esta experiência. O microfone predefinido observado era
48 kHz estéreo e não Bluetooth; isso não identifica o microfone escolhido nas
preferências do fluxo de voz, que ainda precisa de confirmação no ensaio real.

**Reaberto pelo utilizador — sessões de todo o PC (P0):** os valores da
biblioteca são exatos nesse âmbito, mas não cumprem a vista pretendida: quatro
terminais Codex abertos apareciam como zero na biblioteca. A observação Linux
independente confirmou quatro clientes nativos com TTY e tempos de arranque
próprios, sem assumir que estejam a executar um turno. Remover a duplicação de
«Em curso»/«Conversas» e apresentar sessões abertas, histórico conhecido e
arquivo com fontes reais. Turnos em execução/espera são estados separados.
A composição da **.16 está instalada**: census Linux, histórico R2 e
UI33/34/35/37. O census lê metadados dos
processos do próprio utilizador, com executáveis conhecidos, proteção contra
reutilização de PID e eliminação apenas de launchers comprovados. O ensaio real
observou quatro clientes Codex em terminal, todos sem UUID nativo comprovado.
Isso permite mostrar quatro terminais; não prova quatro sessões distintas nem
quatro turnos em execução. PID, pasta e título nunca autorizam envio, Stop ou
permissões. O trabalho do pet vem de reservas/turnos atuais com owner/CID exatos;
não mede janelas de chat abertas mas inativas.

`pet_station_history` usa o inventário de metadados do gestor existente, separado
do cliente que controla as sessões. Não gera respostas nem renova quotas, mas
consulta também modelos locais e pode atualizar o cache de capacidades; não é
um leitor SQLite sem efeitos em ficheiros. Uma leitura real observou índice
Codex de 1740, união de 1744 UUIDs distintos e pelo menos 1001 arquivados. Índice
e união não se somam. Copilot apresentou 33 registos no catálogo local, sem prova
de arquivo zero. Ausência do CLI Codex não valida o zero sintético do helper.
Os valores são observações datadas, não números fixos nem totais globais.

A UI tem uma única grelha de terminais/histórico/arquivo e uma linha de trabalho
do pet. Ollama/Gravity sem histórico nativo suportado mostram a biblioteca com
«Histórico no Coucou» e «Só neste pet»; falha de uma fonte suportada nunca
substitui o histórico externo por esse valor local. A consulta é partilhada,
com cooldown e atualização por evento de conclusão, sem timer novo por cartão.
Leituras falhadas conservam data e valores anteriores marcados como antigos.
Tokens CLI continuam sem produtor e quotas continuam numa fonte distinta.
As 495 fontes estão congeladas para a .16; a composição final inclui também a
UI37. Os contadores e a linha da biblioteca local abrem agora o chat expansivo
do respetivo pet com a lista de sessões. Passaram os 856 testes JS finais,
TypeScript/Vite e 12 grupos de cliques em browser com transporte simulado, além
dos 1040 testes Rust (10 ignorados). Os cinco cartões e a atualização após
arranque frio foram observados no executável permanente. Os cliques nativos
nos contadores e as restantes interações mantêm a sua aceitação própria. O
gestor ainda tem um filtro antigo que
assume Codex para integrações e não resolve Gravity especificamente; corrigir
com o binding físico real, sem inventar provider ou abrir todos por omissão.

**Correção de âmbito do Sistema — regra atual do utilizador:** manter na barra
os ícones das aplicações em segundo plano (Proton, Spotify, etc.), periféricos,
CPU/componentes, clipboard, bateria e notificações. Só a entrada antiga de
Wi-Fi/centro de comandos pode sair, e apenas quando a ilha oferecer todas as
funções na sua própria interface. Abrir popouts/widgets nativos não satisfaz
esta condição. Os oito percursos da .16 passaram como **atalhos para painéis
nativos** antes/depois de um corte provisório; calendário, previsão e notificações
também abriram. Esse ensaio não certifica interface própria e o corte foi
**revertido imediatamente** após a clarificação: os sete widgets originais e
`fusionIntentV1.system=false` foram repostos por alteração só de rightWidgets,
com confirmação visual. O backup e os recibos do ensaio ficaram privados.

A ponte 2.1.1 exige prova explícita `ownUiComplete=true`, ainda ausente no
produtor atual, antes de aceitar qualquer retirada. Mesmo com essa prova futura,
a lista revista conserva os seis indicadores e remove só controlCenterButton.
Listas personalizadas são preservadas. A reversão da antiga lista vazia continua
suportada. Passaram 29 testes das funções QML, incluindo preservação dos ícones,
recusa de atalhos como prova e recuperação da lista anterior.

**P0 por implementar — Sistema com UI própria:** inventariar as funções atuais
do centro de comandos e expô-las na ilha com backend de estado/ações tipadas,
confirmações e erros reais: redes e ligações, Bluetooth/dispositivos, saídas e
entradas de som, luminosidade, perfis de energia e restantes funções existentes.
Pode reutilizar serviços do sistema; a interação principal deve acontecer na
ilha. Não usar popouts nativos como conclusão dessa migração. Preservar os
indicadores pedidos fora da ilha. A aceitação inclui ações reais controladas,
persistência e vários monitores; só depois propor corte seletivo. Qualidade de
voz e o hotplug geral conservam a sua própria fase. A previsão mostra a cidade
configurada no DMS; não a substituir por localização inferida.

### Plano do backend para a interface própria do Sistema

O estudo local confirmou uma via sem widgets: o daemon DMS expõe RPC local em
socket Unix 0600, com verificação de UID/processo/executável. `network.getState`
respondeu com metadados de redes, dispositivos e VPN; foram feitos dois pedidos
de leitura, sem alterações. O primeiro leitor encontrou o greeting espontâneo;
a segunda tentativa correlacionou a resposta pelo ID. O futuro transporte deve
tratar esse greeting, limites e respostas assíncronas. Não está integrado na .16.

O protótipo Rust privado R2 já trata esse protocolo e verifica socket, UID,
executável e continuidade do processo, com tamanho e prazo limitados. Passaram
19 testes offline, incluindo resposta pronta após expirar o prazo; a revisão
independente confirmou a correção. Um ensaio Rust real de leitura respondeu em
19 ms, com API 34 e contagens de coleções, sem guardar nomes de redes, endereços
ou credenciais. Foram três pedidos de leitura no total, incluindo as duas
tentativas Python anteriores. Não houve ações de dispositivos, pedidos a modelos
ou contas. Isto valida o transporte de leitura; DTO público, ações, prompts de
autenticação e interface própria continuam por implementar. O protótipo não está
registado no Tauri nem incluído na candidata .17.

A projeção de redes tem um segundo protótipo privado, com 16 testes offline
passados: seleciona apenas campos tipados para a futura UI, mantém valores
ausentes como desconhecidos e distingue listas vazias, parciais e indisponíveis.
Não transmite endereços IP/MAC, UUIDs, segredos ou erros nativos; nomes de redes
são dados privados da interface. Nomes repetidos ficam separados, sem os tratar
como identidade para comandos. A revisão independente inicial identificou um
limite de datas em falta; uma revisão separada passou a definir `observedAt` em
milissegundos Unix e a rejeitar valores fora do intervalo do JavaScript Date.
Essa correção passou testes de fronteira e revisão independente do delta. Integração e
aceitação continuam pendentes; a projeção não fornece ações nem pode retirar
widgets.

| Grupo | Base existente | Implementação/aceitação em falta |
| --- | --- | --- |
| Redes | NetworkService e RPC local com redes/dispositivos/VPN | DTO sanitizado, alvos opacos, scan, Wi-Fi, ligações guardadas, Ethernet/celular/VPN/hotspot quando disponíveis; autenticação na própria ilha e confirmação real do resultado. |
| Bluetooth | Serviço BlueZ/DMS com emparelhamento e codecs | Estado/dispositivos, ligar/desligar, pesquisa, confiança/remoção e prompts próprios de emparelhamento vinculados ao pedido. |
| Som | Motor Linux já usado pelo Spoty; nova autorização/DTO privados para Sistema | O backend privado passou oito fixtures e 23 testes de áudio no SDK real (os oito incluídos), com registo dos dois comandos no overlay. Não exige Spoty/music, recusa streams/Route explícitos e revalida janela/definição antes da mutação. Revisão independente passou sem novos problemas; UI42, integração e aceitação nativa pendentes. O microfone predefinido usa o helper existente, que pode mover capturas e trocar perfil Bluetooth. |
| Ecrã e energia | Serviços/IPC locais de luminosidade, luz noturna e perfil | Ações tipadas por dispositivo, confirmação por leitura, falhas/busy e perfis explícitos; mostrar bateria/componentes sem retirar os seus indicadores da barra. |
| Orquestração | Estado resumido e atalhos nativos na .16 | Ticket por operação, alvos/geração imutáveis, recebido ≠ concluído, cancelamento/prazos, leituras antigas e ausência de repetição automática após ACK perdido. |
| Credenciais | Tokens/pedidos do serviço nativo | Campos voláteis na própria UI, transporte local sem argumentos de processo, logs ou histórico do chat; resposta/cancelamento apenas do pedido exato. |

O contrato de rede em preparação é privado e **ainda não regista novos comandos**.
Os comandos de áudio só foram registados no overlay privado do SDK; não estão
na .17 instalada. A autorização atesta a janela nativa e a definição de Sistema,
sem alegar prova da aba DOM atualmente aberta.
A UI deve mostrar as funções realmente disponíveis e confirmar estados por
observação; não trocar de dispositivo ou presumir sucesso. A aceitação de uma
leitura não certifica ligar outra rede, emparelhar ou alterar um perfil. O sinal
ownUiComplete continua falso até renderer, backend e ensaios nativos cobrirem a
matriz acordada; só depois se revê a retirada seletiva do centro antigo.

**Autorizações temporárias da equipa do fork (P0, ainda inativas):** o pedido
posterior abrange todas as sessões Codex que trabalham neste desenvolvimento e
exige coerência entre pet e terminal. O protótipo RAM limitado ao broker do
Coucou não satisfaz isso nem o hook CLI externo. É necessário provar a sessão
nativa e a política efetiva nos dois percursos, excluindo sessões alheias ao
fork. Não anunciar permissões ativas nem trabalho noturno desbloqueado enquanto
não existir essa prova. O protótipo privado não altera o executável instalado.

### Registos de validação anteriores

#### .15 / candidata 25

A **0.5.1-mochi.15 foi instalada permanentemente**, com o pacote da
candidata 25 (`374e644cbe1a…`) no prefixo imutável
`~/.local/opt/coucou/0.5.1-mochi.15`. O executável real, o dono do socket e os
três diretórios XDG de produção foram verificados. O launcher e o autostart
passaram a selecionar esta versão; a recuperação por falha continua ativa. A
.14 e as versões de recuperação anteriores foram preservadas. A QA foi parada.

Os **cinco cartões LLM passaram o ensaio nativo de contagens**, com valores de
atividade, conversas e arquivo da biblioteca do pet, âmbito explícito e avisos
separados sobre as fontes externas. As capturas foram observadas. O caso Copilot
parcial vazio conserva os valores locais; não inventa um total externo zero.
Quotas de conta guardadas têm data original e aviso de desatualização. Os tokens
CLI continuam sem produtor ligado e são apresentados como indisponíveis. O
inventário externo Codex ainda não foi aceite: não confundir estes contadores da
biblioteca com todas as sessões dos terminais.

Passaram **827 testes JS, TypeScript e Vite**. As 481 fontes estão congeladas;
o Rust é idêntico ao conjunto que passou **996 testes, zero falhas e 10
ignorados** na revisão 24. Não se representa essa suite como novamente executada
na 25. As preferências foram preservadas; a produção conserva **24 mensagens
públicas em oito conversas**, incluindo seis mensagens e duas tarefas reais
recuperadas da QA. As mensagens sintéticas foram excluídas por hashes exatos,
sem importar o journal completo. As seleções originais mantêm-se.

O rascunho original de 73 caracteres foi reposto no CID original e manteve-se
ao fechar/reabrir, sem envio. Continuam pendentes os commits assinados e a
retirada dos sete widgets direitos após paridade nativa do sistema. A aceitação
dos cartões não certifica o inventário externo, os tokens CLI, a navegação de
sessões completas ou a restante matriz de execução, áudio e monitores. A opção
privada de autorização por sessão ainda não está integrada nem ativa.

#### Candidatas anteriores


A **candidata 22 foi compilada e esteve ativa em QA isolada**, com o dono do socket,
executável e diretórios privados confirmados. O RPM Linux é
`a82a077183ee…`, com 465 fontes congeladas. A revisão passou **948 testes Rust
(zero falhas, 10 ignorados), 785 JS, TypeScript e Vite**. O ledger continua sem
novas tentativas; a aceitação de interação é parcial, conforme descrito abaixo. A versão permanente e os widgets direitos continuam
por migrar. Não contar o arranque como aceitação funcional.

A **candidata 23 foi compilada e esteve ativa em QA isolada**, substituindo a 22 sem
alterar as preferências nem as conversas. O RPM Linux é `ed7c28ba6efb…`, com
471 fontes congeladas; passaram **961 testes Rust, zero falhas e 10 ignorados**,
**795 JS, TypeScript e Vite**. Acrescenta os dois controlos independentes da
ilha nas Definições e a atualização por campo no backend. A confirmação de
gravação não certifica a visibilidade das superfícies nativas.

Na **QA 23**, o Spoty silenciou, repôs o som e ajustou para **31%** um fluxo
silencioso criado exclusivamente pelo ensaio. Cada alteração foi confirmada
por leitura independente do servidor de áudio, com índice/serial/PID do fluxo
revistos. O controlo real expõe a função AT-SPI `toggle button`. Player e fluxo
foram removidos, sem alteração das conversas, modelos, decisões ou dispositivos.
O teste durou cerca de 18 segundos e as capturas anterior/posterior foram
observadas. Isto aceita esse percurso de escrita por aplicação; perfis,
encaminhamento, música autenticada e restantes controlos conservam a sua própria
aceitação. Os resultados 22 abaixo são históricos e não substituem os da 23.

A **fonte 24 foi compilada e arquivada**, sem ativação nativa:
480 fontes, **996 Rust sem falhas (10 ignorados), 823 JS, TypeScript e Vite**.
Integra a identidade Ollama v2, a lista independente por origem, os limites da
consulta Codex e a correção dos contadores/quotas dos cartões. As contagens
conhecidas da biblioteca aparecem com o âmbito do pet; a informação externa
continua separada. A quota vem apenas do cache local sanitizado, com a data
original, e nunca representa tokens. As interfaces reais foram ensaiadas em
54 grupos de browser com transporte nativo simulado; falta aceitar o novo
pacote no Linux antes de anunciar estes dados como entregues.

A revisão do relato de 20/10 encontrou ainda um caso na fonte 24: o inventário
Copilot é sempre parcial e pode estar vazio. Essa fonte não deve substituir as
contagens conhecidas da biblioteca por três valores desconhecidos. A correção
seguinte conserva a biblioteca principal quando a fonte parcial não tem qualquer
contagem exata ou observação positiva; mantém o aviso externo separado. Uma
fonte completa com zero continua a mostrar zero, e observações parciais positivas
continuam identificadas como subtotais. A candidata 24 não foi usada como
aceitação final deste relato. A fonte 25 corrigiu este caso e passou os cinco cartões nativos antes da instalação permanente.

Na **QA 22**, os cinco chats abriram com dono e compositor apresentados, sem
novos pedidos aos modelos nem decisões do integrador. O Codex mostrou o histórico
público de uma sessão real de terminal `#01a156f5`, com estado de trabalho. Ainda
não se repetiu a sessão root que tinha vários turnos ativos, e a lista externa
Codex ainda carregava na captura. Notificações e uma aprovação real de outra
sessão taparam partes das imagens; não contar como aceitação visual completa.

Durante o ensaio, outra intervenção no sistema passou de dois ecrãs para só o
portátil. Com o mesmo PID, o habitat ajustou de 3440 para **1440 píxeis lógicos**,
e a central ficou em **x=340, y=270**, na escala 2. Confirma-se esse percurso real;
o roteiro controlado nas duas direções não foi executado. O integrador não
alterou o modo dos ecrãs. O teste de áudio falhou na preparação do player e,
na segunda tentativa, num seletor AT-SPI do ensaio: pediu `push button`; na QA 23 confirmou-se que o
controlo de mute expõe `toggle button`. Os dois players/fluxos foram removidos; não houve escritas
de áudio, e não se demonstrou uma falha do backend.

A **fonte 21 foi compilada e esteve ativa na QA anterior**. O RPM Linux
`9b8fcf30e62d…` conserva as 456 fontes revistas. O dono do socket e o executável
foram confirmados. Passaram **929 testes Rust, zero falhas e 10 ignorados**,
**764 testes JS, TypeScript e Vite**. Inclui observabilidade 24–27, os ajustes de
viewport, captura de consumo Ollama por tentativa e Spoty 2/3 dedicado a áudio.
O ledger privado arrancou com zero tentativas; isso confirma o bootstrap, sem
certificar ainda contagens de inferências reais. Não houve pedidos aos modelos
nem decisões de aprovação neste ensaio 21.

A **fonte 22 congelada já substituiu a QA 21**, preservando os seus artefactos. Já inclui a
consulta individual de sessões além das 12 linhas do resumo e a interface 28;
as listas completas, quotas e controlos de modelos/memória mantêm-se. A correção
Codex preserva o histórico público com vários turnos ativos, deixando envio e
interrupção indisponíveis. O envio direto também revalida o histórico, e os
sinais em direto não podem ultrapassar essa restrição. Passaram 16 novos ensaios
portáveis, os **948 testes Rust (zero falhas, 10 ignorados)** e os
**785 testes JS, TypeScript e Vite** do conjunto. Inclui dois percursos de
transporte sintético que comprovam a recusa de envio/interrupção no conflito.
A correção de hover exige duas observações de posição, impedindo a abertura
apenas por um pet ou uma superfície passar sob o ponteiro. Passou 56 grupos de
browser; a aceitação Wayland mantém-se pendente. O redimensionamento e os restantes percursos nativos
continuam por aceitar. Os resultados da QA 21 abaixo são históricos; a aceitação nativa da 22 continua pendente.

Na QA 21 abriram os cinco chats com o dono correto e acesso à lista lateral.
Ollama, Gravity, Copilot CLI e GitHub apresentaram o compositor disponível.
O Codex abriu a sessão de terminal selecionada, mas recusou o histórico e a
escrita com «O terminal anunciou mais de um turno ativo nesta sessão»; o caso
continua em diagnóstico, sem escolher um turno arbitrário. As capturas de
Copilot/GitHub têm parte do cabeçalho tapada por outra superfície: não constituem
aceitação visual completa. No Spoty foram observados Música, Som e Aplicações,
com dispositivos reais e sem alterar áudio. Escritas num fluxo sintético e a
mudança de ecrã ainda estão pendentes. A .15 permanente continua por instalar;
a .14 está parada e os sete widgets direitos mantêm-se disponíveis.

O lote 11 já passou a apresentação nativa de
aprovações Codex/Copilot. O 12 passou também o email real: um pedido, um rascunho,
três campos corretos e PDF idêntico à origem, sem envio. O pedido anterior foi
manual, conforme confirmou o utilizador. Passou ainda Stop numa nova conversa
sintética Ollama: um envio, uma interrupção, nenhuma decisão, tarefa `stopped` e
pedido original visível no chat flutuante, sem resposta inventada. Passaram
também o cartão compacto com a escrita focada e o fecho/reabertura da mesma
conversa. A persistência do pedido interrompido e do seu estado foi confirmada novamente
no executável 16, após reinício, sem reenviar o pedido.

A **fonte 13 está integrada e compilada**, incluída no executável QA 14:
corrige invalidações de geometria concorrentes, retira a música de Sistema em
favor do Spotify, abre a sessão capturada e distingue medições desconhecidas.
Inclui o histórico público durável v4 e a sua apresentação no chat. Passaram
**709 testes JS, TypeScript, Vite, 887 Rust e 22 de migração**. O RPM 13 conserva
as 424 fontes revistas e o relay oficial; não foi aceite nem instalado.

A **fonte 14 acrescenta a correção da passagem de teclado** e já arrancou
no executável QA isolado. Em Hyprland, o chat usa um toplevel flutuante normal,
com a mesma interface; só a sua própria janela pode ser posicionada, sem focar
outras aplicações. A expansão da central deixa de pedir foco explicitamente.
Passaram **888 testes Rust, 10 ignorados**. Um protótipo nativo passou clique,
duplo clique e clique dentro do chat fora da caixa, alternando escrita real com
o Brave e regressando ao chat. Esta prova é do protótipo, não da aplicação final.
No executável 14 passaram as mesmas três sequências com o Brave e com o
compositor real deste terminal Codex: seis destinos de escrita confirmados,
rascunhos conservados, sem Enter ou pedidos ao modelo. A central ficou por cima
do chat no primeiro abrir e GTK impôs 1100×760 em vez de 760×620. A **fonte 15**
acrescenta a dimensão explícita da janela própria e recolhe a central ao abrir o
chat. A suite completa voltou a passar: **888 Rust, 10 ignorados**; o RPM foi
compilado e preservado sem arranque. A matriz ainda deve incluir
ilha, pedidos e outras aplicações, preservando sempre os rascunhos.

O candidato 15 foi compilado e preservado sem novo arranque. A **fonte 16 está
integrada, compilada e em QA**: a central passa a pedir teclado apenas após interação num
controlo, com o fundo e o hover passivos. A saída recolhe imediatamente a ilha;
os cartões manuais devem acompanhar o conjunto pet/cartão e recolher ao sair de
ambos. Aprovações/perguntas pendentes e o chat expandido conservam o seu ciclo
próprio. O backend 16 passou 888 testes Rust; a interface integrada passou
719 testes JS, TypeScript e Vite. Os 21 grupos de navegador verificaram a
central, os cartões e o pet físico com transporte nativo simulado; não
substituem o ensaio de teclado em Wayland. No executável 16 passaram novamente
as seis sequências Brave/terminal, os tamanhos 760×620 e 1000×800, o arrasto real
e a estabilidade ao passar o rato. O fundo da central expandida permitiu escrita
real no Brave e no terminal sem os voltar a focar; a saída recolheu em 71 ms.
O percurso físico pet/cartão/pet manteve o cartão, mas a saída de ambos não o
recolheu. A **fonte 17 integrada e compilada** acrescenta a observação nativa do
ponteiro no habitat, porque o WebKit pode omitir `pointerleave` ao sair da máscara
de entrada. O ensaio visual real expôs uma corrida na passagem para o cartão:
o cartão estava visível sobre o pet e já não estava ao chegar ao seu interior.
A máscara do pet termina perto de y=48 e o cartão começa em y=50; os sinais
nativos de ambas as superfícies também chegavam em ritmos diferentes. A
**fonte 18 integrada e compilada** aproxima os ritmos para 33 ms e concede
50 ms à passagem. Passaram 723 testes JS, TypeScript, Vite e 888 Rust. No ensaio
nativo com a identidade Codex confirmada, a passagem pet → cartão manteve a
fala; o regresso ao mesmo pet ainda provocou o fecho. A saída de ambos recolheu
o cartão. O ensaio anterior, rotulado Codex mas com Spotify visível, foi rejeitado
como prova de identidade. A aceitação nativa continua pendente, com o retorno
cartão → pet em diagnóstico. O Spotify tem chat pelo pet físico, mas a
central esconde atualmente as ações genéricas de conversa na vista de música;
essa rota não foi certificada como disponível no painel. Na rota física,
«Conversar com este pet» abriu no 16 o chat flutuante Spotify com a identidade
correta e lista de sessões, sem pedidos ao modelo.

A **fonte 19 está congelada, compilada e ativa em QA**, com 427 ficheiros
revistos. Integra a observação nativa conjunta de posição e dono do ponteiro,
a recuperação de reentrada com movimento real e confirmação do mesmo pet, e
a escolha explícita do monitor pela central e pelo habitat. Integra também a
lista de sessões independente das quotas, a rejeição de detalhes de navegação
antiga e os estados acessíveis das definições. Passaram **725 testes JS,
TypeScript, Vite e 888 testes Rust**. Os 35 cenários de ponteiro usam transporte
nativo simulado; falta aceitar o pacote nos ecrãs e aplicações reais. No ensaio
real 19, a passagem pet → cartão → mesmo pet manteve o cartão, mas a saída
de ambos deixou-o aberto, inclusive numa captura posterior. Esta regressão
bloqueia a promoção. O ensaio dos seis chats parou na primeira verificação
porque a lista lateral estava recolhida; não constitui prova dos seis pets.
Não houve escrita, envios ou decisões neste ensaio. A fonte
congelada não inclui os deltas privados Spotify/contagens nem integra os núcleos
Gravity/Copilot ainda isolados.

A **fonte 20 está congelada, compilada e ativa em QA**, alterando apenas a observação
nativa do ponteiro. Usa a janela realmente sob o rato e transforma as coordenadas
do seu filho para o habitat, numa só consulta. A revisão do GTK 3.24.52 instalado
mostrou que a consulta anterior pode devolver ausência quando o foco é a própria
janela, sem ser um filho. Um ensaio isolado da interface 19 reproduziu a sequência
«saída inicial → entrada DOM → ausência de nova saída nativa» com cartão preso;
isto é compatível com a falha, mas não é um registo dos eventos reais do ensaio.
Passaram novamente 888 testes Rust; as restantes 426 fontes, incluindo a
interface validada com 725 testes JS, são idênticas à fonte 19. Falta o ensaio
nativo do novo pacote, incluindo entradas rápidas e a saída do conjunto pet/cartão.

No **executável 20**, passaram novamente as seis sequências reais de escrita:
clique, duplo clique e clique dentro do chat fora da caixa, alternando com Brave
e com o compositor Codex. Não houve Enter, inferência ou alterações dos
rascunhos. O cartão manteve-se na passagem pet → cartão → pet e continuou
aberto depois da saída; foi fechado pelo controlo explícito. Não repetir builds
por hipóteses de ponteiro sem prova adicional. Cinco chats abriram com dono e
controlos de sessões confirmados por acessibilidade, sem alterações públicas;
o cartão Codex sobrepôs-se às capturas e o chat Codex ainda carregava o histórico.
O sexto percurso, Spotify, parou num nó acessível que mudou/ficou oculto. Isto
não certifica prontidão de histórico, interação ou seis chats completos.

O ensaio de monitores na QA 20 confirmou que a central e o habitat passaram
ambos para a origem correta do portátil (0,270 lógicos, escala 2). A central
ficou centrada em x=340. A superfície dos pets, porém, conservou 3440 píxeis
lógicos de largura no output de 1440, inclusive na leitura posterior. A escolha
do monitor está parcialmente aceite; o redimensionamento do habitat continua
por corrigir. O modo externo e a preferência `primary` foram repostos, com o
mesmo hash das definições anteriores. Não houve escrita, envios ou decisões.

No ensaio local seguinte houve **um envio plain a `gemma3:4b`**, conservado como
um par público na conversa sintética. O modelo respondeu com uma apresentação
genérica, sem seguir o marcador pedido; a validação interrompeu-se antes de criar
a conversa B ou clicar na fala. A resposta real foi revista e reconhecida no
manifesto de QA, sem repetir o pedido. A biblioteca QA tem agora 42 mensagens,
das quais as quatro mensagens reais adicionais continuam separadas para a
preservação final. Isto não aceita o destino físico da fala. Rever também a
qualidade do modelo local e a aplicação da identidade/variante pt-PT ao prompt
plain: o caminho atual usa a persona genérica Mochi e a língua do utilizador.

A matriz nativa de seis modos Pets/Sistema passou no executável 17: pets
desligados, só Sistema, só a barra Pets, ambos desligados, pets físicos sem
centrais e configuração completa. O abrigo antigo não reapareceu e os blocos
da barra conservaram o aspeto definido. O código de opções e lease é o mesmo
no 18; a instalação e a retirada final dos widgets direitos continuam pendentes.

**Continuam por concluir:** fecho do cartão ao sair do conjunto pet/cartão, paridade do teclado com decisões/perguntas pendentes, abertura física da fala
para a sessão exata sobre a central expandida, ensaio de mudança de monitor na
fonte corrigida, instalação permanente, preservação revista das quatro mensagens
e duas tarefas Codex reais acrescentadas na QA, reposição do rascunho original e
commits assinados. Os widgets direitos ainda estão ativos. Existem capacidades
de backend adicionais por desenvolver, discriminadas no quadro abaixo; aceitar
esta entrega corretiva não conclui o objetivo geral.

### Histórico dos candidatos anteriores

O histórico seguinte descreve os lotes anteriores. O candidato 8 corrigiu a
navegação que bloqueava a entrada no 7 e a dependência do singleton no habitat. Passaram
833 testes Rust e 664 JS no candidato 8. O lote 9 passou depois **838 testes
Rust, 10 ignorados, e 668 JS**, TypeScript e Vite; o pacote já arrancou. O arranque
e o lease do modo só Sistema passaram;
a apresentação mantém os blocos nativos. Não há instalação permanente .15.
Passaram também PDF e pasta no pet físico Codex e no centro Ollama, com escolha
do dono e cancelamento. O Ollama foi observado fisicamente durante respostas
locais. Faltam a restante matriz e os documentos/decisões; a QA identificou dois
ajustes de interface para o lote seguinte: avatar sobre estatísticas e um RAF
que podia reabrir o cartão depois do fim de um pedido. O lote 9 separa também
a vida do chat flutuante da configuração do habitat: desativar ou recalcular os
pets físicos não deve fechar uma conversa aberta nem invalidar a sua abertura.
Corrige ainda falas que desapareciam quando não havia espaço lateral na barra:
devem poder surgir abaixo dela, junto do pet, com área de clique limitada.
As conversas locais sem ferramentas passam a publicar progresso e resposta no
pet sem criar tarefas ou recibos de execução. Cada fala conserva o destino
Mochi capturado no início do turno, para que «Ler mensagem» abra a conversa
correta mesmo depois de o utilizador selecionar outra. Este delta ainda está
incluído no candidato 9, ainda a aguardar aceitação nativa. A .14 e os dados reais
continuam preservados; um arranque aceite não conclui a entrega completa.

O código pertence ao worktree `coucou-codex`; a integração Fedora pertence a
`sysworkspace-coucou-integrated`. Não alterar o outro fork. O relay oficial
confiado conserva o mesmo SHA. As entregas são registadas em commits assinados
separados da app e da integração; isso não equivale a concluir o objetivo geral.

O candidato 9 **não será promovido**: a revisão encontrou um destino de fala
externa que podia abrir a sessão entretanto selecionada. O lote 10 captura a
identidade original, resolve-a pela lista canónica antes de habilitar o botão
e não abre janelas ao terminar uma leitura atrasada. Passaram os ensaios com
duas webviews independentes, sem broadcast artificial. O lote também retira a
casinha antiga mesmo quando as opções da nova central estão desligadas e limpa
as métricas antes de apresentar uma conversa diferente: respostas atrasadas ou
falhadas não podem atribuir os tokens de A a B. **673 testes JS**, TypeScript e
Vite passaram. Este código ainda não foi testado no novo executável.

A correção backend do email foi integrada no lote 10, separada do trabalho de
Stop: **841 testes Rust passam, 10 ignorados**. Cinco vetores do formatter
passaram pelo parser real instalado do Betterbird, incluindo português, aspas,
vírgulas, percentagens e caminhos de anexos. Isto corrige os escapes literais
dos campos; a duplicação/desaparecimento de janelas continua sem causa provada.
O ensaio nativo deve confirmar um só pedido, um rascunho e o anexo, sem envio.

O RPM do lote 10 já foi compilado e extraído: SHA `3ea6483c7b37…`, 13 ficheiros,
sete do runtime Copilot e relay oficial intacto; corresponde às 415 fontes
congeladas. Ainda não arrancou. Um novo relato de pedido Codex invisível está
em diagnóstico na instância QA 9; os reinícios ficam guardados para preservar
qualquer decisão em curso. A mensagem de espera do hook não prova apresentação
na UI. Não há promoção ou instalação permanente enquanto esta falha não for
compreendida e tratada.

A fonte seguinte, lote 11, integra o aviso canónico de Stop e a sua
migração compatível (13 testes do importador passaram). A confirmação comum de
apresentação exige o cartão efetivamente aberto, a superfície nativa válida e o
mesmo pedido/sessão/pet; só depois habilita a resposta. Fechar ou substituir um
cartão durante a preparação não pode reabri-lo nem responder ao pedido seguinte.
Uma resposta nativa recusada mantém o pedido por resolver com explicação visível.
O lote 10 foi preservado em RPM e num arquivo privado das fontes exatas antes
destas alterações; a instância que corre continua a ser QA 9. O backend combinado
do lote 11 passou **864 testes Rust, 10 ignorados**, e os 13 do importador;
**696 testes JS**, TypeScript e Vite passaram. Os oito grupos do ensaio de
aprovação usam transporte simulado: não concederam decisões nativas e ainda não
provam a apresentação real. As 421 fontes do lote 11 estão congeladas e o RPM
está a compilar para ensaio isolado de aprovação. A revisão visual confirmou o
pedido interrompido no flutuante a 360/760 píxeis, mas encontrou recorte no cartão
compacto após focar a escrita. Este defeito de altura bloqueia a promoção do lote
11; a correção mínima está a ser preparada separadamente. Faltam o pacote final
validado e a sua aceitação nativa.

A auditoria de promoção identificou quatro mensagens públicas e dois trabalhos
Codex reais acrescentados na QA à conversa já existente. O prefixo das mensagens
e os oito trabalhos originais coincidem; há uma preparação privada que preserva
apenas esse acréscimo, sem substituir a conversa selecionada ou incluir canários
Ollama. Ainda não foi aplicada. A instalação exige revalidar ambos os ficheiros
com a app parada, guardar a QA integral e preservar qualquer alteração posterior.

O candidato 11 arrancou na QA isolada, sem reinícios por falha. Passou a
apresentação real de aprovações sintéticas Codex e Copilot: cartão e sessão
observados, recibo nativo aceite e fecho do cliente sem decisão ou execução.
O Gravity externo sem capacidade devolveu o pedido ao terminal. O primeiro
ensaio foi respondido pelo utilizador, conforme confirmou; não é prova de uma
resposta automática. A pergunta legacy Claude recebeu confirmação e terminou
sem resposta, mas não foi aceite visualmente e não representa um teste de pet
desativado. O lote 11 continua bloqueado para promoção pelo recorte do cartão
compacto; o lote 12 limita-se à altura/scroll desse aviso e à fixture de teste.

As provas listadas abaixo pertencem à base instalada .14; os resultados do lote
.15 e as respetivas pendências estão indicados acima e no quadro de backend.

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

## Descontinuação das funcionalidades substituídas

Orientação explícita do utilizador em 19/10: a nova ilha substitui a antiga.
Ocultar a superfície antiga não conclui a migração. No Linux, `pet-control` é
a única central para Home, pets, sessões, tarefas, providers, modelos e Sistema.
O aspeto original do Coucou serve de referência e os renderers úteis podem ser
reutilizados sem iniciar o compacto, a galeria ou os listeners antigos.

Esta regra aplica-se a cada funcionalidade substituída: retirar a implementação
anterior do arranque, dos atalhos e dos listeners; conservar referências úteis de
layout; encaminhar todos os acessos para o dono atual. O critério de aceitação
inclui arranque a frio, reabertura, atalhos e recuperação, para que uma falha ou
preferência antiga não volte a instanciar a superfície retirada.

A revisão .15 em curso cria uma entrada própria `pet-cards.html`: controlador
partilhado e cartões explícitos de chat, documentos, perguntas e aprovações.
A entrada Linux deixa de abrir `index.html`; o painel `pet-manager` antigo não
é criado. O singleton Mochi anterior também deixa de criar a sua janela Linux;
o habitat dos pets configurados é o único renderer de companheiros.
O controlador fechado não tem wake strip nem input region, e não
inicia o poll de cursor da ilha clássica. Um observador de monitores mantém a
adaptação aos ecrãs; a observação de saída do rato existe só no cartão aberto.
Hooks de progresso atualizam o pet e a sessão, sem abrir o centro antigo.
Uma falha da nova central deve produzir um erro recuperável, sem reativar a
implementação retirada. O candidato 8 já confirmou o arranque e parte dos
percursos nativos; a aceitação das correções do candidato 9 continua pendente.

A composição passou 655 testes JS e 824 testes Rust (10 ignorados). Os ensaios
browser observaram a ausência dos elementos e listeners clássicos, silêncio
perante progresso/final/música e entrega explícita ao Ollama com pets ocultos.
Fechar/reabrir conservou o rascunho de email e o pedido de aprovação exato, sem
responder ou enviar. Foi corrigida a primeira edição perdida ao entrar no
formulário de email. Estes ensaios usam uma bridge simulada; não substituem
a aceitação dos eventos GTK, do IPC nativo e da recuperação no executável.

O candidato 6 foi compilado e arquivado, mas não arrancou nem foi instalado.
A revisão encontrou dois guards antigos dependentes de `companionMode`: o
fecho de cartões ao abrir Sistema/flutuante e a seleção do pet num pedido de
aprovação Mochi. No modo só Sistema podiam deixar sobreposições ou apresentar
o contexto do pet anterior. O lote corretivo e as regressões têm de passar
antes de congelar e compilar o candidato seguinte. Nenhuma aprovação foi dada
para reproduzir estes casos.

O lote seguinte encaminha Help para o destino autoritativo sem restaurar ou
trocar a conversa atual. `pet_chat_open` recebe `requestId` opcional e devolve
se a janela foi efetivamente mostrada; a bridge só anuncia a apresentação após
esse resultado. ID, target, prazo e geração são revistos antes/depois dos awaits
e no efeito GTK. O guard conserva a ordem Chat→pending para pedidos Mochi,
serializando Stop/reset; pedidos externos usam a presença nativa exata e não
inferem término pelo ACK Stop. Contenção produz erro recuperável, sem bloquear
a UI ou tratar o pedido como terminado. O guard não responde, consome ou
autoriza execução. As oito regressões novas estão incluídas nos **832 testes
Rust que passaram**; falta revisão do delta e aceitação nativa do novo pacote.

A QA 7 confirmou uma falha adicional antes de executar qualquer JavaScript:
`navigation` recusava a nova entrada `pet-cards.html`. O lote 8 liga a barreira
às entradas reais configuradas, permite cada página apenas na respetiva janela
e recusa as páginas clássicas Linux. Corrige também o habitat, que ainda lia
a capacidade Layer do singleton retirado: a capacidade passa a ser capturada
na criação GTK e lida pela configuração assíncrona, sem consultar GTK no worker.
A prova browser da entrada dist real passou; o novo arranque nativo continua
obrigatório. Nenhum dado real ou permissão foi alterado por este diagnóstico.

| Funcionalidade | Dono ativo | Critério para concluir a retirada |
| --- | --- | --- |
| Compacto, expansão, Home e galeria | `pet-control` | Arranque, hover, atalhos e tray nunca constroem ou abrem a ilha clássica |
| Gestão de sessões, providers e modelos | Central, com componentes de gestão partilhados | Um único painel; abertura explícita funciona com a apresentação passiva desligada |
| Centro AI DMS anterior | Plugin `iaStatus` desativado na fonte e na sessão | O QML anterior não é instanciado; `ia-barra` mantém-se apenas como backend partilhado das operações migradas |
| Chat, perguntas e aprovações | Cartões explícitos e chat flutuante | Um responder por pedido; IDs, sessão, prazo e decisões humanas preservados; fechar não decide |
| Documentos | Pet configurado escolhido explicitamente | Desligar companions não muda o provider para o default global; uma ingestão e uma concessão por drop |
| Spotify | Pet Spotify e controlos MPRIS | Uma rota por ação, sem revelar o compacto antigo nem anunciar chat inexistente |
| Definições | Hub de configuração atual | Uma gravação autoritativa, preservando preferências e erros |
| Sistema | Nova ilha e ponte DMS | Controlos antigos retirados após paridade; sem duplicação na sessão normal |

O fallback **nativo DMS** por perda de lease é um mecanismo de continuidade
quando o Coucou fecha ou falha. Conserva os controlos essenciais do computador;
não inicia nem reativa a ilha Coucou antiga. A sua retirada não faz parte desta
orientação. A migração do bloco direito só será aplicada após a nova composição
passar a matriz de disponibilidade, documentos e decisões nativas.

## Backend ainda por desenvolver

O quadro distingue código em falta, código privado ainda por integrar e
capacidades implementadas ainda por aceitar/instalar. A .16 permanente é o
executável atual; as propostas privadas seguintes não estão instaladas. Passar
ensaios offline não equivale a entregar uma capacidade nativa.

**Próximos passos da instalação permanente:** a .16 já está ativa. A revisão
exata preservou seis novas mensagens e duas tarefas reais, com o par Ollama
posterior arquivado no mesmo CID de origem e sem importar o prefixo sintético.
O rascunho original foi reposto e confirmado sem envio; faltam os commits assinados.
Os sete widgets direitos foram repostos após a clarificação do utilizador.
A migração definitiva exige UI própria completa e preservação dos indicadores. A barra mantém blocos e intervalos transparentes; não aplicar a fonte
antiga de monitores sobre alterações de outras sessões.

**Sessões e observabilidade:** a fonte 22 já contém o resumo por provider,
consulta individual além das 12 linhas, listas completas, quotas, modelos e
memória. As leituras usam apenas clientes já ligados e configuração existente,
sem reconexão ou criação de defaults. O histórico Codex com múltiplos turnos
ativos pode ser lido, mas envio e interrupção ficam bloqueados até existir um
alvo inequívoco, também na chamada direta. A QA 22 mostrou histórico de uma
sessão externa real; o caso root originalmente ambíguo ainda não foi repetido.
A lista instalada já publica as origens que responderam, com carregamento/erro
separado, sem esperar pela fonte lenta. A captura QA 22 é histórica e não prova
um bloqueio infinito. Os limites de paginação
do backend já foram integrados na fonte 24: prazo assíncrono total de 30 segundos
e limites cumulativos de páginas, linhas, cursores e leituras. Falta aceitar o
pacote revisto no desktop; um timeout nunca representa ausência de sessões.

O ledger Ollama está instalado na .15; o estado inicial da QA era vazio. Regista apenas tentativas feitas pelo
Mochi desde o seu início, com origem imutável e fila limitada; não mede clientes
externos nem reconstrói consumo anterior. Falhas de fila/escrita marcam lacunas,
sem repetir inferências; desconhecido não significa zero. A primeira versão
ainda não tem rotação automática além do limite de 4096 tentativas/4 MiB.

**Interação e monitores:** as seis sequências reais de teclado Brave/terminal
passaram na QA 20. A QA 22 observou a adaptação 3440→1440 no mesmo processo,
quando outra intervenção mudou para o portátil; falta o percurso controlado
nas duas direções. A saída de um cartão manual pode deixá-lo aberto; existe
fecho explícito e fica pendente, sem novas compilações por hipótese. A revisão
identificou uma retenção global de decisões, mas ainda não a ligou ao episódio
real. A abertura espontânea foi reaberta por pedido posterior: a fonte 22 exige
duas observações de posição dentro da mesma entrada e passou 56 grupos de
browser. Falta a confirmação Wayland; aprovações reais de outras sessões não
são uma abertura espontânea e conservam a decisão humana.

**Spoty só para áudio:** a matriz é cinco pets com chat e Spoty para música e som
do PC. Conservam-se ID e histórico anteriores sem acesso LLM nos caminhos novos.
Música, Som e Aplicações mostraram dados reais. A QA 22 não chegou às escritas por um erro do seletor. A QA 23 passou mute,
unmute e volume 31% num fluxo silencioso próprio, com leitura independente e
limpeza confirmada. A mudança de perfis/dispositivos e a música autenticada
ainda não foram aceites.
A quota Codex ainda mostrada no tooltip vem de uma associação antiga, com
correção pequena pendente e sem bloqueio de áudio demonstrado.

**Identidade Ollama:** o diagnóstico confirmou a antiga instrução fixa “You are Mochi”
no texto plain/Ask e a ausência de regra pt-PT no caminho local. A query atual é
acrescentada exatamente uma vez pelo construtor do pedido; não a duplicar no
histórico. A correção v2, integrada e compilada na fonte 24, captura o nome
configurado e a identidade da conversa antes de esperas, preservando ferramentas,
modelos e idioma pedido. Falta a aceitação funcional do pacote revisto.
Isso não prova a causa da resposta QA 16 nem garante a qualidade de um modelo.

**Observabilidade ainda incompleta — confirmação de 20/10:** o ledger de tokens
só está ligado ao Ollama, e só desde o início da captura. A QA 23 ainda descarta
o estado `noObservations`, perdendo a explicação e a data do início. A fonte 24
passou a mostrar essa situação e a data, sem inventar consumo zero.
Os novos cartões CLI não têm totais de tokens ligados. Os três contadores principais
estavam sempre ligados à fonte nativa, mesmo quando a biblioteca do pet já tinha
valores confirmados. A correção 31 integrada apresenta a fonte disponível com o seu
âmbito explícito e manterá o estado externo separado, sem somar duplicados. As quotas nativas já
consultadas no gestor legado/migrado são dados diferentes: a fonte 24 já contém
uma secção própria nos cartões, com fonte, período, frescura e ligação ao detalhe.
Não converter a última resposta em total nem percentagem de quota em tokens.

O inventário Codex só consulta um cliente já ligado e verificado na aplicação.
Depois de reiniciar, pode apresentar indisponível antes de abrir uma sessão;
essa limitação não equivale a não existirem terminais. A próxima proposta de
consulta passiva deve avaliar o inventário de metadados já usado pelo gestor,
com cobertura explícita e identidades revistas. Não voltar a usar a ligação
principal que cancela/reconstrói o estado de decisões só por abrir um cartão.
A UI 31 e a lista 30 já estão integradas na fonte 24 e passaram 823 testes JS
no conjunto. O RPM foi arquivado, mas a correção adicional do caso Copilot parcial
vazio e a aceitação nativa ainda precedem a entrega; os produtores
de tokens CLI e o índice passivo continuam por integrar.

**Gravity gerido ainda não conserva todas as aprovações interativas:** a revisão
privada concreta passou 190 testes portáveis e compilou com o SDK Tauri real;
o seu modo headless publica `approvals:false`. A documentação do CLI indica que
ferramentas em Ask podem ser recusadas sem prompt interativo, mesmo com exit0;
as ferramentas já permitidas, incluindo o workspace, são um caso distinto.
O transporte privado também desliga o gate Coucou. A UI nova pode expor conversa
e tarefas já autorizadas, mas não satisfaz ainda a exigência de capacidades CLI
completas e cartões de aprovação. Não ativar grants, force ou permissões mais
amplas para contornar. É necessário um percurso nativo suportado e aceitação de
ferramentas/negações, além do canary de texto, antes de declarar essa capacidade.

**Voz Ollama e execução geral:** a conversa local de voz usa ainda o Brain com
schema de ações limitadas; a execução geral aprovada está no runner Ask do chat.
A correção de identidade preserva essa política e não a transforma em execução
arbitrária. Falta desenhar a entrada de pedidos de voz no runner existente com
o mesmo owner/CID, aprovação, Stop e confirmação de receção. É uma lacuna de
funcionalidade separada da qualidade STT/TTS adiada; não anunciar paridade geral
voz/texto do Ollama antes de integrar e aceitar esse percurso.

**Desenvolvimento ainda aberto:** adapter nativo Gravity e sessões externas,
continuidade Copilot, recuperação Codex e ferramentas dependentes do cliente,
ciclo completo das concessões de documentos, pesquisa/reprodução Spotify
autenticada, pasta de trabalho Ollama na UI e perfil rápido local com ferramentas.
Qualidade Bluetooth PT/EN, STT/TTS/Raquel e Notes continuam na fase posterior
acordada. Novos núcleos privados não são capacidades do executável atual.

| Prioridade | Capacidade | Estado e próximo trabalho |
| --- | --- | --- |
| Instalado na .15; QA 11 aceite | Apresentação de aprovações Codex/Copilot | O ACK agora exige apresentação nativa efetiva e identidade/prazos exatos; a resposta só confirma sucesso após entrega. Passaram os cartões sintéticos reais de ambos, com recibo aceite e fecho do cliente sem decisão. O utilizador confirmou os cliques nos primeiros cartões; isso não identifica o ator dos Allows históricos. O episódio original não foi capturado durante a espera. Preservar decisões em curso e revalidar o pacote final. |
| Aceitação e instalação | Disponibilidade do Sistema e recuperação da barra | Ponte 2.1 ativa; relógio nativo recuperou sem app e a ilha retomou o espaço ao arrancar. A QA 8 passou arranque, lease por 269 s e slow-hover no modo só Sistema; documentos passaram no centro e no pet físico. A matriz completa dos seis modos passou na QA 17. A QA 23 está ativa; a revisão 22 observou a adaptação para o portátil sem reinício. Falta o percurso controlado bidirecional e a paridade final antes do corte direito. O cartão manual preso conserva fecho explícito e é acompanhado separadamente. Passaram 27 testes da ponte, 838 Rust e 668 JS. |
| P0 reaberto por clarificação | Sistema com interface própria na ilha | Os atalhos nativos estão instalados e ensaiados, mas não cumprem a UI própria pedida. Corte dos sete widgets revertido; apps, periféricos, CPU/componentes, clipboard, bateria e notificações ficam. Ponte 2.1.1 condiciona retirada seletiva apenas do controlCenterButton a capacidades próprias ainda por implementar. |
| P0 | Documentos na nova central | A base candidato 2 passou PDF/pasta, anexo/remoção e Cancelar/reabrir sem nova ingestão. Na QA 8 passaram PDF e pasta no centro Ollama sem companions e no pet físico Codex, com dono correto e cancelamento. A correção 9 mantém estes percursos; falta a aceitação integrada de email/decisões antes de instalar. |
| P0 | Executor geral local para Ollama | Implementado e revisto na .15: aprovação única, pasta capturada, cancelamento/prazos, saída limitada, progresso e receipts por conversa. A retoma Qwen3.5 passou com aprovação humana e execução única comprovada. Stop no cartão pendente também passou: `cancelled/userCancel`, sem arranque nem saída. A capacidade está instalada na .15; os 838 testes pertencem ao lote que a introduziu. |
| P0 | Interagir com sessões Gravity já abertas | Conversa Mochi própria funciona; chat externo ainda indisponível. O núcleo isolado passou 142 testes offline; ciclo de vida passou 156 e os comandos privados passaram 173, incluindo 17 novos. O adapter Tauri concreto está em desenvolvimento privado. A preparação corre fora da UI, Exit impede publicação tardia e não há fallback em memória. São testes sintéticos; o prazo de Exit não certifica um limite estrito de IO/cleanup do sistema operativo. Não está registado no binário .15 e ainda não prova IPC, retoma ou aprovações nativas. Não confundir criar um owner novo com ligar a uma TUI antiga. |
| P1 | Continuidade Copilot ao fechar a TUI | A extensão atual termina com o cliente que a aloja. Núcleo isolado: 75 testes; B0: 81; custódia direta: 61. Dois ensaios limitados de versão não iniciaram SDK/conta/modelo. O protótipo privado C1 executou uma unidade transitória própria: passaram nascimento simples e recolha de órfão, mas o caso bloqueado falhou; quatro filhos foram recolhidos e o fallback final não foi exercitado. C1 não passou, não teve repetição e não habilita SDK, IPC ou continuidade nativa. Modelo e decisões de sessões externas continuam no terminal. |
| P1 | Continuidade Codex completa | Sessões geridas com daemon independente já têm API, UI e prova sem IA. Ferramentas dinâmicas/MCP arbitrárias e recuperação de perda de ligação ainda exigem o cliente nativo ou nova integração comprovada. |
| P1 corrigido, lote 10 | Destino das falas de sessões externas | A QA 9 confirmou mensagem Mochi A → seleção B → abrir A sem trocar B. A falha separada dos snapshots de terminal foi corrigida no lote 10: identidade capturada, consulta canónica antecipada e botão indisponível até resolver. Uma leitura atrasada não abre janelas; uma decisão B conserva o próprio contexto. Ausência de identidade abre apenas a lista de sessões. Prova em duas webviews passou, sem IA/execução; falta o novo executável. Bloqueia a promoção do 9. |
| P1 corrigido, lote 10 | Métricas da conversa correta | Limpar contagens antes de desenhar uma nova CID; só aceitar resposta da geração/provider/CID ainda válidos. Passaram A→B pendente, erro B, C atrasada depois de D e Ollama atrasado depois de selecionar Codex; falta aceitação nativa. |
| P1 encontrado em QA 11 | Mudança de monitor com a app aberta | Durante a troca real do portátil (1440×900 lógicos, escala 2) para DP-5 ultrawide (3440×1440, escala 1), a central reposicionou-se, mas a superfície dos pets conservou 1440×320. O drop físico foi recebido e ingerido pelo Ollama, mas o cartão apresentou só o cabeçalho e não mostrou as ações. Preservar a prova; rever atualização de geometria e ciclo de vida/entrega após hotplug. Um ensaio isolado reproduziu a perda de invalidação de layout que chega enquanto a configuração anterior aguarda; a conclusão antiga substitui a invalidação. A fonte 13 já corrige a invalidação concorrente e ignora o resultado antigo, incluindo falha, sem desativar a superfície atual; três regressões passaram. A fonte corrigida foi compilada, mas o ensaio QA 18 ainda falhou: a central usou medidas do portátil no output externo e o habitat reteve 3440×320. Os filhos seguiam o monitor da janela de cartão oculta; a fonte 19 seleciona o monitor explicitamente pela origem lógica, ainda sem aceitação nativa do hotplug. Não atribuir o episódio inteiro à corrida anterior nem contar o drop como email aceite. |
| P1 | Ciclo de vida dos acessos a documentos | Completar validade/revogação/limpeza das concessões e explicitar o vínculo às conversas. Seleção de partes de uma pasta é uma melhoria adicional. |
| Instalado na .15; QA 12 aceite | Email no cliente real | PDF físico pela ilha chegou ao Ollama. Um Open automatizado gerou um pedido/launcher e uma janela de composição Betterbird, com destinatário, assunto Unicode/aspas/percentagens, corpo e um PDF de 634 bytes cujo hash coincide com a origem. A janela sintética fechou, sem Send. O utilizador confirmou um Open anterior manual; não o contar como duplicação automática. Campos originais foram guardados privadamente com autorização antes do ensaio. A causa do episódio antigo de duas composições não foi provada. Email de pasta não está implementado nem deve ser oferecido. |
| P1 em aceitação | Spoty dedicado à música e áudio do PC | Integrado na QA 21, sem acessos LLM, mantendo ID e histórico. Música, Som e Aplicações foram observados com dados reais, sem alterar áudio. A QA 23 passou mute, unmute e volume31% num fluxo silencioso próprio, confirmado por leitura independente e removido no fim. Faltam os restantes controlos; pesquisa/reprodução autenticada não se prova pela abertura do painel. |
| .17 instalada; cinco cartões e duas rotas locais aceites | Sessões e observabilidade nos cartões | Census/histórico e UI39 instalados; galeria/residentes distinguem CLI e biblioteca. 1061 Rust (10 ignorados), 866 JS, TypeScript/Vite. Leitura nativa dos cinco cartões e abertura da biblioteca Ollama/Gravity preservaram journal e rascunho, sem inferência/conta/decisões. Tokens CLI, identidade/controlabilidade e rotas para o gestor mantêm aceitação separada. |
| P1 por desenhar | Pedidos de voz Ollama com execução geral | Brain local só permite ações limitadas; ligar ao runner Ask aprovado com identidade/Stop/prazos originais, preservando conversa. Qualidade de áudio continua separada. |
| P1 · FE41 privado revisto | Definições e quotas sem efeitos tardios | Bootstrap conserva o último pet; Reload confirma descarte de rascunhos; aplicar perfil após guardar respeita navegação/visibilidade. Abrir/atualizar o gestor passa a ler só o cache; atualização da conta exige o botão explícito e singleflight. Parser mantém null/desconhecido e data original. 872 JS/40 grupos browser, TSC/Vite; sem aceitação nativa ou instalação. A escolha de modelo com ACK tardio ainda pode reabrir Ollama: P1 separado. |
| P1 | Pasta de trabalho Ollama na UI | `assistant_set_workspace` já suporta Ollama; o chat clássico e o flutuante ainda condicionam o seletor aos providers CLI. Expor a operação existente com confirmação e atualização da conversa, sem trocar o destino de um trabalho em curso. A composição .15 foi reaberta para a descontinuação; esta lacuna de UI continua registada. |
| P1 | Perfil rápido Ollama com execução | O default revisto `gemma3:4b` serve conversa e não anuncia tools; `ask` não lhe acrescenta essa capacidade. A execução nativa passou com Qwen3.5 4B escolhido explicitamente. Avaliar conversação/latência antes de rever o perfil rápido; mostrar sempre a capacidade do modelo exato e nunca trocar de modelo/provider silenciosamente. |
| Instalado na .15; QA 16 aceite | Continuidade visual depois de Stop | Aviso vinculado à tarefa/CID exatas, sem resposta inventada ou contexto extra para o modelo. Na QA 12 passaram flutuante, cartão compacto com escrita focada e fecho/reabertura. A QA 16 confirmou o pedido original visível e a tarefa `stopped` inalterada depois de mudar de executável e reiniciar. Zero decisões ou novas mensagens nesse ensaio. ACK Stop não prova limpeza de processos. |
| Aceitação parcial na QA 16 | Histórico completo de pedidos interrompidos | Backend, migração v4 e UI integrados e compilados. O pedido legacy interrompido apareceu no chat depois do reinício, identificado como recuperado e sem posição relativa inventada. O envio plain posterior escreveu v4, conservou esse registo e o par público real; a apresentação combinada após esse envio ainda exige aceitação. Posições capturadas exigem mensagens públicas originais, pares reais e ausência de sobreposição; rejeitar antes de normalizar. Sem contexto extra para o modelo nem retoma automática. Passaram 888 Rust, 719 JS e 22 de migração. |
| Implementado na .15; inferência por aceitar | Identidade e português de Portugal no Ollama | V2 captura dono/CID/nome antes de esperas e inclui nome configurado e pt-PT, respeitando língua explicitamente pedida. 12 regressões e suite conjunta aprovadas. Não houve nova inferência nativa para validar qualidade; conservar a resposta inesperada QA 16 como prova anterior. |
| Implementado na fonte 23; por aceitar | Controlos de disponibilidade da ilha | Definições apresenta os dois controlos independentes. Comando restrito à janela nativa settings, atualização por campo e proteção contra gravações antigas; eventos/readback confirmam os valores sem tratar defaults desconhecidos como confirmados. Passaram 795 JS, TypeScript e Vite; a suite Rust conjunta passou 961 testes (10 ignorados). A QA 23 já executa o novo binário; o ensaio nativo destes botões ainda está pendente. |
| P2 | Densidade e contraste das falas | Na QA 9, o balão físico e a CTA para a CID exata passaram com dois pets na faixa estreita. Rever cabeçalho comprido de tarefas (query/pasta) e contraste sobre janelas com texto; manter identidade da sessão e detalhes acessíveis sem tornar a fala pesada. A oclusão com a central expandida ainda exige ensaio. |
| P2 | Recolher a central ao abrir conversa | No ensaio browser, Conversar na galeria pode deixar a central expandida até o rato sair. Recolher apenas depois de a janela confirmar abertura e se o contexto continuar a ser o mesmo; não recolher uma nova navegação por um ACK antigo. Fora do lote 10. |
| P2 | Estado meteorológico no compacto | O painel nativo completo funciona. A leitura resumida ainda não inclui a cidade consultada ou a hora da observação; fechar contrato da fonte antes de acrescentar esses campos, sem inventar localização ou atualidade. |
| Adiado | Qualidade de voz e notas | Qualidade real Bluetooth PT/EN/mista, latência até Raquel e persistência Notes/voz; conservar o que já existe. O controlo de áudio do PC pertence ao Spoty e está no lote atual. |

### Registo histórico da primeira QA .15

Este registo descreve o primeiro ensaio; o estado de entrega no início deste
documento identifica o executável e os monitores atuais.

A app permanente continua na .14, fonte assinada `23688cc`; a integração base
do anfitrião está no commit assinado `642d74b`. A barra com blocos, título
truncado e folgas foi aceite no portátil, incluindo galeria e os cinco painéis
reais (0,45–0,53 s). O ultrawide está fisicamente desligado; não afirmar um novo
ensaio nativo nesse monitor.

O RPM .15 foi compilado e verificado com os 13 ficheiros esperados, sete do
runtime Copilot e o relay oficial intacto. Está em execução na unidade transitória
`coucou-qa15i.service` (candidato 9), com config/data/cache privados e autostart
desativado. O RPM atual tem SHA `2386bc90de92…`, ligado ao manifesto de 413
fontes revisto; a instalação permanente continua pendente da aceitação completa.
Os quatro ficheiros de dados reais continuavam iguais ao backup no arranque QA.
Launcher e login ainda apontam para a .14; o prefixo permanente .15 não foi aplicado.

- A ponte **MochiSpace 2.0** já lê os estados reais de rede, Bluetooth, bateria,
  som, gravação, clipboard, discos e quatro ícones de aplicações nesta sessão.
  `pet_system` tem tipos limitados, percursos nomeados e ações sobre handles de
  tray exatos. A UI .15 está em implementação; o bloco direito antigo permanece
  até a paridade nativa e a instalação serem confirmadas. O novo host usa os
  componentes DMS originais para preservar menus, discos e gravação.
  Os oito painéis e o menu de uma aplicação já mostraram conteúdo real correto;
  a bateria fica abaixo do compacto. A âncora de discos e menus de tray foi
  corrigida com geometria explícita antes do layout: ambos abriram centrados
  no ensaio nativo posterior, com as três ações de abertura confirmadas.
  Fechar o tray por botão, Escape ou clique fora e fechar o menu por Escape/fora
  devolveu o teclado ao editor sintético nos cinco ensaios reais. Nenhuma decisão
  de agente foi respondida por automação.
  No RPM .15 em QA isolada, os treze acessos passaram pelos botões reais e
  recolheram o cartão da ilha após o ACK, sem tapar os painéis nativos.
  A revisão antes do corte encontrou uma dependência adicional: o IPC nativo
  de definições rápidas exige o botão antigo, que a migração vai remover. A
  ponte passou a abrir diretamente o mesmo componente nativo, independente
  desse botão. Os 819 testes Rust voltaram a passar e os 18 testes da ponte
  incluem a ausência do botão/grelha antiga. O atalho existente Super+A conserva
  o abrir/fechar pelo mesmo host; 480 testes do anfitrião passam.
  O terceiro candidato inclui esta
  correção; falta confirmar o percurso depois de retirar os duplicados.
- O carregamento a quente do novo ficheiro QML encontrou o cache da listagem de
  tipos do motor. Reiniciar apenas `dms.service` resolveu; `readiness` confirmou
  componente pronto. Coucou, conversas e rascunho ficaram abertos.
- O drop da nova central foi reproduzido como falha na .14, mesmo recebendo o
  sinal GTK. O frontend .15 implementa receção, seleção e fixação do dono,
  cancelamento e retry sem duplicar a ingestão. O primeiro RPM .15 entregou o PDF
  ao Ollama e mostrou as ações corretas, mas a galeria tapou os botões. Esta
  falha P1 foi corrigida no frontend: eventos de rato atrasados não podem abrir
  uma galeria recolhida. A regressão browser passou e o segundo RPM .15 está
  passou a repetição nativa: PDF e pasta com opções visíveis, anexo no flutuante
  Ollama, remoção do PDF, Cancelar a pasta e reabrir sem nova ingestão. O compacto,
  relógio/data e hover também passaram. O terceiro candidato conserva esse TS;
  a instalação final aguarda as restantes verificações.
- `local_tasks` implementa o executor Ollama com aprovação única por comando,
  journal limitado por conversa e retoma explícita com novas aprovações. O modelo,
  pasta, política, turno e prazo são capturados antes da execução. Reiniciar
  marca um arranque inconclusivo como desconhecido, sem repetir comandos.
  A revisão independente e a suíte integrada .15 passaram: **819 Rust, zero
  falhas, 10 ignorados**. Falta aceitação com modelo local e decisões humanas;
  não está instalado nem autorizado a executar automaticamente.
  A primeira canary nativa Qwen3 1.7B terminou na validação dos argumentos,
  antes de qualquer pedido de aprovação ou comando. O erro ficou legível na UI.
  Uma sondagem só de propostas no Qwen3.5 4B local gerou o comando exato; o novo
  ensaio na app usa esse modelo escolhido explicitamente. Não flexibilizar o
  parser nem confundir uma resposta textual com execução comprovada.
  O cartão nativo Qwen3.5 mostrou o `/usr/bin/printf` e o argumento esperados,
  mas expirou aos 120 segundos antes do clique humano. O receipt confirma
  `expired/approvalDeadline`, sem arranque nem saída. A retoma é explícita na
  tarefa original e exige uma nova aprovação; a expiração não conta como passe.
  A retoma original foi depois aprovada pelo utilizador: a tarefa sucessora
  manteve a CID e terminou com um único receipt `exited`, código 0, 13 bytes de
  stdout e nenhum stderr, com resultado literal `MOCHI_QA15_OK`. O executor
  nativo passou esse percurso. A revisão seguinte interrompe logo a ronda ao
  expirar a decisão, para conservar o motivo real sem nova resposta do modelo.
- O perfil rápido/raciocínio Ollama conserva agora o esforço da seleção e envia
  `reasoning_effort` apenas quando os metadados do modelo confirmam thinking.
  O mesmo esforço vale em todas as rondas e na retoma. Gemma sem tools continua
  como conversa no modelo escolhido; não há troca implícita de provider.
- A revisão pré-corte encontrou dois problemas de disponibilidade: companions
  desativado ocultava também o Sistema; fechar o Coucou deixaria o relógio e os
  controlos retirados. A correção .15/ponte 2.1 separa os estados e usa um lease
  renovado só pela ilha visível, com identidade por ativação. Status/readiness
  de diagnóstico não reclamam a barra. Desligar Sistema ou sair retira o lease;
  após falha sem saída limpa, o watchdog nativo restaura os controlos em 20 s.
  Preserva os blocos, a intenção aprovada entre arranques e alterações de layout
  personalizadas. Pedidos atrasados de uma ativação retirada não podem reclamar
  novamente a barra. Com só Pets ativo, a ilha usa a altura real da banda nativa
  mais 4 píxeis para não tapar o relógio restaurado; Sistema ativo conserva a
  posição original no centro superior. Os 27 testes da ponte, **823 Rust** e
  **489 testes do anfitrião** passam; a repetição nativa é ainda necessária.
  O candidato 4 confirmou Stop sem aprovação/spawn e o relógio do modo só Sistema
  após 81,9 s. Porém, a ilha clássica ainda podia ocupar a mesma âncora ao receber
  eventos de terminal. O candidato 5 suprime o compacto/hover/lançamento clássico
  nesse modo sem ativar pets, mas revelou perda da cadeia de configuração da
  central e foi rejeitado. A orientação posterior exige retirar o runtime
  clássico, em vez de apenas o ocultar. O habitat reserva a ilha mesmo quando o
  controlo Pets está desligado. A base passou **824 Rust**; a nova composição
  de cartões próprios e central única está em implementação e ainda sem aceite.
- O preparador de migração já conserva os receipts limitados das tarefas Ollama
  e recusa referências cruzadas, resultados privados e dados inválidos: **10
  testes passam**. Nenhum journal real foi alterado nesta revisão.
- A UI prepara ainda recuperação de falhas ao fechar/Escape/redimensionar,
  seleção imediata no roster, paginação de sessões extensas e adaptação ao ecrã.
  O frontend congelado passa **638 testes JavaScript, TypeScript e Vite**;
  browser confirmou Sistema, erros/estado desatualizado, indicador de gravação
  e larguras 320–1440. Isto não substitui o ensaio nativo do novo executável.

Antes de instalar .15, preservar novamente os campos não enviados e fazer
backup privado das preferências e do journal. A .14 não reconhece tarefas de
execução Ollama: um rollback não pode abrir silenciosamente esse journal numa
versão antiga nem substituir conversas novas por um backup antigo.
O backup pré-atualização inclui preferências, pets, journal e preferências Flow,
com recibo de integridade privado. O rascunho de 73 caracteres e o CID foram
confirmados novamente antes de autorizar o encerramento da .14.

## Entidades e capacidades

Só aparecem os pets configurados: Codex, Gravity, Copilot, Ollama, GitHub e
Spotify. Há um pet por provider ou entidade. Modelos Gemini/Claude oferecidos
pelo Gravity são escolhidos dentro desse mesmo pet.

| Pet | Conversa e execução | Dados e integrações |
| --- | --- | --- |
| Codex | Conta CLI nativa, ferramentas/aprovações originais, memória própria Mochi | Sessões externas Codex com alvo exato e histórico público |
| Gravity | Conta CLI nativa e os seus modelos, conversa própria Mochi | A ligação interativa a terminais Gravity existentes continua indisponível |
| Copilot | Conta CLI nativa por ACP, catálogo, esforço, ferramentas e decisões | Ponte na mesma TUI ensaiada; loader global .14 instalado, requer recarga/reabertura das TUIs anteriores |
| Ollama | Modelos locais e ações limitadas instaladas; executor escrito .15 implementado, ainda em aceitação | Documentos e rascunhos de email; manutenção exclusiva de modelos/serviço |
| GitHub | Integração da entidade e conversa própria via Codex, identificada como tal | A sessão da entidade não reutiliza o diálogo do pet Codex |
| Spotify | Abertura/controlo determinístico da app; sem chat AI na capacidade atual | Play/pausa, avanço, pesquisa e reprodução por URI disponível |

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
A fonte 14 usa um toplevel flutuante normal no Hyprland, com restrições nativas
que o mantêm fora do tiling e posicionamento limitado à janela própria. Isto
substitui a camada interativa usada até ao candidato 13, que podia reter o teclado
quando se clicava na janela anteriormente ativa. A ilha conserva layer-shell;
os outros compositores mantêm o caminho existente. Fechar o chat só oculta a
janela, sem cancelar a execução. A aceitação da alteração 14 está por fazer.

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


## Auditoria contínua de UX e propriedade — 19/10

Esta secção acompanha os requisitos adicionais do utilizador e complementa o
quadro de entrega. Não transforma código preparado em funcionalidade instalada.
Cada lacuna conserva dono, dependência e prova necessária; notas históricas que
anunciem uma capacidade mais ampla não substituem estes critérios.

**Propriedade por entidade.** O pet é o dono das aplicações, dados, tarefas e
controlos que representa. Spotify, faixa, artista, reprodução e controlos de
música pertencem ao pet Spotify. A página Sistema não deve duplicar esse cartão
nem encaminhar o utilizador para multimédia DMS. O som global de saída/entrada
continua a ser uma definição do computador. Outros leitores de música precisam
de um dono explícito antes de criar UI; não assumir que toda a sessão MPRIS é
Spotify. A capacidade atual do pet Spotify é MPRIS, sem chat AI anunciado. Esta
regra não introduz nem testa voz, fala ou ditado novos.

| ID e percurso | Necessidade de frontend/UX | Dependência e dono | Estado/prova para fechar |
| --- | --- | --- | --- |
| UX-01 · Sistema → música | Retirar reprodução/faixa/controlo duplicado do Sistema; conservar o cartão e ações no pet Spotify. A meteorologia deve aproveitar o espaço sem coluna vazia. | Frontend; APIs existentes, sem remover contratos da ponte ainda usados por compatibilidade. | Delta privado em preparação. Testar Sistema com reprodução ativa/inativa e Spotify ausente; no pet, confirmar estado e controlos. |
| UX-02 · Abrir pet → visão geral | Mostrar contagens separadas de trabalho em execução, à espera de resposta, disponível e por confirmar; distinguir conversas guardadas de sessões vivas. Dar acesso direto à lista filtrada e ao chat escolhido. | Backend/coordenação: resumo canónico por pet/provider/origem/sessão, autoridade e frescura; frontend: composição e navegação. | Reproduzido em browser: catálogo anuncia três sessões e cartão do centro mostra zero. O centro não recebe as linhas por sessão que o renderer consulta. Não substituir a lacuna por um zero inventado. |
| UX-03 · Fechar terminal | Confirmar se processo/cliente/trabalho continua, manter o pet correto e permitir acompanhar/interromper quando suportado. Fechar janela não equivale a encerrar trabalho. | Backend/QA: lifecycle real por provider, PID/unidade/cliente e capacidades; frontend: estados explicados e ações guardadas. | Relato real do utilizador comunicado à coordenação. `PetSessions.merge` não encerra uma entrada ausente da leitura seguinte; falta contrato autoritativo de ausência/desconhecido. Não introduzir timeout cego para declarar encerramento. |
| UX-04 · Abrir sessão | Usar o chat expandido como abertura principal para origens terminal e Mochi; terminal é opção explícita. Capturar a identidade selecionada antes da ação e conservar rascunhos por destino. | Frontend com `pet_chat_open`/destinos já existentes; backend conserva autorização e capacidades reais. | Há inconsistência confirmada por revisão: o mesmo link do cartão abre terminal no controlador e chat no centro; Conversar pode ignorar a sessão selecionada. Preparar delta sem criar/importar/retomar sessões implicitamente. |
| UX-05 · Gerir/encerrar | Separar Interromper turno, Fechar chat, Retirar da gestão, Encerrar sessão e Arquivar histórico; mostrar efeitos e confirmação só quando aplicável. | Backend expõe efeitos e capacidades exatos; frontend usa nomes correspondentes e não oferece operações inexistentes. | Testar cada efeito em sessões sintéticas, incluindo externa não ligada, trabalho em curso e subagentes. Arquivar ou retirar gestão não prova fim de consumo. |
| UX-06 · Tokens/tempo/quota | Associar métricas à conversa e resposta medidas; dizer Última resposta, modelo medido e duração. Quota da conta e tokens de inferência são âmbitos diferentes. Falha/ausência não vale zero, leitura antiga precisa de indicação. | Backend `pet_usage`/quota com âmbito e frescura; frontend já tem campos medidos, scope, model e updatedAt. | Guardas A→B/resultado atrasado passam. Falta rever rótulos, erro de medição e última leitura no mesmo CID; não anunciar total da sessão com a medição da última resposta. |
| UX-07 · Sistema próprio | Implementar os controlos de rede/Bluetooth, áudio, ecrã e energia na própria ilha. Manter na barra apps/tray, periféricos, CPU/componentes, clipboard, bateria e notificações; a ilha pode consultar esses estados sem retirar os ícones. | Backend/host: estados e ações tipadas por função, autenticação própria e retorno de resultado; frontend: controlos próprios. | Só retirar controlCenterButton após paridade e aceitação completas. `open(panel)` atual é transitório e não satisfaz essa condição. Transporte de leitura e DTO estão em protótipos privados; ações e UI ainda pendentes. |
| UX-08 · Compacto e saída do rato | Pets na linha superior, horas/data por baixo, contorno e largura ajustados. Recolher ilha/cartões ao sair o rato com tolerância de edição; chats expandidos permanecem abertos. | Frontend; manter identidade e autoridade das aprovações ao recolher/reabrir. | Pedido não prioritário anterior, preservado. Testar hover lento, lacunas entre superfícies, teclado exterior, decisão pendente e foco de edição, sem reativar ilha clássica. |
| UX-09 · Pet físico → documento | Drop e feedback no próprio pet, área expandida estável, um dono/ingestão/concessão, PDF e pasta com ações realmente disponíveis. | Frontend + backend/GTK já em aceitação; host/geometria quando muda monitor. | Provas nativas anteriores não cobrem todas as geometrias. Repetir após hotplug e com pet a trabalhar; o evento recebido não basta se o cartão fica vazio. |
| UX-10 · Pedir ajuda | Pet e cartão devem identificar pedido/sessão e abrir a decisão exata. Uma falha de resposta fica visível; recolher não decide e callbacks antigos não afetam outro pedido. | Contrato ACK/apresentação nativa + frontend. | Browser8 e nativo11 Codex/Copilot passaram nos âmbitos descritos no estado de entrega; repetir no pacote final e preservar casos ainda sem prova visual. |
| UX-11 · Stop e histórico | Estado interrompido e pedido original legíveis, sem resposta fictícia; preservar os restantes pedidos anteriores e a ordem pública. | Lote12 corrige altura/scroll; backend futuro precisa de eventos duráveis por conversa além do journal limitado e do aviso atual. | Browser de fonte12 passou curto/longo no recorte real do cartão e flutuantes360/760. Persistência e histórico completo não ficam aceites só por esse teste. |
| UX-12 · Monitores e fluidez | Recalcular superfície, escala, alvo e drop quando muda o monitor, mesmo se a configuração anterior ainda aguarda. Conservar navegação e rascunhos, sem loading que bloqueie toda a UI. | Frontend `CompanionsLink` em delta13 do integrador; backend/host se houver outra causa. | Corrida de invalidação reproduzida isoladamente. Não atribuir exclusivamente a ela o cartão incompleto; índices AT-SPI alterados não provam reload. Aceitar hotplug e retorno no pacote correspondente. |
| UX-13 · Toda a navegação | Matriz de botões/clicks/teclado/scroll/arrasto, ausência de dados, falhas, cancelamento e reabertura em chats, sessões, providers/modelos/perfis, tarefas, documentos e Sistema. | Frontend mantém inventário; necessidades novas são comunicadas à coordenação backend e adicionadas aqui. | Manter prova por percurso e versão. Uma suite verde ou clique emitido não certifica toda a experiência; observar efeito e identidade real. |

A interface deve indicar origem e destino sem expor segredos: pet/provider,
identidade curta de sessão, título/pasta quando úteis, estado e frescura. Mostrar
um modelo escolhido para o próximo turno não prova que foi o modelo usado na
resposta anterior. Contagens de agentes/subagentes devem explicitar a hierarquia
e evitar duplicar a mesma sessão importada/gerida. Resultados desconhecidos ou
leitura desatualizada não podem aparecer como consumo zero, trabalho terminado
ou lista vazia confirmada.

A sessão de frontend coordena estes deltas com a integração: fontes congeladas
para um ensaio não são modificadas em paralelo. Preparação privada e auditoria
prosseguem enquanto o backend trabalha; aplicação/compilação/instalação conservam
versões e provas separadas. Alterações persistentes do anfitrião pertencem a
sysworkspace, com commit assinado próprio; código e dependências ficam no projeto.


### Resultados do delta de propriedade e cartões

A fonte integrada seguinte conserva a correção de geometria do integrador e
aplica quatro ficheiros de frontend: reprodução removida da página Sistema,
meteorologia sem coluna vazia, contagem de sessões conhecidas recebida pelo
cartão do centro e abertura da sessão capturada no chat expandido. A recusa de
abertura tem feedback visível e uma atualização do histórico durante a abertura
não deixa os botões bloqueados. A medição Ollama distingue `partial-turn`,
último turno concluído e âmbito desconhecido; valores não medidos ficam
indisponíveis, e a falha de leitura no mesmo CID limpa os números anteriores.

Passaram 699 testes JavaScript, TypeScript e Vite nesta fonte. A cópia privada
passou dez grupos browser (quatro larguras de Sistema, destinos e dados dos
cartões) e quatro grupos de isolamento de métricas; a repetição na fonte
integrada também passou, incluindo o percurso de cartão/documento/rascunho e
recolha/reabertura de aprovação com transporte simulado. Não equivale a instalação nem a aceitação nativa.
O número de sessões conhecidas não é apresentado como número de trabalhos em
execução: o contrato de autoridade/frescura dos estados continua em UX-02/03.

### UX-14 · Controlos do pet Spotify

A auditoria de interações com transporte simulado reproduziu três defeitos no
renderer atual, independentemente da retirada da música do Sistema:

- Anterior, Pausa, Seguinte, Shuffle e Repeat podem receber `false` sem feedback
  visível. Shuffle/Repeat atualizam a cópia local antes da confirmação, o que
  pode anunciar um estado recusado. O próximo delta deve comunicar erro ou
  confirmação em falta e reconciliar o estado observado, sem repetir comandos.
- Posição e volume são `div` sem papel de slider, nome, foco ou teclas de ajuste.
  Devem ter navegação por teclado, limites e valor acessíveis, mantendo o aspeto
  e os gestos de arrasto existentes.
- `pointercancel` executa o mesmo commit de `pointerup`: cancelar o arrasto de
  posição envia um `seek`. A anulação deve terminar o gesto e repor a leitura
  observada, sem enviar um novo salto na reprodução.

O frontend é dono destas correções. O backend/coordenação foi informado de que
`spotify_control=true` representa aceitação da operação e não prova, por si,
um estado MPRIS observado; usa-se a leitura existente para confirmação. Não
foram enviados comandos à aplicação Spotify real nem introduzidas capacidades
de áudio/voz. Este delta permanece separado da fonte em aceitação.


### UX-15 · Bloqueio de teclado entre aplicações — P0 desta build

O utilizador exige corrigir nesta build o bloqueio das outras aplicações e do
sistema enquanto houver cartões de permissão, chat compacto ou chat expandido
abertos. A prova anterior de escrever num editor com apenas um flutuante aberto
não cobre o relato nem a combinação de superfícies. O integrador é dono da
correção GTK/layer/foco e dos ensaios nativos; o frontend revê as chamadas de
captura/libertação e os temporizadores sem alterar a política de aprovações.

Não basta fechar automaticamente os cartões para conseguir escrever. O critério
é manter a decisão ou conversa aberta, clicar numa aplicação exterior, escrever
e confirmar o texto nessa aplicação, regressar ao pet e continuar normalmente.
Repetir com permissão, chat compacto, flutuante e várias superfícies simultâneas,
incluindo ações do sistema e a geometria depois de mudar de monitor. Preservar
os rascunhos e pedidos reais; usar clientes e editores sintéticos nos ensaios,
sem responder Allow/Deny pelo agente nem aumentar permissões. Um modo de teclado
constante, um comando de foco concluído ou um nó AT-SPI focado não prova sozinho
a libertação real. A causa permanece em investigação até observação nativa.

A hipótese separada de falta de evento para Stop/stream no chat compacto não
foi confirmada: o evento existente `chat-turn-state` popula o turno observado
mesmo durante um envio local. Não é requisito de backend novo; a ordenação de
eventos e o efeito de Interromper devem ser testados com o contrato atual.


**Clarificação posterior do utilizador sobre UX-15:** o bloqueio está delimitado
à escrita no chat de um agente no terminal; as outras aplicações funcionam. O
relato amplo acima conserva o histórico do diagnóstico e não descreve o âmbito
atual confirmado. O próximo ensaio deve identificar a sessão e o cliente TUI,
comparar a escrita antes/durante/depois da abertura do chat Coucou e verificar
as semânticas de leitura, ligação e controlo do cliente. Foco normal num editor
GTK não certifica esse percurso. Rever polling de histórico, `read_only`, attach
e proprietário do input sem alterar globalmente a política de foco, interromper
o agente real ou tomar decisões de ferramentas. Esta correção específica
continua exigida pelo utilizador na build em aceitação.

**Reprodução nativa posterior de UX-15 pelo integrador:** com o chat Ollama
flutuante aberto, o terminal real ficou ativo no Hyprland em seis amostras,
mas o campo de texto e o frame do chat continuaram focados/ativos no AT-SPI; a
escrita de controlo entrou no chat, sem Enter, em vez de entrar no terminal.
Assim, a ligação/subscrição Codex não é necessária para reproduzir este caso.
O integrador mantém a correção GTK/passagem do teclado e os testes nativos;
não se considera resolvido por uma indicação isolada de foco do compositor.

**Seguimento privado de UX-14:** o ensaio do centro identificou uma segunda
origem de atraso: `createControlHome` sincroniza os controlos ao receber um
snapshot, mas não subscreve as notificações locais de `State`. Um comando pode
terminar e o cartão só mostrar a alteração depois da próxima leitura pública.
O delta isolado acrescenta essa sincronização, feedback para recusa/falha de
comunicação, nomes/estados dos botões e foco visível, além dos gestos de slider.
A leitura observada continua a ser a autoridade do estado de reprodução; uma
aceitação não anuncia reprodução confirmada. O trabalho permanece fora da
fonte 13 congelada até revisão, conclusão dos ensaios e integração coordenada.

O seguimento privado de UX-14 concluiu 11 testes de gestos/teclado e 19 testes
Spotify existentes, TypeScript e build Vite, além de 16 grupos no navegador em
360/760 px. Inclui recusa/falha/aceitação sem leitura, proteção de atualização
antiga, pesquisa editável durante espera e controlos indisponíveis com faixa
antiga e player fechado. Prova com transporte simulado: não certifica MPRIS ou
o compositor. O patch revisto para integração posterior continua privado; não
entrou na fonte 13 nem foi instalado.

**Atualização de âmbito UX-15:** o utilizador relatou posteriormente o bloqueio
na aplicação Brave quando o terminal já aceitava texto, pelo que a aceitação
abrange novamente aplicações e ordens de alternância diferentes. Os ensaios
precisam de confirmar o destinatário real do texto em cada passo, mantendo os
cartões abertos e preservando o rascunho. O integrador coordena a janela de
60 segundos de controlo exclusivo do rato/teclado para eliminar interferência.
A revisão de frontend encontrou chamadas explícitas de foco na transição para
chat/mail e atalhos; o polling do chat flutuante não foca o composer. A região
de input da ilha de controlo é reenviada durante a animação; isso não deve ser
interpretado pelo backend como um novo pedido de foco em cada frame ou hover.

**Seguimento privado de UX-02:** depois de corrigir o cartão principal,
a auditoria encontrou o mesmo zero inventado no cabeçalho partilhado e nas
etiquetas/ARIA da linha de residentes (`sessions ?? 0`). O delta separado
propaga desconhecido, conserva zero explícito e identifica os valores do
catálogo como sessões conhecidas. Uma cache local de hooks não substitui um
valor ausente do catálogo. Três testes e seis grupos de navegador em duas
larguras passaram, além de TypeScript/Vite; ainda não foi aplicado à fonte
13/14 nem instalado. A contagem autoritativa de trabalho ativo e a frescura/
continuidade após fechar o terminal continuam dependências de backend abertas.

**UX-15 · Protótipo nativo 14, antes da aceitação da aplicação:** o integrador
preparou no Linux um chat GTK normal (`xdg_toplevel`) flutuante no Hyprland,
com a mesma UI/CSS, e retirou o pedido de foco ao expandir a ilha por hover.
O protótipo passou três percursos reais com Brave no mesmo perfil: clique,
duplo clique e passagem entre campo de texto/interior exterior ao campo;
a escrita chegou à aplicação pretendida e o regresso ao chat funcionou.
A movimentação/fixação da janela própria também foi verificada pelo integrador.
A suíte Rust passou 888 testes, com 10 ignorados. Estes resultados pertencem
ao protótipo e à fonte: ainda não certificam a aplicação 14 empacotada. A
aceitação exige repetir a matriz com essa build e as permissões/chats abertos.
O frontend 709 permanece congelado; os deltas privados de UX-14/02 ficam
separados para revisão posterior.

**UX-15 · Aplicação QA 14, teclado validado no âmbito ensaiado:** o integrador
executou a build real e confirmou três percursos Brave e três no composer do
terminal/TUI, com verificação do destinatário exato do texto, restauro do
rascunho e sem Enter ou mensagens aos agentes. Estes resultados já pertencem
à aplicação QA 14, além da prova anterior do protótipo. Não representam uma
aceitação geral de todas as superfícies ou a promoção permanente da build.
Cinco pets passaram abertura, identidade do proprietário, sessões e fecho;
o reensaio Spotify ficou pendente por um frame transitório.

**Regressões nativas 14 e gate da fonte 15:** durante a aceitação, a ilha
central encobriu o chat e a janela GTK abriu em 1100×760, em vez de 760×620.
O redimensionamento IPC para 760×620 foi observado com sucesso. A fonte 15
ajusta a aplicação serializada do tamanho da janela própria e recolhe o centro
após uma abertura de chat bem sucedida; o integrador mantém estas alterações
nos três ficheiros Rust Linux/chat. A suíte Rust passou 888 testes, com 10
ignorados, e a compilação 15 foi iniciada. A aceitação nativa desta fonte está
pendente: repetir teclado, tamanho e sobreposição e concluir a paridade de
permissão, chat compacto, flutuante, ilha central e pets, incluindo Spotify.
O frontend 709 permanece intacto. Não integrar os deltas privados Spotify/
contagens nem alargar funcionalidades durante este gate; continuar os ensaios
das rotas existentes e guardar novos achados para revisão posterior.

### UX-16 · Resposta tardia de detalhes muda a página atual

O ensaio da fonte frontend congelada reproduziu, com transporte simulado, o
percurso Sessões → pedir Detalhes → abrir Modelos antes da resposta. Quando a
resposta da sessão chega, o renderer substitui a lista de modelos pelos detalhes
da sessão Codex, apesar de a navegação continuar a indicar Modelos. `run()` em
`hub/pages/pets.ts` aplica `result.detail` sem revalidar a rota/seleção que pediu
a leitura. A revisão posterior deve capturar essa identidade e descartar a
apresentação tardia depois de uma navegação ou filtro diferente; operações já
pedidas mantêm o seu resultado factual. É responsabilidade de frontend e não
exige novo contrato de backend. A falha foi apenas reproduzida/documentada;
nenhum delta foi iniciado ou aplicado durante o gate 14.

**Precisão de UX-02/03 confirmada na fonte:** `PetIdentity.active` é um sinal de
apresentação do pet: resulta de presença em tarefas, `sessionId`, estado ou
badge (`mochi/companions.ts`). Não certifica que um processo/sessão esteja vivo.
A linha de residentes converte-o em «Disponível · sessão ativa», pelo que esse
rótulo precisa de revisão quando o contrato de estado for estabilizado. A
contagem publicada também usa a cache local de sessões não terminadas; uma
identidade conservada não comprova continuidade do agente após fechar o
terminal. Backend/QA recebeu esta distinção. A fonte congelada não foi alterada.

**UX-15 · Novo relato da ilha central, ainda aberto:** o utilizador reproduz
outra entrada para o bloqueio: abre a ilha por hover, clica no fundo preto,
retira o rato e deixa de conseguir interagir/escrever noutras aplicações até
a ilha desaparecer. O atraso de recolha também é visível e o utilizador espera
recolha imediata ao sair. Este percurso não é coberto pelos seis testes de
teclado aprovados para o chat flutuante na QA 14.

A revisão da fonte confirma que qualquer `pointerdown` na superfície, incluindo
fundo vazio, chama `fsm.userInteracted()`. Isso converte uma abertura por hover
no atraso normal de recolha (`autoCloseInterval`), em vez do prazo curto de
hover. A notificação nativa de saída só inicia a recolha; o estado de teclado
GTK da ilha continua `ON_DEMAND` enquanto estiver expandida. Estes factos são
consistentes com o relato, mas a correção e prova de causa pertencem ao ensaio
nativo do integrador, que recebeu o caso. Incluir obrigatoriamente hover →
clique neutro → saída → clique/escrita em Brave e terminal antes da recolha;
a escrita exterior deve funcionar sem aguardar o desaparecimento da ilha.
O tempo visual de recolha e a propriedade do teclado devem ser verificados
separadamente, preservando formulários e rascunhos. Nenhum patch de frontend
foi aplicado sem coordenação durante o congelamento.

**UX-15 · Delta frontend 16, preparado para revisão:** a ilha passa a pedir
teclado apenas após interação com controlo/campo ativo. Hover e fundo neutro
não o pedem; a saída recolhe logo e reduz de imediato a região de input, mesmo
durante a animação. Arrasto/entrega de documentos conserva a superfície enquanto
necessário, sem manter teclado fora e sem cancelar a concessão de acesso.

A origem manual é propagada separadamente das apresentações automáticas.
Hover/clique no pet abre cartão manual; sinais de ponteiro identificam pet e
contexto e apenas mantêm o cartão atual. A recolha manual usa o próximo turno
de eventos para permitir a passagem pet/cartão, sem contagem de segundos.
Contextos antigos não reabrem cartões ou trocam o proprietário. Cliques em
controlos de um cartão automático sem decisão pendente — por exemplo Escolher
Documento → Email — passam esse percurso ao regime manual. Campos e anexos
permanecem ao recolher. Permissões/perguntas e pedidos de decisão pendentes
continuam protegidos independentemente dessa origem; chats flutuantes ficam
fora da regra. A ingestão no próprio pet e no cartão mantém proteção específica
até terminar, conservando destinatário e documento, sem decisões/envios novos.

A fonte privada passou 719 testes JavaScript, TypeScript, Vite e 21 grupos no
navegador (8 ilha, 8 cartões, 5 pets físicos), incluindo duas janelas de renderer
com transporte nativo simulado. Foi entregue ao integrador para revisão; não
foi aplicada ao frontend congelado pelo agente UX. A prova Wayland/GTK real é
um gate separado: fundo preto, saídas/entrada, passagem lenta pet↔cartão,
permissões/perguntas abertas e escrita em Brave/terminal, além de arrasto real.
Os deltas independentes Spotify/contagens/detalhes tardios permanecem excluídos.

### UX-17 · Dashboard Spotify oculta ações comuns

O reensaio nativo do integrador confirmou que selecionar Spotify na ilha central
mostra controlos de música e esconde Conversar, Conversas e sessões e Configurar
pet. Não é um frame transitório: `pet-overview` oculta todo o conteúdo genérico
e o cabeçalho partilhado do centro também está oculto. O serviço tem provider
lógico próprio `integration_spotify`, via conta Codex; a falta é de acesso na UI.
Há uma rota na fonte pelo pet físico → menu Chat → cabeçalho Conversar com este
pet, que ainda exige a prova nativa correspondente. A área geral Sessões não
substitui essa rota: filtra Codex e pode identificar o pet Codex em vez de
Spotify. A correção futura deve manter ações comuns acessíveis ao lado da
música; não certificar um botão oculto nem alterar essa composição no delta P0.

**Gate 16 após revisão:** o integrador reviu e aceitou o delta P0 de nove
ficheiros frontend, aplicou-o por correspondência dos hashes e iniciou o
congelamento/compilação conjunta. As alterações independentes de Spotify,
contagens e detalhes tardios continuam fora desse conjunto. Os 719 testes e
21 grupos simulados descritos acima pertencem à fonte entregue; os ensaios
nativos continuam obrigatórios, incluindo duplo clique no campo seguido de
alternância Brave/terminal, fundo da ilha e decisões/perguntas sem recolha.
O fecho manual inicia-se sem prazo de segundos; conserva a animação de recolha
e o contexto, em vez de cancelar trabalho, documento ou rascunho.

**Aceitação nativa 16 — resultado parcial:** o integrador confirmou seis
sequências Brave/terminal, dimensões normal/expandida e arrasto/hover da janela
flutuante. O clique no fundo da ilha expandida deixou escrever na aplicação
pretendida sem refoco artificial e a saída recolheu em 71 ms. A passagem lenta
pet Codex → cartão → pet manteve o cartão em todos os passos. Contudo, sair de
ambos não o fechou após 616 ms; esta falha mantém o gate de recolha manual
aberto. A rota física do Spotify permitiu abrir o chat com o proprietário e as
sessões corretos, sem inferência; o dashboard central continua com a falha de
acesso comum descrita em UX-17.

**Seguimento 17 para a saída do habitat:** o integrador preparou um observador
GDK nativo que emite `companions-pointer-inside` mesmo quando WebKit perde o
`pointerleave`. O delta frontend estreito recebe `false` e liberta apenas os
contextos de ponteiro atualmente dentro do habitat, mantendo pet/contexto.
Ocultação nativa/documento e desativação usam a mesma limpeza. Entrada real
noutro pet liberta um contexto anterior que tenha ficado retido. O sinal nativo
`true` não inventa um pet, abre cartão ou troca de proprietário. Entregas em
curso e decisões/perguntas não são limpas por este mecanismo.

O ensaio de regressão reproduziu o cartão retido por 650 ms na fonte 16 sem
`pointerleave`; o delta privado 17 passou nove grupos com duas janelas reais de
renderer e relay nativo simulado, além de 719 testes, TypeScript e Vite. Inclui
DND no pet com concessão preservada, ocultação/reentrada sem reabertura tardia,
permissão e pergunta intactas. Foi entregue para revisão/integrar com o
observador; falta repetir a saída no Wayland da aplicação empacotada. Os gates
SDK e do balão de mensagem/conversa com origem imutável entre sessões A/B ainda
não são certificados por estes ensaios. Nenhum delta Spotify/contagens/detalhes foi integrado aqui.

**Gate 17 após revisão:** o integrador aprovou o delta de um ficheiro e aplicou
o hash revisto. As verificações partilhadas e a compilação conjunta precedem
a nova prova nativa. O Stop e a persistência observados na QA 16 foram aceites
pelo integrador. Após uma nova janela de coordenação, o canário QA 16 fez um
único envio plain para `gemma3:4b`; o par pedido/resposta ficou corretamente
guardado em A. A resposta real, revista e registada por hash no manifesto,
ignorou o marcador pedido e apresentou Mochi genericamente. O validador parou
antes de criar B ou clicar no CTA do balão. Essa tentativa não foi reenviada:
o gate de origem imutável do balão continua aberto. A auditoria de 42 mensagens
passou: 42 mensagens no total, incluindo quatro mensagens reais adicionais
a preservar; há também duas tarefas reais adicionais. O conteúdo observado não
certifica a etapa UI que não foi executada; a revisão do corpo de envio confirmou
system/history/user no pedido. O frontend 17 fica congelado, com os restantes
deltas separados.


**C1 · Consulta sem alteração após expirar o balão:** a revisão da UI não
identificou uma rota para reexibir a mesma mensagem física real depois dos
9 segundos. `PetDialogues.current/visible` e `NativePetSpeech.current` aplicam
o TTL; snapshots/recibos iguais não renovam a janela. Tarefas → Ler resultado
e Conversas e sessões permitem consultar conteúdo guardado, mas não recriam
o CTA expirado nem aceitam esse percurso físico A/B. Não houve reemissão de
`done`, alteração de relógio/TTL, clique num botão oculto ou nova inferência.
O gate C1 permanece aberto. Fica registada para revisão posterior a UX de
voltar às mensagens de um pet com a sua origem imutável; avaliar primeiro os
contratos de histórico existentes. Nenhuma implementação entrou na fonte 17.

**Correção 18 da passagem pet/cartão — validação frontend isolada:** a revisão
visual nativa 17 encontrou um cartão visível sobre o pet mas já recolhido sobre
o cartão e no regresso ao pet. O habitat e o cartão são superfícies separadas,
com cerca de 2 px de intervalo e sinais de ponteiro amostrados separadamente;
fechar em 0 ms perde a entrada seguinte. A regressão isolada reproduziu o fecho
na fonte 17 quando a entrada nativa no cartão chega 33 ms após a saída do pet,
sem qualquer nova abertura.

O delta privado 18 limita a tolerância manual a 50 ms e impede que sinais
repetidos de saída renovem esse prazo. Exige o observador nativo do cartão a
33 ms, preparado pelo integrador; o observador do habitat já usa esse intervalo.
Mantém a proteção contra movimentos DOM obsoletos fora da superfície, os
pedidos de decisão, ingestões/concessões e os rascunhos, sem alterar chats
expandidos nem a política dos cartões automáticos. Passaram 723 testes,
TypeScript, Vite e 22 grupos de browser com relay nativo simulado. A passagem
desencontrada em ambos os sentidos manteve a apresentação original; a saída
de ambos recolheu em 50,5 ms mesmo com sinais de saída repetidos.

Provas privadas: `.build/ux-card-handoff18/`. A entrega inclui um ficheiro de
runtime e um de testes, com hashes para revisão; não foi aplicada pela sessão
frontend à fonte partilhada. A aceitação visual/teclado na aplicação nativa 18
continua obrigatória. A prova nativa 17 não é aceite para a passagem manual.

### UX-18 · Abertura pendente de um pet desativa ações do cabeçalho de outro

Reproduzido em browser isolado: abrir o chat de Ollama com a resposta de
`pet_chat_open` pendente, fechar o cartão e abrir Codex. Os botões de cabeçalho
Conversar, Sessões e Configurar de Codex ficam desativados até a operação de
Ollama terminar. Anexar/Fechar e os botões do corpo continuam disponíveis;
não se trata de um bloqueio total da aplicação. A variável `pending` de
`buildPetCardHeader` pertence ao componente reutilizado, sem associação ao pet
ou à apresentação. O retorno antigo `false` reativou os botões sem recolher
Codex, trocar proprietário ou emitir outra abertura.

Provas privadas: `.build/ux-audit-v25/header-pending-owner.{mjs,json,png,log}`,
contra o frontend privado 18, cujo cabeçalho é igual ao 17. A correção deve
limitar a indisponibilidade ao destino da operação e impedir que callbacks
antigos alterem o estado da apresentação seguinte. Se também for necessário
invalidar uma abertura nativa já iniciada, coordenar primeiro o contrato com
backend; não substituir a autoridade por uma segunda abertura automática.
Registado e comunicado ao integrador, sem delta de runtime nesta build 18.

**Contrato de aberturas confirmado em leitura pelo backend:** o contador
`ChatWindow.opening` e `open_gate` protegem uma abertura de chat admitida contra
outra abertura/hide nativo mais recente; GTK revalida antes de alterar contexto,
visibilidade e foco. Porém o pedido nativo não recebe `sourceContext`: a ponte
frontend só o usa no evento posterior de apresentação. Fechar ou trocar o cartão
JS não invalida por si a abertura A já admitida, e Sessões/Perfil usam outro
percurso de painel. O guard JS impede recolher B por retorno antigo, mas não
cancela a apresentação nativa de A. Falta uma garantia de validade da intenção
do cartão entre superfícies antes de libertar todos os botões para B.
Revisão estática de `pet_chat.rs`, sem certificar o binário 18.

### UX-19 · Consulta de quotas retém a lista de sessões já recebida

Reproduzido na primeira abertura de Sessões: o inventário já devolveu dados,
mas uma consulta `usage` pendente impede `refresh` de os desenhar. A página
mostra «Nenhuma sessão neste filtro» e Atualizar desativado. Ao resolver apenas
a consulta de quotas, surgem três sessões sem nova leitura do inventário.
O dado vazio era uma apresentação intermédia enganadora, não uma resposta
vazia do backend. A mesma sequência de `refresh` alimenta as restantes áreas
de gestão. Falhas/rejeições de quota têm tratamento; uma resposta lenta também
precisa de deixar o inventário utilizável.

Provas privadas: `.build/ux-audit-v25/management-quota-blocks-list.{mjs,json,png,log}`,
frontend 18 isolado, sem inferência nem ações no sistema. Responsabilidade
frontend: desenhar o inventário logo após a sua leitura e atualizar quotas
separadamente, conservando âmbito, frescura e a navegação entretanto escolhida.
Não exige novo contrato de backend. Não integrado na correção 18 de ponteiro.

**Seguimento privado 19 para UX-16/19:** foi preparado um delta independente
de um ficheiro de gestão. O inventário e a sua atualização deixam de aguardar
quotas; só a geração mais recente pode atualizar a medição, e falhas conservam
a leitura anterior identificada como desatualizada. O resultado de detalhes
exige a mesma navegação/filtro de origem; mudar de filtro também retira a
confirmação anterior sem executar a ação. Os campos de pesquisa conservam-se
quando chegam quotas. Passaram 16 grupos de browser em 360/760 px, 723 testes,
TypeScript e Vite. Provas, patch e hashes: `.build/ux-management19/`. Permanece
fora da fonte partilhada e da aceitação nativa 18; o cabeçalho de UX-18 não foi
alterado. A autoridade de aberturas concorrentes foi pedida ao backend antes
de permitir que uma segunda abertura ultrapasse a primeira.

**Gate 18 após revisão:** o integrador aprovou e aplicou o delta estreito de
ponteiro por hashes; confirmou 723 testes partilhados, TypeScript e Vite antes
do congelamento e compilação nativa. Os deltas independentes de gestão,
Spotify e contagens continuam excluídos. A coordenação mantém a exclusividade
dos ensaios gráficos e a aceitação visual da passagem lenta e do teclado.

### UX-20 · Estados e nomes acessíveis nas definições

A auditoria isolada percorreu General, Active pills, Agents, Chat, Integrations,
Android phone, Shortcuts e Pets, inventariando 116 controlos visíveis. Isto
certifica esses percursos de navegação, não todas as ações desses controlos.
Não houve escritas reais, chamadas a serviços, configuração de hooks nem
ações de áudio. Provas: `.build/ux-audit-v25/settings-navigation18.*`.

Os estados booleanos `aria-pressed` passados ao helper `h` eram tratados como
atributos booleanos HTML: o valor desligado desaparecia e o ligado ficava vazio.
O aspeto do interruptor era visível, mas o seu estado inicial não era exposto
corretamente à acessibilidade; a primeira interação já escrevia texto explícito.
Afeta também paletas/seleções que usem esse helper. O campo de chave do servidor
personalizado não tinha nome acessível próprio. A primeira identificação do
campo como Anthropic foi corrigida ao rever a estrutura da página. Correção
privada separada em preparação,
sem alterar a fonte 18.

### UX-21 · Preferência recusada ao guardar mantém aspeto de valor aplicado

No mesmo ensaio, guardar Open on hover com erro simulado mostra o aviso de
falha, mas conserva o interruptor ligado; o último valor efetivamente guardado
continua desligado. O aviso existe e é útil; falta distinguir o valor por guardar
do valor confirmado, com possibilidade de repetir ou repor. Uma eventual
reposição deve respeitar alterações mais recentes e o resultado autoritativo
do backend, incluindo falhas parciais. Não repor cegamente uma fotografia
anterior nem apresentar sucesso só pelo aspeto do controlo. Não implementado
na build 18 nem no delta de acessibilidade.

**Seguimento privado 20 para UX-20:** o helper serializa os estados ARIA como
texto `true`/`false`, conservando a presença/ausência normal de atributos HTML
como `disabled` e `readonly`. A chave opcional do servidor personalizado recebe
nome explícito traduzido com o contexto do servidor. Passaram dois testes novos,
725 testes no total, TypeScript e Vite; as oito rotas com 116 controlos deixaram
de ter controlos visíveis sem nome no inventário, e o interruptor inicial
desligado expõe `aria-pressed="false"`. Provas, patch e hashes separados em
`.build/ux-settings-accessibility20/`. Não aplicado à fonte partilhada nem
incluído na build 18. O erro de persistência visual de UX-21 continua aberto e
foi comunicado ao backend para revisão da autoridade/efeitos parciais.

**Aceitação nativa 18 — passagem de regresso ainda não aprovada:** o primeiro
ensaio rotulado Codex usou um ponto do pet Spotify; foi excluído e não prova
troca indevida de identidade. Uma abertura fresca confirmou Codex pelo título
e sessão. Nessa sequência, pet → cartão ficou visível, mas o regresso ao mesmo
ponto do pet iniciou a recolha; fora de ambos estava oculto após 156 ms.
Provas privadas: `.build/ux-focus18/manual18-codex-verified.json` e as capturas
referidas nesse registo. A aplicação 18 não fica aceite por esta prova.

Uma regressão com dois renderers reais e relay simulado confirmou a hipótese
de ausência de `pointerenter` no regresso: o sinal nativo `false` liberta o
contexto; o cartão mantém-se na sua área; ao regressar, `true` nativo e um
`pointermove` sem entrada DOM não recuperam o ponteiro do pet. O DOM ainda pode
considerar o botão em hover. O controlo com uma entrada DOM real renova o
contexto e conserva o cartão. Aumentar apenas o prazo não resolve uma entrada
que nunca chega. Prova: `.build/ux-reentry18/browser.*`; não constitui um trace
direto dos eventos do binário nativo.

**Delta privado 21 de recuperação, destinado à próxima candidata 19:** recebe
do observador nativo uma posição lógica do habitat apenas na transição para
dentro. Valida elemento registado/visível no próprio habitat, identidade igual
ao cartão apresentado e contexto de ponteiro anterior que esteja fora. Apenas
renova esse contexto; não abre, reapresenta ou seleciona um pet. Os sinais de
ocultação/saída conservam a limpeza anterior. Passaram 12 grupos de browser,
723 testes, TypeScript e Vite, incluindo ordem posição/entrada DOM, contexto
antigo, proprietário errado, ocultação, DND/concessão e decisões/perguntas.
Provas, patch e hashes: `.build/ux-pet-reentry21/`; fonte partilhada 18 intacta
pela sessão frontend, a aguardar revisão do integrador.

A ausência de entrada DOM também pode impedir um novo hover depois de fechar
o cartão. Este segundo percurso permanece fora do delta de recuperação: uma
posição nativa sem cartão atual nunca o deve reabrir sozinha. Foi proposta
separadamente a correlação de movimento DOM real com posição nativa recente,
armada por saída efetiva, mantendo os mesmos bloqueios de decisões/documentos
e limpando candidatos em ocultação/desativação. Não implementada nem aceite.

**Revisão para a nova candidata 19:** o integrador reviu e aplicou por hashes
os deltas independentes de gestão 19 e acessibilidade 20, incluindo os dois
testes ARIA. A sessão frontend confirmou todos os hashes posteriores. Isto
regista integração nas fontes, sem declarar que a aplicação 19 já está a
correr ou que houve aceitação gráfica. Spotify e contagens continuam fora.
O frontend de companions 18 mantém-se intacto até revisão do delta seguinte.

**Protótipo privado 22 para novo hover sem entrada DOM:** foi implementada a
correlação proposta em `.build/ux-pet-hover22/`, incluindo a recuperação de
ponteiro 21. Uma saída nativa observada com ausência de cartão arma a entrada,
quer a saída chegue antes ou depois do snapshot sem cartão. A abertura exige
movimento DOM real e posição nativa recente, ambos no mesmo botão registado,
no prazo de 100 ms e com os bloqueios normais de hover. O movimento e a posição
podem chegar em qualquer ordem e em pontos diferentes do mesmo pet. A entrada
é consumida antes da operação assíncrona; uma entrada DOM posterior não altera
o contexto nem duplica/cancela a abertura iniciada. Sinais isolados, movimento
sintético, evidência antiga, ocultação/desativação e fecho enquanto o ponteiro
continua dentro não reabrem cartões.

A recuperação de contexto de um cartão já mostrado também exige o último
sinal nativo `true`, que o observador emite antes da posição. Uma posição
atrasada após `false` é ignorada. Este guard reduz o risco de recuperar uma
retenção antiga; não prova ordenação total entre eventos sem um identificador
nativo de geração. Os testes de browser usam snapshots clonados como no IPC e
movimentos reais de Playwright; a ausência de entrada DOM é simulada apenas
no ambiente de teste. Os deltas 21 e 22 permanecem separados para revisão;
nenhuma alteração de geometry/foco/autoridade de decisões entrou no frontend.

O delta combinado 22 concluiu 22 grupos de hover com movimentos reais e
13 grupos de retenção/contexto, além de 723 testes na cópia baseada em 18,
TypeScript e Vite. Inclui entrada DOM entregue ao handler durante a espera
do relay, sem renovar o contexto, cancelar ou duplicar a abertura; posição
atrasada após saída é ignorada. Patch combinado entregue para revisão:
`.build/ux-pet-hover22/frontend-pointer22-combined.patch`, acompanhado de
manifesto de hashes e provas com o hash da fonte. Os 725 testes da nova união
partilhada com gestão/acessibilidade cabem à verificação final do integrador.
A aceitação nativa e a correção de hotplug são provas separadas; estes ensaios
não certificam o desktop nem alteram a fonte partilhada pela sessão frontend.

**Gate da candidata 19 após revisão combinada:** o integrador reviu o delta 22
integral e aplicou o patch combinado diretamente à base 18, sem aplicar antes
o 21. A sessão frontend confirmou o hash posterior de companions
`2af367e134f2fd07873379880d0e238f832184cdc854ebd45e599dabd822d084`.
Gestão 19 e acessibilidade 20 já estão nas fontes da mesma candidata. O
integrador mantém as verificações conjuntas, o congelamento, a compilação e
os ensaios gráficos. A aplicação em execução ainda é 18 neste ponto do registo;
os runtimes revistos ficam congelados. O observador nativo passa a obter
proprietário e coordenadas na mesma consulta GDK, mantendo o DTO frontend.

**Aceitação nativa 19 — retenção no pet melhorou, saída ainda falha:** o
integrador observou pet → cartão → pet com Codex visível nos três passos.
Contudo, sair de ambos não recolheu em 120 ms nem após mais de 30 segundos.
O novo hover não ficou provado, pois o cartão anterior permanecia aberto.
Receipt: `.build/ux-focus19/manual19.json`. Não é aceite como fecho manual.

A análise nativa do integrador na implementação Wayland de GTK 3.24.52 aponta
que a consulta relativa à janela pode devolver apenas um filho e excluir o
próprio toplevel focado. Assim pode emitir `false` inicial e nunca produzir a
transição posterior necessária para libertar o contexto criado pela entrada
DOM. O integrador prepara uma correção nativa da consulta de proprietário e
conversão das coordenadas para o habitat; o frontend e os seus prazos ficam
congelados enquanto esta hipótese é validada.

O ensaio isolado `.build/ux-hold-release23/initial-false.*` reproduziu a condição:
`false` nativo antes de existir contexto, entrada DOM real, ausência de saída
DOM e ausência de novo edge nativo. O cartão ficou manual, com ponteiro do pet
dentro, cartão fora, sem pin, decisão, documento ou prazo de recolha. O controlo
com `true` nativo seguido de `false` libertou o mesmo contexto e fechou. Isto
apoia o mecanismo; não constitui um trace do processo nativo. A sequência
alternativa de uma entrada DOM atrasada depois de `false`, em `baseline.*` no
mesmo diretório, continua apenas um stress condicional. Nenhuma alteração de
frontend ou temporizador foi preparada/aplicada para esta etapa.

O primeiro ensaio de todos os pets que procurava Nova conversa no Ollama
encontrou a lista de sessões recolhida; a ausência do botão nesse estado não
estabelece uma falha do produto. O integrador ajusta o verificador para abrir
a lista antes de procurar o controlo.


**Novo requisito de observabilidade — 20/10, pedido explícito para esta entrega:**
o utilizador não consegue perceber o estado diário dos companheiros pelos
cartões atuais. A ilha e cada pet devem expor uma visão útil da atividade, com
sessões ativas corretas de terminal e pet, total de sessões do provider,
arquivadas e consumo relevante. Para LLM local, pretende tokens cumulativos do
provider, com período e cobertura explícitos. O último turno medido não é esse
total; sessões guardadas, hooks e sinais visuais não provam processos ativos.

O backend prepara o contrato em privado, separando atividade confirmada no
Mochi, observação externa e estado desconhecido, com proveniência, cobertura e
atualização. O frontend prepara a apresentação e percursos em
`.build/ux-observability24/`, sem alterar os runtimes congelados de ponteiro/foco.
Desconhecido deve aparecer como tal, nunca como zero. A interface deve permitir
passar do resumo às sessões e abrir o chat expansivo da sessão exata; o terminal
continua uma opção explícita. O consumo cumulativo local exige contabilidade
persistente no backend. A inclusão integrada será revista após o P0 nativo 20;
não está implementada nem aceite neste ponto. Host, integração e commit assinado
continuam a cargo do integrador. Voz/ditado mantêm-se adiados.


**Bug adicional comunicado pelo utilizador — abertura espontânea do card principal:**
por vezes um pet ativo abre o card de hover sem interação pretendida. Este
percurso não é aceite como aviso automático; decisões/perguntas têm um fluxo
próprio. O rastreio de fontes do integrador não encontrou abertura direta por
progresso no Rust. No frontend de cards, hooks de trabalho retornam sem abrir;
o habitat atualiza diálogo/atividade sem chamar open. A entrada DOM normal do
pet abre imediatamente quando os guards permitem, enquanto a recuperação de
hover exige movimento real e posição recente. A hipótese de um pet se mover
sob um ponteiro parado continua por demonstrar no nativo. Um ensaio isolado de
layout sob ponteiro parado em Chromium não gerou entrada nem abertura (0→0);
não é prova sobre WebKit/Wayland. Nenhum runtime/timer foi alterado nesta análise.


**Referência futura de arquitetura — ecossistema ai-local (20/10):**
o utilizador identificou `/home/pmglo/_projects/ai-local` como um conjunto de
projetos locais, distribuídos por vários repositórios GitHub, onde já trabalhou
em serviços que poderão ajudar a evolução dos pets. Registar para revisão
futura de arquitetura; não constitui pedido de análise ou implementação agora.
A revisão deverá inventariar responsabilidades e contratos, avaliar utilidade
concreta e sobreposição com o fork, e justificar cada eventual reutilização.
Componentes sem utilidade demonstrada não devem ser adotados. O nome ledger
nesta entrega não implica integrar esses serviços, migrar dados ou escolher
uma infraestrutura já existente. Cada repositório mantém código, dependências
e dados no seu próprio âmbito.

A abertura espontânea do card principal fica registada como P2 para mais tarde,
conforme a prioridade transmitida pelo integrador. Não bloqueia por si a build
20; os gates de teclado/fecho manual e a observabilidade mantêm prioridade.


**Frontend de observabilidade 24 — proposta privada pronta para revisão:**
`.build/ux-observability24/HANDOFF.txt` descreve 14 ficheiros de runtime e três
de testes; o manifesto contém hashes anteriores/posteriores e o patch. O contrato
TypeScript é uma cópia exata do v1 entregue pelo backend em
`.build/pet-observability-backend/contract.ts`. O comando `pet_observability`
não recebe argumentos; a interface depende da integração desse backend.

A galeria passa a mostrar trabalho em curso e conversas da biblioteca de cada
pet. O card separa a biblioteca do pet do inventário nativo do provider, com
âmbito e hora. Em curso significa turno observado a trabalhar/aguardar; reserva,
idle e notLoaded não provam um terminal aberto ou fechado. Origem externa pode
ser CLI/editor. Valores null ficam desconhecidos, com subtotais observados
identificados separadamente. Contagens das fontes sobrepostas nunca se somam,
e a lista limitada não serve para calcular o total. A navegação usa o alvo
exato, incluindo proprietário lógico e CID; uma sessão de um pet de serviço
que partilha provider continua a abrir nesse pet.

O recurso de leitura é partilhado por webview, com uma consulta em curso de cada
vez, cooldown/poll visível de cinco segundos e coalescência de eventos de 80 ms.
Usa os quatro sinais existentes de observabilidade, turno, histórico e tarefas.
O cache nativo mantém os seus próprios prazos de 10/15 segundos. Falhas conservam
os valores anteriores e a data, marcados como desatualizados; trocar de pet
limpa dados que não tenham proprietário correspondente. Leituras lentas de
metadados/último turno não bloqueiam chat e navegação. Não foram alterados os
prazos de ponteiro/fecho, foco ou autoridade de decisões.

O consumo aceita apenas contadores cumulativos do provider correto e todos os
seus proprietários, em strings decimais sem perda de precisão. Não soma entrada,
saída e total, nem transforma a última resposta em acumulado. Mostra período,
última medição, cobertura parcial/ausente e limitações de persistência. O backend
privado ainda devolve unavailable; medir consumo real exige integrar e ativar
captura e bootstrap persistente, cuja fatia seguinte é preparada pelo backend.
Nada nesta proposta reconstrói consumo anterior ou cobre outras aplicações.

A proposta de layout aumenta apenas o overview físico de 640×296 para 640×520;
Spotify mantém 300 e decisões conservam as dimensões anteriores. O detalhe da
ilha central passa de 476 para 640 de altura, limitado ao espaço da janela, e
o conteúdo flexível tem scroll. Ações principais ficam antes da informação e
o botão Atualizar tem nome acessível. Os dois testes DND que fixavam a altura
antiga passaram a verificar a preservação da altura capturada durante o gesto.
As correções de contagens privadas 16 estão incluídas e ficam supersedidas por
este pacote; Spotify 15 continua independente.

Passaram 740 testes JavaScript, TypeScript e Vite, e 38 grupos de browser com
IPC simulado (26 no card real, 12 na galeria), em larguras 360/760. Incluem
rotas de sessões exatas, proprietário de serviço partilhado, foco estável,
falhas/repetição, leituras lentas, precisão de tokens, dono ausente e scroll.
O patch passou comparação de hashes e `git apply --check` sem divergências com
a fonte partilhada. Estes ensaios não certificam o desktop nativo, a presença
de processos, o consumo real, nem o fecho manual que continuava sem aceitação
na candidata 20. Compilação conjunta, ensaio gráfico nos monitores, integração
e commits assinados pertencem ao integrador.

**Dependência de evolução da gestão completa:** as listas e ações da gestão
avançada ainda usam o inventário anterior da estação. A uniformização dos
âmbitos, contagens e rótulos com o snapshot canónico permanece necessária;
a proposta dos cards não deve ser apresentada como certificação dessa lista.
A origem das contagens e a capacidade real de interromper/arquivar cada sessão
devem continuar explícitas na evolução do contrato.


A leitura de fontes confirmou a diferença de semântica da gestão anterior:
`ia-barra` considera ACTIVE os estados working, attention, idle e running; o
resumo da gestão chama a esse subconjunto sessões ativas. Inclui sessões
carregadas sem turno e observações de processos, pelo que não equivale a Em
curso do novo snapshot. A lista também deduz Terminal quando não encontra uma
tarefa Mochi correspondente e abre o chat com o identificador nativo anterior,
sem usar o novo OpenTarget. Ausência de tarefa não prova origem externa ou
terminal; as ligações e ações deverão passar a respeitar proprietário e
capacidade comprovados pelo backend. O estado Atual na lista de conversas do
chat identifica a conversa corrente da biblioteca, não um processo vivo.
Esta análise foi apenas de fontes, sem executar o helper, consultar contas ou
ler históricos. Mantém-se como evolução separada; o pacote 24 fica imutável.


**Evidência local para o bug P2 de abertura espontânea:** o utilizador forneceu
o vídeo `/home/pmglo/Videos/Gravações/2026-10-20_01-11-50.mp4`. Fica associado
a esse item para análise posterior, mantendo a prioridade acordada. O vídeo
é privado e permanece fora do Git; a existência do ficheiro foi confirmada,
sem analisar o seu conteúdo nesta etapa. Não constitui ainda uma causa ou
uma reprodução diagnosticada.

**Estado de integração das fontes 24 e adaptação de altura 25 (20/10):**
o integrador aplicou a observabilidade 24 à fonte partilhada; os 17 hashes
finais foram conferidos de novo e coincidem integralmente. Isto confirma a
integração de fonte, sem certificar ainda a build instalada ou o desktop.
O seguimento privado 25 limita o overview físico a `min(520, innerHeight-24)`
e reage a resize sem reconstruir o DOM. Preserva largura 640, Spotify 300,
decisões e regras de foco/fecho. Passaram 742 testes JS, TypeScript e Vite,
além do browser com Island real e IPC simulado em alturas 320/428/560/900:
scroll até ao último turno e foco conservado. O pacote 25 foi entregue ao
integrador para revisão/CAS; a prova nativa continua pendente. O canvas Linux
novo de cards tem dimensão própria, preservando a geometria legacy.

**Atualização de ownership do Spoty:** por orientação posterior, o Spoty passa
a concentrar Spotify e o áudio do PC (saídas, entradas e fluxos de aplicações)
numa interface própria, determinística e sem chat, modelos ou perfis LLM.
O identificador e os dados anteriores são preservados. O agente dedicado
coordena a respetiva fatia frontend/backend e incorpora a proposta Spotify 15
sobre a base 24; os controlos de som gerais saem da ilha para este pet. A nova
validação de conversas abrange cinco chats, com o áudio validado separadamente.
As melhorias de voz/fala/ditado permanecem adiadas. Esta atualização substitui
a interpretação anterior de Spotify como mais um chat de serviço.

**Seguimento privado 26 — consistência da gestão completa:** o resumo e o filtro
do helper passam a dizer sessões sinalizadas, explicitando que incluem sessões
sem turno, e os agentes são apresentados como deteções. A ausência de tarefa
associada deixa de determinar Terminal; a origem vem da observação canónica ou
fica por apurar. A lista completa, árvores e ações de gestão, quotas, modelos e
memória são preservadas. A seleção limitada do snapshot não substitui a lista.

O botão Abrir conversa só recebe um SessionTarget canónico único para o par
backend/ID nativo, com proprietário e CID reais. Duplicados do mesmo destino
são equivalentes; destinos contraditórios, metadados desatualizados, falhas de
leitura ou ausência na seleção ficam indisponíveis com explicação. A abertura
revalida o destino, o pet ativo e a capacidade de chat. O painel só se fecha
quando a abertura é confirmada; falhas/false permitem repetir. As opções de
terminal mantêm nomes explícitos. O índice é preparado uma vez por desenho,
com o recurso de observação partilhado; os eventos são desligados ao sair.

Passaram 35 testes JS focados, TypeScript, Vite e 18 grupos de browser com o
módulo real e IPC simulado em larguras 360/760. Incluem lista completa de 24
registos, CID do pet, externo, dono GitHub, foco, filtros, falhas/repetição e
preservação de quotas/modelos/memória. O pacote foi entregue ao integrador,
sem aplicação pelo frontend nem certificação nativa.

**Necessidade de backend comunicada:** para abrir qualquer registo da gestão,
incluindo os que ficam fora da seleção de 12, falta um lookup readonly canónico
por backend/ID nativo, ou um OpenTarget e capacidades publicados na própria
lista. Deve confirmar proprietário, CID, origem, ambiguidade e disponibilidade.
Até existir, a UI não deduz um destino para os restantes registos. Esta evolução
é separada da captura de consumo Ollama e não deve atrasar os respetivos gates.
A disponibilidade real das ações legacy de interrupção/arquivo/terminal continua
a exigir validação nativa. Os atalhos de tarefas/modelos que ainda abrem o
overview também precisam de alinhamento futuro com o chat expansivo por defeito.

**Correção privada 27 após revisão de 26:** uma resposta atrasada de Abrir
conversa só pode atualizar a mensagem ou esconder o painel se o separador,
a geração de navegação, a abertura mais recente e o SessionTarget canónico
continuarem iguais. A limpeza do estado pendente ocorre sempre. Passaram 35
testes JS focados, TypeScript, Vite, os 18 grupos de regressão da gestão e oito
novos grupos de corrida em 360/760: sucesso/erro depois de mudar de separador,
conversa posterior noutro pet e CID alterado enquanto a abertura esperava.
Ensaios com módulo real/IPC simulado, sem aceitação nativa; proposta entregue
imutável ao integrador. Não foram acrescentados outros blocos à candidata.

**Coerência ainda a rever após esta candidata:** o chat expansivo conserva
rótulos como Terminal/Sessões de terminal para destinos `origin=external`, e
o task-board chama Sessão no terminal a sessões externas. O contrato canónico
esclarece que externo também pode ser um editor ou outro cliente nativo.
Deverá usar Externa enquanto não houver evidência mais específica; o botão
opcional de localizar/abrir terminal precisa de capacidade observada. A seleção
Atual na biblioteca continua a significar conversa escolhida, não processo vivo.
São pontos de observabilidade registados, sem novo patch nesta candidata.

**Revisão do contrato futuro de lookup (privado, depois da candidata):**
o backend propõe `pet_session_lookup` por backend e ID nativo exatos, com
resultado resolved/ambiguous/unavailable/notFound/notSupported/invalidRequest,
OpenTarget e evidência separada de biblioteca e inventário nativo. A revisão
frontend pede explicitar que o ID Codex é Thread.id, que o provider do destino
pode ser um owner lógico e que owner não é o petId. A resolução só permite
navegação e não prova atividade, presença de terminal ou autorização de ações.
Ficam a clarificar no contrato a aceitação de uma evidência nativa antiga para
um destino resolvido e o tratamento de cooldown em cliques sucessivos noutras
sessões: preferir partilhar a consulta/cache ou dar uma indicação de repetição,
mantendo a restante gestão disponível. Pontos enviados ao backend; não houve
integração de UI ou alteração da candidata nesta revisão.

## Pendências adicionais verificadas nas definições

## Decisão para ligar o Sistema nativo à rede

A próxima revisão mantém uma única janela da ilha com navegação controlada pelo
backend. A autoridade será uma janela GTK/WebKit exata, mapeada e visível, com
contexto expandido de Sistema aceite pelo backend e a definição nativa guardada
ativa. A rota aceite é uma intenção de apresentação; não prova DOM, píxeis,
atenção ou gesto físico. As abstrações de igualdade Tauri por nome da janela
não comprovam a sua identidade. Ocultar, destruir, recarregar, mudar de rota ou
desativar o Sistema devem invalidar épocas antigas antes de novas ações.

O executor privado recebe uma referência fraca e um contexto capturado por pedido:
um pedido antigo enfileirado não ganha a época nova depois de sair e reentrar.
Os tokens de pesquisa já bloqueiam scopes antigos; não foi demonstrado um bypass
de pesquisa. A regressão de leitura enfileirada e a correção do contexto passaram
com os módulos reais no executor privado. O worker conserva a operação e o IO até
terminarem. Esta decisão ainda não está no executável .18 nem valida o tab DOM.
O executor R1 privado passou 139 testes inclusivos (32 novos e os 107 anteriores),
com fonte e recibos conferidos pelo integrador. A regressão de leitura enfileirada
falhou antes e passou depois da correção; não é uma aceitação GTK ou de pesquisa
real. A implementação GTK/Tauri está em pasta privada. O pacote R1 passou
1234 testes Rust, com dez ignorados (1069 da base .18, 139 de rede anteriores e
26 novos). A revisão independente conferiu o percurso do contexto capturado
até à escrita, mas encontrou duas falhas de coerência da entrada: posicionamento
podia reaplicar região/teclado antigos depois de revogação e o crash WebKit não
limpava a política/timer nativos se GTK continuasse mapeado. Não foi demonstrada
passagem das guardas de rede. O sucessor R2 corrige estes pontos antes da ativação,
mantendo o pacote anterior fechado. A versão privada R2 passou 1247 testes Rust,
com dez ignorados, incluindo 13 fixtures novas com 79 casos causais. A revisão
independente está em curso. São operações nativas simuladas sobre o sequenciador
de produção, sem aceitação GTK/compositor. Não atribuir estes ensaios à .19
instalada nem a uma reprodução gráfica.

A UI privada FE47 passou 931 testes JS (31 novos), TypeScript/Vite e 60 grupos
de browser com IPC simulado. A revisão não encontrou novos P0/P1. O sucessor
FE47-r2 corrige apenas o texto que prometia pesquisa em todos os adaptadores:
o comando global do daemon usa o dispositivo predefinido. Repetiu os mesmos
931 testes e 60 grupos, sem testes novos para esta alteração de texto. A navegação, os tickets e os
estados desconhecidos têm provas simuladas; GTK/compositor/API34 reais continuam
por aceitar. Recolha e fecho não afirmam cancelar operações já despachadas.

### Definições gerais e resumo semanal — correção .19 instalada

A auditoria43 da janela real de definições com transporte simulado confirmou seis
grupos em duas larguras. Limpar o histórico do resumo semanal mostrou sucesso
após uma recusa simulada de transporte. O bridge engolia erros nessa chamada, e
o backend devolvia void: limpava RAM e apenas registava falhas de escrita em
disco. A correção exigiu confirmação/erro de persistência e apresentação correta;
a .19 aplica esse contrato à parte Recap. A auditoria43 não leu nem apagou
histórico real. O mesmo limite afeta a persistência de ativar/desativar
o recap e precisa de semântica explícita para memória, disco e privacidade.

A preferência de recap desativada deixava `aria-pressed=true`, corrigido pela
FE46. Os toggles gerais ainda conservam a escolha local depois
de falhar a gravação; existe banner de erro, mas falta reconciliação do estado
observado sem sobrescrever alterações recentes ou outras janelas. Os
achados de Recap foram tratados; os toggles gerais conservam a sua fase própria.

O recap guarda contagens e projetos semanais; não representa o catálogo de
sessões externas, histórico de mensagens ou ledger de consumo dos providers.

A correção está instalada na .19 (backend Recap + FE46).
O backend usa recibos causais de memória/persistência: apagar
só é confirmado depois de escrever; uma falha ao desativar a recolha ou ocultar
projetos mantém a proteção nesta sessão e informa que não foi guardada. Os
ensaios usam apenas ficheiros sintéticos, sem tocar no histórico real. O backend
compilou no SDK Linux e passou 24 testes de Recap (dez novos e 14 existentes).
A UI interpreta o recibo e conserva o estado/feedback mais recente.
A composição privada inicial .19 passou 1079 Rust (dez ignorados), 888 JS,
TypeScript e Vite. Foi recusada para promoção quando a revisão encontrou uma
leitura iniciada durante ocultar nomes que podia sobrepor o recibo mais recente.
A reprodução em FE46 confirmou quatro casos (readPreferences/load/open/check).
A revisão cumulativa FE46 conservou o pacote anterior e os testes que falharam.
Os quatro passaram após a correção; leitores iniciados durante a gravação não
podem sobrepor o recibo posterior. A revisão independente não encontrou novos
P0/P1/P2 demonstrados nesse contrato. A composição final passou 900 testes JS,
TypeScript e Vite; os 162 ficheiros Rust ficaram byte a byte iguais à composição
dos 1079 testes. O RPM .19 está instalado e ativo; arranque e reposição do
rascunho foram aceites. Não foi feito um ensaio de mutadores no histórico real.
Esta evidência não resolve os toggles gerais ou Settings/Chat abaixo.

### Identidade de sessões e consumo — próximos contratos

A revisão do protocolo local 0.163 e dos adapters não encontrou prova suficiente
para associar cada TUI arbitrária à sua Thread.id atualmente selecionada.
Thread.id e Thread.sessionId da árvore são diferentes. Descritores de rollout,
PID, pasta, título, variável herdada e ACK de abrir terminal não são autorização
de controlo. Manter separados observação do processo, associação a UUID, ligação
nativa atual e autorização por operação. O caminho já suportado é escolher
explicitamente a thread canónica e respeitar as suas capacidades; não o apresentar
como controlo de um terminal ainda não identificado. O plano privado enumera
13 fixtures necessárias; não é uma implementação nem um bloqueio demonstrado
para todas as versões futuras do Codex.

A [documentação oficial do App Server](https://learn.chatgpt.com/docs/app-server)
define `thread/tokenUsage/updated` para a thread e `account/usage/read` para
atividade de tokens da conta. O schema gerado localmente também prevê consumo
opcional de uma thread pedida. São fontes diferentes das quotas e do ledger
de pedidos observados no Mochi. A geração do schema foi offline, sem consultar
contas ou modelos. Falta implementar produtor/cache/contrato da UI: atualização
da conta explícita, datas e cobertura reais, UUID já resolvido, desconhecido
preservado e nenhuma soma de snapshots cumulativos como se fossem deltas.

O leitor de atividade da conta está implementado na .20 inativa:
`pet_account_usage` consulta só RAM; `pet_account_usage_refresh` inicia uma leitura
explícita do CLI, com autenticação ChatGPT nativa e sem inferência ou comandos
de login/logout. O próprio CLI continua responsável pela gestão dos seus tokens.
Expõe apenas as cinco métricas do resumo, sem dados de identidade,
estimativas de custos ou buckets diários. O âmbito é a última conta observada,
sem alegar que um login externo posterior continua a ser a mesma conta. A data
é a da observação local da resposta; não é uma data de publicação do provider.
Há uma leitura de cada vez, prazo e cooldown, valores desconhecidos preservados
e limpeza dos dados anteriores antes da nova leitura autenticada. Os 22 ensaios
sintéticos do primeiro leitor passaram no SDK Linux, incluindo transporte nativo
simulado e limpeza dos filhos; a revisão independente passou nesse contrato.
O sucessor R2 acrescenta identidade efémera do produtor, revisão monotónica em
início/fim e notificação depois de libertar o lock. Passaram 34 testes inclusivos,
com 12 novos sobre ordenação, limite do contador, callbacks e reinício. O evento
é apenas uma indicação para reler RAM; não pode autenticar, afirmar o estado ou
fixar uma revisão a partir de uma mensagem emitida pelo renderer. Leituras com
a mesma revisão ainda podem ter datas/staleness diferentes e exigem tratamento
causal na UI. A revisão independente do R2 não encontrou P0/P1/P2 no backend.
Identificou um teste que podia passar porque a asserção corria dentro de um
callback protegido por catch_unwind. O sucessor R3 altera apenas esse teste:
regista testemunhos no callback e verifica-os fora do bloco que captura panics.
Os 34 testes passaram novamente no SDK; runtime e IPC ficaram byte a byte iguais
ao R2. A revisão independente deste delta passou. A UI FE50 também foi revista
e integrada; apenas consultas nativas à cache aplicam snapshots. Os ACKs de
atualização invalidam leituras antigas e pedem uma consulta nova à RAM; os eventos
também não aplicam dados nem autenticam. A consulta autenticada continua explícita.
Não há comandos registados na .19 ativa; a integração está na .20 inativa.
Uma consulta real independente ao CLI instalado confirmou o protocolo exato:
initialize com experimentalApi=false, initialized, account/read(false) e
account/usage/read({}). As cinco métricas estavam disponíveis em 2,55 segundos;
o filho próprio terminou e foi recolhido. Sem inferência, thread/turn/start ou
registo de identidade da conta. Os valores ficaram apenas numa prova privada,
sem semear a cache ou a UI. Isto não aceita o percurso Tauri/eventos/interface.

O plano local do produtor de consumo CLI identifica mais uma fronteira concreta:
o ledger ativo admite apenas Ollama e o RPC Codex pode descartar uma notificação
thread/tokenUsage/updated recebida antes do ACK de turn/start. A implementação
futura precisa de preservar a notificação limitada, correlacionar os IDs nativos
com o turno/CID já pertencente ao pet e confirmar a semântica de last/total antes
de registar consumo. O schema, por si, não prova cobertura completa de um turno.
O plano não acrescenta captura CLI à .19 nem à candidata .20.

### Settings / Chat — respostas fora de ordem, corrigidas na .20 inativa

A auditoria44 confirmou quatro grupos com a janela real e IPC simulado. Uma
ligação LM Studio A pendente, seguida de sair/reentrar na página e concluir B,
permite que a resposta antiga A guarde por cima do endereço e modelo mais
recentes de B. O UI corrente conserva B, mas a última gravação pede A. O guard
de intenção deve começar no gesto de ligar e ser verificado depois de metadata
e antes de guardar/selecionar; não basta iniciar a intenção de modelo quando a
resposta chega. O código comum cobre outros providers, mas a prova é LM Studio.

O Save key legado da Anthropic também limpa um rascunho novo introduzido enquanto
uma gravação anterior estava pendente. Deve conservar o texto novo e correlacionar
feedback/pending com o pedido, sem ler segredos do cofre nem inferir valor a partir
da simples presença de chave. As provas usaram chaves e endpoints fictícios;
nenhum serviço, conta ou configuração reais foram tocados.

São itens fora da .19 instalada. A correção privada FE49 passou 905 testes JS,
TypeScript/Vite, 40 grupos simulados de browser e duas reproduções causais,
sem consultas reais a contas/modelos ou segredos. A revisão encontrou um P2
herdado: a leitura inicial de presença de chave ainda convertia desconhecido
em ausente. O sucessor FE49-r2 conserva «Por confirmar» nessa situação. A revisão do delta
passou; manteve 905 testes e 40 regressões simuladas e acrescentou duas
reproduções causais de leitura inicial nula/recusada, sem gravar chaves.
Inclui apenas o bloco de ligações/chave e as traduções órfãs respetivas;
os toggles gerais ficam fora. Correções de uma janela podem usar guards frontend;
a coerência de saves completos entre clientes precisa de revisão própria e não
de CAS presumido.
