# Distribuição desktop

O MeliponarioManager permanece experimental na série `0.x`. Os bundles desta fase são destinados a teste e validação, não implicam uma versão `1.0`.

## Validação em pull request

O workflow `CI` executa, nesta ordem geral:

- instalação das dependências do frontend;
- geração e validação do conjunto de ícones Tauri a partir de `src-tauri/icons/icon.png`;
- build React/TypeScript;
- `cargo check`;
- `cargo clippy --all-targets`;
- `cargo test`;
- `tauri build --no-bundle`, validando a compilação do desktop sem produzir instaladores.

Rust é fixado em `1.94.1` por `rust-toolchain.toml` e também explicitamente no CI.

## Bundles Linux e Windows

O workflow `Build desktop bundles` pode ser executado manualmente na aba Actions. Ele produz:

- Linux: `deb` e `AppImage`;
- Windows: `NSIS` e `MSI`.

Os artefatos são anexados à própria execução do GitHub Actions.

Ao enviar uma tag no formato `app-v*`, por exemplo `app-v0.1.0`, o mesmo workflow cria uma **release draft e prerelease**, deixando os instaladores para revisão antes de qualquer publicação.

## Ícones

O repositório mantém `src-tauri/icons/icon.png` como fonte. O pipeline executa `tauri icon` antes do empacotamento para gerar os formatos e tamanhos específicos de cada plataforma, incluindo ICO no Windows e PNGs de desktop no Linux.

## Assinatura

Os instaladores Windows ainda não possuem certificado de assinatura de código configurado. Antes de distribuição ampla fora de ambiente de teste, configure a assinatura no workflow e proteja as credenciais com GitHub Secrets.

## Dependências reproduzíveis

Enquanto `package-lock.json` e `Cargo.lock` não estiverem versionados, os workflows continuam usando `npm install` e resolução normal do Cargo. Esses lockfiles devem ser gerados pelas ferramentas oficiais em um ambiente com acesso aos registries e então versionados; não devem ser montados manualmente.
