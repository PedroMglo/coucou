**Entrega incremental42 (09/11):** volume/silêncio finitos TaskRuntime estão no pacote normal e na instalação42. Provas de efeitos continuam sobre áudio virtual privado; o arranque instalado foi verificado tecnicamente, sem aceitação visual/humana por sessão bloqueada. Registo assinado do anfitrião pendente. [Composição, comandos e limites](../.build/pet-goal-execution-next-r1/release42-native/VERIFIED.json).

# Concessões de Sistema por pet

Estado de09/11/2026: raiz1261/composição privada42 com execução nativa de tarefas literais de volume/silêncio pelo runtime comum. Build25 passou Tauri de admissão→claim→receita→verificação→conclusão assinada, replay, revogação por pet e Stop. Transportes, preservação62/48 e reserva física herdada têm provas próprias. Instalada38/pacote41 preservados; atualização instalada autorizada. Preserva os43+6 critérios integrais.

## Autoridade e âmbito implementado

O campo opcional `pets.json → pets[PetId].systemAccess` pertence ao writer nativo específico. Configuração2, Policy1, `enabled`, revisão aleatória nativa e lista fechada de grants. Cada grant identifica `subplugin=sound`, operação e recurso. A unidade atual suporta observar, volume0–100% e silêncio das saídas; recurso é a saída física exata ou todas as saídas explicitamente revistas. Entradas, streams, perfis, portas, Rede/Bluetooth e outras componentes não são concedidos por este formato.

O utilizador revê pet, operações e recursos na gestão existente. Confirmar conserva outras preferências e grants para recursos exatos. Cada confirmação gera outra revisão, mesmo ao reaplicar os mesmos scopes; revogar Spotify não revoga Ollama. Defaults, moldes, extensões e gravações antigas do perfil completo não criam nem substituem este campo. Não há nova fonte de estado desejado ou ledger de execução.

| Caller | Estado desta unidade |
| --- | --- |
| nativeUser | Gestão de concessão e revisão/admissão System verificadas por IPC Tauri real em perfil/áudio virtual privados, com escolhas QA. Rotas endereçadas e voz direta implementadas com pet/entrada ASR/sessão/grant originais antes da inferência; voz real, hardware e aceitação humana mantêm prova própria. |
| taskRuntime | Volume/silêncio nativos habilitados para planos finitos literais, mediante concessão própria e admissão27/objetivo62/grant48, lease/controlador e receita exatos. Tauri privado passou para Ollama e Spotify sem modelo. Gestão de concessões e consulta endereçada explícita continuam NativeUser; plano gerado/delegado e arranque automático System não são promovidos. |
| providerModel | Não anunciado. Texto ou argumentos com PetId não autenticam remetente nem concedem autoridade. Exige contexto nativo de turno/tarefa e governação comuns. |

Na voz, Spotify com qualquer política própria de Sistema, inclusive vazia/suspensa/recusada, usa essa política sem fallback para música. Spotify sem o campo conserva o contrato de voz legado. Comandos explícitos de Sistema malformados são recusados sem cair no parser de volume Spotify ou numa chamada de modelo. As outras superfícies legadas continuam pendentes de migração revista. Autoridade humana da UI/DMS, música e concessões próprias do pet são distintas.

## Execução e revogação

O backend captura pet, revisão, operação e chave física antes de esperar; verifica o original antes de cada helper. Observação de revogação é permanente nesse capture. O executor existente preserva exclusão física, identidade por cookie/serial/índice nativos e verificação posterior. As chaves permitem recusar um nome reutilizado observado antes/depois da operação; as chamadas separadas de `pactl` ainda não provam a ausência de substituição no intervalo entre consulta e despacho. Não se declara fechada a aceitação da encarnação física exata. Um grant para todas as saídas cobre essa classe por escolha explícita; um pedido iniciado continua preso à chave capturada.

A coleção nova consulta apenas informação do servidor e sinks, com limites de bytes, quantidade e duração por helper. Não consulta inputs/cards/streams. A publicação filtra de novo tipo e recurso. O limite por helper não é anunciado como prazo agregado da operação. Perda de autorização depois de possível efeito produz resultado não confirmado e aviso de incerteza; não há retry automático ou promessa de desfazer o efeito.

## Dados, migração e restauro

Perfis anteriores sem o campo mantêm a representação e preferências antigas. O writer legado conserva o campo atual, inclusive se receber uma cópia que o omite ou tenta alterá-lo. Na candidata41, o leitor operacional de configuração40 recusou o novo campo sem mutação, enquanto o backup40 o preservou como dados opacos. A prova não cobre arranque normal da app40 sobre esse perfil; não se presume compatibilidade operacional ou rollback instalado.

Backup conserva a política original como dados. A inspeção estrita reconhece o formato fechado e recusa campos/operações/recursos desconhecidos. A projeção operacional do restauro conserva grants e revisão, mas fixa `enabled=false`; música ou ativação do pet não os reativam. Só nova revisão nativa cria uma revisão habilitada. Histórico/original do arquivo permanece preservado.

## Prova e sequência

[Fontes, comandos, falha de fixture preservada e limites](../.build/pet-goal-execution-next-r1/system-pet-sound-grants/VERIFIED.json). Casos dirigidos verificam independência entre pets, ABA, recurso/argumentos trocados, observadores, origem retirada, writers antigos, defaults, restauro suspenso, gestão/ACK e publicação sem dados de outras classes. Não houve serviço de áudio real, inferência, quota, alteração do anfitrião, commit ou instalação.

