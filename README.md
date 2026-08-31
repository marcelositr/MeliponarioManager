# MeliponarioManager

[![Main validation](https://github.com/marcelositr/MeliponarioManager/actions/workflows/ci-main.yml/badge.svg?branch=main)](https://github.com/marcelositr/MeliponarioManager/actions/workflows/ci-main.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/status-experimental-orange.svg)](docs/ROADMAP.md)

Aplicação desktop **local-first** para organização, manejo e rastreabilidade de meliponários. O MeliponarioManager mantém a colônia como uma entidade histórica separada da caixa física, preservando trocas de caixa, manejos, divisões, movimentações e demais eventos ao longo do tempo.

Os dados permanecem no computador do usuário, com persistência em SQLite e arquivos de mídia gerenciados localmente. Não é necessária conta ou serviço em nuvem para utilizar a aplicação.

## Principais recursos

- múltiplos meliponários, espécies, colônias e caixas físicas;
- histórico de ocupação de caixas sem perder a identidade da colônia;
- inspeções, alimentação, produção, manutenção, eventos e fotos;
- divisões, genealogia, ciclo de vida e histórico consolidado;
- transferências, transportes temporários e documentos de movimentação;
- Agenda, alertas, Dashboard e relatórios com exportação CSV e impressão;
- backup completo, restauração validada e diagnóstico de arquivos locais.

## Instalação e uso

Builds experimentais para **Linux** (`.deb` e AppImage) e **Windows** (NSIS e MSI) são publicados na página de [Releases](https://github.com/marcelositr/MeliponarioManager/releases).

O manual de instalação e uso está na [GitHub Wiki](https://github.com/marcelositr/MeliponarioManager/wiki).

## Executar a partir do código

O projeto utiliza Node.js 22 e o toolchain Rust definido em `rust-toolchain.toml`.

```sh
npm ci
npm run desktop:dev
```

## Documentação

A [Wiki](https://github.com/marcelositr/MeliponarioManager/wiki) concentra a documentação para usuários. A documentação de engenharia está organizada em [docs/](docs/README.md), com arquitetura, domínio, persistência, distribuição e processo de release.

Consulte também o [changelog](CHANGELOG.md), o [guia de contribuição](CONTRIBUTING.md) e a [política de segurança](SECURITY.md).

## Status do projeto

O MeliponarioManager está em desenvolvimento experimental na série `0.x`. As versões distribuídas devem ser tratadas como **Pre-release** e podem receber mudanças de interface, fluxos e formato de dados entre versões.

Mantenha backups atualizados, especialmente antes de instalar uma nova versão.

## Licença

Distribuído sob a [MIT License](LICENSE).
