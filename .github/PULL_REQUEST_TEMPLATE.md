## O que mudou

Descreva objetivamente a alteração.

## Por que

Explique o problema ou necessidade atendida.

## Como foi validado

- [ ] Build frontend
- [ ] `cargo fmt --check`
- [ ] `cargo check`
- [ ] `cargo clippy -- -D warnings`
- [ ] `cargo test`
- [ ] Build Tauri sem bundle
- [ ] Teste manual
- [ ] Não aplicável

## Impacto no domínio

- [ ] Não altera o modelo de domínio
- [ ] Altera o modelo de domínio e `docs/DOMAIN.md` foi atualizado

## Impacto em versão ou release

- [ ] Não exige atualização de changelog
- [ ] `CHANGELOG.md` foi atualizado
- [ ] É preparação de release e inclui `docs/releases/<tag>.md`
- [ ] Versões em `package.json`, `Cargo.toml` e `tauri.conf.json` estão sincronizadas

## Checklist

- [ ] A alteração tem escopo claro
- [ ] Não remove histórico ou rastreabilidade indevidamente
- [ ] Documentação foi atualizada quando necessário
- [ ] O CI esperado está verde ou a falha está explicada
