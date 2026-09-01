# Plano de trabalho contínuo

Este arquivo é o ponto de continuidade do trabalho prático no MeliponarioManager.

Ele existe para que o desenvolvimento possa continuar entre conversas diferentes sem depender do histórico do chat. Toda sessão relevante deve atualizar este documento antes de encerrar uma etapa importante.

## Branch de trabalho

`work/field-testing-and-hardening`

Base inicial: `main` após o merge do PR #46 (`7640c69`).

## Objetivo desta fase

Sair do ciclo em que a aplicação foi majoritariamente construída e validada por automação e entrar em uma fase orientada por revisão humana e uso real:

- auditar o projeto inteiro antes de ampliar funcionalidades;
- localizar código frágil, improvisado, desnecessariamente complexo ou pouco profissional;
- testar a aplicação manualmente com dados e fluxos reais;
- registrar qualquer comportamento estranho, regressão ou atrito observado;
- corrigir os problemas encontrados sem perder contratos já estabilizados;
- fortalecer testes automatizados sempre que um problema real revelar uma lacuna;
- manter integridade, rastreabilidade e recuperação de dados como prioridades máximas.

## Regras de trabalho

- Não alterar migrations existentes.
- Mudanças de schema novas exigem migration nova e justificativa explícita.
- Não realizar restauração destrutiva contra a base real do usuário durante testes.
- Antes de corrigir um bug reproduzível, registrar o comportamento e, quando viável, adicionar teste de regressão.
- Preservar contratos IPC e formatos persistidos salvo decisão explícita registrada neste arquivo.
- Não refatorar apenas por estética ou contagem de linhas.
- Não introduzir frameworks, abstrações ou dependências sem problema concreto que justifique isso.
- Atualizar este plano ao concluir blocos relevantes de trabalho.
- Manter uma seção de continuidade suficientemente clara para outra conversa retomar o trabalho sem depender do chat anterior.

## Estado atual

- [x] PR #46 integrado na `main`.
- [x] Dívida técnica principal de backend/frontend reduzida.
- [x] Smoke desktop real via WebDriver adicionado ao pipeline da `main`.
- [x] Branch de trabalho prático criada.
- [x] Plano persistente de trabalho criado.
- [x] CI da branch de trabalho colocado em perfil leve para pushes.
- [x] Auditoria estática geral inicial concluída e registrada.
- [x] ACH-001 corrigido e validado por CI completo no checkpoint temporário PR #47.
- [x] ACH-002 corrigido e validado por CI completo no checkpoint temporário PR #48.
- [x] ACH-003 corrigido e validado por CI completo na branch `validation/field-testing-ach-003`.
- [x] ACH-004 corrigido e validado por CI completo na branch `validation/field-testing-ach-004`.
- [x] ACH-005 corrigido e validado por CI completo na branch `validation/field-testing-ach-005`.
- [x] Corrigir primeiro bloco de achados de integridade/consistência.
- [ ] Iniciar rodada sistemática de testes manuais após o primeiro bloco de correções de alto risco.

## Política temporária de CI da branch

Pushes para `work/field-testing-and-hardening` usam um perfil deliberadamente leve:

- validação de versão;
- validação de links/documentação;
- instalação das dependências frontend quando houver mudança de código;
- build TypeScript/Vite;
- testes de UI e contratos estruturais.

Nesses pushes ficam de fora as etapas caras de sistema/Rust:

- instalação das bibliotecas GTK/WebKit de build;
- geração e validação de ícones de bundle;
- `cargo fmt`;
- `cargo check`;
- Clippy;
- `cargo test`.

Essas validações continuam presentes no workflow e voltam automaticamente fora do push leve da branch de trabalho, inclusive em branches de validação e Pull Requests para `main`. A validação completa da própria `main` permanece separada e inclui build desktop e smoke WebDriver.

A primeira execução após essa alteração terminou com sucesso e confirmou que as etapas Rust/sistema ficaram realmente ignoradas no push desta branch.

Antes da integração final desta branch, revisar se a exceção específica de `work/field-testing-and-hardening` deve ser removida do workflow.

## Fase 0 — Auditoria geral do projeto

Objetivo: descobrir o estado real do repositório antes do teste funcional sistemático. Não corrigir tudo por reflexo; primeiro classificar o que é defeito, risco, dívida aceitável ou apenas preferência estética.

A auditoria desta fase é uma revisão estática ampla. Ela não substitui teste manual nem prova ausência de outros bugs. O objetivo foi encontrar riscos evidentes no estado atual e construir uma fila de trabalho racional.

### Estrutura e arquitetura

- [x] Mapear estrutura atual do frontend, backend, migrations, testes, scripts, assets e workflows.
- [x] Conferir se as responsabilidades descritas em `docs/ARCHITECTURE.md` correspondem ao código real.
- [x] Procurar módulos novamente grandes ou com responsabilidades misturadas após o PR #46.
- [x] Procurar fachadas vazias, divisões artificiais e abstrações sem ganho real.

Conclusão: a decomposição do PR #46 é real e coerente. O projeto não apresenta arquitetura de arquivo-monstro ou camadas artificiais generalizadas. Permanecem alguns hotspots densos, registrados como dívida de manutenção, não como motivo para nova refatoração ampla.

