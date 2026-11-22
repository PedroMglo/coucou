# RPM 54 — anexos e preservação de dados, 18/11/2026

O RPM 54 passou a verificação do executável empacotado em perfis
privados. A versão operacional é a 44. O pacote 54 está conservado no
projeto para ensaio. A nova proposta de instalação continua pendente.

| Artefacto | Identificação |
| --- | --- |
| RPM normal, 28 579 399 bytes | [Coucou-0.5.1-mochi.54-native-27a4d9ec8421108f.rpm](../.build/release-checkpoints/Coucou-0.5.1-mochi.54-native-27a4d9ec8421108f.rpm) |
| SHA256 do RPM | `27a4d9ec8421108fcf2efea9172b4b2de9cff528ab75632c8ad1314fe19ea925` |
| SHA256 do executável empacotado | `7c526f5dd57f4a955411e98e60625a0c96a1e2d8f7fb9ca86f2486c38e1e4fcf` |
| Fontes e evidência | [1372 entradas exatas](../.build/pet-goal-execution-next-r1/release54-attachment-context-20261118/SOURCE-VERSION.json) · [Recibo de reconstrução da evidência da build](../.build/pet-goal-execution-next-r1/release54-attachment-context-20261118/RECOVERED-BUILD-ARTIFACT.json) |

- Dois documentos sintéticos entraram por arrasto GTK no chat Ollama.
  A retirada de um documento deixou o outro na seleção. O segundo
  pedido HTTP conservou a conversa e incluiu o conteúdo da cópia
  selecionada. O conteúdo retirado ficou fora do pedido. Após reinício,
  voltaram os dois registos por mensagem e a conversa original. As
  concessões de leitura continuaram vazias e nenhum pedido foi repetido.
- O centro alternou a vista em X11 e conservou a abertura explícita.
  A visibilidade dos pets foi controlada pelo atalho e por Geral,
  persistiu após reinício e preservou o mesmo pedido pendente até Stop.
- Nove comandos do 54 leram e restauraram o arquivo sintético existente
  de regras 6/7, verificaram as assinaturas e ativaram a configuração
  sem concessões de execução. A janela nativa mostrou a resposta
  histórica e recusou nova admissão, com 12 registos originais
  preservados e zero pedidos de inferência nesse perfil.

A compilação normal e a criação do RPM constam do log. O invólucro
Python ficou sem recibo terminal. A inspeção independente confirmou
os digests do RPM, o manifesto completo e o executável compilado,
incluindo a marca de pacote do Tauri. O instalador restrito verificou
13 ficheiros num prefixo privado.

O ensaio de uma regra nova executou a primeira sucessão e registou a
resposta sintética, mas a própria vista ficou bloqueada ao tentar abrir
a segunda revisão, mesmo após revogação. A fonte posterior corrige esse
estado após confirmação válida. O teste dirigido reproduziu a falha e
passou após a correção. A sucessora privada abriu a segunda revisão na
mesma vista e passou a sequência renovada 6/7, incluindo restauro.
Essa correção posterior terá composição própria.

[Recibos dos ensaios no executável empacotado](../.build/pet-goal-execution-next-r1/release54-attachment-context-20261118/VERIFIED.json)
