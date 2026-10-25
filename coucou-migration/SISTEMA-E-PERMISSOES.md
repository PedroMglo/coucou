# Sistema, componentes e matriz de permissões

Contrato desejado de **25/10/2026**. Completa o capítulo 17 da [arquitetura](ARQUITETURA-COMPLETA.md) e os requisitos SYS-01/02/03, EXE e VOICE. A matriz integral ainda precisa de implementação e aceitação; este documento não ativa acessos ou dispositivos.

## 1. Intenção do utilizador

Os controlos próprios do Sistema devem integrar-se completamente com os pets. A finalidade é permitir que cada pet consulte e controle apenas as componentes concedidas, usando o executor real da componente. A apresentação visual é uma superfície desse backend e não a razão suficiente para o substituir.

O plugin principal Sistema é uma opção por pet. Contém subplugins selecionáveis e operações de leitura/controlo independentes. Ativá-lo não concede o computador inteiro. A relação é muitos para muitos: Spotify e Ollama podem ter Som em simultâneo, com âmbitos próprios.

| Exemplo de intenção | Resultado esperado |
| --- | --- |
| Spotify: Sistema ativo; Som autorizado | Acesso às operações/recursos de som concedidos; Rede, Bluetooth e Energia continuam sem acesso implícito. |
| Ollama: Sistema ativo; Som autorizado | Pode usar Som ao mesmo tempo que Spotify; conflitos são coordenados no executor. |
| Ollama: Rede e outras componentes selecionadas | Só as operações e recursos efetivamente concedidos ficam disponíveis. |
| Retirar Som a Spotify | Bloqueia novos pedidos e invalida pendentes desse pet, preservando o grant de Ollama. |
| Acrescentar um subplugin numa atualização | Fica sem concessão herdada; requer seleção própria. |

Estes exemplos definem a arquitetura; não aplicam concessões reais nesta estação.

## 2. Modelo de autoridade

A chave de uma concessão é **pet × subplugin × operação × recurso/âmbito**. Guarda identificação/revisão, validade, restrições, origem da decisão e limites. Leitura e mutação podem ser concedidas separadamente. O catálogo indica suporte vivo; ter uma permissão não inventa um handler ou dispositivo.

O PetId é derivado do contexto nativo autenticado. Não é aceite como autoridade porque o modelo enviou esse argumento. As superfícies humanas de Sistema e os pets chegam ao mesmo executor com autoridades próprias. Um pet não reutiliza o ticket de uma janela de configuração nem precisa de abrir essa janela em cada ação autorizada.

Os alvos são referências opacas ligadas à identidade/generation/revisão observadas. Um nome de rede ou de dispositivo não é identidade suficiente. Um pedido captura alvo, operação, argumentos, dono, revisão e prazo antes de entrar na fila. Retirar a concessão impede que um pedido antigo ganhe a época nova ao reativar o plugin.

As decisões nativas de NetworkManager, BlueZ, compositor ou serviço equivalente continuam ligadas à operação exata. Segredos são voláteis e tratados no fluxo apropriado, sem entrar no chat, prompt, argumentos de processo ou logs. A disponibilidade do comando de leitura não certifica a mutação ou a autenticação.

## 3. Unidades de desenvolvimento

**Rede e Bluetooth formam um grupo de execução**, como pedido pelo utilizador. Partilham princípios de descoberta, alvos, prompts, revisões, concorrência e reconexão. Isso não funde as concessões: um pet pode receber Bluetooth sem receber Wi-Fi.

Som/Multimédia é outra unidade coerente. Ecrã e Energia/Sessão têm subunidades próprias com aceitação completa. Cada unidade inclui contrato, executor, política, estado, controlos funcionais e verificação. Não se considera uma feature terminada por existir apenas um botão ou DTO.

## 4. Catálogo de âmbito a preservar/completar

O catálogo abaixo consolida os grupos descritos na arquitetura histórica. Só se anunciam as operações suportadas pela plataforma, serviço e dispositivo reais. A contagem de grupos não corresponde a funcionalidades prontas.

