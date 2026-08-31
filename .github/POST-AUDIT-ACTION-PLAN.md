# Plano de ação pós-auditoria

> **Branch de trabalho:** `fix/post-audit-cleanup`  
> **Objetivo:** corrigir os achados da auditoria geral, acelerar o ciclo de CI sem enfraquecer a `main` e preparar uma integração única e limpa.  
> **Regra de acompanhamento:** um item só recebe `[x]` quando estiver realmente concluído e validado. Trabalho parcial permanece `[ ]`.

Este arquivo é o ponto de retomada caso a sessão seja interrompida. Ele é temporário e não representa documentação permanente do produto.

## Estado atual

**Passos 1 a 7 concluídos e validados. A revisão estática final também foi concluída. O pacote agora aguarda teste manual do usuário e integração final.**

Já aplicado:

- `dialog:allow-save` e teste de contrato das capabilities usadas pelo frontend;
- correção da Wiki sobre o destino automático dos backups;
- FAQ da Wiki esclarecendo onde o backup completo é armazenado;
- documentação da Wiki esclarecendo que a exportação CSV usa o seletor nativo de arquivos para escolher nome e destino;
- remoção dos módulos/testes Rust órfãos identificados na auditoria;
- recuperação dos quatro testes úteis de regressão de produção em módulo ativo;
- remoção do ícone gerado versionado e inclusão de `src-tauri/icons/` no `.gitignore`;
- separação de CI rápido para branch/PR, validação completa para `main` e manutenção do workflow de bundles/releases;
- documentação do novo modelo de CI em `CONTRIBUTING.md`, `docs/GITHUB-OPERATIONS.md` e `docs/DISTRIBUTION.md`;
- sanitização central de `AppError::Database` antes da fronteira IPC, com testes de regressão;
- CI rápido do pacote técnico concluído com sucesso, incluindo versão, dependências, ícones, frontend, testes de UI, `cargo fmt`, `cargo check`, Clippy e `cargo test`.

Estado de validação:

- branch baseada na `main` após o merge do PR #44;
- `main` continua no commit do merge do PR #44; nenhum commit acidental adicional foi encontrado nela;
- branch está à frente de `main` e 0 atrás;
- nenhuma migration foi modificada;
- diff atual está limitado ao plano, CI, documentação relacionada, capability, higiene de ícones, limpeza/testes Rust, hardening de erro e correções pontuais da Wiki;
- CI rápido verde validou o conjunto de alterações técnicas;
- commits posteriores ao CI verde são somente documentação da Wiki e atualização deste plano; por decisão do usuário, não é necessário aguardar novo CI para esses commits;
- revisão final do diff concluída;
- **não abrir/mergear o pacote ainda**;
- próximo marco: teste manual da aplicação pelo usuário e, se aprovado, integração final.

## 1. Correções funcionais e permissões Tauri

- [x] Adicionar `dialog:allow-save` à capability desktop usada pela janela principal.
- [x] Revisar todos os usos atuais dos plugins/capabilities do Tauri e confirmar que não há outra permissão necessária ausente.
- [x] Confirmar que a exportação CSV de Relatórios possui o fluxo de permissão correto.
- [x] Registrar teste ou validação que reduza a chance de uma capability obrigatória voltar a faltar silenciosamente.

**Critério de conclusão:** permissões necessárias ao comportamento atual estão explícitas, sem permissões amplas desnecessárias, e o fluxo de exportação não depende de capability ausente.

## 2. Wiki e documentação afetada pela auditoria

- [x] Confirmar a correção manual de `Backup-e-restauracao.md`: o backup é criado automaticamente na área de dados da aplicação e o caminho é informado ao usuário.
- [x] Verificar se README, Wiki e `docs/` continuam coerentes após as correções técnicas desta branch.
- [x] Corrigir qualquer referência que se torne incorreta durante o trabalho.
- [x] Atualizar FAQ e página de Relatórios da Wiki para refletir os fluxos reais de backup e exportação CSV.

**Critério de conclusão:** documentação pública descreve o comportamento real da aplicação, sem duplicação técnica desnecessária.

## 3. Código Rust órfão e legado

- [x] Remover `src-tauri/src/stage3_migration_tests.rs` após confirmar que sua cobertura relevante já existe nos testes ativos.
- [x] Remover `src-tauri/src/record_views.rs` após confirmar que foi substituído por `record_states.rs`/implementação atual.
- [x] Remover `src-tauri/src/record_view_commands.rs` após confirmar que seus comandos não são usados pelo frontend nem registrados no Tauri.
- [x] Fazer uma varredura final por outros arquivos-fonte claramente órfãos antes de encerrar a etapa.

**Critério de conclusão:** nenhum arquivo removido contém comportamento ainda necessário e não permanecem fontes obviamente mortas detectadas nesta auditoria.

## 4. Recuperação de testes úteis

- [x] Revisar `src-tauri/src/production_regression_tests.rs`.
- [x] Integrar aos testes ativos o caso de produção retroativa preservando a caixa histórica correta.
- [x] Garantir cobertura ativa para tipo de produto inválido.
- [x] Garantir cobertura ativa para quantidade de produção não positiva.
- [x] Garantir cobertura ativa de produção aparecendo na timeline da colônia.
- [x] Remover `production_regression_tests.rs` somente depois que os casos úteis estiverem cobertos em arquivos compilados pelo crate.

**Critério de conclusão:** toda cobertura útil do arquivo órfão está em testes efetivamente executados por `cargo test`.

## 5. Higiene do repositório

- [x] Ajustar `.gitignore` para arquivos de ícones gerados pelo fluxo `npm run icons`, preservando apenas os arquivos-fonte que devem ser versionados.
- [x] Revisar a árvore do repositório por temporários, outputs gerados e restos de desenvolvimento claramente indevidos.
- [x] Não apagar branches antigas nesta etapa; limpeza de branches será tratada separadamente depois da integração.

