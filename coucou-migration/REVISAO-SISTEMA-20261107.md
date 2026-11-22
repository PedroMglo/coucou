# Revisão de Sistema — 07/11/2026

Pedido posterior do utilizador: usar as definições/widgets já existentes na estação e permitir aos pets controlá-los de forma autorizada. **Não construir definições próprias de Wi-Fi, Bluetooth, Som e restantes componentes para os substituir.**

Esta decisão substitui a intenção anterior de criar outro centro de Sistema e retirar `controlCenterButton`. Preservam-se os sete widgets, a UI atual, preferências e acesso manual com o Coucou desligado. O Coucou mantém a gestão de pets, grants, aprovações e trabalhos, e integra os serviços/IPC existentes.

A matriz pet×subplugin×operação×recurso e todos os 43 IDs originais, mais os seis Guardian, permanecem no objetivo. Spotify e Ollama podem partilhar Som com grants distintos. Um pet pode ser designado responsável continuado por uma componente/configuração sob os seus grants. Ser dono durante a execução significa assumir a operação/tarefa e reservar temporariamente o recurso autorizado, mantendo o controlo humano.

A interface ativa observada é DMS/Quickshell. Foram lidos os handlers existentes de centro de controlo/definições e as fontes declaradas em sysworkspace. Não se abriram painéis nem se alteraram configurações, rede, dispositivos ou instalação. Presença de IPC não comprova todas as ações ou aceitação nativa.

O backend reutiliza os brokers e executores Coucou já funcionais e as APIs/serviços da interface existente. A navegação e a mutação são capacidades distintas. Chamadas legadas/leases de ocultação e dependências de painéis próprios têm de ser migradas; o acesso humano não passa pela autoridade de um pet ou pelos pedidos do Coucou.

Contrato normativo detalhado: [PET-SYSTEM-INTEGRATION.md](../docs/PET-SYSTEM-INTEGRATION.md). As fontes históricas em `fontes/` e o commit documental anterior conservam-se intactos. Esta revisão é local e não autoriza commits, pushes, instalação ou alterações persistentes no anfitrião.

## Responsável principal pelas definições

Quando designado pelo utilizador, o System Guardian pode ser o responsável continuado pelas componentes de Sistema concedidas. Os serviços existentes mantêm o estado efetivo, os widgets mantêm a interface e o runtime conserva autoridade e arbitragem. Não é proprietário dos ficheiros nem concede a si próprio acessos. Spotify/Ollama podem conservar Som por grants próprios; o acesso humano e o trabalho independente não dependem do Guardian. A designação não ativa serviços, remediação automática ou permissões.

## Impacto no caminho para produção

A decisão elimina a construção/manutenção de ecrãs Coucou equivalentes às definições já usadas e a substituição do centro de controlo. Mantém adaptação de operações, permissões, eventos, concorrência, recuperação, hardware e aceitação instalada. Não existe uma baseline de esforço restante comparável que sustente uma redução em horas/dias; por isso esta revisão não atribui um novo prazo global nem reutiliza estimativas históricas. O trabalho é retirado do âmbito duplicado, sem fechar SYS/GUA por documentação.