### Backend Rust

- [x] Revisar tratamento de erros e fronteira IPC.
- [x] Procurar caminhos de panic explícito, placeholders e código incompleto em produção.
- [x] Revisar transações e operações multi-etapa que precisam ser atômicas.
- [x] Revisar SQL, validação de entrada e invariantes de domínio nos fluxos críticos.
- [x] Procurar duplicação importante de regras e helpers quase idênticos.
- [x] Revisar manipulação de arquivos, backup/restore e caminhos externos.
- [x] Verificar hotspots de legibilidade e manutenção.

Conclusão: backup/restore, arquivos gerenciados, validação temporal e várias invariantes estão bem protegidos. Os principais problemas encontrados estão na coordenação entre fatos persistidos, Agenda e estado retornado para a interface.

### Frontend React/TypeScript

- [x] Revisar páginas, hooks, componentes e clientes IPC principais.
- [x] Procurar estado duplicado, efeitos frágeis, race conditions e chamadas IPC sem tratamento coerente.
- [x] Procurar casts perigosos e `any` explícito.
- [x] Revisar padrão de mutações e recarga após escrita.
- [x] Procurar componentes com responsabilidades excessivas após a decomposição recente.
- [x] Revisar mensagens de erro e carregamento nos fluxos de maior risco.

Conclusão: TypeScript está em modo `strict` e não foi encontrado uso explícito de `as any` na busca inicial. O risco relevante está em semântica assíncrona: algumas mutações confundem falha de recarga com falha de gravação e algumas páginas possuem loaders sem proteção contra resposta fora de ordem.

### Banco e migrations

- [x] Conferir ordem e continuidade das migrations existentes (`0001` a `0017`).
- [x] Revisar constraints, índices, triggers e relacionamentos críticos nas migrations recentes.
- [x] Procurar regras importantes mantidas apenas na aplicação quando deveriam ter segunda defesa no SQLite.
- [x] Conferir testes de migrations e compatibilidade histórica existentes.

Conclusão: há boa segunda defesa de SQLite em ocupação/estado de caixa, Agenda, transporte e anexos. As migrations recentes possuem testes específicos de upgrade/backfill. A identidade cadastral foi harmonizada no ACH-005 na aplicação sem migration nova, porque uma constraint normalizada imediata poderia tornar bancos já existentes com colisões incompatíveis e o `lower()` nativo do SQLite não reproduz a mesma normalização Unicode usada pelo Rust.

### Testes

- [x] Mapear o que é realmente testado e o que é teste estrutural/textual.
- [x] Procurar testes que passam sem exercitar comportamento de interface real.
- [x] Identificar fluxos críticos sem regressão automatizada suficiente.
- [x] Evitar transformar porcentagem de cobertura em objetivo do produto.

Conclusão: o backend possui testes comportamentais úteis. A suíte frontend protege muitos contratos por leitura/regex dos fontes, o que é útil, mas não equivale a renderizar e operar a interface. O smoke WebDriver atual cobre uma navegação real, porém curta. Novos testes comportamentais devem nascer de bugs concretos, não de meta de cobertura.

### CI, supply chain e release

- [x] Revisar gatilhos e responsabilidades de cada workflow.
- [x] Confirmar Actions fixadas por SHA e permissões adequadas nos fluxos principais.
- [x] Revisar caches, lockfiles, auditorias e separação entre CI rápido e validação completa.
- [x] Procurar passos caros sem benefício proporcional durante a fase longa de trabalho.

Conclusão: a base de CI/release está bem endurecida. O único ajuste necessário para esta fase foi o perfil leve específico da branch de trabalho; PR para `main` e `main` continuam mantendo gates pesados.

### Segurança e dados locais

- [x] Procurar sinais de segredos, caminhos pessoais e resíduos comuns de debug versionados.
- [x] Revisar capabilities Tauri e acesso a filesystem/dialogs.
- [x] Revisar exposição de erros internos para a UI nos pontos críticos.
- [x] Revisar operações destrutivas, restauração, substituição e exclusão de arquivos.

Conclusão: não surgiu achado crítico de segurança na revisão inicial. Capabilities são restritas, existe CSP, erros de banco são sanitizados na fronteira comum e os fluxos de backup/restore e arquivos gerenciados possuem proteções relevantes de caminho, integridade e rollback.

### Documentação e coerência

- [x] Conferir documentos centrais contra o comportamento atual.
- [x] Procurar contratos documentados divergentes da implementação.
- [x] Conferir scripts/comandos documentados contra `package.json` e workflows nos pontos afetados.

Conclusão: a documentação está geralmente alinhada, mas `docs/AGENDA.md` expôs uma divergência funcional importante: a execução especializada promete transação única incluindo a próxima tarefa derivada, enquanto o código atualmente reconcilia a Agenda após o commit do fato/tarefa.

### Saída da auditoria

- [x] Registrar achados concretos nesta página com severidade e evidência.
- [x] Separar correções necessárias de sugestões opcionais.
- [x] Definir ordem inicial de correção antes de mudanças amplas.

## Resultado geral da auditoria inicial

O projeto não apresenta sinais de base descartável ou arquitetura improvisada generalizada. As áreas de backup/restore, arquivos gerenciados, migrations recentes, segurança Tauri, CI/release e boa parte das invariantes de domínio estão em estado sólido para uma aplicação `0.x`.

