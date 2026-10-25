# Objetivo integral e decisões de execução

Revisão de **25/10/2026**. A secção seguinte conserva literalmente o objetivo persistente fornecido pelo utilizador. Não é uma declaração de conclusão. O estado e os limites estão em [ESTADO-E-PLANO.md](ESTADO-E-PLANO.md).

## Texto literal

Concluir o sistema completo de Pets do fork Coucou como assistentes digitais de IA persistentes, autónomos, cooperativos e sempre disponíveis enquanto a infraestrutura permitir, capazes de trabalhar de forma proativa e autorizada no dia a dia.

PRIORIDADE PRINCIPAL: concluir a arquitetura coerente, robusta, verificável e extensível já em desenvolvimento. Preservar os 43 requisitos originais, todos os pedidos posteriores, as decisões existentes e o trabalho realizado. Seguir docs/PETS-ARCHITECTURE.md, docs/PETS-ROADMAP.md, docs/PET-CAPABILITIES-CONTRACT.md e a restante documentação vigente. Não reiniciar, substituir ou reestruturar componentes funcionais sem necessidade demonstrada.

ARQUITETURA MULTIAGENTE: todos os Pets existentes e futuros devem ser entidades autónomas de primeira classe, incluindo Codex, Gravity, Copilot CLI, Ollama e outros adaptadores configuráveis. Cada Pet deve poder ser chamado diretamente pelo utilizador, manter identidade, capacidades, contexto, sessões, memória e objetivos, receber e executar tarefas, desenvolver iniciativas autorizadas e colaborar com outros Pets, respeitando as capacidades reais do respetivo fornecedor.

COMUNICAÇÃO E COOPERAÇÃO: preservar e evoluir a comunicação direta entre Pets, sem obrigar todas as interações a passar por um coordenador. Implementar ou consolidar um protocolo comum para descoberta de capacidades, mensagens, pedidos, respostas, delegação, eventos, resultados e acompanhamento de tarefas. Suportar colaboração paralela e assíncrona, correlação de mensagens, rastreabilidade, controlo de concorrência, prevenção de ciclos, conflitos, duplicações e delegações ilimitadas. Preferir reutilizar mecanismos existentes em vez de introduzir infraestrutura redundante.

COORDENAÇÃO: permitir que o Pet Ollama assuma preferencialmente o papel de coordenador global, com visão dos objetivos, prioridades, estados e capacidades dos restantes Pets, sem se tornar intermediário obrigatório, dependência central ou ponto único de falha. A coordenação deve ser uma capacidade funcional configurável, separada do motor de inferência e da infraestrutura de controlo. Outros Pets devem continuar operacionais quando o coordenador, Ollama ou qualquer fornecedor estiver indisponível.

RUNTIME AUTÓNOMO: separar conceptualmente a identidade/interface visual de cada Pet, a sua execução persistente e os respetivos motores de IA. Permitir que os Pets permaneçam disponíveis para receber eventos, gerir tarefas, comunicar e executar operações determinísticas sem manter modelos locais carregados ou processos de inferência permanentemente ativos. Invocar Ollama, Codex, Copilot ou outros motores apenas quando necessário e suportado. Preservar tarefas e estados através de encerramentos da interface, reinícios e falhas, dentro das limitações reais do ambiente e dos fornecedores.

AUTONOMIA CONTROLADA: completar objetivos/tarefas/filas duráveis, agendamento, eventos, proatividade configurável, execução e verificação, recuperação de falhas, memória individual e partilhada, documentos, fontes pessoais, sessões e decisões nativas, regras, utilização, voz e notificações. Garantir que tarefas iniciadas diretamente num Pet e tarefas delegadas são acompanhadas pelo mesmo sistema de controlo, sem retirar independência aos agentes.

SEGURANÇA E GOVERNANÇA: manter uma matriz de permissões de Sistema por Pet/subplugin/operação, aprovações nativas, isolamento adequado, auditoria, limites de recursos, custos, concorrência e profundidade de delegação. Nenhum Pet pode obter permissões adicionais apenas por receber uma delegação de outro. Operações destrutivas, alterações sensíveis, envios externos e ações irreversíveis devem respeitar as autorizações explícitas aplicáveis. Preservar dados, preferências, credenciais, permissões e Wi-Fi.

