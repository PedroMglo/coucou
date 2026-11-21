# Coucou .64 preparada para entrega — 21/11/2026

O RPM normal `0.5.1-mochi.64` passou os ensaios privados abaixo. A
sessão operacional executava a .62 na verificação desta proposta.
O launcher da conta apontava para um prefixo .63 com o executável
exato dessa versão. A fonte registada no sysworkspace apontava
para a .62. [Observação sem alterações](../.build/station-boot-bd97006a-issue1/OPERATIONAL-OBSERVED.json).

A verificação posterior ao novo arranque observou a .63 em
execução. A .66 reúne a correção da pausa e da mudança de
ligação dos serviços. [Estado posterior](PETS-INSTALLED-DELIVERY-66.md).

| Artefacto | Identificação |
| --- | --- |
| RPM | [Coucou-0.5.1-mochi.64-native-229eb8e8d4773f50.rpm](../.build/release-checkpoints/Coucou-0.5.1-mochi.64-native-229eb8e8d4773f50.rpm) |
| SHA-256 do RPM | `229eb8e8d4773f50e9ae05686e7492f503b5fdb45d3636b3c5b9423bfdcd9a25` |
| SHA-256 do executável | `ebfbbad3a889d62d812a292ea3a889161840a9371af8ee11d3b6ef2e1b05af2d` |
| Fontes | [Manifesto do pacote](../.build/pet-goal-execution-next-r1/release64-voice-20261121/SOURCE-VERSION.json) e [arquivo de 1 376 entradas](../.build/pet-goal-execution-next-r1/release64-voice-20261121/SOURCE-ARCHIVE.json) |
| Ensaios | [Recibo técnico agregado](../.build/pet-goal-execution-next-r1/release64-voice-20261121/VERIFIED.json) |
| Preparação do registo | [Patch revisto para quatro ficheiros](../.build/pet-goal-execution-next-r1/release64-voice-20261121/installed-proposal/sysworkspace-coucou64.diff) e [verificação em seco](../.build/pet-goal-execution-next-r1/release64-voice-20261121/installed-proposal/REVIEW.json) |

## Verificação

No próprio RPM .64, a escuta retomou o microfone virtual selecionado
e recusou outro dispositivo com o mesmo nome. Os dois percursos de
Stop interromperam uma resposta de voz local Piper em reprodução.
Depois de reiniciar, o texto e a conversa continuaram disponíveis.
A revisão, sucessão e renovação 6/7 produziram três respostas
sintéticas. O restauro preservou oito registos assinados e deixou
a autoridade inativa. Os leitores .62, .63 e .64 restauraram os
mesmos 54 ficheiros de cada amostra nas sete combinações ensaiadas.
[Compatibilidade dos arquivos](../.build/station-boot-bd97006a-issue1/archive-readers64.json).

O próprio RPM .64 reduziu em 57,5% a CPU mediana do Hub visível
face à .62 nas três rondas privadas com um companion ocupado.
O código de renderização foi conservado da .63.
O cliente Android em JVM recebeu um diff do Codex por TLS da .64
e deixou de poder pedi-lo depois da revogação.
[Validação do issue](PETS-VALIDATION.md#hub-cpu-issue1-2111) ·
[Ensaio do cliente Android](ANDROID_LINK.md#android-client-against-rpm-64).

## Ativação proposta

Após autorização própria, confirmar o estado dos três binários
relevantes, a sessão e os ficheiros do sysworkspace. Encerrar Coucou
normalmente, confirmar que a instância única libertou o nome no
D-Bus e verificar uma cópia Coucou específica. Preparar a .64 no
prefixo novo e preservar .44, .62 e .63. Aplicar apenas o launcher
pelo gestor HOME com backup transacional. Arrancar e confirmar o
binário, a instância única, a janela e a preservação dos dados
e escolhas acompanhados. Registar os quatro ficheiros Coucou no
sysworkspace num commit focado com hooks e assinatura ativos.
[Ensaio isolado da transição](../.build/station-boot-bd97006a-issue1/adoption/handoff62-to64-b/RESULT.json).

O âmbito proposto usa zero quota Copilot/GitHub e zero inferência
real ou paga. A autorização anterior abrangia a .62. Aceitação
humana, voz física, Android físico e medição na sessão Hyprland
operacional continuam abertas.
