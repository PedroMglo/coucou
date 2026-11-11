# Método de desenvolvimento e entrega do Coucou

Aplicação pedida pelo utilizador em 8 de novembro de 2026, após análise da sessão
`01a1505c-e9e3-7bd2-87c0-fd9424be0538`. O agente dessa sessão deve adotar este
processo e continuar o objetivo integral em curso. Este documento orienta a
execução; a arquitetura e os requisitos vigentes continuam a definir o produto.

O resultado pretendido é reduzir o tempo entre implementação, integração e
utilização real, conservando a segurança, os dados e todos os critérios do sistema
completo. A unidade de progresso é um percurso funcional verificável e integrado.

## Objetivo e autorizações

- Preservar os 43 requisitos originais, os seis do Guardian e as decisões
  posteriores. Entregas incrementais não reduzem o objetivo integral a um MVP.
- Continuar com um único agente de desenvolvimento. A colaboração entre pets do
  produto continua no âmbito; não criar auxiliares para desenvolver ou rever.
- Preservar alterações de outras sessões, fontes, dados, preferências, hooks e
  assinaturas. Não executar reset, clean, stash automático ou substituições para
  alinhar a árvore com uma fotografia antiga.
- Respeitar as autorizações mais recentes da sessão. O desenvolvimento/testes
  diretos na máquina e a atualização local foram autorizados; uma versão histórica
  desta permissão não deve voltar a bloquear a mesma ação. Permanecem aplicáveis
  os limites de operações sensíveis, dados, quota e efeitos externos.
- Esta instrução não levanta a suspensão de commits, pushes ou merges do Coucou.
  Usar a autorização efetivamente vigente se houver indicação posterior.
- Alterações persistentes no Fedora ou na configuração global seguem
  `/home/pmglo/.codex/context/sysworkspace.md`, o AGENTS.md de sysworkspace e um
  commit focado nesse repositório, com hooks e assinatura ativos. Código,
  dependências e ambientes do projeto permanecem no Coucou. Esta fronteira deve
  acompanhar qualquer colaboração posteriormente autorizada.
- Manter a integração com DMS/Quickshell e as definições existentes do sistema,
  conforme a decisão de 7 de novembro; evitar voltar a criar painéis equivalentes.

## Uma entrega funcional em curso

Escolher o próximo percurso pela dependência real e pelo resultado utilizável.
Manter uma única entrega em implementação/integração; a lista de trabalho futura
não é trabalho em curso. Resolver dependências indispensáveis dentro da entrega.
Melhorias laterais ficam na lista futura, sem prolongar o critério de conclusão.

Cada percurso deve poder ser demonstrado do início ao resultado. Exemplo:
recuperar um objetivo delegado após reinício, conservar consumo e reservas,
permitir Pause/Cancel e impedir repetição de efeitos confirmados. Implementação,
interface funcional, persistência e prova integram a mesma entrega quando forem
necessárias a esse resultado.

Procurar unidades de algumas horas a dois dias focados. Esta referência serve
para decomposição; não é um prazo nem justifica atalhos. Se a unidade crescer,
identificar um resultado independente que possa ser integrado e delimitar o
restante. Não transformar pequenas alterações internas em dezenas de entregas
privadas sem integração.

Usar os grupos e IDs existentes em `PETS-DELIVERY-GROUPS.md` e respetivo JSON,
roadmap e matriz de requisitos. Não criar uma segunda taxonomia de requisitos.

Antes de editar, preparar uma ficha curta no estado/checkpoint já utilizado:

```text
Entrega e IDs relacionados:
Comportamento observável e limite do âmbito:
Base identificada e componentes afetadas:
Critérios de aceitação verificáveis:
Verificações necessárias e provas reutilizáveis:
Condição de integração/entrega e dependência seguinte:
```

Confirmar apenas o estado e as instruções relevantes à unidade. Evitar repetir
uma auditoria global, reler todo o histórico ou rever requisitos inalterados a
cada retoma. Registar uma lacuna que impeça a execução e continuar o trabalho
independente autorizado.

