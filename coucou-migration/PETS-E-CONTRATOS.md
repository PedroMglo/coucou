# Pets, fábrica, adaptadores e contratos

Arquitetura desejada e inventário de código de **25/10/2026**. Este documento aprofunda os capítulos 5–8 da [arquitetura completa](ARQUITETURA-COMPLETA.md). O catálogo no final é uma fotografia das definições, não uma declaração de que os 25 pets são assistentes operacionais.

## 1. Um pet é um dono lógico persistente

O pet conserva PetId, configuração, contexto, sessões, memória e objetivos. A imagem, nome, aliases, conta e modelo não substituem esse identificador. A identidade permanece mesmo com o motor desligado. O utilizador pode chamar diretamente cada pet nas capacidades declaradas.

Um runtime por pet é uma responsabilidade lógica do backend, não a obrigação de criar um processo do sistema operativo para cada personagem. Workers físicos são criados conforme o trabalho e os recursos. Uma operação determinística pode executar sem qualquer LLM. Um modelo local descarregado continua representado por um pet configurado, cujo estado de inferência fica indisponível ou à espera.

Os tipos atuais são LLM, serviço e observador. Não são níveis de importância. Um observador tem capacidades limitadas pelo adaptador existente; o objetivo de o tornar um assistente completo exige implementar essas capacidades. Um serviço oferece operações próprias e pode ter assistência LLM adicional com dono separado.

## 2. Fábrica e propagação de melhorias

A origem declarativa é `windows/src-tauri/pets.factory.json`. Os resolvedores nativo e TypeScript usam a mesma ordem: base, molde, extensão e override do pet. `pets.default.json` é uma projeção de compatibilidade; não deve tornar-se outra origem de verdade independente.

| Camada | Conteúdo apropriado |
| --- | --- |
| Base | Estrutura comum e defaults que não concedem autoridade. |
| Molde | Comportamento comum a LLM ou aplicação/serviço. |
| Extensão | Reutilização técnica por família, como o transporte de uma conta Codex. |
| Pet | Identidade e escolhas explícitas: provider, nomes, aliases, acessos e execução. |
| Configuração do utilizador | Preferências editáveis validadas e preservadas durante upgrades. |

Blueprints são imutáveis; configurações são cópias independentes. Alterar uma preferência de Spotify não modifica Codex ou Notion. Uma correção de cancelamento, contexto ou decisão no componente comum deve beneficiar todos os adaptadores compatíveis sem replicar código por personagem.

A propagação não transmite permissões. Uma nova capacidade tem de aparecer indisponível/desativada até haver implementação, suporte, configuração e concessão. Provider e routing são explícitos; não se inferem pelo nome, imagem ou aliases. O campo provider vazio antigo é preservado e tratado de acordo com o seu contrato, sem conversão silenciosa em outra conta.

Para acrescentar um pet: definir identidade, molde e adaptador; declarar operações reais; mapear configuração/modelos; integrar contexto e decisões; garantir cancelamento e preservação; validar percursos positivos e recusas; só depois anunciar a capacidade. Se falta um protocolo necessário, o pet continua observador ou com essa operação indisponível.

## 3. Contrato de capacidades

O descritor v1 identifica pet, tipo, adaptador e revisões. Cada capacidade transporta suporte, disponibilidade, motivo, origem da observação, frescura e limites. Abrange conversa, streaming, Stop, ferramentas, decisões, anexos, imagens, voz, continuidade, execução, memória, fontes, Sistema, consumo e cooperação.

O caller também importa: gestão humana de notas, uso de memória num turno e uso autónomo num objetivo são capacidades diferentes. Publicar CRUD humano como «memória autónoma» é incorreto. A configuração de um modelo não prova prontidão de credencial, instância, tools, áudio ou sessão.

Os descritores devem ser consumidos pelo chat, voz, gestão, runtime, ferramentas e futuras superfícies remotas. A execução revalida as condições determinantes. Uma cache expirada pode exigir atualização; uma mudança de dono, modelo, política ou contexto invalida uma autorização mesmo antes de expirar o TTL.

## 4. Contrato de adaptador

