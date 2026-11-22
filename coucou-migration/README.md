> **Passagem vigente — 11/11/2026:** [Handoff integral para continuação e revisão independente](HANDOFF-CONTINUIDADE-INTEGRAL-2026-11-11.md). Contém objetivo, arquitetura, 49 critérios, código/estado/provas, limites, método e runbook. Instalada 43 preservada; privada 58 compilada, aceitação 6/7 gráfica/instalada pendente. Esta publicação de código/documentação foi autorizada sem assinatura Git por comando; depois dela o trabalho para até novas instruções. As fotografias anteriores abaixo permanecem históricas.

<!-- revoked-rule-intents-v1 -->
> **Continuidade local 11/11/2026:** instalada43 inalterada; privada56 valida consolidação de intenções revogadas, revisão e preservação do formato5. Próximo: sucessão governada com saldo restante. [Estado canónico vigente](../docs/PETS-IMPLEMENTATION-STATUS.md). O dossier remoto de25/10 e as notas abaixo são históricos; sem novo push.

> **Continuidade local09/11/2026:** instalada43 preservada. Medição curta e inventário OS/kernel manual/TaskRuntime integrados e validados em privada35, ainda não instalados. Próximo: candidata normal44. [Fontes/provas locais vigentes](../.build/pet-goal-execution-next-r1/guardian-inventory-runtime/VERIFIED.json). Dossier remoto25/10 histórico; sem novo push.






> **Nota local de continuidade — 09/11/2026:** instalação42 ativada tecnicamente, com pacote/fontes1261 preservados e tarefas nativas finitas de Som. Esta nota local não altera a publicação documental histórica nem declara49 requisitos completos. Aceitação visual instalada e registo assinado do anfitrião continuam pendentes. Estado canónico e limites: [PETS-IMPLEMENTATION-STATUS](../docs/PETS-IMPLEMENTATION-STATUS.md) e [prova42](../.build/pet-goal-execution-next-r1/release42-native/VERIFIED.json).

# Coucou Migration — dossier de revisão do fork

**Revisão posterior de 07/11/2026:** a UI de Sistema passa a ser a existente na estação; não construir definições próprias duplicadas nem retirar `controlCenterButton`. Mantêm-se controlo pelos pets, grants seletivos e todos os requisitos. [Decisão e impacto](REVISAO-SISTEMA-20261107.md). A restante fotografia histórica conserva a data/âmbito originais.

Revisão de **25 de outubro de 2026**. Este dossier reúne a arquitetura desejada de **todo o sistema**, o objetivo integral, os requisitos de aceitação, o estado observado e o plano de evolução. Destina-se à revisão pelo utilizador enquanto o desenvolvimento continua.

Local: `/home/pmglo/Projects/coucou-codex/coucou-migration`. Branch: `feat/mochi-cli-accounts`. A pasta é documentação de revisão; criá-la não migra dados, não altera a aplicação instalada e não implica reescrever o produto.

**Extensão local posterior:** o pedido seguinte acrescenta System Guardian e volta a suspender toda a publicação. Ler [REVISAO-GUARDIAN-20261025.md](REVISAO-GUARDIAN-20261025.md) para os novos contratos e estado. O corpo original e as fontes abaixo conservam a fotografia publicada.

## Por onde começar

1. [ARQUITETURA-COMPLETA.md](ARQUITETURA-COMPLETA.md): visão integral do produto, componentes, contratos, funcionamento e critérios de conclusão. É o documento principal para leitura seguida.
2. [OBJETIVO-INTEGRAL.md](OBJETIVO-INTEGRAL.md): objetivo literal e decisões posteriores que condicionam o trabalho.
3. [PETS-E-CONTRATOS.md](PETS-E-CONTRATOS.md): fábrica, identidades, catálogo dos 25 pets, adaptadores, sessões e permissões nativas.
4. [SISTEMA-E-PERMISSOES.md](SISTEMA-E-PERMISSOES.md): catálogo das componentes do computador e matriz por pet, subplugin, operação e recurso.
5. [DADOS-MEMORIA-E-FONTES.md](DADOS-MEMORIA-E-FONTES.md): conversas, anexos, memória, calendários, dados pessoais e PostgreSQL.
6. [REQUISITOS-E-ACEITACAO.md](REQUISITOS-E-ACEITACAO.md): os 43 requisitos originais, dependências, critérios e ligação aos capítulos deste dossier.
7. [ESTADO-E-PLANO.md](ESTADO-E-PLANO.md): o que existe, o que foi apenas testado em privado e o que falta desenvolver ou entregar.
8. [MIGRACAO-E-ENTREGA.md](MIGRACAO-E-ENTREGA.md): compatibilidade, dados antigos, backup, adoção, instalação e rollback.
9. [DECISOES-PARA-REVISAO.md](DECISOES-PARA-REVISAO.md): decisões estabelecidas e pontos cujo desenho ainda precisa de ser fechado.

## Como interpretar

**Arquitetura desejada** descreve o resultado a construir. **Integrado localmente** significa código adotado na árvore de trabalho, podendo ainda não estar instalado. **Privado** significa incremento isolado, com as provas e limites indicados. **Entregue** exige aceitação na composição instalada. A existência de um desenho, pet no catálogo ou teste positivo não equivale a todas essas etapas.

Todos os 43 requisitos continuam abertos nos seus critérios integrais. O texto não reduz o objetivo às funcionalidades que já passam nos ensaios atuais. Melhorias visuais estão adiadas; os controlos funcionais de autorização e gestão continuam necessários.

Publicação autorizada posteriormente pelo utilizador: exclusivamente esta pasta, numa branch de documentação. A suspensão de commits do código mantém-se.

## Fontes e manutenção

As [fontes integrais](fontes/README.md) preservam os documentos normativos, históricos e upstream consultados. O [manifesto](fontes/MANIFESTO.json) identifica origem e SHA-256 de cada cópia; não contém perfis, conversas ou credenciais pessoais. Algumas fontes são históricas ou específicas de macOS e não certificam capacidades atuais de Linux.

A arquitetura de execução continua mantida em [docs/PETS-ARCHITECTURE.md](../docs/PETS-ARCHITECTURE.md), com o [roadmap](../docs/PETS-ROADMAP.md) e o [estado](../docs/PETS-IMPLEMENTATION-STATUS.md). Este dossier é uma fotografia consolidada para revisão, não uma segunda implementação nem uma prioridade concorrente. Alterações aprovadas na revisão devem ser refletidas nos contratos canónicos antes de implementar algo incompatível. A data e o estado desta fotografia não se atualizam sozinhos quando o desenvolvimento avança.

Há um [registo da verificação documental](VERIFICACAO.md) e um [índice das fontes](fontes/README.md) para localizar os detalhes preservados.