Os problemas encontrados têm um padrão claro: funcionalidades foram adicionadas em várias etapas e algumas regras passaram a existir em uma camada sem serem propagadas para todas as outras. Os maiores riscos são de **estado parcialmente atualizado**, **Agenda temporariamente incoerente**, **mensagem falsa de falha depois de uma gravação bem-sucedida** e **definições diferentes de identidade/duplicidade conforme o fluxo usado**.

Não há justificativa para reescrever o projeto, trocar stack, adicionar framework ou fazer outra decomposição geral neste momento.

## Ordem inicial de correção

1. **ACH-001** — fechar a fronteira transacional entre fato/tarefa e reconciliação da Agenda. **Concluído em 2026-09-01.**
2. **ACH-002** — garantir reconciliação da Agenda quando mudanças de estado invalidarem ou moverem tarefas derivadas. **Concluído em 2026-09-01.**
3. **ACH-003** — separar no frontend sucesso da mutação e falha de refresh. **Concluído em 2026-09-01.**
4. **ACH-004** — corrigir/definir reversão de transferência externa comum antes do teste manual desse fluxo. **Concluído em 2026-09-01.**
5. **ACH-005** — unificar regras de identidade/duplicidade de cadastros e importação antes de criar nova constraint. **Concluído em 2026-09-01.**
6. Testar manualmente esses fluxos e usar os resultados para decidir a prioridade dos achados médios/baixos.

Os achados marcados como **investigando** não devem ser corrigidos até a semântica desejada ser confirmada pelo domínio ou pelo teste manual.

## Fase 1 — Teste manual orientado por uso real

### Inicialização e navegação

- [ ] Iniciar a aplicação com a base de dados real existente.
- [ ] Confirmar carregamento normal da Visão geral.
- [ ] Percorrer todas as páginas principais sem mutações destrutivas.
- [ ] Registrar erros visuais, falhas de carregamento, travamentos ou mensagens inadequadas.
- [ ] Verificar comportamento em resolução compatível com o ambiente real de uso.

### Cadastros e estados principais

- [ ] Revisar meliponários.
- [ ] Revisar espécies.
- [ ] Revisar caixas.
- [ ] Revisar colônias.
- [ ] Verificar buscas, filtros, estados vazios e navegação entre registros.

### Manejo

- [ ] Testar inspeções.
- [ ] Testar alimentação.
- [ ] Testar produção.
- [ ] Testar eventos.
- [ ] Testar manutenção.
- [ ] Observar especialmente validações, mensagens de erro e dados exibidos após salvar.

### Agenda

- [ ] Revisar Agenda e alertas com dados existentes.
- [ ] Criar item manual descartável quando houver contexto seguro.
- [ ] Verificar conclusão/cancelamento conforme os fluxos atuais.
- [ ] Confirmar projeções derivadas e reconciliação sem duplicações aparentes.

### Movimentações e transporte

- [ ] Revisar histórico de movimentações.
- [ ] Revisar ciclo de transporte quando houver dados adequados.
- [ ] Verificar documentos e estados derivados.
- [ ] Confirmar que nenhuma ação permitida viola o estado atual da colônia/caixa.

### Fotos, anexos e arquivos gerenciados

- [ ] Abrir pelo menos uma foto existente.
- [ ] Abrir ou revelar pelo menos um arquivo gerenciado.
- [ ] Verificar estados de arquivo ausente ou inconsistente quando houver exemplo seguro.
- [ ] Confirmar que erros de sistema de arquivos chegam à UI de forma útil e sem vazamento técnico indevido.

### Relatórios e CSV

- [ ] Abrir cada relatório com dados reais.
- [ ] Verificar filtros e estados vazios.
- [ ] Exportar CSV pelo seletor nativo.
- [ ] Confirmar nome e destino escolhidos.
- [ ] Abrir o CSV resultante e conferir estrutura básica.
- [ ] Testar impressão quando houver ambiente adequado.

### Backup e recuperação

- [ ] Criar backup completo.
- [ ] Confirmar que o caminho criado é informado corretamente.
- [ ] Verificar existência física do backup.
- [ ] Inspecionar diagnóstico/validação de backup quando disponível.
- [ ] Preparar posteriormente uma cópia descartável para teste de restauração.
- [ ] Não restaurar sobre a base real durante esta fase inicial.

## Fase 2 — Correções guiadas pelos testes e pela auditoria

Para cada problema real encontrado:

1. registrar ou atualizar a entrada em **Achados**;
2. confirmar severidade e impacto;
3. localizar a causa;
4. criar ou ampliar teste de regressão quando viável;
5. corrigir com a menor mudança coerente possível;
6. rodar validações afetadas;
7. marcar o achado como corrigido;
8. validar manualmente quando o fluxo exigir interação real;
9. registrar commit/PR correspondente.

## Fase 3 — Hardening após os testes

- [ ] Ampliar cenários de migrations a partir dos casos reais encontrados.
- [ ] Ampliar testes de backup/restore com bases descartáveis.
- [ ] Revisar integridade histórica e invariantes reveladas pelo uso real.
- [ ] Revisar UX de operações repetitivas com base em atritos observados.
- [ ] Revisar acessibilidade por teclado nos fluxos realmente utilizados.
- [ ] Ampliar smoke/E2E desktop apenas para cenários que tragam cobertura útil e estável.