| Operação | Garantia pretendida |
| --- | --- |
| Descobrir/configurar | Validar o modelo, endpoint/instância e conta sem carregar modelos ou autenticar por mera consulta quando isso não é necessário. |
| Iniciar turno | Capturar dono, conversa, destino, contexto e cancelamento antes do IO. |
| Continuar | Associar o ID nativo canónico e o workspace exatos; recusar associação incerta. |
| Streaming | Entregar eventos com origem e ordem; falha parcial não equivale a resposta concluída. |
| Ferramentas | Preservar namespace, argumentos, callId, canal de decisão e autoridade. |
| Stop | Cancelar o turno e recolher apenas processos próprios, preservando o recurso até ao fim físico. |
| Resultado | Separar saída do modelo, efeito, consumo observado e incerteza. |
| Recuperar | Conservar associação/proveniência e impedir replay de efeitos ou dados sem a decisão necessária. |

Codex, Gravity e Copilot têm protocolos e momentos de adoção do ID diferentes. O código comum trata o que é comum; a tradução específica preserva essas diferenças. Uma bridge reutilizada não transforma as três entidades num único pet ou numa única sessão.

Para Ollama e LM Studio, a identidade inclui provider, modelo e servidor/instância efetivos. Não se substitui um pelo outro quando está indisponível. Para API, a credencial pertence ao mecanismo próprio e não aparece no descritor público. Uma credencial revista é comparada nativamente no envio sem publicar derivados secretos.

## 5. Continuidade e permissões dos agentes originais

Conversa local, sessão nativa e processo são entidades distintas. Ao retomar, pasta e intenção de criação devem ser compatíveis com a associação guardada. Uma mudança incompatível é recusada antes do prompt; «Nova conversa» preserva o arquivo e cria uma transição explícita.

Os pets de agentes devem conservar ficheiros, comandos, projetos e ferramentas/MCP disponíveis no perfil/sessão nativos. O utilizador não deve perder essas capacidades por usar o chat do pet. O protocolo pode, contudo, não expor uma função da TUI: a limitação é apresentada e aceite por capacidade.

Sandbox, modos de aprovação e políticas nativas não são substituídos por uma permissão genérica do Coucou. Retomar não reenvia flags mais permissivas para contornar decisões. Configuração de um agente não altera a política ACP de outro. Sessões externas precisam de associação de controlo comprovada; descoberta por pasta, PID ou nome só autoriza observação no âmbito adequado.

Uma decisão comum vincula sessão/turno/callId, operação/argumentos/alvo, política e prazo. Chat, voz, terminal e telefone autorizado são representações do mesmo pedido, quando suportado. Uma resposta válida consome-o uma vez. Decisão de outro pet, token repetido, configuração alterada e resultado tardio são recusados.

## 6. Serviços e partilha física de conta

Notion, Resend e outros serviços podem usar o transporte Codex para assistência. O dono continua a ser o PetId do serviço: diálogo, memória, tarefas, permissões e decisões não são os do pet Codex. A conta física partilhada entra no controlo de recursos e consumo, sem fundir a autoridade.

Spotify tem executor determinístico próprio e uma associação de assistência declarada. A disponibilidade de conversa assistida exige a configuração real e não elimina o controlo de música existente. Música, som global, documentos, rascunhos e envios são capacidades independentes.

O pet GitHub está retirado temporariamente. A definição e os dados são preservados para compatibilidade, mas criação de trabalho novo não o promove a ativo. Copilot CLI não é esse pet. Gravity também não é esse pet. A restrição de quota real em testes de Copilot/GitHub permanece, mesmo que os protocolos sintéticos passem.

## 7. Matriz de aceitação por família

| Família | Prova necessária para completar a capacidade |
| --- | --- |
| Codex | Associação/continuidade, ferramentas e decisões nativas, Stop/recolha, contexto, consumo, recuperação e perfil autorizado. |
| Gravity | Mesmas garantias nas operações expostas; limites do canal de aprovação e de sessões externas explicitados, sem bypass inventado. |
| Copilot CLI | Contrato ACP, política e IDs exatos; fixtures sem quota agora e aceitação operacional identificada como pendente. |
| Ollama / LM Studio | Modelo/instância exatos, limites, metering, ferramentas declaradas, contexto, falhas e autonomia nas rotas admitidas. |
| API | Configuração/credencial/destino, formatos, streaming, limites e tipos de conteúdo suportados; sem pressupor shell ou voz. |
| Serviços | Operação tipada, autenticação, alvo e resultado reais; assistência conserva o dono do serviço. |
| Observadores | Cobertura/origem/frescura; nenhuma observação é apresentada como controlo ou execução. |

