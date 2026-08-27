# Distribuição desktop

O MeliponarioManager permanece experimental na série `0.x`. Os bundles desta fase são destinados a teste e validação e são publicados como **GitHub Pre-release**.

## Validação em Pull Request

O workflow `CI` valida a aplicação antes da integração em `main`.

A sequência geral inclui:

- checkout do repositório;
- dependências de sistema necessárias ao Tauri em Linux;
- Node.js 22;
- Rust 1.94.1 com `rustfmt` e `clippy`;
- instalação das dependências do frontend;
- geração dos ícones desktop a partir de `assets/app-icon.svg`;
- validação da configuração e dos arquivos de ícone usados pelos bundles;
- build React/TypeScript;
- `cargo fmt --all -- --check`;
- `cargo check`;
- `cargo clippy --all-targets -- -D warnings`;
- `cargo test`;
- build Tauri com `--no-bundle` para validar o binário desktop sem produzir instaladores.

Rust também é fixado em `1.94.1` por `rust-toolchain.toml`.

## Bundles Linux e Windows

O workflow `Build desktop bundles` pode ser executado manualmente na aba Actions ou disparado por uma tag de versão.

Plataformas atuais:

- Linux `ubuntu-22.04`: `deb` e AppImage;
- Windows `windows-latest`: NSIS e MSI.

Em execução manual, os bundles são anexados à própria execução do GitHub Actions como artefatos para teste.

## Tags de distribuição

Tags públicas seguem a convenção:

```text
v0.x.y
```

Exemplo:

```text
v0.7.1
```

O workflow de bundles reage a tags `v0.*`.

Antes de criar uma tag, a versão precisa estar sincronizada em:

- `package.json`;
- `src-tauri/Cargo.toml`;
- `src-tauri/tauri.conf.json`.

A tag deve apontar para um commit já integrado em `main` e com CI verde.

Tags publicadas são tratadas como imutáveis. Se uma tentativa de distribuição revelar um erro de empacotamento após a criação da tag, a correção segue em um novo `PATCH` em vez de reescrever a tag anterior.

## Notas de release versionadas

Cada versão distribuída deve possuir um arquivo correspondente em:

```text
docs/releases/v0.x.y.md
```

Exemplo:

```text
docs/releases/v0.7.1.md
```

Durante um build disparado por tag, o workflow carrega esse arquivo e usa seu conteúdo como descrição da GitHub Release. Se as notas correspondentes à tag estiverem ausentes, o build de release deve falhar em vez de publicar uma descrição improvisada.

## GitHub Release

Para tags `v0.*`, o pipeline cria ou atualiza uma release com:

- título `MeliponarioManager <tag>`;
- conteúdo vindo de `docs/releases/<tag>.md`;
- `Draft` habilitado inicialmente;
- `Pre-release` habilitado;
- bundles Linux e Windows anexados à mesma release.

O draft permite revisar título, notas e arquivos antes da publicação manual da pré-release.

## Ícones

A fonte oficial dos ícones desktop é:

```text
assets/app-icon.svg
```

O script `npm run icons` executa `tauri icon` para gerar os formatos exigidos pelas plataformas antes dos builds e bundles.

Além de gerar os arquivos, `src-tauri/tauri.conf.json` precisa declarar explicitamente os ícones usados pelo bundler:

```json
"bundle": {
  "active": true,
  "targets": "all",
  "icon": [
    "icons/32x32.png",
    "icons/128x128.png",
    "icons/128x128@2x.png",
    "icons/icon.icns",
    "icons/icon.ico"
  ]
}
```

Essa declaração é necessária porque diferentes empacotadores exigem formatos específicos. O AppImage precisa localizar um ícone PNG quadrado e o MSI/WiX precisa localizar um `.ico`.

O comando `npm run bundle:check` valida tanto a lista de `bundle.icon` quanto a existência dos arquivos gerados. O CI e o workflow de bundles executam essa verificação imediatamente após `npm run icons`.

Arquivos gerados em `src-tauri/icons/` não são tratados como fonte de design independente.

### Incidente da tag `v0.7.0`

A primeira tentativa de distribuição confirmou que apenas gerar os ícones não era suficiente. O `.deb` e o NSIS chegaram a ser produzidos, mas o AppImage falhou por não encontrar um ícone quadrado configurado e o MSI/WiX falhou por não encontrar um `.ico` declarado.

A tag `v0.7.0` foi preservada sem reescrita. A correção foi preparada como `v0.7.1`, com configuração explícita e validação preventiva no CI.

## Assinatura Windows

Os instaladores Windows ainda não possuem certificado de assinatura de código configurado.

Enquanto essa limitação existir:

- builds Windows devem ser tratados como experimentais;
- a release deve informar claramente que os instaladores não são assinados;
- credenciais futuras de assinatura devem ser armazenadas apenas em GitHub Secrets ou mecanismo equivalente;
- chaves e certificados privados nunca devem ser adicionados ao repositório.

## Dependências reproduzíveis

`package-lock.json` e `Cargo.lock` ainda não estão versionados.

Enquanto isso, os workflows usam `npm install` e a resolução normal do Cargo. Essa situação é aceitável para a fase experimental atual, mas reduz a reprodutibilidade exata dos builds.

Os lockfiles devem ser gerados pelas ferramentas oficiais com acesso aos registries e versionados em uma alteração própria. Não devem ser montados manualmente.

Depois que `package-lock.json` estiver versionado, o pipeline pode migrar de `npm install` para `npm ci`.

## Fluxo recomendado

1. concluir a alteração em uma branch curta;
2. atualizar documentação e changelog quando necessário;
3. abrir Pull Request para `main`;
4. aguardar CI verde;
5. integrar a PR;
6. confirmar a versão e as notas em `docs/releases/`;
7. criar a tag `v0.x.y` a partir de `main`;
8. aguardar os bundles Linux e Windows;
9. revisar a release em draft;
10. publicar como Pre-release.

A política de versionamento e publicação está detalhada em [RELEASES.md](RELEASES.md).