| Componente | Operações e dados pretendidos | Verificação/limites |
| --- | --- | --- |
| Wi-Fi | Estado do gestor/adaptador, scan suportado, rádio, redes disponíveis/guardadas, ligação/desligação e autoconnect. | Alvos inequívocos, prompts nativos, estado posterior separado do ACK; não desligar rádio nos testes atuais. |
| Ethernet | Dispositivos, estado, perfis/ligação e configurações expostas. | Apenas handlers observados; não inferir controlo por existir lista. |
| Rede móvel | Estado e perfis disponíveis quando existe hardware/serviço. | Ausência explícita; dados sensíveis e autenticação próprios. |
| VPN | Perfis, estado, ligar/desligar quando suportado. | Identidade do perfil e resultado posterior, sem exportar segredos. |
| Hotspot | Disponibilidade, configurar/iniciar/parar. | Credenciais/exposição e efeitos sobre a ligação atual requerem política própria. |
| Bluetooth | Adaptadores, estado, descoberta, dispositivos, ligação, emparelhar, confiança e remoção; codecs quando suportados. | Prompt ligado ao pedido/dispositivo exatos; efeitos físicos e reconexão observados. |
| Reprodução e som | Música/MPRIS, aplicações/streams, volume, mute, dispositivos, perfis, portas e balanço. | Permissão de música separada de som global; rever streams e efeitos suportados. |
| Microfone/captura | Entradas, seleção, volume/mute e perfis aplicáveis. | Mudar microfone pode mover capturas/trocar perfil Bluetooth; não tratar como ação sem impacto. |
| Luminosidade | Dispositivo/monitor, estado e ajuste suportado. | Dependência do compositor/hardware; verificação por leitura. |
| Luz noturna | Estado, temperatura e agenda suportados. | Não prometer controlo que o compositor não oferece. |
| Preferências de sessão | Tema e outras opções efetivamente presentes. | Persistência e estado aplicado, além do ACK. |
| Não incomodar | Estado e política de notificações. | Separar silêncio de indisponibilidade; preservar decisões urgentes conforme política. |
| Inibição de suspensão | Estado, aquisição e libertação suportados. | Posse e fim da inibição ligados ao trabalho; não deixar concessões órfãs. |
| Energia e bateria | Perfis, bateria e detalhes/componentes disponíveis. | Preservar indicadores; disponibilidade/erro explícitos. |
| Ações de sessão | Bloquear, suspender, hibernar, terminar sessão, reiniciar e desligar quando suportados. | Decisões e alcance próprios; não são ensaiadas no anfitrião por autorização genérica de desenvolvimento. |
| Disco, periféricos e componentes | Preservar dados/ações de discos, CPU, periféricos e plugins ativos. | Inventário real antes de migrar; não remover grupos opcionais presentes. |
| Ferramentas e apresentação | Color picker, edição/layout, definições, scroll e contexto existentes no acesso antigo. | Paridade funcional e atalhos antes da retirada seletiva; melhorias visuais adiadas. |

Calendário, música, meteorologia, notificações e definições rápidas mantêm o seu papel no centro integrado. A consulta de uma fonte meteorológica ou serviço tem a configuração e cobertura próprias; não é uma permissão implícita para comunicação externa arbitrária.

## 5. Ciclo de uma operação

1. Resolver capacidade e recurso atuais; capturar caller, concessão e alvo.
2. Validar argumentos, política, revisões, prazo e recurso físico.
3. Persistir a intenção necessária antes da mutação.
4. Obter a decisão/autenticação nativa aplicável e revalidar antes do despacho.
5. Despachar uma vez; distinguir receção, ACK e fim do worker.
6. Observar a pós-condição suportada e guardar recibo/estado incerto.
7. Publicar ao dono original, preservando revogação e resultados tardios.

Se o ACK se perde, não se repete a mutação automaticamente. Cancelar a observação não desfaz o comando. Não se envia uma operação oposta como compensação improvisada. A reserva física permanece enquanto há IO/limpeza próprios por terminar.

O transporte local verifica serviço/socket, UID e continuidade do processo quando aplicável. Trata greeting, respostas assíncronas, correlação, tamanho e prazo. A aceitação de um protótipo de leitura não valida todos os handlers de ação ou a interface instalada.

## 6. Revogação e concorrência

Vários pets podem partilhar um recurso. O executor serializa mutações incompatíveis e conserva o dono de cada pedido. Uma concessão não confere exclusividade permanente. Uma alteração do dispositivo exige nova resolução, sem reutilizar o mesmo nome como se fosse o alvo anterior.

Desativar Sistema/subplugin, alterar a concessão ou perder o caller invalida pedidos pendentes e novas fases do trabalho. O backend conserva os efeitos já enviados e o que falta observar. Revogar Spotify não altera a política de Ollama. Uma delegação também não amplia acesso às componentes.

## 7. Centro legado, widgets e Wi-Fi

Os sete elementos atuais a preservar são `controlCenterButton`, `gravacao`, `systemTray`, `udiskieMenu`, `clipboard`, `notificationButton` e `battery`. Indicadores personalizados e acesso aos componentes também são preservados.

Apenas `controlCenterButton` é candidato a retirada seletiva futura, depois de inventário do que está ativo e paridade funcional/visual aceite. Abrir o painel antigo como fallback pode preservar acesso temporário; não comprova que o substituto está completo. Os outros seis elementos não são removidos por esse corte.

O desenvolvimento de uma operação explícita de desligar Wi-Fi é capacidade de produto, não configuração pretendida da estação. O utilizador quer manter Wi-Fi. Os testes atuais não autorizam desligar a rede, emparelhar dispositivos ou alterar perfis físicos.

## 8. Aceitação requerida

Fixtures cobrem recusas entre componentes, plugin principal sem grant global, alvos repetidos/substituídos, pedido antigo após reativação, prazos, prompts errados, ACK perdido, concorrência e revogação isolada. Percursos nativos verificam o serviço real e o efeito no alvo autorizado.

O cenário central usa Spotify e Ollama com Som: ambos consultam/controlam apenas o recurso concedido; retirar um não afeta o outro; Rede/Energia continuam recusadas. Voz usa exatamente os mesmos contratos. Uma simulação de áudio/rede não fecha aceitação do hardware ou da instalação.

As provas históricas e os protótipos existentes estão no [arquivo](fontes/docs/PETS-ARCHITECTURE-HISTORY-2026-10-23.md). Os números e prioridades antigos não substituem esta especificação nem indicam ativação atual.