## Ciclo de execução e critérios de passagem

| Fase | Resultado necessário para avançar |
|---|---|
| Preparar | Base, âmbito, critérios, dependências e verificação inicial relevante identificados; falhas preexistentes distinguidas. |
| Implementar | Menor alteração coerente que satisfaz os critérios, reutilizando fábrica, runtime, autoridade e persistência existentes. |
| Verificar | Provas proporcionais ao risco, incluindo o percurso funcional e os casos de falha relevantes. |
| Integrar | Fontes comprovadas na árvore principal de desenvolvimento e regressões da composição afetada verificadas. |
| Preparar candidata | Fontes, ambiente, resultados e pacote ligados por identificadores verificáveis. |
| Instalar quando autorizado | Mesmo pacote validado ativado, versão anterior preservada e compatibilidade dos dados confirmada. |
| Aceitar e continuar | Resultado observado no âmbito exigido, estado atualizado e próxima dependência escolhida. |

Integrar cedo. Não esperar pela conclusão de todo um grupo para integrar um
percurso independente e coerente. Capacidades incompletas podem permanecer
desativadas com os mecanismos existentes, sem conceder permissões nem anunciar
disponibilidade que não possuem.

Se uma etapa de entrega depender de uma autorização realmente ausente, registar
o bloqueio e manter a candidata reproduzível. Esse bloqueio não impede unidades
independentes, mas não autoriza acumular variantes privadas da mesma entrega.
Uma falha técnica que afete o resultado prometido deve ser resolvida antes de
declarar a unidade concluída.

## Verificação proporcional ao risco

| Momento | Verificação |
|---|---|
| Durante a implementação | Testes dirigidos ao comportamento alterado e verificações rápidas relevantes de tipos, sintaxe ou contratos. |
| Na integração | Regressões das componentes afetadas e percurso completo na aplicação, incluindo os casos críticos de falha da unidade. |
| Na candidata à entrega | Suite geral aplicável sobre as fontes finais, pacote, arranque e compatibilidade/migração/restauro quando afetados. |
| Após instalação | Versão e executável ativos, arranque, preservação dos dados e funcionamento do percurso entregue. |

Não repetir suites gerais ou gerar outro pacote por uma alteração pequena sem
risco transversal identificado. Alterações a autoridade, formatos persistentes,
recuperação e fronteiras entre componentes podem exigir cobertura mais ampla;
explicar brevemente o motivo concreto. Não diminuir essa cobertura por pressão de
prazo.

Reutilizar uma prova apenas quando fontes e pressupostos relevantes continuam
válidos. Identificar origem da prova, âmbito e condição de validade. Uma mudança
posterior invalida as provas afetadas; não apresentar uma execução anterior como
verificação da composição final.

Cada teste novo deve distinguir um comportamento exigido ou reproduzir uma falha
relevante. Evitar testes que apenas espelham a implementação. Identificar testes
ignorados e o impacto na aceitação. Se falhar o próprio ensaio, corrigir o ensaio
e repetir o conjunto afetado, reutilizando o binário se este não mudou.

Preferir fixtures e protocolos sintéticos para a repetição. Hardware, áudio,
compositor e providers reais exigem prova no ambiente adequado quando o critério
o requer. Não consumir quota real Copilot/GitHub. Outras chamadas reais respeitam
as autorizações e orçamento existentes, preferindo a opção gratuita ou económica
adequada. Simulação não comprova compatibilidade com um serviço real.

## Composição e entrega reproduzíveis

Reutilizar os comandos e mecanismos de build/instalação existentes. Automatizar
um ritual repetido quando reduzir preparação manual comprovada; não transformar
a adoção deste método num projeto de tooling ou numa nova framework de QA.

Cada candidata deve identificar: commit base; diferenças locais e ficheiros
necessários quando não houver commit da alteração; versões/lockfiles e ambiente;
comandos/resultados; pacote e hash; versão ativa/anterior; limites de migração e
recuperação. Proteger dados privados e manter capturas fora do Git.