**Critério de conclusão:** executar ferramentas locais de geração não deve poluir o `git status` com artefatos que não pertencem ao versionamento.

## 6. CI separado por finalidade

### Pull Request / branch

- [x] Criar ou ajustar um CI rápido para Pull Requests destinados à `main`.
- [x] Manter nele as validações essenciais: versão, dependências, frontend build/testes, `cargo fmt`, `cargo check`, `clippy` e `cargo test`.
- [x] Retirar o build Tauri desktop pesado do caminho obrigatório de todo PR, desde que a cobertura essencial permaneça protegida.
- [x] Preservar o contexto obrigatório `check` exigido pelo ruleset da `main`, ou atualizar o ruleset apenas se estritamente necessário e de forma consciente.
- [x] Avaliar filtros de caminho para que mudanças somente documentais não recompilarem desnecessariamente frontend/Rust quando isso puder ser feito sem perder validação relevante.

### `main`

- [x] Criar ou ajustar um workflow completo para `push` na `main`.
- [x] Incluir o build Tauri desktop completo na validação da `main`.
- [x] Manter validações de versão, build e testes pertinentes ao estado integrado.

### Release

- [x] Confirmar que o workflow de bundles/releases continua responsável pelos artefatos de distribuição e não duplica trabalho sem necessidade.

**Critério de conclusão:** PRs recebem feedback rápido e obrigatório; a `main` recebe validação completa; releases continuam com validação de distribuição apropriada.

## 7. Hardening pequeno e seguro

- [x] Revisar a fronteira de erros Rust → IPC para evitar que erros brutos de `sqlx`/SQLite possam chegar ao frontend por engano.
- [x] Centralizar uma mensagem pública segura para erros internos de banco sem esconder mensagens de validação e `NotFound` úteis ao usuário.
- [x] Revisar novamente caminhos de arquivos gerenciados, abertura de arquivos, backup/restauração e capabilities Tauri após as mudanças.
- [x] Não iniciar nesta branch refatoração estrutural ampla de módulos grandes.

**Critério de conclusão:** o hardening corrige riscos concretos encontrados sem transformar o PR em uma refatoração de arquitetura.

## 8. Validação e fechamento

- [x] Revisar o diff completo desta branch contra a `main`.
- [x] Confirmar que migrations existentes não foram alteradas indevidamente.
- [x] Confirmar que nenhuma chave, token, credencial ou dado pessoal foi introduzido.
- [x] Confirmar que README, Wiki e `docs/` continuam consistentes.
- [x] Confirmar execução satisfatória do CI rápido da branch/PR para as alterações técnicas.
- [x] Confirmar que a estratégia de CI completo da `main` está configurada corretamente.
- [x] Preparar resumo final das mudanças para teste manual do usuário.
- [ ] Somente depois do teste manual, preparar a integração final de uma vez, evitando branches adicionais desnecessárias.
- [ ] Antes de encerrar esta sequência, lembrar explicitamente a dívida técnica abaixo.

**Critério de conclusão:** pacote pronto para teste manual e integração única, com escopo conhecido e sem pendências ocultas da auditoria.

### Checklist de teste manual

Este teste não precisa reproduzir todo o CI. O objetivo é confirmar os fluxos visíveis afetados pelo pacote:

- [ ] Abrir a aplicação na branch `fix/post-audit-cleanup` e confirmar inicialização normal com os dados existentes.
- [ ] Abrir **Relatórios**, exportar um CSV e confirmar que o seletor nativo permite escolher nome e destino.
- [ ] Confirmar que o CSV é realmente criado e pode ser aberto normalmente.
- [ ] Abrir **Dados**, criar um backup completo e confirmar que a aplicação informa o caminho criado automaticamente.
- [ ] Navegar pelas telas principais e confirmar que não há regressão visual ou erro evidente de carregamento.
- [ ] Se houver fotos/anexos de teste disponíveis, abrir ou revelar pelo menos um arquivo gerenciado.
- [ ] Não executar restauração destrutiva apenas para este smoke test; restauração permanece coberta pelos testes e pode ser exercitada separadamente com uma cópia descartável dos dados.

Se todos os itens aplicáveis estiverem normais, o pacote pode seguir para a integração final.

---

# Dívida técnica para depois desta limpeza

Estes itens **não devem ser esquecidos**, mas ficam fora do escopo desta branch salvo se algum deles se tornar necessário para corrigir um bug atual.

- [ ] Dividir módulos Rust grandes, especialmente `record_corrections.rs`, `agenda.rs` e `data_management.rs`.
- [ ] Dividir páginas/componentes frontend grandes, especialmente Movimentações, Agenda, Assets e Relatórios.
- [ ] Adicionar um conjunto pequeno de smoke/E2E desktop que exercite fluxos reais do Tauri, inclusive dialogs/capabilities.
- [ ] Avaliar pin de GitHub Actions de terceiros por SHA completo para hardening de supply chain.
- [ ] Revisar cobertura de testes após a reorganização dos módulos.
- [ ] Limpar branches antigas e temporárias depois de confirmar que todo conteúdo relevante está integrado.
- [ ] Revisar periodicamente o tempo de CI e cache sem reduzir a proteção da `main`.

## Estado de retomada

Quando uma nova sessão precisar continuar este trabalho:

1. abrir esta branch;
2. ler este arquivo;
3. considerar concluídos **somente** os itens marcados `[x]`;
4. ler também a seção **Estado atual** para identificar o próximo marco;
5. conferir o último diff/commit antes de continuar;
6. não iniciar dívida técnica enquanto houver item obrigatório acima em aberto, salvo correção necessária para desbloquear o pacote.
