# Coucou .66 — pacote privado de 21/11/2026

A .66 reúne duas correções de atualização de serviços. A .65
foi interrompida antes de gerar um RPM. O serviço de login da
estação executa a .63 após o novo arranque. O sysworkspace
continua a registar a .62. A ativação da .66 requer autorização
própria. [Observação da estação](../.build/station-boot-bd97006a-issue1/OPERATIONAL-AFTER-BOOT.json).

| Artefacto | Identificação |
| --- | --- |
| RPM | [Coucou-0.5.1-mochi.66-native-76dfdf30e36274d3.rpm](../.build/release-checkpoints/Coucou-0.5.1-mochi.66-native-76dfdf30e36274d3.rpm) |
| SHA-256 do RPM | `76dfdf30e36274d3fc51652f758629908fdb52ea5e0d036b5c511aaebc4a70b3` |
| SHA-256 do executável | `e7eb0b81f8c4954f601cd83a859f9a2f0bb47f6a2711e9c7dbf3222e04073955` |
| Fontes | [Manifesto](../.build/pet-goal-execution-next-r1/release66-service-connection-20261121/SOURCE-VERSION.json) e [arquivo de 1 376 entradas](../.build/pet-goal-execution-next-r1/release66-service-connection-20261121/SOURCE-ARCHIVE.json) |
| Preparação | [Receita Fedora e verificação dos 13 ficheiros](../.build/pet-goal-execution-next-r1/release66-service-connection-20261121/BUILD.json) |
| Ensaios | [Recibo nativo agregado](../.build/pet-goal-execution-next-r1/release66-service-connection-20261121/VERIFIED.json) |

## Percursos do próprio RPM

- **n8n.** Zero GETs após pedidos feitos em pausa ou com o pet
  desativado. Também houve zero GETs complementares quando a
  pausa, a desativação, a troca de instância ou a rotação da
  chave aconteceu durante uma leitura. O cartão, o agendamento,
  a rota compatível da instância ativa e o detalhe de erro
  continuaram funcionais. A execução pendente pôde ser retomada
  e apareceu uma vez. [Ensaios](PETS-VALIDATION.md#integracoes-pausa-2111).
- **Guardian.** Inventário de distribuição e kernel, objetivo
  finito sem inferência e baseline de CPU e memória pela
  interface. O ensaio confirmou concessões próprias por pet e
  recurso, revogação, recusa de repetição e reinício.
  [Recibo .66](../.build/pet-goal-execution-next-r1/guardian-current-20261121/VERIFIED-66.json).
- **Regras 6/7.** Passaram revisão, sucessão, renovação agendada,
  três respostas sintéticas, Stop e reinício sem repetição.
  O restauro conservou oito registos assinados e a janela mostrou
  o resultado histórico com execução recusada. Os leitores .63 e
  .66 restauraram os mesmos 54 ficheiros de cada arquivo de ensaio
  nas quatro combinações, com autoridade inativa.
  [Compatibilidade dos arquivos](../.build/pet-goal-execution-next-r1/release66-service-connection-20261121/ARCHIVE-COMPATIBILITY.json).

Os ensaios usaram perfis gerados e zero inferência real ou paga.
Os serviços remotos, o hardware e a aceitação humana conservam
o âmbito pendente. Qualquer ativação da .66 depende de autorização
própria e registo assinado no sysworkspace.
