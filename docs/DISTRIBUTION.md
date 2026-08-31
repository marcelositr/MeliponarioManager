# Distribuição desktop

O pipeline de distribuição gera bundles experimentais para Linux e Windows. Versões públicas são associadas a tags `v0.x.y` e publicadas como GitHub Pre-release.

## Validação contínua

A validação é dividida em duas camadas.

### Branches e Pull Requests

O workflow `CI` roda em branches de trabalho e Pull Requests para `main`. O job obrigatório `check` executa:

1. validação dos metadados de versão;
2. instalação reproduzível com `npm ci`;
3. geração e validação dos ícones;
4. build React/TypeScript;
5. testes da interface;
6. `cargo fmt --all -- --check`;
7. `cargo check --locked`;
8. `cargo clippy --locked --all-targets -- -D warnings`;
9. `cargo test --locked`.

O build Tauri desktop completo fica fora do caminho obrigatório de cada Pull Request para reduzir o tempo de feedback sem remover as verificações de frontend, Rust e testes.

### `main`

O workflow `Main validation` roda após pushes para `main` e também pode ser iniciado manualmente. Ele repete as verificações essenciais e acrescenta:

```bash
npm run tauri -- build --no-bundle
```

Assim, a branch protegida recebe uma validação integrada completa sem transformar cada iteração de desenvolvimento em um build desktop pesado.

O ambiente Linux usa Node.js 22 e Rust 1.94.1. A mesma versão Rust está fixada em `rust-toolchain.toml`.

## Workflow de bundles

O workflow `Build desktop bundles` aceita:

- execução manual por `workflow_dispatch`;
- execução automática para tags `v0.*`.

Matriz atual:

| Plataforma | Runner | Formatos |
| --- | --- | --- |
| Linux | `ubuntu-22.04` | `.deb` e AppImage |
| Windows | `windows-latest` | NSIS e MSI |

Em execução manual, os bundles são anexados à execução como artefatos de teste. Em execução por tag, também são anexados à GitHub Release em draft.

## Versão e notas

Antes da tag, mantenha sincronizados:

- `package.json`;
- `src-tauri/Cargo.toml`;
- `src-tauri/tauri.conf.json`.

A tag deve usar `v0.x.y` e apontar para `main`. O workflow compara o nome da tag com os metadados do aplicativo.

As notas devem existir em:

```text
docs/releases/<tag>.md
```

A ausência desse arquivo interrompe o build de release. O conteúdo é usado diretamente no corpo da GitHub Release.

## Ícones

A fonte oficial do ícone é:

```text
assets/app-icon.svg
```

`npm run icons` gera os formatos de plataforma em `src-tauri/icons/`. Essa pasta é gerada e não deve ser versionada. A configuração `bundle.icon` de `src-tauri/tauri.conf.json` declara os PNGs, `.ico` e `.icns` consumidos pelos empacotadores.

`npm run bundle:check` valida a configuração e a presença dos arquivos gerados. Arquivos em `src-tauri/icons/` não são fontes de design independentes.

## Dependências reproduzíveis

`package-lock.json` e `src-tauri/Cargo.lock` são versionados. Os workflows usam `npm ci` e comandos Cargo com `--locked`.

Atualizações devem ser feitas pelas ferramentas dos respectivos ecossistemas. Não edite lockfiles manualmente.

## Diagnóstico no Linux

Algumas combinações de driver gráfico, ambiente desktop e WebKitGTK podem causar falha ou corrupção de renderização. O seguinte workaround pode ser testado somente para confirmar esse cenário:

```bash
GIO_MODULE_DIR= WEBKIT_DISABLE_DMABUF_RENDERER=1 <comando-do-aplicativo>
```

- `GIO_MODULE_DIR=` evita carregar módulos GIO problemáticos naquele processo;
- `WEBKIT_DISABLE_DMABUF_RENDERER=1` desativa o caminho DMABUF do WebKitGTK;
- não exporte essas variáveis globalmente;
- não aplique o workaround quando a renderização funciona normalmente.

Esse ajuste não corrige falhas de build, dependências ausentes ou problemas gerais da aplicação.

## Assinatura no Windows

Nenhum certificado de assinatura de código está configurado.

Enquanto isso:

- instaladores Windows devem ser tratados como experimentais;
- a limitação deve aparecer nas notas de release;
- certificados e chaves privadas devem permanecer fora do repositório;
- uma integração futura deve usar GitHub Secrets ou mecanismo equivalente.

## Checklist de publicação

1. integrar o Pull Request de release em `main`;
2. confirmar versão, changelog e notas;
3. criar a tag no commit aprovado;
4. aguardar todos os jobs da matriz;
5. baixar e inspecionar os artefatos;
6. conferir o draft da GitHub Release;
7. realizar teste manual mínimo dos bundles;
8. publicar como Pre-release.

A política completa está em [RELEASES.md](RELEASES.md).