Composição41 e manutenção nativa com dois pets passaram: política/revisão/recursos preservados no backup/restauro e `enabled=false` após projeção operacional. Duas falhas de preparação do ensaio estão preservadas no checkpoint: expectativa errada sobre backup opaco e ficheiro privado de escolhas com modo inadequado; não exigiram alterações ao produto. A voz NativeUser foi integrada depois da41: estado/volume0–100/silêncio das saídas, captura do alvo antes da espera, revalidação da origem/grant, Stop e recusa de sucesso após incerteza. [Provas e limites1236](../.build/pet-goal-execution-next-r1/system-sound-voice-adoption/VERIFIED.json). O caminho direto de voz não comprova admissão/journal de tarefa autónoma; essa consolidação, TaskRuntime/ProviderModel, migração integral do legado, restante matriz e aceitação funcional/instalada continuam pendentes. O Guardian continua sem especialização operacional; estas concessões são uma base comum, não a conclusão do pet ou do Sistema.

## Tarefas e alvo físico comum — raiz1239

`pet_runtime/physical_target.rs` conserva a representação legada Spotify e distingue uma saída de Sistema por pet, revisão do grant e chave. `pet_audio/task.rs` aceita apenas volume/silêncio tipados como dados, sem ferramenta de modelo ou PetId fornecido no texto. Os controllers de passos e receitas reutilizam Owned, Authority, reserva/consumo, recurso, intenção durável, Stop, resultado e recuperação; não há outro scheduler ou ledger. O executor sintético exige intenção Started antes da alteração, revalida depois de esperas e nunca repete um efeito incerto.

Scope3 de passo e Scope4 de receita/critério aplicam-se apenas à admissão manual2 já revista, com domínios HMAC próprios. As representações/HMAC anteriores mantêm-se. A verificação de Som usa formato2, observações do mesmo recurso e Unknown quando falta telemetria; o modelo não pode usar os novos snapshots como feedback de planeamento. Leitores antigos, migração/restauro destes novos formatos e o controlo nativo completo ainda precisam de prova específica.

A fábrica nativa mantém `native_dispatch_available=false`: um pedido real de tarefa de Som é recusado antes de consultar o serviço. A integração está validada com fixtures; não equivale a uma operação autónoma disponível. O transporte `pactl` usa ligações separadas e deixa uma lacuna entre identificação e alteração. Próxima dependência: reter o objeto PipeWire original pela API Quickshell já usada pelo DMS, sem recriar definições, mantendo supervisor, autoridade, limites, recurso e journal existentes. Remoção, reconexão, ACK real e identidade/grants precisam de prova antes de abrir a guarda. Esta limitação também impede declarar completos os percursos físicos anteriores baseados em `pactl`.

[Fontes, comandos, resultados, falhas conservadas e limites1239](../.build/pet-goal-execution-next-r1/runtime-system-sound-steps/VERIFIED.json). A candidata41/instalada38 não incluem esta unidade nem a adoção de voz1236.

### Transporte nativo retido — leitura1242

`pet_audio/pipewire_reader.rs` usa o mesmo `pet_helper::Resident`, agora também com um único stream Unix autenticado pelo processo supervisionado. O helper Quickshell é código fixo, com perfil temporário privado, sem janelas, setters, perfis autenticados ou nova infraestrutura persistente. Cada troca vincula sessão aleatória, sequência, prazo e autoridade original; um canal incerto não aceita replay. O mesmo OwnedChild mantém a família/recurso até à recolha. PID/grupo/uid e código de confiança não constituem sandbox contra código arbitrário do mesmo utilizador.

O leitor captura uma única referência QObject por id/serial nativos, conserva-a entre consultas e não a procura novamente pelo nome/default. Perda observada de objeto, readiness ou ligação retira a captura. A consulta real nesta máquina enumerou duas saídas, conservou o alvo em duas leituras e recusou um comando de alteração não suportado, sem escrever no áudio. Dez casos do supervisor e a regressão afetada do clipboard passaram; a primeira compilação falhou em três tipos de fixtures novos e foi corrigida. [Fontes, comandos e limites1242](../.build/pet-goal-execution-next-r1/pipewire-retained-reader/VERIFIED.json).

Esta prova é de leitura do serviço real. Não demonstra remoção/reconexão física, setter/ACK ou aceitação humana/voz. O despacho de tarefa de Som continua fechado: falta mapear o recurso concedido sem renovar grants, executar através do objeto retido com a intenção/orçamento originais e confirmar a pós-condição com evidência nativa independente do cache otimista. A API já usada pelos widgets pode atualizar o cache ao receber um setter; isso não é ACK do servidor. Não houve nova instalação ou alteração dos widgets.

### Efeito nativo tipado — executor1243

O mesmo driver tem agora um modo de efeito escolhido apenas pelo backend. O construtor de leitura fixa o modo sem efeitos, independentemente do ambiente herdado; argumentos e mensagens não o promovem. Volume0–100/silêncio usam uma referência QObject capturada uma vez, id/serial e cookie da instância nativa. Todos os helpers apontam explicitamente ao mesmo endpoint local; reinício ou perda observada retira a captura. O cookie é consultado antes/depois da ligação e antes das trocas, com limites e autoridade originais.

Prepare conserva os argumentos sem alterar o serviço. Um callback obrigatório é a fronteira de intenção/orçamento que a integração de tarefas tem de ligar à Authority existente. Depois de eventuais esperas, o backend revalida caller e origem de preferências; só então envia commit com uma transação consumível uma vez. O helper não aprova planos nem interpreta texto de políticas. Replay é recusado. Aceitação do transporte e cache otimista não são confirmação: outra ligação nativa lê a mesma encarnação e verifica a pós-condição. O escritor é consumido e a sua família, bem como a do observador, é recolhida antes de libertar o recurso físico comum; a validade é conferida novamente após a recolha. ACK ausente não provoca repetição.

Na raiz1243 passaram alterações de volume/silêncio num daemon PipeWire privado com saídas virtuais, recusa do callback, Stop após a intenção, replay após outra alteração, substituição com o mesmo nome e reinício com id reutilizado. A leitura da instalação PipeWire do utilizador continuou sem alterações. A biblioteca fora de cfg(test) passou check; o percurso Tauri real de autoridade ainda não foi exercitado nesta unidade. [Fontes, comandos, falha de compilação preservada e limites1243](../.build/pet-goal-execution-next-r1/pipewire-retained-effects/VERIFIED.json).

