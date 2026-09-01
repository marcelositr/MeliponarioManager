# Plano de trabalho contínuo

Este arquivo é o ponto de continuidade do trabalho prático no MeliponarioManager. Ele deve permitir retomar o projeto sem depender do histórico do chat e registrar decisões, validações, riscos resolvidos e trabalho ainda pendente.

## Branch de trabalho

`work/field-testing-ach-006`

Base histórica inicial: `main` após o merge do PR #46 (`7640c69`).

Base atual deste bloco: `main` após o squash merge do PR #49 (`4761b14644bcbccd56a7a7e00f65f1c197178624`).

A partir do ACH-006, os achados foram agrupados em rodadas compactas na mesma branch para reduzir ciclos administrativos sem reduzir revisão, regressões ou gates técnicos.

## Regras de trabalho

- Não alterar migrations existentes.
- Mudanças de schema novas exigem migration nova e justificativa explícita.
- Não realizar restauração destrutiva contra a base real do usuário durante testes.
- Antes de corrigir bug reproduzível, registrar o comportamento e, quando viável, adicionar regressão.
- Preservar contratos IPC e formatos persistidos salvo decisão explícita registrada aqui.
- Não refatorar apenas por estética ou contagem de linhas.
- Não introduzir frameworks, abstrações ou dependências sem problema concreto.
- Código tocado por correção real deve ficar legível para revisão humana.
- Atualizar este plano ao concluir blocos relevantes.
- Se o CI falhar e o log não puder ser obtido diretamente, parar e solicitar o trecho do erro em vez de tentar rotas repetidas de extração.

## Estado atual

- [x] PR #46 integrado na `main`.
- [x] Auditoria estática geral inicial concluída.
- [x] ACH-001 corrigido e validado.
- [x] ACH-002 corrigido e validado.
- [x] ACH-003 corrigido e validado.
- [x] ACH-004 corrigido e validado.
- [x] ACH-005 corrigido e validado.
- [x] Fluxos ACH-001..ACH-005 validados manualmente sem regressões observadas.
- [x] Fotos, criação/validação de backup e impressão testadas manualmente e aprovadas.
- [x] Exceção temporária de CI leve removida antes da integração do primeiro bloco.
- [x] PR #49 integrado na `main` por squash merge no commit `4761b14`.
- [x] `main` pós-PR #49 validada pelo full-check #29 (`33523844254`) e auditoria de dependências #21 (`33523844158`).
- [x] Rodada 1 compacta, ACH-006..ACH-008, concluída e validada pelo CI #586 (`33527575099`) no SHA `316e7aa`.
- [x] Rodada 2 compacta, ACH-009..ACH-011, concluída e validada pelo CI #589 (`33529103020`) no SHA `6c0404b`.
- [ ] Rodada 3 de hardening e validações manuais restantes.

## Política de CI

Durante a fase longa anterior ao PR #49, pushes em `work/field-testing-and-hardening` usaram perfil leve para feedback rápido, mantendo os gates Rust completos nas branches de validação e PRs. Essa exceção foi removida antes da integração.

Desde o commit `efb6c4f`, mudanças de código na branch de trabalho executam novamente o perfil completo:

- build frontend;
- testes frontend;
- `cargo fmt --check`;
- geração e validação dos ícones de bundle;
- `cargo check --locked`;
- Clippy com warnings tratados como erro;
- `cargo test --locked`.

A documentação preserva essa informação para explicar por que runs antigos da branch longa aparecem com perfil reduzido.

## Resultado da auditoria inicial

A auditoria não encontrou motivo para reescrever o projeto, trocar stack ou criar outra decomposição geral. A arquitetura backend/frontend está efetivamente separada, TypeScript usa `strict`, migrations `0001..0017` permanecem contínuas, o SQLite possui defesas importantes de integridade, backup/restore e arquivos gerenciados têm proteções relevantes, capabilities Tauri são restritas, existe CSP e o CI/release usa lockfiles, caches e Actions fixadas.

O padrão dos defeitos encontrados foi de regras que evoluíram em momentos diferentes e deixaram inconsistências entre persistência, estado operacional, Agenda, interface e projeções. Os achados ACH-001..ACH-011 foram tratados sem ampliação artificial de arquitetura.

## Ordem executada

