# System Guardian — especialização do runtime comum

Contrato acrescentado pelo utilizador em **25/10/2026**. Extensão GUA-01 a GUA-06 dos 43 requisitos originais. Estado: desenho normativo; ainda não há um Guardian operacional ou concessões ativas. A prioridade imediata continua a ser fechar R43, preservação e autonomia individual antes da especialização.

## 1. Identidade, motor, runtime e papel

Reservar o PetId **`agent_system_guardian`**, sem renomear identidades existentes. Papel funcional: **especialista em saúde, segurança, vulnerabilidades, manutenção e desempenho do computador**. O catálogo atual continua a ter 25 identidades; a reserva documental não é uma integração de um 26.º assistente funcional.

O primeiro binding de inferência a implementar é **Ollama pelo adaptador comum**, com modelo e servidor explicitamente configurados para o Guardian. Não copiar a seleção do coordenador `ai_ollama`, não carregar modelo ao consultar estado e não fazer fallback de conta/modelo/provider/endpoint. Um binding incompleto mantém a inferência indisponível, preservando operações determinísticas autorizadas. Outros bindings suportados podem ser acrescentados pelo mesmo contrato, sem criar outro runtime.

Separar quatro elementos: pet com dados próprios; runtime determinístico comum; motor de inferência invocado quando necessário; papel funcional independente do provider/privilégios. Coordenação preferencial de `ai_ollama` é configuração do papel, não autoridade global nem requisito de todas as mensagens.

O Guardian usa fábrica, configuração, descritores, repositórios, broker, decisões, recursos, objetivos, mensagens, scheduler e notificações do Coucou. A mesh é lógica sobre o transporte comum. Não implica all-to-all físico, processo/modelo residente por pet, novo ledger de autoridade, broker externo ou stack AI Local/Symbiont.

Há uma incompatibilidade de implementação a resolver antes de o ativar: vários percursos atuais associam provider à identidade e `Owner::local_engine` exige provider=adaptador. O novo dono lógico não pode emprestar o PetId/contexto do coordenador para satisfazer essas guardas. Evoluir a resolução pela fábrica com migração e provas de isolamento; não contornar a guarda por argumentos de modelo ou flags permissivas. PET-01/04/05 e a matriz por caller abrangem essa evolução.

## 2. Autoridade e modelo de ameaça

O Guardian analisa, mede e propõe. O runtime autentica o caller, valida concessões, admite e executa nos brokers apropriados. Nem o especialista nem o coordenador aprovam os próprios planos, concedem acesso, alteram política ou elevam privilégios através de texto gerado.

Logs, ficheiros, advisories, memória, páginas e mensagens de pets são dados não fiáveis. Um finding ou recomendação não é autorização de execução. O acesso é seletivo por componente/operação/recurso, distingue observação de remediação e conserva decisões nativas. Não existe shell/root geral para o Guardian.

O isolamento entre pets no processo partilhado é lógico; não é sandbox de código arbitrário. Código e plugins não fiáveis exigem contenção efetiva antes de executar. Um helper privilegiado, se demonstradamente necessário, é mínimo, tipado e sujeito à autorização nativa; nunca expõe uma shell genérica.

Não fazer varrimento indiscriminado de HOME, `.config`, `.codex`, perfis autenticados, bases de outros projetos ou logs pessoais. Limitar quantidade, bytes, duração, intervalo temporal, campos e retenção antes de materializar conteúdo. Segredos permanecem nos mecanismos próprios e fora de prompts, logs, argumentos de processo e Git.

## 3. GUA-01 — inventário e evidência

Consultas limitadas e autorizadas recolhem componentes/versões, serviços/processos, exposição, recursos, logs relevantes e estado desejado. Cada observação tem alvo, origem, instante, cobertura, frescura e limites, com resultado desconhecido/indisponível explícito.