IMPLEMENTAÇÃO: avaliar primeiro a arquitetura real e os componentes existentes, distinguindo capacidades implementadas de funcionalidades apenas documentadas. Preferir integração incremental, modular, testável e compatível com o sistema atual. A arquitetura deve permitir acrescentar novos Pets, motores, ferramentas e integrações sem exigir alterações extensivas aos restantes. PostgreSQL permanece opcional e exige benefício demonstrado, sem acesso às bases de dados de outros projetos.

ÂMBITO E RESTRIÇÕES: descontinuar temporariamente apenas o Pet GitHub enquanto entidade, conservando os seus dados; Gravity e Copilot CLI permanecem no âmbito. Trabalhar como único agente de desenvolvimento desta intervenção, sem criar novos agentes de desenvolvimento auxiliares e sem novos commits, pushes ou merges. Esta restrição não impede implementar ou testar a colaboração entre Pets do produto. Frontend/UX limitado a bloqueadores funcionais, aprovações, preservação e entrega. Não consumir quota real de GitHub CLI/Copilot em testes; utilizar alternativas gratuitas adequadas ou, quando estritamente necessário e autorizado, a opção mais económica. Respeitar as instruções da estação e registar alterações persistentes do anfitrião no sysworkspace.

ORDEM DE EXECUÇÃO: primeiro consolidar a arquitetura e contratos; depois concluir identidade, capacidades, sessões, persistência, autonomia e execução individual dos Pets; em seguida garantir comunicação, cooperação, delegação e coordenação resilientes; depois completar as integrações e capacidades funcionais pendentes. Ajustar esta sequência às dependências reais e ao estado já implementado, sem refazer trabalho concluído.

CONTINUIDADE E CONCLUSÃO: continuar autonomamente após cada incremento, atualizar documentação, arquitetura e evidências, identificar a próxima unidade útil e executá-la dentro das permissões disponíveis. Não declarar o objetivo concluído até todos os requisitos aplicáveis estarem implementados, integrados, testados, verificados no âmbito correto e entregues, com exceções e pendências explicitamente resolvidas. Testes verdes isolados, protótipos privados ou documentação sem implementação não bastam para declarar o sistema completo.

## Pedidos posteriores e contexto a preservar

O pedido desta revisão exige a pasta `coucou-migration` com a arquitetura desejada completa dos pets e de todo o sistema, documentação detalhada para análise e continuação do desenvolvimento depois da sua criação. Não substitui os 43 requisitos nem reduz o objetivo ao backend do incremento atual.

Continuam vigentes: prioridade da arquitetura; pets de vários providers; fábrica comum; independência de Gravity/Copilot/Codex e serviços; voz integral; permissões alinhadas com os agentes nativos; anexos múltiplos; fontes/memória com âmbito; Sistema por pet/subplugin/operação e partilha de componentes. Só a entidade GitHub está temporariamente retirada, com dados preservados.

As instruções mais recentes mantêm um único agente de desenvolvimento, sem novos commits, pushes ou merges. A colaboração entre pets do produto faz parte do objetivo. Frontend/UX está limitado a bloqueadores funcionais, aprovações, preservação e entrega. Ensaios usam gratuito primeiro, sem quota real Copilot/GitHub. PostgreSQL local é uma possibilidade a avaliar, não uma integração ativa nem autorização para consultar bases de outros projetos.

Wi-Fi, instalação, preferências, credenciais e dados pessoais são preservados. Alterações persistentes do anfitrião seguem sysworkspace; dados/ambientes do projeto ficam no projeto. Esta documentação não altera as permissões da estação.

## Exceção posterior para publicação deste dossier

O utilizador autorizou enviar **apenas `coucou-migration` ao remoto** e pediu o link da pasta. A autorização abrange o commit/push necessário deste dossier numa branch de documentação. Não abrange o código, outras alterações locais ou merge. O texto literal anterior é preservado como foi recebido.

## Proveniência

O literal também está preservado em [fontes/OBJETIVO-LITERAL.txt](fontes/OBJETIVO-LITERAL.txt), SHA-256 `33709fce53a7fb98571a86cf5fecbf7dbb278ad824eca0f4ec450113aac9f377`. O objetivo e o histórico de execução do repositório estão na [cópia integral de PETS-GOAL.md](fontes/docs/PETS-GOAL.md). O estado do controlo de Goals da aplicação é independente da existência destes ficheiros; criar documentação não o conclui ou reativa automaticamente.
