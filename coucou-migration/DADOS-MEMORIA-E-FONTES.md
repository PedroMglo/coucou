# Dados, conversas, anexos, memória e fontes

Revisão de **25/10/2026**. O objetivo é contexto útil e persistente com propriedade, proveniência, revisão e controlo. Este documento distingue os formatos atuais dos contratos ainda por completar. Complementa os capítulos 13–15 e 20 da [arquitetura](ARQUITETURA-COMPLETA.md).

## 1. Conversa, memória e ficheiro físico são coisas distintas

A biblioteca comum guarda conversas e arquivos em `assistant/conversations.json`, dentro do diretório de dados resolvido pela plataforma. O facto de várias conversas partilharem um JSON não concede contexto global aos pets. O backend seleciona o dono e a conversa relevantes. Os fornecedores CLI mantêm também histórico/sessões nos seus mecanismos nativos; não se afirma que o JSON local contenha toda a história externa.

`ConversationId` identifica a conversa local; `NativeSessionId` identifica a sessão do agente. O runtime de objetivos tem repositório próprio. Notas/preferências têm catálogo próprio em `assistant/memories.json`. Anexos têm cópias/manifestos e referências próprias. Credenciais não pertencem a nenhum desses conteúdos do modelo.

Os caminhos são descritos relativamente à resolução de dados da aplicação. Um perfil privado de teste, o perfil selecionado e a instalação podem ter raízes diferentes; não se deve presumir um caminho universal nem mudar de perfil sem rever os dados e os writers.

## 2. Inventário lógico de persistência

| Repositório | Dados e garantias |
| --- | --- |
| Configuração | Pets, perfis/modelos, acessos e preferências; validação, revisão e preservação. |
| Conversas/arquivos | IDs locais, mensagens públicas, ligação nativa, tarefas resumidas e proveniência compatível com a versão. |
| Cópias/anexos | Bytes privados, IDs, manifestos, referências, retenção e recuperação; nome não é identidade. |
| Fontes | Inscrição/consentimento, revisão, dono/âmbito, disponibilidade, frescura e cobertura. |
| Memória | Notas/preferências e futuramente projetos/episódios/derivados; origem, revisão, sensibilidade, validade e retenção. |
| Runtime/autoridade | Objetivos, planos, concessões, reservas, claims, tentativas, resultados, efeitos/recibos, recuperação e histórico. |
| Regras | Definições, gatilhos, concessões, ocorrências e entrega de notificações. |
| Consumo | Observações por tentativa/thread/conta, período, origem e cobertura, sem estimar ausência como zero. |
| Segredos | Apenas referências aos mecanismos locais/nativos apropriados; sem cópias em Git, prompts ou documentos. |

Há relações entre repositórios e fases de publicação. Não existe atomicidade automática entre vários ficheiros só porque estão na mesma pasta. O desenho usa revisões, exclusão de escritores, marcadores/recibos e recuperação explícita.

## 3. Vários anexos na mesma conversa

O defeito relatado pelo utilizador é a aparente substituição do anexo anterior ao adicionar outro. O destino obrigatório é uma coleção de ficheiros/pastas por conversa, com adição cumulativa, seleção e remoção individual. A migração conserva o anexo único antigo como primeiro elemento, sem inventar leituras ou consentimentos anteriores.

Cada item tem ArtifactId, dono/CID, ordem, revisão, origem, tipo, dimensão e informação de cobertura. Um turno captura IDs/revisões e limites antes do IO. Adicionar outro ficheiro durante uma chamada não altera o conjunto já capturado. Nomes iguais não fundem itens. Entregas atrasadas de drag/drop não passam para outra conversa ou janela.

Copiar um ficheiro não autoriza ler toda a pasta, toda a conta ou versões futuras. Manifestos de recuperação também não concedem leitura. A extração valida tipos, tamanho, quantidade, profundidade, bytes/tokens e tempo antes de materializar conteúdo sem limite. Indica partes não lidas e formatos não suportados pelo destino.

Selecionar para contexto, retirar seleção, revogar leitura, apagar a conversa e apagar a cópia são operações separadas. Cópias com referências partilhadas, origem antiga incompleta ou recuperação pendente são conservadas até haver prova suficiente para limpeza.

## 4. Retenção e recuperação de anexos