O callback nativo foi ensaiado com fixtures de controlo, não com o ledger real de um objetivo; essa ligação permanece pendente. Falta relacionar as chaves/concessões existentes com cookie/id/serial sem as renovar, adotar o executor no Target/journal comuns, completar admissão determinística e controlo funcional, e provar preservação/leitores/aceitação apropriada. TaskRuntime/ProviderModel continuam sem nova capacidade anunciada. Não houve alteração do áudio do utilizador, instalação, criação de serviço persistente ou conclusão do Guardian.

### Concessões e tarefas nativas — raiz1245

`pet_audio/native_task.rs` relaciona a chave já concedida com o cookie da mesma instância PipeWire, id e serial do nó. A conversão do cookie hexadecimal do Pulse serve apenas a comparação; os bytes da chave, perfil e revisão do grant mantêm-se. Campos nativos do Node não são publicados no JSON. A captura confere o inventário antes/depois e recusa ambiguidades, outra instância, seletor ausente, novo dispositivo com o mesmo nome ou perda do alvo; não usa nomes como fallback. Os clientes Pulse têm endpoint local fixo, autospawn desativado e estado temporário próprio, sem copiar perfis autenticados.

O Target comum conserva uma âncora de leitura sobre o objeto original. Cada efeito cria um escritor para a mesma instância/id/serial antes da espera física; a intenção original da Authority é chamada antes do commit. Escritor e observador são recolhidos pelo supervisor comum. A âncora, que nunca escreve, verifica de novo o estado final. O volume segue a escala dos widgets nativos (`audio.volume ×100`); o Pulse relaciona a identidade. Os percursos legados de Pulse/voz conservam o seu histórico e exigem migração/paridade explícitas, sem presumir equivalência de escala.

Cinco cenários com serviços Pulse/PipeWire privados e Authority real passaram: passo único/orçamento e replay que conserva uma alteração externa; receita de dois passos com conclusão certificada e recusa após alteração externa; grants de pets independentes/revogação observada permanente; substituição/reinício sem efeito ou reserva; revogação durante consulta nativa observada em execução, sem worker perdido. A consulta de identidade no serviço real do utilizador passou sem alterações. [Fontes, comandos, deltas, falha de compilação preservada e limites1245](../.build/pet-goal-execution-next-r1/native-sound-task-adoption/VERIFIED.json).

Esta prova usa admissão2/validade preparadas por fixtures. Não é a admissão determinística operacional, autoridade Tauri, voz real ou entrega instalada. Os limites da âncora são180s/256trocas e os slots de leitura existentes; revisão/expiração/Stop/cache/recolha precisam de aceitação nas superfícies funcionais. Seguem extensão fechada da admissão comum e controlos mínimos, preservação/restauro/leitores e percurso nativo efetivo. A fábrica pública continua a recusar Som antes do IO; nenhum caller recebe capacidade nova só por esta integração. A candidata41 e a instalada38 não contêm este incremento; Guardian operacional e o objetivo integral continuam pendentes.

## Admissão determinística de Sistema — contrato em implementação (09/11/2026)

Uma tarefa mecânica de Sistema usa a fila, Review/Approved, Authority, orçamento e
registo de efeitos existentes, sem carregar um modelo. A nova família de admissão27
(Goal62, Grant48, Scope4/receita5, critérios5/verificação3) distingue-se dos antigos
passos Sound manuais: o objetivo autónomo revê antecipadamente operações, argumentos,
causas, critérios completos, pet, revisão da concessão e encarnação nativa exata
(cookie do core, node id e serial), antes de hash/reserva. A política serializada é
dado, não autoridade; só o Review retido pode produzir Approved. RuntimeOnly exige
zero chamadas de modelo e não permite fallback.

A primeira composição é um plano finito para uma saída, com contexto dedicado sem
histórico/memória selecionada. Não promove automaticamente regras, delegações ou
planeamento gerado: esses percursos conservam os próprios contratos e lacunas.
Ausência de modelo não retira a identidade nem autoriza efeitos. Nova concessão,
recriação de dispositivo/core, mudança de argumentos ou critérios requer nova revisão.
O gate público continua fechado até controlos funcionais, recuperação/lifecycle e
preservação compatíveis estarem comprovados. Este contrato não declara entrega.

A raiz1249 implementa este contrato na admissão27/Goal62/Grant48 e liga-o a
passos4/receitas5/critérios5/verificação3 existentes, com domínios separados sem
alterar os bytes legados quando a política está ausente. O parser de ações é
partilhado, puro e independente da plataforma; transporte continua Linux.
Sete identidades (Codex, Copilot, Gravity, Ollama, LM Studio, API e Spotify)
passaram admissão e conclusão nativas privadas **com metadados de provider
sintéticos**. Stop antes do efeito produz Cancelled e grant revogado, sem débito.
Não demonstra providers operacionais nem confirmação humana/Tauri.

Restam a superfície funcional de revisão/confirmar/admitir, arranque explícito
e a entrega. O formato48 de retirada de uma autorização órfã ainda precisa de
contrato e provas; recuperação após crash, migração/leitores antigos, restauro e
recolha de anchors fora das locks globais também continuam gates explícitos.
Não usar a admissão peer existente para confirmar um Review System: a observação
System deve ser escolhida pelo estado nativo retido, não por um texto/JSON recebido.
Não abrir taskRuntime/providerModel nem renovar autoridade ao reconstruir alvos.
Fontes, comandos, falhas conservadas, deltas exatos e limites: [VERIFIED](../.build/pet-goal-execution-next-r1/deterministic-system-admission/VERIFIED.json).

## Controlos nativos da admissão System (09/11/2026, em integração)

O comando nativo de preparação recebe referências do objetivo, limites e critérios
tipados; não recebe uma política, concessão ou encarnação de dispositivo do cliente.
Captura a saída e permissões pelo backend, apresenta a revisão completa e usa a
mesma confirmação de admissão. A evidência System é escolhida pelo Review retido
pelo backend, sem chamar o classificador peer nem resolver um modelo. Aprovar
apenas admite na fila; o gate de execução pública continua independente e fechado
enquanto as provas de preservação/lifecycle restantes estiverem pendentes.

