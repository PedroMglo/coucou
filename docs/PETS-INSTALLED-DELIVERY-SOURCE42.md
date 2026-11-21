> **Atualização de 14/11 — proposta em espera.** A continuação encontrou e
> reproduziu uma saída móvel que ainda publicava o pet GitHub retirado a partir
> de cache/configuração antiga. A correção está na unidade
> `.build/retired-mobile-20261114/`, fora deste RPM42. A revisão abaixo permanece
> identificada e preservada; aguardar o fecho desta unidade e a composição que
> a inclua antes de substituir a instalação. A continuação automática não
> constitui aprovação da instalação nem de commits.

# Entrega proposta — Coucou 0.5.1-mochi.47 / Source42

Data: 14/11/2026. **Preparada; ainda não instalada.** A composição operacional
indicada é 0.5.1-mochi.44/Source35. Esta proposta substitui a proposta45/37,
cujo pacote não contém as correções posteriores. Commits do fork, pushes e
merges permanecem suspensos.

## O que muda

- Mantém a correção da classificação Busy no chat móvel e a conservação do
  cartão n8n quando se seleciona uma sessão antiga do assistente.
- Aplica a decisão do proprietário na fábrica: aplicações/serviços apresentam
  as funções próprias; os pets LLM conservam conversa e modelos aplicáveis.
- Retira também o controlo genérico de sessões do cabeçalho dos serviços.
- Preserva os dados e escolhas existentes. Sem novo formato, concessão,
  dependência, provider ou modelo introduzido por esta entrega.

As provas antigas de voz, regras6/7, recursos e restantes componentes conservam
as fontes e limites originais. Não são promovidas a aceitação integral42.

## Pacote fixado

- Identificador: `fr.louisraille.coucou`; versão `0.5.1-mochi.47`.
- RPM: `.build/release-checkpoints/Coucou-0.5.1-mochi.47-native-826415f41af83241.rpm`.
- SHA-256 RPM: `826415f41af83241b3829e5f30258cd75de10eaa0f2ae1853e9bc9bd85c719bf`.
- SHA-256 executável: `7c1485fa5af3fd520285c5927a5a17ea5f1e41e47a3397ab525813c6257b22c0`.
- Manifesto: `.build/upstream-parity-20261111/SOURCE42-DOMAIN-HEADER-RELEASE47.json`.
- 1323 inputs; hash do arquivo de fontes e ambiente na receita preparada.
- Instalador existente limitado aos 13 payloads revistos; relay preservado.

## Evidência e limites

102 testes frontend e TypeScript; 64 testes Rust dirigidos reutilizados por
igualdade dos fontes de backend. Build nativa e verificações do pacote já
existentes foram reutilizadas. Não se executou uma suite geral nova para esta
proposta.

A aceitação nativa é composta por dois ensaios do mesmo executável:

1. Primeiro ensaio: cartão n8n legível, cabeçalho sem sessões/chat/modelos,
   recusas nativas, workflow/erro/retirada por HTTP/TLS e um turno Codex sintético.
   Falhou na leitura imediata do snapshot após reinício; o resultado continua
   exit1 e não é apresentado como execução integral verde.
2. Probe dirigido: publicação real antes/depois de reinício e preservação dos
   IDs/mensagens Codex/n8n, sem novo turno de inferência. O evento de serviço
   chegou 105,22 ms e 159,53 ms após pong; pong não sincroniza o renderer.

Um probe intermédio teve timeout no interruptor. Não foi reproduzido no último
ensaio nem declarado corrigido. Sem Android físico, aceitação humana/instalada,
quota real ou conclusão dos 49 critérios. Processos privados encerrados.

Recibos locais: `FACTORY-DOMAIN42-NORMAL47-VERIFIED.json` e
`PUBLICATION-RESTART42-VERIFIED.json` em `.build/upstream-parity-20261111/`.
Capturas privadas não são publicadas.

## Alterações da estação propostas

O patch permanece dentro do projeto, em
`.build/upstream-parity-20261111/installed-source42-proposal/`.
O dry-run do patch, os JSON e a sintaxe do launcher passaram. As quatro fontes
atuais do sysworkspace foram verificadas e preservadas:

- `domains/desktop/recipes/coucou-fork.json`: versão, fontes e provas.
- `domains/desktop/home/.local/bin/coucou`: apontar para47.
- `domains/desktop/COUCOU.md`: procedimento e limites.
- `manifests/applications.json`: referência da composição.

A aplicação proposta usa o instalador restrito e o gestor HOME existentes,
preserva os prefixos44/43 e altera apenas o launcher selecionado. Antes da
substituição: confirmar atividade do Coucou, encerrar normalmente quando seguro,
fazer backup específico e verificar compatibilidade pelos leitores atual e
candidato. Após arranque: confirmar executável/versão, dados, preferências e
funcionalidade entregue. A revalidação de precondições antes de aplicar continua
obrigatória; esta preparação não demonstra efeitos de uma instalação futura.

Sem alteração de autostart, serviços, rede, compositor, credenciais, modelos ou
aprovações globais. Sem fixtures ou grants de QA no perfil pessoal. Recuar o
launcher não desfaz alterações de dados; não há restauro automático prometido.

## Autorização necessária

A instalação tem de incluir uma exceção estritamente focada para o registo
assinado em sysworkspace, preservando hooks e alterações alheias. O objetivo
atual suspende novos commits. A publicação anterior sem assinatura não autoriza
esta nova alteração da estação. A [skill sysworkspace](/home/pmglo/.agents/skills/sysworkspace-system/SKILL.md)
exige «concluir com um commit focado no sysworkspace».

O pedido de aprovação abrange este RPM47/42, a substituição limitada descrita e
o seu registo focado e assinado em sysworkspace. Não abrange commits/push/merge
do fork nem publicação do sysworkspace. Enquanto estiver pendente, o trabalho
pode continuar em unidades independentes de preservação funcional e autonomia.