## Achados

### ACH-001 — Fato pode ser persistido antes de a reconciliação da Agenda falhar

- **Status:** corrigido e validado
- **Severidade:** alta
- **Área:** backend / Agenda / transações
- **Evidência:** `src-tauri/src/commands.rs`, `src-tauri/src/agenda_execution.rs`, `src-tauri/src/admin_commands.rs`, `src-tauri/src/agenda/derived.rs`.
- **Comportamento observado:** criação/correção/anulação de fatos podia concluir a gravação principal e somente depois chamar `agenda::reconcile_*`. Na execução especializada da Agenda, fato e conclusão da tarefa eram commitados antes da reconciliação que cria/ajusta o próximo compromisso.
- **Risco:** uma falha de reconciliação podia retornar erro ao usuário mesmo com parte da operação já persistida. Em criação direta, uma tentativa repetida poderia gerar outro fato. Na execução de tarefa, a tarefa poderia já estar concluída quando a interface recebesse falha.
- **Comportamento esperado:** a unidade de consistência prometida pelo fluxo deve ser atômica.
- **Contrato:** o comportamento agora atende `docs/AGENDA.md`: fato, próximo compromisso derivado e conclusão da tarefa permanecem na mesma unidade transacional nos fluxos corrigidos.
- **Correção:** criação direta, execução especializada, correção e anulação passaram a usar helpers `*_tx` e reconciliação derivada dentro da transação chamadora; a transação só é confirmada depois da Agenda ficar coerente.
- **Teste de regressão:** adicionado cenário que força falha na inserção da próxima tarefa derivada e confirma rollback da conclusão da tarefa e do fato.
- **Validação:** CI completo #507 (`33506010767`) no commit `f8a954c`: frontend, `cargo fmt`, `cargo check --locked`, Clippy com `-D warnings` e testes Rust aprovados.
- **Commit/PR:** implementado na `work/field-testing-and-hardening`; checkpoint de validação PR #47, destinado somente a CI e sem integração na `main`.

### ACH-002 — Mudanças de estado podem deixar Agenda derivada obsoleta até nova reconciliação

- **Status:** corrigido e validado
- **Severidade:** alta
- **Área:** backend / Agenda / ciclo de vida / movimentações / cadastros
- **Evidência:** `src-tauri/src/agenda/derived.rs`, `src-tauri/src/lifecycle.rs`, `src-tauri/src/movements/creation.rs`, `src-tauri/src/repository/occupancy.rs`, `src-tauri/src/box_states.rs`, `src-tauri/src/master_data/meliponaries.rs` e `src-tauri/src/reversals/`.
- **Comportamento observado:** ciclo de vida, transferência interna/externa, troca de caixa, arquivamento, mudança de estado da caixa e reversões podiam mudar a validade ou o contexto de uma tarefa derivada sem atualizá-la imediatamente. Além disso, inspeção/alimentação derivadas carregavam a caixa histórica do último fato em vez da ocupação ativa usada pelo planejamento futuro.
- **Risco:** tarefa pendente e alerta podiam mostrar meliponário/caixa/contexto antigo ou continuar existindo quando já deveriam desaparecer, exigindo reinicialização para `reconcile_all()` curar parte dos casos.
- **Comportamento esperado:** uma operação que invalida ou muda o contexto de planejamento derivado deve deixar a Agenda coerente ao retornar sucesso.
- **Correção:** inspeção e alimentação derivadas passaram a projetar a ocupação ativa atual; lifecycle, transferências, troca de caixa, estado da caixa, arquivamento/reativação de meliponário e reversões executam a reconciliação necessária dentro da mesma transação da mudança de domínio. Planejamento manual/genérico não é removido pelo arquivamento. Transporte temporário permanece fora por não alterar o contexto persistido.
- **Teste de regressão:** adicionados cenários para troca de caixa, transferência interna/externa, lifecycle, aposentadoria de caixa, archive/reactivate, reversões e falha induzida na Agenda exigindo rollback da própria mudança de estado/contexto.
- **Validação:** CI completo #534 (`33509790464`) no commit `8173b93`: frontend, `cargo fmt`, `cargo check --locked`, Clippy com `-D warnings` e testes Rust aprovados.
- **Commit/PR:** implementado na `work/field-testing-and-hardening`; checkpoint de validação PR #48, destinado somente a CI e sem integração na `main`.

### ACH-003 — Frontend confunde falha de refresh com falha da mutação já concluída

