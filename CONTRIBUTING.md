# Contribuindo com o MeliponarioManager

O projeto usa branches curtas, Pull Requests e validação automatizada antes da integração em `main`. Mudanças devem preservar a integridade histórica do domínio e manter a documentação correspondente atualizada.

## Antes de começar

Consulte o [índice da documentação](docs/README.md) e, para mudanças funcionais, o [modelo de domínio](docs/DOMAIN.md).

Requisitos de desenvolvimento:

- Node.js 22;
- Rust 1.94.1 com `rustfmt` e `clippy`;
- dependências nativas exigidas pelo Tauri 2 na plataforma usada;
- npm para instalar as dependências fixadas em `package-lock.json`.

Instale as dependências com:

```bash
npm ci
```

Para abrir a aplicação desktop em desenvolvimento:

```bash
npm run desktop:dev
```

O frontend isolado pode ser iniciado com `npm run dev`, mas os recursos que dependem de SQLite, sistema de arquivos ou plugins desktop só funcionam pelo Tauri.

## Branches

Crie a branch a partir da `main` atual e mantenha um único propósito por branch.

Prefixos adotados:

- `feat/`: funcionalidade;
- `fix/`: correção;
- `docs/`: documentação;
- `refactor/`: refatoração sem mudança funcional intencional;
- `test/`: cobertura de testes;
- `chore/`: dependências, CI, distribuição e manutenção.

Exemplos:

```text
feat/colony-inspections
fix/feeding-history
docs/domain-model
chore/update-tauri
```

Não desenvolva diretamente em `main`.

## Commits

Use mensagens curtas no estilo Conventional Commits:

```text
feat: add colony inspection flow
fix: preserve colony history on box transfer
docs: document release policy
refactor: separate colony and box rules
test: cover backup manifest validation
chore: update desktop dependencies
```

Commits devem explicar uma mudança coerente. Evite misturar formatação ampla, atualização de dependências e alteração funcional no mesmo commit.

## Pull Requests

Cada Pull Request deve:

- resolver um objetivo claro;
- explicar o problema e a solução;
- informar as validações executadas;
- registrar impactos no domínio, schema, dados e compatibilidade;
- atualizar a documentação e o changelog quando aplicável;
- manter a branch atualizada com `main` antes do merge;
- passar pelo status check obrigatório `check`.

A integração é feita por squash merge. A branch temporária é removida depois do merge.

## Validação local

Execute as verificações compatíveis com a mudança.

Metadados, dependências e frontend:

```bash
npm run version:check
npm ci
npm run icons
npm run bundle:check
npm run build
npm run test:ui
```

Backend Rust:

```bash
cd src-tauri
cargo fmt --all -- --check
cargo check --locked
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

Aplicação desktop sem gerar instaladores:

```bash
npm run tauri -- build --no-bundle
```

O workflow `CI` executa nas branches e Pull Requests o conjunto rápido que sustenta o status obrigatório `check`: versão, dependências, ícones, frontend, testes e verificações Rust. O build Tauri completo com `--no-bundle` é executado pelo workflow `Main validation` depois da integração em `main`. Os bundles de distribuição permanecem sob responsabilidade do workflow de release.

Uma validação local não executada deve ser marcada como não aplicável ou explicada no Pull Request.

## Regras de domínio e dados

- colônia e caixa são entidades distintas;
- mudanças históricas geram registros, transições ou novas ocupações;
- fatos anulados e operações revertidas permanecem auditáveis;
- alertas, Dashboard, timeline e relatórios são projeções, não fontes paralelas de verdade;
- operações com múltiplos efeitos devem ser transacionais;
- migrations integradas são imutáveis.

Uma alteração de schema deve usar a próxima migration numerada e incluir cobertura para instalação nova e upgrade quando houver impacto em dados existentes.

## Atualização da documentação

Use a matriz de responsabilidade em [docs/README.md](docs/README.md). Em resumo:

- regras de domínio: `docs/DOMAIN.md` e documento temático;
- arquitetura ou persistência: `docs/ARCHITECTURE.md`;
- interface: `docs/UI.md`;
- mudança relevante para usuários: `CHANGELOG.md`;
- preparação de versão: `CHANGELOG.md` e `docs/releases/<tag>.md`.

Não registre na documentação oficial prompts, conversas de desenvolvimento, caminhos pessoais, resultados temporários de uma ferramenta ou planos já abandonados.

## Versionamento e releases

O projeto segue Semantic Versioning na série `0.x`:

- correção compatível incrementa `PATCH`;
- funcionalidade compatível incrementa `MINOR`;
- tags distribuídas usam `v0.x.y`;
- releases públicas são marcadas como GitHub Pre-release enquanto o projeto permanecer experimental.

A versão deve permanecer sincronizada em:

- `package.json`;
- `src-tauri/Cargo.toml`;
- `src-tauri/tauri.conf.json`.

Consulte [Política de releases](docs/RELEASES.md) e [Distribuição desktop](docs/DISTRIBUTION.md).
