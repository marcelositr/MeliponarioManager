# MeliponarioManager

[![CI](https://github.com/marcelositr/MeliponarioManager/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/marcelositr/MeliponarioManager/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/status-experimental-orange.svg)](docs/ROADMAP.md)

Aplicação desktop local-first para gerenciamento de meliponários, com foco em manejo, histórico e rastreabilidade de colônias de abelhas sem ferrão.

> **Status:** projeto experimental em desenvolvimento contínuo. O MeliponarioManager permanecerá na série `0.x` e não possui meta de chegar à versão `1.0.0`.

## Visão geral

O MeliponarioManager organiza o plantel sem confundir a colônia biológica com a caixa física que ela ocupa. Mudanças de caixa, divisões, movimentações, perdas, inspeções e demais fatos relevantes preservam o histórico em vez de sobrescrever o passado.

A aplicação é desktop e local-first. O frontend React conversa com o backend Rust através do IPC do Tauri; regras de domínio e persistência permanecem no backend. Os dados estruturados ficam em SQLite e os arquivos de mídia ficam no diretório de dados da aplicação.

## Recursos atuais

### Plantel e estrutura

- múltiplos meliponários;
- cadastro de espécies;
- colônias com origem, situação, data de instalação e relação de colônia-mãe;
- caixas físicas separadas das colônias;
- histórico de ocupação de caixas;
- movimentação de colônias entre caixas livres sem apagar ocupações anteriores.

### Manejo

- inspeções com força da colônia, rainha, postura, reservas, crias, pragas, observações, ações realizadas e próxima inspeção;
- alimentação e suplementação com quantidade, unidade, resposta observada e próximo manejo;
- manutenção de caixas físicas com preservação do contexto histórico da colônia ocupante;
- fotos associadas às inspeções, armazenadas fora do banco em área de mídia gerenciada;
- produção de mel, pólen, própolis, cera, cerume e outros produtos.

### Histórico e rastreabilidade

- eventos operacionais e biológicos por colônia;
- timeline unificada com ocupações, inspeções, eventos, alimentação, produção, movimentações, manutenção e ciclo de vida;
- divisões e multiplicações com criação de descendentes e genealogia;
- transferências internas, transferências externas e transportes temporários;
- documentos estruturados associados às movimentações, incluindo GTA, autorizações, notas fiscais, recibos, declarações, protocolos, certificados e outros registros;
- baixa por perda, inativação e reativação como transições explícitas de ciclo de vida;
- alertas derivados dos dados atuais de manejo, sem tabela paralela de pendências.

### Operação e dados

- dashboard operacional com situação do plantel, força das últimas inspeções, distribuição por espécie, ocupação de caixas, alertas, produção e movimentações recentes;
- backup do SQLite e da mídia;
- exportação portátil em JSON;
- relatório gerencial em Markdown;
- restauração preparada com validação de integridade e backup de segurança antes da troca dos dados.

## Arquitetura

```text
React + TypeScript
        |
        | Tauri IPC
        v
Rust / regras de domínio
        |
        | SQLx
        v
SQLite + arquivos locais de mídia
```

Princípios principais:

- o frontend não acessa o SQLite diretamente;
- a colônia é uma entidade histórica;
- caixa física e colônia são entidades distintas;
- fatos históricos relevantes não são sobrescritos;
- alertas, timeline e dashboard são visões derivadas dos registros reais;
- arquivos binários não são armazenados como BLOB no SQLite.

A documentação detalhada do modelo está em [docs/DOMAIN.md](docs/DOMAIN.md).

## Dados locais

O banco SQLite é criado automaticamente no diretório de dados da aplicação e recebe as migrations durante a inicialização.

Fotos importadas pelo backend são mantidas em `media/inspections/<inspection-id>/`, enquanto o SQLite armazena apenas metadados e caminhos relativos.

Recursos de backup, exportação, relatório e restauração são descritos em [docs/DATA-MANAGEMENT.md](docs/DATA-MANAGEMENT.md).

## Referências de rastreabilidade

A modelagem de origem, plantel, movimentações e documentos considera como referência conceitual fluxos utilizados por GEFAU, GEDAVE e GTA no Estado de São Paulo.

O MeliponarioManager **não substitui sistemas oficiais, não emite autorizações e não certifica validade jurídica de documentos**. Essas referências servem para estruturar os dados e preservar rastreabilidade sem importar burocracia desnecessária para o uso cotidiano.

## Tecnologias

- Rust 1.94.1;
- Tauri 2;
- React 19;
- TypeScript;
- Vite 8;
- SQLite;
- SQLx 0.9.

## Desenvolvimento local

### Pré-requisitos

- Node.js 22;
- Rust 1.94.1;
- dependências de sistema exigidas pelo Tauri para a plataforma utilizada.

Instale as dependências do frontend:

```bash
npm install
```

Execute apenas o frontend:

```bash
npm run dev
```

Execute a aplicação desktop em modo de desenvolvimento:

```bash
npm run desktop:dev
```

Gere um build desktop local:

```bash
npm run desktop:build
```

## Qualidade e CI

Pull Requests para `main` passam pelo workflow de CI, que valida:

- geração dos ícones desktop;
- build React/TypeScript;
- formatação Rust com `cargo fmt --check`;
- `cargo check`;
- `cargo clippy --all-targets -- -D warnings`;
- `cargo test`;
- build Tauri sem bundle para validar o binário desktop.

## Distribuição

O projeto possui pipeline para gerar:

- Linux: `.deb` e AppImage;
- Windows: NSIS e MSI.

Enquanto o projeto permanecer experimental, as versões publicadas no GitHub são tratadas como **Pre-release**. Instaladores Windows ainda não possuem assinatura de código e devem ser considerados builds de teste.

Consulte [docs/DISTRIBUTION.md](docs/DISTRIBUTION.md) e [docs/RELEASES.md](docs/RELEASES.md) para o fluxo completo.

## Documentação

- [Modelo de domínio](docs/DOMAIN.md)
- [Roadmap experimental](docs/ROADMAP.md)
- [Gerenciamento e segurança dos dados](docs/DATA-MANAGEMENT.md)
- [Distribuição desktop](docs/DISTRIBUTION.md)
- [Política de versões e releases](docs/RELEASES.md)
- [Changelog](CHANGELOG.md)
- [Guia de contribuição](CONTRIBUTING.md)

## Versionamento

O projeto segue Semantic Versioning no formato `vMAJOR.MINOR.PATCH`, permanecendo intencionalmente na série `0.x`.

- correções compatíveis incrementam `PATCH`;
- novas funcionalidades compatíveis incrementam `MINOR`;
- tags de distribuição usam o formato `v0.x.y`;
- o histórico de desenvolvimento anterior à primeira release pública permanece registrado em commits e Pull Requests, sem criação retroativa de tags artificiais.

## Contribuição

Alterações relevantes são desenvolvidas em branches curtas e entram em `main` por Pull Request. Convenções de branches, commits, testes e documentação estão em [CONTRIBUTING.md](CONTRIBUTING.md).

## Licença

Distribuído sob a licença MIT. Consulte [LICENSE](LICENSE).
