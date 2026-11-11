# System Guardian — especialização do runtime comum

Contrato acrescentado pelo utilizador em **25/10/2026**. Extensão GUA-01 a GUA-06 dos 43 requisitos originais. Estado: primeira unidade manual de Saúde implementada e validada em composição privada; a instalação42 ainda não a contém. A especialização integral conserva GUA-01 a GUA-06 e todas as dependências de autonomia/preservação.

## 1. Identidade, motor, runtime e papel

PetId estável **`agent_system_guardian`**, sem renomear identidades existentes. Papel funcional: **especialista em saúde, segurança, vulnerabilidades, manutenção e desempenho do computador**. A fábrica em desenvolvimento tem26 identidades, incluindo o Guardian com observação manual de Saúde. A instalação42 mantém o catálogo25; a contagem não certifica autonomia ou o cumprimento integral de GUA.

O primeiro binding de inferência permitido é **Ollama pelo adaptador comum**, com modelo e servidor explicitamente configurados para o Guardian. Não copiar a seleção do coordenador `ai_ollama`, não carregar modelo ao consultar estado e não fazer fallback de conta/modelo/provider/endpoint. Um binding incompleto mantém a inferência indisponível, preservando operações determinísticas autorizadas. Outros bindings suportados podem ser acrescentados pelo mesmo contrato, sem criar outro runtime.

Separar quatro elementos: pet com dados próprios; runtime determinístico comum; motor de inferência invocado quando necessário; papel funcional independente do provider/privilégios. Coordenação preferencial de `ai_ollama` é configuração do papel, não autoridade global nem requisito de todas as mensagens.

O Guardian usa fábrica, configuração, descritores, repositórios, broker, decisões, recursos, objetivos, mensagens, scheduler e notificações do Coucou. A mesh é lógica sobre o transporte comum. Não implica all-to-all físico, processo/modelo residente por pet, novo ledger de autoridade, broker externo ou stack AI Local/Symbiont.

A base R6 já separa dono lógico de motor e a fábrica aceita um LLM sem adaptador base com motores locais explicitamente permitidos. Não duplicar essa resolução nem emprestar o PetId/contexto do coordenador. Em09/11, a lacuna de identidade sem motor em `Owner::from_runtime_descriptor` foi corrigida pela resolução comum `Blueprint::runtime_identity_adapter`: um LLM configurável sem adaptador base conserva o namespace lógico antes de escolher um modelo. Fábrica dinâmica, donos nativos e runtime determinístico foram verificados com fixtures; a prova de um novo Guardian operacional continua pendente. Esta identidade não configura inferência, autoriza efeitos ou concede acesso ao Sistema. A matriz comum de [concessões de Sistema](PET-SYSTEM-ACCESS-CONTRACT.md) já admite Som por NativeUser e tarefas finitas pelo TaskRuntime original. Saúde está implementada para NativeUser; TaskRuntime de Saúde e ProviderModel permanecem fechados. Esta distinção não certifica a especialização integral do Guardian. PET-01/04/05, preservação e aceitação por caller continuam aplicáveis.

## 2. Autoridade e modelo de ameaça

O Guardian analisa, mede e propõe. O runtime autentica o caller, valida concessões, admite e executa nos brokers apropriados. Nem o especialista nem o coordenador aprovam os próprios planos, concedem acesso, alteram política ou elevam privilégios através de texto gerado.

Logs, ficheiros, advisories, memória, páginas e mensagens de pets são dados não fiáveis. Um finding ou recomendação não é autorização de execução. O acesso é seletivo por componente/operação/recurso, distingue observação de remediação e conserva decisões nativas. Não existe shell/root geral para o Guardian.

O isolamento entre pets no processo partilhado é lógico; não é sandbox de código arbitrário. Código e plugins não fiáveis exigem contenção efetiva antes de executar. Um helper privilegiado, se demonstradamente necessário, é mínimo, tipado e sujeito à autorização nativa; nunca expõe uma shell genérica.

Não fazer varrimento indiscriminado de HOME, `.config`, `.codex`, perfis autenticados, bases de outros projetos ou logs pessoais. Limitar quantidade, bytes, duração, intervalo temporal, campos e retenção antes de materializar conteúdo. Segredos permanecem nos mecanismos próprios e fora de prompts, logs, argumentos de processo e Git.

## 3. GUA-01 — inventário e evidência

Consultas limitadas e autorizadas recolhem componentes/versões, serviços/processos, exposição, recursos, logs relevantes e estado desejado. Cada observação tem alvo, origem, instante, cobertura, frescura e limites, com resultado desconhecido/indisponível explícito.