1. **ACH-001** — atomicidade entre fatos e reconciliação da Agenda. Concluído em 2026-09-01.
2. **ACH-002** — reconciliação da Agenda após mudanças de estado/contexto. Concluído em 2026-09-01.
3. **ACH-003** — separar sucesso da mutação de falha posterior de refresh. Concluído em 2026-09-01.
4. **ACH-004** — reversão segura de transferência externa sem lifecycle anterior quando `active` é historicamente derivável. Concluído em 2026-09-01.
5. **ACH-005** — identidade/duplicidade coerente entre criação, edição e importação. Concluído em 2026-09-01.
6. Validação manual do primeiro bloco e integração pelo PR #49. Concluído em 2026-09-01.
7. **Rodada 1: ACH-006..ACH-008.** Concluída em 2026-09-01, CI #586 verde.
8. **Rodada 2: ACH-009..ACH-011.** Concluída em 2026-09-01, CI #589 verde.
9. **Rodada 3:** hardening e validações manuais restantes. Próximo bloco.

## Fase 1 — Validação manual orientada por uso real

### Confirmado manualmente

- [x] Fluxos diretamente afetados por ACH-001..ACH-005.
- [x] Fluxo de fotos.
- [x] Criação de backup completo.
- [x] Caminho físico e diagnóstico/validação do backup.
- [x] Impressão.

### Ainda não considerar concluído por inferência

#### Inicialização e navegação

- [ ] Iniciar a aplicação com a base real existente em uma rodada final controlada.
- [ ] Confirmar carregamento normal da Visão geral.
- [ ] Percorrer todas as páginas principais.
- [ ] Registrar erros visuais, falhas de carregamento, travamentos ou mensagens inadequadas.
- [ ] Verificar resolução compatível com o ambiente real de uso.

#### Cadastros e manejo

- [ ] Revisar meliponários, espécies, caixas e colônias em conjunto.
- [ ] Verificar buscas, filtros, estados vazios e navegação contextual.
- [ ] Revisar inspeções, alimentação, produção, eventos e manutenção numa passada final.

#### Agenda, movimentações e transporte

- [ ] Revisar Agenda e alertas com dados existentes após ACH-007.
- [ ] Revisar histórico de movimentações.
- [ ] Revisar ciclo de transporte quando houver dados adequados.
- [ ] Verificar documentos e estados derivados.

#### Arquivos gerenciados

- [x] Fotos validadas manualmente.
- [ ] Abrir ou revelar pelo menos um arquivo gerenciado não-foto.
- [ ] Verificar estado de arquivo ausente/inconsistente com exemplo seguro.
- [ ] Confirmar que erros de filesystem chegam à UI de forma útil e sem vazamento técnico indevido.

#### Relatórios e CSV

- [ ] Abrir relatórios com dados reais.
- [ ] Verificar filtros e estados vazios.
- [ ] Exportar CSV pelo seletor nativo.
- [ ] Confirmar nome e destino.
- [ ] Abrir o CSV resultante e conferir estrutura básica.
- [x] Impressão validada manualmente.

#### Backup e recuperação

- [x] Criar backup completo.
- [x] Confirmar caminho e existência física.
- [x] Inspecionar diagnóstico/validação disponível.
- [ ] Testar restauração somente contra cópia/base descartável.
- [x] Não restaurar sobre a base real nesta fase.

## Achados

### ACH-001 — Fato podia persistir antes de a reconciliação da Agenda falhar

- **Status:** corrigido e validado.
- **Severidade:** alta.
- **Área:** backend / Agenda / transações.
- **Correção:** criação, execução especializada, correção e anulação de fatos usam helpers transacionais e reconciliação derivada antes do commit.
- **Regressão:** falha induzida na criação do próximo compromisso prova rollback do fato e da conclusão da tarefa.
- **Validação:** CI completo #507 (`33506010767`) no commit `f8a954c`.
- **Checkpoint:** PR #47, usado somente para validação e fechado sem merge.

### ACH-002 — Mudanças de estado podiam deixar Agenda derivada obsoleta

- **Status:** corrigido e validado.
- **Severidade:** alta.
- **Área:** backend / Agenda / lifecycle / movimentações / cadastros.
- **Correção:** Agenda derivada usa ocupação ativa atual e é reconciliada transacionalmente nos fluxos que alteram contexto ou disponibilidade.
- **Regressões:** troca de caixa, transferências, lifecycle, estado de caixa, archive/reactivate e reversões, incluindo rollback em falha da Agenda.
- **Validação:** CI completo #534 (`33509790464`) no commit `8173b93`.
- **Checkpoint:** PR #48, usado somente para validação e fechado sem merge.

