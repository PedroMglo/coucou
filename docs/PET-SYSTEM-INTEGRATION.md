# Integração de Sistema com as definições e widgets existentes

Decisão normativa de **07/11/2026**, por pedido explícito do utilizador. Complementa o capítulo 13 de [PETS-ARCHITECTURE.md](PETS-ARCHITECTURE.md), preserva SYS-01/02/03, EXE, VOICE e GUA e substitui a intenção anterior de construir definições próprias ou retirar o centro de controlo existente.

## Resultado pretendido

O Coucou integra a interface de Sistema já usada na estação. Não desenvolve outro painel de Wi-Fi, Bluetooth, Som, Energia ou preferências para a substituir. A interface e o acesso manual existentes continuam disponíveis com o Coucou desligado, em desenvolvimento ou indisponível. Os sete widgets e indicadores úteis são preservados, incluindo `controlCenterButton`.

Os pets consultam e controlam as componentes concedidas. O trabalho que permanece no Coucou é identidade, capacidades, autorização, adaptadores, contexto, admissão/execução, recursos, eventos, verificação e recuperação. Gestão de grants, decisões, estado do trabalho e encaminhamento para o painel existente são superfícies funcionais próprias; não se tornam um segundo editor das mesmas definições.

## Camadas e fonte de verdade

| Camada | Responsabilidade |
| --- | --- |
| UI existente da estação | Apresentar definições, estado e prompts do utilizador, preservando aparência, atalhos e comportamento próprios. Na estação atual é DMS/Quickshell. |
| Serviço/componente existente | Ser a fonte efetiva do estado: NetworkManager, BlueZ, áudio, compositor, energia ou serviço realmente utilizado. A UI e o adaptador observam o mesmo recurso. |
| Adaptador de integração Coucou | Descobrir versão/capacidade e expor operações tipadas sobre IPC/API suportados ou o mesmo serviço subjacente; observar eventos e pós-condições. |
| Broker/runtime comum | Autenticar o pet/caller, validar grants e alvo antes do efeito, ordenar conflitos, conservar causalidade, intenção, recibos, cancelamento e incerteza. |
| Pet | Pedir e acompanhar operações dentro dos seus grants; raciocinar quando necessário. Nunca atribuir permissões a si próprio. |

Não há um segundo daemon, scheduler, ledger de autoridade ou estado desejado global. Não se impõe DMS a outras plataformas: usam-se adaptadores para as superfícies existentes, com suporte/indisponibilidade explícitos.

A operação programática não precisa de abrir a UI. Reutiliza-se o IPC/API efetivo do componente ou o backend que já o alimenta; não é obrigatório forçar todos os comandos pela janela. Isto não autoriza reimplementar a lógica de rede/áudio como outro gestor. Não se copia configuração para uma fonte paralela nem se fixa um diretório de cache como contrato.

## Autoridade e «dono quando necessário»

Mantém-se **pet × subplugin × operação × recurso/âmbito**, com concessões independentes de leitura e controlo. Sistema ativo não é acesso global. Spotify e Ollama podem partilhar Som; retirar um grant não retira o outro.

O utilizador pode também designar um pet como responsável continuado por uma componente/configuração, através de um papel configurável e dos grants correspondentes. Essa responsabilidade pode persistir entre tarefas; as reservas de execução sobre recursos concretos continuam temporárias e o utilizador conserva o controlo.

O System Guardian é o especialista previsto para assumir essa responsabilidade principal quando designado pelo utilizador. O papel identifica o responsável por observar, conservar contexto autorizado, acompanhar estado e propor/executar manutenção nas componentes concedidas; não transfere para o pet a propriedade dos ficheiros nem a fonte de verdade dos serviços. Não depende de ser coordenador nem concede prioridade absoluta sobre outros pets. A designação não ativa grants, remediações automáticas, serviços ou autostart por si só.