A cache comum conserva revisões expiradas/duplicadas até sair do bloqueio global,
e só então recolhe os processos auxiliares; a invalidação de contexto encaminha
essa recolha para o executor blocking existente. Não introduz outro scheduler.

Na raiz1254, `pet_runtime_prepare_system` está registado e ligado ao painel de
objetivos. A consulta usa `pet_sound_snapshot` com o pet capturado e limita-se às
saídas concedidas. Uma resposta antiga após trocar de pet/vista é ignorada.
Guardar usa a criação determinística/contexto dedicado existente; rever produz
o mesmo Review/Approved/Authority, e o ACK após resposta perdida não consulta um
modelo ou renova dados, concessões, prazo ou saldo. O campo `nativeSystemPlan`
é escolhido na confirmação provém do Pending nativo retido, não do cliente.

`goals.systemAdmission` é apenas NativeUser e passivo: apoio à revisão, com alvo
ainda necessário, nunca disponibilidade operacional dos efeitos TaskRuntime. O
painel mostra o executor ainda em integração e não oferece claim/run de Sistema
como disponível sem essa projeção. Volume/silêncio neste formulário são estados
pretendidos de uma tarefa, não substituição dos widgets/definições do sistema.

A cache comum de admissão transfere referências descartadas para fora da lock
global em expiração, duplicados, conflito e erro; a invalidação do contexto usa
o executor blocking existente. Há prova nativa privada de expiração/duplicado
com retenção até libertar a lock. Não é prova de recuperação de helpers após
crash, nem cobre ainda as caches nativas de passos/receitas.

Provas e falhas conservadas: [VERIFIED](../.build/pet-goal-execution-next-r1/system-admission-controls/VERIFIED.json). As três confirmações novas usam Frames/escolhas e metadados de provider sintéticos sobre serviços nativos privados. O bundle web privado e check da biblioteca não demonstram a composição Tauri ou instalação. Próximo: retirada de grants48 órfãos, preservação62/restauro/leitores anteriores, caches de alvos e recuperação de famílias de processos; depois composição e teste nativo dos controlos antes de abrir o gate público.

## Retirada de autorização System sem admissão (09/11/2026, em implementação)

O grant48 emitido antes da publicação na fila pode ficar órfão. A sua retirada
reutiliza a revisão/decisão/transação de retirada existentes, com referência3
exclusiva do formato48. Não reconstrói AdmissionSpec, alvo ou executor, não usa
modelo e não renova concessões, prazo, saldo ou aprovação. Exige grant canónico
autenticado, não associado à admissão e comprovadamente nunca reclamado, mais
perfil original, inventário, inodes e ausência de artefactos de IO. Uma admissão
ou claim omitidos no ficheiro do objetivo não provam ausência de utilização.

A publicação conserva formato48/domínio HMAC System e hash da política System,
specHash e approvedAt originais. O novo estado revogado recebe revisão, data, recibo e assinatura próprios pela mesma autoridade; os dados originais ficam vinculados na referência.
ACK perdido confirma a mesma escolha mesmo após o TTL, sem nova autorização.
Referências1/2 mantêm os formatos antigos; formato3 não se pode converter em2.
O painel distingue a redução de autoridade System de resposta/inferência.

A raiz1255 implementa a referência3 na retirada System48. A construção e validação
reconhecem exclusivamente o grant canónico48, e a publicação mantém o respetivo
formato/domínio. O estado nativo é calculado pela família original do grant,
conservando também as famílias anteriores. A proposta nativa transporta o hash
System opcional autenticado; ausência mantém os bytes antigos. O painel mostra
“Sistema sem inferência” e não apresenta a operação como resposta de um modelo.

Provas privadas usam três donos (Ollama, Copilot e Spotify), sem quota nem servidor
ativo durante a retirada. Só o grant canónico muda, o objetivo e concessões ficam
iguais, e uma projeção queued com a prova original é recusada como revogada.
Publicação parcial, retirada recusada antes de commit e ACK após perda/reinício
preservam a escolha original; uma admissão ou claim omitido impede retirar.
Os leitores antigos da aplicação e o restauro/migração completos ainda precisam
de aceitação própria. Isto fecha a lacuna da retirada48, não a entrega do Sistema
ou do Guardian. [Evidência exata](../.build/pet-goal-execution-next-r1/system-orphan-retirement/VERIFIED.json).

## Preservação nativa System62/48 — composição privada42 (09/11/2026)

A aceitação distingue preservar bytes de compreender ou executar o formato. Um
leitor anterior pode criar/verificar backup opaco; inspeção operacional/restauro
de autoridade exige codecs capazes de validar objetivos62, grants48 e referência3.
Formato desconhecido não autoriza reescrever, migrar destrutivamente ou executar.

O teste usa só perfis gerados pelo publicador real, com estados queued, concluído,
contabilização incerta e grant órfão retirado. A contabilização incerta é produzida
por intenção iniciada sem despacho nativo e sem ACK; não prova crash físico nem
efeito perdido. O restauro deve conservar consumo e desconhecidos como histórico,
quarentenar autoridade antiga, suspender concessões System e impedir replay, sem
renovar saldo, prazo ou consentimento. A composição42 é privada até aceitação
própria; não implica instalar ou afirmar validação humana/janela/hardware.

A composição privada42 passou os quatro percursos de backup/restauro nativo.
Objetivos, grants48 (incluindo retirada3) e orçamentos conservaram bytes/hashes;
a projeção de configuração preservou as concessões com enabled=false. O runtime
recusa o replay dos donos admitidos e apresenta consumo como snapshot histórico,
sem renovar autoridade. O leitor41 aceitou revisão de backup opaco e recusou
a inspeção de autoridade nos quatro casos, sem alterar payloads. Não foi testado
o arranque normal desse leitor nem uma janela/aceitação humana. A fixture de
incerteza não despacha áudio. [Composição, comandos e limites](../.build/pet-goal-execution-next-r1/system-preservation42/VERIFIED.json).