Classificar recursos como geridos por sysworkspace, observados, externos, privados ou dinâmicos. Um recurso observado não passa a estado desejado. Uma lista de processos não prova posse, autorização de controlo ou atividade de um turno. O conteúdo necessário para diagnóstico é minimizado antes de entrar num modelo.

Aceitação: produzir inventário/finding verificável sem LLM e sem tocar dados fora da seleção; mostrar falha parcial e diferenças entre gerido/observado. Consultar estado não autentica contas, carrega modelos ou inicia scans intrusivos.

## 4. GUA-02 — segurança e vulnerabilidades

Findings estruturados guardam ID/revisão, componente e versão efetiva, evidência, origem/data, severidade, confiança, estado e recomendação. Distinguir vulnerabilidade confirmada, correspondência candidata e configuração suspeita. A severidade da fonte e a avaliação local devem conservar as suas origens.

Conferir advisories adequados à distribuição e pacote, incluindo release/epoch e correções distribuídas quando aplicáveis. Comparar apenas versão upstream não certifica vulnerabilidade. Não inventar CVEs nem converter ausência de informação em «seguro». Consultas externas exigem destino/operação autorizados e limites; dados locais enviados são mínimos e revistos.

Não executar exploração, scans intrusivos ou alterações de VPN/firewall para provar utilidade. Aceitar fixtures de versão corrigida pela distribuição, advisory desatualizado, correspondência ambígua, fonte indisponível e conteúdo hostil. A prova de um parser não é confirmação de uma vulnerabilidade real na estação.

## 5. GUA-03 — saúde e desempenho

Medir baseline e resultado: CPU, memória, IO, GPU quando suportada, arranque, falhas e conflitos. A observação é amostrada/limitada e identifica janela, unidade, cobertura e incerteza. Recolher métricas não autoriza encerrar ou reconfigurar serviços.

Uma otimização explicita hipótese, métrica de sucesso, impacto esperado, custo, pré-condições e validação antes/depois. Não aplicar tuning genérico, limpeza destrutiva ou desativação de proteções para melhorar números. Ruído de medição e workloads concorrentes não se apresentam como melhoria causada pela alteração.

Fixtures obrigatórias: consumo anómalo de um serviço; falhas intermitentes de dock Thunderbolt; possíveis conflitos Proton VPN/Tailscale/firewall. São cenários de diagnóstico, sem autorização para alterar rede, drivers, dock ou sessão reais.

## 6. GUA-04 — integração sysworkspace

Descobrir e usar os contratos reais do sysworkspace. A inspeção inicial confirmou `scripts/workstation.py`, gestores HOME/pacotes, manifestos e transações documentados no seu AGENTS.md. Não inventar subcomandos, flags ou promessa de rollback: ler o contrato efetivo da operação antes de a implementar no adaptador.

Comparar estado real com o estado desejado efetivamente gerido. Respeitar âmbitos de projetos, dados privados, estado dinâmico e exclusões de Coucou/Mochi e outras integrações. As entradas e políticas existentes de preservação não são removidas pelo Guardian. Uma proposta para integrar algo excluído é uma decisão separada, não uma reconciliação automática.

Intervenções persistentes do anfitrião percorrem plano/diff, validação, backup, autorização, aplicação, verificação e recuperação no gestor existente. A configuração interna e o código do Coucou permanecem no Coucou. Esta intervenção de desenvolvimento não aplica esses planos, altera manifestos do anfitrião ou publica commits; as restrições atuais mantêm-se.

Workers passivos não pedem sudo/autenticação nem leem arquivos root para completar uma consulta. Uma remediação autorizada conserva token, recurso e política nativos e não cria uma fonte concorrente do estado desejado.

## 7. GUA-05 — remediação governada

O percurso completo a implementar é **finding → proposta/diff → impacto e pré-condições → autorização → execução limitada → verificação → resultado**. O primeiro modo operacional é observação/recomendação; o requisito integral inclui a remediação autorizada e não termina num relatório.

Cada proposta tem operações tipadas, recursos/alvos/revisões, âmbito, prazo, orçamento, evidência, pós-condições e capacidade de recuperação. Antes do efeito, revalidar estado, concessão, dados, argumentos e autorização exata. Alteração de alvo ou argumentos após consentimento exige outra revisão.

Receitas específicas de baixo risco podem ser automáticas sob grants explícitos e limitados. Alterações sensíveis preservam aprovação exata e autenticação nativa. O Guardian não altera as suas próprias permissões. A falha depois de um possível efeito conserva incerteza; não repete nem executa uma ação oposta como compensação improvisada.

