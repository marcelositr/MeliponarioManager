# Changelog

Todas as mudanças relevantes do MeliponarioManager serão registradas neste arquivo.

O projeto utiliza Versionamento Semântico no formato `vMAJOR.MINOR.PATCH` e permanecerá na série experimental `0.x`.

## [Unreleased]

### Added

- Fundação inicial do repositório e fluxo de desenvolvimento.
- Bootstrap da aplicação desktop com Rust, Tauri 2, React, TypeScript e Vite.
- Persistência local com SQLite e SQLx.
- Migration inicial para meliponários, espécies, caixas, colônias, inspeções, eventos e histórico de ocupação de caixas.
- Tela inicial com verificação do estado da aplicação e conexão com o banco local.
- Workflow de CI para frontend, formatação, compilação e testes Rust.