A atualização instalada está autorizada pelo utilizador. A instalada38 continua
preservada enquanto se fecha a composição e a aceitação aplicável; a build42
é privada, e a candidata41 permanece imutável. Isto é uma condição técnica
de entrega, não falta de autorização. Próxima dependência: recolher alvos de
passos/receitas fora da lock global e provar o lifecycle dos helpers antes
de abrir os efeitos públicos; depois controlo Tauri e composição/entrega.

## Recolha de alvos de passos e receitas (09/11/2026, em implementação)

As caches nativas existentes retiram revisões/alvos expirados, libertados ou
recusados para uma lista local de descarte. A posse do alvo só termina depois
de soltar a lock global da cache, incluindo entradas novas recusadas e duplicadas.
As chamadas de libertação do serviço de leases também soltam a sua lock antes
de destruir a entrada e limpar as caches. A confirmação durável do drain continua
a anteceder esta libertação; expiração não demonstra morte de workers. Não muda
formatos, saldo, consentimento, scheduler ou gate público de Sistema. A prova
será de lifetime nativo privado e regressões do caminho existente; não de crash
do backend, aceitação humana ou instalação.

A recolha foi implementada nas duas caches existentes e nos quatro pontos de
libertação do serviço. Dois testes com alvos nativos privados conservaram a
posse durante a lock e descartaram após a saída, incluindo entradas recusadas,
expiração, conflito/capacidade e libertação da lease exata. As91 regressões de
passos/receitas e6 de manutenção passaram; quatro helpers privados permanecem
excluídos da invocação direta e são documentados na prova. Não foi repetida a
suite integral. A build22 privada42 antecede esta correção; crash real do backend,
controlos Tauri e entrega instalada mantêm aceitação própria.
[Fontes, comandos e limites](../.build/pet-goal-execution-next-r1/system-task-cache-reaping/VERIFIED.json).

## Tauri nativo: revisão, admissão e crash antes do efeito (09/11/2026)

A build23 privada42 contém a raiz1259 e o bundle atual. Em Fedora/Wayland,
WebKit acedeu aos comandos Tauri originais com Frames reais, configuração/dados
gerados e bus/áudio virtual privados. O writer criou a concessão exata do pet;
um objetivo sem modelo recebeu revisão de alvo, critérios e orçamento. O token
foi perdido após SIGKILL do backend; o auxiliar readonly terminou em cerca de50ms,
e o reinício preservou objetivo/concessão e recusou o token com conflict.

O segundo percurso confirmou a admissão27/objetivo62/grant48 antes do crash.
Após reinício, bytes do grant e orçamento conservaram-se; Cancel revogou apenas
a autorização do objetivo pelo writer original. Nenhum efeito foi despachado.
As escolhas de aprovação foram geradas por QA; isto valida IPC/Frames/writers
e o lifecycle anterior ao efeito, não aceitação humana nem crash durante/após
um possível efeito. A recolha após morte foi observada pelo supervisor privado,
não atribuída ao backend morto. A primeira tentativa de automação falhou por
faltarem flags de automação no ensaio; foi preservada, sem alterar o produto.

Próxima unidade: falha real do worker no limite intenção/despacho/recibo,
preservando incerteza, saldo e ausência de replay no journal existente. Segue
a composição/execução Tauri e a entrega instalada autorizada. Voz real, restantes
componentes de Sistema, Guardian e os requisitos integrais não ficam concluídos
por estas provas. [Fontes, comandos, resultados e limites](../.build/pet-goal-execution-next-r1/system-native-crash-lifecycle/VERIFIED.json).

## Crash no limite do efeito nativo (09/11/2026, em implementação)

A prova usa um subprocesso nativo com o Store/Authority/Owned originais e áudio
virtual privado. Pausas apenas cfg(test) delimitam intenção/orçamento iniciados
antes do despacho e observação nativa após o efeito, antes do recibo canónico.
SIGKILL não executa destructors: a prova tem de observar a família nativa e
reabrir o mesmo perfil. Estado atual do dispositivo não substitui o recibo
original. Na ausência deste, conservar efeito/consumo desconhecidos, impedir
replay e não renovar autorização, prazo ou saldo. Esta prova do worker não
equivale ao despacho Tauri completo nem a aceitação humana/hardware.

Três casos passaram com o executor e journal originais: antes de commit (saída
virtual permaneceu100%), commit escrito com ACK por ler (saída observada35%) e
observação independente devolvida sem recibo canónico (35%). Após SIGKILL, os
auxiliares terminaram; a reabertura conservou a tentativa started, consumo
desconhecido, orçamento/prazo/identidades e recusou recuperar recibo ou repetir
o claim. Nenhum critério ou conclusão foi inferido do estado atual do áudio.
Os10 casos do transporte residente e91 de passos/receitas passaram; os quatro
helpers privados excluídos da invocação direta estão identificados na prova.

A alteração de código é exclusivamente cfg(test); a reconstrução das fontes
de produção da build23 é exata. Não se confunde este worker nativo com execução
Tauri completa. A exclusão atual do Coordinator é em memória: esta prova não
fecha a reserva física contra um novo backend enquanto um auxiliar órfão estiver
parado/vivo. Essa reserva entre processos é a próxima dependência antes de abrir
o gate público; não pode ser substituída por expiração da lease ou morte só do
backend. [Comandos, fontes, falhas preservadas e limites](../.build/pet-goal-execution-next-r1/system-effect-crash-recovery/VERIFIED.json).

## Reserva física de áudio entre processos (09/11/2026, em implementação)

A reserva de AudioEffect usa o runtime nativo explicitamente selecionado, sem
modelo ou novo ledger. Reutiliza a validação de pastas/ficheiros do registry
privado e distingue o lock normal do lock herdado: o primeiro mantém o unlock
explícito existente, o segundo só se liberta quando se fecha o último descritor.
O descritor é CLOEXEC no backend e só passa ao helper de efeito revisto. Não se
apaga o ficheiro para libertar nem se infere posse por PID ou lease expirada.