### ACH-003 — Frontend confundia falha de refresh com falha da mutação concluída

- **Status:** corrigido e validado.
- **Severidade:** alta.
- **Área:** frontend / estado assíncrono / UX.
- **Correção:** `runMutationFlow` distingue `success`, `mutation-failed` e `refresh-failed`; importação de espécies segue a mesma semântica.
- **Regressão:** `tests/mutation-flow.test.ts` executa Promises reais para os três resultados.
- **Nota:** Agenda foi revisada e não reproduzia o bug porque seu `reload()` já captura a própria falha.
- **Validação:** CI completo #542 (`33512022812`) no SHA `0570b7c`.

### ACH-004 — Transferência externa podia não ser reversível sem lifecycle anterior

- **Status:** corrigido e validado.
- **Severidade:** alta.
- **Área:** backend / reversões / movimentações.
- **Correção:** se não houver lifecycle anterior, a reversão restaura `active` somente quando a cronologia comprova que esse estado inicial é derivável; nenhum lifecycle retroativo é fabricado.
- **Regressão:** transferência externa de colônia criada ativa sem lifecycle restaura status, meliponário, caixa e Agenda e mantém zero registros artificiais de lifecycle.
- **Validação:** CI completo #548 (`33517247992`) no SHA `01c8473`.

### ACH-005 — Cadastro, edição e importação divergiam sobre identidade

- **Status:** corrigido e validado.
- **Severidade:** média.
- **Área:** backend / dados mestres / importação.
- **Correção:** identidade compartilhada usa `trim` + lowercase Unicode: meliponário por nome global, caixa/colônia por código no meliponário e espécie por nome científico ou fallback nome popular + gênero.
- **Compatibilidade:** edição que não altera identidade continua possível em bases antigas com colisões preexistentes.
- **Schema:** nenhuma migration nova; uma constraint normalizada imediata poderia quebrar legado e o `lower()` do SQLite não reproduz a normalização Unicode do Rust.
- **Validação:** CI completo #565 (`33519618773`) no SHA `db99ec6`.

### ACH-006 — Loaders podiam aceitar resposta fora de ordem ou rejeição sem tratamento útil

- **Status:** corrigido e validado.
- **Severidade:** média.
- **Área:** frontend / concorrência assíncrona.
- **Evidência final:** `src/lib/latest-request.ts`, `tests/latest-request.test.ts`, `src/pages/MovementsPage.tsx`, `src/pages/AssetsPage.tsx` e `src/pages/assets/AssetsMaintenancePanel.tsx`.
- **Correção:** controlador simples de “última requisição vence” protege carregamentos de movimentações, retornos, documentos e manutenção; resposta obsoleta não aplica estado nem encerra loading da requisição atual; falhas atuais geram feedback controlado. Reabrir documentos do mesmo movimento dispara nova tentativa, mesmo após erro anterior.
- **Escopo:** o fluxo de fotos que já possuía proteção adequada não foi refeito por reflexo; apenas pontos realmente frágeis foram endurecidos.
- **Regressões:** Promises reais provam resolução fora de ordem, erro obsoleto versus erro atual, invalidação e ownership do `onSettled`.
- **Validação:** CI #586 (`33527575099`) no SHA `316e7aabc91837c53db6ea946140ef3c46b897c4`, com frontend, `cargo fmt`, bundle, `cargo check`, Clippy e testes Rust verdes.

### ACH-007 — Alertas operacionais não respeitavam arquivamento do meliponário

- **Status:** corrigido e validado.
- **Severidade:** média.
- **Área:** backend / alertas / arquivamento.
- **Decisão de domínio:** meliponário arquivado fica fora da operação diária, coerente com a Agenda, mas seu histórico não é apagado.
- **Correção:** alertas globais e filtrados excluem meliponários arquivados, incluindo colônia fraca e tarefa vencida.
- **Regressão:** `archived_meliponary_is_excluded_from_operational_alerts_without_losing_history` prova que inspeção e tarefa permanecem persistidas, que a tarefa continua `pending` e que os alertas retornam ao reativar o meliponário.
- **Validação:** CI #586 (`33527575099`) no SHA `316e7aa`.

