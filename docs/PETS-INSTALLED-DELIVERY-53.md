# Proposta de entrega Coucou 53 — 18/11/2026

**Proposta suspensa após confirmação de reenvio de anexos retirados.**
Num perfil privado e com dois ficheiros sintéticos, retirar o primeiro
anexo deixou apenas o segundo na seleção ativa. No turno seguinte, o
RPM 53 voltou a incluir os bytes do primeiro através da mensagem anterior
guardada em memória. O binário exato da versão 44 mostrou o mesmo efeito
num perfil separado. A composição a propor para entrega tem de incluir
a correção e passar de novo este percurso. [Recibo do RPM 53](../.build/attachment-turn-context-20261118/rpm53-private-1/RESULT.json).

O RPM normal 0.5.1-mochi.53 passou a validação técnica dos percursos
afetados. A preparação da entrega fica no projeto. A instalação
operacional continua a ser a 0.5.1-mochi.44.

| Artefacto | Identificação |
| --- | --- |
| RPM normal, 28 571 145 bytes | [Coucou-0.5.1-mochi.53-native-21dae4be20cbf50f.rpm](../.build/release-checkpoints/Coucou-0.5.1-mochi.53-native-21dae4be20cbf50f.rpm) |
| SHA256 do RPM | `21dae4be20cbf50f6a64e4b4ef03959083a8e9bd0a35ebe99ff9037bf759dbc1` |
| SHA256 do executável empacotado | `aa0bd6ee29e6f8f0685f2b1974f6ea900fa1957cf851c45e0057e452a1578048` |
| Fontes compiladas | [Manifesto de 1370 entradas](../.build/operational53-20261118/SOURCE1.json) e o relay original adicional |
| Arquivo de fontes | [Coucou-0.5.1-mochi.53-source-b37ed6e548e17879.tar.gz](../.build/release-checkpoints/Coucou-0.5.1-mochi.53-source-b37ed6e548e17879.tar.gz) |
| SHA256 do arquivo de fontes | `0780999f15cd464f6f0964e73ce0013f55c725f4b387800cfadbf116b45f375c` |
| Prova agregada | [Verificação do executável do RPM](../.build/operational53-20261118/VERIFIED.json) |
| Alteração proposta do sysworkspace | [Patch revisto](../.build/operational53-20261118/installed-proposal/sysworkspace-coucou53.diff) e [recibo](../.build/operational53-20261118/installed-proposal/REVIEW.json) |

O 53 corrige a regressão confirmada no próprio RPM 52. Quando a
configuração em segundo plano atualizava a barra, o centro já aberto
podia passar a inativo. O backend protege agora a navegação em curso ou
aceite enquanto reconcilia a barra compacta. Reutiliza o estado e a
autorização nativos. [Comparação com o 52 e provas da correção](PETS-INSTALLED-DELIVERY-52.md).

- No Hyprland real, o próprio RPM 53 abriu o centro na primeira
  oportunidade, manteve-o aberto ao aplicar a configuração em segundo
  plano e recolheu com o comando explícito. Duas invocações por linha
  de comandos alternaram o centro na mesma instância com D-Bus privado.
  As teclas globais do portal ficam fora deste ensaio.
- Em X11 privado, passaram o abrir e fechar repetidos, as chamadas
  antigas de abertura explícita e a recusa de pet inválido. O atalho
  e o controlo Geral mostraram e ocultaram os pets. A preferência
  persistiu após reinício. O pedido pendente conservou dono, destino
  e CID, reabriu na conversa certa e ficou retirado após Stop. Um
  turno Codex sintético, sem inferência real nem execução do comando.
- No perfil sintético de regras 6/7 já restaurado por comandos
  nativos, o 53 apresentou a regra e a resposta histórica. Recusou a
  nova admissão e preservou os 12 registos originais verificados.
  A prova de restauro original mantém o seu âmbito no 50.
- A prévia nativa de backup no mesmo perfil sintético deu o mesmo
  inventário com os executáveis exatos 44, 52 e 53. Foi efetuada no
  local admitido do perfil. A primeira tentativa do 53 parou antes de
  arrancar o binário por falta do socket D-Bus privado. A segunda
  passou. Não criou arquivo nem concedeu autoridade de execução.
  O inventário sintético não prova downgrade dos dados operacionais.

O instalador restrito verificou 13 ficheiros do pacote, incluindo o
relay original e os recursos Copilot, num prefixo privado de QA. O
executável e as dependências ELF foram verificados. O código dos 29
testes dirigidos é o mesmo da correção privada validada. A seguir à
integração, cinco ficheiros de metadados passaram para a versão 53.

## Âmbito exato proposto

1. Autorizar este RPM e os registos Coucou focados e assinados no
   sysworkspace. O âmbito anterior foi utilizado na entrega da
   instalada44. Confirmar disponibilidade do assinador com hooks e
   configuração YubiKey ativos e revalidar os quatro ficheiros
   propostos antes de alterar o anfitrião.
2. Rever a atividade e parar normalmente só a instalada44 quando
   estiver disponível para substituição. Rever e autorizar o backup
   nativo específico do perfil Coucou, com fingerprint próprio.
   Inspecionar o backup com os leitores 44 e 53. Preservar os prefixos
   existentes, dados, contas, credenciais, modelos e preferências.
3. Usar o instalador restrito no prefixo53. Aplicar pelo gestor HOME
   apenas a mudança de `.local/bin/coucou`, com backup transacional.
   Manter serviços, autostart, rede, configuração do compositor,
   atalhos e hooks instalados.
4. Confirmar arranque, versão, UI e preservação no perfil operacional.
   Usar os percursos autorizados sem inferência de conversa real ou
   paga e sem quota real Copilot/GitHub. Concluir apenas o registo
   assinado focado no sysworkspace. Commits do fork, pushes e merges
   continuam suspensos.

Reverter o launcher não reverte dados. Se a validação operacional
falhar e os leitores divergirem, conservar os dados e submeter o
restauro específico à revisão antes de o aplicar. Aceitação humana,
hardware físico, uso prolongado e a conclusão dos 49 requisitos
continuam pendentes.