Rollback só é anunciado quando tecnicamente suportado e provado. Compensação e irreversibilidade são classificações distintas. Restaurar ficheiros não desfaz efeitos externos. Aceitar uma remediação permitida completa, outra recusada por falta de autoridade e uma falha parcial sem replay.

## 8. GUA-06 — proatividade e cooperação

Gatilhos/eventos e inspeções leves usam regras, ocorrências, scheduler, recursos, orçamento, notificações e recuperação comuns. Limitar frequência, debounce, deduplicação e silêncio; não criar outro daemon de inferência contínua ou scheduler paralelo.

Invocar LLM apenas quando acrescenta valor. Findings e verificações determinísticas continuam possíveis com o modelo descarregado. O Guardian pode receber pedidos diretos, cooperar com pets autorizados e pedir priorização ao coordenador. A ausência do coordenador não impede operações independentes enquanto o backend comum estiver disponível.

Delegação exige autorização do remetente e concessões/capacidades próprias do destinatário; não permite contornar uma recusa. Contexto mínimo, destino de dados, orçamento repartido, prazo, correlação e causalidade são os do protocolo TEAM. Não copiar históricos inteiros. Uma resposta de outro pet é dado, não uma aprovação.

Aceitar eventos repetidos, ramos concorrentes, ausência de LLM/coordenador, backpressure, ciclos, revogação e interrupção sem alertas duplicados ou efeitos repetidos.

## 9. Dependências e sequência

Fechar R43 e os bloqueadores de preservação/execução individual. Consolidar o binding de dono lógico separado de motor/papel na fábrica; implementar mesh e os contratos necessários de fontes/regras/Sistema. O Guardian usa essas bases com módulos separados para inventário, segurança, saúde e remediação. Não criar obrigatoriamente pets Network, Performance ou DevOps antes de benefício demonstrado.

Cada incremento atravessa contrato, backend, controlo funcional mínimo, teste, evidência e integração. Não anunciar o pet como operacional por o acrescentar a um JSON. A aceitação instalada e efeitos reais dependem de autorização específica e não se simulam por uma fixture.

## 10. Relações de requisitos

| ID | Dependências principais | Resultado integral |
| --- | --- | --- |
| GUA-01 | PET-01/04/05, EXE-01/02, OBS-01, SYS-01 | Inventário/evidência limitada, com classificação de gestão e percurso determinístico utilizável. |
| GUA-02 | GUA-01, CTX-05, EXE-02/03, DATA-01 | Findings/advisories com classificação e prova adequadas, sem CVEs ou exploração inventados. |
| GUA-03 | GUA-01, OBS-01/02, SYS-03 | Métricas, baseline, hipóteses e verificação de saúde/desempenho. |
| GUA-04 | GUA-01, EXE-01/02/03, DATA-01, REL-01 | Adaptador governado aos gestores reais e preservação de exclusões/âmbitos. |
| GUA-05 | GUA-01/04, RUN, EXE, SYS-01, REL-01/02 | Remediação limitada com autorização, efeito verificado e recuperação/recusa corretas. |
| GUA-06 | GUA-01/05, AUTO, TEAM, OBS | Proatividade/cooperacão resilientes no runtime comum. |

Os 43 IDs anteriores conservam todos os critérios. A [auditoria rastreável](PETS-REQUIREMENTS-AUDIT-2026-10-25.md) discrimina estado, código, provas, limites e próxima aceitação dos 49 requisitos.


## 11. Integração de Sistema revista em 07/11/2026

O Guardian é o especialista previsto para ser o responsável principal pelas componentes de Sistema que o utilizador lhe atribuir. Essa designação pode persistir entre tarefas, mas não concede permissões, exclusividade, prioridade absoluta, remediação automática ou autostart. As reservas de execução concretas são temporárias e o controlo humano mantém-se independente. Outros pets podem ter grants próprios sobre as mesmas componentes, sem depender do Guardian ou do coordenador.

O Guardian usa os serviços e definições/widgets já existentes, pelos adaptadores/brokers comuns e grants próprios. Não cria outro centro de controlo, fonte de configuração ou infraestrutura privilegiada. Diagnóstico/proposta, remediação governada, acesso humano independente e alterações persistentes via sysworkspace conservam os seus contratos. Ver [PET-SYSTEM-INTEGRATION.md](PET-SYSTEM-INTEGRATION.md).

## 12. Primeira unidade operacional de saúde (09/11/2026, em implementação)

O Guardian entra pela fábrica como agent_system_guardian, dono lógico LLM sem adaptador base e com Ollama explicitamente permitido. Servidor/modelo/credencial têm de ser escolhidos no editor existente; nenhuma seleção é copiada de ai_ollama e a observação determinística não invoca modelos.

