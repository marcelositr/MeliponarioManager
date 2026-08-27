# Changelog

Todas as mudanças relevantes do MeliponarioManager são registradas neste arquivo.

O formato segue os princípios do [Keep a Changelog](https://keepachangelog.com/) e o projeto utiliza Semantic Versioning na série experimental `0.x`.

## [Unreleased]

Nenhuma mudança preparada após a pré-release `v0.7.1`.

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

[Unreleased]: https://github.com/marcelositr/MeliponarioManager/compare/v0.7.1...HEAD
[0.7.1]: https://github.com/marcelositr/MeliponarioManager/releases/tag/v0.7.1
[0.7.0]: https://github.com/marcelositr/MeliponarioManager/tree/v0.7.0
