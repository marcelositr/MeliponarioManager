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

Marque as verificações aplicáveis ao escopo da mudança.

- [ ] `npm run version:check`
- [ ] `npm run docs:check`
- [ ] `npm ci`
- [ ] `npm run icons && npm run bundle:check`
- [ ] `npm run build`
- [ ] `npm run test:ui`
- [ ] `cd src-tauri && cargo fmt --all -- --check`
- [ ] `cd src-tauri && cargo check --locked`
- [ ] `cd src-tauri && cargo clippy --locked --all-targets -- -D warnings`
- [ ] `cd src-tauri && cargo test --locked`
- [ ] `Dependency security audit` revisado quando manifests ou lockfiles mudaram
- [ ] Teste manual executado quando o comportamento alterado exige validação de interface ou runtime
- [ ] Itens não executados estão explicados abaixo

O build Tauri desktop completo e o smoke test de inicialização pertencem ao workflow `Main validation`, executado após a integração em `main`. Eles não devem ser duplicados no CI rápido de todo Pull Request apenas para produzir o status obrigatório `check`.

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
