# Consulta de progresso por projeto — 17/11/2026

Atualização corrente: a consulta inclui agora as ocorrências de acompanhamento
sem notas opcionais, corrigidas e validadas no binário privado `0a3e9482…`.
[Correção, provas e limites](#ocorrências-do-acompanhamento-sem-notas-opcionais).
O relato inicial abaixo conserva a composição1342 e a prova histórica próprias;
a recorrência posterior está documentada em [acompanhamento](PETS-PROJECT-FOLLOWUP-2026-11-17.md).

A gestão do projeto tem agora uma consulta explícita dos seus objetivos,
critérios, provas e respostas. Está integrada nas fontes1342, com aceitação
nativa privada no binário `9a83c8cb96be9d9b44c43799fd7c87a6ea0a27cf44a454c498a7fa5e4fab4488`.
É uma compilação de desenvolvimento; não é o RPM49 congelado nem a instalada44.

O backend acrescenta à listagem existente apenas ID/revisão do projeto
explicitamente capturado no contexto do objetivo, ou `null` quando não existe.
Não infere ligação pelo nome, pasta ou conversa coincidente. Não altera o formato
persistido dos objetivos, o catálogo ou a autoridade.

Depois de consultar o catálogo e escolher o projeto, o utilizador abre
“Objetivos e progresso deste projeto” e escolhe “Consultar objetivos do projeto”.
Cada objetivo permite consultar o seu estado/provas e, separadamente, o recibo
da resposta. Usa os leitores existentes de Goal/History/Result. Não há IO
passivo, inferência, novo ledger, scheduler, percentagem ou conclusão agregada.
O leitor da resposta é partilhado com a supervisão existente; a vista de progresso
não instancia o controlador de ações de execução.

A apresentação distingue estado na listagem, observação das provas, critérios
originais, revisão capturada/atual do projeto, associação retirada e ausência de
referência de conclusão nativa. Uma resposta guardada não verifica os critérios.
Mostra contagens autenticadas e resultados desconhecidos com o seu âmbito. No
histórico restaurado, apresenta a contabilidade posterior como desconhecida;
um subtotal vazio não é descrito como consumo zero ou saldo recuperado.

Apenas os quatro leitores necessários passaram a aceitar a janela do chat, e
somente para o pet que essa janela mostra. Leitura de todos os pets ou de outro
pet é recusada. Mutação, revisão de autoridade e execução conservam a lista
anterior de janelas autorizadas. Arquivar o projeto mantém os dados consultáveis.
Alterar o projeto/seleção invalida os resultados apresentados; respostas tardias,
registos de outro dono/projeto, revisão de objetivo alterada e listagens sem
metadados de projeto são recusados. Listagens incompletas ficam identificadas.

A validação nativa na composição atual criou um objetivo com resposta finita,
um com retirada de associação durante HTTP e um controlo legado do mesmo pet.
O painel incluiu os dois objetivos explicitamente ligados e excluiu o legado.
A resposta foi lida pelo recibo existente; o segundo objetivo conservou uma
chamada, zero ferramentas,65536 bytes debitados e um resultado desconhecido.
As consultas não repetiram inferência nem alteraram os bytes do runtime.

Arquivar→consultar→reiniciar passou. Backup/restauro nativos→painel histórico→
resposta→reinício também passou, com `grants:[]`. Um ensaio adicional apenas de
leitura tornou visível o débito desconhecido, sem repetir os turnos. Três
inferências sintéticas estabeleceram o cenário; a consulta/restauro/reinícios
acrescentaram zero. Não houve inferência real/paga ou uso de contas pessoais.

[Recibo agregado e composição](../.build/project-progress-20261117/VERIFIED.json),
[delta de produto preservado](../.build/project-progress-20261117/PRODUCT-DELTA.json),
[testes web/tipos](../.build/project-progress-20261117/web2/RESULT.json),
[testes Rust](../.build/project-progress-20261117/rust1/RESULT.json),
[preservação da instalação](../.build/project-progress-20261117/PRESERVATION.json).
Os86 testes web dirigidos incluem os leitores antigos afetados; os7 Rust usam
exatamente os mesmos825 inputs nativos da compilação atual. Não se repetiu a
suite integral sem delta que o justificasse. Os caminhos privados não são Git.

As capturas observadas permitem ler resposta, revisão, incerteza e débito em
posições diferentes do painel com scroll; o topo de cada recorte pode estar fora
da imagem. Não são aceitação gráfica global, humana, de hardware ou instalada.
A instalada44, launcher, preferências e RPM49 conservam os hashes anteriores.
Commits, pushes e merges continuam suspensos.

Esta entrega é consulta explícita de progresso. O requisito integral AUTO-04
continua aberto: rotina prolongada, atualização governada do contexto do projeto,
partilha/autorização próprias e recorrência ainda exigem integração e prova.
Segue esse percurso com scheduler, orçamento, autoridade e domínios existentes,
sem promover o consentimento manual ou de um objetivo para autorização futura.


## Ocorrências do acompanhamento sem notas opcionais

Correção delimitada integrada e validada em17/11, depois da entrega
[PETS-PROJECT-FOLLOWUP](PETS-PROJECT-FOLLOWUP-2026-11-17.md). A listagem usa
ContextBinding::project(), incluindo a fotografia própria de ContextBinding12.
A vista antiga exigia memory/Selection2, embora o novo acompanhamento aceite
notas vazias. Consequência: o objetivo aparecia na lista do projeto mas a consulta
dizia incorretamente que não lhe pertencia e escondia o botão da resposta.

O leitor da vista usa agora o projeto explícito do contexto12. Quando há notas,
a sua fotografia tem de coincidir integralmente; continuam obrigatórios o pet,
ID do projeto e revisão capturada da listagem. O comprovativo apresenta o projeto
correto também sem notas. Notas2 e leitores anteriores mantêm o comportamento.
Não há alteração de backend, formato, autoridade, scheduler, ledger ou execução.
É uma falha da integração nova do fork, não uma regressão atribuída ao oficial.

Provas: os7 testes anteriores passaram;3 testes adicionais falharam antes da
correção e passaram depois.24 web dirigidos e TypeScript, incluindo gestão de
projetos e escolha de acompanhamento, passaram na fonte Source1/1357. O backend
continua exatamente nos829 inputs NATIVE-SOURCE18 da entrega anterior; não se
repetiram testes Rust sem delta nativo. O novo binário privado é
`0a3e948239c54ac2ee10d2c635c4a05b3c02b23b87de805e29b435670fa28774`.

A recusa foi reproduzida no binário anterior14036407/native-before3. O ensaio
native-after2 usa a aplicação Tauri/WebKit atual, IPC real e o MESMO perfil QA
anteriormente gerado, nos seus caminhos admitidos. Consultou as três ocorrências
sem notas: duas respostas exatas e uma tentativa desconhecida com1 chamada,
65536B debitados e incerteza. Reinício e nova consulta passaram, sem inferência,
HTTP, nova concessão, execução ou alteração dos bytes de configuração/runtime.
A cópia do perfil foi recusada pela admissão; a segunda tentativa também tinha
raízes de dados diferentes. A tentativa after1 parou no diretório temporário do
Podman bloqueado pelo sandbox; after2 executou o mesmo teste autorizado com o
runtime temporário disponível. Nenhuma política foi alterada. Diagnósticos são
preservados, não aceitação. As imagens X11 BEFORE/AFTER foram observadas: erro
anterior e resposta/débito atuais legíveis em recortes com scroll.

Recibos privados: `.build/project-progress-followup-20261117/VERIFIED.json`,
`web-red/RESULT.json`, `web1/RESULT.json`, `app-build1/RESULT.json`,
`.build/upstream-parity-20261111/project-progress-followup-before3/RESULT.json`
e `project-progress-followup-after2/RESULT.json`. Source2 acrescenta apenas esta
documentação à fonte Source1 executada. Não é um pacote normal ou instalação.
Instalada44, launcher, preferências e RPM49 continuam exatos. Os49 critérios
originais permanecem intactos e abertos no seu âmbito integral.

A recorrência/observação próprias já têm prova na entrega de acompanhamento;
pedido de decisão governado, uso prolongado, restauro nativo da composição nova,
entrega instalada e aceitação humana continuam etapas próprias. Esta leitura
não concede execução, aprova uma sugestão do modelo ou verifica critérios livres.
