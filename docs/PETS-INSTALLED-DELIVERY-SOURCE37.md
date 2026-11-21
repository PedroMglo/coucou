# Proposta de entrega45/Source37 —13/11/2026

**Preparada e validada no âmbito do chat do telefone; não instalada nem autorizada.**

**Correção posterior:** Source38 corrigiu perda do cartão n8n ao selecionar a
conversa, reproduzida neste RPM45. O arquivo45/37 permanece exato e não contém
essa correção. Preparar uma normal46 identificada/revista para a próxima
entrega; autorização35 ou eventual autorização deste RPM45 não se transfere.
Ver PHONE-N8N-SOURCE38-VERIFIED.json e a matriz para fontes/âmbitos.
A instalação operacional continua44/Source35. A autorização anterior35 não
abrange esta substituição. Commits do fork, pushes e merges continuam suspensos.

## Mudança concreta

A admissão nativa pode recusar antes de HTTP quando a configuração/perfil de
inferência está ocupado. A35 classificava essas duas recusas como falha de rede.
A37 conserva a correção36: usa o busy/mensagem já existentes e identifica a
contenção no diagnóstico. Um ficheiro funcional mudou relativamente à35,
mais cinco ficheiros de versão44→45. Sem dependências novas, migração, retry,
fallback, mudança de destino, provider, modelo, conta, autoridade ou permissões.

## Artefactos fixados

- Versão normal:0.5.1-mochi.45; identificador fr.louisraille.coucou.
- RPM: .build/release-checkpoints/Coucou-0.5.1-mochi.45-native-e91116b940d243d5.rpm.
- SHA-256 RPM: e91116b940d243d581eb27c72aac8463f2a725c8db240bc661a11191c338fdbe.
- Executável empacotado: 8c49d5201067ddc65c1d8afe7e81f5a8fccf70a6f02d5f671ce78419266fd3c2.
- Arquivo: .build/release-checkpoints/Coucou-0.5.1-mochi.45-source-6fe84e4cfb3c2f1e.tar.gz.
- SHA-256 arquivo: 74589ae0c0ff63b15e0798afe4b5b93dd57820dd3ed7c777cace69ed93aa01cf.
- Manifesto: .build/upstream-parity-20261111/SOURCE37-PHONE-ADMISSION-RELEASE45.json.

Compilação nativa nova,1323 inputs iguais durante o build, Cargo locked/offline,
receita Fedora existente;13 payloads permitidos e relay preservado. ELF e
bibliotecas verificados. O prefixo extraído está apenas no projeto.
FORK45-SOURCE37-BUILD.json e SOURCE37-DELTA.json fixam o delta e os artefactos.

## Prova e limites

O próprio executável do RPM normal45 passou UI privada/modelo explícito/TLS com
pin,7 HTTPs sintéticos, conversa/continuação/reset, Stop/Busy, revogação,
retirada de modelo, isolamento da conversa desktop, reinício/preferências e
nenhum replay/resposta tardia.112 consultas de estado só de leitura. Duas
atestações reais do processo confirmam HOME/XDG/roots de credenciais/Secret
Service privados; rede só loopback e ecrã privado. Capturas foram observadas
antes dos efeitos. QA encerrado; instalada35/settings/pets/launcher/relay iguais.
PHONE-NORMAL45-SOURCE37-VERIFIED.json agrega os recibos privados.

O controlo negativo usa Captured::capture real com mutex ocupado: antes falha
Unreachable≠Busy; depois11 testes dirigidos passam. Esse backend é byte a byte
igual36→37; não se repetem os testes por mudar só a versão. A contenção inicial
não foi forçada no ensaio nativo. A causa exata dos incidentes22/82 continua
aberta. O primeiro arranque de QA45 falhou antes da app porque a sandbox
impediu o Podman; conservado separadamente, sem o classificar como defeito da app.

Sem Android físico, aceitação humana/instalada/global, inferência real/paga ou
quota Copilot/GitHub. Provas anteriores de voz, código/diffs, serviços,6/7 e
recursos mantêm as suas fontes e limites; não são promovidas a ensaios nativos37.

Prova adicional posterior à proposta inicial: o mesmo RPM45 foi comparado com
o oficial nas perguntas móveis, com duas respostas simples/múltiplas Unicode
ao socket por composição, dez fronteiras e reinício sem replay. Rota Coucou
explícita apenas no perfil de QA; zero modelo/IDE real. Recibo privado
PHONE-QUESTIONS-SOURCE37-VERIFIED-COMPARISON.json. Os artefactos e o patch da
estação não mudaram; não amplia o âmbito instalado proposto nem autoriza45.

## Âmbito que exige autorização própria

1. Concluir primeiro o registo final assinado35 já autorizado, quando a YubiKey
   estiver disponível; preservar o índice e alterações alheias.
2. Rever/aplicar o patch de quatro fontes Coucou em sysworkspace, preparado
   apenas em .build/upstream-parity-20261111/installed-source37-proposal/.
   REVIEW.json fixa bases/hashes. Patch dry-run, JSON e sintaxe do launcher
   passaram. Registo focado e assinado antes da nova substituição; manter hooks.
3. Rever atividade/IO de arranque e fechar normalmente só o Coucou44 quando
   não interromper trabalho ativo. Backup nativo específico dos dados Coucou,
   revisto por fingerprint e verificado pelos leitores44/45; sem copiar HOME,
   cofres, credenciais ou modelos recursivamente.
4. Preparar apenas .local/opt/coucou/0.5.1-mochi.45 pelo instalador restrito.
   Preservar44 e43. Aplicar pelo gestor HOME apenas .local/bin/coucou, com backup
   transacional. Sem alterações de autostart, serviços, compositor ou rede.
5. Verificar dono/binário/versão/UI/dados/escolhas após o arranque normal.
   Sem fixtures/grants/modelos novos no perfil pessoal, inferência de conversa
   real/paga ou quota real Copilot/GitHub. Se atividade de arranque não permitir
   cumprir esse limite, parar antes da substituição.
6. Concluir recibos e registo focado assinado da entrega em sysworkspace.
   A exceção de commits seria exclusivamente para esta integração da estação;
   fork/push/merge permanecem suspensos.

O patch não foi aplicado à estação e o prefixo pessoal45 está ausente. O registo
final35 foi concluído em1462828 com assinatura verificada, preservando o índice
alheio e o commit de rede c5c7b2b. A autorização45 continua pendente.
Não será repetida a instalação35. Não há downgrade/restauro automático: um
eventual regresso de launcher exige rever a compatibilidade dos dados.

A confirmação decorre do limite explícito do proprietário antes de nova
substituição instalada. A [skill da estação](/home/pmglo/.agents/skills/sysworkspace-system/SKILL.md)
exige «concluir com um commit focado no sysworkspace»; por isso a proposta
inclui apenas essa exceção de registo assinado, sem publicação Git.