Por exemplo, o Guardian pode acompanhar Rede/Bluetooth, Energia e saúde do computador, enquanto Spotify e Ollama dispõem de concessões próprias de Som. Um pedido direto a qualquer destes pets passa pelo mesmo broker. Conflitos sobre o mesmo dispositivo são arbitrados no runtime, e a ausência do Guardian não bloqueia as operações independentes ou os widgets humanos.

Ser dono significa assumir a responsabilidade operacional por uma tarefa autorizada e, quando preciso, reservar temporariamente o recurso exato. Não significa tornar-se proprietário permanente de ficheiros, da UI, do computador ou das políticas. A reserva tem prazo e dono, é revogável e só arbitra operações conflituantes. A alteração manual observada invalida pré-condições e prevalece sobre uma reposição automática antiga; o pet não combate o utilizador nem impede o acesso aos widgets.

UI, chat, voz e pets conservam autoridades distintas. Um clique humano nativo não é interceptado pela fila de permissões do Coucou. Inversamente, um pet não recebe autoridade humana só por chamar IPC da UI, abrir uma página ou conhecer o nome de um controlo. Permanecem os prompts e a autenticação nativa aplicáveis.

Não há shell, argumentos IPC livres, clique genérico ou configuração arbitrária como ferramenta para contornar esta matriz. Se uma função só existir na UI e não houver contrato programático seguro, anuncia-se navegação/interação assistida ou indisponibilidade de automação, sem fingir controlo autónomo.

## Estado, concorrência e recuperação

O pedido captura pet, tarefa, recurso/identidade, revisão, argumentos, prazo e grant antes de esperar. O executor revalida imediatamente antes do despacho. Eventos da UI/serviço e leitura posterior mantêm ambos coerentes; a mudança feita pelo pet aparece nos widgets e a mudança manual é observada pelo pet.

ACK de IPC, abertura pedida e estado efetivo são evidências distintas. Perda do ACK não repete mutações. Revogação impede novos efeitos e invalida pedidos antigos mesmo após reativação; não desfaz efeitos já enviados. Falha ou ausência da UI não provoca troca silenciosa de backend, endpoint ou alvo. Um backend direto já configurado pode permanecer funcional se o contrato o permitir, sem depender do estado de uma janela.

Credenciais, SSIDs, nomes de dispositivos e conteúdo privado não entram em logs/prompts sem âmbito explícito. A descoberta é limitada e passiva; não ativa serviços, rádios, autenticação ou modelos só para consultar disponibilidade.

## Adoção do que existe

A inspeção de 07/11 encontrou DMS 2.0.0 e Quickshell 0.3.2. A instância usa uma cópia QML de cache preparada pelo helper `dms-session-local` gerido em sysworkspace; esse cache não passa a ser fonte do Coucou. O `DMSShellIPC.qml` ativo coincide em hash com o pacote observado. Existem os handlers `control-center` (`open`, `openWith`, `status`) e `settings` (`open`, `openWith`); a sua presença em código não prova aceitação visual, nem controlo de todas as operações de Sistema.

No Coucou, `pet_system.rs`, `pet_native_network_core`, `pet_bluetooth` e os controladores de Som são base a reaproveitar. O protocolo legado `pets-system` e a lease visual de `pet_system.rs` têm de ser revistos: dependências de painéis próprios ou ocultação do centro existente não fazem parte do resultado pretendido. Conservar executores/guardas funcionais e compatibilidade até confirmar a nova rota; não apagar backend útil por estar ligado à UI anterior.

Uma adaptação futura do DMS deve ser mínima, tipada e versionada, sem criar definições duplicadas. Alterações do anfitrião, sessão ou fontes globais seguem sysworkspace; código/configuração interna do Coucou ficam no projeto. As exclusões existentes de reconciliação Coucou/DMS não são removidas por esta decisão. Não houve alteração da estação, instalação ou acesso real a rede/dispositivos nesta revisão.

## Incremento implementado na raiz em 09/11/2026