- **Status:** corrigido e validado
- **Severidade:** alta
- **Área:** frontend / estado assíncrono / UX de erro
- **Evidência:** `src/hooks/useAppData.ts`, `src/lib/mutation-flow.ts`, `tests/mutation-flow.test.ts` e importação de espécies no mesmo hook. `src/pages/AgendaPage.tsx` foi revisada e não reproduz o defeito: seu `reload()` já captura a própria falha e a mutação não é reclassificada como falha por esse motivo.
- **Comportamento observado:** o helper global `runMutation` envolvia `await action()` e `await refresh()` no mesmo `try/catch`. Se o backend gravasse com sucesso e somente a recarga falhasse, a função retornava `false` e a interface podia manter dialog/estado de erro como se nada tivesse sido salvo. A importação CSV repetia o mesmo problema ao retornar `null` depois de uma importação já persistida.
- **Risco:** repetição manual da operação, fatos duplicados e mensagens enganosas.
- **Comportamento esperado:** sucesso da escrita deve ser distinguido de falha ao sincronizar a visão. Falha de refresh deve solicitar nova tentativa de carregar os dados, não repetir a gravação.
- **Correção:** criado `runMutationFlow`, helper puro que distingue `success`, `mutation-failed` e `refresh-failed`. O hook global retorna sucesso quando a escrita concluiu, mesmo que a recarga posterior falhe, e mostra mensagem explícita informando que os dados foram salvos e que a tela deve ser atualizada antes de repetir. A importação de espécies preserva o resultado importado e aplica a mesma mensagem de sincronização.
- **Teste de regressão:** `tests/mutation-flow.test.ts` executa Promises reais e prova três contratos: sucesso completo, falha da mutação sem tentar refresh e mutação bem-sucedida seguida por refresh rejeitado classificada como `refresh-failed`.
- **Validação:** CI leve #540 (`33511753482`) aprovou build e testes frontend no commit `f407ce2`. Em seguida, a criação da branch `validation/field-testing-ach-003` disparou o perfil completo no commit congelado `0570b7c`; o CI #542 (`33512022812`) terminou com sucesso em frontend, `cargo fmt`, bundle checks, `cargo check --locked`, Clippy com `-D warnings` e testes Rust.
- **Commit/PR:** implementado na `work/field-testing-and-hardening`; checkpoint congelado em `validation/field-testing-ach-003` no SHA `0570b7c`. O usuário pode abrir manualmente um PR contra `main` para revisão/histórico, sempre sem merge.

### ACH-004 — Transferência externa comum pode não ser reversível por falta de lifecycle anterior

- **Status:** corrigido e validado
- **Severidade:** alta
- **Área:** backend / reversões / movimentações
- **Evidência:** `src-tauri/src/reversals/movements.rs`, `src-tauri/src/reversals/tests.rs`, `src-tauri/src/operational.rs` e `migrations/0002_core_domain.sql`.
- **Comportamento observado:** uma colônia recém-criada já nasce `active`, mas esse estado inicial não gera obrigatoriamente uma linha em `colony_lifecycle_records`. Nesse caso, uma transferência externa válida era posteriormente bloqueada na reversão porque o código exigia um lifecycle anterior para descobrir o estado a restaurar.
- **Risco:** um erro operacional comum de transferência externa podia não ser corrigível pelo mecanismo de reversão existente.
- **Comportamento esperado:** quando o estado anterior é inequivocamente derivável do histórico válido, a reversão segura deve conseguir restaurá-lo; quando não for derivável, deve continuar bloqueando.
- **Correção:** a reversão continua preferindo o último `new_status` de lifecycle anterior. Quando não existe lifecycle anterior, ela confirma que a transferência não antecede a entrada da colônia no plantel e restaura `active`, que é o estado inicial definido pelo schema e a mesma inferência histórica usada por `operational::ensure_colony_available_at`. Nenhuma linha retroativa de lifecycle é criada. As demais barreiras de segurança da reversão permanecem intactas: somente a transferência efetiva mais recente, sem fatos posteriores, com status atual `transferred` e caixa anterior ativa/livre pode ser restaurada automaticamente.
- **Teste de regressão:** adicionado cenário com colônia criada `active`, sem qualquer lifecycle, inspeção/Agenda pendente, transferência externa e reversão imediata. O teste exige restauração de `active`, meliponário, caixa e tarefa derivada, confirma `reversed_at` na movimentação e prova que a quantidade de registros de lifecycle permanece zero.
- **Validação:** CI completo #548 (`33517247992`) no commit `01c8473`: frontend, `cargo fmt`, bundle checks, `cargo check --locked`, Clippy com `-D warnings` e testes Rust aprovados. O run anterior #546 parou apenas no `rustfmt`; a única diferença mecânica pedida pelo formatter foi aplicada antes do run verde.
- **Commit/PR:** implementação funcional em `b3a614e`, regressão em `fbc5b00` e formatação mecânica em `01c8473`; checkpoint congelado em `validation/field-testing-ach-004` no SHA `01c8473`. Um PR manual contra `main` pode ser aberto apenas para revisão/histórico e deve ser fechado sem merge.

### ACH-005 — Cadastro, edição e importação discordam sobre identidade/duplicidade

