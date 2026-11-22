# Coucou 55 — proposta de entrega, 18/11/2026

O RPM 55 reúne a correção dos anexos e a abertura da segunda sucessão
na mesma vista. Os percursos abaixo passaram no próprio executável
empacotado em perfis privados. A versão 44 continua operacional.

| Artefacto | Identificação |
| --- | --- |
| RPM Fedora, 28 578 612 bytes | [Coucou-0.5.1-mochi.55-native-cb8289a2291a7f28.rpm](../.build/release-checkpoints/Coucou-0.5.1-mochi.55-native-cb8289a2291a7f28.rpm) |
| SHA256 do RPM | `cb8289a2291a7f281aed23bfca7b7b6d0efdfded08361c7c6f2f5f5423534baa` |
| SHA256 do executável | `fce695e5412c1691c3225ac26559f324a89e66c04317e6986f342448518275f3` |
| Fontes exatas | [1373 entradas](../.build/pet-goal-execution-next-r1/release55-rule-succession-20261118/SOURCE-VERSION.json) · [Arquivo com o relay original](../.build/release-checkpoints/Coucou-0.5.1-mochi.55-source-be9a7a5b4f59a15e.tar.gz) |
| Evidência | [Validação do pacote](../.build/pet-goal-execution-next-r1/release55-rule-succession-20261118/VERIFIED.json) · [Alteração proposta da estação](../.build/pet-goal-execution-next-r1/release55-rule-succession-20261118/installed-proposal/sysworkspace-coucou55.diff) |

## Ensaios do executável

- Depois de retirar uma das duas cópias, o pedido seguinte incluiu
  apenas o anexo ainda selecionado. A conversa e os dois registos
  voltaram após reinício, com leitura por autorizar e sem reenvio.
- Uma nova sucessão abriu na mesma vista após revogação, com a escolha
  por confirmar, o prazo original e uma ocorrência por gastar. Passaram
  a revisão com saldo restante e a renovação finita, três respostas
  sintéticas, recuperação após perda do processo e reinício sem repetição.
- A cópia e o restauro preservaram oito registos assinados, três
  objetivos e 12 payloads originais. A janela mostrou as respostas
  históricas. Uma nova execução foi recusada, sem inferência.
- Em X11 privado, a abertura e a alternância do centro, a visibilidade
  pelo atalho e por Geral, o reinício e o retorno ao mesmo pedido
  pendente passaram. O Stop retirou esse pedido do percurso do atalho.
- Os executáveis 44 e 55 deram o mesmo inventário de backup no perfil
  sintético de comparação. O ensaio de aplicação do patch e a
  verificação de sintaxe do launcher passaram em ficheiros privados.

Os ensaios usaram X11 privado, dados gerados e destinos sintéticos. A
primeira tentativa de preservação parou na abertura do D-Bus, antes de
executar o comando de backup. A repetição com D-Bus privado passou.
O perfil operacional, áudio físico e aceitação humana continuam a
exigir provas próprias.

## Âmbito proposto

1. Autorizar este RPM e um registo Coucou focado e assinado no
   sysworkspace. Rever o estado dos quatro ficheiros afetados e a
   disponibilidade do assinador com os hooks e a YubiKey ativos.
2. Confirmar a disponibilidade da instalação 44, encerrá-la normalmente
   e rever o backup nativo específico com o seu fingerprint.
   Inspecionar a cópia com os leitores 44 e 55.
3. Instalar no prefixo versionado 55 pelo instalador restrito e alterar
   somente o launcher através do gestor HOME, com backup transacional.
   Preservar prefixos antigos, dados, credenciais, modelos, destinos,
   preferências, serviços, compositor, autostart e políticas.
4. Confirmar arranque, versão, vista dos pets e dados no perfil
   operacional. Usar apenas os percursos autorizados, sem consumir
   quota Copilot/GitHub nem inferência de conversa real ou paga.
   Limitar o registo assinado às fontes da intervenção em sysworkspace.
   Commits do fork, pushes e merges continuam suspensos.

Em caso de reversão, o launcher tem o seu backup transacional e os
dados requerem revisão própria antes de qualquer restauro.