### ACH-008 — Semântica temporal da ocupação restaurada por reversão

- **Status:** semântica confirmada e validada; não exigiu mudança de regra de produção.
- **Severidade:** média.
- **Área:** backend / histórico / reversões.
- **Decisão de domínio:** reversão desfaz a consequência a partir do momento da reversão; não reescreve o passado como se a movimentação/lifecycle nunca tivesse ocorrido.
- **Regressões:** testes temporais exigem que o intervalo original seja preservado, o destino permaneça registrado durante seu período real e a ocupação restaurada na origem comece em `reversed_at`.
- **Validação:** CI #586 (`33527575099`) no SHA `316e7aa`.

### ACH-009 — Timeline aplicava decoração administrativa em padrão N+1

- **Status:** corrigido e validado.
- **Severidade:** baixa.
- **Área:** backend / desempenho / manutenção.
- **Correção:** `timeline.rs` carrega estados administrativos da colônia em uma única query `UNION ALL`, organiza por `source_type/source_id` e aplica as decorações em memória. A quantidade de queries de decoração não cresce mais por item da timeline.
- **Semântica preservada:** anulação continua prevalecendo sobre correção; reversão de movimentação prevalece sobre anulação/correção; lifecycle revertido e ocupação corrigida mantêm títulos/severidade esperados.
- **Regressões:** testes puros cobrem precedência e preservação de detalhes; testes integrados existentes continuam exercitando `timeline::by_colony` contra SQLite real de teste.
- **Commits:** `bc62af0` (mudança funcional) e `6c0404b` (somente rustfmt).
- **Validação:** CI #589 (`33529103020`) no SHA `6c0404bca23e0d6547cbb4f2083410716dbc8bcc`, totalmente verde.

### ACH-010 — Hotspots de legibilidade excessivamente comprimidos

- **Status:** concluído para o escopo desta fase e validado.
- **Severidade:** baixa.
- **Área:** manutenção / qualidade de código.
- **Decisão:** não executar varredura cosmética em arquivos sem motivo funcional. O contrato do achado era melhorar código realmente tocado pelo trabalho.
- **Aplicação:** a refatoração do ACH-009 transformou o trecho tocado da timeline em SQL multilinha, estruturas nomeadas e helpers explícitos, removendo a decoração comprimida e a lógica N+1. Trechos antigos em módulos não tocados continuam como dívida oportunística e devem ser melhorados quando houver trabalho funcional neles.
- **Validação:** `cargo fmt`, `cargo check`, Clippy e testes Rust aprovados no CI #589.

### ACH-011 — Cobertura frontend era majoritariamente estrutural

- **Status:** concluído para esta fase e validado.
- **Severidade:** baixa.
- **Área:** testes / frontend.
- **Decisão:** não instalar framework novo nem perseguir percentual de cobertura. Bugs reais devem gerar o menor teste comportamental estável possível.
- **Cobertura adicionada:** `tests/mutation-flow.test.ts` e `tests/latest-request.test.ts` executam Promises reais. Os testes de latest-request cobrem resolução fora de ordem, rejeição obsoleta versus atual, invalidação e garantia de que requisição obsoleta não encerra o loading pertencente à requisição atual.
- **Validação:** 54 testes frontend passaram no CI #589, além de toda a cadeia Rust.

## Validação das rodadas compactas

### Rodada 1 — ACH-006..ACH-008

- Branch: `work/field-testing-ach-006`.
- Checkpoint técnico: `316e7aabc91837c53db6ea946140ef3c46b897c4`.
- CI #586 / run `33527575099`: **success**.
- Gates verdes: build e testes frontend, `cargo fmt`, bundle checks, `cargo check --locked`, Clippy e `cargo test --locked`.
- Não foi criada branch de validação separada, conforme a decisão de reduzir rodadas administrativas.

### Rodada 2 — ACH-009..ACH-011

- Mesma branch: `work/field-testing-ach-006`.
- Checkpoint técnico: `6c0404bca23e0d6547cbb4f2083410716dbc8bcc`.
- CI #589 / run `33529103020`: **success**.
- Gates verdes: build frontend, 54 testes frontend, `cargo fmt`, bundle checks, `cargo check --locked`, Clippy e `cargo test --locked`.
- O run #588 (`33528908316`) havia parado apenas no `rustfmt`; o log foi obtido diretamente e pediu uma única dobra mecânica da assinatura de `by_colony`, aplicada em `6c0404b` sem alteração semântica.