O writer nativo obtém a reserva antes de nascer; observa e executa com a mesma
reserva até à recolha. Efeitos legados e endereçados de Som adotam o mesmo nome
no runtime configurado. Leitores não tomam a reserva. Acesso negado/ocupado não
concede outra permissão, fallback ou efeito. Isto coordena processos confiáveis
do mesmo utilizador; não é sandbox contra código arbitrário que altera ficheiros
do mesmo UID ou fecha descritores de propósito. As autorizações continuam nos
writers/brokers existentes. O primeiro ensaio de órfão parado terminou pela
semântica normal de grupos do kernel e não chegou a comprovar a condição; será
preservado e complementado por sessão separada apenas na fixture adversarial.

A reserva foi implementada e comprovada com o writer nativo parado numa sessão
separada: depois de SIGKILL do backend, um novo writer foi recusado; a mesma
reserva só voltou a ser obtida após terminar a família original. Leitura manteve-se
disponível e o probe não despachou efeito. O descritor CLOEXEC do backend passa
apenas ao helper revisto. A variante herdada mantém o lock por último close; o
writer normal do registry conserva unlock explícito, incluindo cópia por fork.

Passaram seis casos de ficheiros/kernel/cancelamento,16 helpers,4 registry,6 Som
legado, dois percursos nativos de efeito/replay/substituição e os três crashes de
tarefa com consumo desconhecido. A build24 privada42 compilou; na aplicação
Tauri real, o NativeUser endereçado alterou só a saída virtual, e a admissão62/48,
reinício, token antigo recusado e Cancel continuaram válidos. As aprovações são
QA, não aceitação humana. Gate TaskRuntime mantém-se fechado nesta composição.
Próximo: projetar capacidades por caller e ativar/testar o percurso de tarefas
revistas na aplicação nativa, antes da composição e entrega instalada autorizada.
[Fontes exatas, comandos, falhas preservadas e limites](../.build/pet-goal-execution-next-r1/audio-effect-process-fence/VERIFIED.json).

## Adoção pública de tarefas determinísticas de Som (09/11/2026, em integração)

O percurso nativo já comprovado pode ser invocado pelo runtime original para
planos literais finitos de volume/silêncio. A disponibilidade não autentica caller,
concede acesso ou permite recuperar um Ready a partir de JSON. Admissão System,
concessão própria, encarnação nativa, lease/controlador, orçamento, revisão de
receita e critérios continuam obrigatórios e revalidados antes do efeito.

A projeção distingue NativeUser/TaskRuntime para volume e silêncio; gestão de
concessões e consulta endereçada explícita continuam NativeUser. ProviderModel
não ganha catálogo, operação ou grant. Unknown permanece Unknown até à resolução
ativa autorizada do alvo. Não se promove arranque automático/delegado de System,
inferência, outros subplugins ou concessões por omissão. A aceitação é o percurso
Tauri de execução/conclusão e recusa/revogação, sobre dispositivos virtuais
privados; a exposição na raiz de desenvolvimento não equivale a instalação.

A adoção foi implementada: volume/silêncio anunciam NativeUser/TaskRuntime em
Linux, sem novos grants; gestão/consulta/admissão mantêm NativeUser e o catálogo
ProviderModel continua fechado. Availability permanece Unknown/TargetRequired
até resolução ativa autorizada. Os51 casos de capacidades, seis de Som e cinco
de controlador web passaram; tipos, bundle e build25 privada42 compilaram.

Na aplicação real, Ollama e Spotify completaram duas operações cada pela fila,
Authority, lease, revisão de receita e critérios nativos originais, com recibos
verificados e conclusão assinada. Repetição não mudou o ledger/dispositivo.
Revogar Spotify não alterou Ollama e bloqueou a receita retida de Spotify.
Stop durante espera por IO no grafo privado impediu novos efeitos. Zero chamadas
a modelos, sem efeito em dispositivos do utilizador. Uma primeira corrida ficou
parcial após as duas conclusões por Busy no writer de criação do terceiro objetivo;
o ensaio passou a repetir apenas Busy conhecido de dados/preparação/leitura
idempotentes com os mesmos argumentos, nunca despacho/resultado incerto.

A integração na raiz e build privada não é entrega instalada nem aceitação humana.
Segue composição nativa com identificador normal, preservação do pacote e
atualização já autorizada, respeitando o registo do anfitrião. Os restantes
subplugins, voz real, Guardian e requisitos integrais continuam com critérios
próprios. [Fontes, comandos e limites](../.build/pet-goal-execution-next-r1/system-task-public-adoption/VERIFIED.json).



## Saúde limitada e Guardian — integração local09/11/2026

A mesma Policy1 suporta Health/Observe/HostHealth e mantém Sound/Observe/Volume/Mute em recursos de áudio separados. O writer valida a combinação subplugin/operação/recurso; um grant de Saúde não satisfaz Som, nem o inverso. Capturas reutilizam invalidação/revisão próprias, gate nativo endereçado e revalidação antes/depois das leituras. Outro pet conserva a sua concessão quando uma é retirada.

pet_health_snapshot é NativeUser apenas e não escolhe modelos, publica memória ou admite tarefas. Consulta três caminhos procfs fixos e limitados; devolve origem/instante/cobertura/erro parcial. Contadores de CPU exigem baseline separado; finding de memória é amostra com limiar explícito5%, não vulnerabilidade ou autorização de tuning. Máximos:8835 bytes lidos e8192 de resultado, com janela de publicação250ms; não se promete interromper um syscall bloqueado. O editor existente permite rever concessão e consultar Saúde, sem criar definições próprias de hardware.

