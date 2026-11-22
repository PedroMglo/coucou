**17/11 — também recuperado desktopToggle apenas na fonte posterior.**
A privada12230fac mostra/oculta os pets configurados com o atalho e o controlo
Geral, preserva a opção no reinício e o pedido/CID ainda pendente ao ocultar.
O RPM51 abaixo não inclui esta correção nem a de abrir/fechar. Continua suspensa
esta proposta; preparar uma sucessora própria antes de nova entrega.
[Contrato e provas](PETS-UPSTREAM-PARITY-2026-11-11.md#desktop-pets-1711).

**17/11 — proposta51 suspensa por regressão do atalho abrir/fechar.**
O pacote51 permanece intacto e conserva as provas abaixo. O atalho abre sempre o
centro; a alternância foi corrigida e validada numa composição privada posterior,
sem novo RPM. Não instalar51 como se incluísse essa correção. A próxima proposta
exige artefacto próprio, compositor real e autorização operacional própria.
[Comparação, correção e limitações](PETS-UPSTREAM-PARITY-2026-11-11.md#ilha-terminal-1711).

# Candidata51 com atalhos corrigidos — 17/11/2026

O próprio executável do RPM `0.5.1-mochi.51` passou os percursos afetados pela
correção: reabrir o pedido CLI exato duas vezes, retirá-lo com Stop e não o voltar
a abrir; alternar entre pets nos dois sentidos; regressar a permissões/perguntas
legadas sem responder. **Não está instalado nem autorizado para instalação.**
A instalada44/Source35, dados, escolhas e preferências estão preservados.

| Identificação | Valor |
| --- | --- |
| Aplicação normal | `fr.louisraille.coucou`, `0.5.1-mochi.51` |
| RPM, 28566526 bytes | [Coucou-0.5.1-mochi.51-native-64187cf810ec4596.rpm](../.build/release-checkpoints/Coucou-0.5.1-mochi.51-native-64187cf810ec4596.rpm) |
| SHA256 do RPM | `64187cf810ec4596f77d89e28a6b111531c01aae78169e6c77669607263db6d9` |
| SHA256 do executável embalado | `fae4139c640021a0586fddca61da0ed1d6a709d86d1b8b17c7ab0b9888bf69ef` |
| Fontes compiladas | [Manifesto1367](../.build/operational51-20261117/SOURCE1.json), mais o relay original adicional |
| Arquivo de fontes | [Coucou-0.5.1-mochi.51-source-087a33fc3b77edb1.tar.gz](../.build/release-checkpoints/Coucou-0.5.1-mochi.51-source-087a33fc3b77edb1.tar.gz) |
| SHA256 do arquivo | `7241a0ae2cf16f3a6588f191e6c1be29a4222a28b88f7ed7331a0607280e5fd1` |

O identificador no nome do arquivo é a referência ao manifesto; os bytes
comprimidos têm o SHA256 completo acima. HEAD de75abcb não representa toda esta
fonte. Commits do fork, pushes e merges continuam suspensos.

## Comportamento e comparação

No RPM50, Ctrl+Alt+A era recebido mas ignorava o pedido CLI ativo na conversa.
A51 reutiliza a coleção de destinos e a apresentação existentes, com ID/dono/CID
exatos, conservando prioridade legada. A navegação não responde nem aprova.
O pedido terminado por Stop deixa de ser um destino. Não acrescenta backend,
ledger, formato, autoridade, provider ou modelo; mantém domínios da fábrica.
n8n continua serviço de orquestração, sem chat/LLM genérico.

O oficial0.5.0/841a81ce não contém o transporte CLI novo, mas preserva o contrato
útil de regressar a permissões/perguntas. Esse percurso e o ciclo next/previous
passaram no oficial e, agora, no próprio51. O workspace Documents representa o
Claude ativo da fixture; os outros donos foram Codex, n8n e Ollama.
[Comparação independente e negativa original50](PETS-UPSTREAM-PARITY-2026-11-11.md#atalhos-pedidos-cli-1711).

## Validação e limites

A receita Fedora existente compilou com Cargo locked/offline e dependências do
projeto. O instalador restrito foi usado só num prefixo de QA dentro do projeto:
13 payloads, relay original, recursos Copilot, ELF e ausência de RPATH de projeto
verificados. A única transformação do executável foi o marcador Tauri UNK→RPM.
Capturas X11 reais do ambiente virtual privado foram observadas com pedidos
legíveis; a visibilidade foi medida antes de selecionar a conversa oculta.

Dois percursos nativos51 passaram: um único turn/start RPC sintético criou o
pedido CLI; o caso legado não iniciou turnos. Zero modelos reais, HTTP de
inferência, comandos aprovados/executados ou quota Copilot/GitHub. A escolha de
rota/revisão de atalhos pertence apenas à fixture própria, sem consentimento
humano fabricado ou alteração de políticas.

Os44 testes web e TypeScript são reutilizados por identidade exata do código
privado corrigido; só cinco ficheiros de versão mudaram para51. O backend da50
continua exato, por isso as suas provas de rotina/decisão/reinício/restauro e as
anteriores de voz, código/diffs, telefone e recursos conservam o âmbito anterior.
Não foram repetidas nem se apresentam como aceitação global51. Falhas/limites
anteriores continuam nos respetivos recibos; nenhum efeito desconhecido foi
repetido para melhorar uma observação.

Continuam pendentes compositor Wayland real (sessão observada bloqueada/ecrãs
desligados), hardware/Android/áudio físicos, uso prolongado, aceitação instalada/
humana, paridade integral e restantes critérios dos49 requisitos. A causa das
recusas telefónicas antigas não foi demonstrada; os sucessos50 não a explicam.

## Próximo passo e autorização

A51 sucede à proposta50, que fica arquivada sem aplicação. Os RPM49/50 e as
quatro fontes de proposta50 foram preservados; não aplicar aquelas fontes como
se entregassem51. Não foi alterado sysworkspace, launcher ou prefixo pessoal.

Antes de propor substituição: verificar a composição51 no compositor real numa
área/perfil isolados, rever backup nativo específico e compatibilidade, preparar
as fontes51 no projeto e obter autorização própria para este RPM e registo(s)
Coucou focado(s)/assinado(s) em sysworkspace. A exceção Source35 foi concluída.
Preservar44/43, dados, modelos e credenciais; manter hooks/assinatura. Sem mudar
autostart, serviços, rede ou políticas globais por conveniência de teste.
Enquanto esse gate estiver fechado, continuar lacunas funcionais delimitadas
de paridade e dependências49 sem acrescentar infraestrutura desnecessária.

[Verificação agregada](../.build/operational51-20261117/VERIFIED.json),
[preservação](../.build/operational51-20261117/PRESERVATION.json),
[reutilização proporcional das provas](../.build/operational51-20261117/PROOF-REUSE.json),
[observação gráfica privada](../.build/operational51-20261117/VISUAL-REVIEW.json).