## Pontos fortes confirmados

- arquitetura backend/frontend decomposta de forma real;
- TypeScript em modo `strict`;
- migrations `0001..0017` preservadas;
- SQLite com `foreign_keys` e defesas importantes de ocupação/estado/transporte;
- Agenda com proteção de fonte derivada pendente e reconciliação transacional nos fluxos corrigidos;
- backup/restore com validação de integridade, schema, manifesto, hashes e estratégia de staging/rollback;
- arquivos gerenciados com proteção de caminho e metadados preservados quando arquivo físico desaparece;
- capabilities Tauri restritas e CSP presente;
- erros de banco sanitizados na fronteira pública comum;
- workflows com Actions fixadas por SHA, lockfiles e caches;
- auditoria npm/RustSec separada;
- build/release com validação própria;
- testes comportamentais focais agora cobrem falhas assíncronas reais do frontend.

## Decisões registradas

### 2026-08-31 — Início da fase prática

O desenvolvimento passou a ser guiado por revisão humana e uso real. Automação permanece como barreira de segurança, mas não substitui validação humana.

### 2026-08-31 — Auditoria antes do teste funcional sistemático

A auditoria ampla foi usada para identificar riscos concretos sem abrir reescrita geral, troca de stack ou refatoração estética ampla.

### 2026-09-01 — ACH-001..ACH-005 estabilizados

Os cinco primeiros achados foram corrigidos e validados em checkpoints completos. O bloco foi exercitado manualmente, junto com fotos, backup e impressão, e depois integrado pelo PR #49.

### 2026-09-01 — PR #49 integrado

O PR #49 (`fix: integrate field-testing hardening ACH-001 through ACH-005`) foi integrado na `main` por squash no commit `4761b14644bcbccd56a7a7e00f65f1c197178624`. O PR havia passado no CI #571 (`33521970903`) e na auditoria de dependências #20 (`33521970798`); a `main` pós-merge passou novamente no full-check #29 (`33523844254`) e auditoria #21 (`33523844158`).

### 2026-09-01 — Rodada 1 ACH-006..ACH-008 concluída

A branch única `work/field-testing-ach-006` corrigiu concorrência assíncrona dos loaders, definiu que meliponário arquivado fica fora dos alertas operacionais sem perder histórico e congelou por testes a semântica temporal não retroativa das reversões. O CI #586 (`33527575099`) passou integralmente no SHA `316e7aa`.

### 2026-09-01 — Rodada 2 ACH-009..ACH-011 concluída

A mesma branch removeu o N+1 de decoração da timeline, aplicou a política de legibilidade apenas no código realmente tocado e ampliou testes comportamentais de frontend sem introduzir nova stack. O CI #589 (`33529103020`) passou integralmente no SHA `6c0404b`.

## Rodada 3 — Próximo passo

Executar uma rodada compacta de hardening e validação final, ainda na mesma branch salvo decisão explícita de integração antes disso:

1. testar arquivo gerenciado não-foto e estados seguro de ausente/inconsistente;
2. revisar relatórios com dados reais e exportar/abrir CSV;
3. testar restauração somente contra cópia/base descartável;
4. revisar acessibilidade por teclado nos fluxos realmente usados;
5. ampliar migration/backup hardening apenas se casos concretos revelarem lacuna;
6. ampliar smoke/E2E somente para cenário concreto, estável e de valor real;
7. fazer varredura final deste plano e separar pendência real de melhoria opcional;
8. se houver código na Rodada 3, executar CI completo antes de preparar integração; se não houver código, encerrar documentalmente sem PR artificial.

## Handoff

Para retomar em outra conversa:

1. usar `work/field-testing-ach-006` como branch ativa enquanto este bloco não for integrado;
2. ler **Estado atual**, **Achados**, **Validação das rodadas compactas** e **Rodada 3**;
3. conferir o head mais recente e seu CI;
4. não refazer ACH-001..ACH-011 sem evidência nova;
5. manter `main` intocada até a próxima integração deliberada;
6. preservar e atualizar este arquivo ao concluir a Rodada 3 ou integrar o bloco.
