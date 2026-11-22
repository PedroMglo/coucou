<!-- factory-domain-policy-20261113 -->
**Regra normativa atual do proprietário —13/11:** as funcionalidades pertencem
ao domínio do pet. A base comum não atribui chat ou modelos a todos os pets.
Os moldes existentes `llm`/`application` declaram o limite, que prevalece sobre
extensões de conta e é aplicado no backend. Serviços representam operações da
ferramenta; referências antigas a Codex e conversas antigas são dados preservados,
sem nova inferência. LLMs continuam a exigir adaptador/configuração explícitos.
Esta regra altera intencionalmente as descrições históricas de assistentes
Codex genéricos nos serviços abaixo. Ver [fábrica](PET-FACTORY.md#funcionalidades-por-domínio-13112026)
e [estado/provas atuais](PETS-UPSTREAM-PARITY-2026-11-11.md).
<!-- end-factory-domain-policy-20261113 -->

# Binding explícito de inferência por pet

A revisão `inferenceRevision` pertence ao pet e é emitida pelo backend em cada alteração nativa, incluindo limpar ou reaplicar a mesma escolha. Mantém-se no formato2 após a limpeza; não se herda nem se altera através do editor legado do documento inteiro. A confirmação humana liga pet, revisão original e alteração completa, com a origem nativa ainda válida imediatamente antes da gravação atómica. Capturas que observam uma alteração ou perda da fonte ficam retiradas; restaurar valores anteriores não as reativa. Uma configuração inválida ou incompleta não é convertida numa escolha legada. O perfil autoritativo é revalidado antes de efeitos e a projeção em memória usada na reserva evita IO dentro dos mutexes de chat/fila.

Os metadados são separados por motor, destino, modelo e contexto de autenticação: anónimo, credencial opcional legada ou credencial explicitamente exigida. Uma observação num contexto não confirma outro. A consulta passiva usa apenas configuração e evidência com prazo; a leitura ativa usa a mesma captura de configuração para o pedido e para a chave de cache. O registo aceita configuração2 sem indisponibilizar os restantes pets. O grupo R6 integrado localmente em F1 liga resposta por `taskRuntime`, memória de objetivo e planeamento limitado ao mesmo controlador, autoridade, orçamento, reserva física e transporte. A admissão usa a mesma projeção de configuração para rever o motor/modelo/destino; o transporte resolve a captura original depois de consultar recibos duráveis e nunca o servidor global. A memória mantém o dono lógico e a concessão identifica o adaptador que recebe os dados. Alteração durante IO retira a captura e impede sucesso tardio. A prova sintética cobre os dois sentidos Ollama/LM Studio, corpo real antes do hash/reserva, recibo sem novo efeito, revogação, receita aprovada/verificada e recusa sem efeitos; não substitui aceitação humana/nativa instalada. Ferramentas de outro dono e percursos de voz incompatíveis conservam a barreira de integração. [Checkpoint de integração R6 F1](../.build/pet-goal-execution-next-r1/integration1034-inference-binding-r6f1/CHECKPOINT.json): 48 ficheiros de código integrados a partir das fontes exatas comprovadas; instalação .32 inalterada. O restauro aceita estritamente os campos novos, preserva escolhas/revisões como dados e aplica apenas os grants da nova decisão de ativação. O leitor C4 anterior recusa formato2 sem sobrescrever os bytes. Provas nativas319/320/322 usam GTK/WebKit, protocolos e perfis privados sintéticos; não são aceitação humana/instalada.

Decisão normativa R6, em implementação. Preserva os43requisitos e GUA-01..06; não ativa o Guardian, novos acessos ou serviços. Usa a fábrica e o runtime existentes.

Um PetId e os canais lógicos da fábrica continuam donos de conversas, memória, sessões, objetivos e permissões. A escolha de motor é um dado próprio, opcional e completo: versão, adaptador suportado, endpoint explícito, modelo concreto e referência explícita de credencial. Partilhar motor/servidor não funde donos ou concessões. O papel funcional permanece independente.

A ausência da escolha conserva o percurso legado e os seus bytes. Uma escolha presente inválida/incompleta bloqueia a resolução; não seleciona o motor/modelo/endpoint global. Inicialmente são suportados os adaptadores locais Ollama/LM Studio já existentes, somente em pets cuja fábrica os declare. Outros bindings exigem adaptador operacional e contrato próprio. Contas Codex/Copilot/Gravity não mudam por herança. Metadados e estado são observações, não grants; uma consulta passiva não lê segredos, autentica, carrega modelos ou faz IO de inferência.

A escolha vive no próprio Pet em pets.json. Configuração1 sem o campo continua byte-compatível; a primeira escolha explícita requer configuração2. Leitores antigos recusam o novo dado em vez de o ignorarem. Limpar uma escolha é uma decisão explícita e conserva o formato2, o histórico e as preferências legadas. Não há migração/escrita automática dos perfis instalados. Backup/restauro transporta o dado e mantém as barreiras de autoridade/revalidação existentes.

O backend captura pet/canal, motor, endpoint, modelo, referência/revisão de credencial, política e recurso físico antes de esperar. O recurso identifica o servidor físico conservadoramente e é partilhado sem partilhar identidade. Trocar de seleção não redireciona trabalho em curso. O grant e o corpo real revistos têm de corresponder à escolha capturada. Revogação/alteração impede novos efeitos; resultados incertos e dados já entregues conservam a sua proveniência. Nenhum binding concede ferramentas locais: cada pet tem política própria, ausente significa desativada.

A gestão usa alteração de campo/revisão e origem nativa; uma gravação antiga de outras preferências não elimina uma escolha recente. O percurso manual e o de objetivos usam a mesma resolução, orçamento, transporte, persistência e exclusão existentes, sem outro scheduler ou ledger de autoridade. Metadados de modelo podem ser partilhados apenas quando destino e contexto de autenticação coincidem. Erro de um motor não apaga o pet ou o trabalho de outro.

Aceitação do grupo: dois donos explícitos no mesmo motor conservam dados/grants e disputam o mesmo recurso físico; também funcionam em motores separados. Confirmar destino original após espera/alteração, ausência de fallback, isolamento/revogação, compatibilidade/reinício/restauro e percurso funcional de gestão. Validar com providers sintéticos e testes dirigidos; provas de peças inalteradas são reutilizadas. Uma base de binding não torna Guardian/CLI/autonomia geral operacionais por presença no catálogo.

## Contenção de leitura antes do chat local interativo —12/11/2026

A observação da captura distingue `Current`, `Busy` e `Changed`. Um mutex
ocupado não prova mudança de destino e não retira a captura. A verificação
síncrona devolve um erro próprio de indisponibilidade temporária para `Busy`;
`Changed` conserva a retirada permanente e a recusa de redirecionamento.

Na preparação interativa do servidor e antes do envio do chat simples, o backend
pode aguardar essa validação de leitura por até seis segundos por validação,
com relógio monotónico e Stop/contexto verificados durante e após a espera.
Não segura mutexes entre observações. Conserva a captura, a identidade e a
reserva física originais; não escolhe outro provider/modelo/conta/endpoint.
Alteração observada, Stop ou prazo esgotado impedem esse envio.

A espera ocorre antes de qualquer pedido de inferência e não repete transporte,
aprovação ou contabilização. As barreiras existentes de preferências são
mantidas e verificadas de novo após a espera. A fence de dispatch continua
`FnOnce`: o runtime de objetivos conserva a sua política não bloqueante e a
chamada nativa única, sem renovar orçamento ou autoridade. Não há novo formato,
scheduler, ledger ou permissão. Esta correção não presume a causa exata de uma
recusa nativa intermitente anterior, nem altera todos os restantes pontos de
admissão/indisponibilidade das preferências.

No protocolo existente do telefone, contenção nativa de recurso/captura devolve
`busy` com mensagem fixa de indisponibilidade temporária; uma mudança de destino
capturado devolve recusa interna sanitizada. Não se apresentam estas recusas
como falha de rede. O código `busy` já existia; não há nova versão de protocolo.

## Recuperação do servidor OpenAI-compatible —12/11, aceitação nativa privada33

A comparação com o oficial Linux0.5.0 confirmou uma perda no modo de pets: a
ligação e a chave personalizada eram guardadas, mas o motor não podia ser
escolhido na conversa de um pet. O original completou a conversa pela API
sintética; o normal30 não ofereceu o servidor, apesar da ligação saudável.

A correção em validação reutiliza `Choice`/revisão/CAS e o transporte `custom`
existentes. A fábrica permite esse motor nos dois pets que já admitem escolha
Ollama/LM Studio; não cria outro pet, não altera a identidade nativa, não muda
contas Codex/Copilot/Gravity ou o Guardian, nem herda uma seleção. A referência
`configuredCustomEndpoint` exige a chave do servidor personalizado já vinculado
ao endereço exato; `none` não usa uma chave guardada implicitamente. A alteração
ou remoção da chave retira a captura anterior pela mesma geração de credencial
usada no LM Studio. Sem fallback de servidor, modelo, conta ou autenticação.

Tal como no original, o servidor compatível pode estar noutra máquina. A revisão
mostra destino e saída de dados, com aviso de HTTP remoto sem encriptação; uma
credencial remota só é admitida por HTTPS. Não se envia credencial de um endereço
a outro. A escolha concreta do modelo é conservada, não substituída pelo primeiro
modelo de um catálogo posterior.

O formato2 e a estrutura de `Choice1` mantêm-se, com novas variantes explícitas;
um leitor que não conhece `custom` deve recusar o perfil em vez de o regravar.
A recusa do executável anterior e backup/restauro serão verificados no percurso
nativo antes de declarar entrega. Perfis antigos sem esta escolha conservam bytes.

Isto recupera a conversa do original. Compatibilidade OpenAI não confirma
capacidades de ferramentas, contexto ou tarefas autónomas: o catálogo comum
continua a indicar suporte desconhecido ou indisponível onde falta prova, sem
tratar o servidor como Ollama/LM Studio. As permissões e concessões do pet
continuam independentes. Testes sintéticos/isolados, build e aceitação nativa
estão em curso; instalada43, publicação e quota real permanecem preservadas.


**Resultado posterior, Source33:** UI/HTTP sintético autenticado/código/reinício/
recusa após remover chave passaram; contrato de dono/permissões conservado.
Treze comandos CLI verificaram cópia/restauro inativo exatos, configuração e
zero grants. O leitor30 aceita revisão de backup opaco e recusa ativação da nova
escolha sem alterar dados. Fonte32 compilada e33 alinhada só por documento,
com falha original de igualdade conservada. `CUSTOM-SOURCE33-VERIFIED.json`.
Não certifica servidor remoto/TLS, ferramentas/voz/autonomia custom, janela do
restauro, experiência instalada, aceitação humana ou49 critérios integrais.
A decisão do perfil gerado é automatizada de QA, não consentimento humano.