A revisão de catálogo1 permite adicionar identidades futuras declaradas sem substituir dados anteriores e sem grants/modelos herdados. O restauro reconhece apenas o campo conhecido e passa pelo resolvedor tipado; revisões futuras e campos desconhecidos continuam recusados. Restauro suspende concessões e não renova consentimento. [Provas e limites](../.build/pet-goal-execution-next-r1/guardian-health-inventory/VERIFIED.json). O código está em composição privada29, ainda fora da instalação42.

## Saúde nativa no runtime comum —09/11/2026

`system.health.observe` v1 aceita apenas `{}`. É um único passo dedicado RuntimeOnly/zero modelos, com concessão própria Health/Observe/HostHealth, identidade de arranque/namespaces capturada antes do hash e aprovações originais. Criterion7/SystemHealthObserved certifica cobertura válida dos3 contadores; não diagnóstico, otimização ou segurança. Policy2/Admission28/Goal63/Step5/Recipe6/Verification4 distinguem a nova família. Domínios assinados anteriores preservados; não há novo scheduler ou ledger.

NativeUser pode consultar diretamente; TaskRuntime pode admitir o plano finito pela mesma revisão/autoridade de Sistema. ProviderModel, geração/delegação de Saúde e executionStart automático permanecem indisponíveis. Consulta passiva não lê procfs nem seleciona modelo. Janela2s para tarefas,250ms manual; cada amostra conserva limites de bytes e publicação após revalidação. Prontidão/verificação usam observações limitadas sob a concessão original. A build privada31 comprovou Guardian/Ollama, reinício, revogação independente e replay recusado; não foi instalada. Restauro63/48 passou pela CLI nativa com autoridade histórica em quarentena e ativação sem grants; a entrega continua pendente. [Prova](../.build/pet-goal-execution-next-r1/guardian-health-preservation/VERIFIED.json). [Provas exatas e limites](../.build/pet-goal-execution-next-r1/guardian-health-runtime/VERIFIED.json).


Entrega incremental43 (09/11): mesmas bases nativas31 com versões alinhadas, pacote normal43 comprovado e instalado, seleção/grants/modelos preservados. Guardian disponível desligado;26 entradas de catálogo não significam26 ativos/operacionais. Prova automatizada e registo assinado do anfitrião pendente; não fecha GUA integral. [Composição, preservação, instalação e limites](../.build/pet-goal-execution-next-r1/release43-native/VERIFIED.json).

## Medição curta de Saúde — privada32,09/11/2026

`system.health.baseline`v1 aceita apenas `{}` e reutiliza Health/Observe/HostHealth, sem conceder inventário, processos, logs, IO/GPU ou alterações. São duas amostras dos3 contadores fixos sob o mesmo pet/revisão/arranque/namespaces, separadas por pelo menos1s, até6s totais. Limite17670bytes de métricas e8192bytes do resultado; identidade/política têm leituras limitadas próprias. CPU é estimativa agregada fora de idle/iowait; deltas inválidos/zero/reset/suspensão ficam indeterminados. Memória deriva diferença apenas com total consistente. Não se declara saúde, segurança ou melhoria.

NativeUser dispõe do controlo existente; TaskRuntime usa uma tarefa finita dedicada, um passo/zero modelos pela admissão comum. Intenção e reserva originais antecedem a medição; prontidão não recolhe métricas extra. O predicado `SystemHealthBaselineMeasured` só é satisfeito por amostras completas e derivados válidos. A verificação reutiliza o recibo assinado original, sem recolher outra medição. Stop/revogação/prazo são revistos durante espera/IO e antes da publicação. ProviderModel, geração/delegação e arranque automático continuam fechados. Policy2/Admission28/Goal63/Step5/Recipe6/Criteria7/Verification4 mantêm as famílias assinadas de Saúde; Som anterior fica compatível.

Build32/Tauri nativo e preservação CLI passaram em perfis gerados; o leitor43 recusa a nova evidência sem destruir dados anteriores. Restauro conserva objetivos/rascunhos/recibos, põe autoridade em quarentena e não renova grants. Decisões QA são sintéticas, não aceitação humana. A instalação43 não contém esta medição. [Fontes, comandos e limites](../.build/pet-goal-execution-next-r1/guardian-health-baseline/VERIFIED.json).

## Inventário seletivo OS/kernel — privada33,09/11/2026

Primeiro recurso fechado: Inventory/Observe/HostComponents, próprio por pet e separado de Health e Som. Consulta apenas a identificação efetiva da distribuição e o tipo/release do kernel em execução, com origem/cobertura/tempo e classificação observado externo/dinâmico. Estado desejado/sysworkspace é desconhecido até integração comprovada. Máximos8451bytes de fontes,8192 de resultado e2s; nenhuma execução de shell, pacote/processo/serviço/log, rede, dado pessoal, alteração ou modelo. Revisão e publicação revalidam o dono/revisão/arranque/namespaces originais. Consulta passiva não recolhe dados. NativeUser apenas nesta unidade; autonomia/ProviderModel/delegação fechados, não se declara GUA-01 completo. Leitor43 deve recusar o grant novo sem o destruir; restauro conserva-o suspenso. Estado: implementado na raiz e validado nativamente em privada33; não instalado, sem admissão autónoma.

