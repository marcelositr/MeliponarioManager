## Resumo

Descreva objetivamente a mudança.

## Motivação

Explique o problema ou a necessidade atendida.

## Solução

Registre as decisões relevantes e os limites desta alteração.

## Impacto

- Domínio:
- Schema ou migrations:
- Dados existentes:
- Interface:
- Compatibilidade:
- Riscos conhecidos:

Use `Não se aplica` quando necessário.

## Validação

- [ ] `npm run version:check`
- [ ] `npm ci`
- [ ] `npm run icons && npm run bundle:check`
- [ ] `npm run build`
- [ ] `npm run test:ui`
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo check --locked`
- [ ] `cargo clippy --locked --all-targets -- -D warnings`
- [ ] `cargo test --locked`
- [ ] Build Tauri com `--no-bundle`
- [ ] Teste manual
- [ ] Itens não executados estão explicados abaixo

### Observações da validação

Informe ambiente, comandos omitidos e motivo de qualquer falha conhecida.

## Documentação e release

- [ ] Documentação temática atualizada
- [ ] `docs/ARCHITECTURE.md` atualizado quando a estrutura mudou
- [ ] `CHANGELOG.md` atualizado quando relevante
- [ ] Notas e versões atualizadas quando esta é uma preparação de release
- [ ] Não há impacto documental ou de release

## Checklist final

- [ ] O escopo é único e claro
- [ ] O histórico não é removido indevidamente
- [ ] Operações relacionadas permanecem transacionais
- [ ] Migrations já integradas não foram alteradas
- [ ] Erros públicos não expõem detalhes internos ou dados pessoais
- [ ] A branch está atualizada com `main`
- [ ] O status check obrigatório está aprovado
