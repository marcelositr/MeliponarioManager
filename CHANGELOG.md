# Changelog

Todas as mudanças relevantes do MeliponarioManager são registradas neste arquivo.

O formato segue os princípios do [Keep a Changelog](https://keepachangelog.com/) e o projeto utiliza Semantic Versioning na série experimental `0.x`.

## [Unreleased]

## [0.8.0] - 2026-08-30

### Added

- Fundação histórica para estados físicos de caixas, com transições `active`, `maintenance` e `retired`, consulta de histórico e proteção SQLite contra nova ocupação em caixa não ativa.
- Política temporal central para timestamps operacionais em horário local, normalizados para `YYYY-MM-DD HH:MM:SS` na fronteira Tauri.
- Validação histórica da disponibilidade da colônia para inspeções, alimentações e produção, permitindo lançamentos retroativos válidos anteriores a perda, inativação ou transferência externa.
- Teste explícito de upgrade a partir de um schema representativo da `v0.7.1`, preservando ocupações, eventos, fotos, documentos e relações ao aplicar a migration nova.
- Novo shell desktop enterprise com menu superior, barra contextual, sidebar agrupada e recolhível, workspace e status bar.
- Temas claro, escuro e seguir sistema baseados em tokens compartilhados, com preferência persistida localmente.
- Contexto visual persistente de meliponário ativo, incluindo visão consolidada e escopo operacional aplicado às telas que possuem contrato confiável.
- Componentes reutilizáveis para dialogs, confirmações, toolbars, ícones consistentes e fichas internas de registros.
- Ficha-resumo inicial de colônia como infraestrutura para áreas internas futuras.
- Migration `0014_audit_and_record_corrections.sql` com auditoria antes/depois, arquivamento administrativo, correção/anulação de fatos e suporte a reversões seguras sem apagar o histórico original.
- Políticas administrativas específicas para editar, arquivar, reativar ou excluir somente cadastros mestres nunca utilizados, além de correção/anulação para fatos operacionais.
- Migration `0015_operational_agenda.sql` com tarefas manuais e derivadas, prioridade, estados operacionais, linhagem de origem e histórico de reagendamento, cancelamento e conclusão.
- Agenda operacional com filtros por vencimento, tipo, prioridade, colônia e meliponário, criação manual, reagendamento, cancelamento, tarefa ignorada, duplicação e execução contextual.
- Execução transacional de tarefas de inspeção, alimentação e manutenção, registrando o fato real antes de concluir o compromisso correspondente.
- Reconciliação idempotente de tarefas derivadas de `next_inspection_at`, `next_feeding_at` e `next_maintenance_at`, incluindo recuperação de compromisso ausente sem duplicação.
- Centros operacionais dedicados de Colônia, Caixa e Meliponário, expondo Agenda, alertas e projeções contextuais sem criar nova fonte de verdade.
- Integração da Agenda ao shell/sidebar/router e atalhos de navegação entre Agenda, alertas, colônias e caixas.
- Testes frontend específicos da Etapa 4 para rota da Agenda, contexto ativo, alertas acionáveis, conjunto de ações e consumo dos record centers.
- Documentação dedicada da Agenda em `docs/AGENDA.md`.
- Central de Relatórios dedicada com visão operacional, produção, custos registrados, Agenda, histórico de colônia e consolidação por meliponário, calculados por serviços Rust tipados.
- Exportação CSV UTF-8 com separador `;`, ordem determinística, cabeçalhos humanos, escaping de conteúdo e proteção contra formula injection em campos textuais.
- Saída imprimível dos relatórios pela WebView e impressão do sistema, com CSS específico para papel e fundo claro mesmo quando a aplicação está em tema escuro.
- Documentação dedicada dos relatórios em `docs/REPORTS.md`, incluindo filtros, semântica de métricas, estado efetivo, CSV, impressão e limitações.
- Migration aditiva `0017_managed_attachments.sql` para anexos associados a meliponários, preservando metadata no SQLite e binários na área local gerenciada.
- Painel contextual de Arquivos no Record Center do Meliponário, com seleção por Dialog nativo, descrição, abertura, revelação e remoção confirmada.
- Diagnóstico de arquivos que cruza fotos e anexos registrados com o filesystem e informa assets ausentes ou arquivos sem referência, sem limpeza automática.
- Manifest v1 para backups completos, com formato, versão, schema e inventário de assets.
- Exportação JSON estrutural versionada com IDs, relações e metadata de fotos/anexos, declarando explicitamente que binários não estão incorporados.
- Estado da janela persistido pelo plugin oficial `tauri-plugin-window-state`.
- Thumbnails lazy para fotos de inspeção com limite de bytes para prévia e estados explícitos de arquivo ausente ou prévia indisponível.
- Documentação dedicada de arquivos gerenciados em `docs/FILES.md` e testes frontend específicos do Bloco 5C.
- Importação dedicada de listas de espécies em CSV, com seleção nativa, prévia antes da gravação, detecção de duplicidades e importação transacional sem alterar cadastros existentes nem assumir validação legal por estado.

### Changed

- `weak` e `recovering` passam a ser tratados como valores legados administrativamente ativos; condição de fraqueza operacional é derivada da última inspeção.
- Alertas usam a mesma referência temporal local e deixam de tratar `colonies.status = 'weak'` como fonte de verdade para fraqueza.
- Dashboard consolida `weak` e `recovering` legados na projeção administrativa `active`, mantendo força de manejo separada pela última inspeção.
- Timestamps de ocupação, ciclo de vida, inspeção, alimentação, produção, eventos, divisões, movimentações, manutenção, documentos e fotos são normalizados antes de chegar aos serviços existentes.
- A versão exibida no aplicativo deixa de ser hardcoded e passa a vir dos metadados reais do Tauri, com fallback seguro fora do runtime desktop.
- Dashboard passa a priorizar indicadores executivos, situação do plantel, ocupação, alertas, produção e movimentações em uma composição mais compacta.
- Meliponários, espécies, caixas, colônias, inspeções, alimentação, produção, histórico, divisões, movimentações, manutenção e ciclo de vida adotam toolbars, tabelas e dialogs no lugar de formulários permanentes nas telas principais.
- Fotos deixam de ocupar um conceito principal de navegação e permanecem acessíveis como contexto transitório dentro de Manutenção, sem perda de funcionalidade.
- A interface mantém abordagem desktop-first, mas passa a adaptar shell, sidebar, grids, toolbars, dialogs e tabelas sem depender de um piso estrutural de 900 px; `1024x768` é a faixa de primeira classe e `800x600` permanece operacionalmente suportado.
- Fatos anulados e operações revertidas deixam de participar das projeções operacionais atuais, mas permanecem visíveis em histórico e auditoria.
- Alertas de inspeção, alimentação e manutenção vencidas passam a usar tarefas pendentes da Agenda como fonte operacional, evitando um segundo vencimento derivado em paralelo.
- Alertas passam a transportar `task_id`, contexto de meliponário/caixa/colônia e `recommended_action`, permitindo navegação direta para Agenda ou manejo recomendado.
- O seletor de meliponário ativo passa a filtrar Agenda, alertas, Dashboard, colônias, caixas e fluxos de manejo, preservando catálogos globais apenas onde o domínio exige destinos externos ao contexto, como transferências.
- Dashboard passa a integrar resumo da Agenda e alertas acionáveis; quando há contexto ativo, indicadores sem contrato de escopo confiável não são misturados silenciosamente com a visão filtrada.
- Fichas de Colônia, Caixa e Meliponário passam a consumir projeções `record_center` do backend como hubs de leitura e navegação.
- A área `Dados` volta a ficar dedicada a backup, restauração e exportação estrutural, enquanto relatórios operacionais passam a possuir rota, menu e sidebar próprios.
- Relatórios operacionais aplicam estado efetivo no backend: fatos anulados ou revertidos não entram silenciosamente em totais válidos, e histórico completo/auditoria é um modo explícito.
- Fotos de inspeção deixam de exigir caminho digitado e passam a usar seleção nativa, contexto humano, abertura/revelação do arquivo e carregamento de prévia sob demanda.
- Backup completo passa a validar manifest, schema e inventário de mídia antes do staging; restauração aplica troca de banco/mídia com cópia de segurança e rollback local.
- JSON portátil passa a ter contrato estrutural explícito e não é apresentado como mecanismo de restauração completa.

### Fixed

- Nova ocupação é rejeitada explicitamente pelo backend quando a caixa de destino não está ativa.
- Próxima inspeção, alimentação e manutenção são rejeitadas quando anteriores ao fato que as originou após normalização temporal consistente.
- Confirmações críticas de ciclo de vida e remoção de fotos deixam de usar diálogos genéricos do navegador e passam a explicar a consequência da operação.
- Reconciliação da Agenda recupera tarefa derivada artificialmente ausente mantendo idempotência após execuções repetidas.
- Ordenação combinada dos alertas usa uma consulta externa após o `UNION ALL`, mantendo prioridade/data/chave compatíveis com SQLite.
- Reabertura de transporte temporário não pode criar dois transportes abertos para a mesma colônia, incluindo proteção contra bypass direto pelo SQLite.
- Registros de arquivos ausentes permanecem preservados e são sinalizados em vez de serem apagados silenciosamente.
- Falhas na gravação de metadata de anexo compensam a cópia física recém-criada, evitando arquivo gerenciado órfão no fluxo normal de importação.
- Dialogs preservam foco durante edição, menus de ações passam a flutuar sem deslocar tabelas e grupos de controles recebem espaçamento/wrapping consistente na camada visual compartilhada.
- Contraste dos temas claro/escuro, seleção de texto da superfície desktop e composição responsiva recebem hardening após o field test da release candidate.

## [0.7.1] - 2026-08-27

Correção de distribuição preparada após a primeira tentativa de empacotamento da tag `v0.7.0`.

### Fixed

- Declara explicitamente os ícones de bundle no `src-tauri/tauri.conf.json`, incluindo PNGs quadrados para Linux, `icon.ico` para Windows e `icon.icns` para compatibilidade de plataforma.
- Corrige a falha do AppImage que interrompia o bundler por não encontrar um ícone quadrado configurado.
- Corrige a falha do MSI/WiX que não encontrava um arquivo `.ico` configurado para o instalador Windows.

### Changed

- Sincroniza `package.json`, `src-tauri/Cargo.toml` e `src-tauri/tauri.conf.json` em `0.7.1`.
- Adiciona validação automática da configuração e existência dos ícones gerados antes do build desktop e dos bundles de distribuição.
- Mantém a tag `v0.7.0` imutável e utiliza um incremento `PATCH` em vez de reescrever a tentativa anterior.

## [0.7.0] - 2026-08-27

Primeira tag pública do projeto e primeira tentativa de distribuição. Esta versão consolida o desenvolvimento realizado antes da criação das primeiras tags públicas; os marcos `0.1` a `0.6` existiram como planejamento de evolução e não como releases publicadas.

O pipeline chegou a gerar o `.deb` no Linux e o instalador NSIS no Windows, mas a etapa completa de distribuição falhou antes da criação da GitHub Release: o AppImage exigia um ícone quadrado explicitamente configurado e o MSI/WiX exigia um `.ico` declarado no bundle. A tag foi preservada e a correção seguiu como `v0.7.1`.

### Added

- Fundação desktop com Rust, Tauri 2, React, TypeScript, Vite, SQLite e SQLx.
- Suporte a múltiplos meliponários, espécies, caixas físicas, colônias e histórico de ocupação de caixas.
- Inspeções com contexto histórico da caixa ocupada na data registrada.
- Eventos operacionais e biológicos e timeline unificada por colônia.
- Alimentação e suplementação com acompanhamento de próximo manejo.
- Produção de mel, pólen, própolis, cera, cerume e outros produtos.
- Divisões e multiplicações com criação de descendentes e consulta de genealogia.
- Movimentações internas, externas e transportes temporários preservando origem, destino e contexto histórico.
- Manutenção de caixas físicas sem confundir manutenção com troca de caixa.
- Alertas derivados de inspeções, alimentação e condição da colônia, sem estado paralelo persistido.
- Ciclo de vida explícito para perda, inativação e reativação de colônias.
- Documentos estruturados vinculados às movimentações, incluindo compatibilidade com o campo legado `document_reference`.
- Fotos vinculadas às inspeções com arquivos armazenados na área local de mídia e metadados mantidos no SQLite.
- Interface operacional para cadastros, inspeções, alimentação, produção, eventos, timeline, alertas, divisões, genealogia, movimentações, documentos, fotos, manutenção e ciclo de vida.
- Dashboard operacional derivado pelo backend com situação do plantel, força das últimas inspeções, distribuição por espécie, ocupação de caixas, alertas, produção e movimentações recentes.
- Backup do SQLite e da mídia, exportação portátil em JSON, relatório gerencial em Markdown e restauração preparada com validação de integridade e backup de segurança.
- Pipeline de bundles Linux (`deb` e AppImage) e Windows (NSIS e MSI).

### Changed

- CI ampliado para validar geração de ícones, build frontend, formatação Rust, `cargo check`, Clippy com warnings tratados como erro, testes e build Tauri sem bundle.
- Rust fixado em `1.94.1` para desenvolvimento e CI.
- Ícones desktop passam a ser gerados a partir de `assets/app-icon.svg`.
- Fluxo de distribuição padronizado em tags `v0.x.y`.
- Documentação reorganizada para refletir o estado real da aplicação e separar README, domínio, roadmap, gerenciamento de dados, distribuição e política de releases.

### Security

- Content Security Policy configurada para restringir a aplicação ao conteúdo local e aos protocolos necessários ao IPC e aos assets do Tauri.
- Cabeçalho `X-Content-Type-Options: nosniff` habilitado na configuração da aplicação.

[Unreleased]: https://github.com/marcelositr/MeliponarioManager/compare/v0.8.0...HEAD
[0.8.0]: https://github.com/marcelositr/MeliponarioManager/compare/v0.7.1...v0.8.0
[0.7.1]: https://github.com/marcelositr/MeliponarioManager/releases/tag/v0.7.1
[0.7.0]: https://github.com/marcelositr/MeliponarioManager/tree/v0.7.0