A revisão opcional do catálogo separa perfis legados25 de perfis que já reconheceram26 identidades. Apenas identidades com revisão de introdução posterior podem ser acrescentadas em memória, sem permissões, motor, ferramentas ou tarefas. Preferências anteriores não são substituídas. Um perfil que já reconheceu uma identidade e depois a perdeu é recusado; leitores anteriores recusam o campo novo sem o destruir. A projeção pets.default.json continua gerada da fábrica.

Saúde/Observar/HostHealth é uma concessão independente de Som, pelo mesmo writer e captura revogável. A primeira consulta nativa é manual, limitada a contadores procfs fixos de CPU, memória e uptime. Não enumera processos, nomes, logs, ficheiros pessoais, sockets, pacotes ou dados de projetos. Devolve origem, cobertura, falha parcial e limites de amostragem, com finding determinístico apenas quando a métrica observada o suporta. CPU é contador cumulativo; uma amostra não certifica utilização ou melhoria.

A gestão existente permite rever esta concessão e consultar o resultado por pet. TaskRuntime e ProviderModel não recebem esta capacidade nesta unidade; a integração autónoma seguirá a admissão comum, sem novo scheduler ou ledger. GUA-01/03 ficam parciais: inventário alargado, segurança/advisories, baseline temporal, sysworkspace, remediação e proatividade mantêm todos os critérios originais.



## 13. Provas e limites da integração manual (09/11/2026)

A build privada29 provém das fontes1266; os controlos de concessão/consulta e o runtime/draft/reinício foram verificados na mesma build em Tauri/WebKit nativo. O validador fechado do restauro passou16 casos dirigidos e o percurso CLI real, incluindo um rascunho Guardian não vazio. O leitor42 foi iniciado apenas sobre perfil gerado e recusou catalogRevision. Configuração, objetivos, todos os registos de rascunho e recibos anteriores foram preservados; foi observada e verificada apenas uma migração vazia compatível de rascunhos, pelo que não se declara imutabilidade de todos os bytes desse livro. O upgrade nativo25→26 preservou os25 perfis e os bytes antigos até uma gravação explícita.

O restauro conservou os bytes do rascunho e suspendeu os grants de ambos os pets. Como não havia consentimento autónomo original, foi usado o percurso existente de preparação independente de autoridade futura, com revisão sintética própria; não se renovou autoridade histórica. A navegação nativa conserva o último destino até o frontend estar pronto, e respostas tardias não substituem navegação mais recente.

Provas, comandos, tentativas parciais, composição e limites em [VERIFIED](../.build/pet-goal-execution-next-r1/guardian-health-inventory/VERIFIED.json). Não é aceitação humana/instalada, inventário completo, utilização de CPU por percentagem, diagnóstico de vulnerabilidades, remediação ou proatividade. Segue a admissão de Saúde no runtime comum antes de regras/cooperação autónoma; não criar outro scheduler/ledger.

## 14. Admissão autónoma de Saúde — em implementação09/11/2026

system.health.observe1 recebe apenas um objeto vazio e lê os mesmos três contadores limitados da consulta manual. A operação passa pelo Store, Review, Authority, fila, Owned, recursos, passos, receitas e recuperação comuns; não criar outro scheduler ou ledger. O dono, a revisão Health/Observe/HostHealth e a identidade kernel/namespace do anfitrião são capturados antes do hash/reserva. Mudança de alvo ou política é recusa, nunca redirecionamento.

A primeira admissão é finita, dedicada, com um passo e orçamento RuntimeOnly/zero modelos, sem contexto de conversa, memória selecionada, dados recebidos ou inferência. O predicado nativo certifica recolha válida e limitada dos três contadores; não certifica que o computador esteja seguro/saudável ou que uma otimização tenha resultado. A mesma amostragem respeita autoridade original, cancelamento, prazo e revalidação antes da publicação.

A família Health distingue Policy2, Admission28, Goal63, Step5, Recipe6, Criteria7 e Verification4. A família Sound conserva os bytes/versões anteriores. Grant48 continua o mesmo formato/domínio e vincula o hash semântico exato da política, incluindo o recurso de Saúde; não se concede autoridade por JSON, caller metadata ou restauração. Exigir revisão humana e critérios completos antes do hash, intenção/budget antes de IO, recibo assinado e verificação para concluir. ProviderModel, geração/delegação e execução automática de Saúde permanecem fechados nesta unidade.

### Prova privada31 e limites da unidade (09/11/2026)