A limpeza precisa de uma conclusão durável da operação original e da cadeia de referências. Restauro parcial, nome reutilizado, falha entre publicações e limpeza interrompida não justificam eliminar a origem. Os envelopes e marcadores de recuperação devem permitir retomar a finalização sem repetir a importação ou reautorizar dados.

O protótipo de anexos múltiplos tem ensaios privados extensos; a conclusão integral e a integração/backup/rollback ainda constam de CTX-01/02 e REL-01. Não se apresenta esse protótipo como correção entregue na .30. Uma build nova não deve ser ativada sobre dados que exijam leitores mais recentes sem plano de compatibilidade.

## 5. Memória individual e partilhada

| Classe desejada | Conteúdo e autoridade |
| --- | --- |
| Preferência | Escolha revista do utilizador, com origem e âmbito. Não concede uma operação por si só. |
| Nota | Informação explícita selecionável/corrigível; pode ter sensibilidade e validade. |
| Contexto de projeto | Factos e referências ligados ao projeto autorizado, sem tornar todo o filesystem acessível. |
| Episódio | Resumo de acontecimento/trabalho com ligações às evidências e ao contexto de origem. |
| Resumo/derivado | Informação produzida a partir de fontes identificadas; mantém dependências, cobertura e invalidação. |
| Memória partilhada | Itens concedidos a destinatários/âmbitos específicos; não há biblioteca global implicitamente visível. |

Cada item precisa de MemoryId, dono, âmbito, revisão, proveniência, classificação de sensibilidade, validade e política de retenção. Correção, exclusão e revogação propagam a invalidação aos índices/derivados aplicáveis. Expiração retira disponibilidade sem necessariamente apagar a prova histórica. A retenção final deve ser revista e visível.

O incremento integrado atual implementa preferências/notas de origem humana, por pet/conversa, CRUD, pesquisa estruturada/lexical e páginas limitadas com CAS. Não implementa automaticamente as restantes classes ou partilha. O catálogo inicial tem limites de 1 MiB e 256 itens; esses limites de armazenamento não substituem os limites de contexto do modelo.

Memória recuperada é dado, não instrução de sistema nem concessão. Um texto «ignora as permissões» numa nota não altera o backend. Guardar uma preferência sobre uma ferramenta não autoriza o seu uso. Projetos e destinatários exigem âmbitos explícitos.

## 6. Seleção, revisão e destino

O contexto selecionado usa IDs e revisões exatos e tem limites acumulados. A captura valida os registos reais, o dono, âmbito, validade e bytes. Uma edição noutro pet não muda os dados selecionados; uma alteração no item selecionado invalida a captura.

Antes do envio, a revisão apresenta os dados completos dentro dos limites, origem, destino/modelo e âmbito de consentimento. Há três fronteiras distintas:

| Uso | Consentimento e continuidade |
| --- | --- |
| Pedido sem sessão remota persistente | Um turno exato; token de revisão de uso único, associado a caller/pet/CID/pergunta/destino. |
| Sessão CLI nativa | Contexto de sessão explicitamente revisto; proveniência de possível entrega durável antes do prompt, com destino e ID nativos. |
| Objetivo autónomo | Dados/destino e âmbito do objetivo revistos antes da admissão e do hash canónico; um token manual não autoriza rondas autónomas. |

Os dados entram como conteúdo temporário identificado, sem copiar as notas brutas para a pergunta ou histórico públicos. O fornecedor pode conservar contexto já recebido; retirar a nota localmente não o apaga do fornecedor.

Para CLI, a união das referências anteriores é conservada. Não se esquece silenciosamente uma nota já entregue, não se trunca a proveniência para caber no limite e não se cria outra sessão quando o ID ficou desconhecido após possível entrega. A transição de contexto deve ser explícita e suportada; «Nova conversa» conserva o arquivo anterior.

## 7. Revogação durante trabalho e reinício

Alteração, exclusão, expiração ou revogação bloqueiam nova reutilização pela app. A validade é acompanhada durante IO e esperas por decisão/comando, e conferida diretamente antes de novos efeitos e da classificação final. A supervisão usa o cancelamento e a reserva existentes, sem adquirir outro recurso físico.