Autonomia de CLI é uma linha própria: conseguir conversar não garante controlo dos limites internos da ferramenta. A aceitação do runtime não pode ser copiada de Ollama para Codex/Copilot/Gravity sem essa prova.

## 8. Catálogo declarado na fotografia

A tabela seguinte é gerada apenas do ficheiro de fábrica do repositório. «Rota» é configuração declarada, sujeita ao resolvedor, à retirada e à configuração efetiva. Não foram consultadas contas ou preferências privadas para a produzir.

| PetId | Nome declarado | Tipo | Adaptador | Rota de chat declarada | Estado/limite |
| --- | --- | --- | --- | --- | --- |
| `integration_claude` | Claude Code | observer | — | — | Observador; sem executor LLM declarado |
| `agent_cursor` | Cursor | observer | — | — | Observador; sem executor LLM declarado |
| `agent_antigravity` | Gravity | llm | antigravity | antigravity | LLM; capacidades condicionadas ao adaptador/configuração |
| `agent_codex` | Codex | llm | codex | codex | LLM; capacidades condicionadas ao adaptador/configuração |
| `agent_gemini` | Gemini CLI | observer | — | — | Observador; sem executor LLM declarado |
| `agent_copilot` | Copilot CLI | llm | copilot | copilot | LLM; capacidades condicionadas ao adaptador/configuração |
| `agent_muse` | Muse Code | observer | — | — | Observador; sem executor LLM declarado |
| `agent_opencode` | OpenCode | observer | — | — | Observador; sem executor LLM declarado |
| `agent_amp` | Amp | observer | — | — | Observador; sem executor LLM declarado |
| `agent_hermes` | Hermes | observer | — | — | Observador; sem executor LLM declarado |
| `agent_claude-desktop` | Claude Desktop | observer | — | — | Observador; sem executor LLM declarado |
| `ai_anthropic` | Anthropic | llm | — | anthropic | Chat API; configuração/credencial e aceitação próprias |
| `ai_google` | Google AI | llm | — | google | Chat API; configuração/credencial e aceitação próprias |
| `ai_openai` | OpenAI | llm | — | openai | Chat API; configuração/credencial e aceitação próprias |
| `ai_ollama` | Ollama | llm | ollama | ollama | LLM; capacidades condicionadas ao adaptador/configuração |
| `ai_lmstudio` | LM Studio | llm | lmstudio | lmstudio | LLM; capacidades condicionadas ao adaptador/configuração |
| `integration_resend` | Resend | service | codex | integration_resend | Serviço; só operações/configuração efetivas |
| `integration_n8n` | n8n | service | codex | integration_n8n | Serviço; só operações/configuração efetivas |
| `integration_vercel` | Vercel | service | codex | integration_vercel | Serviço; só operações/configuração efetivas |
| `integration_github` | GitHub | service | codex | integration_github | Retirado temporariamente; preservar dados |
| `integration_notion` | Notion | service | codex | integration_notion | Serviço; só operações/configuração efetivas |
| `integration_calcom` | Cal.com | service | codex | integration_calcom | Serviço; só operações/configuração efetivas |
| `integration_stripe` | Stripe | service | codex | integration_stripe | Serviço; só operações/configuração efetivas |
| `integration_music` | Apple Music | service | — | — | Serviço; só operações/configuração efetivas |
| `integration_spotify` | Spotify | service | spotify; assistência codex | integration_spotify | Serviço; só operações/configuração efetivas |

São 25 identidades declaradas, 24 não retiradas. Nem todas têm adaptador de execução ou configuração operacional. O resolvedor desativa execução nova da identidade retirada mesmo que a definição histórica conserve routing.

Fontes integrais: [fábrica](fontes/docs/PET-FACTORY.md), [contrato v1](fontes/docs/PET-CAPABILITIES-CONTRACT.md) e [estado canónico fotografado](fontes/docs/PETS-IMPLEMENTATION-STATUS.md).
