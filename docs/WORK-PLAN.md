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
- [ ] Auditoria geral do projeto em andamento.
- [ ] Iniciar rodada sistemática de testes manuais após a auditoria inicial.

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

Essas validações continuam presentes no workflow e voltam automaticamente quando o trabalho for submetido por Pull Request para `main`. A validação completa da própria `main` permanece separada e inclui build desktop e smoke WebDriver.

Antes da integração final desta branch, revisar se a exceção específica de `work/field-testing-and-hardening` deve ser removida do workflow.

## Fase 0 — Auditoria geral do projeto

Objetivo: descobrir o estado real do repositório antes do teste funcional sistemático. Não corrigir tudo por reflexo; primeiro classificar o que é defeito, risco, dívida aceitável ou apenas preferência estética.

### Estrutura e arquitetura

- [ ] Mapear estrutura atual do frontend, backend, migrations, testes, scripts, assets e workflows.
- [ ] Conferir se as responsabilidades descritas em `docs/ARCHITECTURE.md` correspondem ao código real.
- [ ] Procurar módulos novamente grandes ou com responsabilidades misturadas após o PR #46.
- [ ] Procurar dependências circulares, fachadas vazias e abstrações sem ganho real.

### Backend Rust

- [ ] Revisar tratamento de erros e fronteira IPC.
- [ ] Procurar `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!` e caminhos que possam derrubar a aplicação.
- [ ] Revisar transações e operações multi-etapa que precisam ser atômicas.
- [ ] Revisar SQL dinâmico, validação de entrada e invariantes de domínio.
- [ ] Procurar duplicação importante de regras e helpers quase idênticos.
- [ ] Revisar manipulação de arquivos, backup/restore e caminhos externos.
- [ ] Verificar código morto, funções públicas sem necessidade e comentários que escondem dívida.

### Frontend React/TypeScript

- [ ] Revisar páginas, hooks, componentes e clientes IPC.
- [ ] Procurar estado duplicado, efeitos frágeis, race conditions e chamadas IPC sem tratamento coerente.
- [ ] Procurar casts perigosos, `any`, non-null assertions e suposições não validadas.
- [ ] Revisar dialogs e formulários para validação duplicada ou divergente do backend.
- [ ] Procurar componentes com responsabilidades excessivas ou abstrações puramente cosméticas.
- [ ] Revisar mensagens de erro e estados vazios/carregando.

### Banco e migrations

- [ ] Conferir ordem e imutabilidade das migrations existentes.
- [ ] Revisar constraints, índices, triggers e relacionamentos críticos.
- [ ] Procurar regras importantes mantidas apenas na aplicação quando deveriam ter segunda defesa no SQLite.
- [ ] Conferir testes de migrations e compatibilidade histórica existentes.

### Testes

- [ ] Mapear o que é realmente testado e o que é apenas teste estrutural/textual.
- [ ] Procurar testes frágeis que passam sem exercitar comportamento real.
- [ ] Identificar fluxos críticos sem regressão automatizada razoável.
- [ ] Não aumentar cobertura por porcentagem; priorizar risco de produto.

### CI, supply chain e release

- [ ] Revisar gatilhos e responsabilidades de cada workflow.
- [ ] Confirmar que Actions continuam fixadas por SHA e permissões estão mínimas.
- [ ] Revisar caches, lockfiles, auditorias e separação entre CI rápido e validação completa.
- [ ] Procurar passos redundantes ou validações caras sem benefício proporcional.

### Segurança e dados locais

- [ ] Procurar segredos, caminhos pessoais e resíduos de debug versionados.
- [ ] Revisar capabilities Tauri e acesso ao filesystem/dialogs.
- [ ] Revisar exposição de erros internos para a UI.
- [ ] Revisar operações destrutivas, restauração, substituição e exclusão de arquivos.

### Documentação e coerência

- [ ] Conferir README, Wiki e `docs/` contra comportamento atual.
- [ ] Procurar documentação histórica apresentada como comportamento atual.
- [ ] Conferir scripts/comandos documentados contra `package.json` e workflows.

### Saída da auditoria

- [ ] Registrar achados concretos nesta página com severidade e evidência.
- [ ] Separar correções necessárias de sugestões opcionais.
- [ ] Definir ordem de correção antes de iniciar mudanças amplas.

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

## Fase 2 — Correções guiadas pelos testes

Para cada problema real encontrado:

1. registrar na seção **Achados**;
2. definir severidade e impacto;
3. localizar a causa;
4. criar ou ampliar teste de regressão quando viável;
5. corrigir com a menor mudança coerente possível;
6. rodar validações afetadas;
7. marcar o achado como resolvido e registrar commit/PR correspondente.

## Fase 3 — Hardening após os testes

- [ ] Ampliar cenários de migrations a partir dos casos reais encontrados.
- [ ] Ampliar testes de backup/restore com bases descartáveis.
- [ ] Revisar integridade histórica e invariantes reveladas pelo uso real.
- [ ] Revisar UX de operações repetitivas com base em atritos observados.
- [ ] Revisar acessibilidade por teclado nos fluxos realmente utilizados.
- [ ] Ampliar smoke/E2E desktop apenas para cenários que tragam cobertura útil e estável.

## Achados

Nenhum achado registrado ainda.

Formato sugerido para novos achados:

### ACH-XXX — Título curto

- **Status:** aberto | investigando | corrigido | validado
- **Severidade:** crítica | alta | média | baixa
- **Área:**
- **Como reproduzir:**
- **Comportamento observado:**
- **Comportamento esperado:**
- **Causa:**
- **Correção:**
- **Teste de regressão:**
- **Commit/PR:**

## Decisões de trabalho

### 2026-08-31 — Início da fase prática

A partir desta branch, o desenvolvimento passa a ser guiado prioritariamente por testes manuais e uso real da aplicação. Automação continua sendo uma barreira de segurança e regressão, mas não substitui validação humana dos fluxos.

O trabalho será mantido neste arquivo para permitir continuidade entre conversas e reduzir dependência do hardware local do usuário.

### 2026-08-31 — Auditoria antes do teste funcional sistemático

Antes da rodada manual completa, será feita uma auditoria ampla do repositório. A finalidade não é reescrever código que já funciona, mas detectar falhas ocultas, atalhos frágeis, inconsistências de arquitetura e pontos de manutenção que tenham escapado durante o desenvolvimento acelerado por IA.

### 2026-08-31 — CI leve durante a fase longa de trabalho

Pushes para a branch de trabalho usam feedback frontend/documental rápido. As validações Rust caras permanecem no mesmo workflow e são reativadas automaticamente em Pull Requests para `main`. A `main` continua com sua validação desktop completa.

## Próximo passo

Concluir a **Fase 0 — Auditoria geral do projeto**, registrar os achados concretos e definir sua prioridade antes da primeira rodada sistemática de testes manuais.

## Handoff para a próxima conversa

Se uma nova conversa precisar continuar este trabalho:

1. abrir `docs/WORK-PLAN.md` na branch `work/field-testing-and-hardening`;
2. ler **Estado atual**, **Achados**, **Decisões de trabalho** e **Próximo passo**;
3. conferir os commits mais recentes da branch;
4. continuar do primeiro item não concluído;
5. atualizar este arquivo antes de encerrar uma etapa importante.
