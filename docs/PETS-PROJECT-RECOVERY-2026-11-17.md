# Recuperação dos dados de projeto — 17/11/2026

O percurso privado de preservação e restauro passou na composição atual: fontes
1340/SOURCE3, binário `6873c5e830df847f9850fa588946f5bb5e72c176abdef3a8f31161ee2d03105a`.
Reutiliza a manutenção nativa existente. Não acrescenta código de produto,
ledger, scheduler, formato ou autoridade. Não foi instalado.

Foram preservados três projetos, as notas, os três objetivos anteriores e duas
seleções com snapshot completo do projeto. As respostas do objetivo local e de
uma conversa Codex nova permaneceram consultáveis após reinício e restauro.
A tentativa revogada conserva `uncertain`, sem saída aceite e com reserva de
65536 bytes. Os registos assinados foram verificados e mantidos em quarentena.

O histórico restaurado declara `restoredSnapshotOnly`, `cumulativeCharge:null`,
`accountingComplete:false` e estado posterior ao snapshot desconhecido. A prova
dos valores anteriores vem dos bytes preservados e das assinaturas; não se
apresenta esse histórico como contabilidade atual completa ou autorização viva.

## Percurso verificado

1. Backup nativo do perfil QA do objetivo anterior, revisão, restauro inativo
   integralmente idêntico ao arquivo, quarentena da autoridade, revisão da
   configuração e admissão com escolhas explícitas e `grants:[]`.
2. Consulta real pela UI/IPC Tauri dos projetos/notas, contexto e resposta dos
   objetivos. Claim e retoma recusados. Reinício sem execução.
3. Conversa Codex nova com projeto e nota escolhidos, revisão nativa completa e
   consentimento de dados inicialmente desmarcado. Uma única chamada de modelo
   sintética pelo protocolo CLI real do adaptador. O modelo é explícito em
   `thread/start`; o `turn/start` inclui apenas os dados escolhidos. A nota não
   entra no histórico público. O código atual persiste `NativeContext2`, com
   CID, sessão, projeto/revisão e referências da nota corretos.
4. Segundo backup/restauro nativos, agora incluindo os metadados de recuperação
   existentes. Conserva os dados operacionais e a proveniência da sessão Codex.
   Uma revisão QA das preferências aprova apenas os dados e mantém voz, mobile,
   autostart, atalhos, hooks e credenciais suspensos. Conserva os bytes das
   preferências e não renova grants. Depois dessa barreira, o envio na conversa
   histórica é recusado especificamente por ser histórico restaurado.
5. Nova consulta gráfica nativa, reinício e verificação de hashes: resposta,
   critérios, projetos e metadados presentes, sem replay CLI ou HTTP.

## Provas e limites

[Recibo agregado e composição exata](../.build/project-recovery-20261117/VERIFIED.json),
[22 recibos terminais de manutenção](../.build/project-recovery-20261117/MAINTENANCE-RECEIPTS.json),
[execuções e falhas conservadas](../.build/project-recovery-20261117/RUNS.json),
[preservação da instalação e fontes](../.build/project-recovery-20261117/PRESERVATION.json).
Os caminhos privados não fazem parte da publicação Git.

A execução11 guardou a resposta Codex, mas falhou depois por procurar o modelo
no campo errado do protocolo. Conserva o resultado original de falha; o
suplemento verifica o turno já concluído. Não o repetimos. As execuções12 e15
validam leitura/reinício e segundo restauro desse mesmo resultado. As restantes
falhas do observador estão discriminadas nos recibos, incluindo prontidão da UI,
invalidação do catálogo após uma nota e a barreira independente de preferências.

Num ensaio anterior, o PATH da fixture foi atribuído depois de arrancar o
WebDriver: o CLI encontrado recusou a conta no HOME privado vazio, antes de
qualquer turno. O ensaio corrigido verifica HOME/PATH do processo nativo e usa
apenas a fixture. Houve um único turno sintético nesta unidade, zero inferências
reais/pagas e nenhuma conta pessoal usada. O ID sintético da sessão não é UUID;
a recusa ao tentar classificá-lo como sessão externa prova apenas validação do
ID, não aceitação do percurso externo com uma sessão Codex real.

As imagens privadas observadas permitem ler a resposta histórica do objetivo,
a resposta Codex, o aviso de restauro e o compositor desativado. A imagem do
consentimento mostra a nota e a caixa desmarcada; a revisão completa está
parcialmente fora do recorte. Isto não é aceitação gráfica global, humana ou da
instalação operacional. A instalação44, o launcher, preferências e RPM49
congelado mantêm os hashes anteriores; a decisão própria para49 continua pendente.
Não houve commits, pushes, merges ou alterações persistentes do anfitrião.

O requisito integral de projetos continua aberto. Segue a consulta do progresso
do projeto a partir dos objetivos/resultados existentes, com acesso explícito
às provas e sem outro armazenamento. Planeamento, partilha, atualização de
contexto e recorrência de projeto exigem os seus próprios contratos e validação.
A comparação integral com o Coucou oficial e a entrega instalada continuam com
os limites registados na matriz de paridade; esta prova não os encerra.
