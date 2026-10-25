# Migração, compatibilidade e entrega

Plano desejado de **25/10/2026**. Documenta o percurso necessário para levar o código validado à aplicação sem perder dados ou capacidades. Não executa uma migração nem autoriza substituir a instalação durante a criação do dossier.

## 1. Evolução do fork e fronteiras

O destino é evoluir o fork existente com componentes modulares e contratos comuns. A pasta `coucou-migration` não representa decisão de criar outro produto, multirepo ou stack. Reaproveitar código útil, testar incompatibilidades concretas e integrar unidades coerentes.

O projeto gere código, dependências locais, lockfiles, containers e dados de ensaio. Alterações persistentes do Fedora, aplicações globais, sessão, serviços ou configuração pessoal pertencem ao sysworkspace e seguem as instruções da estação. Diagnóstico sem alteração não exige uma intervenção de sistema.

Commits/pushes/merges de código estão suspensos. A exceção posterior do utilizador autoriza publicar exclusivamente `coucou-migration` numa branch de documentação, sem merge. Esta restrição prevalece sobre instruções históricas de integração. O plano de migração/entrega do código não é executado por essa publicação. Hooks e assinatura YubiKey permanecem preservados quando commits voltarem a ser autorizados.

## 2. Inventário antes da adoção

Identificar a árvore/branch, fontes exatas, snapshot de build, executável e assets; perfil e raízes de dados; helpers, atalhos, autostart e processos escritores; conversas/arquivos, rascunhos, anexos/cópias, fontes, memórias, runtime/autoridade, regras e segredos referenciados.

Preservar também as definições dos pets retirados, preferências de voz/modelo, configurações das integrações, indicadores e acesso às funções legadas. Não exportar recursivamente HOME, `.codex` ou `.config`. O inventário revisto contém referências/mecanismos sem tokens ou perfis autenticados.

## 3. Adoção de código validado

Uma unidade é aplicada a partir das fontes exatas verificadas, com pré-imagens e controlo de alterações concorrentes. Não sobrescrever edições de outra origem. Ficheiros experimentais não registados não entram no build por estarem perto dos módulos ativos.

As provas distinguem código compilado, assets, fixtures e binário. Uma mudança em Rust pode reutilizar prova web apenas se os inputs web forem os mesmos. Timeout de observação não prova que um runner terminou. Não modificar o SDK enquanto o runner original ainda utiliza os inputs; consumir o seu resultado original antes de reutilizar fontes.

A composição adotada recebe check/build/testes apropriados e percursos de preservação/regressão afetados. Não repetir suites sem mudança ou preocupação pendente, nem usar quantidade de ensaios como substituto da aceitação funcional.

## 4. Compatibilidade de formatos

| Área | Regra de migração |
| --- | --- |
| Configuração/fábrica | Preservar personalizações e provider vazio antigo; defaults novos não concedem acessos. |
| Conversas e anexos | Preservar IDs, mensagens, arquivos, contexto e cópias; coleção nova não apaga o anexo único antigo. |
| Retenção/recuperação | Não limpar origem com finalização parcial, nome reutilizado ou referências desconhecidas. |
| Memória e fontes | Conservar dono, âmbito, proveniência, revisões, validade e concessões; não ampliar grants por conversa a todo o pet. |
| Objetivos e autoridade | Conservar critérios, consumo, admissões, claims, recibos e incerteza; não lançar trabalho ao converter. |
| CLI | Preservar associação nativa e política; formato novo não pode perder possível entrega de contexto. |
| Regras | Migração não ativa uma regra, recupera aprovação perdida ou duplica ocorrência. |

Os schemas são versionados e, quando viável, condicionais ao uso da funcionalidade. No incremento privado atual, metadados de memória CLI requerem conversa versão7; seleção em objetivos requer contexto2/objetivo11. Leitores antigos têm de recusar esses formatos sem escrita destrutiva. Formatos antigos sem campos novos preservam o comportamento/bytes nos percursos demonstrados.

Os ensaios de leitor anterior do incremento privado não certificam todos os leitores históricos. A instalação só pode regressar a um leitor compatível ou a um backup/restauro coerente. Um downgrade do executável sozinho não é rollback de dados.

## 5. Backup consistente e writers

Antes de uma migração real, tratar workers/decisões e identificar todos os escritores. Dois destinos/perfis que apontam para dados partilhados não podem ser tratados como cópias independentes. Parar ou coordenar os writers conforme o protocolo de migração e capturar estado consistente, incluindo rascunhos não enviados.

O backup privado inclui os repositórios relacionados e cópias necessárias, permissões, versão, manifesto e hashes. Segredos ficam no mecanismo original ou no procedimento de backup autorizado; não são promovidos a Git. Testar restauro em ambiente isolado antes de confiar no arquivo.

A importação usa estágios limitados, destino e posse verificados, exclusão de escritores, validação antes de publicação e marcadores recuperáveis. Nome de diretório, ficheiro existente ou caminho dentro de HOME não provam posse. Corrupção, versão desconhecida ou origem incompleta são preservadas com motivo de recusa.

## 6. Aceitação anterior à ativação

Usar perfis privados e dados sintéticos para: primeiro arranque; migração sem inferência; seleção de perfil; criação/arquivo; anexos; memória/fontes; decisões; objetivos interrompidos; Stop/recolha; concorrência; falha de gravação; publicação parcial; reinício e downgrade recusado. Confirmar que o perfil da instalação e a app operacional não foram usados pelo ensaio.

Modelos de teste gratuitos são a primeira escolha. Protocolos sintéticos cobrem Copilot/GitHub sem quota real. Uma fixture não é aceitação da conta operacional. Ensaios de voz exigem prova de áudio real para fechar VOICE-02; testes não autorizam alterar Wi-Fi/dispositivos.

## 7. Entrega instalada

A entrega deve associar executável, assets, helpers e configuração correta. Verificar arranque, acesso à ilha/pets, ferramentas, credenciais referenciadas, atalhos, autostart/login e comportamento ao fechar janelas. O backend só continua fora da UI enquanto o processo/serviço escolhido estiver operacional.

Aceitar também suspensão, reinício, perda de provider, hotplug de áudio/rede, trabalhos e decisões interrompidos, falha de persistência e recuperação. Confirmar o estado real após as ações, não apenas o exit code do instalador ou um ACK.

A eventual intervenção persistente na instalação/global/session deve ter fontes/procedimento revistos em sysworkspace e seguir a autorização e restrição de commits aplicáveis. O desenvolvimento do repositório não altera silenciosamente a app do utilizador.

## 8. Rollback e efeitos externos

Antes de ativar, definir os gatilhos de rollback, binário anterior, backup compatível e writers a parar. Conservar a composição nova para diagnóstico privado sem expor conteúdo. Não misturar backup antigo com journal/autoridade novos como se formassem uma transação única.

Restaurar dados não desfaz efeitos externos, envios ou comandos já executados. O registo de efeitos/recibos e incerteza tem de ser reconciliado para não reexecutar uma ação após o restauro. O orçamento antigo e as aprovações não são recriados por conveniência.

## 9. Condições para retirar funcionalidades antigas

Inventariar o que está ativo, mapear os percursos equivalentes e aceitar suporte efetivo e acesso do utilizador. Preservar os sete widgets/indicadores até às condições específicas da [arquitetura de Sistema](SISTEMA-E-PERMISSOES.md). Não apagar entidades/dados antigos porque uma nova superfície deixou de os mostrar.

Recursos visuais históricos entram no inventário de preservação; não ganham prioridade de redesign. A conclusão REL-01/02 exige composição e aceitação integrada, além de testes privados de um módulo.