Classificar recursos como geridos por sysworkspace, observados, externos, privados ou dinâmicos. Um recurso observado não passa a estado desejado. Uma lista de processos não prova posse, autorização de controlo ou atividade de um turno. O conteúdo necessário para diagnóstico é minimizado antes de entrar num modelo.

Aceitação: produzir inventário/finding verificável sem LLM e sem tocar dados fora da seleção; mostrar falha parcial e diferenças entre gerido/observado. Consultar estado não autentica contas, carrega modelos ou inicia scans intrusivos.

## 4. GUA-02 — segurança e vulnerabilidades

Findings estruturados guardam ID/revisão, componente e versão efetiva, evidência, origem/data, severidade, confiança, estado e recomendação. Distinguir vulnerabilidade confirmada, correspondência candidata e configuração suspeita. A severidade da fonte e a avaliação local devem conservar as suas origens.

Conferir advisories adequados à distribuição e pacote, incluindo release/epoch e correções distribuídas quando aplicáveis. Comparar apenas versão upstream não certifica vulnerabilidade. Não inventar CVEs nem converter ausência de informação em «seguro». Consultas externas exigem destino/operação autorizados e limites; dados locais enviados são mínimos e revistos.

Não executar exploração, scans intrusivos ou alterações de VPN/firewall para provar utilidade. Aceitar fixtures de versão corrigida pela distribuição, advisory desatualizado, correspondência ambígua, fonte indisponível e conteúdo hostil. A prova de um parser não é confirmação de uma vulnerabilidade real na estação.

## 5. GUA-03 — saúde e desempenho

Medir baseline e resultado: CPU, memória, IO, GPU quando suportada, arranque, falhas e conflitos. A observação é amostrada/limitada e identifica janela, unidade, cobertura e incerteza. Recolher métricas não autoriza encerrar ou reconfigurar serviços.

Uma otimização explicita hipótese, métrica de sucesso, impacto esperado, custo, pré-condições e validação antes/depois. Não aplicar tuning genérico, limpeza destrutiva ou desativação de proteções para melhorar números. Ruído de medição e workloads concorrentes não se apresentam como melhoria causada pela alteração.

Fixtures obrigatórias: consumo anómalo de um serviço; falhas intermitentes de dock Thunderbolt; possíveis conflitos Proton VPN/Tailscale/firewall. São cenários de diagnóstico, sem autorização para alterar rede, drivers, dock ou sessão reais.

## 6. GUA-04 — integração sysworkspace

Descobrir e usar os contratos reais do sysworkspace. A inspeção inicial confirmou `scripts/workstation.py`, gestores HOME/pacotes, manifestos e transações documentados no seu AGENTS.md. Não inventar subcomandos, flags ou promessa de rollback: ler o contrato efetivo da operação antes de a implementar no adaptador.

Comparar estado real com o estado desejado efetivamente gerido. Respeitar âmbitos de projetos, dados privados, estado dinâmico e exclusões de Coucou/Mochi e outras integrações. As entradas e políticas existentes de preservação não são removidas pelo Guardian. Uma proposta para integrar algo excluído é uma decisão separada, não uma reconciliação automática.

Intervenções persistentes do anfitrião percorrem plano/diff, validação, backup, autorização, aplicação, verificação e recuperação no gestor existente. A configuração interna e o código do Coucou permanecem no Coucou. Esta intervenção de desenvolvimento não aplica esses planos, altera manifestos do anfitrião ou publica commits; as restrições atuais mantêm-se.

Workers passivos não pedem sudo/autenticação nem leem arquivos root para completar uma consulta. Uma remediação autorizada conserva token, recurso e política nativos e não cria uma fonte concorrente do estado desejado.

## 7. GUA-05 — remediação governada

O percurso completo a implementar é **finding → proposta/diff → impacto e pré-condições → autorização → execução limitada → verificação → resultado**. O primeiro modo operacional é observação/recomendação; o requisito integral inclui a remediação autorizada e não termina num relatório.