Cancelar um processo não liberta o recurso antes de recolha/limpeza. O sistema conserva efeitos e consumo conhecidos/incertos. Reiniciar não renova consentimento, não repete dados brutos porque há metadados e não apaga a incerteza de entrega.

No SDK privado atual, conversas com proveniência nativa usam formato condicional versão7. Objetivos com seleção usam contexto versão2 e registo versão11; sem seleção mantêm os formatos antigos. Leitores anteriores devem recusar dados desconhecidos preservando bytes, em vez de os normalizar perdendo proveniência. Essas mudanças privadas ainda exigem adoção e migração/entrega próprias.

## 8. Fontes persistentes

Inscrever uma fonte define um principal de autorização, operações, revisões, prazo, frescura e cobertura. O catálogo atual por pet/CID não é autorização para todas as rotinas. Evoluir para objetivos/regras requer schema/migração e captura por ocorrência, conservando os grants antigos nos seus âmbitos.

Vários calendários/fontes devem poder ser selecionados explicitamente. Um limite atual de uma fonte por owner não satisfaz esse destino. Regras usam apenas fontes que a regra e o pet tenham autorização para consultar. Mudança de revisão ou revogação invalida os trabalhos dependentes.

**ICS** é um formato de ficheiro/exportação de calendário. Uma importação não sincroniza a conta nem autoriza escrever eventos. Consultas preservam fonte, UID/revisão aplicável, timezone, intervalo e cobertura. Recorrências e fusos precisam de limites e aceitação; não se interpreta uma leitura parcial como calendário vazio.

**RSS** começa por leitura limitada com destinos e redirecionamentos revistos, TLS e tamanho/prazo aplicados antes de descarregar tudo. **IMAP** começa por pastas/operações de leitura selecionadas, número de mensagens e bytes limitados no protocolo, sem envio ou eliminação implícitos. Conteúdo remoto continua não fiável.

## 9. Pesquisa, índices e embeddings

A primeira implementação deve preferir filtros estruturados e pesquisa lexical suficientes para a escala observada. Índices são derivados e reconstruíveis, sujeitos às mesmas concessões e invalidação. Um índice não permite consultar itens de outro pet/projeto.

Embeddings locais ou externos só entram quando houver benefício medido e limites operacionais. Enviar conteúdo a um serviço de embeddings é outro destino de dados e exige concessão própria. Escolher PostgreSQL ou pgvector não resolve sozinho proveniência, autorização, apagamento de derivados ou recuperação.

## 10. PostgreSQL: utilidade e decisão

**Pode trazer vantagem**, sobretudo quando as relações entre objetivos, memória, fontes e eventos, a pesquisa e a concorrência justificarem transações e consultas mais fortes. **Ainda não está integrado nem é requisito prévio para continuar**. O armazenamento local existente evita introduzir um daemon apenas por estar disponível.

A decisão deve medir volume, latência, conflitos, custo de migração, recuperação, operação offline e disponibilidade. O contrato de repositório deve permitir uma implementação local e, se compensar, PostgreSQL sem mudar a autoridade dos pets.

Uma adoção usa base/schema/utilizador exclusivos do fork, privilégios mínimos, migrações versionadas, segredo referenciado e backup/restauro testados. Preserva IDs, relações, revisões e concessões; não lê bases de outros projetos. Dados de fornecedores externos e efeitos no computador continuam fora de uma transação SQL e precisam de reconciliação.

## 11. Critérios de aceitação de dados

Os ensaios incluem concorrência, corrupção/versão desconhecida, crash antes/depois de publicação, dados com mesmo nome, isolamento por pet/CID/projeto, expiração, revogação durante IO/decisão, restauro e leitor antigo. Testam também biblioteca vazia, memória numa conversa sem mensagens e a conservação do CID no arquivo.

A entrega exige que a seleção e as decisões estejam acessíveis ao utilizador, que os transportes suportados usem o conjunto exato e que as limitações de dados já enviados sejam compreensíveis. APIs privadas ou um CRUD verde não fecham MEM/CTX/DATA por si só.

Fontes: [arquitetura canónica](fontes/docs/PETS-ARCHITECTURE.md), [estado](fontes/docs/PETS-IMPLEMENTATION-STATUS.md), [validação](fontes/docs/PETS-VALIDATION.md) e [avaliação AI Local](fontes/docs/AI-LOCAL-REUSE.md).
