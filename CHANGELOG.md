# Changelog

Todas as mudanças relevantes do MeliponarioManager são registradas neste arquivo.

O formato segue os princípios do [Keep a Changelog](https://keepachangelog.com/) e o projeto utiliza Semantic Versioning na série experimental `0.x`.

## [Unreleased]

Nenhuma mudança preparada após a primeira pré-release pública.

## [0.7.0] - 2026-08-27

Primeira versão preparada para distribuição pública como **GitHub Pre-release**. Esta versão consolida o desenvolvimento realizado antes da criação das primeiras tags públicas; os marcos `0.1` a `0.6` existiram como planejamento de evolução e não como releases publicadas.

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

[Unreleased]: https://github.com/marcelositr/MeliponarioManager/compare/v0.7.0...HEAD
[0.7.0]: https://github.com/marcelositr/MeliponarioManager/releases/tag/v0.7.0
