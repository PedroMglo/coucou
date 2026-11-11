# Fontes e coleções persistentes de calendários

Contrato vigente09/11/2026; CTX-03 e parte de CTX-04, dependência de AUTO-03. Coleções de conversa/retensão verificadas na privada39; âmbito próprio do pet e objetivo unbound verificados na composição de desenvolvimento build44 com provider sintético. Preservação dos novos formatos passou em perfil gerado; entrega/aceitação humana pendentes. [Composição, comandos e limites](../.build/pet-goal-execution-next-r1/persistent-calendar-collections/VERIFIED.json).

## Identidade, formato e consentimento

`pet_sources` continua a fonte de metadados em `sources.json` no perfil Coucou. Registry1 conserva seleção única e bytes legados. Registry2 explicitamente escolhido admite várias fontes enabled para o mesmo `PetId/ConversationId`; não altera a forma dos Source/Owner existentes. Registry3 acrescenta o dono explícito `{petId, scope: "pet"}`, separado do dono antigo `{petId, conversationId}`. Não depende de criar/reter uma conversa e não fica disponível automaticamente em chats. Não há downgrade de versões nem promoção de inscrições antigas. IDs, revisões e CAS são do escritor comum `private_registry`; nomes/UIDs iguais não fundem fontes nem escolhem a primeira. A consulta singular recusa ambiguidade.

Um Source conserva id, revision, kind=ics, name, path absoluto, enabled e owners explícitos. O NativeUser revê as fontes e o dono na conversa ou no painel de perfis do pet e confirma PersistentCalendarRead para adicionar/reativar associações. Renúncia pode usar MetadataOnly. Retirar uma associação preserva os outros donos; reativar uma fonte partilhada suspensa cria uma associação própria, sem ativar os outros. A permissão documents do pet continua necessária. A seleção não autoriza objetivos, regras, modelos alternativos, escrita ou sincronização de conta.

`sources.calendar.manage` distingue a gestão NativeUser de `sources.calendar`, cujas ferramentas só recebem os IDs da seleção capturada. Gestão não resolve modelo, autentica contas ou lê ICS; consulta passiva permanece unknown/targetRequired. O mesmo componente funcional é usado pelos dois compositores de conversas geridas, para os pets/adaptadores aplicáveis; uma sessão externa sem esse percurso não herda suporte.

## Captura, consulta e revogação

`selected_many` captura os membros de um único snapshot. Os membros partilham a lease do registry; `validate_group` lê configuração/verifica o snapshot uma vez por coleção durante a observação. O mesmo `pet_calendar::from_scopes` sela anexos e fontes inscritas, mantém o turno/CID/dono originais, fixa cada revisão e volta a validar antes/depois de IO e publicação. Adicionar/remover fontes não redireciona um turno iniciado.

Uma coleção oferece catálogo e ferramentas existentes com sourceId explícito por consulta; não lê eventos de uma fonte implícita. Falta de ficheiro/parser é unavailable com proveniência, não calendário vazio confirmado. Um membro inicialmente indisponível não é reaberto sob a mesma captura. Alteração/revogação descarta o resultado e cancela o turno original; uma edição do registry invalida conservadoramente capturas anteriores desse livro. As associações de outros donos permanecem nos dados, mas não se promete que pedidos de calendário já em curso sobrevivam a uma edição global.

Limites:64 entradas no livro,32 enabled por dono,33 fontes combinadas com anexos,512KiB por fonte,8 consultas e128KiB de saída partilhados por turno,24KiB de catálogo/prompt. Conteúdo, descrições e URLs são dados não fiáveis. Não executam instruções nem concedem autorização para mensagens, ferramentas ou mudança de política.

## Preservação e âmbito da prova

O backup nativo conserva o livro inteiro; o restauro/ativação mantém IDs/caminhos/nomes/owners/versão e suspende todas as fontes, com as revisões do escritor. A43 aceita o formato1 e recusa projetar a ativação do2 sem o destruir; a inspeção de settings/pets é separada e pode ser compatível. A importação de raiz legada mantém a barreira1.

Passaram componentes Rust, o processador real com dois ICS sintéticos, controlos web simulados, compilação nativa e26 recibos de CLI. O primeiro perfil QA incompleto foi recusado e conservado; a prova final usa preferências completas geradas, sem copiar perfis autenticados. Não prova contas reais, inferência, aprovação humana, janela instalada ou rotina diária. Fontes próprias de objetivo/regra/ocorrência, agendamento, notificações e aceitação operacional continuam no roadmap. Desenvolvimento/testes usam fixtures; não foi efetuada ingestão de documentos pessoais ou alteração da PIF.

## Retenção da conversa vazia —09/11/2026

A primeira prova na janela privada38 revelou que uma associação criada antes da primeira mensagem podia ficar ligada a um CID não gravado, substituído ao reiniciar. A correção1277 reutiliza `Chat.retain_public_conversation` antes da publicação do livro, sob o mesmo CAS e consentimento de fontes. Só fixa o dono/CID já existente da fábrica; não cria, restaura ou envia conversas. Consentimento/CAS são verificados antes desta gravação e, se a retenção falhar, não se publicam novas associações. Uma associação órfã antiga pode ser revogada sem a reativar ou inventar o seu dono.

Depois de carregar a biblioteca, o reset conserva a conversa vazia referida no livro de fontes. A sonda é só de metadados, sem locks de Chat/configuração adicionais nem leitura de ICS; livro desconhecido conserva dados por precaução. Retenção da conversa e publicação da fonte são gravações ordenadas, sem promessa de transação entre ficheiros: uma falha posterior pode deixar a conversa vazia retida, sem conceder leitura de fontes.

A janela Tauri39 verificou adicionar/rever/confirmar dois calendários, recusa de consentimento ausente/CAS antigo, revogação de uma associação preservando outro dono, reinício com o mesmo CID/revisões/livro e arquivo da conversa vazia ao criar outra após reinício. Usou perfil gerado e inputs humanos sintéticos; não prova aprovação humana ou pergunta real ao modelo. [Provas exatas e limites](../.build/pet-goal-execution-next-r1/calendar-conversation-preservation/VERIFIED.json).

## Uso por objetivos —09/11/2026

O percurso explícito de recortes usa contexto próprio e consentimento do objetivo, a partir de fontes próprias do pet ou referências inscritas numa conversa sua, sem copiar mensagens. A inscrição de conversa continua sem conceder autonomia. Preservação e leitura histórica do Goal64 passaram em perfil gerado; Source3/Goal65 também passaram no restauro nativo inativo/quarentena/ativação sem grants e nos leitores atual/anterior, conservando consumo e contexto históricos. [Prova e limites](../.build/pet-goal-execution-next-r1/pet-source-preservation/VERIFIED.json). Entrega e scopes de regra/ocorrência permanecem abertos. [Contrato específico](PET-GOAL-CALENDAR-CONTEXT.md).