A superfície Sistema encaminha Wi-Fi, Bluetooth, Som e Bateria para `control-center openWith` do DMS existente; Definições rápidas usa `control-center open` e Área de transferência usa `clipboard open`. Os identificadores fechados de secção e os ACKs foram conferidos nas fontes do pacote DMS2.1.1 instalado. Uma confirmação IPC continua a não provar que a página/recurso pretendido foi observado no ecrã.

A consulta periódica de estado deixou de criar ou renovar leases que ocultavam widgets nativos. O processo novo não reclama o centro humano; os entry points antigos de release permanecem apenas por compatibilidade dos callers. Os editores próprios de Rede/Som saem desta superfície. Brokers, grants e controlos do pet Spotify conservam a sua autoridade própria. Metadados, Discos, Gravação e Tray mantêm o bridge tipado anterior até haver paridade aceite; não se apagam módulos úteis.

Os10 testes Rust dirigidos e a verificação de tipos passaram. Não foi enviado qualquer pedido de abertura/mutação ao DMS durante estes testes; a sessão está bloqueada. O delta está na raiz depois da candidata40 e ainda não foi empacotado/instalado. A40 já inclui o reforço de recuperação financiada; a instalada continua38. Sem alteração persistente do anfitrião, autorização nova, commits ou quota. [Fontes, comandos e limites](../.build/pet-goal-execution-next-r1/system-native-settings/VERIFIED.json).

## Plano de execução e aceitação

1. Inventariar operações e versões realmente suportadas nas unidades existentes: Rede/Bluetooth; Som/Multimédia; Ecrã/Energia/Sessão. Conservar grants separados.
2. Reutilizar/adaptar os brokers e executores existentes; separar navegação para a UI de leitura e mutação programáticas. Retirar do backlog a construção de ecrãs de definições equivalentes e a substituição de `controlCenterButton`.
3. Migrar chamadas legadas/lease visual com compatibilidade e sem retirar o acesso manual. Não aplicar ao anfitrião por autorização genérica de testes.
4. Demonstrar ação autorizada de um pet refletida na UI existente, alteração manual observada sem luta de escritores, grants independentes, conflito/revogação/alvo substituído e ACK perdido sem replay.
5. Verificar uso manual com Coucou parado, erro de IPC sem fallback inseguro, voz nos mesmos contratos e ausência de prompts Coucou em operações humanas exteriores. Fixtures não fecham hardware, UI real ou entrega instalada.

Esta mudança reduz a duplicação de UI; não reduz o objetivo dos pets, a matriz de permissões, o catálogo funcional nem os critérios de segurança e aceitação. Não torna o Guardian operacional nem conclui SYS por si só.

## Concessões próprias de Sistema — unidade iniciada em 09/11/2026

A configuração revista de cada pet já dispõe de `systemAccess`, separado de música, UI e motor. A primeira unidade implementa observar/volume/silêncio das saídas de Som, com recurso exato ou todas as saídas explicitamente revistas; entradas, streams, Rede/Bluetooth e restantes operações mantêm os requisitos próprios. Cada gravação humana gera revisão própria; a revogação de um pet não afeta outro e um consumidor antigo não revive por reativação. O percurso inicial é NativeUser endereçado ao pet, sem anunciar TaskRuntime/ProviderModel antes de integrar a admissão efetiva desses callers. O acesso legado Spotify/música conserva o seu contrato até migração revista e não concede acesso às novas rotas. Gestão de grants não inicia inferência, efeitos ou definições de hardware. [Contrato e aceitação](../.build/pet-goal-execution-next-r1/system-pet-sound-grants/CONTRACT.json).

A unidade está implementada com gestão nativa, rotas endereçadas NativeUser, projeção de capacidades e restauro suspenso. Adoção TaskRuntime/ProviderModel e migração do legado permanecem explícitas. Ver [contrato de concessões](PET-SYSTEM-ACCESS-CONTRACT.md) para formato, prova e limitações.