- **Status:** corrigido e validado
- **Severidade:** média
- **Área:** backend / dados mestres / SQLite / importação
- **Evidência:** `src-tauri/src/identity.rs`, `src-tauri/src/repository/entities.rs`, `src-tauri/src/master_data/{boxes,colonies,meliponaries,species}.rs`, `src-tauri/src/master_data/identity_tests.rs`, `src-tauri/src/species_import.rs` e `migrations/0002_core_domain.sql`.
- **Comportamento observado:** criação de caixa/colônia dependia do `UNIQUE` padrão do SQLite, enquanto edição comparava códigos com `lower(trim(...))`. Meliponário/espécie também possuíam validação de edição mais estrita que criação. A importação de espécies usava outra chave própria.
- **Risco:** catálogo com duplicidades semanticamente equivalentes e comportamento diferente conforme a porta de entrada usada.
- **Comportamento esperado:** cada entidade deve possuir uma definição explícita e única de identidade operacional, aplicada de forma coerente em criar, editar e importar, sem tornar bancos antigos com colisões inutilizáveis.
- **Correção:** criado módulo interno de identidade compartilhada. Meliponário usa nome normalizado globalmente; caixa e colônia usam código normalizado dentro do meliponário; espécie usa nome científico normalizado quando presente e, na ausência dele, nome popular + gênero. A normalização é `trim` + lowercase Unicode, sem remoção de acentos ou alteração do conteúdo interno. Criação, edição e importação CSV passaram a consumir a mesma regra. Edição só executa a barreira de duplicidade quando a identidade normalizada muda, permitindo manutenção não identitária de registros em bancos antigos que já possuam colisões.
- **Decisão de schema:** nenhuma migration nova foi adicionada. Uma constraint normalizada imediata poderia falhar em bancos existentes que já contenham colisões e o `lower()` do SQLite não equivale à normalização Unicode do Rust. As constraints atuais permanecem como segunda defesa para colisões exatas; eventual defesa normalizada no banco exige primeiro estratégia explícita de diagnóstico/resolução de legado.
- **Teste de regressão:** criação rejeita variações por espaços/caixa de meliponário, caixa e colônia e preserva o escopo de caixa/colônia por meliponário; espécie cobre identidade científica e fallback nome popular + gênero; importação CSV usa a mesma chave; edições rejeitam novas colisões; e uma colisão legada inserida diretamente no SQLite continua permitindo edição que não altere a identidade.
- **Validação:** CI completo #565 (`33519618773`) no commit `db99ec6`: frontend, `cargo fmt`, bundle checks, `cargo check --locked`, Clippy com `-D warnings` e testes Rust aprovados. O run anterior #561 parou apenas no `rustfmt`; as três diferenças mecânicas apontadas foram aplicadas antes do run verde.
- **Commit/PR:** implementado na `work/field-testing-and-hardening`; checkpoint congelado em `validation/field-testing-ach-005` no SHA `db99ec6`. Um PR manual contra `main` pode ser aberto apenas para revisão/histórico e deve ser fechado sem merge.

### ACH-006 — Alguns loaders de página permitem resposta fora de ordem ou rejeição não tratada

- **Status:** aberto
- **Severidade:** média
- **Área:** frontend / concorrência assíncrona
- **Evidência:** `src/pages/MovementsPage.tsx` e `src/pages/AssetsPage.tsx`; `AgendaPage.tsx` já possui padrão melhor de sequência/cancelamento que pode servir de referência.
- **Comportamento observado:** certas recargas disparadas por `useEffect` não possuem token de sequência/cancelamento e algumas são lançadas com `void` sem captura local de erro.
- **Risco:** ao trocar rapidamente a seleção, resposta antiga pode sobrescrever dados da nova seleção; rejeições podem não produzir estado de erro útil.
- **Comportamento esperado:** páginas contextuais devem ignorar respostas obsoletas e apresentar falha de carregamento de forma controlada.
- **Correção:** padronizar o padrão simples já existente em páginas mais robustas.
- **Teste de regressão:** priorizar somente se o comportamento puder ser reproduzido de forma determinística.
- **Commit/PR:** pendente.

### ACH-007 — Alerta de colônia fraca não considera arquivamento do meliponário

- **Status:** investigando
- **Severidade:** média
- **Área:** backend / alertas / arquivamento
- **Evidência:** `src-tauri/src/alerts.rs` filtra condição e status da colônia em `weak_alerts`, mas não junta/filtra `meliponaries.archived_at`. A própria Agenda derivada exclui meliponários arquivados.
- **Comportamento observado:** em consulta global, uma colônia fraca pertencente a meliponário arquivado aparenta continuar gerando alerta operacional mesmo após reconciliação/reinício.
- **Ponto a confirmar:** o domínio afirma que arquivamento administrativo não equivale a ciclo de vida da colônia. Precisamos decidir se o modo global deve deliberadamente continuar alertando sobre unidades arquivadas ou se arquivamento significa retirá-las da operação diária.
- **Correção:** nenhuma até confirmar a semântica durante teste manual/uso esperado.
- **Teste de regressão:** se confirmado como bug, adicionar caso de meliponário arquivado com última inspeção `weak`.
- **Commit/PR:** pendente.

### ACH-008 — Semântica temporal da ocupação restaurada por reversão precisa ser confirmada

- **Status:** investigando
- **Severidade:** média
- **Área:** backend / histórico / reversões
- **Evidência:** helpers de reversão restauram a caixa criando nova ocupação com início no horário da reversão, não no horário do fato original revertido.
- **Comportamento observado:** reverter hoje uma transferência antiga encerra a ocupação de destino hoje e reabre a origem hoje. O intervalo entre o fato original e a reversão continua representado como período real no destino.
- **Ponto a confirmar:** isso é correto se “reversão” significa desfazer a consequência a partir de agora; é incorreto se significa corrigir um fato histórico lançado por engano como se nunca tivesse produzido aquela ocupação válida.
- **Correção:** nenhuma até fixar a semântica de produto. Não reescrever história por suposição.
- **Teste de regressão:** após decisão, testar intervalos `started_at/ended_at`, não apenas a caixa atual.
- **Commit/PR:** pendente.

