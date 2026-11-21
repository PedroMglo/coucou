# RPM 52 e correção do arranque do centro — 18/11/2026

O RPM 52 passou os percursos de abrir e fechar o centro e mostrar e ocultar
os pets em X11 privado. No Hyprland, uma atualização da configuração ainda
podia fechar o centro já aberto. A proposta de instalação fica suspensa.
A instalação operacional é a 0.5.1-mochi.44.

| Artefacto | Identificação |
| --- | --- |
| RPM normal, 28 570 645 bytes | [Coucou-0.5.1-mochi.52-native-6a88427a9e6648a0.rpm](../.build/release-checkpoints/Coucou-0.5.1-mochi.52-native-6a88427a9e6648a0.rpm) |
| SHA256 do RPM | `6a88427a9e6648a095522816004695100649721ff6367881f72aa945ee1c6fe2` |
| SHA256 do executável empacotado | `301ab556ed19f840aa213184a3e0c9b61c0a699510560d461aca46d7bb58e07a` |
| Fontes exatas com 1369 entradas e relay original adicional | [Manifesto 52](../.build/operational52-20261117/SOURCE1.json) |
| Binário privado com correção do arranque | `73b79717300d6ce82a818675b1f4360e325da398acdd763d7b02b10097f7307a` |
| Resultados e limites | [Verificação agregada](../.build/operational52-20261117/VERIFIED.json) |

O executável extraído do RPM alternou a vista do centro quatro vezes e
conservou as chamadas antigas de abertura explícita. Uma tentativa com
pet inválido foi recusada sem alterar a revisão. O ensaio de presença
mostrou e ocultou a janela dos pets pelo atalho e pelo controlo Geral.
A preferência de mostrar sobreviveu ao reinício. Ocultar, fechar e abrir
a conversa pelo atalho manteve o mesmo pedido, dono e CID. O Stop retirou
esse destino. Houve um turno RPC Codex sintético e zero inferência real
ou decisões sobre o comando pedido.

No Hyprland, o primeiro abrir podia chegar a expandido, depois passar a
inativo ainda durante o arranque. Após estabilização, o abrir e fechar por
pedido nativo funcionou. Uma chamada normal de configuração da barra, com
as preferências duráveis intactas, reproduziu a perda da vista expandida
no próprio executável do RPM 52. O recibo da reprodução tem sucesso
porque confirma o problema. A origem exata da chamada concorrente no
primeiro arranque não foi instrumentada.

A correção integrada usa o bloqueio já existente do contexto nativo.
A configuração em segundo plano pode mostrar ou recolher a barra
compacta. Uma navegação iniciada ou já aberta mantém a sua rota, com
reposição de geometria pela operação existente quando aplicável. A
autorização do Sistema continua a ser avaliada pelo runtime nativo.
Passaram os 29 testes dirigidos do núcleo e o ensaio no Hyprland com o
binário privado corrigido. O centro abriu na primeira oportunidade,
manteve-se expandido durante a reconfiguração e recolheu com o segundo
pedido explícito, com as preferências iguais.

O sucessor deve conter esta correção e passar as verificações do próprio
executável. A aceitação humana, a entrega operacional autorizada e a
aceitação integral dos 49 requisitos continuam pendentes.
[Preservação](../.build/operational52-20261117/PRESERVATION.json).