Guardian e Ollama completaram admissão/claim/receita/critério assinado via Tauri real, em perfil gerado e com decisões QA explícitas, sem motor escolhido, POST de inferência, quota paga ou controlo de áudio. Não é aceitação humana nem entrega instalada. Contextos dedicados passaram a reconhecer o namespace lógico declarado pela fábrica, sem declarar um adaptador de inferência operacional.

O coletor é comum. A janela manual continua250ms; tarefas têm janela limitada2s incluindo verificações nativas de autoridade antes/depois das leituras e antes da publicação. O limite8835bytes refere-se aos3 contadores por amostra; identidade usa ainda boot_id limitado64bytes e metadados dos2 namespaces fixos. Preparação de receita e verificação podem observar amostras adicionais, também autorizadas e limitadas; o orçamento conta o passo de ferramenta, não cada chamada interna de prontidão/verificação. Não confundir percentagem de CPU com ticks cumulativos. Sem novos processos/serviços persistentes.

Step5/Recipe6/Verification4 têm domínios HMAC próprios de Saúde; Grant48 conserva o formato/domínio original e vincula o hash semântico da política exata. Famílias anteriores de Som não mudam. A execução pública31, reinício e leitor42 foram comprovados; backup/restauro63/48 também passou em11 comandos CLI nativos: objetivos concluídos preservados, autoridade histórica em quarentena, concessões desativadas e ativação futura sem novas concessões. A composição/entrega continua pendente. [Prova própria](../.build/pet-goal-execution-next-r1/guardian-health-preservation/VERIFIED.json). GUA-01/03 continuam parciais, restantes módulos Guardian e49 critérios integrais abertos. Evidências: `.build/pet-goal-execution-next-r1/guardian-health-runtime/VERIFIED.json`.


Entrega incremental43 (09/11): mesmas bases nativas31 com versões alinhadas, pacote normal43 comprovado e instalado, seleção/grants/modelos preservados. Guardian disponível desligado;26 entradas de catálogo não significam26 ativos/operacionais. Prova automatizada e registo assinado do anfitrião pendente; não fecha GUA integral. [Composição, preservação, instalação e limites](../.build/pet-goal-execution-next-r1/release43-native/VERIFIED.json).

## Medição curta de desempenho — integrada e validada em privada32,09/11/2026

GUA-03 começa por `system.health.baseline`v1, argumentos `{}`, duas amostras dos mesmos3 contadores, separadas por pelo menos1s, com limite total6s. CPU é estimativa agregada pelos deltas dos contadores Linux, sem dupla contagem de guest; reset, delta nulo/incoerente, origem/revisão alterada ou intervalo incompatível produzem estado indeterminado, nunca0% ou saúde presumida. Memória conserva valores antes/depois e só deriva diferença quando o total é consistente. Não cobre IO/GPU/processos, diagnóstico ou tuning.

A operação reutiliza Health/Observe/HostHealth por pet e a identidade capturada antes do hash. Espera/IO/publicação revalidam a autoridade original. Admissão e intenção/orçamento duráveis antecedem as duas amostras; prontidão verifica apenas identidade e autoridade. A verificação usa o recibo assinado da medição original, sem medir outra vez para substituir a evidência. NativeUser e o plano finito TaskRuntime mantêm revisão própria; ProviderModel, geração/delegação e arranque automático continuam fechados. O formato novo deve ser recusado pelo leitor43 sem perda de dados. Sem alterações na instalação43 ou grants implícitos. Estado: implementada na raiz e validada na composição privada32; não entregue na instalação43.

A métrica de CPU é a fração agregada de tempo reportado fora de idle/iowait. O kernel documenta que iowait pode ser impreciso e diminuir; a medição trata deltas incoerentes como indeterminados e não atribui causa a processos: https://docs.kernel.org/filesystems/proc.html#miscellaneous-kernel-statistics-in-proc-stat .

A prova privada32 abrange o botão nativo de medição, dois objetivos Guardian/Ollama sem modelo, igualdade da evidência original com a verificação assinada, reinício, revogação independente e replay recusado. O leitor43 recusa os novos objetivos sem destruir registos anteriores; acrescentou apenas uma importação legada vazia verificada. Backup/restauro preservou os dois objetivos, rascunho não vazio e10 artefactos de passos/receitas/verificações, mantendo autoridade histórica em quarentena e ativação futura sem grants. Testes dirigidos de contadores/espera, runtime, Som partilhado e controlos passaram. Tudo usa perfis gerados e consentimento QA sintético; não é aceitação humana, inferência real ou entrega instalada. [Provas exatas](../.build/pet-goal-execution-next-r1/guardian-health-baseline/VERIFIED.json).

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