### ACH-009 — Timeline aplica decoração administrativa em padrão N+1

- **Status:** aberto
- **Severidade:** baixa
- **Área:** backend / desempenho
- **Evidência:** `src-tauri/src/timeline.rs` carrega as entradas e consulta estado administrativo adicional por item durante a decoração.
- **Comportamento observado:** quantidade de queries cresce com o tamanho da timeline.
- **Risco:** bases com histórico grande podem apresentar lentidão progressiva.
- **Comportamento esperado:** projeção deve conseguir carregar metadados administrativos em lote ou na query principal quando desempenho real justificar.
- **Correção:** adiar até medição/manual test indicar impacto. Não otimizar por esporte.
- **Teste de regressão:** se corrigido, preservar estados e ordenação; benchmark opcional somente se necessário.
- **Commit/PR:** pendente.

### ACH-010 — Hotspots de legibilidade ainda têm estilo excessivamente comprimido

- **Status:** aberto
- **Severidade:** baixa
- **Área:** manutenção / qualidade de código
- **Evidência:** `src-tauri/src/production.rs`, `src-tauri/src/dashboard.rs`, `src-tauri/src/reversals.rs` e submódulos possuem trechos com nomes de uma letra, SQL/queries extensas em linha única e cadeias de chamadas muito comprimidas.
- **Comportamento observado:** código passa nas ferramentas, mas exige esforço desnecessário de revisão e aumenta risco de erro em manutenção futura.
- **Comportamento esperado:** código tocado por correções funcionais deve ficar claro o suficiente para revisão humana sem depender de decodificação mental.
- **Correção:** oportunística. Melhorar somente os trechos modificados por trabalho real; não abrir refatoração cosmética de dezenas de arquivos.
- **Teste de regressão:** os testes funcionais existentes devem permanecer iguais.
- **Commit/PR:** pendente.

### ACH-011 — Cobertura frontend é majoritariamente estrutural, não comportamental

- **Status:** aberto
- **Severidade:** baixa
- **Área:** testes / frontend
- **Evidência:** `npm run test:ui` usa o runner do Node e vários testes leem arquivos-fonte para validar padrões por regex. O smoke WebDriver cobre inicialização e uma navegação curta.
- **Comportamento observado:** contratos de estrutura são bem protegidos, porém há pouca execução real de dialogs, formulários, estados de erro e sequências de mutação.
- **Risco:** regressões de comportamento podem passar mesmo quando os testes textuais permanecem verdes.
- **Comportamento esperado:** bugs reais corrigidos nesta fase devem ganhar o menor teste comportamental estável capaz de impedir sua volta.
- **Correção:** não instalar stack de testes nova apenas por cobertura. Expandir WebDriver ou introduzir teste focal somente quando houver ganho concreto.
- **Teste de regressão:** nasce junto com os achados de alta/média severidade quando viável.
- **Commit/PR:** pendente.

## Pontos fortes confirmados na auditoria

- arquitetura backend/frontend realmente decomposta após o PR #46;
- TypeScript em modo `strict`;
- nenhuma ocorrência inicial de `TODO`, `unsafe` ou `as any` encontrada nas buscas de auditoria;
- fronteira IPC normaliza timestamps e aplica disponibilidade histórica nos manejos públicos relevantes;
- banco habilita `foreign_keys`;
- migrations `0001..0017` contínuas e preservadas;
- testes de upgrade/backfill cobrem migrations recentes importantes;
- constraints/triggers protegem ocupação/estado de caixa e transporte;
- Agenda possui índice único para fonte derivada pendente;
- backup/restore valida integridade, schema, manifesto e hashes e possui estratégia de staging/rollback;
- arquivos gerenciados rejeitam caminhos fora da área permitida e preservam metadados quando o arquivo físico desaparece;
- capabilities Tauri são restritas e a aplicação possui CSP;
- erros de banco são sanitizados na fronteira pública comum;
- workflows usam Actions fixadas por SHA, lockfiles e caches;
- auditoria de dependências npm/RustSec está separada e versionada;
- build/release mantém validação própria e não foi enfraquecido pelo perfil leve da branch.

## Decisões de trabalho

### 2026-08-31 — Início da fase prática

A partir desta branch, o desenvolvimento passa a ser guiado prioritariamente por testes manuais e uso real da aplicação. Automação continua sendo uma barreira de segurança e regressão, mas não substitui validação humana dos fluxos.

O trabalho será mantido neste arquivo para permitir continuidade entre conversas e reduzir dependência do hardware local do usuário.

### 2026-08-31 — Auditoria antes do teste funcional sistemático

Antes da rodada manual completa, será feita uma auditoria ampla do repositório. A finalidade não é reescrever código que já funciona, mas detectar falhas ocultas, atalhos frágeis, inconsistências de arquitetura e pontos de manutenção que tenham escapado durante o desenvolvimento acelerado por IA.

### 2026-08-31 — CI leve durante a fase longa de trabalho

