# Contribuindo com o MeliponarioManager

O MeliponarioManager usa um fluxo simples baseado em branches curtas, Pull Requests e validação automatizada antes da integração em `main`.

## Branches

Use nomes objetivos e relacionados a um único propósito:

- `feat/...` para novas funcionalidades;
- `fix/...` para correções;
- `docs/...` para documentação;
- `refactor/...` para refatorações;
- `test/...` para testes;
- `chore/...` para manutenção do projeto, CI e distribuição.

Exemplos:

- `feat/colony-inspections`
- `fix/feeding-history`
- `docs/domain-model`
- `chore/repository-polish`

Evite desenvolver diretamente em `main` quando a alteração for relevante.

## Pull Requests

Cada Pull Request deve:

- resolver um objetivo claro;
- evitar misturar mudanças não relacionadas;
- explicar o que mudou e por quê;
- informar como a alteração foi validada;
- preservar a rastreabilidade histórica do domínio;
- atualizar documentação quando comportamento, arquitetura ou fluxo de uso mudar;
- atualizar o changelog quando a mudança for relevante para uma futura release.

Mudanças de domínio devem ser documentadas em [docs/DOMAIN.md](docs/DOMAIN.md).

## Commits

Preferimos mensagens curtas no estilo Conventional Commits:

- `feat: add colony inspection flow`
- `fix: preserve colony history on box transfer`
- `docs: document release policy`
- `refactor: separate colony and hive box models`
- `chore: harden desktop distribution`

Essa convenção organiza o histórico, mas não depende de uma ferramenta específica.

## Validação local

Antes de abrir ou atualizar um Pull Request relevante, execute os checks aplicáveis.

Frontend:

```bash
npm ci
npm run icons
npm run build
```

Backend Rust:

```bash
cd src-tauri
cargo fmt --all -- --check
cargo check --locked
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

Validação do desktop sem gerar instaladores:

```bash
npm run tauri -- build --no-bundle
```

O workflow `CI` executa a mesma linha geral de validação em Pull Requests para `main`.

## Modelo de domínio

A rastreabilidade tem prioridade sobre atalhos de implementação.

Exemplos:

- trocar uma colônia de caixa cria uma nova ocupação e encerra a anterior;
- uma manutenção pertence à caixa física, mesmo quando preserva o contexto da colônia ocupante;
- uma foto pertence à inspeção que lhe dá contexto;
- documentos pertencem à movimentação correspondente;
- alertas são derivados dos dados de manejo e não mantidos como um segundo estado independente.

A interface pode usar linguagem popular e direta, mas o domínio e os dados devem permanecer tecnicamente consistentes.

## Versionamento

O projeto segue Semantic Versioning e permanece intencionalmente na série `0.x`.

- correções compatíveis incrementam `PATCH`;
- novas funcionalidades compatíveis incrementam `MINOR`;
- versões distribuídas usam tags `v0.x.y`;
- não existe meta de lançamento `v1.0.0`.

Os campos de versão devem permanecer sincronizados em:

- `package.json`;
- `src-tauri/Cargo.toml`;
- `src-tauri/tauri.conf.json`.

Enquanto o projeto estiver em fase experimental, releases públicas no GitHub são marcadas como **Pre-release**, mesmo quando a versão interna usa o formato normal `0.x.y`.

A política completa está em [docs/RELEASES.md](docs/RELEASES.md).

## Mudanças destinadas a uma release

Uma preparação de release deve, no mínimo:

- sincronizar a versão nos três arquivos do projeto;
- criar ou atualizar a seção correspondente em `CHANGELOG.md`;
- adicionar `docs/releases/v0.x.y.md` com as notas públicas daquela versão;
- confirmar que README e documentação refletem o comportamento real;
- passar pelo CI antes da criação da tag;
- criar a tag somente a partir de `main` já integrada.

O pipeline de bundles e os formatos gerados estão descritos em [docs/DISTRIBUTION.md](docs/DISTRIBUTION.md).