Implementação inicial na raiz: `pet_host_inventory`, concessão Inventory/Observe/HostComponents e controlo explícito no editor existente. O leitor só admite o symlink conhecido para `/usr/lib/os-release`; a ausência de `/etc/os-release` permite usar explicitamente a fonte vendor, identificada na evidência. Campos selecionados repetidos, symlinks desconhecidos e texto inválido falham fechados; campos alheios não são publicados. [Formato de referência](https://github.com/systemd/systemd/blob/main/man/os-release.xml). Estado: controlos e grants próprios passaram em Tauri nativo privado33, reinício/leitor43 e preservação CLI; não instalado e sem admissão autónoma.

Prova privada33: Guardian/Ollama conservaram grants próprios; Saúde não autorizou Inventário, nem Inventário autorizou Saúde. O controlo existente publicou distribuição/kernel com origem/hash e estado desejado desconhecido, sem modelo. Revogação de um pet não afetou o outro; reinício conservou configuração/revisões exatas. Leitor43 recusou o grant desconhecido sem reescrever a configuração. Backup/restauro nativo preservou dados e concessões suspensas, com preparação independente de autoridade futura e ativação sem grants, sem renovar execução histórica. Consentimento QA sintético, não aceitação humana/produção instalada; processos/pacotes/serviços/logs, diagnóstico de vulnerabilidade e remediação continuam pendentes. [Provas, comandos e composição](../.build/pet-goal-execution-next-r1/guardian-host-inventory/VERIFIED.json).

O limite8451bytes do inventário conta apenas as3 fontes de componentes, incluindo deteção de excesso; as verificações originais limitadas de política e identidade/namespace têm IO adicional. A janela de publicação2s abrange também essas verificações e a espera do worker, sem prometer interromper um syscall bloqueado.

## Inventário no runtime comum — privada35,09/11/2026

`system.inventory.observe`v1 recebe apenas `{}` e exige Inventory/Observe/HostComponents próprio. Usa Binding/Target/Snapshot de Inventário distintos de Saúde e Policy3 com campo inventory. Reutiliza a família de leitura finita do anfitrião Admission28/Goal63/Step5/Recipe6/Criteria7/Verification4, Grant48 e os mesmos Store/Review/Authority/fila/leases/recursos/intenções/orçamentos. Não criar scheduler/ledger por especialização. Os domínios HMAC históricos e bytes de Som/Saúde mantêm-se: novo alvo/operação/predicado integram o corpo assinado e o hash semântico System exato; leitor43 recusa variantes desconhecidas sem as destruir.

Captura dono, concessão e arranque/namespaces antes do hash/reserva. Um passo dedicado, zero inferência/dados selecionados, critério tipado de inventário completo e limitado; não diagnostica segurança/estado desejado. Prontidão só valida identidade/autoridade; intenção e reserva duráveis precedem as fontes. Verificação usa o recibo assinado original, sem nova leitura. Stop/revogação/prazo conservam prioridade e não renovam autoridade após reativação/reinício. NativeUser e TaskRuntime finito; ProviderModel, geração/delegação e arranque automático continuam fechados. Estado: implementado e integrado na raiz, validado em Tauri nativo privado35; instalação43 preservada.

Prova privada35: o formulário guardou um objetivo Guardian com contexto dedicado e sem motor; a revisão exata apresentou pet/grant/arranque/namespaces/operação, e a recusa não concedeu execução. Separadamente Guardian/Ollama completaram dois objetivos por admissão/fila/leases/receitas/Authority originais, com0 chamadas ao modelo e1 ferramenta. O recibo assinado de cada passo foi conservado como evidência final; reinício/replay/revogação independente passaram. Leitor43 devolveu invalidRecord para o objetivo novo e recusou a configuração Inventory, sem destruir os registos prévios; apenas uma importação legada vazia compatível de rascunhos pode ser acrescentada e é verificada explicitamente. Backup/restauro preservou objetivos, rascunho e10 artefactos assinados; autoridade histórica em quarentena, concessões suspensas e ativação futura sem grants.

A revisão de código encontrou uma leitura adicional na confirmação de receitas. Preparação e confirmação de Inventário/baseline agora validam só identidade/autoridade; contadores de teste por alvo comprovam0 invocações do coletor até à aprovação e1 operação através de efeito/verificação/replay. Uma operação baseline conserva as suas2 amostras internas. Health.observe antigo mantém o seu contrato de observações de prontidão. A compatibilidade de Saúde com múltiplos critérios anteriormente revistos foi preservada; Inventory novo exige exatamente o seu critério único. Domínios HMAC anteriores e bytes dos campos opcionais ausentes não mudam.

HostComponents v1 significa apenas distribuição efetiva e kernel em execução. Pacotes, processos, serviços, logs ou novas classes de fontes exigem novo âmbito/operação/concessão revista; não ampliar retroativamente este grant. NativeUser e TaskRuntime finito não autorizam envio destes dados a modelos, memória ou outros pets. Perfis/decisões QA sintéticos não são aceitação humana, inferência real, proatividade ou entrega instalada. [Composição, comandos e limites](../.build/pet-goal-execution-next-r1/guardian-inventory-runtime/VERIFIED.json).

## Arranque pedido de um plano finito de Sistema — contrato09/11/2026

Uma escolha explícita na admissão original pode pedir execução do plano exato quando houver recurso, com conversa/vista fechada e backend vivo. `ExecutionStart/nativeSystemPlan` integra o hash assinado da admissão; ausência conserva execução manual. Não é regra recorrente, autostart, serviço, autorização de modelos ou autonomia geral do pet. Usar o requested_dispatch/Driver/Owned/Authority e os mesmos recursos/receitas/intenções/orçamentos/recolha existentes, sem outro scheduler/ledger.

O Driver usa só configuração e alvo tipado do plano assinado, nunca resolve motor/conta/modelo nem chama inferência. O backend deriva apenas a receita correspondente à política/critério/alvo/concessão exatos já aprovados; não aprova um plano novo/gerado nem concede acesso. Som exige a encarnação original; Saúde/Inventário exigem pet/revisão/arranque/namespaces originais. Stop/prazo/revogação, recursos físicos, intenção antes do efeito e verificação do recibo conservam os contratos. Erro/ACK/efeito incerto não autoriza replay; manifestos não reconstroem Ready/autoridade.

A nova variante fechada é recusada por leitores antigos; campos opcionais ausentes conservam os formatos/bytes manuais anteriores. Restauro põe pedidos/autoridade históricos em quarentena e não os reativa. Sem modelos/quota real nos testes, efeitos de Som só no grafo virtual privado. Estado: implementado e integrado nas fontes1274; validado em componentes nativos com identidades/decisões sintéticas e no controlo funcional simulado. Tauri/dispatcher real, reinício/restauro do novo modo, leitor anterior e entrega pendentes. A candidata44 ficou congelada à parte e não inclui esta unidade; instalação43 preservada.
