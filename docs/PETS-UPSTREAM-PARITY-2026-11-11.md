**21/11 — RPM 62 instalado e verificado na sessão operacional.**
O executável corresponde ao RPM validado e a página Pets abriu pela
janela nativa. A cópia específica e o prefixo .44 permanecem disponíveis.
Os ensaios de foco, diffs, Cal.com e regras 6/7 mantêm os seus âmbitos
privados descritos abaixo. [Entrega e limites](PETS-INSTALLED-DELIVERY-62.md).

**20/11 — ditado com o microfone escolhido indisponível.**
O oficial 0.5.0 abriu o predefinido sem perguntar. O RPM 56 conservou
a escolha explícita e recusou gravar com outra fonte. A correção
na fonte atual assinala a opção indisponível e abre o seletor a partir
do aviso. Depois de escolher uma fonte disponível, passaram ditado,
colagem e reinício no binário privado. [Comparação e provas](PETS-VALIDATION.md#ditado-microfone-indisponivel-2011).

**20/11 — reduzido o custo de arranque com os pets de desktop desligados.**
O fork evita alocar a janela até à primeira ativação nesta configuração.
O ensaio privado retirou um processo WebKit e mediu menos 27 a 41 MiB
de PSS. A ativação, a ocultação e o reinício passaram no binário
corrigido. O pacote 55 congelado conserva o comportamento medido
na comparação abaixo.
[Resultados do arranque e limites](PETS-VALIDATION.md#arranque-pets-desligados-2011).

A ativação e ocultação também passaram em Wayland num perfil privado.
O centro ficou aberto, o mesmo pedido Codex sintético continuou acessível
e o Stop encerrou-o sem reaparecer após reinício. No modo de barra, os pets
só foram mostrados depois de chegar a geometria do compositor.
[Percurso Wayland e limites](PETS-VALIDATION.md#wayland-pets-centro-2011).

O próprio RPM 56 confirmou o modo de barra e o ciclo dos pets em
Wayland privado. No mesmo pacote passaram anexos nos chats Ollama e
Anthropic, o restauro das conversas e a revisão/sucessão de regras 6/7.
Os ensaios usaram perfis gerados. A instalação 44 continua a ser a
composição operacional. O novo formato da biblioteca requer um
backup anterior compatível para reabrir as conversas numa reversão
para a 44. [Pacote, evidência e limites](PETS-VALIDATION.md#rpm56-percursos-2011).

**20/11 — recursos e resposta durante dois percursos ativos.**
No Hub com a mesma geometria, o RPM 55 teve mediana de 64 ms até
ao segundo frame com conteúdo pronto, igual à do oficial 0.5.0.
O 55 usou menos CPU e mais memória. No cartão Vercel, a mediana
ficou em 15 ms no 55 e 24 ms no oficial, com alturas diferentes
e maior PSS no fork. A 44 e a 55 tiveram valores próximos no Hub.
Os resultados descrevem a navegação ensaiada em X11 privado.
[Método, resultados e limites](PETS-VALIDATION.md#recursos-ativos-2011).

**18/11 — Vercel com atualização nativa e detalhe estável.**

| Percurso | Classificação | Comportamento observado |
| --- | --- | --- |
| Atualizar, listar, abrir detalhe e recuperar de 401 | Preservado no ensaio nativo | Oficial e fork consultaram o mesmo caminho do servidor sintético e mostraram os estados, commit, branch e ligação. |
| Manter o detalhe aberto durante atualização | Limitação herdada, corrigida | Oficial e fork anterior trocavam o deploy pelo mais recente. A correção mantém o ID escolhido e indica se esse deploy sair da lista atual. |

[Comparação, composição e limites](PETS-VALIDATION.md#vercel-1811).

**18/11 — recuperado abrir o chat API a partir do pet Anthropic.**
A fábrica já distinguia a conversa Anthropic da associação de um motor
ao runtime. O fork abria a janela flutuante vazia porque a abertura só
consultava essa associação. A correção usa a rota API declarada do pet.
A janela corrigida passou arrasto de ficheiros, pedido ao gateway
local sintético, resposta e reinício. Os adaptadores API passam a
mostrar o resultado observado da preparação junto de cada cópia.
OpenAI e Google AI abriram conversas distintas com as rotas esperadas.
A validação ponta a ponta desta entrega cobre Anthropic em perfil privado.
[Prova e limites](PETS-VALIDATION.md#anexos-api-1811).

**18/11 — preparação textual por cópia visível no histórico local.**
O arrasto de ficheiros para a conversa mantém o percurso comparado
com o oficial. O fork acrescenta seleção por conversa, IDs e
proveniência. Num binário de desenvolvimento, o resultado parcial
identifica a cópia afetada e persiste após reinício. A comparação de
cobertura aplica-se ao chat local ensaiado. A instalada 44 continua
operacional e a correção está fora do RPM 55 já verificado.
[Comparação e provas](PETS-VALIDATION.md#cobertura-anexos-1811).

**18/11 — correções de anexos e sucessão reunidas no RPM 55.**
O pacote confirmou a retirada de anexos e o histórico após reinício.
Também conservou os percursos X11 do centro e da visibilidade, com
retorno ao pedido pendente original. O ciclo governado 6/7 do fork
passou no mesmo executável. Estas provas cobrem os percursos privados
descritos. A instalada operacional continua na 44.
[Proposta de entrega do 55](PETS-INSTALLED-DELIVERY-55.md).

**18/11 — anexos e controlos do centro verificados no próprio RPM 54.**
O pacote confirma a correção do reenvio após retirar uma cópia e
os dois registos por mensagem após reinício. Passaram também
atalhos e preferência de visibilidade em X11 privado, incluindo a
preservação do mesmo pedido pendente. O restauro 6/7 manteve leitura
histórica e recusa de execução. O bloqueio da segunda sucessão é
uma lacuna do runtime de pets do fork. A versão 44 continua instalada.
[Provas e composição](PETS-INSTALLED-DELIVERY-54.md).

**18/11 — retirado o reenvio de conteúdo de anexos históricos.**
O gesto de arrastar e conversar sobre um ficheiro está comprovado no
oficial 0.5.0 e no fork. A seleção individual de cópias por conversa é
uma extensão do fork. Nas versões 44 e 53, retirar uma cópia alterava
a seleção visível e o pedido atual, mas deixava os bytes preparados
na mensagem anterior que o Coucou reenviava. A correção validada em
binário privado conserva o diálogo e valida a seleção para cada novo
envio dos ficheiros. A proposta 53 fica suspensa.
[Comparação do percurso e prova](PETS-VALIDATION.md#retirada-anexos-1811).

**18/11 — anexos múltiplos com proveniência visível por pedido.**
A comparação com o oficial 0.5.0 mantém a prova anterior de arrastar
e conversar sobre um ficheiro. A extensão de múltiplas cópias por
pet/conversa pertence à arquitetura atual. O RPM 53 conservava
os IDs de duas cópias, mas só mostrava os nomes no texto do pedido.
A fonte posterior mostra esses IDs e a fase num resumo recolhido.
O executável de desenvolvimento leu os mesmos dados após reiniciar
em X11 privado. Um novo pedido sem anexos conservou a história
e não voltou a enviar o conteúdo dos ficheiros.
[Comparação dos dois binários e limites](PETS-VALIDATION.md#anexos-por-pedido-1811).

**18/11 — recursos em repouso do RPM 53 comparados com a 44 e o oficial.**
Seis arranques nativos em perfis privados no Hyprland, com ordem
oficial/44/53/53/44/oficial. Cada medição durou cerca de 13 segundos
após oito segundos de aquecimento. Companions, centro persistente e
Flow estavam desligados. Sem automação de janelas ou inferência.

| Executável exato | PSS média | CPU média (% de um núcleo) | WebKit | Camada |
| --- | ---: | ---: | ---: | --- |
| Oficial 0.5.0 | 416,46 MiB | 105,81 | 5 | 720×320 |
| 44 operacional, em perfil privado | 464,82 MiB | 1,01 | 6 | 720×560 |
| RPM 53, em perfil privado | 455,43 MiB | 1,01 | 6 | 720×560 |

A 44 e a 53 tiveram a mesma geometria e contagem de processos. A 53 usou
9,38 MiB menos de PSS em média nestes arranques.
A CPU medida foi cerca de 1% de um núcleo em ambas. O 53 usou mais PSS
do que o oficial neste ensaio, com geometria e quantidade de processos
diferentes. A CPU do oficial mede esse estado de arranque específico.
A amostra curta deixa em aberto fluidez durante uso, GPU, energia, fugas
e comportamento no perfil operacional.
[Recibos de cada arranque](../.build/operational53-20261118/RESOURCES-VERIFIED.json).

**18/11 — assunto e data do Resend observados completos no RPM 53.**
No Hyprland real, a captura do interior do cartão sintético mostrou o
assunto e a data completos. As medidas antes e depois coincidiram, e o
WebKit reportou a mesma cor corrigida nos dois textos. O ensaio usa só
um evento nativo sintético. A imagem permanece privada. A medição
anterior de contraste com cores planas mantém o seu âmbito.
[Prova e tentativas conservadas](../.build/operational53-20261118/RESEND-VISUAL-VERIFIED.json).

**18/11 — RPM 53 verificado no Hyprland e em X11 privado.**
O próprio pacote preserva o centro durante a configuração em segundo
plano e as conversas ao ocultar os pets. Passaram primeira abertura,
atalho por linha de comandos, reinício, leitura histórica 6/7 e
prévia de backup sintética. A proposta com hashes e alteração focada
do sysworkspace está pronta. A instalada continua na 44.
[Provas, limites e âmbito de entrega](PETS-INSTALLED-DELIVERY-53.md).

**18/11 — RPM 52 validado em X11 privado. Corrigido o fecho do centro em Wayland na fonte posterior.**
O 52 alternou o centro, guardou a visibilidade dos pets após reinício e
reabriu o mesmo pedido pendente. No Hyprland, a configuração em segundo
plano ainda podia fechar o centro no 52. A correção integrada preserva
a navegação explícita e passou 29 testes dirigidos e um ensaio no
Hyprland em binário privado. A proposta 52 fica suspensa. A instalada
continua na 44. [Artefactos, evidência e limites](PETS-INSTALLED-DELIVERY-52.md).

**17/11 — recuperado mostrar/ocultar os pets configurados, em composição privada.**
O oficial0.5.0 alterna a saída/regresso do seu pet; na privada anterior o atalho
não tinha efeito. A adaptação usa os pets atuais e a transação de preferências
existente. Atalho real, controlo Geral, visibilidade nativa, reinício e preservação
de um pedido Codex sintético/CID passaram, sem aceitar/executar o comando.
14 Rust dirigidos/64 web/tipos; instalada44 e RPM51 preservados. O binário privado
12230fac tem versão interna51, mas não é esse RPM. Falta pacote sucessor,
compositor real e entrega própria; a paridade útil e os49 não estão concluídos.
[Contrato, comparação, provas e limites](PETS-UPSTREAM-PARITY-2026-11-11.md#desktop-pets-1711).

<a id="desktop-pets-1711"></a>
## 17/11 — atalho e preferência para mostrar/ocultar os pets atuais

**Problema confirmado:** no oficial0.5.0/841a81ce, desktopToggle faz o pet sair
para o desktop e voltar. A privada7d274c3c recebeu as teclas mas manteve
companionMode false/false/false e a janela companions oculta. O controlador
singleton Linux foi retirado intencionalmente; o atalho ainda dependia da sua
capacidade, sempre off no modo atual. A correção preserva o gesto útil, adaptado
à apresentação dos pets configurados, sem ressuscitar o singleton ou ativar pets
sem uso. Oficial: onDesktop false/true/false; privada corrigida12230fac:
companionMode e visibilidade nativa companions false/true/false.

**Contrato e implementação:** desktopToggle envia a intenção atómica
`toggleDesktopVisibility` pelo comando existente `pet_roster_settings_patch`.
O controlo Geral existente envia `setDesktopVisibility { enabled }`, aguarda o
recibo nativo e apresenta erro de gravação sem antecipar sucesso. Rust serializa
contra as preferências atuais, persiste antes de publicar e devolve o campo
existente companionMode. Uma gravação geral desatualizada preserva esse campo,
como já preservava o roster. Só as mesmas janelas nativas de preferências têm
acesso; a alternância não transporta um valor booleano desejado nem aceita campos
extra. A variante Rust é uma estrutura vazia: a variante unit de Serde aceitava
campos extra, detetados pelo teste e corrigidos antes de qualquer aceitação.
Não há novo comando, formato persistente, ledger, provider, modelo ou autoridade.
A ação de apresentação não pausa nem retoma trabalho; não volta a consultar
integrações/Spotify como efeito de alterar apenas a visibilidade.

O controlo e o atalho partilham o mecanismo de preferências existente, com
intenções serializadas e observações nativas mais recentes a prevalecer sobre
recibos atrasados. A política da fábrica por domínio fica intacta: n8n é serviço/
workflows; esta recuperação não acrescenta chat LLM ao n8n. Mostrar pets mantém
os filtros/roster atuais, não equivale a tornar todo o catálogo visível.

**Provas atuais:** 64 testes web dirigidos e TypeScript passaram. Os14 testes
Rust do módulo de preferências passaram, incluindo duas alternâncias concorrentes,
payload ambíguo, falha de persistência e gravação geral desatualizada.3983 testes
não relacionados foram filtrados. A compilação privada fixa1369 entradas.
No próprio binário12230fac, teclas XTest→frontend→IPC→preferência persistida→janela
nativa mostrada/oculta passaram. O controlo Geral mostrou os pets; um reinício
conservou a opção e a janela visível, mantendo os restantes campos das preferências.

Depois, um Codex explicitamente configurado no perfil privado produziu um único
pedido pendente por RPC sintético. Ocultar os pets pelo controlo Geral conservou
a conversa visível, exatamente o contexto imediatamente anterior à ação, o mesmo
pedido, dono, destino e CID. A captura física X11 privada mostra o pedido por
decidir com os pets ocultos. Só depois se usou Stop próprio; não houve aceitação
nem execução do comando. Zero inferência HTTP, providers reais ou perfis pessoais.
As imagens atuais foram observadas; o fundo preto da região transparente mantém
o limite gráfico X11, sem certificar transparência/qualidade no compositor real.

**Tentativas conservadas:** as duas primeiras comparações oficiais pararam numa
pré-condição incorreta do observador (campo ausente no DTO original e greeting).
A terceira observou o ciclo original após sair da saudação. Na privada corrigida,
a tentativa1 pediu um pet ausente do roster e foi corretamente recusada; a2
consultou decisões na janela settings, recusada pela ACL. A3 conservou a conversa
e o mesmo pedido, mas comparou o contexto com uma leitura anterior à criação do
turno. Não capturou a diferença no momento da ação. A4 guardou contexto inicial,
imediatamente anterior a ocultar e posterior: os dois últimos coincidem no mesmo
CID/destino; o inicial ainda não tinha esse binding. Não se certifica a causa
exata da tentativa3 por inferência, nem se alterou produto/ACL para passar os
ensaios. PREPARED contém os brutos e as qualificações.

**Estado:** implementado, integrado na fonte e validado nativamente no âmbito
privado descrito. Não é novo RPM, instalação, ensaio Hyprland/Wayland, aceitação
humana, paridade de todos os atalhos ou conclusão dos49. O RPM51 congelado não
inclui esta correção nem a alternância abrir/fechar do centro. A sessão real foi
consultada: desbloqueada mas com ecrã desligado; nenhuma QA foi iniciada nela.
Segue agrupar estas correções num pacote sucessor próprio, verificar os percursos
afetados nesse executável e obter aceitação da composição real antes de propor
uma substituição operacional. A associação rápida ao terminal mantém o limite
já documentado; não acrescentar infraestrutura sem nova evidência.

[Verificação](../.build/desktop-shortcut-20261117/VERIFIED.json),
[delta](../.build/desktop-shortcut-20261117/DELTA.json),
[tentativas](../.build/desktop-shortcut-20261117/PREPARED.json),
[preservação](../.build/desktop-shortcut-20261117/PRESERVATION.json).

**17/11 — limite de associação do terminal comparado, sem alteração de produto.**
No oficial e na privada atual, peer vivo→janela correta/tecla q; criador terminado
antes do payload→apenas pasta própria. Socket/credenciais/PID nativos, janelas e
EWMH de QA. A captura/associação existente é igual nos dois. Este limite herdado
não prova a causa da tentativa anterior com relay rápido, que permanece aberta.
Zero inferência, atalhos globais ou alterações instaladas. Segue desktopToggle;
não acrescentar cache/protocolo para resolver uma regressão ainda não demonstrada.
[Provas e limites](PETS-UPSTREAM-PARITY-2026-11-11.md#relay-lifecycle-1711).

<a id="calcom-agenda-2111"></a>
## 21/11 — agenda Cal.com e estado de validação

O oficial 0.5.0 e o fork anterior pediam a primeira página de
reservas sem especificar a ordenação no pedido. Ambos aceitavam
uma data inválida como texto e ignoravam reservas sem data. Na
comparação com HTTP sintético, o binário anterior mostrou
«Invalid Date» e apresentou «No calls scheduled» quando faltava
a data ou a API devolvia um estado de erro com uma lista vazia.

O fork passou a pedir ordenação por início no próprio servidor.
O leitor comum verifica o estado da resposta e a data de cada
reserva antes de publicar a agenda. No cenário sintético em que
a primeira página dependia da ordenação pedida, chegou a reserva
mais próxima e o desktop e o telefone mostraram-na primeiro.
[Referência de reservas do projeto Cal.com](https://github.com/calcom/cal.com/blob/main/agents/skills/calcom-api/references/bookings.md#list-bookings).

A janela nativa mostrou a falha de leitura para os três casos
inválidos e conservou o estado de agenda vazia para uma leitura
válida. Um cliente TLS sintético com certificado emparelhado recebeu
a agenda, a retirada do cartão perante erro e a recuperação.
O telefone recebeu os títulos e as horas, sem os dados do
participante fornecidos pela fixture. A prova usou a árvore de
desenvolvimento, um servidor local e a versão de API já configurada.
O RPM 61 foi congelado antes desta alteração.
[Resultados e fontes exatas](../.build/calcom-agenda-20261121/VERIFIED.json).

<a id="notion-titulos-2111"></a>

## 21/11 — títulos compostos no cartão Notion

O parser do Coucou oficial 0.5.0 e o da variante usada como referência
são iguais. Liam apenas o primeiro segmento do título de cada página ou
base de dados. No ensaio, a página «QA Projeto Água — Norte» e a base de
dados «QA Projeto Água — Sul» apareciam ambas como «QA Projeto».

A correção junta os segmentos `plain_text` pela ordem em que chegam.
A janela nativa distinguiu os dois títulos, abriu cada destino através
do capturador local e preservou o estado de título vazio. Quando a lista
vazia voltou a ter resultados, um título cujo primeiro segmento estava
vazio foi apresentado por inteiro. O telefone sintético por TLS recebeu
os mesmos nomes, incluindo a lista vazia e a recuperação, sem os URLs.

Os dois ensaios usaram respostas HTTP sintéticas com autenticação e a
versão da API esperada. A adaptação de destino está na árvore de QA.
A fonte do produto conservou o pedido Notion existente. A correção é
posterior ao instantâneo do RPM 62.
[Recibos da comparação](../.build/notion-titles-20261121/VERIFIED.json) ·
[Contrato de texto formatado da Notion](https://developers.notion.com/reference/rich-text).

<a id="stripe-saldos-2111"></a>
## 21/11 — Stripe separa saldos por moeda e disponibilidade

O poller do oficial juntava os campos `available` e `pending` de todas
as moedas e atribuía o total à primeira. Com os mesmos valores gerados,
a função do oficial, executada no binário de ensaio, produziu `28,60 EUR`
a partir de EUR, JPY, USD, ISK e UGX. A correção mantém os valores
disponíveis e pendentes separados por moeda. Usa as unidades da API de
Stripe e formata os inteiros sem passar por representação em vírgula
flutuante. [Objeto Balance](https://docs.stripe.com/api/balance/balance_object) ·
[Unidades do Stripe](https://docs.stripe.com/currencies#minor-units).

No cartão nativo corrigido passaram o saldo de várias moedas, um valor
acima do limite de inteiros seguros do JavaScript, uma recusa de acesso
à lista de pagamentos, uma resposta de saldo inválida, saldo vazio e
recuperação. Um telefone sintético recebeu por TLS os saldos disponíveis
com moeda e os pagamentos sem descrições privadas. A recusa retirou o
cartão do telefone e a recuperação voltou a publicá-lo. O ensaio nativo
usou HTTP privado e a função do oficial copiada só para a variante de QA.
A correção está na fonte e fica para uma composição posterior ao RPM 59.
[Resultados e âmbito](../.build/stripe-balances-20261121/VERIFIED.json).

<a id="stripe-consulta-em-curso-2111"></a>
## 21/11 — autorização do Stripe durante uma consulta

O código do oficial 0.5.0 e a fonte do RPM .66 guardavam a chave
antes da leitura do saldo e reutilizavam-na para pedir os pagamentos.
No ensaio nativo da fonte anterior à correção, Pause, desativar o pet
ou mudar a chave durante a leitura do saldo ainda permitiam o segundo
pedido com a chave anterior. Mudar a chave durante a leitura dos
pagamentos permitia publicar a resposta antiga. Um erro dessa leitura
também podia aparecer depois de Pause.

A correção usa a condição de admissão existente e confirma a chave
atual antes de cada GET e depois da resposta. Nos três cenários do
saldo houve zero pedidos adicionais. Nos cinco cenários houve zero
atualizações tardias. Uma consulta autorizada voltou a apresentar
saldo disponível, pendente e pagamentos no cartão nativo.

Os dois executáveis de desenvolvimento diferem na correção e usaram
a mesma adaptação privada das origens HTTP. A referência do oficial
neste percurso vem da leitura do código. O ensaio não abrange TLS real
do Stripe nem a instalação operacional.
[Recibos](../.build/stripe-inflight-20261121/VERIFIED.json).

<a id="servicos-resposta-2111"></a>
## 21/11 — falhas de leitura em Vercel, Resend, Notion e Cal.com

Com chaves sintéticas e sem rota externa, o oficial 0.5.0 e o RPM 58
tentaram abrir os quatro túneis HTTP CONNECT. O proxy de teste recusou
as ligações e nenhum dos dois publicou um erro de leitura no evento
nativo do cartão. No RPM 58, os quatro cartões continuaram a indicar
«Connected · loading…». O ensaio visual do oficial em Wayland ficou
inconclusivo. A comparação dos dois pacotes usou o evento nativo como
observação comum.

A fonte passa a publicar a falha de ligação ou de resposta e mantém
os avisos HTTP existentes. Uma lista válida sem elementos tem estado
próprio. O binário de desenvolvimento mostrou nos quatro cartões uma
resposta válida, perda de ligação depois de sucesso, HTTP 401 e 403,
JSON ilegível, estrutura de lista inválida, lista vazia e recuperação.
O cartão ocultou a informação anterior durante a falha e voltou a
apresentar os dados após recuperação. A leitura de Notion usou apenas
a operação de pesquisa já existente.

O ensaio positivo redirecionou os quatro destinos para loopback só na
cópia compilada para teste. Uma execução separada com esse desvio
desligado também mostrou as quatro falhas de túnel. A fonte do produto
conserva os URLs originais. Não houve acesso a serviços reais, mudança
de certificados ou alteração instalada. [Recibos e delta de teste](../.build/service-list-status-20261121/VERIFIED.json).

O próprio RPM 59 confirmou a indicação de falha de ligação nos quatro
cartões com os destinos originais. O ensaio de recuperação conserva o
âmbito do binário de desenvolvimento.
[Verificação do RPM](PETS-VALIDATION.md#rpm59-servicos-2111).

### Atualização de um serviço em pausa ou desativado

**Degradado no RPM .64 e corrigido em desenvolvimento.** Um comando manual chegou ao n8n depois
de Pause e depois de o pet ser retirado da lista de ativos.
A periodicidade automática já respeitava o estado. A fonte pós-.64
faz a verificação comum antes de admitir uma atualização manual.
O ensaio nativo da correção confirmou zero GETs nos dois casos.
A atualização pelo cartão e a agendada mantiveram a leitura de
workflows da instância escolhida após reativação.
Este comportamento de pausa ainda não foi comparado no oficial.
[Reprodução e âmbito](PETS-VALIDATION.md#integracoes-pausa-2111).

<a id="n8n-instancia-2111"></a>
## 21/11 — mudança de instância n8n com IDs de execução iguais

No oficial 0.5.0 e no RPM 58, o cartão mostrou a execução `101` da
instância A. Depois de selecionar B, houve uma leitura autenticada em
B, mas o ID `101` foi tratado como repetido. O cartão continuou com A.
B só apareceu quando a listagem devolveu `102`. O ensaio abriu o cartão
nativo e usou o comando de atualização que a interface usa. As duas
tentativas iniciais de interação ficaram registadas à parte.

A correção identifica a execução pela ligação configurada e pelo ID.
A memória de deduplicação existente guarda apenas o hash. O poller
confirma a ligação antes de aceitar a leitura e após obter o detalhe.
No binário de desenvolvimento, o `101` de B apareceu uma vez e uma
resposta atrasada de A não alterou o cartão de B. Passaram também a
troca de chave, a equivalência da barra final no URL e o detalhe de
erro com regresso ao cartão. O primeiro poll manteve-se silencioso.

As provas usam dois servidores locais gerados, HTTP de leitura, cofre
privado e X11. O RPM 59 confirmou a mudança com ID igual, a resposta
atrasada de A e o detalhe de erro.
[Comparação e recibos](../.build/n8n-instance-switch-20261121-r2/VERIFIED.json) ·
[Verificação do RPM](PETS-VALIDATION.md#rpm59-servicos-2111).

<a id="flow-portal-wayland-2111"></a>
## 21/11 — colagem do ditado por portal em Wayland privado

O Coucou oficial 0.5.0, da árvore `841a81ce`, e o RPM 59 receberam
a decisão de teste por D-Bus. Em cada pacote, o botão **Colar novamente**
inseriu o último ditado três vezes num editor GTK independente. O
editor confirmou as três teclas Ctrl+V com foco. As duas primeiras
colagens usaram a mesma sessão do portal. Depois do reinício, o pacote
abriu outra sessão e apresentou o token que o portal de teste lhe
tinha dado. Ambos repuseram o clipboard anterior e conservaram a cópia
que fizemos durante a espera antes da reposição.

À recusa do portal, os dois pacotes deixaram o editor intacto e o ditado
no clipboard. A 59 mostrou a orientação para colar manualmente. A
colagem está preservada no percurso ensaiado. O portal era uma fixture
num D-Bus privado. O compositor Sway e o editor GTK receberam as teclas
e o conteúdo por Wayland. Nas primeiras tentativas, o observador
procurou o token na pasta antiga, algumas inserções não chegaram ao
editor e a navegação do oficial regressou ao ditado após reinício.
Na comparação final, o observador leu a pasta certa, espaçou as teclas
de teste, confirmou a receção pelo GTK e abriu o menu quando necessário.

Uma consulta separada encontrou o portal GNOME real num compositor
descartável. O portal expôs teclado, `NotifyKeyboardKeysym` e
`ConnectToEIS`. Nessa sessão foi consultada apenas a disponibilidade.
Falta o percurso com consentimento humano e inserção através do portal
real. O Hyprland operacional continua com a alternativa manual no
âmbito da consulta anterior. [Provas](PETS-VALIDATION.md#rpm59-flow-portal-2111).

<a id="flow-copia-wayland-2011"></a>
## 20/11 — histórico de ditado e alternativa à colagem automática em Wayland

No Coucou oficial Linux 0.5.0 e no RPM 57, clicar em **Copiar** nas
entradas antiga e recente fez chegar ao clipboard privado os dois
textos corretos. Uma leitura independente com `wl-paste` confirmou os
bytes, incluindo acentos e quebras de linha. O histórico ficou intacto.

Num D-Bus privado sem o portal RemoteDesktop, **Colar novamente**
deixou o último ditado no clipboard nos dois pacotes. O campo de teste
ficou vazio. No 57, a orientação para colar com Ctrl+V ficou visível
nos dois modos da interface, na barra do Flow e junto aos pets. O ensaio
usou Sway privado e entradas geradas. O ensaio posterior cobre a
inserção num editor GTK com portal sintético.
[Comparação posterior](#flow-portal-wayland-2111).
[Recibos dos dois pacotes](../.build/upstream-parity-20261111/DICTATION-WAYLAND-57-VERIFIED.json).

Na sessão Hyprland ativa da estação, uma consulta sem ativação D-Bus
confirmou que o serviço do portal já estava a correr e que a interface
RemoteDesktop não estava disponível nessa sessão. A consulta não pediu
consentimento e não acedeu ao clipboard da estação.
[Consulta do portal](../.build/upstream-parity-20261111/HYPRLAND-REMOTE-DESKTOP-PROBE-20261120.json).

<a id="flow-portal-hyprland-2111"></a>
## 21/11 — colagem com o portal real do Hyprland

O oficial 0.5.0 e o binário atual de desenvolvimento passaram pelo
botão «Paste again» no Hyprland 0.57.2 isolado. O portal público e o
backend do Hyprland eram os da estação e estavam ativos no D-Bus de
teste. Os atalhos estavam disponíveis, e o portal não oferecia
`org.freedesktop.portal.RemoteDesktop`. A consulta passiva ao portal
da sessão operacional confirmou a mesma ausência.

Ambas as versões disponibilizaram o último ditado no clipboard,
com o histórico e o campo de destino intactos. O desenvolvimento
apresentou a instrução para clicar no destino e premir Ctrl+V.
Uma colagem automática com portal que disponibilize teclado
continua a exigir um ensaio com a escolha explícita no diálogo.
[Resultados com os portais reais](../.build/wayland-layer-qa-20261120/DICTATION-REAL-HYPRLAND-VERIFIED.json).

<a id="flow-navegacao-2011"></a>
## 20/11 — navegação entre dicionário, snippets e estilos

O oficial Linux 0.5.0 e o RPM 57 completaram, cada um, 36 transições
entre as três páginas. A sequência oficial → 57 → 57 → oficial usou
quatro arranques com perfis privados e o mesmo conteúdo gerado. Cada
transição esperou pelos dados da página de destino e por dois callbacks
de animação com o conteúdo presente. Os ficheiros de teste conservaram
os bytes e o perfil não fez pedidos externos detetados pelo ensaio.

| Em navegação, duas amostras por pacote | Oficial 0.5.0 | RPM 57 |
| --- | ---: | ---: |
| Medianas entre clique DOM e segundo callback | 30 / 30 ms | 31 / 30,5 ms |
| PSS mediana da árvore da aplicação | 498,4 / 491,9 MiB | 506,5 / 507,7 MiB |
| CPU da árvore da aplicação, em percentagem de um núcleo | 148,4 / 147,1% | 46,5 / 46,9% |

Neste ensaio curto, o tempo da navegação ficou próximo. O RPM 57 usou
mais memória proporcional e menos CPU na sua árvore de processos.
A amostra foi obtida em Sway privado sem ecrã, com renderização
por software e WebDriver. O tempo começa no clique DOM e não mede a
entrada física nem a apresentação do frame no monitor. As medições
excluem o compositor e o WebDriver. Consumo, fluidez e uso prolongado
no perfil operacional continuam em aberto.
[Amostras e recibos](../.build/upstream-parity-20261111/FLOW-NAVIGATION-57-VERIFIED.json).

<a id="flow-navegacao-60-2111"></a>
## 21/11 — navegação e consumo no RPM 60

O oficial 0.5.0 e o RPM 60 completaram 36 transições cada um entre
o dicionário, snippets e estilos. Os quatro arranques seguiram a
sequência oficial, 60, 60, oficial e usaram a mesma fixture. Os
ficheiros conservaram os bytes e a rede estava limitada ao loopback.

| Duas amostras de navegação | Oficial 0.5.0 | RPM 60 |
| --- | ---: | ---: |
| Mediana até ao segundo callback | 45 / 44,5 ms | 45,5 / 46,5 ms |
| PSS mediana da aplicação | 770,1 / 762,9 MiB | 738,9 / 749,6 MiB |
| CPU da aplicação, em percentagem de um núcleo | 599,6 / 586,2% | 483,9 / 488,8% |

Nesta amostra curta, o tempo ficou próximo e o RPM 60 usou menos
memória proporcional e CPU. A carga total da máquina variou entre
amostras. O ensaio usou Sway isolado, sem ecrã e com renderização
por software. A comparação com a série do RPM 57 tem condições
diferentes. Falta medir a experiência instalada e o uso prolongado.
[Amostras e recibos](../.build/wayland-layer-qa-20261120/FLOW-NAVIGATION-60-VERIFIED.json).

<a id="recursos-instalada-62-2111"></a>
## 21/11 — consumo da sessão instalada .62

Durante cinco minutos, a observação passiva do executável que ficou
instalado mediu **1 199,5 MiB de PSS mediana** na árvore da aplicação.
As 61 amostras ficaram entre 1 148,3 e 1 258,2 MiB. O consumo médio de
CPU foi **114,9% de um núcleo**, dos quais 87,4% nos processos WebKit
de conteúdo. O conjunto de processos permaneceu estável.

A amostra decorreu com a barra e os centros de pets e de sistema
configurados na sessão operacional. A atividade do utilizador e a
visibilidade das vistas não foram controladas. Este resultado mede o
consumo observado nessa sessão. A comparação com o oficial usa um
perfil isolado e uma carga igual para os dois executáveis.
[Amostra instalada](../.build/installed62-usage-20261121/RESULT.json).

## 21/11 — amostra da .63 após novo arranque

O serviço de login executou o RPM .63 pelo launcher da conta.
Em 60 s, 13 amostras da aplicação e dos seus processos filhos
registaram uma PSS mediana de 950,0 MiB, entre 901,2 e 1 009,0 MiB.
A CPU média foi 106,3% de um núcleo, dos quais 73,6% nos processos
WebKit de conteúdo. O conjunto de processos manteve-se estável.

A visibilidade do Hub, o trabalho dos pets e a carga do anfitrião
ficaram fora do controlo experimental. A amostra caracteriza a sessão
observada. A comparação controlada do issue #1 continua a ser a das
rondas privadas emparelhadas.
[Observação da .63](../.build/installed63-sample-20261121/RESULT.json).

<a id="flow-navegacao-62-2111"></a>
## 21/11 — reconexão do microfone escolhido

| Percurso | Classificação | Comportamento observado |
| --- | --- | --- |
| Microfone retirado e recolocado durante «OK Coucou» | Degradado na .62 e corrigido em desenvolvimento | A .62 exigiu desligar e voltar a ligar a escuta. A variante verificou o identificador nativo e retomou a escuta no microfone escolhido. Recusou um segundo dispositivo com o mesmo nome. |
| Continuidade e Stop na mesma sessão | Preservado no ensaio privado da variante | O novo pedido recebeu o contexto anterior. Durante a espera, Stop impediu a reabertura e a repetição do pedido. O reinício preservou a conversa. |
| Indicação quando a captura termina após perder o microfone | Degradada na .66 e corrigida em desenvolvimento | Na .66, a preferência ficou ligada e a página de Voz ficou sem indicação da captura. A compilação de desenvolvimento mostrou a escuta, a espera pelo mesmo dispositivo e a inatividade após o prazo. Validado com áudio virtual. |
| Reconexão no oficial e no fork após perda breve | Melhorado no fork no ensaio nativo | Com a escuta estável, o oficial 0.5.0 ficou oito segundos sem ligação após voltar a mesma origem com outro número de série. Desligar e voltar a ligar recuperou a captura. A .66 e a compilação de desenvolvimento retomaram automaticamente em menos de um segundo. Tanto o oficial como o fork libertaram o microfone ao desligar. |
| Microfone físico e Bluetooth | Ainda não verificado | O ensaio usou duas origens PipeWire virtuais e uma política de ligação ao dispositivo explicitamente escolhido. |

[Reconexão e continuidade](PETS-VALIDATION.md#voz-reconexao-2111) ·
[Estado da captura](PETS-VALIDATION.md#hotword-estado-2111) ·
[Comparação nativa com o oficial](../.build/upstream-hotword-replug-20261121/VERIFIED.json).

## 21/11 — animação do Hub em repouso

| Percurso | Classificação | Oficial e fork |
| --- | --- | --- |
| Animação, navegação e reabertura do Hub | Preservado no ensaio nativo | Dicionário e General conservaram os seus controlos e animação. |
| Cadência do cabeçalho sem interação | Alterado intencionalmente | O código oficial 0.5.0 pede um frame a cada atualização do ecrã. A .62 manteve esse comportamento. O RPM .63 conserva cerca de 10,4 frames/s em repouso e retomou cerca de 61 frames/s com o ponteiro. |
| CPU do Hub visível | Degradada no cenário original do issue, corrigida nos RPMs .63 e .64 | Três rondas curtas, com os RPMs exatos .62 e .63 e Sway privado por software, mediram uma redução mediana de 71,4%. As medianas com Hub oculto foram 28,10% e 26,85% de um núcleo. A .64 conserva o mesmo código de renderização. |
| CPU e memória na sessão operacional | Amostradas na .63 após o novo arranque | A amostra passiva observou a versão lançada pelo serviço de login. A comparação controlada com a .62 na estação continua em aberto. |

[Medições, comportamento e limites](PETS-VALIDATION.md#hub-cpu-issue1-2111).

## 21/11 — navegação espaçada no oficial e na .62

O ensaio repetiu duas vezes cada executável em perfis privados, na
ordem oficial, .62, .62, oficial. Cada execução teve 25 segundos de
repouso e 70 segundos de navegação, com 42 mudanças entre dicionário,
snippets e estilos. O mesmo conteúdo sintético ficou intacto.

| Navegação | Oficial 0.5.0 | RPM 62 |
| --- | ---: | ---: |
| Mediana até ao segundo callback | 45,5 / 46,0 ms | 43,5 / 43,0 ms |
| Percentil 95 até ao segundo callback | 63,9 / 83,2 ms | 66,9 / 65,0 ms |
| PSS mediana da aplicação | 758,7 / 754,2 MiB | 733,8 / 735,4 MiB |
| CPU média em percentagem de um núcleo | 556,1 / 565,3% | 468,2 / 465,3% |

As medianas de navegação ficaram próximas. Nas duas repetições, a .62
usou menos CPU e memória proporcional para o mesmo percurso. A carga
do anfitrião variou entre amostras. O Sway privado usou renderização
por software. A latência mede o renderer e os seus callbacks, enquanto
a amostra da instalação representa a sessão operacional que estava em uso.
[Série e âmbitos](../.build/installed62-usage-20261121/FLOW62-SUSTAINED-VERIFIED.json).

Num ensaio separado, a .62 desenhou miniaturas escondidas nas vistas
compacta, galeria e detalhe. Na barra compacta houve 1 440 desenhos
escondidos e 576 visíveis em 2,5 segundos. A fonte atual salta as
miniaturas sem caixa de layout antes de abrir o contexto de desenho.
Nas três vistas privadas, houve zero desenhos escondidos e todas as
miniaturas visíveis pintaram. A passagem da galeria ao detalhe e a
volta também passaram. O ganho de CPU na sessão instalada continua por
medir numa composição que inclua esta alteração.
[Comparação dos desenhos](../.build/mini-visible-20261121/VERIFIED.json).

<a id="relay-terminal-2011"></a>
## 20/11 — foco da sessão com o relay original no RPM 57

O Coucou oficial Linux 0.5.0 (`841a81ce`) e o RPM 57 exato receberam os
eventos pelo socket local com o mesmo binário do relay distributivo.
Em cinco sessões novas por pacote, a operação nativa `open_session`
selecionou a janela da pasta certa e essa janela recebeu uma tecla de
teste. A segunda janela, do mesmo processo mas com título de outra
pasta, nunca recebeu foco. O peer mantido vivo também passou. Quando
o processo que ligou o socket terminou e foi recolhido antes de chegar
o evento, ambos os pacotes enviaram apenas a pasta ao capturador de
editor, mesmo com as janelas ainda abertas.

O ensaio usou X11, socket e IPC nativos, com D-Bus e perfis privados,
um gestor EWMH de teste, janelas próprias e um capturador em lugar do
editor. Neste âmbito, o comportamento está preservado e a limitação
do peer terminado vem do oficial. A causa da falha intermitente
anterior e o foco num terminal real em Wayland continuam por verificar.
[Recibos dos dois pacotes](../.build/upstream-parity-20261111/RELAY-TERMINAL-57-VERIFIED.json).

<a id="relay-terminal-wayland-2111"></a>
## 21/11 — abrir a sessão no WezTerm nativo em Wayland

O oficial 0.5.0 (`841a81ce`) e os RPM 59 e 60 receberam pelo relay
distributivo uma sessão iniciada num WezTerm real. Um segundo WezTerm
ocupava o foco. A ação nativa identificou o contexto da sessão e
entregou uma única vez a pasta exata ao capturador de editor nos três
pacotes. Nenhuma das duas janelas mudou de foco.

A ausência de foco neste percurso é uma limitação comum. O ensaio
usou Sway privado sem XWayland, sem rede externa e sem iniciar o editor
real. O foco nativo em Hyprland e os terminais com mecanismos próprios
continuam a precisar de ensaio. A prova X11 acima conserva o seu âmbito.
[Resultados dos dois pacotes](../.build/wayland-layer-qa-20261120/TERMINAL-WAYLAND-VERIFIED.json).

<a id="relay-terminal-hyprland-2111"></a>
## 21/11 — foco da sessão em Hyprland

O oficial 0.5.0 e o RPM 60 repetiram, no Hyprland 0.57.2 isolado,
o comportamento observado no Sway. Receberam os hooks de um WezTerm
real, encaminharam apenas a pasta para o capturador do editor e
deixaram o foco no segundo WezTerm. A comparação identifica uma
limitação comum no percurso Wayland nativo.

O binário privado com a adaptação confirmou pelo compositor a mudança
de foco para o WezTerm associado ao relay. Depois de fechar o terminal,
a mesma ação encaminhou uma única vez a pasta para o editor. O backend
reutiliza o socket local de Hyprland e confirma o endereço e o processo
da janela antes do foco. [Resultados da comparação e da correção](../.build/hyprland-terminal-focus-20261121/BASELINE-AND-NATIVE-VERIFIED.json) ·
[Dispatcher do Hyprland](https://wiki.hypr.land/0.57.0/configuring/core/dispatchers/).

No binário de desenvolvimento seguinte, dois WezTerms enviaram hooks
para Codex e Copilot com o mesmo ID de sessão. Mantendo o processo do
peer vivo durante um evento de observação, o compositor confirmou a
janela certa para cada backend. O botão «Abrir o terminal desta sessão»
do chat flutuante focou o WezTerm Codex. Depois de o fechar, o comando
original continuou a abrir só a pasta. A prova da interface começou
pela abertura nativa do chat com a sessão externa escolhida.
[Provas e limites](PETS-VALIDATION.md#hyprland-foco-2111).

A correção seguinte cobre também a saída imediata do relay. O
mesmo ensaio com Codex e Copilot passou três vezes no desenvolvimento
corrigido, depois de falhar três vezes na composição anterior.
A data de arranque do ascendente observado permite distinguir
se o seu PID foi entretanto reutilizado.
O executável do RPM 61 repetiu os três ensaios com sucesso no Hyprland
isolado. No X11 privado, a terceira tentativa de cada série ficou
sem confirmação do foco ou abertura da pasta.
[Provas do pacote](PETS-VALIDATION.md#rpm61-nativo-2111).

<a id="relay-lifecycle-1711"></a>
## 17/11 — fronteira de vida do processo que liga o relay

Comparação delimitada no oficial0.5.0/841a81ce e na privada7d274c3c atual.
A função note_session_window, session_window.rs e platform/linux_focus.rs são
iguais. O socket obtém o PID do peer pelo kernel; a janela é procurada na cadeia
de processos disponível quando o hook é tratado. Não se confia num PID enviado
no payload para escolher a janela.

Dois casos próprios por aplicação: um filho vivo ligou o socket e permaneceu
vivo até o processamento; a janela correta recebeu foco e uma tecla q efetiva.
Noutro, o filho ligou o socket, passou o descritor ao processo pai por SCM_RIGHTS,
terminou e foi recolhido antes de o pai entregar o JSON. As duas janelas do pai
continuavam disponíveis, mas ambas as aplicações abriram apenas a pasta no
capturador code. PID/credenciais/socket, backend e efeitos X11 são nativos; peer,
janelas e gestor EWMH são fixtures declaradas. Não é um IDE, shell de produção,
Hyprland ou ensaio da quota de um provider. Não houve inferência/decisão nem
registo de atalhos globais; os dois efeitos de pasta foram só no programa de QA.

Isto demonstra um limite herdado nesta fronteira de vida. **Não estabelece que
a tentativa fixed2 anterior tenha seguido exatamente esta sequência**, nem que
todos os casos rápidos sejam equivalentes. O relay original e a sua confiança
continuam intactos. O caso anterior fica conservado e sem causa certificada;
fiabilidade prolongada/real continua aberta. Não foi criado um cache, formato,
protocolo ou autoridade adicional com base nessa hipótese. Capturar toda a árvore
em todos os eventos também teria custo não demonstrado como necessário.

[Verificação e recibos](../.build/fast-relay-review-20261117/VERIFIED.json),
[preparação e protocolo](../.build/fast-relay-review-20261117/PREPARED.json).
Próxima comparação funcional delimitada: desktopToggle, antes de agrupar nova
candidata normal com as correções existentes. A sessão gráfica real foi observada
disponível, mas o ecrã desligou antes do arranque da QA atual: nenhuma aplicação
foi iniciada nessa tentativa e não há aceitação gráfica nova.

**17/11 — recuperada a alternância abrir/fechar no centro dos pets, em composição privada.**
O oficial alternou quatro pressões; o RPM51 abriu sempre. A correção consulta o
contexto nativo e reutiliza a recolha existente, sem usar o modo da vista legada.
46 web/tipos, compilação e percurso nativo privado passaram, incluindo chamadas
antigas de abertura e recusa de pet inválido. O RPM51 não contém esta correção.
Terminal: foco/tecla/pasta passaram no âmbito controlado; fallback com relay rápido
na privada continua sem causa demonstrada. Instalada44 e RPM49/50/51 preservados.
[Contrato, comparação e limites](PETS-UPSTREAM-PARITY-2026-11-11.md#ilha-terminal-1711).

<a id="ilha-terminal-1711"></a>
## 17/11 — abrir/fechar a ilha e abrir a janela da sessão

**Regressão confirmada e corrigida na fonte:** no oficial0.5.0/841a81ce, o atalho
toggleIsland alterna expandido/recolhido. No RPM51 exato fae4139c, abre o centro
dos pets em todas as pressões. A escolha do centro atual é compatível com a
arquitetura; perder o gesto de recolher não é necessário a essa escolha. No
mesmo protocolo de observação, oficial2 produziu true/false/true/false,
normal51-2 true/true/true/true e privada7d274c3c true/false/true/false.

O atalho envia agora `toggle: true` opcional na operação existente pet_panel_open.
Rust consulta o contexto nativo aceite; se expandido, usa o mesmo hide/collapse
do controlo de fechar, senão abre a secção existente. Os chamadores antigos sem
o argumento continuam a abrir explicitamente; duas chamadas e a recusa de um
pet inválido sem mudar a revisão passaram nativamente. Coordenadas/pet/secção
continuam validados antes da alternância. Não há novo comando IPC, formato
persistente, ledger, autoridade, provider ou modelo. O argumento IPC novo é
opcional; a política da fábrica por domínio e os seus dados ficam intactos.

**Terminal, âmbito real das provas:** oficial1 e normal51-1 usaram o relay
original92bf5488, um processo de QA com duas janelas X11 e um EWMH mínimo sintético.
A tecla global escolheu a janela cujo título correspondia à pasta da sessão;
uma tecla q chegou efetivamente à janela correta, sem selecionar a janela-isca.
Retirar ambas as janelas fez chegar apenas a pasta própria ao capturador code,
sem abrir editor real. O mesmo passou nos três casos de comparação com um peer
filho mantido vivo: socket, PID/ancestralidade, backend, mensagem EWMH e foco são
nativos; o peer e o gestor EWMH são fixtures explícitas. Não é uma prova de
terminal/IDE/gestor de janelas reais, WezTerm ou Hyprland.

**Limites e tentativas conservadas:** a primeira privada não selecionou a sessão
e parou antes das ações; a segunda recebeu os hooks/selecionou a sessão, mas
abriu a pasta apesar da janela existir. A causa exata não foi demonstrada. O
backend do terminal é igual ao do oficial; o sucesso com peer mantido vivo não
explica nem corrige o fallback observado com o relay rápido. A fiabilidade dessa
associação permanece aberta. A terceira confirmou o terminal e falhou após
mudanças de foco entre observações. A quarta guardou o estado nativo antes de
cada tecla, manteve uma webview observadora e o ponteiro fora do popup. A
comparação oficial2/normal51-2 repetiu este protocolo para separar a correção de
uma alteração do observador. Não se afirma qual mudança de observação explica
isoladamente o resultado anterior. Nenhuma inferência, decisão ou comando de
utilizador foi executado; o capturador code é um programa sintético próprio.

41 testes de atalhos e5 de destinos, TypeScript e compilação privada passaram.
Os testes nativos exercitam também o argumento omitido e as validações do comando
Rust; não se repetiu a suite Rust geral. O código de contexto/autoridade/collapse
foi reutilizado sem alterações. A fonte compilada está fixa em SOURCE1; a
documentação posterior fica em SOURCE2, sem alteração de produto após aceitação.

As imagens X11 privadas mostram o centro aberto e o conteúdo recolhido; há um
fundo preto na região transparente, pelo que não certificam pintura/transparência
ou qualidade gráfica geral. A aceitação no compositor real continua pendente.
O artefacto privado tem versão interna51, mas **não é o RPM51 congelado**.
A proposta51 fica suspensa; preservar esse pacote e preparar uma sucessora
própria após fechar as lacunas funcionais delimitadas, sem instalar ou publicar.
Todos os49 requisitos e critérios originais mantêm os respetivos âmbitos.

[Verificação agregada](../.build/island-terminal-shortcuts-20261117/VERIFIED.json),
[fontes/delta](../.build/island-terminal-shortcuts-20261117/DELTA.json),
[tentativas/qualificações](../.build/island-terminal-shortcuts-20261117/PREPARED.json),
[preservação](../.build/island-terminal-shortcuts-20261117/PRESERVATION.json).

**17/11 — RPM51 com a correção dos atalhos verificado, sem instalação.**
No próprio executável embalado passaram pedido CLI/dono/CID, duas reaberturas sem
decisão, Stop/retirada e navegação legada equivalente ao oficial0.5.0. Um turno
RPC sintético, zero modelos reais;44 web/tipos reutilizados por código exato.
Instalada44 e RPM49/50 preservados. A proposta50 foi sucedida; Wayland real,
âmbito próprio para entrega operacional, paridade integral e49 continuam abertos.
[Artefacto, provas, limites e próximo passo](PETS-INSTALLED-DELIVERY-51.md).

**17/11 — atalhos comparados com o oficial; corrigida a reabertura de pedidos CLI.**
Avançar/recuar entre pets e voltar a permissões/perguntas passaram no oficial0.5.0
e na composição privada corrigida. No RPM50 congelado, o atalho era recebido mas
não reabria o pedido CLI ainda ativo. A correção reutiliza o destino existente;
reabriu o mesmo pedido/CID duas vezes, sem decidir, e deixou de o reabrir após
Stop.44 testes web/tipos e percursos nativos privados passaram. Instalada44 e
RPM50 preservados; o RPM50 não contém esta correção. Segue um pacote novo,
antes de proposta operacional. Sem quota real ou aceitação Wayland/humana/global.
[Contrato, comparação, provas e limites](PETS-UPSTREAM-PARITY-2026-11-11.md#atalhos-pedidos-cli-1711).

<a id="atalhos-pedidos-cli-1711"></a>
## 17/11 — navegação global e pedidos da conversa

Comparação delimitada com o RPM oficial0.5.0, fonte841a81ce1d75b65882b30139e6b716d0837dce30
em árvore separada. O oficial e o binário privado corrigido71e4889d passaram
Ctrl+Alt+Right/Left com ciclo/retorno inverso e Ctrl+Alt+A para uma permissão e
uma pergunta recebidas pelo socket nativo. O peer original continuou sem resposta.
O workspace Documents representa o Claude ativo nesta fixture; os outros donos
observados são Codex, n8n e Ollama. Não foi reposto um catálogo de pets sem uso.

**Problema confirmado:** no executável exato do RPM50 (046989dd), ocultar a
conversa com um pedido Codex ainda pendente e usar Ctrl+Alt+A não a reabria.
O backend recebeu o atalho; o pedido/dono/CID continuavam ativos. O frontend só
consultava a permissão/pergunta legada. A correção acrescenta o primeiro destino
retido da coleção existente e chama a apresentação canónica do pedido, com o seu
ID/target. Conserva prioridade legada, dono/CID, configuração e política dos
domínios. A ordem é a da coleção, não uma garantia cronológica global.
Não acrescenta ledger, formato, autoridade, backend ou modelo; não responde ao
pedido. O transporte CLI atual não existe no oficial: a falha é de integração do
contrato de navegação útil do original na arquitetura do fork.

**Validação efetiva:** negativa nativa no RPM50;44 web (39 atalhos/5 destinos) e
TypeScript passaram; compilação privada com1367 entradas explícitas; mesmo caso
nativo passou. Uma segunda execução reabriu o mesmo pedido duas vezes, confirmou
que continuava sem resposta durante a navegação, acionou Stop no próprio turno e
verificou a retirada do pedido e ausência de reabertura. Apenas cinco turn/start
RPC sintéticos em todas as tentativas CLI desta unidade, sem processo de modelo,
HTTP de inferência, aprovação ou execução do comando apresentado. A comparação
legada atual não iniciou turno. Stop pode cancelar/recusar a origem; não é uma
aprovação. Imagens X11 reais do ambiente virtual privado foram observadas com a
permissão CLI e as duas apresentações legadas legíveis.

Tentativas de observação anteriores ficam conservadas: CID inicial ausente foi
obtido pelo leitor nativo usado pelo frontend; hide genérico foi recusado pela
ACL e substituído pela API existente pet_chat_hide; o clique que ocultava a sua
webview bloqueava a resposta WebDriver. A visibilidade passou a ser lida noutra
janela antes de selecionar a conversa oculta. No oficial, foram corrigidos apenas
o reconhecimento do workspace Claude e a suposição errada de classes CSS. A
captura original cortada não certifica pintura gráfica. Não se alteraram ACLs ou
políticas para facilitar os ensaios. O runner Node direto foi necessário porque
o wrapper filho --test, neste sandbox, só devolvia uma falha de ficheiro sem
subtestes; os44 testes foram realmente executados pelo runner direto.

O artefacto privado71e4889d mantém versão interna50 mas **não é o RPM50** e não
foi instalado. O RPM50 permanece arquivado e a proposta50 não deve ser aplicada
como se incluísse a correção. Próximo: pacote normal de identidade própria,
verificação dos percursos afetados no seu executável, depois compositor real e
âmbito operacional autorizado. As provas anteriores de voz/código/diffs/telefone
mantêm os seus limites; não se declara paridade integral ou49 concluídos.

[Verificação agregada](../.build/shortcut-routing-20261117/VERIFIED.json),
[negativa nativa](../.build/shortcut-routing-20261117/NATIVE-RED.json),
[brutos/qualificações](../.build/shortcut-routing-20261117/QUALIFICATIONS.json),
[preservação](../.build/shortcut-routing-20261117/PRESERVATION.json).

**17/11 — chat do telefone verificado no próprio RPM50, sem alteração de produto.**
O probe de sete pedidos sob consultas readonly e o ciclo completo TLS/conversa/
reset/cancelamento/permissão/modelo/503/reinício passaram.14 POSTs sintéticos,
zero modelo real; instalada44 e RPM50 preservados. As falhas pré-HTTP22/82
continuam conservadas, com causa não demonstrada; não foram reproduzidas neste
âmbito50. Não é aceitação prolongada, física, humana ou global.
[Comparação com oficial0.5.0, provas e limites](ANDROID_LINK.md#phone-chat-normal50).

**17/11 — candidata50 normal preparada e verificada em QA, sem instalação.**
O executável do RPM50 passou contexto n8n/Codex por domínio, revisão própria de
uma rotina→captura/contexto12→admissão32→Driver→uma resposta sintética, consulta
e reinício sem replay. Histórico atual (8 objetivos/4 contextos12, decisões e
reserva65536) e6/7 antigos passaram leitura/recusas, após18 comandos nativos de
restauro/quarentena/revisão/ativação com grants[]. Resposta e decisão restaurada
foram vistas após reinício na captura X11 virtual real e no WebDriver. Instalada44,
preferências e artefacto49 preservados; sem quota real, aceitação humana ou49
integrais. Wayland continua bloqueado; a entrega operacional exige âmbito próprio.
[Artefacto, falhas conservadas, provas e proposta](PETS-INSTALLED-DELIVERY-50.md).

# Valor do Coucou oficial: comparação delimitada e correções

**17/11 — dados de projeto/conversas preservados no restauro atual.**
Catálogo, contexto dos objetivos e resposta/proveniência Codex2 passaram dois
restauros nativos e reinícios sem replay, no binário privado atual. Esta prova
de compatibilidade não encerra paridade integral nem entrega instalada.
[Resultado e limites](PETS-PROJECT-RECOVERY-2026-11-17.md).

**17/11 — legibilidade dos widgets: correção integrada, aceitação gráfica aberta.**
O assunto Resend e as datas de serviço passam a usar o tom secundário existente.
As mesmas cores escuras estão no oficial841a81ce: é uma limitação herdada,
não uma regressão demonstrada do fork. No cartão nativo, o cálculo com cores
planas/alpha passou de2,44/3,91 para6,30; não inclui gradientes/composição nem
certifica acessibilidade global. Os seis percursos passaram, com HTTP n8n local,
detalhe/retorno/erro, reinício e retirada de chave; restantes cinco usam eventos
sintéticos, sem contas remotas. As imagens X11 ficaram parcialmente pretas/cortadas
mesmo após posicionar só janelas privadas: não foram aceites graficamente.
O compositor operacional está ocupado e foi apenas consultado. Não há alteração
de backend/domínio, instalação ou pacote49, nem chamadas reais de modelo.
[Fontes, medições, brutos e limites](../.build/service-card-contrast-20261117/VERIFIED.json).

**17/11 — MCP do Codex: configuração visível, sem chamada de ferramenta.**
O executável Codex encontrado no PATH do processo exato da instalada44, com o
seu HOME/CODEX_HOME, leu openaiDeveloperDocs ativo no URL pedido. Nenhuma
configuração foi alterada. A revisão das fontes atuais confirma herança do
perfil/MCP no app-server e preservação da política nativa ao retomar; não houve
thread, inferência, refresh de conta ou execução MCP. Não certifica ferramentas
disponíveis numa conversa já retomada nem paridade integral dos perfis.
[Âmbito e fontes exatas](../.build/codex-native-profile-parity-20261117/VERIFIED.json).

**17/11 — acesso direto à voz recuperado no dashboard LLM, após o pacote49.**
O Hub e o motor de voz já estavam preservados; faltava uma entrada direta no
cartão atual quando os pets flutuantes estão desligados. Um botão passa a usar
o routing de voz existente da fábrica, sem mudar backend ou política de domínio.
Na compilação privada `ffb3cad0…`, clique nativo no Ollama com provider global
Codex passou ASR→modelo/broker sintéticos→resposta/TTS→fim natural→Stop no Hub→
reinício, com mesmo CID e restantes escolhas. n8n não mostra chat/voz genéricos.
Três testes dirigidos,12 de cartões de serviço e TypeScript passaram. A prova
usa áudio virtual; não certifica hardware, modelos reais ou instalada. Os brutos
falhados de composição/prontidão/observador estão preservados e qualificados.
**Não está no RPM49 congelado nem na instalada44.**
[Provas, imagens observadas e limites](../.build/voice-card-entry-20261116/VERIFIED.json).

## Estado atual16/11

**Candidata49 preparada16/11 — não instalada nem autorizada.** O próprio RPM
passou contexto n8n/Codex por domínio, calendário/reinício/sucessão/revisão/
renovação,18 comandos de restauro com grants[] e respostas históricas6/7/10,
e voz com ASR/TTS reais exclusivamente virtuais, modelo sintético e reinício.
Fontes1332 exatas, dados/identidades/modelos preservados, metadados de suspensão/
proveniência qualificados. Zero quota de modelo real; sem aceitação humana,
física, instalada49 ou paridade global. Instalada44/Source35 intacta.
Substitui a proposta48; exige autorização própria e registo focado/assinado.
[Artefacto, provas, limites e aplicação proposta](PETS-INSTALLED-DELIVERY-49.md).

**Código/diffs48:** seis percursos nativos atuais passaram com o relay original,
Back/Escape e centro, em X11 privado e sem inferência. O retorno no compositor
operacional continua aberto: a tentativa atual parou antes de iniciar a app por
falta de região livre, preservando as superfícies existentes. Não foi necessária
alteração de produto. [Provas e falhas preservadas](../.build/diff-return48-20261116/VERIFIED.json).


**Contexto por domínio aceite no RPM49:** n8n→widget próprio sem contagens CLI,
Codex→Sessões, com cliques nativos entre pets. A política é a existente nos moldes;
nenhuma regra foi acrescentada à base. O mesmo49 passou ciclo de calendário,
restauro nativo6/7/10 e voz por API com ASR/TTS virtuais, resposta e reinício.
Não há aceitação instalada ou global. [Provas/limites49](PETS-INSTALLED-DELIVERY-49.md).
A comparação original0.5.0 mantém referência841a81ce e os âmbitos anteriores;
recursos/diffs48 abaixo não são medições/ensaios49.

## Recursos atuais16/11 — comparação delimitada48/oficial e48/44

Oito arranques normais sequenciais, dois blocos ABBA, perfis privados; Hyprland
Wayland sem automação, foco ou interação com janelas. Configuração comum sem
companions/controlos/Flow e chat nunca aberto. Por caso:8s de aquecimento,25 amostras
durante cerca13s; conjunto de processos estável, PSS legível e0 inferências.

| Bloco | Executável | PSS médio (MiB) | CPU média (% de um núcleo) | WebKit |
| --- | --- | ---: | ---: | ---: |
| Oficial/48 | Oficial0.5.0 | 385,08 | 101,43 | 4 |
| Oficial/48 | Candidata48 | 426,01 | 0,51 | 5 |
| 44/48 | Mesmos bytes da instalada44, perfil privado | 431,14 | 0,71 | 5 |
| 44/48 | Candidata48 | 431,89 | 0,63 | 5 |

As superfícies oficial/fork diferem720×320/720×560: a CPU não prova maior fluidez
ou eficiência global. Há cerca40,93 MiB adicionais e um WebKit adicional neste
perfil, sem atribuir causalidade apenas à contagem de processos. No bloco44/48,
geometria e contagem coincidem: diferença média+0,76 MiB e sem aumento medido de CPU.
São só duas execuções por versão, não prova estatística de ausência de regressões.
Não foram medidos GPU/VRAM/energia, fugas longas, percursos ativos ou uso instalado;
o restante trabalho do anfitrião não foi controlado.78 identidades de processos
medidos foram verificadas terminadas; instalação/preferências mantiveram hashes.
[Métricas, âmbito e recibos](../.build/resources48-20261116/VERIFIED.json).


<!-- n8n-role-clarification-20261113 -->
**Papel do n8n:** a integração existente consulta execuções, detalhes e resultados
de workflows. Não há nesta entrega criação, edição ou execução governada de
workflows demonstrada. A entrada `integration_n8n` da fábrica é idêntica ao HEAD
publicado: serviço associado ao Codex por configuração predefinida. A conversa
pertence ao pet da nossa aplicação, não ao n8n. O único turno sintético usado
na reprodução verifica que selecionar essa sessão não substitui os dados do
workflow; não acrescenta uma funcionalidade de conversa nem comprova agentes
orquestrados pelo n8n. Não se ativou n8n no perfil operacional. O papel útil do
pet deve orientar a evolução; a conversa não é requisito para observar o serviço.
<!-- end-n8n-role-clarification-20261113 -->

<!-- factory-domain-rule40-20261113 -->
**Regra vigente do proprietário:** a recuperação do oficial respeita o domínio
representado pelo pet. Conversa, modelos e métricas de inferência só se recuperam
onde existe um papel LLM e um adaptador/configuração aplicável. Serviços mantêm
as funções próprias; não se tenta repor um chatbot genérico em n8n, Notion,
Vercel, Resend, Cal.com, Stripe ou Spotify. Não se repõem pets sem uso.

A política é declarada nos moldes/domínios `llm` e `application`, fora da base
comum. As extensões de conta não ultrapassam o limite. Esta primeira aplicação
fecha conversa/listagem e seleção de modelos/admissão/controles de inferência;
a adequação das restantes funcionalidades aos domínios continua a ser auditada.
Identidade, memória, dados e mecanismos comuns de objetivos não são eliminados.

A Source40 implementa o limite no resolvedor Rust/TypeScript, capacidades,
backend e controlos. Os defaults serializados são conservados para compatibilidade;
referências antigas a Codex nos serviços não concedem nova inferência. História
permanece legível. Foi também fechado o carregamento/envio pela conta global
quando um serviço sem chat está selecionado, incluindo cartões independentes.
98 testes frontend e64 testes Rust dirigidos passam. O binário privado88/40
passou recusas n8n, workflow/TLS/retirada/reinício e um turno apenas no pet LLM
Codex, com mesmo CID sem replay; todo o registo antigo n8n de QA ficou igual.
FACTORY-DOMAIN40-VERIFIED.json. A imagem n8n está escura: só o gate DOM/IPC/TLS
funcional está aceite. Sem pacote normal40, instalação ou aceitação integral.

Normal46/Source39 fica arquivada: o ensaio foi interrompido por esta instrução
antes do gate inicial, com zero pedidos n8n e zero turnos. Não é falha do produto
nem aceitação nativa. A próxima entrega deve incluir a política de domínio,
sem aproveitar autorização35 ou a pergunta pendente do RPM45.
<!-- end-factory-domain-rule40-20261113 -->

<!-- domain-header42-20261113 -->
**Fonte42/normal47 em preparação; instalada35/44 preservada.** A revisão do
cabeçalho encontrou “Sessões deste pet” nos serviços, apesar de a fábrica já
retirar esse controlo. A negativa usou o cabeçalho real e falhou nessa asserção;
o controlo LLM passou. A42 faz o componente existente respeitar
`presentation.sessions`, com um ficheiro de produto e o seu teste alterados.
102 testes frontend e TypeScript passam; os64 testes Rust da40 são reutilizados
porque todos os fontes Rust/fábrica/defaults são iguais. Sem novo formato,
permissão, modelo, provider, conta ou migração. Conversas antigas preservadas.

O RPM47/41 compilado fica arquivado, sem proposta/instalação: não contém a
correção do cabeçalho. O build47/42 está em curso; ainda não é aceitação nativa
deste pacote. Os recibos40 mantêm o seu âmbito privado anterior. Próximo:
validar o executável final e a apresentação usando controlos nativos existentes,
depois preparar revisão própria de entrega. Sem commits do fork/push/merge.
<!-- end-domain-header42-20261113 -->

## Âmbito e referências

Esta entrega complementa o objetivo dos pets autónomos. Recupera comportamentos
úteis do original dentro da arquitetura atual; não repõe a aplicação antiga nem
exige todos os pets do seu catálogo. A fábrica continua orientada aos pets que o
proprietário pretende usar. Permanecem os 49 requisitos e as decisões de
identidade, inferência explícita, runtime comum e permissões seletivas.

Prioridade reforçada pelo proprietário: verificar e recuperar as componentes
úteis do original antes de acrescentar capacidades ou infraestrutura. A
preservação integral **ainda não está demonstrada**. Código presente, build e
testes isolados não bastam para classificar um percurso como preservado.

- Oficial: [Coucou Linux 0.5.0](https://github.com/Louis-CFM/coucou/releases/tag/linux-v0.5.0),
  commit [`841a81ce1d75b65882b30139e6b716d0837dce30`](https://github.com/Louis-CFM/coucou/tree/841a81ce1d75b65882b30139e6b716d0837dce30).
  A referência foi confirmada pela API GitHub; não é uma afirmação de que seja a
  versão oficial mais recente. A tag Windows 0.5.0 aponta para o mesmo commit.
- Árvore oficial separada, detached e sem alterações à árvore de desenvolvimento:
  `.build/coucou-official-841a81ce/`.
- RPM oficial descarregado e extraído dentro do projeto, sem instalação:
  SHA-256 `fe5f2b09bc667d92d032caaa5c21610c62f4011b14734c8a1d17c1670c6485de`,
  coincidente com o digest do asset publicado. Executável extraído:
  `a0c01c3cce3ef4288c880bc0d9bd2683531254283d2a4c21f738ca9d50e38928`.
- Fork de partida: `feat/mochi-cli-accounts`, HEAD
  `de75abcb3eb87bc4f4f59fbc329ac7dad1e2b53e`, inicialmente limpo.
  Privada 58: `c1e364ce9f5f5031b3edb67b09f44f3a597df64a09152b67466e060becd6515b`.
  As 1322 entradas do seu manifesto coincidiam com a árvore antes da correção.
- Primeira composição corrigida: privada 60, versão interna `0.5.1-mochi.44`, executável
  `701db947a9b664197e4d6d4ec38557ed91c65424554bc4e27f10749e6dcfefe5`.
  Manifesto: `.build/upstream-parity-20261111/SOURCE2.json`.
- Candidata histórica68, anterior à correção de rascunhos77: mesma versão interna, executável
  `9c30f180d845a3e82660c3be0fd0492ec80093d80bb161232ee577b5e07b8394`.
  Manifesto: `.build/upstream-parity-20261111/SOURCE10-FLOW-OFF.json`.
  A62 recuperou a ligação do centro ao diff; a63 recuperou acessos/feedback Flow;
  a66 conserva os erros do chat; a67 retira espaço reservado por memória fechada;
  a68 encaminha ditado desligado/não preparado para a configuração existente.
- Árvore atual Source38/privada87: um mapper n8n e dois testes,1323 inputs
 exatos, binário215ba7c8…, identidade dev. Perda na37 reproduzida após seleção;
 38 conserva workflow com HTTP/TLS/conversa sintéticos, retirada e reinício.
 PHONE-N8N-SOURCE38-VERIFIED.json. Preparar normal46; RPM45 permanece congelado.
- Candidata anterior Source37, normal45, preservada, não instalada nem autorizada:
  compilação nova1323 inputs, RPM e91116b9…/binário8c49d520…; o próprio pacote
  passou UI/TLS/7 HTTPs sintéticos/Stop/Busy/revogação/reinício/112 leituras.
  PHONE-NORMAL45-SOURCE37-VERIFIED.json. Backend igual36 e cinco metadados de
  versão; um ficheiro funcional mudou desde a instalada35. Demais provas
  mantêm fonte própria. [Proposta45](PETS-INSTALLED-DELIVERY-SOURCE37.md).
- Composição instalada atual Source35, normal44, instalada em13/11: nova compilação,1323
  inputs exatos. Voz: set-main/CAS/recibo/resposta/TTS/Stop/mesmo CID/reinício e
  recusa por permissão em perfis privados; `VOICE-ROSTER-SOURCE35-VERIFIED.json`.
  RPM SHA-256 `98149dc1007d0eb705698ff03430bbfc48499a4b06afc46839945de8861a5027`.
  Executável `87400570bf911c5c5fe60204c19ad5f7def464b753b531ad957f44d326bd9e9d`. Outras provas abaixo mantêm fonte/âmbito;
  não foram promovidas para aceitação integral35.
- Composição anterior Source33, normal44, não instalada: código compilado na
  Source32 e mesmo RPM, com alinhamento só documental do arquivo completo.
  A falha original do wrapper por esse documento foi conservada; não se alega
  nova compilação ou igualdade integral32. Qualificação explícita no recibo.
  RPM: `e63aff2c9692525402fc3e25358c014ee6e3b40bd0d770b291e57257d75e06ff`.
  Executável: `10492269eb366487d5d273124e865b5265b73f346fb72f1094c2c07138f24571`.
  `SOURCE33-CUSTOM-DOCUMENTATION.json`, `CUSTOM-SOURCE33-VERIFIED.json`:
  servidor compatível por pet/UI/HTTP/reinício/recusa sem chave/preservação CLI,
  mais regressão dos cartões de serviço no âmbito sintético. Agora a própria33
  tem revisão6/7/HTTP/recuperação/preservação (`RULE67-SOURCE33-VERIFIED.json`)
  e12 etapas de apresentação real dos serviços (`COMPOSITOR-SERVICES33-VERIFIED.json`).
  Restantes provas mantêm limites; [entrega33 preparada](PETS-INSTALLED-DELIVERY-SOURCE33.md),
  sem substituir instalada43 ou assumir autorização da proposta29.
- Composição normal anterior Source30, normal44, não instalada: seis cartões de serviço
  recuperados por reutilização dos widgets existentes, com n8n HTTP nativo,
  reinício e retirada de credenciais sintéticas verificados em privado.
  RPM: `f6857842513e6e3902a3771146dc4512e37eeb3f84c1fb68f852630eddfbf153`.
  Executável normal: `177fd967693a4e06c21e2d80f1046cda060140a3941498d03145512f684db897`.
  Manifesto1323: `SOURCE30-SERVICE-CARDS.json`.
  Recibo: `SERVICE-CARDS-SOURCE30-VERIFIED.json`; aceitação gráfica parcial,
  APIs remotas/hardware/instalada não certificados. As provas anteriores abaixo
  mantêm a sua composição exata, sem serem promovidas a Source30.
- Composição normal anterior Source29: privada85 e RPM normal44/Source29 preparados, não instalados.
  Preparação interativa aguarda contenção de leitura com Stop/prazo e destino
  original; feedback `busy` usa o protocolo existente. Fixtures de espera/HTTP
  e percursos nativos completos85/RPM29 passaram nos seus âmbitos.
  RPM: `53d738c52921c5629ba67491b3c792ba67208a11728e747674b5ac05678c6f9b`.
  Executável normal: `2446d17676c80e9710dbde615bdf5735cfc41abc148c4beec8be7bae4d0674d7`.
  Recibo: `INFERENCE-READER-SOURCE29-VERIFIED.json`.
  O mesmo executável normal29 tem agora prova gráfica própria no Hyprland real:
  cartão/diff Codex legível e conversa Ollama com cinco linhas de código, mesma
  conversa/resposta após reiniciar e um único HTTP sintético, sem replay.
  Recibo: `GRAPHICAL-CODE-DIFF-SOURCE29-VERIFIED.json`.
  A revisão/sucessão6/7 tem igualmente prova integrada atual e preservação CLI/
  janela restaurada, em perfis gerados: `RULE67-SOURCE29-VERIFIED.json`.
- Composição normal anterior Source25: privada84 e RPM normal44/Source25 preparados, sem instalação.
  Ambos passaram o percurso nativo privado do chat do telefone e a correção HTTP.
  RPM: `6111b52cce0205b5ccca68e764045ba6771fa41f0cf5329cffa1285558de91cb`.
  Executável normal: `7eb8df44aa37efb57e4a770a80e93f0b5444a67560e09011f715a03906531153`.
  Isto não certifica toda a aplicação, hardware ou experiência instalada.
- Composição normal anterior Source22, candidata privada81 e RPM normal44/Source22,
  preservada com as suas próprias provas:
  Executável da candidata81:
  `c02812fe975bec248f7683eb5f2cc5bca952eefec3122df487da360ab2b6df7f`.
  Executável normal extraído no prefixo privado:
  `4c40ea6a84e8d488aa7e492a2d27c2e85caca24fb5bb1d43f67e7e17d2648e93`.
  O chat flutuante é criado no primeiro uso; rascunhos, hooks, documentos/código
  e voz têm provas próprias abaixo. Os RPMs Source19/21 e as suas provas
  continuam preservados com o âmbito original, sem serem reclassificados como22.
- Instalada atual: `0.5.1-mochi.44`, Source35, executável87400570…;
  `installed-source35-delivery/INSTALLED44-TECHNICAL.json`.
- Instalação anterior43 preservada, executável
  `878408b2d07d1f19371876355b75110e52e65395cdb3463c6b7ee1e5e0e01446`.
  Não foi substituída. Executar o seu binário com perfil gerado não equivale a
  aceitar a experiência do perfil operacional; esta última continua pendente.

Os ensaios usam o executável nativo/Tauri/WebKit, perfis XDG e D-Bus privados e
eventos sintéticos recebidos pelo socket nativo. Não copiam perfis autenticados,
não consomem inferência paga nem executam os comandos apresentados pelas
fixtures. As unidades explicitamente identificadas de chat e voz também usaram
o Gemma3:4b instalado no Ollama local real, com pedidos limitados e sem fallback.
Os ensaios não constituem aceitação humana. Capturas e recibos ficam privados
em `.build/upstream-parity-20261111/`.

<a id="diff-editor-2111"></a>

## 21/11 — abrir o ficheiro a partir do diff

Nos RPM 61 e 62, o relay original publicou uma edição Codex. O cartão do
pet mostrou as linhas removida e adicionada. A ativação do botão
«Open in VS Code» no WebKit chamou o editor sintético uma só vez,
com o caminho absoluto de um ficheiro com espaços e Unicode. A
navegação voltou à mesma vista e o ficheiro conservou os bytes.

O oficial 0.5.0 e o RPM 61 usam o mesmo código Rust para validar e
encaminhar o ficheiro. Os ensaios no compositor isolado com o oficial
chegaram ao evento de edição, mas ficaram sem reprodução completa da
abertura do leitor. O resultado funcional observado pertence aos
RPM 61 e 62. O ensaio usou WebKit privado e um capturador de argumentos.
[Comparação no 61](../.build/wayland-layer-qa-20261120/DIFF-EDITOR-61-VERIFIED.json) ·
[Percurso no 62](../.build/wayland-layer-qa-20261120/diff-editor-fork62-a/RESULT.json).

A comparação no X11 privado confirmou depois o percurso completo no oficial
0.5.0 e no RPM .66. Cada executável encaminhou uma vez o ficheiro exato para
o capturador do editor e voltou ao mesmo diff. [Recibos](../.build/upstream-diff-editor-20261121/VERIFIED.json).

## Resultado observado

| Componente | Classificação e evidência | Original, fork e ação |
| --- | --- | --- |
| Receção de atividade e alterações por hooks | **Preservado no contrato nativo ensaiado, atualizado na77** | Oficial0.5.0 e77 executaram o relay original real: Codex, Copilot e Gravity conservaram atividade e Edit canónico nas suas formas de transporte, sem mistura entre pets com o mesmo ID de sessão. Teclas nativas, Stop e abertura pelo centro passaram na77. Reutiliza o armazenamento existente, sem segundo controlador ou ledger. Não certifica todos os formatos de ferramentas de IDEs ou providers reais. |
| Abertura do diff no cartão Linux | **Perdido na instalada43/privada58; recuperado desde60 e observado graficamente no normal29 atual** | No oficial, a edição recebida pelo socket abre o diff.43/58 substituíam o overview sem expor o leitor/comando `toggleDiff`. A correção reutiliza `Ticker`/`buildDiffCard`, botão acessível e Ctrl+E/Escape. O RPM normal29 recebeu o Edit pelo relay original, mostrou +1/−1, abriu pelo botão e apresentou `layer_before` removido/vermelho e `layer_after` adicionado/verde, sem sobreposição no cartão. O retorno nativo foi confirmado pelo DOM. As capturas antigas cortadas/sobrepostas conservam esse limite; a prova29 atual não é aceitação da instalada ou de todas as rotas/providers. |
| Atividade imediata e concisão do cartão | **Degradado; corrigido na privada 60** | O original privilegia o trabalho atual. O cartão do fork mostrava extensos painéis de sessões, conta, quotas e proveniência, mantendo só uma linha de atividade. A correção coloca novamente a atividade/diff junto da identidade e recolhe os diagnósticos por omissão; os controlos existentes continuam acessíveis. As consultas de diagnóstico do componente só são solicitadas enquanto a secção está aberta ou por atualização explícita. Não é uma medição de melhoria global de CPU. |
| Isolamento de alterações entre sessões | **Preservado e verificado na correção** | Na 60, duas sessões do mesmo pet mostram textos distintos; trocar de sessão fecha o diff anterior, regressar permite abrir o diff correto. Escape fecha primeiro o diff e conserva o cartão. O armazenamento e os limites existentes permanecem. |
| Diff a partir do centro partilhado | **Degradado; recuperado desde a privada62** | A60 mostrava a atividade no centro, mas não abria a alteração. A correção encaminha pet, backend, sessão e ID da alteração para o leitor canónico existente. A seleção precede a abertura; decisões pendentes conservam prioridade. O primeiro ID, zero, é válido. Não se duplica o armazenamento de diffs. |
| Abrir o ficheiro pelo leitor de diffs | **Preservado no ensaio nativo comparado.** | O oficial 0.5.0 e o RPM .66 receberam o mesmo Edit por relay, abriram o diff por Ctrl+E e passaram uma vez ao capturador do editor o caminho exato, com espaços e Unicode. Voltar e reabrir conservaram as linhas do mesmo diff e o ficheiro ficou intacto. O ensaio comparado usou X11 privado. [Recibos](../.build/upstream-diff-editor-20261121/VERIFIED.json). |
| Apresentação Linux e fábrica de pets | **Alterado intencionalmente** | O original usa a ilha e as pills; o fork usa pets/cartões e fábrica comum. A comparação não justifica reverter essa decisão nem restaurar pets sem utilidade para o proprietário. |
| Catálogo de integrações | **Presença preservada no código; funcionamento integral ainda não verificado** | Os25 identificadores do catálogo oficial continuam no fork, que acrescenta o Guardian. Isto não significa25 pets ativos nem obriga a repor pets sem uso. A retirada funcional temporária de GitHub continua intencional. O recibo privado `CATALOG-REFERENCE-COMPARISON.json` verifica presença/IDs, sem certificar ligações, credenciais, efeitos ou interfaces. |
| Cartões especializados na vista de cada pet | **Perdidos no normal29; recuperados funcionalmente30/33 e observados no compositor33** | Oficial0.5.0: n8n Refresh→GET autenticado→resultado/detalhe/retorno/erro.29: dados chegavam mas cartão/Refresh ausentes.30 reutiliza seis widgets originais no resumo comum. Agora33 passou12 etapas no compositor real, n8n com22 GET locais autenticados, Refresh/sucesso/erro/detalhe/Back, reinício sem resultado repetido e retirada de chave sem mais leituras. Notion/Vercel/Resend/Cal.com/Stripe passaram apresentação de eventos nativos sintéticos; detalhe Vercel/Back também. Nenhuma mutação ou inferência. Área privada lateral após mudança observada para ultrawide; galeria operacional intacta. Falhas gráficas anteriores conservadas. Texto secundário Resend pouco contrastado; sem aceitação geral de qualidade, contas remotas ou instalada. `COMPOSITOR-SERVICES33-VERIFIED.json`; comparação original e correção em `SERVICE-CARDS-SOURCE30-VERIFIED.json`. |
| Pet GitHub | **Alterado intencionalmente** | A retirada temporária de `integration_github`, com preservação dos seus dados, é decisão do proprietário. Não classificar essa retirada como regressão. Gravity, Copilot, Codex e Ollama continuam no âmbito. |
| Hub: Flow, notas, dicionário, snippets, estilos, definições, agentes, chat, voz, integrações, telefone e atalhos | **Preservado no âmbito de navegação ensaiado** | Doze destinos abriram com conteúdo no oficial, no executável exato da instalada43 com perfil privado e na candidata60. Isto verifica abertura/navegação; não certifica gravação, áudio, serviços ou todos os controlos de cada página. |
| Aprovações e perguntas nativas | **Preservadas no transporte ensaiado; cartões atuais observados no compositor real** | Oficial0.5.0 e candidata68 conservaram recusa Codex e pergunta Claude no socket. O RPM normal Source29 mostrou agora comando/botões completos e pergunta/opções legíveis no Hyprland; cliques nativos devolveram `deny` e exatamente `QA_B` ao socket correspondente. Reinício sem pedidos/inferência repetidos, zero comandos executados. Perfis e escolhas automatizadas inteiramente sintéticos, não aceitação humana ou de providers/sessões operacionais. Cortes antigos X11 e atribuição diferencial inconclusiva conservam o âmbito histórico; não se repôs qualquer workaround no produto. |
| Código Markdown na conversa do pet | **Preservado no âmbito nativo ensaiado; observado graficamente no normal29 atual** | O renderer original mantém blocos/cópia. O normal29 mostrou cinco linhas TypeScript legíveis numa conversa flutuante nativa Hyprland do Ollama, com modelo/destino sintéticos explícitos; o mesmo CID e mensagens voltaram após reinício, com um único POST e sem replay/execução de código. O controlo de cópia estava visível, sem tocar no clipboard pessoal. A cópia efetiva permanece validada no X11 privado anterior, com renderer/bridge de hashes iguais; não se certifica por isso o clipboard Wayland pessoal ou todos os providers. |
| Diffs Write/MultiEdit por pet e sessão | **Preservados nos seis casos nativos atuais48; retenção alterada intencionalmente** | RPM48 exato: relay oficial→socket→cartão/centro para Codex/Copilot/Gravity, Write/MultiEdit. Ficheiro, linhas e contagens coincidem com as seis observações originais; Back/Ctrl+E/Escape e isolamento passaram. Seis imagens X11 privadas legíveis. Um MultiEdit permaneceu aberto23s com Ctrl+P existente e ponteiro privado no cartão; não prova pin isoladamente. O ensaio original completo falhou por limpeza5,2s após Stop; preserva-se esse bruto. A falha histórica35 de Back no Hyprland continua sem causa exata estabelecida;48 não iniciou nessa sessão por ausência de região livre, sem mover superfícies operacionais. Não certifica IDEs reais, todos os payloads, reinício ou Back no compositor operacional. [Provas/limites48](../.build/diff-return48-20261116/VERIFIED.json);35 preservada no seu âmbito. |
| Espaço útil para conversar/ler código | **Degradado; corrigido na privada67** | O chat original não reserva espaço para os novos controlos de memória do fork. Na66, mesmo fechados, reservavam142,1px numa janela760×620, reduzindo a conversa a20px. A67 conserva esses controlos e o limite de260px, mas usa a altura do conteúdo:32px fechados, conversa130,1px, bloco de cinco linhas75px inteiramente visível ao deslocar. Abrir/fechar a memória conserva16 controlos acessíveis e overflow. Medição deste perfil/viewport, sem extrapolação para todos os ecrãs. |
| Erros de resposta nas superfícies de chat | **Degradado no fork; recuperado na privada66/67** | O percurso original usa a vista de nota para mostrar erros. A superfície flutuante do fork não usa essa vista e apagava o erro ao atualizar o histórico. A correção mantém feedback no cartão e reutiliza `SessionChat.failure.message` no chat flutuante, associado à conversa correta. A67 recusa uma ronda incompleta, mostra o erro após refresh, conserva o pedido para revisão, não inventa mensagens e não repete o HTTP. A comparação da superfície original neste caso é de código; a falha e a correção do fork foram observadas nativamente. |
| Rascunhos e espaço do compositor | **Degradado no fork; correção funcional validada na privada77** | O original mantém o compositor simples. No fork/Source10, estado vazio e dois controlos de recuperação aparecem sempre e disputam espaço com o modelo e o texto. A correção partilhada recolhe recuperação/reconciliação num resumo de14px, conserva erros/conflitos/rascunho recuperado visíveis e fecha a secção ao mudar de conversa. O cartão de chat tem mínimo320px limitado ao viewport e deslocação vertical. A77 passou teclado real, guardar, nova conversa, revisão/cópia efetiva, origem idêntica, erro de limite, conversa flutuante e reinício, sem inferência. Aceitação gráfica global e entrega instalada pendentes. |
| Anexar ficheiro por arrastar e conversar sobre o conteúdo | **Preservado no âmbito nativo comparado** | Oficial0.5.0 e candidata68 receberam um ficheiro sintético por negociação GTK real, com origem/destino observados. O original mostrou a sequência de entrega e recebeu a ação pelo botão desenhado no canvas; a68 recebeu-o na conversa do pet. Nos dois, os bytes chegaram ao Ollama/modelo sintéticos exatos, a resposta mostrou código e a cópia para o clipboard privado coincidiu com o bloco. Sem eventos/tokens de drop forjados. X11 privado; PDF/pastas/Wayland e todos os formatos/providers não são certificados por este caso de texto. |
| Anexos por pet/conversa, cópia gerida e leitura após reinício | **Alterado intencionalmente; validado nativamente na68 neste âmbito** | O fork mantém a cópia independente do ficheiro de origem, liga-a ao pet/conversa e exige nova revisão de leitura após reiniciar. O ensaio alterou a origem depois do drop: o modelo recebeu a cópia anterior. Reiniciar preservou conversa/cópia, sem restaurar implicitamente o grant. A UI da conversa reviu a cópia exata e renovou apenas a leitura própria; o outro pet continuou sem contexto. É uma decisão automatizada sobre dados de QA, não consentimento humano ou autorização de dados pessoais. O original tem uma caixa global e varrimento por idade observados no código; não é motivo para recuperar esse modelo de autoridade. |
| Servidor personalizado OpenAI-compatible | **Perdido no percurso dos pets30; recuperado funcionalmente no normal33** | Oficial0.5.0 passou ligação/chave/modelo/resposta SSE. No modo de pets30, a ligação ficava guardada mas o motor não podia ser escolhido na conversa.33 reutiliza transporte e binding/editor comuns: UI de revisão por pet, modelo/endereço/chave exatos, resposta/código e mesmo CID após reinício; retirar chave recusa novo envio sem HTTP anónimo ou outro motor. Backend/conta nativos e permissões mantidos, sem novos pets/grants. Treze recibos CLI preservam bytes e escolha;30 permite revisão opaca de backup mas recusa ativação. Conversa/erro flutuantes legíveis no X11 privado; remoto/TLS/voz/ferramentas/autonomia/instalada não certificados para este motor. `CUSTOM-SOURCE33-VERIFIED.json`, com qualificação de compilação32/alinhamento documental33 e falhas conservadas. |
| Contexto do residente por domínio no centro | **Degradado no contrato do fork48; corrigido e validado no próprio RPM49** | Oficial0.5.0 oferece o widget n8n (prova30), sem este centro.48 encaminhava o botão direito n8n para Sessões/contagens CLI.49 consome contextAction/metrics da fábrica: clique a partir do Codex abre o widget n8n, sem contagem CLI/chat; inverso conserva Sessões Codex. Duas imagens atuais observadas em X11 privado, zero inferências. Widget sem chave neste caso; backend/HTTP anterior48 mantêm seu âmbito. Sem entrega instalada ou menu flutuante certificado. [Prova49](PETS-INSTALLED-DELIVERY-49.md). |
| Chat e modelos em pets de aplicações/serviços | **Retirada intencional por domínio40/42; validada no RPM48** | Os widgets de ferramenta do oficial são o valor a preservar. O fork até39 associava assistentes Codex genéricos aos serviços. A regra do proprietário retira conversa/modelos/métricas/sessões de inferência desses pets, sem apagar preferências ou história. A política pertence aos moldes llm/application, nunca à base, conta ou preferência individual. O RPM48 mostrou n8n com workflow e sem controlos LLM, recusou chat/modelos/envio e conservou dados após reinício; conversa sintética só no Codex. Outros serviços conservam contratos/fixtures; não há nova aceitação nativa integral de todos eles. Não é regressão a recuperar nem entrega instalada. [Âmbito48](PETS-INSTALLED-DELIVERY-48.md). |
| Chat integral e outros motores de inferência | **Ainda não verificado quanto à paridade integral** | As provas de código/anexos usam uma inferência sintética por execução, sem autenticação ou quota real. Os percursos equivalentes dos outros providers e das suas ferramentas continuam abertos. Preservar destinos/modelos escolhidos e ausência de fallback. |
| Spotify/MPRIS: reprodução e controlos | **Preservado no âmbito nativo comparado** | RPM oficial0.5.0 e RPM normal44/Source10 passaram abrir sem efeito, reprodução/pausa, anterior/seguinte, shuffle, repetição, volume, posição ligada à faixa exata, sinal externo de metadados, reinício sem repetição de comandos e desaparecimento do leitor. Foram clicados controlos reais; o serviço MPRIS sintético registou as chamadas D-Bus efetivas. Capturas de ambos inspecionadas e legíveis. No fork, uma recusa sintética de Next foi mostrada sem retry automático. Não certifica Spotify real, dispositivos, pesquisa externa ou qualidade sonora. |
| Resumo semanal e partilha | **Preservados nos percursos nativos ensaiados; apresentação atual observada no compositor real** | Oficial0.5.0 e Source10 exportaram PNGs iguais no mesmo X11 privado. O RPM normal Source29 mostrou resumo e partilha completos no Hyprland, exportou PNGs1080×1920 com/sem projeto, registou atividade sintética pelo socket e conservou ocultação/histórico/imagem após reinício. As exportações atuais Wayland e anteriores X11 diferem nos pixels/fontes, com mesmos dados/layout/privacidade observados; não se alega igualdade entre backends. Semana anterior semeada explicitamente; não prova disparo automático de segunda-feira, clipboard de imagem ou uso instalado. |
| Guarda-roupa: escolha e persistência existentes | **Preservado no âmbito nativo comparado** | Escolher Crown pela UI mostra a coroa e guarda a preferência, mantida após reinício nos dois RPMs. O oficial grava settings.json; o fork grava o payload canónico da fábrica e preserva o ficheiro legado. Não foi necessário alterar o produto. Captura atual legível. Não prova escolhas individuais de todos os pets ou todas as roupas/animações. |
| Atalhos globais X11: editor, mute, guarda-roupa e chat | **Preservado no âmbito nativo comparado; revisão alterada intencionalmente** | RPM oficial0.5.0 e RPM atual44/Source10 registaram três atalhos num X11 privado. O editor recebeu teclas reais e mudou mute para Ctrl+Alt+Shift+S; a tecla anterior deixou de executar, a nova alternou duas vezes e os outros atalhos mantiveram-se após essa alteração de preferências. Ctrl+Alt+G abriu o guarda-roupa e Ctrl+Alt+Space abriu o chat do pet Ollama, com foco e escrita real de qakeytext, sem envio. A configuração persistiu após reinício. Perfis novos válidos registaram inicialmente em ambos; após edição/revisão nativa, o fork exige nova revisão no reinício e não regista implicitamente. A revisão foi clicada apenas num perfil sintético, sem representar aprovação humana. Não prova todos os atalhos, layouts, conflitos entre apps, portal Wayland ou sessão instalada. |
| Atalhos next/previous e retorno a permissões/perguntas | **Preservado no âmbito nativo comparado** | Oficial0.5.0/841a81ce e privada71e4889d: mesmo ciclo completo e inverso, com workspace Claude legítimo, e retorno ao pedido original ainda sem resposta. XTest/socket nativos privados; sem aceitação Wayland/instalada. [Provas](#atalhos-pedidos-cli-1711). |
| Retorno por atalho aos pedidos CLI da arquitetura atual | **Degradado no RPM50; corrigido, integrado e validado no próprio RPM51** | O original abria permissões/perguntas legadas. O fork50 recebia goToAlert mas ignorava o destino CLI ativo. A51 reutiliza requestDestinations/presentRequest, preserva dono/CID e prioridade legada; Stop retira o pedido. O oficial não tem este transporte CLI: é lacuna de integração do fork. Falta compositor real e entrega operacional autorizada. [Provas51](PETS-INSTALLED-DELIVERY-51.md). |
| Abrir/fechar a ilha no modo atual de pets | **Degradado no RPM51; corrigido e validado na privada7d274c3c** | Oficial alterna abrir/fechar;51 abre sempre. A correção usa contexto nativo e collapse existente; chamadas sem toggle conservam abertura explícita. Mesmo protocolo de teclas e estado antes de agir. Falta pacote sucessor e compositor real. [Provas](#ilha-terminal-1711). |
| Centro dos pets durante a configuração da barra | **Degradado no 52 em Wayland. Corrigido e verificado em binário privado posterior.** | O centro chegava a expandir e a atualização em segundo plano podia ocultá-lo. O backend preserva agora a navegação existente enquanto reconcilia a barra compacta. Confirmado no Hyprland com primeira abertura, atualização em segundo plano e fecho explícito. [Provas](PETS-INSTALLED-DELIVERY-52.md). |
| Mostrar/ocultar os pets no desktop | **Degradado no fork anterior; adaptado intencionalmente e recuperado na privada12230fac** | Oficial faz sair/voltar o singleton; o fork anterior recebia a tecla sem efeito. Agora alterna a apresentação dos pets configurados, sem mudar pausa/roster/modelos. Atalho, controlo Geral, janela real, reinício e pedido pendente intacto passaram em X11 privado. Falta pacote sucessor, compositor real e entrega. [Provas e limites](#desktop-pets-1711). |
| Abrir janela/pasta da sessão | **Limitação corrigida e verificada no RPM 62.** | Oficial e RPM 60 abriram só a pasta no Hyprland isolado. A .62 focou o WezTerm certo, incluindo pelo botão do chat. A .61 teve perdas de efeito em X11. A .62 passou duas séries de cinco sessões no X11 privado e abriu a pasta quando a ligação perdeu a origem. [Provas e âmbito](PETS-VALIDATION.md#rpm62-nativo-2111). |
| Multimédia adicional e outros atalhos | **Ainda não verificado de ponta a ponta nesta comparação** | Permanecem efeitos fora dos percursos documentados, incluindo dispositivos/volume do anfitrião, conflitos/layouts, portal Wayland e outras ações globais. |
| Acessos de ditado/notetaker e feedback Flow | **Degradado; recuperado no âmbito nativo ensaiado** | O original oferece ditar, gravar chamada e abrir Coucou na barra Flow. O fork oculta essa barra no modo de pets, só oferece ditado no Ollama e ancora todo o feedback nesse pet. Na62, um perfil só com Codex não tem esses acessos/feedback. A correção oferece as três ações existentes em qualquer menu de pet e acesso à página de ditado pelo centro; o feedback global dispensa Ollama, incluindo catálogo vazio sintético. Gravação ativa apresenta Terminar. A63 abria o Hub na última página; a65/66 corrigiu o destino explicitamente para ditado. Os ensaios de feedback usam eventos sintéticos; não captam áudio. |
| Workers de reconhecimento e síntese escolhidos | **Validados com áudio/texto sintéticos; paridade de qualidade ainda não verificada** | A configuração existente resolve `auto` para Whisper e escolhe Edge/Raquel. O worker atual transcreveu7,10s de áudio sintético local em CUDA; o Edge produziu MP3 válido de7,34s. Sem download de modelo, reprodução ou mudança de preferências. Houve uma substituição de palavra (`voz`→`vós`), sem amostra original comparável: não se atribui regressão nem se certifica precisão. São workers reais; não é um teste integrado da sessão nativa de voz. |
| Entrada de voz no dashboard LLM atual | **Acesso direto ausente no49; corrigido e validado nativamente após49, sem instalação** | O oficial já oferecia voz global e pelo pet; esses mecanismos estavam preservados. O novo dashboard não tinha entrada direta sem os pets flutuantes. O botão reutiliza a rota explícita da fábrica e o backend existente. Ollama selecionado/global Codex, ambos os modos de apresentação desativados: clique real→áudio virtual→resposta/TTS→Stop pelo Hub→mesmo CID após reinício, sem replay. n8n permanece sem chat/voz genéricos. Outros LLMs têm prova dirigida de routing, sem chamadas reais. RPM49 e instalada44 não contêm esta alteração. [Âmbito e falhas retidas](../.build/voice-card-entry-20261116/VERIFIED.json). |
| Assunto Resend e datas dos widgets | **Limitação herdada corrigida. Assunto e data de Resend verificados no RPM 53.** | Oficial e fork usavam tons escuros. A correção usa o tom secundário existente. A observação no Hyprland confirmou uma data e um assunto sintéticos completos em Resend. A comparação anterior de cores planas permanece separada. Restantes widgets, contas remotas, outros ecrãs e aceitação humana conservam o seu âmbito anterior. [Prova delimitada](../.build/operational53-20261118/RESEND-VISUAL-VERIFIED.json). |
| Sessão de voz integrada, áudio virtual | **Validada na68 e no próprio RPM49, com âmbitos distintos** |49: API nativa explícita Ollama→microfone virtual→ASR configurado→modelo/broker sintéticos (mudar pet principal)→resposta/Edge→saída privada gravada. Fim natural, retorno visível à escuta, palavras da resposta por ASR local independente e CID/mensagens após reinício, sem replay. Só mainPill mudou no perfil QA; outras escolhas intactas. Primeira tentativa consultava a janela errada e permanece falhada; repetição no webview proprietário passou, sem mudar produto. Pico PCM1,06214 não certifica qualidade física/ausência de clipping. Entrada gráfica da68 mantém seu âmbito anterior;49 não certifica entrada gráfica, modelo real, hardware ou instalada. [Prova49](PETS-INSTALLED-DELIVERY-49.md). |
| Notetaker: dois canais, transcrição, intervenientes, exportação e preservação | **Preservado no âmbito nativo comparado** | Oficial0.5.0 e candidata68 passaram gravação/Stop pela UI, microfone e som do computador exclusivamente virtuais, dois intervenientes, Markdown igual à nota guardado em Documentos privados, remoção dos WAV após transcrever e nota idêntica após reinício. Foram usados os mesmos áudios sintéticos naturais e os seis ficheiros originais de Parakeet/diarização, já instalados e verificados contra os hashes pinned. Sem alterar os modelos da estação. As transcrições diferem numa palavra final; não se certifica equivalência de precisão, idioma ou desempenho a partir destes dois casos. O resumo por Claude permaneceu desligado e sem aceitação própria. |
| Ditado, colagem X11, dicionário, snippets e estilos | **Preservado no âmbito nativo comparado** | Oficial0.5.0 e candidata68 passaram manter/libertar RightCtrl, captura virtual, Parakeet real, transformações, inserção confirmada num campo de destino e reposição do clipboard anterior. Palavra personalizada, substituição e snippet multilinha preservado foram aplicados; configuração de estilos e histórico sobreviveram ao reinício. X11/clipboard/teclas inteiramente privados; nenhum dado, áudio ou aplicação pessoal. A transformação paga por Claude permaneceu desligada: guardar estilos não prova esse efeito. Não certifica colagem Wayland/Hyprland ou hardware. |
| Histórico e colagem do ditado em Wayland | **Preservados no âmbito nativo comparado.** | Oficial e RPM 57 copiaram as entradas selecionadas. Oficial e RPM 59 inseriram o texto num editor GTK em Sway privado, repuseram o clipboard e retomaram o percurso após reinício com portal sintético. Perante recusa, ambos deixaram o ditado no clipboard. A 59 mostrou a alternativa manual. Falta consentimento e inserção através de um portal real. [Cópia](#flow-copia-wayland-2011) · [Colagem](#flow-portal-wayland-2111). |
| Exportação Markdown de uma nota com título longo | **Limitação herdada corrigida e verificada no RPM 58.** | No oficial e no fork anterior, um título longo fazia a escrita falhar sem aviso. O 58 preserva o título e a transcrição, escolhe um nome de ficheiro livre e mostra falhas de escrita. Passaram pela janela nativa dois Markdown e dois PNG de controlo. [Provas](PETS-VALIDATION.md#rpm58-markdown-2011). |
| Resumo de chamadas e escolha do modelo | **Limitação herdada, corrigida na fonte atual.** | O oficial e o RPM 56 repetem a transcrição noutro modelo quando o selecionado falha. A correção usa apenas a configuração atual, mostra a falha e preserva a transcrição e o resumo anterior. Passaram os percursos manual e automático com gateway sintético. [Provas](PETS-VALIDATION.md#notetaker-modelo-exato-2011). |
| Notetaker com voz só no microfone | **Limitação herdada, corrigida na fonte atual.** | O oficial e o RPM 56 concluem a nota a partir do sinal disponível, mas omitem a cobertura. A correção indica a faixa do computador vazia ou silenciosa no aviso, na nota e no Markdown. Passaram áudio virtual, dois canais de controlo e leitura compatível de uma nota anterior. [Provas](PETS-VALIDATION.md#notetaker-so-microfone-2011). |
| Microfone escolhido indisponível | **Alterado intencionalmente na seleção. Recuperação melhorada na fonte atual.** | O oficial 0.5.0 abre a fonte predefinida. O RPM 56 mantém a escolha e recusa a troca. O aviso corrigido abre o seletor com a opção ausente identificada. O ditado volta a funcionar após escolha explícita, com colagem e reinício verificados em X11 privado. [Prova](PETS-VALIDATION.md#ditado-microfone-indisponivel-2011). |
| Escuta contínua “OK Coucou” | **Degradado no fork; corrigido e validado nativamente em Source21/22, incluindo modelo local real na22** | No RPM normal Source19, a opção persistiu ligada e o log anunciou escuta, mas não houve captura. Na79, o diagnóstico revelou uma falsa indicação de gravação de preferências: pré-carga e escuta capturavam a origem em paralelo. A80 captura uma única origem e reutiliza-a na pré-carga, compara as opções após obter o mutex e só anuncia escuta quando a captura abriu. Passou áudio negativo sem chamada, chamada real transcrita por Whisper, uma inferência HTTP sintética, resposta no pet, sinal Edge virtual, desligar e reinício sem replay. Três ciclos adicionais com72 leituras nativas também passaram, sem inferência. O oficial mantém o gate Parakeet+chave Claude: preferência/gate/reinício ensaiados, percurso pago integral não ensaiado. Na Source22, a UI/Whisper, uma resposta do Gemma3:4b real, resposta do pet, Edge registado na saída virtual e reinício sem replay passaram; pico0,9634. Duas gerações reais gratuitas nesta unidade, incluindo uma tentativa anterior que o QA interrompeu antes da síntese. Hardware/qualidade/instalação não certificados. |
| Ligação Android: UI, sessões, diffs, recusa e privacidade | **Preservado no âmbito nativo comparado com telefone sintético** | Oficial0.5.0 e RPM normal Source22 passaram ativação/QR, TLS com pin do certificado gerado pelo backend, token errado recusado, hooks/sessão e diff exato incluindo ID0, recusa devolvida ao socket original, retirada de diffs com desconexão e nova negociação, rotação com token antigo recusado, desligar sem listener e reinício da app conservando identidade/token/opção. Cofre Secret Service real, HOME/D-Bus/X11 privados e rede só de loopback. Sem inferência, aprovação humana ou alteração de produção. Não certifica Android físico, relay, chat/modelos, serviços, quotas ou cópia do link. Perguntas têm prova própria37 na linha seguinte. Recibo `PHONE-NATIVE-VERIFIED-COMPARISON.json`. |
| Saldos Stripe com várias moedas | **Limitação herdada corrigida e validada em desenvolvimento.** | A função oficial juntava disponíveis e pendentes e rotulava o total com a primeira moeda. O cartão corrigido separa moeda e disponibilidade, preserva precisão e lida com erros e saldo vazio. A projeção móvel usa o saldo disponível. [Provas e âmbito](#stripe-saldos-2111). |
| Estado da leitura dos cartões Vercel, Resend, Notion e Cal.com | **Falha de ligação corrigida e verificada no RPM 59.** | Oficial e RPM 58 não publicaram erro perante a recusa de ligação. O 59 mostrou a falha nos quatro cartões. A resposta inválida, a lista vazia e a recuperação passaram na variante de desenvolvimento com HTTP privado. [Prova do pacote](PETS-VALIDATION.md#rpm59-servicos-2111). |
| Próximas reservas Cal.com | **Limitação herdada, corrigida em desenvolvimento.** | A primeira página passou a ser pedida por ordem de início. As datas inválidas, os campos de início em falta e um estado de erro na resposta têm aviso próprio. A agenda vazia e a recuperação passaram na janela nativa e o cartão voltou ao telefone sintético por TLS. [Comparação e limites](#calcom-agenda-2111). |
| Títulos compostos do Notion | **Limitação herdada, corrigida em desenvolvimento.** | O parser do oficial e o do fork apresentavam só o primeiro segmento. A janela corrigida distingue página e base de dados com nomes semelhantes, conserva os destinos de abertura e envia os títulos completos ao telefone sintético. [Comparação e limites](#notion-titulos-2111). |
| Troca de instância n8n com IDs iguais | **Limitação herdada corrigida e verificada no RPM 59.** | Oficial e RPM 58 mantinham o cartão da ligação anterior quando a nova devolvia o mesmo ID. O 59 distinguiu as ligações e descartou uma resposta atrasada de A depois de selecionar B. [Prova do pacote](PETS-VALIDATION.md#rpm59-servicos-2111). |
| Cartão móvel n8n e conversas históricas | **Regressão histórica37 corrigida38; o domínio atual não admite novas conversas LLM no n8n** | Fixtures mostraram estado/texto do assistente como workflow ou cartão sem evento. No RPM45, conversa flutuante preservou o cartão; selecionar a sessão realmente criada retirou o cartão da nova ligação TLS apesar de pong e workflow no desktop.38 usa a atividade nativa existente, com o mesmo lifetime/guard de pet.37 testes, build87 e percurso nativo completo: workflow/erro, conversa sintética/seleção, dados mínimos nos serviços, retirada, reinício/evento104 sem repetir turno. Oficial confirmou base/erro/retirada; falhas de reinício mantidas sem causa exata. Captura tardia não certifica cartão visível; sem exposição pessoal, quota real, Android físico ou instalada. PHONE-N8N-SOURCE38-VERIFIED.json mantém o âmbito histórico. A política40+ conserva os dados antigos e o widget de workflows, recusando nova inferência no serviço; não se pretende recuperar chat LLM no n8n. |
| Perguntas respondidas pelo telefone | **Preservadas no percurso nativo comparado oficial/normal45; rota Coucou explícita** | Dois AskUserQuestion sintéticos por composição atravessaram socket nativo/renderer/TLS com pin, com os mesmos textos/opções e respostas Unicode simples/múltiplas ao socket original. Dez fronteiras por composição: sem capacidade não recebe/responde; rótulo inválido, fingerprint antiga e duplicação não resolvem; desligar desconecta/recusa, reativar conserva a pergunta e reiniciar não repete concluídas. Mesma fixture, duas falhas do observador oficial conservadas e qualificadas; zero modelo/IDE real. A rota Decidir no Coucou foi escolhida/verificada apenas no perfil privado; preservar a escolha operacional é intencional. Não comprova Android físico/biometria/humano ou instalada. Nenhuma alteração de produto necessária. `PHONE-QUESTIONS-SOURCE37-VERIFIED-COMPARISON.json`. |
| Validação de inferência sob contenção de leitura | **Degradada na verificação síncrona; corrigida e validada com fixtures; integração nativa saudável85/RPM29 passada** | Antes, observação `Busy` produzia erro de destino alterado. A regressão determinística26 falhou; Source29 distingue Busy/Changed e aguarda leitura antes do chat interativo simples, com prazo monotónico/Stop e captura/reserva originais. HTTP da fixture ficou em zero durante a espera e foi exatamente um após libertar; Stop e troca de destino enviaram zero, sem redirecionamento ou reativação da captura retirada. A fence continua de uso único. O telefone usa `busy` sanitizado em vez de falha de rede. Isto não identifica nem encerra o episódio nativo intermitente anterior. |
| Chat do telefone: escolha, conversa, cancelamento e reinício | **Preservado no percurso nativo50; causa das recusas antigas não demonstrada** | Oficial0.5.0 completou sete pedidos sintéticos, continuação/reset, cancelamento, retirada de permissão/modelo e reinício sem replay. A privada84/Source25 completou o mesmo percurso com isolamento da conversa desktop consultado nativamente; o original não oferece essa API de leitura. Normal Source22 e diagnóstico82 tiveram uma recusa intermitente antes do segundo HTTP, sem alterações nas preferências. Outros ensaios82/83/84 passaram; isso não encerra a falha nem confirma a sua causa diferencial. O RPM próprio Source25 passou agora este percurso; Android físico/relay/instalada e a recusa intermitente continuam pendentes.  O próprio RPM50 passou probe e ciclo completo,14 POSTs sintéticos/752 leituras readonly, sem reproduzir a recusa. A causa antiga não fica encerrada por esse sucesso; [limites50](ANDROID_LINK.md#phone-chat-normal50). |
| Recusa da admissão nativa apresentada como falha de rede | **Degradada na35; corrigida36/37 e validada no próprio RPM normal45; não instalada** | Original0.5.0 não tinha esse guard do fork.35 transformava duas recusas locais anteriores ao HTTP em `unreachable`.36 usa o `busy` existente e diagnóstico `configurationBusy`, conservando recusa/autoridade/destino. Negativa com captura de produção e11 testes dirigidos; o próprio normal45 passou UI/TLS/sete HTTPs sintéticos/112 leituras/reinício. Backend igual36 e cinco metadados de versão;13 payloads/relay verificados. Não força contenção inicial no ensaio nativo nem prova a causa histórica22/82. `PHONE-NORMAL45-SOURCE37-VERIFIED.json`. |
| Erros HTTP do chat local no telefone | **Degradado também no oficial; corrigido e validado nativamente na privada84/Source25** | O oficial e normal Source22 apresentaram `unreachable` depois de uma resposta HTTP503 real da fixture: o cliente conservava o corpo e descartava o status. Source25 conserva o status e dá-lhe precedência sobre números no corpo. O telefone recebe apenas `provider`/mensagem fixa; um503 cujo corpo contém401 continua a ser503. Mantém a indicação de modelo ausente404, sem retry, redirecionamento ou fallback. Testes dirigidos e percursos nativos84/RPM Source25 passaram; isto não é aceitação integral da aplicação ou da instalada. |
| Cliente Android e diffs | **Fontes preservadas e transporte Kotlin verificado com o RPM .64** | Os 364 ficheiros em Git coincidem com a referência oficial. Foi usada uma cópia privada sem alargar o checkout parcial. Passaram a compilação JVM 17, 74 testes dirigidos e uma ligação TLS ao RPM .64, com recusa de pin errado, diff Codex e reconexão após revogar a partilha. APK, interface Android, câmara, biometria, relay e equipamento físico continuam por verificar. [Percurso e limites](ANDROID_LINK.md#android-client-against-rpm-64). |
| Operações originais por voz dentro do broker | **Preservadas nas variantes nativas controladas33; HOME agora atestado** | Cinco nomes originais: snapshot, listar pets sem mutação, abrir janela GTK de QA pelo lançador real, preparar URI mailto Unicode/multilinha num capturador sem correio/envio e Play/Playing em MPRIS privado. Mais recusa por permissão. Seis casos frescos com HOME/XDG/roots de credenciais atestados no processo, ASR/TTS reais com áudio virtual e provider sintético; CID/mensagens/efeitos sem replay após reinício.12 rondas sintéticas, zero modelo real/pago. Erro de HOME do helper anterior detetado, corrigido e qualificado em15 runners retidos; os históricos não isolados não foram importados. Não prova variantes de mutação do roster, recursos granulares, hardware, providers/serviços reais ou instalada. `VOICE-EFFECTS-SOURCE33-VERIFIED.json`; qualificação antiga em `VOICE-QA-HOME-ISOLATION-QUALIFICATION33.json`. |
| Mudar o pet principal por voz | **Degradado na33; corrigido35, validado novamente no RPM49 por API** |35 recuperou resposta/histórico/TTS após o CAS do roster. No49, ASR virtual→broker/modelo sintéticos→Antigravity para Codex→resposta/Edge/fim natural→reinício passou, com mesmo CID e restantes escolhas intactas. Não prova compreensão por modelo real, entrada gráfica49, hardware ou uso instalado. Recusa por permissão35 mantém o âmbito anterior. [Prova49](PETS-INSTALLED-DELIVERY-49.md). |
| Voz física/instalada, notetaker e telefone reais | **Ainda não verificado de ponta a ponta nesta comparação** | A comparação não fez gravação física, emparelhamento, chamadas pagas ou mudanças de dispositivos. A leitura passiva da configuração confirma runtimes/modelo ASR existentes e saída Edge; hotword está desligado por preferência, não é perda demonstrada. Áudio virtual, fixtures ou uma página do Hub não substituem áudio, ligação móvel e uso reais. O perfil operacional usa Codex; não foi trocado por Ollama para estes testes. Capacidades macOS não são automaticamente regressões de Linux. |
| Fluidez, consumo e experiência instalada | **Consumo medido na .62. Navegação espaçada comparada com o oficial.** | A sessão .62 mostrou 1 199,5 MiB de PSS mediana e 114,9% de um núcleo em cinco minutos de atividade não controlada. Com a mesma navegação privada, as medianas de tempo ficaram próximas e a .62 usou menos CPU e PSS. O desenho de miniaturas escondidas foi corrigido na fonte posterior à .62. [Sessão instalada](#recursos-instalada-62-2111) · [Navegação e miniaturas](#flow-navegacao-62-2111). |

**Defeito visual delimitado, em aberto12/11:** a inspeção das capturas de decisões revelou
que o primeiro cartão de permissão da68 fica cortado no X11 privado, apesar de
layout DOM completo e do clique/ACK funcionarem. A pergunta seguinte foi legível.
A captura GDK da janela nativa também mostra o corte: não é apenas a captura
WebDriver. Esperar mais, focar a janela própria e uma experiência privada sem
composição acelerada não estabeleceram a visibilidade. O cartão de recusa do oficial foi inspecionado e é completo nesse mesmo
X11. O novo RPM normal do fork também reproduz o corte. As regiões nativas de
bounding/clip e o layout DOM medem a largura completa. Experiências privadas
com composição de software, visibilidade e largura CSS não resolveram. A
hipótese de reutilizar o tratamento de primeira pintura compilou na69, mas
falhou visualmente em duas aberturas novas: o seu único delta foi retirado,
com os1322 inputs novamente iguais a Source10. A instrumentação privada71 confirmou que o WebKitWebView, a GtkBox e a GtkApplicationWindow têm allocation e clip completos de720×560, mesmo durante o pedido. Isso exclui a hipótese de um widget nativo alocado apenas à faixa visível. A experiência72 de largura inicial/natural constante também falhou em duas aberturas novas, apesar de transporte e reinício passarem. Instrumentação e protótipo foram retirados; a árvore voltou exatamente a Source10. Não permanece complexidade
de produção sem benefício demonstrado. O mesmo defeito de apresentação foi observado no resumo/partilha do RPM atual e no leitor de diffs da68/Source10, este confirmado por captura GDK própria. As linhas, o isolamento de sessões, a rota pelo centro e os efeitos/ACK continuam a passar; a visibilidade completa continua por provar. Alterar temporariamente o recorte depois da abertura fez o resumo reaparecer, mas as variantes estáticas73 (`overflow:clip`) e74 (sem raio no recorte interior) falharam desde o arranque. Ambas foram retiradas; os1322 inputs voltaram exatamente a Source10. Não repetir essas tentativas como soluções. `CARD-PAINT-CURRENT-SCOPE.json` liga fontes e imagens. A causa e o âmbito Wayland/instalado
continuam por estabelecer; a correção necessária é mostrar o primeiro comando
e os botões completos, com prova nativa, preservando as decisões governadas. A sessão Wayland encontra-se bloqueada, e não foi desbloqueada.
`DECISION-VISIBILITY-OPEN.json` conserva fontes, imagens e âmbito. A aceitação
gráfica deste cartão **continua aberta**, mesmo com os recibos de transporte
verdes. Não certificar paridade integral ou entrega instalada com estes ensaios.

**Qualificação vigente da evidência gráfica:** a repetição com compositor privado também produziu cortes no executável oficial0.5.0, tanto no drawable como na região visível do ecrã privado. As imagens anteriores legíveis do oficial continuam válidas nesses casos, mas já não bastam para atribuir o defeito ao fork. A causa permanece por determinar; não está provada uma regressão específica do fork nem a paridade gráfica. `GRAPHICAL-CAPTURE-QUALIFICATION.json` liga os casos comparáveis. Esta qualificação prevalece sobre a atribuição mais forte das observações históricas acima. O xcompmgr foi obtido como RPM Fedora44 assinado e extraído apenas em `.build`, executado só no X11 privado; nenhuma aplicação/configuração do anfitrião foi instalada ou alterada. A tentativa de ocultar outra janela foi recusada pela capacidade nativa e não se alterou a política. A documentação [GTK sobre captura de janelas](https://docs.gtk.org/gdk3/func.pixbuf_get_from_window.html) explicita limitações para regiões ocultas; também foi capturada a região visível da raiz privada, sem eliminar a incerteza de renderização. A aceitação no Hyprland real continua pendente.

**Qualificação atual12/11 — normal Source29:** os cartões de permissão/pergunta e
as vistas de resumo/partilha foram agora observados completos e legíveis no
Hyprland real, com cliques e efeitos próprios confirmados, sem alteração de
produção. `COMPOSITOR-DECISIONS-RECAP-SOURCE29-VERIFIED.json`. Isto fecha a lacuna
de apresentação destes percursos no âmbito atual privado; mantém a incerteza
histórica X11 e não certifica todos os cartões, instalada ou paridade global.

## Correção, prova e limites

`windows/src/views/pet-overview.ts` volta a compor o ticker e o leitor de diffs
existentes. A seleção usa `State.findDiff`/`State.diffsFor` e as sessões já
mantidas pelo runtime da interface. Não há novo formato persistido, permissões,
provider, polling de atividade ou motor de inferência. O ticker usa os frames do
cartão; os diagnósticos continuam nos componentes comuns existentes.

Foi também corrigida a divergência de contrato identificada na passagem:
`RuntimeGoal` inclui 67 e `RuntimeAdmissionSpec` inclui 29 e a referência tipada
`ruleOccurrence`, correspondente ao backend. Essa divergência pertence ao
desenvolvimento do fork; não é uma funcionalidade perdida do oficial. A correção
não altera os formatos Rust nem concede autoridade.

Provas privadas relevantes:

- `official2/RESULT.json`: diff original aberto pelo percurso nativo, com ambos
  os textos esperados. A primeira tentativa não selecionou corretamente a pill;
  não é apresentada como aceitação.
- `installed43-private2/RESULT.json`: o executável da instalada reproduziu a ausência do diff e conservou a navegação das doze páginas, com perfil privado; o processo operacional não foi substituído. A primeira tentativa parou no guard de posse por encontrar dois processos com o mesmo executável; a seleção foi corrigida para aceitar apenas o filho do WebDriver próprio.
- `fork60-essentials/RESULT.json`: mesma navegação e diff recuperado na candidata final.
- `fork58-exact-session/RESULT.json`: ausência do percurso no cartão da sessão
  explicitamente selecionada. Tentativas anteriores sem layer-shell ou com
  seleção ambígua não são usadas para concluir a regressão.
- `fork59-diff1/RESULT.json`: primeira recuperação funcional do diff.
- `fork60-extended/RESULT.json`, `SESSION-ISOLATION.json`, `SIMPLIFIED-CARD.json`
  e `DIFF.png`: composição final com rato/teclado, Escape, duas sessões,
  diagnósticos recolhidos e novamente abertos. A captura foi inspecionada.
- `host-native-development-20261107/build60/BUILD-RESULT.json`, na unidade privada
  existente: TypeScript, Vite e binário nativo concluídos com inputs inalterados.
- `WEB-DIRECTED-RESULT.json`: 38 testes dirigidos de diffs/hooks, retenção,
  lifecycle/viewport do cartão e sessões passaram, sem ignorados. Não foi repetida
  a suite geral anterior de 1566 casos, por não haver delta de backend/autoridade.
- `center60-before/RESULT.json`, `center62-after/RESULT.json`: ausência e
  recuperação do diff no centro, incluindo texto e sessão corretos. A61 falhou
  por rejeitar o ID zero; falha conservada, corrigida e coberta pelo teste.
- `CENTER-DIRECTED-RESULT.json`:22 casos reais passaram, incluindo ID zero,
  seleção exata da sessão e prioridade de uma decisão pendente.
- `flow62-before/RESULT.json`, `flow63-after/RESULT.json`: perfil nativo só com
  Codex. Antes, seis verificações de acessos/feedback falham; depois passam,
  juntamente com abertura real do Hub pelo menu e centro e limpeza em idle.
  O critério inicial só verificava o Hub aberto, não o destino de ditado;
  a verificação posterior exige a página exata e conserva esta limitação.
  Os eventos de estado de ditado/notas são sintéticos; iniciar gravação ou
  captar/transcrever áudio não foi ensaiado. Zero chamadas de inferência.
- `host-native-development-20261107/build63/BUILD-RESULT.json`: TypeScript,
  Vite e binário nativo concluídos com inputs inalterados, sem instalação.
- `code67-multiline/RESULT.json`, `CODE-LAYOUT.json`, `MEMORY-OPEN-LAYOUT.json`
  e `CODE.png`: cinco linhas de código via HTTP real/sintético, reabertura da
  conversa, área útil recuperada e controlos de memória ainda acessíveis.
  A captura foi inspecionada. `code66-layout-before/CODE-LAYOUT.json` conserva
  a medição anterior; ter o código no DOM não provava a qualidade da vista.
- `code67-incomplete/RESULT.json`: ronda sem marcador final corretamente
  recusada, erro visível após refresh e um único pedido HTTP, sem comandos.
  Os primeiros ensaios de código falharam por uma fixture sem esse marcador;
  isso não é regressão do protocolo. Expuseram, contudo, a perda real de erro
  na interface. A primeira correção só cobria o cartão; a65 falhou no percurso
  flutuante, corrigido na66 usando a falha por sessão já existente.
- `CHAT-ERROR-BEFORE.log`, `CHAT-FLOATING-ERROR-BEFORE.log`: os dois casos novos
  reproduziram a perda antes da correção. `CHAT-FINAL-DIRECTED-RESULT.json`:
  64 testes reais passaram, sem ignorados; incluem erro após refresh,
  isolamento entre conversas e ausência de reenvio. O delta67 seguinte é só
  a altura de conteúdo dos controlos de memória e tem prova nativa própria.
- `voice-workers1/RESULT.json`: workers ASR/TTS reais com referências da
  configuração instalada, frase sintética e preferências byte a byte estáveis.
  Não prova captura, reprodução, interrupção ou resposta por uma sessão nativa.
- `voice67-native10/RESULT.json`, `VOICE-CONVERSATION.json`,
  `RESTARTED-CONVERSATION.json`, `AUDIO-LINKS.json` e áudio privado: cadeia real
  pela API nativa, captura/reprodução exclusivamente virtuais, uma inferência
  sintética, Stop e preservação sem replay. As tentativas1..9 foram conservadas:
  configuração de módulos/clientes/portas/relógio da fixture, inventário de
  janelas ocultas com timeout de causa indeterminada e servidor sintético
  inicialmente sem o protocolo de voz `/api/chat`. O Pulse de testes de volume
  reutilizado inicialmente não incluía adaptador/metadata para reproduzir áudio;
  foram acrescentados apenas no servidor privado. A descodificação GStreamer
  do MP3 também passou isoladamente. Não houve alteração do servidor de áudio
  real, instalação, modelo descarregado ou quota paga. A voz escolhe o perfil
  da fábrica; preencher apenas campos legados de modelos no settings da fixture
  não substitui essa escolha. Não se classificaram essas diferenças como
  regressões do produto.
- `host-native-development-20261107/build67/BUILD-RESULT.json`: TypeScript,
  Vite e binário nativo da composição atual concluídos com inputs inalterados.
- `flow67-native2/RESULT.json`, `center67-native/RESULT.json`: destinos exatos
  do Flow, feedback com Codex e catálogo vazio sintético, diff nativo contextual
  e pelo centro na sessão correta, na mesma composição67. A primeira tentativa
  `flow67-native` teve timeout WebDriver durante o inventário inicial de janelas,
  antes dos critérios Flow. A causa não foi estabelecida; a falha foi conservada
  e a retoma sequencial passou. Não foi tratada como regressão comprovada.
- `BUILD68-NATIVE-SCENARIOS.json`: os cinco percursos passaram na composição
  atual, com recibos próprios: `voice68-graphical`, `flow68-native`,
  `code68-readable`, `code68-incomplete` e `center68-native`.
  Voz: menu do pet, feedback visível, áudio virtual com sinal, Stop, conversa
  preservada e reinício sem replay. Flow: o botão de ditado desligado abre a
  página exata de configuração, sem iniciar captação ou descarregar modelos.
  Código: bloco de várias linhas legível e retido, controlos de memória
  acessíveis; erro: recusa/feedback/revisão sem reenvio; diff: sessão e conteúdo
  corretos no cartão e centro. Zero quota real Copilot/GitHub ou inferência paga.
  `voice68-graphical/VOICE-REPLY.png` foi inspecionada.
- `host-native-development-20261107/build68/BUILD-RESULT.json`: build atual com
  inputs inalterados. A68 não herda aceitação da67: tem as cinco provas nativas
  acima. O delta de produção67→68 é apenas o encaminhamento de ditado desligado
  em `flow/mod.rs`; a cadeia Authority/formatos6/7 continua sem delta de produção.
- `NOTES-NATIVE-COMPARISON.json`, `notes-official-natural1/RESULT.json` e
  `notes68-natural1/RESULT.json`: notetaker comparado ponta a ponta, incluindo
  UI, áudio virtual dos dois canais, modelos reais, exportação e reinício.
  `TRANSCRIBED-NOTE.json`, `CAPTURE-mic.wav`, `CAPTURE-system.wav`, `NOTE.md`
  e `RESTARTED-NOTES.json` retêm as provas privadas. Os resumos por modelo
  foram desligados; zero inferência de conversa ou quota paga.
  `model-assets/MODELS-REFERENCE.json` liga os seis modelos existentes aos
  hashes do fonte. A falta deles num perfil gerado não era perda da função.
  As primeiras tentativas conservam falhas da fixture (diretório/seletor/
  observadores) e uma amostra sintética com reconhecimento deficiente. Foi
  corrigida uma mistura incorreta dos canais estéreo da fixture; não se
  atribuiu causalidade ou regressão ao produto sem comparação equivalente.
- `DICTATION-NATIVE-COMPARISON.json`, `dictation-official-x11-2/RESULT.json` e
  `dictation68-x11-2/RESULT.json`: ditado e destino de colagem confirmados,
  transformações e preservação descritos acima. Os logs nativos confirmam
  Parakeet em ambos, com modelos privados copiados dos ficheiros pinned.
  `PASTED.png` da68 e `NOTES.png` de ambos foram inspecionadas; a captura das
  notas mostra a vista de resumo vazio, não certifica qualidade da transcrição.
  O texto transcrito/exportado está nas provas estruturadas próprias.
  A primeira execução oficial preparou o clipboard com uma colagem assíncrona
  que chegou ao campo do ensaio; a preparação foi corrigida para `flow_copy`
  já existente no oficial. A execução seguinte passou. Não era evidência de
  regressão do produto. A primeira execução68 parou numa condição Python da
  fixture antes do ensaio de tecla/colagem; a falha foi conservada.
- `COPY-DOCUMENT-NATIVE-COMPARISON.json`, `COPY-DOCUMENT-BATCH3.json`,
  `documents-official-x11-3/RESULT.json` e `documents68-x11-3/RESULT.json`:
  entrega por GTK, conteúdo no HTTP e cópia exata passaram em ambos.
  `copy68-x11-2/RESULT.json` também cobre cópia sem anexo. A68 acrescenta
  provas de isolamento/cópia gerida/reinício/renovação por UI, com recibos
  `MANAGED-COPY.json`, `RESTARTED-DOCUMENT.json` e `ATTACHMENT-REVIEW.png`.
  A revisão foi inspecionada. As imagens da origem e destino foram observadas
  antes do arrastar; no oficial, `CURRENT-ACTION.png` foi observada antes de
  clicar “Ask a question about it” no canvas. Não se confundiu DOM oculto com
  um controlo gráfico alcançado. As primeiras execuções conservam erros de
  fixture: janela auxiliar, título X11 Latin1, animação inicial/recolha da ilha
  e consulta de documento a partir do Hub (corretamente recusada). A repetição
  usa o cartão certo e os percursos públicos; nenhuma política foi desativada.
- `DECISIONS-VERIFIED-COMPARISON.json`, `decisions-official-x11-2/RESULT.json`
  e `decisions68-x11-3/RESULT.json`: recusa e pergunta pela UI nativa, decisões
  devolvidas pelo socket original e reinício sem inferência/replay. Os cartões
  têm capturas próprias. O seletor inicial incluía indevidamente o texto da tecla
  de atalho (`DenyN`); a repetição atual2 passou a recusa, mas parou numa
  asserção de presença de DOM já oculto. A3 valida o ACK e a pergunta seguinte,
  sem confundir esse DOM com um pedido ativo. O resultado final do original2
  continua válido, sem o repetir por mero aumento de contagem.
- `release44-parity-current-20261112/{BUILD-RESULT,PACKAGE-PREPARED}.json`, na
  unidade privada existente: novo RPM normal Source10,1322 inputs exatos,
  SHA `c3f640229ece93f1fcea38e846d724166921be62f1962999798ee1f0480044b5`,
  13 ficheiros validados pelo instalador restrito existente, incluindo relay
  original e recursos Copilot, arquivo de fontes e ELF. Só foi preparado um
  prefixo dentro do projeto; a instalação43 não foi substituída. O executável
  empacotado tem SHA `1480a0a582579ae9b4c9497653ffaf83cdab707fdb696fe9ef73f10b6d35ef99`.
  `normal44-card1` confirma recusa/ACK e o corte gráfico. Os ensaios69 e de
  estilos são diagnósticos falhados para a correção visual; não são aceitação.
- `official-essentials*/HUB-PAGES.json`: navegação original observada; os resultados
  finais dessas execuções continuam falhados/inconclusivos para decisões. Não
  os apresentar como execuções totalmente verdes.

O campo `syntheticConsent:true` herdado do template nos primeiros recibos de
diffs não representa uma decisão: esses ensaios não chamaram operações de
autoridade/consentimento. `humanApproval` e `humanAcceptance` permaneceram falsos;
o harness seguinte corrigiu o metadado. A prova funcional dos diffs está nas
entradas/saídas nativas e nas verificações de sessão, não nesse campo.

Uma amostra em `official-idle1/METRICS.json` pediu ocultação de janelas, mas não
estabeleceu o mesmo estado de repouso do runtime. Não é um benchmark de idle e
foi excluída da comparação de desempenho. Ocultar uma janela nativa, recolher o
cartão, pausar o runtime e sair da aplicação não são estados equivalentes.

Após reinício do servidor do assistente, duas chamadas ao anfitrião ficaram
sem devolver resultado e foram interrompidas antes de criarem os diretórios de
ensaio. O acesso voltou a responder e os percursos foram executados com recibos
próprios. Não se atribui a interrupção ao Coucou ou a uma rejeição de autorização.

As amostras seguintes pedem ocultação das janelas e pausa pelo evento nativo do
tray, após dois segundos de estabilização; duram aproximadamente seis segundos,
sem builds simultâneas ou inferência. Medem a família de processos própria por
`/proc`, usando PSS para memória e CPU em percentagem de um core. Não medem o
perfil operacional do proprietário.

| Executável / recibo | Processos | PSS final | CPU observada | Estado DOM após a pausa pedida |
| --- | ---: | ---: | ---: | --- |
| Oficial0.5.0 / `official-paused/METRICS.json` | 9 | 404,5 MiB | 87,8% de um core | Sem `.view.on` |
| Instalada43 em perfil privado / `installed43-paused/METRICS.json` | 11 | 537,8 MiB | 36,2% de um core | `pet-overview` ainda marcado `.on` |

Há mais processos e memória nesta composição do fork, observação que merece
acompanhamento. Os estados de interface não ficaram equivalentes, a amostra é
curta e a otimização do binário43 não foi verificada independentemente. A
apresentação e os serviços residentes também diferem por desenho. Não se conclui
uma melhoria de CPU, um benchmark de idle ou uma regressão de desempenho global.
A fluidez e o consumo no uso instalado continuam sem aceitação própria.

## Comparação de recursos dos RPMs,12/11

Quatro perfis novos, na ordem oficial/fork/fork/oficial, usaram os executáveis
empacotados e o mesmo estado funcional observado: Hub Ditado visível, ditado
inativo, avatar recolhido, pausa do backend confirmada pela API, sem objetivos,
inferência ou compilação do projeto. O X11 e o GL por software são privados.
As capturas dos dois Hubs foram inspecionadas. As famílias de processos ficaram
estáveis, e todas as leituras de PSS estavam disponíveis.

| RPM / recibo | Duração | Processos | PSS final | CPU, percentagem de um core |
| --- | ---: | ---: | ---: | ---: |
| Oficial0.5.0 / `resources-official-release3` | 13.00s | 9 | 351.1 MiB | 204.3% |
| Fork44/Source10 / `resources-fork-release3` | 13.11s | 11 | 427.2 MiB | 104.4% |
| Fork44/Source10 / `resources-fork-release4` | 13.12s | 11 | 429.6 MiB | 104.2% |
| Oficial0.5.0 / `resources-official-release4` | 13.02s | 9 | 349.8 MiB | 204.5% |

O fork tem cerca de78MiB PSS adicionais nesta amostra e dois processos a mais.
Grande parte da diferença observada está nos processos WebKit, com uma parte
no processo principal; isto não prova a causa ou memória desperdiçada.
A CPU foi inferior no fork nesta carga.100% nesta coluna corresponde a um
core, não à máquina completa. Não concluir uma melhoria global de CPU,
fluidez instalada ou um defeito de memória a partir destas amostras curtas.
O PSS ainda variou durante cada intervalo. A geometria do controlador e o
conteúdo dos Hubs diferem por desenho: o fork inclui a cópia explícita de
texto e desloca o conteúdo seguinte. Não são imagens idênticas nem um teste
de idle, áudio físico, inferência, uso com os pets ativos ou latência percebida.

`RESOURCES-VERIFIED-COMPARISON.json` indexa os quatro `METRICS.json` e os
estados observados. As tentativas anteriores1/2 pararam na preparação, sem
medição: o observador não podia substituir `invoke`, que é não configurável e
não gravável. Foi usado o ACK direto da API nativa, preservando a política.
O próximo passo é avaliar o custo das superfícies/processos residentes em
função da sua utilidade, com compatibilidade; não assumir que toda a diferença
de arquitetura é uma regressão nem abrir infraestrutura nova para a mascarar.

## Continuação delimitada

1. Verificar outras integrações essenciais e preparar a composição normal
   para recursos comparáveis e entrega. Recusa/perguntas também passaram no
   âmbito sintético privado.
   Notetaker e ditado/colagem/dicionário/snippets/estilos passaram nos âmbitos
   nativos descritos, tal como cópia e anexos de texto; resumo/limpeza pagos,
   Wayland, outros formatos/providers e hardware ficam abertos. Reutilizar o áudio
   virtual já comprovado, sem infraestrutura de produto adicional. Voz,
   código/diffs já têm prova nativa delimitada; continuam pendentes hardware,
   providers reais nos âmbitos autorizados e experiência operacional.
   Corrigir regressões essenciais confirmadas antes de novas capacidades.
2. Reutilizar os recibos de comparação concluídos. Medir CPU/PSS com o
   mesmo perfil, estado observado, intervalo e sem builds simultâneas; explicitar
   diferenças de otimização entre release oficial e binário de desenvolvimento.
   A comparação dos RPMs acima substitui a amostra antiga para este estado
   privado específico. Avaliar o custo residente e depois a fluidez visível e
   a instalação operacional no âmbito revisto.
3. Priorizar outras regressões essenciais que essa comparação confirmar. Não
   abrir nova infraestrutura ou restaurar pets indiscriminadamente.
4. Revisão e sucessão 6/7, três respostas HTTP sintéticas, Stop/reinício
   e abertura do histórico restaurado passaram no próprio RPM 62 em
   perfil privado. A .62 foi instalada e a página Pets abriu no perfil
   operacional. A aceitação humana do percurso 6/7 continua pendente.
   [Provas do RPM 62](PETS-VALIDATION.md#rpm62-nativo-2111).

Sem commits, pushes, merges, alterações globais, substituição instalada, quota
Copilot/GitHub ou inferência paga. O objetivo integral permanece aberto.

O produtor opt-in em `rule_revision_tests.rs` está incluído apenas sob
`cfg(test)`. A equivalência de produção desse incremento relativamente à60
foi registada em
`.build/pet-goal-execution-next-r1/rule-governed-revisions/SOURCE3-PRODUCT-EQUIVALENCE.json`.
Os incrementos posteriores de interface têm manifestos próprios até68 e provas
delimitadas; não são apresentados como equivalentes a todas as provas da60.

### Controlos Spotify/MPRIS comparados12/11

`MPRIS-VERIFIED-COMPARISON.json` liga os resultados de `mpris-official-release2` e `mpris-fork-release2` aos hashes dos RPMs. Perfis HOME/XDG, D-Bus e X11 privados; nenhuma conta, credencial, reprodução pessoal ou inferência. A UI atual conserva a superfície Música/Som/Aplicações intencionalmente partilhada com os mecanismos existentes. As provas verificam apenas Música. O volume escolhido chegou ao leitor em59% no original e60% no fork, dentro de3 pontos do alvo60%; o seek foi90s e89,55s, dentro de3s do alvo90s e dirigido a `/com/spotify/track/qa1`. Não se deduz identidade de precisão entre geometrias distintas. As tentativas de harness1 falharam na seleção do pet original e no tipo do keysym; foram corrigidas apenas na fixture, sem alteração do produto.

### Resumo, exportação e guarda-roupa comparados12/11

`RECAP-VERIFIED-COMPARISON.json` liga oficial `recap-official-release2` e RPM atual `recap-fork-release3`. A semana anterior foi criada como fixture explícita antes do arranque, sem alterar o relógio;20min, uma sessão,3ficheiros, +12/−4linhas e2comandos. Separadamente, eventos reais no socket acrescentaram uma sessão atual com1ficheiro, +2/−1linhas e1comando; o texto de comando foi apenas observado, nunca executado. O botão Share image e os controlos de ocultar/guardar produziram PNGs reais na pasta Pictures do HOME privado. Hash com nomes `e7050a4d27c413649d6ab87be758108c6bf1afb49d84377211f1ba278f0eefd4`; sem nomes `29f51bc4a1a06e1edeef344a9ab47157c8ba964a1b24e142863dde0456338a3f`, iguais entre os dois RPMs. A preferência e o PNG sem nomes sobrevivem ao reinício. A inspeção confirma a ausência do projeto no PNG oculto, mas também o corte da própria UI de resumo/partilha do fork: backend e ficheiro corretos não certificam o percurso gráfico. Crown foi selecionada e persistiu nos dois; o ficheiro legado preservado não é a fonte canónica atual. Tentativas anteriores do harness ficam conservadas; nenhuma alteração do produto foi necessária para estes contratos.

### Atalhos globais comparados12/11

`SHORTCUTS-VERIFIED-COMPARISON.json` liga `shortcuts-official-release4` e `shortcuts-fork-release5` aos RPMs identificados. A tecla foi gravada no editor real, e os efeitos chegaram do XTest privado ao registrador nativo, ao callback e à UI/persistência. O teste escreveu no chat depois do atalho, com foco nativo, sem clique para focar o campo e sem enviar mensagens. No fork, o recibo de revisão confirmou registration=verified/publication=published; após reinício, as preferências mantêm-se e a tecla não executa sem nova revisão. Esse custo de interação decorre do contrato existente de efeitos por sessão e deve ser distinguido da preservação dos dados. Foram conservadas as falhas anteriores do harness; seleção de campo oculto, tipo input/textarea e navegação assíncrona não são regressões do produto. A inspeção do chat atual também identificou controlos de recuperação de rascunhos sempre expostos, mesmo com rascunho vazio, ocupando espaço que o original dedicava à conversa. A próxima correção recolhe estes controlos, preservando conflitos/erros visíveis e todas as operações de recuperação.

### Compositor e rascunhos: correção funcional na privada77

Fonte atual `SOURCE19-DRAFT-TOOLS-LAYOUT.json`,1322 inputs. O delta perante Source10 limita-se a `views/draft-status.ts`, `style.css` e à geometria do prompt em `island/island.ts`; backend, autoridade e formatos de rascunho não mudam. O componente comum aplica-se aos compositores compacto e flutuante, preservando escolhas por pet e conversa. Os64 testes de persistência/identidade/CAS/recuperação e39 de chat passaram com o runner do projeto; a aceitação nativa é prova própria, não uma consequência da contagem. TypeScript, bundle e compilação nativa77 passaram com inputs exatos.

`DRAFT-TOOLS-VERIFIED.json` liga a build e `draft-copy77-pointer1`. Teclas e ponteiro físicos privados acompanham os controlos observados. Foram testados: editor de atalhos e revisão própria, escrita no chat do Ollama configurado como fixture-free, guardar, abrir/fechar o seletor, criar outra conversa (fechando a recuperação antiga), rever origem/destino/texto, confirmar a cópia pelo runtime e reler ambos os registos nativos. A origem permaneceu exatamente igual; o destino recebeu os mesmos nove caracteres. Erro com65537 unidades mantém todo o texto e o aviso fora da secção recolhida; corrigir o texto permite voltar a guardar. Origem e destino, seleção de conversa e rascunho sobreviveram ao reinício, sem enviar inferência. A conversa flutuante mostra os mesmos controlos recolhidos. São decisões automatizadas de fixture privada; não constituem aprovação humana.

A hipótese de um defeito de Escape foi rejeitada: a imagem anterior à tecla já mostrava o cartão oculto, e o dispatcher inicialmente alterado nem processa essa tecla. O delta experimental78 e o teste com pressuposto errado foram retirados, com Source19 novamente exato. Sincronizar o ponteiro privado permitiu concluir o percurso sem alterar a lógica de Escape. Não reaplicar esse delta. A renderização geral continua inconclusiva no ambiente X11 privado, inclusive em casos oficiais; funcionalidade nativa, visibilidade DOM e capturas parciais não certificam a apresentação ou a instalação. Nesse checkpoint, Source19 ainda não estava no RPM normal. O empacotamento Source19 foi depois concluído e validado no âmbito descrito abaixo; Source10 e a instalação43 permanecem preservados.


### Código e hooks na composição77,12/11

`CODE-COPY77-VERIFIED.json` liga `code-copy77-current1` ao executável77 e ao
manifesto Source19. O chat flutuante atual recebeu a resposta de um provider
Ollama HTTP sintético explicitamente configurado como fixture-free; as cinco
linhas de código ficaram legíveis na captura inspecionada e o botão de cópia
pôs exatamente o bloco completo no clipboard privado. Um único pedido de
inferência sintético, nenhum provider real ou quota. A prova anterior de
anexos/reinício mantém a sua composição original; não foi repetida nem
reatribuída à77.

`HOOK-CHANNELS-VERIFIED-COMPARISON.json` compara `hooks-official-relay2` e
`hooks-fork77-relay3`. Foi executado o relay oficial fixado por SHA-256
`92bf5488ce4ff290d4573ceca24ad3d3e63520d1c16cd30b22073bffe14b8b67`,
igual aos bytes do RPM normal do fork. A77 de desenvolvimento não o contém
junto do ELF; o ensaio identifica explicitamente essa composição. Os eventos
chegaram pelo socket nativo, e uma subscrição passiva observou os payloads
recebidos sem substituir handlers ou estado da aplicação.

Codex, Copilot e Antigravity conservaram atividade, nome do pet, texto anterior
/novo e contagens. O mesmo identificador de sessão nos três pets não misturou
alterações. Ctrl+E e Escape vieram do teclado X11 privado; Stop conservou o
diff, e o botão real do centro partilhado abriu a alteração da origem certa.
A normalização usou tool_name/tool_input para Codex, toolName/toolArgs JSON
para Copilot e toolCall/conversationId/workspacePaths para Antigravity, além
dos seus nomes de eventos. Os comandos eram só metadados e não foram executados.
Não houve decisões de autoridade, inferência ou consumo de quota neste caso.

**Limite do contrato:** os diffs usaram Edit canónico com a forma de transporte
de cada adaptador. A atividade também usou nomes próprios de ferramenta, mas
isto não certifica todos os formatos de edição de uma IDE real. O código
original e o atual aceitam Edit/MultiEdit/Write para construir diffs; outros
nomes não se tornam suportados por terem passado este ensaio. Integrações
reais, semântica de todos os eventos e quotas permanecem por verificar.

As capturas de Copilot e Gravity mostram o leitor legível; a primeira de Codex
continua cortada no X11 privado, e uma do oficial também. A prova funcional
dos três canais não fecha a paridade gráfica global. As tentativas anteriores
ficam preservadas: seleção repetida de uma pill ausente no original, primeira atividade ausente num arranque cuja prontidão não foi observada e
título nativo incorreto da janela Pets.
O ensaio final esperou o arranque, observou os payloads nativos e usou os
alvos revistos. A causa da primeira ausência não ficou isolada; não se
atribui uma correção de produto. Não houve delta de produto nesta comparação.


### RPM normal Source19 preparado e ensaiado12/11

A composição normal `0.5.1-mochi.44` contém agora as correções atuais de
rascunhos/geometria e a recuperação anterior dos diffs, Flow e erros do chat.
A receita Fedora existente compilou com Cargo locked/offline e1322 inputs
exatos. RPM privado: SHA-256
`4f5611d67c6bf222be80d6d501ff78962b33317e8fe86a3666b6995cd2a8348b`.
Executável empacotado:
`bab621408835024048cdfb80fb8ffdcc57cd0798f5bed2f0edc66636c23898e8`.
O RPM e os outputs Source10 foram conservados antes da compilação.

`SOURCE19-NORMAL-PACKAGE-VERIFIED.json` liga a fonte, o arquivo, o pacote,
o prefixo de QA e os recibos `code-copy44-source19-1` e
`hooks-normal44-source19-1`. O instalador restrito existente foi reutilizado
somente para extrair13 ficheiros permitidos para dentro do projeto. Relay
original igual, recursos Copilot iguais às fontes, ELF/ldd sem dependências
em falta ou caminhos do ambiente de desenvolvimento. Nenhum ficheiro da
instalação operacional ou de sysworkspace foi alterado.

O próprio executável do RPM, com identificador normal e perfis novos, passou
código/cópia exata, os três canais de atividade e Edit canónico, Ctrl+E/Escape,
Stop, isolamento por pet e abertura pelo centro. Uma inferência HTTP sintética
para código; zero chamadas reais/pagas. A inspeção mostra o código legível e
o diff Gravity legível; as primeiras capturas de Codex e Copilot continuam
cortadas no X11 privado. Não se deduz aceitação gráfica global dos ACKs ou do DOM.
As aplicações de QA terminaram, e a instalação43 mantém PID5480 e hash original.

**Estado de entrega:** implementado e integrado nas fontes; compilado e
validado nativamente nos percursos privados enumerados; não instalado nem
aceite humanamente. O estado da sessão Hyprland foi novamente consultado:
bloqueada e ecrã desligado. A autorização da estação exige sessão desbloqueada
para a interação gráfica; o pedido de disponibilidade já está pendente.
Não se desbloqueou a sessão nem se presume resposta/consentimento.

`RULE67-CURRENT-CORE-SCOPE.json` compara a fonte da prova6/7 com Source19:
323 inputs de runtime/autoridade/preservação comparados permanecem iguais, e
os15 inputs alterados estão enumerados. Essa equivalência só reutiliza as
conclusões do core; a revisão/sucessão6/7 na UI da composição normal atual,
a apresentação Hyprland/hardware e a entrega instalada continuam por concluir.
O próximo passo retoma essas aceitações e as integrações reais autorizadas;
novas capacidades de autonomia permanecem atrás da recuperação útil. Commits,
pushes e merges continuam suspensos.


### Aceitação técnica6/7 no mesmo RPM Source19

`RULE67-SOURCE19-VERIFIED.json` fecha os percursos técnicos desta composição:
revisão própria6/7, três respostas sintéticas, recuperação da sondagem pelo
widget real, Stop/reinício sem replay, preservação CLI do perfil executado e
janela restaurada com regra/resposta histórica e recusa de execução. Os oito
registos assinados, três Goal67 e catálogo de regras conservaram os seus bytes.
As continuações e falhas de fixture permanecem enumeradas; não se misturam
provas da60 nem se fabricam decisões humanas. [Contrato, percurso e limites](PET-RULES.md#revisão67-aceitação-técnica-no-rpm-source191211).

Não houve nova infraestrutura ou mudança do produto nesta unidade. A aceitação
humana/instalada e gráfica geral permanece aberta. O próximo trabalho verifica
restantes percursos úteis de voz/hotword/atalhos e integrações, e retoma Hyprland
quando disponível. Novas capacidades de autonomia continuam atrás dessa
prioridade. Não repetir esta aceitação6/7 na mesma fonte sem delta ou risco novo.


### Escuta contínua: regressão confirmada e correção Source21

A composição normal44/Source19 (`hotword-source19-native3`) falhou antes de
abrir o microfone virtual, apesar de conservar a preferência ligada e escrever
“listening”. Com os mesmos dispositivos e configuração, o turno manual abriu a
captura (`hotword-source19-diagnose4`). A79 acrescentou diagnóstico e confirmou
“gravação das preferências” na preparação, sem haver uma nova gravação: a
pré-carga e a escuta consultavam a origem simultaneamente
(`hotword-source20-diagnose5`). Não se atribuiu a falha a permissões/dispositivos.

A correção em `voice/mod.rs` captura `Input` uma vez antes dos dois workers e
reutiliza o seu `RecognitionScope` na pré-carga. Mantém as verificações de
origem/configuração após esperas e durante o percurso. `voice/input.rs` espera
pelo mutex de Flow antes de comparar o microfone e a reprodução; leitores
correntes deixam de revogar um turno só por ocuparem esse mutex. Mudança real,
origem inválida ou mutex envenenado continuam a recusar. Falhas anunciam uma
mensagem existente no Flow; o log “listening” passa para depois da abertura real.
Não há novo provider, fallback, formato persistido ou política desativada.

`hotword-source21-native6` passou nativamente na80: frase negativa com ASR real e
zero inferência; “Ok, cucu…” positivo, uma chamada ao endpoint/modelo sintéticos
configurados, resposta visível em `ai_ollama`, áudio Edge efetivamente registado
na saída virtual, UI-off e conversa idêntica após reinício, sem replay. O pico
flutuante observado foi1,058768; isto prova sinal, não qualidade/ausência de
saturação. A captura do pet foi inspecionada. `hotword-source21-lifecycle8` passou
três arranques/paragens e72 leituras nativas de estado/configuração, sem nova
inferência. A7 interrompeu apenas numa referência de controlo substituída antes
da tecla/clique de desligar; permanece preservada, sem repetir efeitos incertos.
Os8 testes de Input e6 de RecognitionScope passaram na fonte exata.

O RPM oficial0.5.0 (`hotword-official-gate2`) conservou o gate real sem chave
Claude, a preferência e o desligar/reinício. Não se inventou chave nem se fez
inferência paga para ultrapassá-lo. A comparação não certifica o percurso de
voz Claude integral do original, outros providers, microfone físico ou entrega
instalada. O perfil operacional43 continua intacto.

A disponibilidade Hyprland foi confirmada nesta unidade. Duas observações em
janelas regulares de QA da80 não aceitaram o diff: o cenário derivado antigo
observou um overview sem dados correspondentes e não obteve um diff pelo
atalho. As capturas por endereço/PID exatos mostram regiões transparentes e
sobreposição. Isto não prova regressão de pintura nem paridade; os resultados
negativos ficam preservados, sem workaround de produto. É necessário retomar
com o cenário de hooks atual já comprovado e a composição gráfica aplicável.


O mesmo percurso passou depois no **RPM normal Source21**, SHA
`ccfe8f70b41adeab8a0db1fb78fc75582c3de2b0020c2ac533e02c765b2ee07d`,
sem instalar sobre a43. Os13 ficheiros permitidos, relay original, recursos
Copilot, arquivo de1322 inputs e transformação única do marcador RPM foram
verificados pelo procedimento de prefixo já existente, dentro do projeto.
O binário extraído é `c6d81f2b6b9793d1c9659bd26b86804dec733a8dbad599ffa8ab64be29896f5e`.

`hotword-normal44-source21-10` passou áudio negativo/positivo, ASR, uma resposta
HTTP sintética, resposta do pet, sinal Edge, UI-off e conversa preservada após
reinício sem replay. O pico observado1,069367 continua sem certificar qualidade.
`hotword-normal44-source21-missing11` passou microfone ausente: **nenhum outro
input foi aberto**, zero inferência, aviso efetivamente visível no pet,
desligar e reinício. As duas capturas do pet foram inspecionadas. O aviso usa o
cabeçalho Flow existente (“Dictation”), ainda genérico para este erro de voz;
não se certifica uniformidade editorial da UI.

Recibo agregado privado: `HOTWORD-SOURCE21-VERIFIED.json`. O delta de produção
relativamente a Source19 contém apenas `voice/mod.rs` e `voice/input.rs`.
A aceitação6/7 anterior mantém a composição Source19 identificada; não foi
renomeada para Source21. A revisão gráfica geral, qualidade/hardware, outros
providers e entrega instalada continuam abertas. A43 conserva o SHA anterior
e foi observada a correr no PID2631834; o PID5480 da fotografia anterior já
não é o processo atual. Os processos privados terminaram, e a disposição das
duas janelas de terminal existentes voltou a ser observada após a QA.


### Source22: primeiro uso da conversa e ensaio gráfico qualificado

O RPM Source21 passou novamente os três canais de hooks, pela cópia verificada
do relay oficial: nomes de ferramentas, `Edit` canónico, Ctrl+E/Escape nativos,
Stop, isolamento entre pets com o mesmo SID e abertura pelo centro. Recibo
`hooks-normal44-source21-current1/RESULT.json`; zero inferência/execução.

A análise das duas tentativas Hyprland anteriores encontrou uma condição de QA
incompatível: forçavam `COUCOU_LAYER_SHELL=0`. Em Wayland isso dá modo Off;
não é criada a superfície companions e `CompanionsLink.init` termina antes de
registar `pet-session-select`. Os logs e os manifests sem companions confirmam
a cadeia. Não eram uma prova do encaminhamento de produção. Recibo
`HYPRLAND-REGULAR-QA-QUALIFICATION.json`. O inventário do compositor permite
identificar layers por endereço, PID e geometria; não se capturou a instalada.
A sessão estava desbloqueada mas com ecrã desligado nesta continuação.

Para reduzir trabalho de UI sem uso, Source22 conserva o estado nativo da
conversa mas cria `pet-chat` apenas depois de validar um pedido de abertura.
A criação e posicionamento continuam em GTK, com os mesmos contratos, origem,
generation/open_gate, arrasto de documentos e fecho. Não muda provider/modelo,
dados, formatos ou permissões. O frontend já subscreve antes de ler o contexto
inicial, permitindo o primeiro carregamento sem perder o evento.

A81 passou abrir por Expandir, recusar destino inválido sem criar webview,
escrever/guardar um rascunho, fechar/reabrir na mesma janela e reiniciar com
conversa/rascunho idênticos e janela ausente até novo uso. Captura legível
inspecionada; primeira disponibilidade observada1,027s, sem certificar latência
instalada. `lazy-chat81-native4/RESULT.json`,12 testes nativos de destino e6
casos web passaram. As tentativas de QA anteriores permanecem preservadas:
o nome de vista antigo, o clique sem sincronização e a espera WebDriver no
webview ocultado não são aceitação do produto nem novas regressões confirmadas.

O **RPM normal Source22** passou os mesmos percursos, incluindo os três canais
de hooks/diffs/centro, hotword virtual e documento/código/cópia/reinício com
renovação explícita da leitura. A receção de documentos instalou os handlers
na janela criada no primeiro uso; o corpo HTTP observado contém os bytes da
cópia gerida, não a alteração posterior do ficheiro de origem. O código visível
e o clipboard privado são iguais. A espera do contexto após reinício foi
corrigida na QA; os resultados parciais anteriores permanecem preservados e
os três percursos já válidos não foram repetidos. Recibo
`NATIVE-PACKAGE-RESUMED-RESULT.json` na unidade privada Source22.

A comparação terminou na ordem oficial → Source21 → Source22 → Source22 →
Source21 → oficial. Os valores PSS abaixo são a média dos valores finais dos
dois ensaios por versão, cada um com25 amostras em cerca de13s.

| Executável identificado | Processos | WebKit web | PSS médio final | CPU médio,1 núcleo |
|---|---:|---:|---:|---:|
| Oficial0.5.0/841a81ce | 9 | 4 | 357,59MiB | 206,91% |
| Fork normal44/Source21 | 11 | 6 | 432,89MiB | 106,47% |
| Fork normal44/Source22 | 10 | 5 | 406,54MiB | 106,20% |

O delta remove **um WebKit e26,35MiB** de PSS médio final relativamente a21;
o CPU ficou essencialmente igual. O fork ainda excede o oficial em48,95MiB
e um processo neste cenário. Atribui-se a remoção à janela adiada, apoiada
pelo manifest sem `pet-chat` e pela diferença6→5 de processos WebKit; não se
atribui o custo restante a uma causa ou leak sem outra prova.

Âmbito: Hub Dictation observado, backend pausado, perfis vazios, conversa nunca
aberta, X11 privado, renderização software pedida, sem inferência/compilação do
projeto durante as amostras. Conjuntos de processos estáveis e PSS legível nos
seis ensaios. Não certifica repouso global, GPU, fluidez instalada ou consumo
com modelos carregados. Não se compara um debug com um release.

Recibo agregado `LAZY-CHAT-SOURCE22-VERIFIED.json`. RPM SHA
`2ed6fe9e2b2d44a0841937db5f6d274410690e26afbadd2a375f9cc3a4b7c62b`;
binário extraído `4c40ea6a84e8d488aa7e492a2d27c2e85caca24fb5bb1d43f67e7e17d2648e93`.
Fonte exata1322 inputs; delta de produção só `lib.rs` e `pet_chat.rs`.
A43 conserva binário, preferências e fábrica/modelos operacionais, verificados
por SHA, e foi observada em execução no PID2631834. Não houve instalação,
publicação ou nova alteração de formato/ledger.

A resposta real com o modelo local gratuito escolhido foi entretanto verificada
no âmbito da secção seguinte. As escolhas operacionais Gemma3:4b/Qwen3:8b
permanecem intactas; a aceitação não se estende a todas as capacidades/modelos. A apresentação
Hyprland requer a composição layer-shell correta e ecrã disponível; o cenário
preparado identifica cada superfície por PID/endereço/geometria. Gráfica geral,
hardware, outros providers e entrega instalada continuam por verificar.


### Modelo local real: resposta, código e preservação Source22

O RPM normal Source22 fez **uma única geração real** no Ollama local com
`gemma3:4b`, já escolhido/instalado. O perfil privado configurou explicitamente
esse modelo em modo rápido; não alterou a seleção operacional Codex nem a
fábrica. Um relay local de observação manteve payload e resposta sem alteração,
recusou outros destinos/modelos e limitou a uma geração. O modelo anuncia
completion/vision, sem ferramentas nativas: o pedido não levou tools, e a UI
mostrou “Só conversa”. Sem inferência paga, download ou execução do código.

O serviço devolveu HTTP200,269 tokens de entrada e62 de saída. O texto SSE
observado corresponde à mensagem nativa guardada, retirando só um newline
terminal. A resposta contém `soma(a,b)` e `return a + b`, mas acrescentou prosa
apesar de se pedir apenas código. Não se certifica qualidade geral por este caso.

A primeira QA comparou o primeiro código observado durante o streaming com
o clipboard final e falhou; conserva-se esse resultado parcial. O replay do
SSE real, sem nova geração, mostrou primeiro apenas `function`, depois o bloco
completo. Esperar pela conclusão nativa e pelo texto final tornou a comparação
correta: código/cópia/reinício passaram (`recorded-stream-code-source22-1`).
É replay declarado, não uma segunda geração nem medição de latência real.

O perfil que recebeu a geração real foi reaberto diretamente, com o mesmo
endpoint e alvo exato. Mensagens, CID e código ficaram iguais; o código foi
copiado integralmente no clipboard privado (`real-local-chat-source22-resume3`).
O relay desta continuação só aceitou metadados e recusou geração: nenhum novo
pedido de inferência ocorreu. A captura desse perfil real foi inspecionada.
A continuação anterior esperava incorretamente `localExecution` num read
histórico, campo opcional/transitório; não se inventou a sua persistência nem
se restauraram snapshots para o ensaio passar.

**Limite de fluidez:** modelo não carregado antes do pedido, primeiros bytes
em42,883s e resposta concluída em44,798s no serviço local. Não é comparação
frio/quente nem prova de fluidez instalada. O cliente de voz rápido tem timeout
30s, pelo que voz com este modelo real precisa de ensaio próprio, sem assumir
que a prova sintética fecha essa dependência.

Recibo privado `REAL-LOCAL-CHAT-SOURCE22-VERIFIED.json`; artefactos originais,
resultado parcial e continuações preservados. Não houve delta de produção,
substituição instalada ou publicação nesta unidade. Gráfica Hyprland geral,
áudio físico, outros providers, ferramentas/raciocínio e restantes49 critérios
continuam com a sua aceitação própria.


### Voz com o modelo local real e observação Hyprland Source22

A fonte atual manteve as1322 entradas do manifesto, sem alteração de produção.
No RPM normal Source22, `real-local-voice-source22-3` passou o percurso completo
privado: opção nativa ligada, áudio negativo sem chamada, áudio positivo
sintético no microfone virtual, VAD/Whisper real, pedido `/api/chat` ao
Gemma3:4b instalado, resposta no pet Ollama, síntese Edge registada na saída
virtual, desligar pela UI e reinício com o mesmo CID/mensagens e sem replay.
A comparação independente confirmou os bytes da resposta e o sinal gravado
(pico0,9634). Recibo agregado: `REAL-LOCAL-VOICE-SOURCE22-VERIFIED.json`.

Foram duas gerações locais reais nesta unidade, zero chamadas pagas e zero
respostas de inferência sintéticas. A primeira tentativa não reconheceu o wake
na nova gravação e não fez pedido. A segunda recebeu a resposta fria em2,964s,
mas o QA interpretou ausência de stream como fim da fala e desligou durante a
preparação: o resultado falhado e o áudio sem sinal foram conservados. A terceira
esperou o início real da reprodução e a transição do listener após a síntese,
e passou; o pedido já quente levou0,576s. São observações diferentes, sem
benchmark controlado. O limite rápido30s foi mantido: o pedido de chat anterior
de44,8s não demonstrou um problema nesse percurso de voz. O modelo pediu que se
concretizasse a pergunta vaga; isto não certifica qualidade geral ou uma resposta
sobre o fenómeno físico. Hardware, Bluetooth e instalada permanecem por aceitar.

A sessão Hyprland estava desbloqueada e ligada. As capturas da região das layers
identificadas por PID/endereço ficaram encobertas; nenhuma foi aceite e nenhum
clique foi enviado ao diff sem observação prévia. O render WebDriver estava
completo, mas não substitui a captura do compositor. O arranque normal privado,
sem WebDriver/variáveis de automação, não apresentou a layer auxiliar
`coucou-pet-control`, ao contrário do arranque com automação. Selecionar apenas
a janela principal não resolveu a diferença; reaplicar as preferências já
inativas pela API existente também não resolveu a captura. Existem ainda
superfícies operacionais na mesma área, que foram preservadas. Não se conclui
causa única nem regressão gráfica do produto a partir deste ensaio.

As tentativas `layer-normal44-source22-1` a`4` conservam os resultados e a
observação negativa; a`plain1` documenta o arranque sem automação. Os runners
privados foram terminados por identidade e executaram a limpeza, sem tocar no
processo operacional. Não há workaround de produção, nova instalação ou aceitação
humana. O próximo passo gráfico é obter uma observação sem sobreposição e sem
confundir o comportamento de automação com o arranque normal, antes da entrega.


### Ligação Android comparada na composição Source22

O original0.5.0 (`phone-official-native-9`) e o RPM normal Source22
(`phone-normal44-source22-4`) passaram o mesmo percurso nativo, desde a UI até
ao TLS e ao socket de decisão. O protocolo recebeu a edição canónica do relay
original, incluindo o primeiro ID0 e os textos anterior/novo; o telefone
sintético recusou um pedido sem executar o comando apresentado. A retirada de
diffs desligou o cliente e a nova ligação deixou de receber essa capacidade.
Rotação e reinício da app preservaram o certificado; o token anterior foi
recusado e a opção desligada persistiu. Os QR e controlos foram observados.

O Secret Service foi real e privado, separado da rede isolada da app/cliente.
O namespace só teve loopback e um endereço RFC1918 sintético. O sandbox criou
antepassados de ficheiros com proprietário coerente; o primeiro namespace
simples tinha UIDs não mapeados e foi recusado pelo guard do fork, que foi
mantido. Não houve ligação à LAN, cofre operacional, provider real ou quota.
As falhas de preparação/seleção de nós obsoletos foram conservadas; não são
provas de regressão nem foram transformadas em resultados aprovados.

Não foi necessária uma correção de produção. O servidor, hub e emparelhamento
são byte a byte iguais aos do oficial; as diferenças de UI revistas tratam de
gravação/erros/cópia e da seleção de diffs pela sessão atual. Serviços e cópia
do link foram apenas revistos em código. O cliente Android tem a mesma árvore
Git; ficheiros ausentes do checkout parcial não foram classificados como perda.

Recibo: `PHONE-NATIVE-VERIFIED-COMPARISON.json`, com transcrições privadas,
identidades dos binários e limites. Android físico, descoberta Wi-Fi/biometria,
relay, chat, perguntas e cartões de serviços/consumo mantêm aceitação própria
pendente. A unidade seguinte pode reutilizar este isolamento para verificar o
chat do telefone com provider sintético, enquanto a prova gráfica Hyprland
continua pendente de observação sem sobreposição. Instalada43 intacta.

### Chat do telefone e correção HTTP —12/11, Source25

Comparação delimitada: `phone-chat-official-native-3` e
`phone-chat-status84-native-1`, com executáveis identificados nos respetivos
recibos. Cada caso completo fez sete POSTs para o provider sintético explicitamente
selecionado, através do backend real. Usou UI Tauri/WebKit, cofre Secret Service,
TLS com pin do QR e rede privados; não usou contas, perfis ou inferência reais.
Continuação e reset mantiveram o contexto esperado; cancelamento e desligar a
permissão fecharam o upstream antes da cauda tardia; retirar o modelo impediu novo
pedido. Escolha e switches desligados sobreviveram ao reinício, sem replay.

O defeito HTTP partilhado tem correção concreta em `local_chat.rs` e
`phone_link/chat_backend.rs`, sem novo formato ou autoridade. Três filtros Rust
passaram (cliente HTTP/status, mensagens de erro e classificador); a privada84
passou também o percurso completo com503 e401 enganador no corpo privado. O corpo
não apareceu nos pacotes enviados ao telefone. Contrato: `ANDROID_LINK.md`.

Recibo independente provisório:
`.build/upstream-parity-20261111/PHONE-CHAT-SOURCE25-PARTIAL.json`.
O RPM normal Source25 foi compilado, verificado no prefixo privado e passou
`phone-chat-normal44-source25-1` com prova própria. O agregado final está em
`PHONE-CHAT-SOURCE25-VERIFIED.json`; o recibo provisório permanece preservado. A imagem do modelo da84 foi inspecionada após libertar o gate na
mesma invocação de ferramenta: não é prova de inspeção visual anterior ao envio;
a escolha exata foi verificada independentemente pelo backend nativo.

**Problema aberto:** `phone-chat-normal44-source22-3` e
`phone-chat-refusal82-readers-2` recusaram o segundo pedido antes do HTTP; o último
incluía apenas consultas concorrentes de leitura. A origem nativa exata não foi
confirmada. A inspeção identificou que `Captured::check` transforma `Busy` em texto
de destino alterado, mas Source25 não muda essa validação e não se atribui essa
causa ao episódio sem prova. Os casos bem-sucedidos83/84 não fecham o problema.
Os resultados parciais/falhados mantêm-se preservados. Instalada43, dados,
preferências, modelos e restrições de publicação mantêm-se; os49 critérios globais
continuam abertos.

O novo RPM usou a receita Fedora existente, dependências offline e o relay
original fixado. Os13 ficheiros do payload, fontes arquivadas, transformação
única UNK→RPM, dependências ELF e ausência de RPATH de projeto/contentor foram
verificados pelo instalador existente, apenas num prefixo privado do projeto.
Compilação445,46s,1322 entradas imutáveis. O normal Source25 fez sete POSTs
sintéticos e passou o mesmo percurso nativo completo da84, incluindo503 com401
no corpo privado. As duas imagens foram inspecionadas antes de libertar os
gates deste caso normal. Nenhum sucesso fecha a recusa intermitente conhecida.

Verificação operacional final por leitura: mesmo processo43/PID2631834/startTicks12394108,
mesmos hashes de binário, preferências e fábrica. Recibo:
`INSTALLED43-AFTER-PHONE25.json`. Não houve instalação, publicação, quota paga ou
aceitação humana. Próxima entrega: corrigir a recusa intermitente com causa
provada e concluir a apresentação/entrega das funcionalidades úteis ainda abertas.

### Contenção de leitura e destino original —12/11, Source29

A regressão `inference-busy26-baseline` confirmou que o check síncrono transformava
`Busy` em erro de destino alterado. Source29 mantém os três estados e aguarda
apenas a validação de leitura antes de preparar/enviar o chat local interativo
simples, por até seis segundos por validação, sem mutexes retidos entre polls.
Stop/contexto, prazo e preferências são revistos antes de IO; não há retry de
pedido, nova concessão, fallback, ledger ou formato. O runtime de objetivos
conserva a admissão não bloqueante e a fence `FnOnce`.

As fixtures provaram HTTP zero enquanto o leitor segurava o mutex; depois,
exatamente um pedido ao destino/modelo capturados. Stop enviou zero e conservou
a exclusão física até libertar os donos; mudar o binding enviou zero a ambos os
endereços e retirou a captura, sem a reativar ao restaurar valores. Prazo
esgotado e fence assíncrona recusaram antes de HTTP/usage capture. As provas de
leitor/cliente de Source28 são reutilizadas apenas para os três ficheiros com
hashes idênticos em29; feedback/wire e integração nativa têm provas próprias29.

Privada85 e RPM normal29 passaram separadamente UI, escolha explícita, TLS,
sete POSTs sintéticos, consultas readonly concorrentes, continuação/reset,
mensagem `busy`, cancelamento/revogação, retirada de modelo e reinício sem
replay. Isto não força contenção do mutex pela GUI: essa condição foi provada
nas fixtures. Recibo independente: `INFERENCE-READER-SOURCE29-VERIFIED.json`.
Contrato: [PET-INFERENCE-BINDING.md](PET-INFERENCE-BINDING.md).

O episódio nativo intermitente22/82 permanece sem causa exata confirmada. Um
probe pré-correção25 completou sete respostas e640 consultas readonly, sem o
reproduzir; não se relabelam as falhas antigas nem se fecha o incidente por
sucessos posteriores. Novas categorias fixas permitem distinguir futuras
recusas sem divulgar erro de provider. A próxima unidade passa para a
apresentação atual de código/diff, mantendo o incidente no acompanhamento.

Observação Hyprland apenas de leitura: sessão disponível, uma saída ativa e a
região da ilha43 transparente naquela captura, com conteúdo de outras apps
visível. A presença da layer não prova oclusão. Não se alteraram widgets,
janelas operacionais ou preferências; a observação não é aceitação gráfica.
O processo43, binário, preferências e fábrica mantêm identidade/hashes no recibo
`INSTALLED43-AFTER-INFERENCE29.json`. Sem quota real/paga, publicação, instalação
ou aceitação humana. Os49 critérios integrais continuam ativos.

### Apresentação real de código/diff —12/11, normal Source29

Não foi necessário alterar código de produção. O mesmo RPM Source29 foi usado
em perfis gerados, com Tauri/WebKit/IPC reais sobre o compositor Hyprland da
estação. A âncora nativa posicionou apenas o cartão de QA. Uma superfície
auxiliar exposta pela automação foi recolhida pelo percurso existente
`pet_panel_open`→`pet_control_configure`, conservando as preferências privadas
inativas e todas as permissões; não se destruiu a janela nem se adicionou CSS,
capacidade ou workaround ao produto. O cartão e o diff ficaram integralmente
visíveis. Captura e PID/endereço confirmaram a superfície antes de cada clique.

O cartão Codex mostrou sessão externa sintética, ficheiro e +1/−1; o botão
abriu as linhas removida/adicionada, observadas na captura real. O controlo de
retorno foi observado antes do clique e a ausência do leitor confirmada pelo
DOM; não se alega uma captura posterior ao retorno. O primeiro ensaio
`anchor1` conserva aceitação só das linhas legíveis, com sobreposição parcial;
`reconcile2` conserva a prova sem essa sobreposição. `reconcile1` falhou apenas
num nome de comando errado do harness (`get_settings`), corrigido para `boot`;
o resultado falhado não foi relabelado.

A conversa flutuante Ollama usou `fixture-free` num endpoint HTTP sintético
explicitamente configurado. A UI e o pedido real conservaram dono/modelo/query;
as cinco linhas de código foram comparadas exatamente com o DOM e observadas
na janela nativa. Após fechar a conversa e reiniciar o processo no mesmo perfil,
o CID e as duas mensagens voltaram iguais e legíveis, com um único pedido HTTP.
A abertura normal sem target/session tem `conversationId:null` no contexto de
abertura, por contrato: consulta a conversa atual. O harness captura o CID
nativo antes do envio e compara-o depois; esta prova não é de abertura de uma
sessão explicitamente fixada. A cópia foi apenas observada como controlo;
o clipboard pessoal permaneceu intacto. Um pequeno sprite independente no
cabeçalho não tapava dono/modelo/código ou controlos relevantes.

Recibo independente: `GRAPHICAL-CODE-DIFF-SOURCE29-VERIFIED.json`, com hashes das
capturas privadas, perfil/recibos, pedido HTTP, persistência e qualificações.
Fonte1322 e executável/RPM mantiveram hashes. A instalada43 conservou o mesmo
processo/binário/preferências/fábrica; quatro runtimes temporários foram limpos e
não ficaram janelas/layers/drivers/buses dos ensaios. Recibos
`INSTALLED43-AFTER-GRAPHICAL29.json` e `GRAPHICAL29-CLEANUP.json`.

Esta é prova atual das duas componentes úteis, não paridade gráfica integral,
aceitação humana/instalada, hardware ou todos os49 critérios. O incidente
intermitente pré-HTTP continua separado. Próximo: atualizar a aceitação técnica
integrada6/7/preservação para este executável e só depois propor uma substituição
instalada com âmbito concreto; os restantes percursos úteis e critérios mantêm
as suas dependências e restrições.


### Revisão/sucessão6/7 e preservação atuais —12/11, Source29

O executável normal atual, sem novo delta de produção, completou os percursos
nativos de revisão6/7, saldo herdado, uma nova alocação finita explícita,
interrupção/recuperação e Stop/reinício. Três respostas HTTP sintéticas ao todo.
O ensaio inicial parou com `Busy` depois de gravar a reserva e o grant filho,
antes de enfileirar. A continuação conservou os mesmos IDs, reserva, aprovação
original e prazo e produziu apenas a resposta pendente. A causa exata da
contenção não foi isolada; isto não encerra o incidente pré-HTTP do telefone.
O guard de uma preparação de QA esperava incorretamente reserva vazia e parou
sem IO; esse resultado continua falhado, com a correção do harness documentada.

Treze comandos CLI atuais preservaram oito registos assinados, três objetivos67
e formatos1/6/6/7/7. O restauro inativo foi byte a byte exato; revisão/configuração
do novo perfil conservaram a autoridade histórica em quarentena, sem renovar
execução. A app atual abriu esse restauro, mostrou a resposta e recusou admitir
ocorrência, sem criar objetivo ou HTTP;12 payloads originais continuam iguais.
O executável antigo57 recusou a ativação sem alterar payloads.

Recibo `RULE67-SOURCE29-VERIFIED.json`, com hashes, fontes e falhas conservadas.
Perfis/decisões inteiramente sintéticos, escolha automatizada de QA, não aprovação
humana. Instalada43 intacta; QA próprio fechado e runtimes temporários removidos.
Não é aceitação instalada/hardware/global49. A dependência técnica atual6/7 está
fechada neste âmbito; continuar a apresentação de decisões/resumo semanal e os
restantes percursos úteis, sem permanecer a aperfeiçoar esta infraestrutura.


### Decisões e resumo/partilha no compositor —12/11, Source29

O RPM atual mostrou o cartão Codex com comando e botões completos. Após observar
a captura real, a recusa nativa chegou ao socket original. A pergunta seguinte
mostrou texto/opções e devolveu exatamente QA_B; reinício sem replay. Nenhum
comando, modelo ou clipboard pessoal foi usado; escolhas apenas de QA sintético.

Resumo e partilha também ficaram completos. Eventos de atividade no socket
registaram a ocorrência corrente; uma semana anterior explicitamente semeada
alimentou o resumo. Exportação nativa privada gerou PNG1080×1920, com e sem
projeto. Ocultação/histórico e PNG sem projeto mantiveram-se após reinício real.
O PNG atual Wayland difere do oficial anterior X11, incluindo rasterização/peso
de texto; ambos foram inspecionados, com mesmos dados/layout/privacidade neste
âmbito. A igualdade anterior no mesmo X11 conserva apenas essa prova.

Falhas de preparação continuam falhadas: exigência de movimento da âncora,
assunção de contexto auxiliar e recusa WebDriver element-click antes de efeito.
O percurso concluído usou ações nativas de ponteiro após conferir geometria e
hit-test do alvo observado. Não se alterou o produto/permissões para facilitar
QA, nem se atribuiu uma causa diferencial à recusa de automação.

Recibo `COMPOSITOR-DECISIONS-RECAP-SOURCE29-VERIFIED.json`, com imagens privadas,
fontes, resultados e limites. Instaladas43/processo/preferências/fábrica intactos;
QA próprio encerrado e seis runtimes temporários removidos. Sem aceitação
humana/instalada/global49 ou do clipboard de imagens/disparo de segunda-feira.
A [entrega instalada](PETS-INSTALLED-DELIVERY-SOURCE29.md) está concretamente
preparada e aguarda âmbito autorizado; as restantes lacunas da matriz e a falha
intermitente pré-HTTP mantêm-se, sem nova infraestrutura alheia à recuperação.


### Recursos atuais e nova regressão de cartões —12/11

O normal Source29 foi comparado novamente com o oficial pinned, na ordem
oficial→atual→atual→oficial, em perfis novos/X11 privado/GL por software,
Hub de ditado aberto, pausa confirmada e chat não aberto. Os quatro conjuntos
de processos permaneceram estáveis, sem PSS indisponível ou inferência. Médias
finais: oficial362,018MiB, atual411,476MiB; diferença49,458MiB e mais um processo
WebKit. CPU206,235% contra105,939% de um núcleo, neste cenário. É custo observado,
não prova isolada de regressão nem motivo para redesenhar a framework. Recibo
`RESOURCES-SOURCE29-VERIFIED.json`. A medição de arranque normal no Hyprland,
sem automação, foi recusada pelo guard de sessão bloqueada/ecrã desligado antes
de arrancar qualquer app; não é uma espera nem uma medição de repouso concluída.
Instalada43/processo/preferências/fábrica mantidos; processos medidos encerrados.

Foi encontrada uma regressão funcional de maior prioridade: o resumo de pets
só reutiliza o cartão especializado Spotify. A receção nativa n8n está saudável,
mas os cartões/Refresh/detalhes do serviço não são apresentados nesse percurso.
O oficial passou cofre privado→Refresh UI→GET local sintético→cartão→detalhe,
retorno e erro (`service-cards-official2`). O normal29 recebeu os mesmos dados e
não mostrou cartão/Refresh (`service-cards-source29-before2`). A presença de um
nó Spotify escondido não é um cartão n8n: o primeiro guard QA foi corrigido para
contar só elementos visíveis; o seu resultado falhado fica preservado.

A correção Source30 está implementada e passa testes dirigidos, ainda sem
aceitação nativa enquanto compila: reutiliza os renderers de serviço existentes
no cartão do pet, mantém identidade/chat/contexto, exclui GitHub retirado e não
inicia rede ou inferência ao renderizar. A notificação n8n recebida é conservada
apenas no estado UI existente, com a mesma expiração, para não tratar texto do
assistente como resultado de workflow. Não cria wire/persistência/autoridade.
A proposta instalada Source29 continua identificada pelos seus bytes e não é
substituída implicitamente por esta nova composição; instalação não autorizada.


### Cartões de serviço recuperados —12/11, Source30

Regressão confirmada com executáveis reais: o oficial0.5.0 mostra o cartão n8n,
Refresh, resultado e detalhe; o normal Source29 recebe os mesmos dados pelo
poller nativo mas não oferece cartão/Refresh no resumo do pet. A correção
reutiliza os seis renderers existentes nesse resumo: n8n, Vercel, Notion,
Resend, Cal.com e Stripe. Identidade, chat e configuração permanecem acessíveis;
não ativa pets no perfil operacional. GitHub continua temporariamente retirado.

Normal Source30 compilado e verificado no prefixo privado com o instalador
existente:13 payloads, relay/Copilot, ELF e arquivo de fontes. O percurso n8n
passou duas vezes com Secret Service real e API loopback sintética: guardar
credenciais, reiniciar, clicar Refresh, seis GETs autenticados, sucesso/detalhe/
retorno/erro. O mesmo perfil reiniciou sem repetir resultado; retirar as chaves
interrompeu leituras e mostrou falta de configuração após novo reinício.
Nenhuma mutação de workflow, inferência ou conta real. Os outros cinco serviços
passaram projeção de fixtures no renderer nativo, incluindo detalhe/retorno
Vercel; isso não prova as suas APIs remotas. Typecheck e testes dirigidos passaram.

**Limite gráfico explícito:** foram inspecionadas capturas legíveis de Vercel
lista/detalhe, erro/falta de chave n8n e Stripe. O detalhe de sucesso n8n continua
recortado e uma captura Notion ficou preta no X11 privado, apesar de DOM,
geometria e controlos funcionais. Foco e espera de desenho corrigiram um frame
Vercel desatualizado, sem resolver todos os cortes. Não há aceitação gráfica
integral nem novo workaround no produto. Falta observação no compositor real
quando a sessão estiver desbloqueada, além de APIs/instalada/hardware aplicáveis.

Recibo `SERVICE-CARDS-SOURCE30-VERIFIED.json`, fontes1323 e delta de quatro
ficheiros face a29; sem alteração de backend, autoridade, wire ou formato
persistente. As provas29 de6/7/código/decisões continuam com fonte/âmbito próprios.
Instalada43, processo, binário, preferências e fábrica verificados intactos;
processos e runtimes privados destes ensaios encerrados. A proposta instalada29
é um artefacto diferente e não autoriza substituir implicitamente por30.
Objetivo integral/49 critérios ativos, sem commits/pushes/merges.


### Servidor personalizado recuperado por pet —12/11, Source33

O oficial0.5.0 completou ligação/chave/modelo e resposta SSE sintética. O
normal30 guardava a ligação mas não oferecia o motor na conversa do pet.
A correção reutiliza o transporte OpenAI-compatible e o binding/CAS/editor
existentes, nos dois pets que já admitem escolha de motor. Não cria pets,
ledger, formato estrutural ou concessões. Mantém dono, identidades, permissões
e contas Codex/Copilot/Gravity; Guardian conserva o contrato anterior.

O normal atual passou pela UI: ligar com chave sintética no Secret Service,
“Configurar num pet”, revisão/confirmar motor/endereço/modelo/referência de
credencial, resposta autenticada com código e reinício no mesmo perfil/CID.
O código/resposta e o erro após retirar a chave foram inspecionados legíveis
na conversa flutuante X11. Retirar a chave recusou novo envio, sem HTTP anónimo,
outro motor/modelo ou replay. Os outros pets mantiveram configuração/permissões;
a biblioteca Codex tinha apenas metadados vazios, sem mensagens ou chamadas.
A escolha admite destino remoto explícito; os testes nativos usaram só loopback,
sem provar TLS/contas remotas ou qualidade de modelos reais.

Treze recibos CLI atuais completaram backup/restauro inativo byte a byte,
preparação independente de autoridade de perfil sem grants, revisão de
configuração e ativação com zero grants. O leitor30 permite a revisão opaca de
backup, sem interpretar o motor, mas recusou ativação sem alterar payloads.
Reinício da app no perfil original e preservação CLI são provas distintas;
não se declara ensaiada a janela do perfil restaurado.

Os cartões de serviço também passaram novamente neste executável: seis GETs
n8n autenticados, detalhe/retorno/erro/reinício/retirada da chave, mais cinco
projeções de fixtures. APIs remotas desses cinco não certificadas. Sucesso n8n
continua recortado no X11 privado, embora erro/detalhe sejam legíveis; falta
observação no compositor real desbloqueado. Nenhum workaround de pintura.

**Composição auditável:** a compilação normal pertence a Source32. O wrapper
falhou a igualdade integral porque foi editado apenas `docs/PET-FACTORY.md`
durante a build; resultado original conservado como falso. Todos os inputs de
código/runtime/build/recursos são iguais. Source33 alinha o arquivo completo de
fontes com esse documento, verifica novamente13 payloads/ELF/relay/Copilot e
usa o mesmo RPM, sem alegar recompilação. A prova nativa acima é própria do
executável atual. O protótipo31 ficou sem aceitação depois de uma revisão
identificar ordem de captura de credencial LM Studio; corrigida antes desta
compilação, com testes de metadados próprios passados.

Falhas de QA conservadas: campo abaixo do viewport, expectativa incorreta de
biblioteca sem metadados, recusa indevidamente esperada no backup opaco e ramo
de autoridade histórico aplicado a perfil sem grants. A continuação CLI usou
os mesmos recibos/pair e não repetiu efeitos concluídos. Uma expectativa antiga
de teste Sistema foi corrigida segundo bloco de produção comprovadamente igual
a30 e contrato de arranque finito assinado; não se alterou essa política.

Recibo `CUSTOM-SOURCE33-VERIFIED.json`. Instalada43/processo/binário/preferências/
fábrica intactos, QA próprio encerrado e runtimes temporários próprios limpos.
Sem inferência real/paga ou publicação. Compatibilidade HTTP não prova voz,
ferramentas ou tarefas autónomas deste motor; o catálogo mantém essas barreiras.
Provas29 de6/7/código/diff/decisões/resumo e provas22 de voz conservam as suas
composições e âmbitos. Sem aceitação humana/instalada/global49. A autorização
pendente para o artefacto29 não autoriza33. O objetivo integral permanece ativo.


### Compositor33 — capturas recusadas e âmbito preservado

A sessão voltou a estar desbloqueada, mas as capturas do cartão estavam tapadas
pela galeria operacional43. A atribuição inicial à app de QA foi corrigida:
o inventário exato mostrou `coucou-pet-control` do PID2631834, enquanto o
auxiliar do PID190540 de QA já estava oculto. O WebKit próprio renderizava o
cartão IDLE completo; isso não prova visibilidade real do sucesso ou cliques
através de outra layer. Nenhuma das capturas tapadas foi aceite.

Há apenas o monitor eDP-1 ativo,1440×900 lógicos, sem região/monitor alternativo
selecionado pelo guard. A tentativa de deslocação foi recusada antes de arrancar
qualquer app. Não se ocultou a galeria operacional, alterou o monitor/política
global ou inventou aceitação gráfica. Os três ensaios terminaram, os seus
runtimes próprios foram limpos e não ficaram layers/janelas/PIDs próprios.
Processo/binário/preferências/fábrica43 conservados.

`COMPOSITOR-SERVICES33-QUALIFICATION.json` conserva imagens, falhas, correção da
atribuição e integridade. A visibilidade atual dos serviços fica pendente até
existir área sem sobreposição; não repetir tentativas equivalentes nessa
condição. A próxima dependência independente é a aceitação técnica6/7 da
candidata33 e a preparação da entrega atual com âmbito explícito.


### Revisão/sucessão6/7 e preservação na candidata atual —12/11, Source33

`RULE67-SOURCE33-VERIFIED.json` comprova agora o executável normal atual, sem
delta de produção: revisão nativa própria, saldo herdado sem aumentar limites,
renovação finita explícita, três respostas HTTP sintéticas e recuperação após
interrupção real do backend. A sondagem gasta, decisão, reserva e prazo originais
foram conservados; não houve repetição daquele IO. O Hub fechado manteve o
backend e a resposta automática ocorreu com a alocação finita revista.

O primeiro ensaio terminou com `ruleAuthority:Busy` na revogação final, depois
das três respostas. O resultado falhado foi preservado. A continuação no mesmo
perfil/endpoint/composição revogou a decisão exata, cancelou uma revisão sem
confirmá-la e reiniciou: zero novo HTTP e objetivos/consumos originais iguais.
Não foi identificado o writer exato da contenção e isto não encerra o episódio
pré-HTTP do telefone. A revogação final foi exercida por IPC nativo; não se
inventa aqui um clique humano no botão ou consentimento pessoal.

Treze comandos CLI preservaram oito registos assinados, três objetivos67 e os
formatos1/6/6/7/7. Restauro inativo exato, autoridade histórica em quarentena,
ativação sem grants; a build57 recusou ativação sem mudar payloads. O próprio
normal33 abriu o restauro, mostrou resposta histórica e recusou execução:
zero objetivo novo/inferência e12 payloads originais iguais. O registo nativo
de recuperação foi conservado e comparado. Não foi repetida a segunda época
de restauro, cuja prova anterior conserva o seu âmbito.

Capturas de revisão6/carry7/renovação/recuperação e resposta restaurada foram
inspecionadas tecnicamente. Perfis e escolhas são inteiramente de QA; nenhuma
aprovação humana, chamada real/paga, instalação ou alteração do anfitrião.
Instalada43 com o mesmo processo/binário/settings/fábrica; QA próprio fechado.
Não é paridade integral ou aceitação dos49 critérios. Prosseguir as componentes
úteis e a entrega autorizada, sem voltar a aperfeiçoar esta infraestrutura.


### Cartões de serviço no compositor atual —12/11, Source33

`COMPOSITOR-SERVICES33-VERIFIED.json`:12 etapas nativas no Hyprland real,
com perfil/HOME/cofre privados, mesma candidata normal33 e zero delta de
produção. A sessão mudou independentemente para DP-5 ultrawide3440×1440.
O inventário identificou área lateral livre: só o anchor da própria superfície
QA foi reposicionado pelo comando existente. A galeria/app43, monitores e
políticas globais não foram alterados. As tentativas sobrepostas anteriores
continuam conservadas e não foram promovidas a sucessos.

n8n:22 GET locais autenticados no poller real, botão Refresh, resultado de
sucesso/detalhe/Back, erro/detalhe, reinício sem repetir resultado e retirada
de chave com estado explícito e zero novas leituras. Sem workflow, POST de
serviço, inferência ou quota. Notion/Vercel/Resend/Cal.com/Stripe: projeções de
eventos nativos sintéticos; deployment/detalhe/Back Vercel também observados.
Não são provas HTTP ou contas remotas desses cinco serviços.

Cada captura da região real foi inspecionada antes do próximo controlo, com
PID/camada e ausência de interseção de outra camada registados. As superfícies
ficaram completas; texto secundário do Resend permaneceu pouco contrastado.
Aceita-se este percurso funcional/apresentação, não qualidade visual geral,
semântica de todas as capacidades do pet, aceitação humana ou instalada.
Processos/drivers/cofre/camadas QA encerrados, runtime próprio removido e
instalada43 com processo/binário/settings/fábrica/launcher iguais. Capturas são
privadas, não versionadas. Próximo: entrega atual no âmbito autorizado e
restantes componentes úteis; não é necessário novo workaround de pintura.


### Inventário de formatos de diffs — início da próxima unidade

A leitura comparada33/oficial0.5.0 confirmou igualdade byte a byte de
`core/diff.ts`, `views/diff.ts`, `diff.test.mjs`, `livediff.test.mjs` e
`agent_hooks.rs`. O encaminhamento `island/hooks.ts` difere pela integração
de sessões/pets. Ambos os parsers estruturam Edit/MultiEdit/Write; nomes de
ferramentas adicionais não constituem automaticamente perda do fork.
`HOOK-DIFF-FORMAT-SOURCE-AUDIT33.json` é apenas auditoria de fontes, não nova
prova nativa. Próximo: ampliar o harness existente com Write/MultiEdit via
relay oficial real, campos sintéticos e vistas por pet, sem executar IDE ou
consumir quota. A decisão instalada33 permanece pendente de resposta própria.


### Write/MultiEdit e vistas por pet —12/11, Source33

`HOOK-FORMATS-SOURCE33-VERIFIED.json`: seis comparações nativas com o oficial
pinned, usando o seu relay real, socket e campos normalizados Codex/Copilot/
Gravity. O mesmo ID de sessão foi usado nos três pets; linhas, nomes de ficheiro
e contagens coincidiram integralmente.33 passou Ctrl+E/Escape, centro e regresso
às sessões após Stop, incluindo Codex além dos5,2s de limpeza do original.
Os nomes Write/MultiEdit foram payloads sintéticos no contrato canónico: não
prova que cada IDE real emita esses formatos nem executou escrita/ferramenta.
Todos os caminhos de ficheiros desses eventos permaneceram ausentes; zero HTTP
ou inferência. Parser/vista/testes de diffs continuam iguais aos do original.

O ensaio oficial terminou falhado no regresso tardio, depois dos seis formatos
válidos. A UI mostrou Codex idle; o Stop original limpa a tarefa/diffs após5,2s.
A expectativa de retenção adicional pertencia ao harness da arquitetura dos
pets. Resultado/erro/DOM/código fonte foram conservados; não se converteu o
resultado global original em verde nem se inventou uma regressão do fork.

Cinco capturas X11 atuais mostram código legível; Codex MultiEdit ficou
recortado. Um único percurso separado no compositor real, pelo controlo
existente Ver última alteração, mostrou as quatro linhas/+2/−2 e Back completos.
A imagem X11 continua sem aceitação; não é equivalência global de backends.
Nenhum workaround de pintura, novo ledger, delta de produção ou recompilação.
QA próprio/containers/camadas encerrados e runtime falhado próprio removido;
instalada43 com o mesmo processo/binário/settings/fábrica/launcher.

Próximo inventário: as cinco operações originais de voz continuam declaradas
no broker comum (`pills`, `music`, `service_info`, `open_app`, `prepare_email`).
Isto ainda é evidência de fontes, não aceitação nativa desses efeitos. A voz do
original chama Claude/Haiku e tenta o modelo de chat após falhar; não usa o
servidor custom configurado. Portanto, a ausência atual de voz custom não é,
por si, perda comprovada do original. O roteamento atual continua explícito;
o fallback observado no código original não satisfaz a decisão do proprietário.
Comparar os percursos governados existentes antes de acrescentar capacidade.


### Operações originais por voz no broker —12/11, Source33

`VOICE-OPERATIONS-SOURCE33-VERIFIED.json`: dois percursos nativos atuais com
HOME/XDG/X11/microfone/saída exclusivamente privados. Whisper real reconheceu
áudio sintético, Ollama local sintético propôs a operação, o broker devolveu
recibo e a segunda ronda gerou a resposta falada por Edge na saída virtual.
`services.snapshot` devolveu leitura; `desktop.launch` com permissão desligada
foi recusado com permissionDenied/notDispatched. Os quatro grants de efeitos
do pet estavam explicitamente falsos. Nenhuma aplicação, email ou efeito de
sistema real foi executado; zero modelos/quota reais ou pagos. ASR local e TTS
Edge gratuito são workers reais, não mocks da captura ou injeção de transcript.

Consulta e continuação da recusa passaram Stop/reinício com CID/mensagens
conservados e sem novo pedido. O primeiro ensaio da recusa confirmou o broker,
mas terminou com áudio zero depois de esperar uma ligação genérica e4s. Esse
resultado permanece falhado e a causa exata da ausência de playback não foi
atribuída ao produto. A continuação reutilizou perfil/endpoint e preservou os
dois enunciados existentes; um novo turno próprio de QA fez apenas duas rondas
sintéticas, aguardou o callback nativo de início e amostras reais antes de Stop.
São seis rondas sintéticas no total. Não se reciclaram consentimentos/grants
nem se fez reset do histórico. O áudio inicial continua sem aceitação.

O primeiro recibo da consulta herdou metadados hostSystemReads:false e
buildRecord:null do helper antigo; qualificação explícita conserva o recibo e
corrige a interpretação: foram lidos quatro campos não secretos ASR/TTS e
inventário de processos, com binário/SHA/argumento/fonte33 verificados. Não se
repetiu o percurso só para corrigir esses campos. A montagem do verificador
foi ajustada a humanApproval:false, presente também nas falhas; não transforma
flag ausente humanAcceptance em aprovação humana.

As cinco operações originais e a declaração web_search de Claude continuam
no catálogo; a presença é evidência de fontes. Este incremento não certifica
ativar/remover pets, música, abrir app permitida, preparar email, pesquisa web,
autoridade granular por subplugin/operação/recurso, modelos/providers reais,
voz física/Bluetooth, qualidade/fluidez geral ou instalada. O snapshot partilhado
existente não é por esta prova certificado como filtragem granular de recursos.

QA/servidores de áudio/drivers/containers encerrados; runtime falhado próprio
removido, perfis/áudios/recibos privados preservados e instalada43 com o mesmo
processo/binário/settings/fábrica/launcher. Zero delta de produção ou instalação.
Próximo: efeitos restantes apenas em fixtures privadas, reutilizando broker,
launcher/mailto e MPRIS existentes; entrega33 continua dependente do âmbito
explícito já pedido. Todos os49 critérios mantêm aceitação própria em aberto.


### Qualificação de isolamento HOME — operações de voz33

O helper herdado isolava XDG/áudio mas mantinha HOME, permitindo leituras de
metadados locais fora do âmbito declarado; a aplicação privada de QA não foi
encontrada. As provas anteriores de operações33 não provam isolamento completo
e foram retiradas do mapa vigente até casos corrigidos com atestado do
ambiente de cada processo. Recibos/áudios são privados e conservados, sem os
reescrever como sucessos isolados. `VOICE-QA-HOME-ISOLATION-QUALIFICATION33.json`
lista os runners retidos afetados e os limites; zero inferência real/paga no
provider loopback destes casos e sem trace de lançamento/mailto no app1.
Não se afirma integridade universal de HOME; processo/binário/settings/fábrica
operacionais43 foram comparados e mantêm hashes. Source22 real-local usa outro
helper com HOME explícito; as suas provas não são revogadas por associação.

A correção é só no QA: HOME explícito, roots pessoais de CLI retiradas e
atestado em /proc do processo nativo próprio antes de usar o perfil. Novos
casos partem de perfis limpos de QA, sem importar o histórico não isolado.


### Voz: cinco operações em fixtures com HOME atestado — Source33

`VOICE-EFFECTS-SOURCE33-VERIFIED.json` substitui a aceitação atual das operações
de voz anteriormente ligada ao helper que não isolava HOME.15 runners antigos
permanecem privados e conservados; o seu comportamento pode ter prova própria,
mas não certifica o isolamento de HOME declarado. O erro foi identificado pela
não descoberta da aplicação privada de QA. XDG/áudio estavam isolados; leituras
de metadados em HOME não foram excluídas. A instalação43 foi comparada nos seus
processo/binário/settings/fábrica/launcher; não se inventa integridade universal
de HOME ou da configuração de todas as ferramentas. Dados antigos não foram
importados para os novos testes.

Agora há12 atestados de ambiente nos processos nativos próprios, antes da
utilização e após reinício: HOME/XDG/XAUTHORITY/DISPLAY/áudio/D-Bus privados,
roots de credenciais pessoais de CLI retiradas. Referências ASR/TTS com tilde
foram resolvidas para os mesmos ficheiros já escolhidos; cache de modelos em
modo offline, sem novos downloads. Pesos/runtimes existentes do projeto foram
partilhados por referência explícita. As duas respostas de cada turno vieram
do provider Ollama sintético local; ASR Whisper e Edge gratuitos foram reais,
com gravação/reprodução exclusivamente virtual e sem transcript injetado.

Seis casos atuais: services.snapshot/readOnly sem sessões/planos/serviços
pessoais; pets.roster/list sem mudar configuração; desktop.launch permitido
com ficheiro desktop privado verificado, lançador real e janela GTK própria
observada; mail.prepare com URI/assunto Unicode/corpo multilinha exatos num
capturador local sem abrir correio ou enviar; spotify.control/Play num D-Bus
privado, com uma chamada e Playing observados; desktop.launch recusado por
permissionDenied antes do despacho. Só o grant existente necessário foi ligado
no pet de QA, outros efeitos desligados. Os recibos mantêm effectVerified:false;
a observação da janela/leitor é evidência independente da fixture, não uma
certificação dos efeitos no mundo real.

Conversas/CIDs e efeitos conservaram-se no reinício; sem replay.12 rondas
sintéticas nos casos aceites. Falhas anteriores desta unidade conservadas:
app1 com HOME herdado não despachou; snapshot-home2 com HOME correto recusou
iniciar por caminhos de runtime ainda com tilde, antes de inferência; music-home1
fez um Play no leitor privado, mas o provider de QA exigiu JSON num ACK que pode
ser textual. O novo music-home2 usou outro bus/leitor e não voltou ao alvo antigo.
Não se transformaram esses resultados globais em sucessos nem se atribuiu uma
regressão ao produto por uma asserção errada do harness.

QA/fixtures/servidores/containers encerrados e só runtimes temporários próprios
removidos; perfis/áudios/recibos privados preservados. Fonte33/pacote/instalada43
intactos, sem commit/publicação. O catálogo e estes caminhos comuns preservam
valor original dentro da arquitetura dos pets; não certificam todos os comandos
por voz, mutações do roster, granularidade subplugin/operação/recurso, modelos/
providers/contas reais, pesquisa web, serviços reais, hardware/Bluetooth ou uso
instalado. Outros49 critérios e a entrega instalada conservam gates próprios.

### Recuperação da alteração do pet principal por voz — Source35,13/11

Na33, o ensaio nativo mudou Antigravity→Codex mas retirou a própria origem de
voz antes da segunda resposta, do histórico e do TTS. Esse resultado falhado
continua conservado (`VOICE-ROSTER33-REGRESSION.json`). A35 reutiliza o CAS
nativo do roster: catálogo só resolve nomes; só o commit confirmado do próprio
turno pode avançar a sua testemunha, se o pet que responde continuar ativo.
Escritas gerais/externas, dono substituído, cancelamento anterior ao despacho,
falha de persistência e retirada do próprio pet não renovam a captura.

Na candidata normal35, entrada no menu nativo do Ollama→microfone virtual→
Whisper real→proposta Ollama sintética→resolução do catálogo/CAS→recibo aceite→
resposta→Edge real/saída virtual→Stop→reinício passou. O TTS foi observado no
início e gravado durante dois segundos antes de Stop; a conclusão natural da
frase completa não foi certificada. Só mainPill mudou;
Ollama continuou ativo, configuração completa dos pets/modelo/provider/CID
conservaram-se. Mesmo CID antes do turno, depois e após reiniciar; sem replay.
Uma segunda sessão privada confirmou permissionDenied antes de desktop.launch,
com capturadores locais que impediriam qualquer abertura pessoal inesperada.
Quatro atestados de ambiente dos processos próprios e quatro rondas de modelo
sintético. Zero chamadas a modelos de conversa reais/pagos, áudio físico ou
aceitação humana. A proposta
sintética da ação não mede compreensão de comandos por um modelo real.

`VOICE-ROSTER-SOURCE35-VERIFIED.json` fixa os recibos/áudio/imagem/limites. Quatro
famílias dirigidas passaram. Dois testes antigos usavam custom como identidade
de pet e foram recusados pela autorização de fontes: o adaptador não é um pet.
A fixture passou a usar um dono declarado no mesmo endpoint/modelo sintético,
sem alterar a política de produção. Falhas de compilação/QA anteriores foram
retidas; o protótipo34 não foi compilado como candidata. Build35 nova e exata,
1323 inputs; sem reutilização documental do pacote32/33. Instalada43 conserva
processo/binário/settings/fábrica/launcher. Sem instalação, alteração do anfitrião,
commit ou publicação. As provas anteriores mantêm fonte/âmbito próprios.

Outras variantes de roster, entrada/fluidez instalada, hardware/Bluetooth,
providers/serviços reais e os49 critérios globais continuam abertos. Próximo:
gate6/7/preservação na mesma35 e proposta instalada concreta; manter a fonte
congelada e evitar novas abstrações ou recompilações sem defeito confirmado.


### Revisão/preservação6/7 na candidata35 —13/11

`RULE67-SOURCE35-VERIFIED.json`: o cenário nativo atual passou na primeira
execução, com revisão/sucessão6, carry do saldo original, renovação explícita
de uma alocação finita, três respostas HTTP sintéticas, Hub fechado, interrupção
real do backend/recuperação da sondagem com decisão/saldo/prazo originais,
revogação e reinício sem replay. Não foram criadas novas concessões para
contornar falhas. A contenção antiga33 conserva a qualificação própria.

Treze comandos CLI atuais de revisão/backup/inspeção/restauro/quarentena/ativação
preservaram8 registos assinados,3 Goal67, formatos1/6/6/7/7 e12 payloads originais
exatos. A janela nativa restaurada mostrou a resposta histórica e recusou
execução: zero HTTP/novo objetivo/renovação de autoridade. Capturas atuais foram
inspecionadas; painéis e limites legíveis, parte inferior do formulário fora
do viewport, sem certificação de toda a apresentação. HOME/XDG/D-Bus privados
atestados nos processos; QA encerrado e instalada43 intacta.

É aceitação técnica de perfis gerados; escolhas de QA não são consentimento
humano. Não fecha instalada, hardware, contas/efeitos reais nem49 critérios.
A fonte35/pacote mantêm-se congelados, sem nova compilação ou código nesta
unidade. Próximo: conclusão natural da fala e proposta concreta instalada35;
não repetir o percurso6/7/preservação sem novo delta/defeito que o justifique.


### Frase falada completa e reabertura sem replay — Source35,13/11

`VOICE-NATURAL-SOURCE35-VERIFIED.json`: a gravação privada contém a frase
completa “Resposta sintética de voz confirmada.”, reconhecida independentemente
com os mesmos pesos Whisper locais, sem prompt, vocabulário ou língua forçados.
O grafo conservado mostra que o Node de reprodução GStreamer terminou e surgiu
um novo microfone antes de Stop/terminação pela QA. A reabertura só de leitura
do mesmo perfil manteve o CID anterior ao turno, duas mensagens, settings e
configuração inteira dos pets; zero nova ação/modelo. Sem terceiro pedido de voz.

Os dois ensaios globais originais permanecem falhados. O primeiro observador
comparava coucou com o nome real Coucou. O segundo comparava só o número41;
o Node41/serial203 já terminara e um Link41/serial257 reutilizara o ID. O teste
já ultrapassara a espera do estado listening antes dessa falha; não foi guardada
uma captura DOM após a fala. A aceitação técnica deriva dos grafos/áudio
conservados e da continuação nativa de leitura, não de reescrever RESULT como
sucesso. O observador futuro usa tipo/id/serial e o dono gst-launch-1.0; só
preflight, sem repetir a mutação. A captura da resposta visível foi inspecionada.

ASR de entrada/TTS foram reais, áudio exclusivamente virtual, modelo de conversa
sintético; reconhecimento da gravação em CPU, com o mesmo modelo escolhido,
sem mudar preferências. Stop explícito depois do fim natural não foi repetido;
a prova atual separada de interrupção/Stop/reinício conserva o seu âmbito.
Sem aceitação física, timbre/latência comparados com o oficial, contas reais,
humana, instalada ou global49. QA encerrado, dados/gravações conservados,
instalada43 intacta; nenhuma alteração de código/compilação/host/publicação.

[Proposta instalada35](PETS-INSTALLED-DELIVERY-SOURCE35.md): artefacto exato,
backup específico, quatro fontes/patch da estação e13 payloads verificados
pelo instalador existente em prefixo só do projeto. Aplicação e exceção de
registo assinado em sysworkspace aguardam âmbito explícito do proprietário.

<!-- code-diff-source35-20261113 -->
### Código/diffs na composição atual — Source35,13/11

`HOOK-FORMATS-SOURCE35-VERIFIED.json`: seis casos sintéticos Write/MultiEdit
atravessaram o relay oficial real, socket e UI nativos de Codex/Copilot/Gravity.
Linhas/contagens iguais às seis observações nativas do oficial0.5.0/841a81ce;
Ctrl+E/Escape, centro, seleção por pet/sessão e regresso ao Codex depois dos5,2s
passaram na35. O ensaio oficial completo continua falhado no regresso tardio:
essa retenção não fazia parte do comportamento original.

Cinco imagens privadas X11 são legíveis; Codex MultiEdit X11 recortado continua
sem aceitação. Uma captura separada da camada35 no Hyprland real mostrou o
ficheiro e as quatro linhas/+2/−2, sem outra camada de nível igual ou superior
sobreposta. Esta imagem foi observada e aceite tecnicamente. O ensaio gráfico
completo ficou **falhado** depois: o controlo Back já não existia após cerca de22s
de espera pela observação. O código fecha cartões e limpa a seleção do diff ao
sair do modo expandido, mas o instante/causa exatos não foram instrumentados;
não se declara regressão nova nem sucesso integral desse ensaio. O regresso
funcional tem a prova separada dos seis casos nativos, sem promoção a Back no
compositor. As tentativas sem área livre/prazo de observação também ficam
preservadas; `CODE-DIFF35-COMPOSITOR{1,2,3}-QUALIFICATION.json`.

Dois ambientes nativos35 tiveram HOME/XDG/bus próprios atestados. Zero IDE,
ferramenta, modelo ou ficheiro de código executado; os caminhos dos eventos são
apenas metadados inexistentes.1323 inputs atuais e artefactos exatos; QA próprio
encerrado e cinco verificações da instalação43 intactas. Nenhuma alteração de
produção/build/instalação/publicação. Não certifica todos os formatos de IDEs
reais, persistência de diffs após reinício, aceitação humana ou global49.
<!-- end-code-diff-source35-20261113 -->

<!-- installed35-delivery-20261113 -->
### Recursos da composição normal atual no Hyprland — Source35,13/11

`RESOURCES-SOURCE35-VERIFIED.json`: quatro arranques normais, pela ordem
oficial/35/35/oficial, com HOME/XDG/bus privados atestados, sem WebDriver,
interação de janelas, chat ou áudio. Depois de8s,25 amostras durante cerca de13s.
PSS final médio: oficial384,11MiB;35,422,49MiB; diferença **+38,38MiB** e um
WebKit adicional (4→5). CPU medida: oficial102,17% de um núcleo;35,0,63%.

Os conjuntos de processos mantiveram identidade e terminaram. São observações
delimitadas: camadas720×320 no oficial e720×560 no fork, possível oclusão pelas
camadas operacionais43 e restante atividade do anfitrião. Não provam a causa da
diferença de CPU, equivalência geral de recursos, energia/GPU, fuga de memória,
fluidez ou experiência instalada prolongada. O custo extra não é, por si só,
uma regressão confirmada ou motivo para nova infraestrutura. Esta prova não
substitui a medição29/X11, que mantém o seu cenário específico.

### Entrega instalada Source35/44 —13/11

Autorização explícita do proprietário recebida para o artefacto35 proposto e
registos focados assinados Coucou em sysworkspace; fork/push/merge continuam
suspensos. O primeiro registo `16d53be` passou hooks e assinatura YubiKey antes
da substituição. Índice de outras sessões preservado. O registo final é tratado
em separado pelo recibo `installed-source35-delivery/HOST-SIGNED-REGISTRATION.json`.

A43 tinha outro PID após reinício observado, sem causa atribuída ao agente.
Só o seu Quit nativo revisto foi enviado; terminal externo Codex preservado.
Backup específico Coucou, com fingerprint observado, verificado pelos leitores43
e35. Prefixo43 conservado; instalador restrito publicou44, gestor HOME aplicou
exclusivamente `.local/bin/coucou` e guardou backup transacional. Nenhum serviço
ou compositor recarregado. Processo1127174/startTicks18236395, dono D-Bus e
executável87400570… confirmados no perfil normal; sem automação de arranque.

`installed-source35-delivery/INSTALLED44-TECHNICAL.json`:49 ficheiros existentes,
48 com estado igual, só log alterado, nenhum ausente/novo; modelo grande por
metadados. Settings/pets JSON integrais iguais, incluindo seleção, modelos,
destinos, nomes/aliases e permissões; seis caminhos de integração e relay
preservados. Hub original com histórico/ditado e versão44 e página Pets atual
observados em capturas privadas da janela real. Só se navegou para Pets;
nenhum formulário de configuração, grant ou pedido de inferência foi aplicado.

O instalador releu os13 payloads como `already verified`; plano HOME terminou
com0 alterações. Check estrutural da estação e24 testes Coucou passaram.
Uma primeira invocação parcial de testes usou o entry point de imports errado;
o teste DMS não carregou nesse comando. O discovery normal passou, sem mudar
código ou repetir ensaios nativos. Não houve quota real Copilot/GitHub nem
inferência de conversa de teste. Provas QA35 de voz/regras/diffs mantêm os seus
âmbitos; entregar o mesmo binário não é repetir esses percursos no perfil pessoal.
Áudio físico/Bluetooth, contas reais, fluidez instalada prolongada, aceitação
humana e restantes49 critérios continuam abertos. Backup e capturas não são
versionados. Reverter launcher não desfaz migrações de dados.
<!-- end-installed35-delivery-20261113 -->

<!-- real-free-local-voice35-20261113 -->
### Conversa de voz com o modelo local real gratuito — Source35,13/11

`VOICE-LOCAL-REAL-SOURCE35-VERIFIED.json`: menu do pet privado→Whisper/áudio
virtual→modelo já instalado e escolhido `gemma3:4b` no Ollama local→resposta
correta em pt-PT a23+8→TTS→fim natural do recurso/retoma da escuta→Stop e
reinício com o mesmo CID/duas mensagens, sem outro pedido. Um único HTTP real
local,6,94s incluindo carga;1092 tokens de entrada e37 de saída declarados pelo
servidor. A resposta foi «Trinta e um, meu caro, é a soma de vinte e três com
oito, sem demora.» Sem ação proposta; permissões de ação do perfil gerado
estavam falsas. Proxy encaminhou bytes reais sem resposta fabricada, redirect,
fallback ou download; não mudou keepalive nem perfil operacional/Codex.

A observação de TTS usa identidade tipo/ID/serial e guardou a fase nativa
`listening` depois do fim. A transcrição independente da gravação completa
**não passou**: «E um, meu caro, é a soma de23,8, sem demora.» Não confirma a
primeira palavra nem toda a frase; não está determinada perda de áudio ou erro
de reconhecimento. Ambos os resultados brutos ficam conservados: percurso
nativo0, leitor ASR estrito1. Não se repete a conversa/ASR para obter verde e
não se certifica a qualidade acústica completa. A bolha de resposta privada
foi observada legível; sem aceitação humana ou áudio físico.

Duas instâncias nativas com HOME/XDG/bus próprios atestados. QA encerrado;
instalação44/PID1127174, binário, settings/pets, launcher e relay iguais.
1323 inputs exatos; nenhum delta de produto ou nova instalação. Esta unidade
usa a autorização original de testes gratuitos num perfil independente; não
é teste de inferência do perfil pessoal nem consumo Copilot/GitHub/pago.
Nomes legados SYNTHETIC/FIXTURE e parte da descrição bruta do helper permanecem
qualificados no agregado: os recibos de wire/modelo/usage são de inferência real
local. Não promovem para35 os cinco efeitos antigos33 ou aceitação global49.
<!-- end-real-free-local-voice35-20261113 -->

<!-- phone-admission36-20261113 -->
### Recusa local da admissão apresentada como falha de rede — Source36

O classificador oficial0.5.0/841a81ce atribui `unreachable` a erros locais sem
status HTTP; não tinha o novo guard comum de inferência do fork. A35 acrescenta
a admissão capturada, mas o consumidor do telefone só reconhecia algumas
recusas nativas como ocupação. O erro real «A configuração de inferência está
ocupada.» e o do snapshot do perfil caíam na regra genérica de rede.

A prova negativa bloqueia apenas o mutex de uma fonte `Settings` privada,
chama o `Captured::capture` de produção e verifica que a recusa inicial não
é uma falha de provider. Falhou exatamente `Unreachable` vs `Busy`; não foi
falha de compilação/ambiente. A36 reconhece as duas mensagens nativas de
ocupação no classificador existente e mantém o campo de diagnóstico na
categoria `configurationBusy`. A admissão continua recusada e nenhum HTTP
é iniciado por essa prova. Nenhuma espera/repetição, destino, modelo, conta,
autoridade ou política foi alterado.

Onze testes dirigidos passaram e a build privada86 da Source36/1323 inputs
exatos completou o percurso real UI/TLS/HTTP do telefone: sete respostas
sintéticas, continuação/reset, cancelamento/Busy, revogação/desconexão,
modelo retirado, histórico desktop separado e reinício sem replay.128 leituras
concorrentes de `phone_chat_status`, sem escrita/falha. Duas atestações do
processo confirmam HOME/XDG/roots de credenciais e Secret Service privados;
namespace de rede diferente, só interface loopback. Controlos observados antes
de ativar ligação e modelo. QR superior parcialmente fora do viewport na
segunda captura não é aceitação integral do seu layout.

`PHONE-SOURCE36-VERIFIED.json` fixa os recibos e limites. A contenção inicial
não foi forçada no ensaio nativo; a causa é demonstrada pelo teste dirigido do
produtor real, e o transporte Busy existente foi observado nativamente numa
ronda concorrente. Isso não atribui retroativamente os episódios22/82 à mesma
causa; continuam conservados/em aberto. Não houve modelo real/pago, Android
físico, conta remota, aceitação humana ou alteração instalada. Só
`phone_link/chat_backend.rs` difere da35; QA encerrado e instalação35 intacta.
A36 está implementada/integrada na árvore e validada em fixtures/nativamente
em desenvolvimento; falta pacote normal e entrega autorizada própria.
<!-- end-phone-admission36-20261113 -->

## Registos de atualização anteriores — histórico

Estas notas preservam as composições, falhas e decisões anteriores; consultar primeiro o estado atual e as provas de âmbito específico.

**Candidata48 preparada16/11 — ainda não instalada.** O RPM normal atual passou
n8n sem chat/modelos genéricos, conversa Codex sintética, calendários/reinício/
automático, revisão/sucessão/renovação,13 comandos nativos de preservação e leitura
restaurada dos formatos6/7 e10 com execução recusada. Provas e falhas qualificadas
no recibo; sem quota real no RPM, aceitação humana ou paridade global. Instalada44/
Source35 e preferências preservadas;49 critérios intactos e abertos. A substituição
requer autorização própria do artefacto e registo Coucou focado/assinado.
[Âmbito, artefacto, provas e aplicação proposta](PETS-INSTALLED-DELIVERY-48.md).

**16/11 — caso real corrigido; paridade oficial permanece delimitada.**
A nova rotina selecionou9h/10h de fontes diferentes no modelo local real, com dados
novos e resposta nativa preservada. Isso corrige a falha concreta registada, sem
certificar seleção geral ou comparar uma rotina equivalente no oficial841a81,
que continua **não verificada**. Os moldes LLM/application mantêm as funcionalidades
pertinentes por domínio; nenhuma conversa é herdada por serviços. Voz, código/diffs,
integrações, fluidez, recursos e instalada conservam os seus âmbitos de prova próprios.
Segue preparação da próxima candidata/aceitação operacional, sem recertificação global
da matriz; não autoriza substituição da instalação. [Provas e limites](PET-RULES.md).

**16/11 — qualidade do novo percurso autónomo; paridade não presumida.**
A publicação do contexto passou contenção real e o teste nativo completo atual.
Na nova rotina multicalendário, um teste real/gratuito do modelo escolhido recebeu
nomes/horários corretos mas inventou outros. Isso é uma falha confirmada do fork
neste percurso. Não existe aqui prova de um percurso autónomo equivalente na
referência oficial841a81, pelo que não se classifica automaticamente como regressão
do chat original. Estado comparativo: **ainda não verificado**; aceitação funcional
da nova rotina: **falhou**. Correção necessária: garantir o contexto efetivamente
usado no adaptador e repetir a prova factual com dados novos, preservando modelo,
voz/chat/código/diffs e as fronteiras por domínio. n8n não ganha chat por esta
correção. [Dados, resposta e diagnóstico](../.build/rule-calendar-link-20261116/SEMANTIC-FAILED-QUALIFICATION.json).

**16/11 — regra de recuperação reafirmada na camada de domínio da fábrica.**
Os moldes `llm`/`application` governam as funcionalidades aplicáveis; a base comum
e extensões de conta não podem distribuir chat/inferência a serviços. Foram
corrigidas descrições antigas da fábrica que ainda sugeriam essa possibilidade.
A prova funcional40/42 mantém o âmbito e limites registados abaixo; esta edição
não reclama nova aceitação instalada ou paridade global. [Contrato atual](PET-FACTORY.md).

**14/11 — integração da retirada móvel concluída em candidata privada:** [RPM47 atual, prova e âmbito de entrega](PETS-INSTALLED-DELIVERY-RETIRED-MOBILE.md). O ensaio nativo passou no próprio pacote; a instalação continua44.

<!-- retired-mobile-20261114 -->
**14/11 — retirada do pet GitHub aplicada também ao telefone.** Cache antiga
produzia um cartão móvel apesar da retirada no desktop; a negativa dirigida
falhou antes da correção. O resolvedor da fábrica é agora reutilizado na
projeção TypeScript, seleção apresentada nas definições, política efetiva
nativa e sanitização do hub. Os IDs de protocolo e dados guardados permanecem.
A prova inclui seleção antiga, envio a cliente ligado e snapshot de reconexão;
Notion e os restantes serviços mantêm o comportamento próprio.

38 testes web,53 testes Rust do hub e TypeScript passaram. A primeira compilação
falhou por uma referência incorreta ao módulo da fábrica e foi corrigida; o
teste adicional de UI precisou do status running no stand-in. Recibos conservados
em `.build/retired-mobile-20261114/VERIFIED.json`. A alteração final após o teste
Rust foi apenas a fixture web; os fontes de produto mantêm bytes iguais.
Sem Android físico, nova aceitação Tauri, pacote ou instalação. A proposta47/42
fica em espera: o RPM arquivado não contém esta correção. Segue compor a candidata
atual e manter a recuperação por domínio. Sem commits/push/alterações do anfitrião.
<!-- end-retired-mobile-20261114 -->

<!-- publication-restart42-20261114 -->
**14/11/2026 — publicação nativa após reinício verificada no mesmo RPM47/42.**
O handle80845 já não existia; o recibo do primeiro probe era terminal, com
falha ao observar o interruptor após reinício e sem processos próprios vivos.
Um novo ensaio dirigido, com instrumentação do interruptor e sem alterar os
fontes de produto, passou HTTP/TLS, publicação do cartão n8n e reinício.
O frame `services` chegou 105,22 ms após `pong` inicialmente e 159,53 ms após
reinício. `pong` não é barreira da publicação pendente do renderer; esperar o
evento real resolve o critério de observação do snapshot vazio neste percurso.
Históricos sintéticos Codex/n8n ficaram exatos; zero novos turnos de modelo,
quota, perfis pessoais ou alterações instaladas. Fontes42 continuam iguais.

Prova privada: `.build/upstream-parity-20261111/PUBLICATION-RESTART42-VERIFIED.json`.
As duas falhas anteriores permanecem: o timeout intermédio do interruptor não
foi reproduzido nem declarado corrigido. É validação nativa dirigida em X11 e
rede privados; não é aceitação humana, Android físico, instalada ou global49.
Próximo: revisão própria do pacote47/42 e preservação, mantendo a auditoria
funcional por domínio. Sem commits, push ou substituição da instalação.
<!-- end-publication-restart42-20261114 -->


### Continuação do objetivo autónomo17/11 — observação de projetos

A entrega [Source17/native9](PETS-PROJECT-FOLLOWUP-2026-11-17.md) acrescenta
acompanhamento dentro do domínio LLM e preserva os limites dos serviços/n8n.
As correções de contenção, relógio e cálculo estrutural pertencem ao percurso
novo do fork; o oficial0.5.0 não possui este contrato de objetivos. Não constituem,
por si, evidência de regressão ou paridade com o oficial. A matriz anterior,
qualificações de instalada44/RPM49 e ausência de aceitação global mantêm-se.


### 17/11 — integração da vista de projeto com o acompanhamento novo

Corrigida a recusa de consulta das ocorrências de projeto sem notas opcionais.
A aplicação anterior14036407 reproduziu a falha; a atual0a3e9482 leu duas respostas
e uma tentativa desconhecida, reiniciou e manteve os mesmos ficheiros, sem
inferência.24 web/tipos; backend inalterado. Este contrato autónomo não existia
no oficial0.5.0; é uma correção de integração do fork, não prova de regressão ou
paridade geral com o original. A matriz e limites instalados/humanos conservam
os seus âmbitos. [Provas e limites](PETS-PROJECT-PROGRESS-2026-11-17.md#ocorrências-do-acompanhamento-sem-notas-opcionais).


### 17/11 — pergunta de acompanhamento e resposta como nota

O binário privado bbac7a16 integra pergunta proveniente de relatório nativo,
revisão separada, nota de projeto, recusa de origem revogada e reinício. Uma
utilização posterior da nota exigiu objetivo/seleção/consentimento próprios.
As referências técnicas ficam recolhidas; as imagens atuais mostram a pergunta
e a resposta legíveis.5 Rust/79 web/tipos;4 chamadas sintéticas,0 reais/pagas.
É evolução do fork no seu domínio de inferência, não recuperação de uma função
atribuída ao oficial0.5.0 nem aceitação de paridade global/instalada. Serviços/n8n,
voz, código/diffs e as provas anteriores conservam os seus âmbitos.
[Contrato, provas e limites](PETS-PROJECT-DECISION-2026-11-17.md).

### 17/11 — prazo e preservação atuais das decisões do projeto

`b321a700…`: guarda UTC+monotónico corrigido com publicação Registry2 real;
7 Rust/79 web-tipos reutilizados, arquivo/restauro/rearquivo/restauro e reinícios
nativos, leitura histórica de escolhas/recusas e débito desconhecido, sem renovar
autoridade ou repetir inferência. O bloqueio de preparação seed5 continua aberto.
É preservação da evolução do fork no domínio de inferência, não nova aceitação
do Coucou oficial, voz/código/diffs/serviços ou da instalada44. As classificações
de paridade anteriores mantêm os seus âmbitos e limites.
[Provas e limitações](PETS-PROJECT-DECISION-2026-11-17.md#prazo-e-restauro-da-composição-atual).

### 17/11 — regressões de contenção do runtime corrigidas

Consulta autenticada com fence breve e selo de captura limitado ao mesmo resultado
sob Busy anterior à publicação passaram no binário privado0ba38c5e, com resposta
sintética e reinício. São correções da evolução do fork, não funcionalidades novas
atribuídas ao oficial0.5.0. Preservam voz, código/diffs e domínios anteriores sem
alargar as suas provas de paridade. A pintura física após reinício será verificada
no compositor real; não se aceita a captura X11 virtual em branco como prova.
[Provas e limites](PETS-PROJECT-DECISION-2026-11-17.md#consultas-e-publicação-da-captura-sob-contenção).