Cada proposta tem operações tipadas, recursos/alvos/revisões, âmbito, prazo, orçamento, evidência, pós-condições e capacidade de recuperação. Antes do efeito, revalidar estado, concessão, dados, argumentos e autorização exata. Alteração de alvo ou argumentos após consentimento exige outra revisão.

Receitas específicas de baixo risco podem ser automáticas sob grants explícitos e limitados. Alterações sensíveis preservam aprovação exata e autenticação nativa. O Guardian não altera as suas próprias permissões. A falha depois de um possível efeito conserva incerteza; não repete nem executa uma ação oposta como compensação improvisada.

Rollback só é anunciado quando tecnicamente suportado e provado. Compensação e irreversibilidade são classificações distintas. Restaurar ficheiros não desfaz efeitos externos. Aceitar uma remediação permitida completa, outra recusada por falta de autoridade e uma falha parcial sem replay.

## 8. GUA-06 — proatividade e cooperação

Gatilhos/eventos e inspeções leves usam regras, ocorrências, scheduler, recursos, orçamento, notificações e recuperação comuns. Limitar frequência, debounce, deduplicação e silêncio; não criar outro daemon de inferência contínua ou scheduler paralelo.

Invocar LLM apenas quando acrescenta valor. Findings e verificações determinísticas continuam possíveis com o modelo descarregado. O Guardian pode receber pedidos diretos, cooperar com pets autorizados e pedir priorização ao coordenador. A ausência do coordenador não impede operações independentes enquanto o backend comum estiver disponível.

Delegação exige autorização do remetente e concessões/capacidades próprias do destinatário; não permite contornar uma recusa. Contexto mínimo, destino de dados, orçamento repartido, prazo, correlação e causalidade são os do protocolo TEAM. Não copiar históricos inteiros. Uma resposta de outro pet é dado, não uma aprovação.

Aceitar eventos repetidos, ramos concorrentes, ausência de LLM/coordenador, backpressure, ciclos, revogação e interrupção sem alertas duplicados ou efeitos repetidos.

## 9. Dependências e sequência

Fechar R43 e os bloqueadores de preservação/execução individual. Consolidar o binding de dono lógico separado de motor/papel na fábrica; implementar mesh e os contratos necessários de fontes/regras/Sistema. O Guardian usa essas bases com módulos separados para inventário, segurança, saúde e remediação. Não criar obrigatoriamente pets Network, Performance ou DevOps antes de benefício demonstrado.

Cada incremento atravessa contrato, backend, controlo funcional mínimo, teste, evidência e integração. Não anunciar o pet como operacional por o acrescentar a um JSON. A aceitação instalada e efeitos reais dependem de autorização específica e não se simulam por uma fixture.

## 10. Relações de requisitos

| ID | Dependências principais | Resultado integral |
| --- | --- | --- |
| GUA-01 | PET-01/04/05, EXE-01/02, OBS-01, SYS-01 | Inventário/evidência limitada, com classificação de gestão e percurso determinístico utilizável. |
| GUA-02 | GUA-01, CTX-05, EXE-02/03, DATA-01 | Findings/advisories com classificação e prova adequadas, sem CVEs ou exploração inventados. |
| GUA-03 | GUA-01, OBS-01/02, SYS-03 | Métricas, baseline, hipóteses e verificação de saúde/desempenho. |
| GUA-04 | GUA-01, EXE-01/02/03, DATA-01, REL-01 | Adaptador governado aos gestores reais e preservação de exclusões/âmbitos. |
| GUA-05 | GUA-01/04, RUN, EXE, SYS-01, REL-01/02 | Remediação limitada com autorização, efeito verificado e recuperação/recusa corretas. |
| GUA-06 | GUA-01/05, AUTO, TEAM, OBS | Proatividade/cooperacão resilientes no runtime comum. |

Os 43 IDs anteriores conservam todos os critérios. A [auditoria rastreável](PETS-REQUIREMENTS-AUDIT-2026-10-25.md) discrimina estado, código, provas, limites e próxima aceitação dos 49 requisitos.
