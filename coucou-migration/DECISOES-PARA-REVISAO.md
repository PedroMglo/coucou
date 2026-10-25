# Decisões estabelecidas e pontos para revisão

Fotografia de **25/10/2026**. Esta lista ajuda a analisar o desenho completo. Não é uma sequência de confirmações obrigatórias nem interrompe o desenvolvimento já autorizado. Decisões incompatíveis com os contratos vigentes precisam de revisão documentada antes da implementação.

## 1. Decisões já estabelecidas pelo utilizador

| Tema | Decisão vigente |
| --- | --- |
| Finalidade | Assistentes digitais de IA persistentes, autónomos, proativos e cooperativos para o dia a dia. |
| Prioridade | Arquitetura antes de tudo; depois pets e capacidades conforme dependências. Frontend/UX só para funcionalidade, decisão, preservação ou entrega. |
| Identidade | Todos os pets existentes/futuros são entidades próprias; não desenvolver apenas Codex. |
| Coordenação | Ollama é preferência configurável; não é intermediário obrigatório nem ponto único de falha. |
| Comunicação | Chamada direta e cooperação paralela/assíncrona entre pets, com limites e autoridade próprios. |
| Fábrica | Reutilizar a framework comum; propagar correções sem propagar permissões ou fundir dados. |
| Sistema | Plugin por pet, subplugins/operações/recursos seletivos, partilháveis entre pets. |
| Organização | Rede/Wi-Fi e Bluetooth pertencem à mesma unidade de desenvolvimento. |
| Voz | Canal integral do sistema e das suas decisões, com os mesmos contratos de chat/terminal. |
| Agentes nativos | Preservar ferramentas/MCP, sessões, sandbox e pedidos de permissão originais nos percursos suportados. |
| Dados | Vários anexos por conversa, memória e fontes com âmbito/proveniência/revisões, migração e preservação. |
| GitHub | Só a entidade/pet GitHub está temporariamente retirada; conservar dados. Gravity e Copilot CLI continuam. |
| Ensaios | Gratuito primeiro, depois o mais económico adequado se necessário/autorizado; sem quota real Copilot/GitHub agora. |
| Desenvolvimento | Um único agente; commits/pushes/merges de código suspensos. Exceção posterior: publicar apenas este dossier numa branch de documentação, sem merge. |
| Anfitrião | Preservar Wi-Fi, dados, credenciais e instalação; alterações persistentes seguem sysworkspace. |
| Revisão | Manter este dossier completo enquanto o desenvolvimento prossegue. |

## 2. Desenhos de implementação ainda por fechar

| Questão | Direção atual e alternativas | Como decidir |
| --- | --- | --- |
| Runtime fora do processo desktop | Evoluir o backend existente. Um serviço separado pode permitir continuidade após saída total da UI, mas acrescenta lifecycle e entrega. | Demonstrar a necessidade, definir dono/login/stop/recovery e provar a continuidade sem dois writers. |
| PostgreSQL | Local versionado continua suficiente para a fase atual; PostgreSQL é opcional. | Medir pesquisa, volume e concorrência; avaliar operação offline, migração e backup/restauro. Base/role próprios sempre. |
| Memória derivada | Itens com fontes/revisões e invalidação explícita. Ainda falta fechar políticas de episódios, resumos e projetos. | Definir cobertura, sensibilidade, retenção, correção e teste de exclusão/invalidação antes de automatizar. |
| Memória partilhada | Partilha por item/coleção e destinatário/âmbito; sem contexto global implícito. | Demonstrar necessidade de colaboração e mínimo contexto; provar revogação e destino de dados. |
| Pesquisa vetorial | Não é pré-requisito; começar por filtros/pesquisa lexical. | Comparar qualidade/latência com corpus autorizado e custo de embeddings/índices; concessão própria para serviço externo. |
| Coleções de fontes | Evoluir pet/CID para objetivo/regra/ocorrência sem alargar inscrições antigas. | Fechar schema, consentimento multicalendário, revisão, falha parcial e revogação. |
| Regras/agendamento | Reutilizar runtime/autoridade; política explícita para suspensão, offline e DST. | Definir skip/coalesce/decisão, limites, fuso e cenários de deduplicação antes da rotina diária. |
| Mensagens entre pets | Envelope comum e entrega durável com mecanismos existentes. | Escolher schema e ACK/recovery com base no runtime atual; demonstrar se é necessária alguma fila adicional. |
| Autonomia por CLI | Preservar manual; só admitir autonomia onde limites internos e efeitos sejam controláveis. | Aceitar por adaptador os limites de tools, custo, tempo, consumo, decisões e recuperação; não usar flags permissivas como solução. |
| Superfície de dados | Seleção/revisão funcional mínima comum aos pets, distinguindo turno/sessão/objetivo. | O utilizador consegue rever dados/destino/âmbito e recusar sem consumo; sem redesign cosmético. |
| Sistema por plataforma | Serviços/executores existentes com operações tipadas e capacidades reais. | Inventariar handlers, autenticação, alvos e efeitos no anfitrião; não extrapolar protótipo para todas as plataformas. |
| Voz completa | Intenções tipadas sobre os mesmos contratos; microfone/escuta contínua configuráveis. | Matriz de operações, decisões inequívocas, Stop e aceitação PT/EN/misto com áudio real. |
| Extensões móveis | Preservar protocolos existentes; novas capacidades não são expostas por herança. | Definir autorização por dispositivo/operador/capacidade, versão e revogação antes de expor dados/controlo adicionais. |

## 3. Critérios para rever uma proposta

Uma alteração deve explicar o problema concreto, o benefício, os contratos afetados, dados/autoridade e migração, alternativas rejeitadas por evidência e aceitação. A arquitetura deve permitir acrescentar pets, ferramentas e fontes sem reescrever os restantes.

As opções não devem trocar a escolha do utilizador silenciosamente, ativar acesso por conveniência ou tratar um conteúdo de modelo como política. Reaproveitar componentes úteis é preferível quando satisfazem o contrato. Introduzir infraestrutura nova exige benefício demonstrado.

## 4. Como incorporar a revisão

As observações podem referir capítulo/ficheiro ou ID de requisito. Depois de uma decisão, atualizar a arquitetura canónica, contratos, roadmap e estado/evidências afetados, mantendo o objetivo e o histórico. Atualizar também este dossier ou emitir nova revisão com data; não deixar duas versões a contradizer-se sem indicação.

Enquanto não houver decisão incompatível, o trabalho continua na próxima unidade autorizada: consentimento e envio de memória em objetivos. A existência destes pontos em aberto não apaga o âmbito completo nem justifica anunciar capacidades como prontas.