Enquanto os commits do Coucou estiverem suspensos, usar os mecanismos locais de
composição e manifesto já existentes para identificar as fontes exatas. Uma
fotografia local não é um commit nem uma publicação remota. Não criar um sistema
paralelo de snapshots, e não fazer commits para contornar a suspensão.

Construir a candidata identificada e promover o mesmo pacote validado até à
instalação. Se mudar uma entrada da build, identificar uma candidata nova e
revalidar o que perdeu validade. Uma alteração só documental deve ser distinguida
das entradas que alteram o executável.

Na máquina real, usar perfis de desenvolvimento para ensaios repetidos e os
dispositivos reais quando necessários. Nas interações gráficas, seguir
`pet-desktop status`, `pet-desktop windows` e o alvo exato; observar antes e depois
de agir. Preferir CLI/API/acessibilidade verificáveis quando disponíveis.

A instalação segue o prefixo versionado e o procedimento sysworkspace existentes.
Preservar a versão anterior; confirmar a compatibilidade dos dados antes de
prometer rollback. Recuar o executável não desfaz efeitos externos nem garante
que um leitor antigo aceite dados novos.

## Estado, documentação e continuidade

Usar o estado atual já existente como ponto de entrada. Ao fechar uma entrega,
atualizar o resumo e os critérios/IDs afetados e ligar as provas detalhadas.
Atualizar contratos e arquitetura quando a decisão ou comportamento mudar;
evitar repetir a mesma narrativa em todos os documentos por cada microcorreção.
Preservar evidência histórica e alterações alheias.

O checkpoint deve permitir retomar com: base/composição, resultado já integrado,
provas válidas e seus limites, bloqueio exato e próximo comando ou ação útil.
Manter esse resumo curto e atual. Uma compactação não é motivo para refazer a
auditoria ou os testes já comprovados e ainda válidos.

Reportar progresso por percurso: em curso, verificado no âmbito, integrado,
candidato, instalado e aceite. Distinguir prova sintética, nativa automatizada,
hardware/provider real e aceitação instalada. Número de testes, catálogo ou
documentação não substituem o resultado funcional.

Registar nas entregas, sem nova infraestrutura de métricas: início, integração,
instalação, esperas por autorização/serviços, retrabalho e principais verificações.
Acompanhar tempo até integração/instalação e quantidade/idade de incrementos
privados. Só estimar marcos após algumas entregas comparáveis, considerando
dependências e incerteza. Não inventar percentagens globais.

## Aplicação imediata nesta sessão

1. Ler este método e confirmar o ponto real em curso, preservando o trabalho.
2. Se a atualização incremental já em preparação ainda estiver pendente, concluir
   essa entrega: recuperação, Pause/Cancel, consumo/reservas/autorizações,
   compatibilidade, pacote e instalação autorizada. Reutilizar provas válidas;
   acrescentar somente o que faltar para aceitar a composição final. A indicação
   histórica de build 32/33 não obriga a voltar a uma versão anterior.
3. Se a atualização já estiver entregue, escolher a próxima dependência real.
   Consolidar os percursos de autonomia e cooperação já construídos antes de
   aumentar a variedade de mecanismos ou níveis privados.
4. Prosseguir pelas dependências vigentes dos providers, memória/fontes, rotinas,
   Sistema, voz e Guardian. Nenhum bloco é retirado do objetivo.
5. Confirmar à sessão que este método foi adotado, indicar a entrega corrente,
   critérios de saída e o próximo passo concreto; continuar autonomamente sem
   esperar por nova confirmação de autorizações já concedidas.

O objetivo global só fica concluído quando todos os critérios aplicáveis do
sistema integral estiverem implementados, integrados, verificados e entregues,
com pendências e exceções explicitamente resolvidas.

## Referências

- [Tarefas delimitadas e evidência de aceitação no Codex](https://learn.chatgpt.com/use-cases/issue-to-tested-pr).
- [Entregas pequenas e frequentes](https://dora.dev/capabilities/working-in-small-batches/).
- [Integração contínua e builds reproduzíveis](https://dora.dev/capabilities/continuous-integration/).