Pushes para a branch de trabalho usam feedback frontend/documental rápido. As validações Rust caras permanecem no mesmo workflow e são reativadas automaticamente em Pull Requests para `main`. A `main` continua com sua validação desktop completa.

### 2026-08-31 — Auditoria inicial encerrada sem reescrita ampla

A revisão estática encontrou problemas reais, mas não justificou reescrita, troca de stack ou nova refatoração geral. O trabalho deve atacar primeiro consistência transacional e de estado. Legibilidade, desempenho e expansão de testes serão tratados somente onde o uso real ou uma correção concreta justificar.

### 2026-09-01 — ACH-001 fechado com validação completa

A fronteira transacional entre fatos e Agenda foi corrigida nos fluxos cobertos pelo ACH-001. O checkpoint temporário PR #47 executou o perfil completo de CI e o run #507 (`33506010767`) terminou com sucesso, incluindo Clippy com `-D warnings` e testes Rust. O PR de validação não deve ser integrado na `main`.

### 2026-09-01 — ACH-002 fechado com validação completa

Mudanças de contexto operacional agora reconciliam a Agenda derivada antes do commit. O checkpoint temporário PR #48 executou o perfil completo de CI e o run #534 (`33509790464`) terminou com sucesso, incluindo `cargo fmt`, `cargo check --locked`, Clippy com `-D warnings` e testes Rust. O PR de validação não deve ser integrado na `main`.

### 2026-09-01 — ACH-003 fechado com validação completa

O fluxo global de mutações agora distingue falha da escrita de falha posterior de refresh. Uma gravação concluída não retorna mais `false` só porque a visão não conseguiu recarregar; a interface informa que os dados foram salvos e orienta usar **Atualizar** antes de repetir a operação. A importação CSV recebeu a mesma proteção. A Agenda foi reavaliada e ficou fora do patch porque seu `reload()` já captura a própria falha sem reclassificar a mutação como malsucedida. O CI leve #540 (`33511753482`) aprovou build e testes frontend. A branch congelada `validation/field-testing-ach-003` disparou também o CI completo #542 (`33512022812`) no SHA `0570b7c`, aprovado em todos os gates. Um PR manual pode ser aberto apenas como checkpoint/revisão e deve ser fechado sem merge.

### 2026-09-01 — ACH-004 fechado com validação completa

A reversão de transferência externa agora consegue restaurar uma colônia originalmente criada como `active` mesmo quando não existe lifecycle anterior, desde que a própria cronologia da entrada no plantel torne essa inferência segura. O mecanismo não cria lifecycle retroativo e preserva todas as barreiras existentes contra reversão insegura. O CI completo #548 (`33517247992`) passou no checkpoint congelado `validation/field-testing-ach-004` / `01c8473`, incluindo frontend, `cargo fmt`, `cargo check --locked`, Clippy com `-D warnings` e testes Rust. Um PR manual pode ser aberto apenas como checkpoint/revisão e deve ser fechado sem merge.

### 2026-09-01 — ACH-005 fechado com validação completa

As quatro famílias de dados mestres agora possuem identidade operacional explícita e compartilhada entre criação, edição e, no caso de espécie, importação CSV. A regra usa `trim` + lowercase Unicode; meliponário é identificado pelo nome global, caixa e colônia pelo código dentro do meliponário e espécie pelo nome científico quando presente ou por nome popular + gênero no fallback. Colisões antigas não são reescritas nem impedem manutenção que preserve a identidade existente. Nenhuma migration nova foi criada porque uma constraint normalizada agora poderia quebrar bancos com colisões preexistentes e não reproduziria com segurança a normalização Unicode via SQLite. O CI completo #565 (`33519618773`) passou no checkpoint congelado `validation/field-testing-ach-005` / `db99ec6`, incluindo frontend, `cargo fmt`, `cargo check --locked`, Clippy com `-D warnings` e testes Rust. Um PR manual pode ser aberto apenas como checkpoint/revisão e deve ser fechado sem merge.

## Próximo passo

O bloco inicial ACH-001..ACH-005 está corrigido e validado. O próximo passo previsto pelo plano é iniciar a rodada sistemática de **testes manuais orientados por uso real**, começando pelos cadastros e fluxos diretamente afetados por essas correções antes de escolher automaticamente o próximo achado médio/baixo.

Se for desejado manter o mesmo histórico visual dos checkpoints anteriores, `validation/field-testing-ach-005` está congelada no SHA `db99ec6` para um PR manual contra `main`, sempre sem merge.

O próximo achado aberto por ordem numérica é o **ACH-006**, mas ele não deve ser implementado automaticamente antes de usar os resultados da rodada manual para confirmar prioridade e reprodução determinística.

## Handoff para a próxima conversa

Se uma nova conversa precisar continuar este trabalho:

1. abrir `docs/WORK-PLAN.md` na branch `work/field-testing-and-hardening`;
2. ler **Estado atual**, **Ordem inicial de correção**, **Achados**, **Decisões de trabalho** e **Próximo passo**;
3. conferir os commits mais recentes da branch;
4. continuar pelo próximo passo registrado, sem refazer a auditoria do zero;
5. atualizar este arquivo ao fechar cada achado ou ao tomar decisão de domínio importante.
