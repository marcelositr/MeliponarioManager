# Política de versões e releases

Este documento define como o MeliponarioManager transforma o estado integrado de `main` em uma versão distribuível.

## Princípios

O projeto segue Semantic Versioning, mas permanece intencionalmente na série `0.x`.

Isso significa:

- `PATCH` para correções compatíveis;
- `MINOR` para novas funcionalidades compatíveis;
- ausência de meta de versão `1.0.0`;
- evolução contínua guiada por uso real e maturidade do domínio.

## Versão, tag e release

Esses conceitos são relacionados, mas diferentes.

### Versão

É a identidade técnica da aplicação e deve permanecer sincronizada em:

- `package.json`;
- `src-tauri/Cargo.toml`;
- `src-tauri/tauri.conf.json`.

### Tag

É o marcador imutável do commit distribuído.

Formato:

```text
v0.x.y
```

Exemplo:

```text
v0.7.0
```

### GitHub Release

É a página de distribuição associada à tag. Contém título, notas públicas e bundles gerados pelo pipeline.

## Por que usamos GitHub Pre-release sem sufixo `-rc`

Enquanto o MeliponarioManager estiver em fase experimental, as releases públicas são marcadas como **Pre-release** no GitHub.

A versão interna continua usando SemVer simples, como `0.7.0`, em vez de `0.7.0-rc.1`.

Essa política mantém a mesma versão entre frontend, Cargo, Tauri e instaladores de todas as plataformas. Em especial, evita criar uma segunda regra apenas para o versionamento numérico exigido pelo MSI/WiX no Windows.

O caráter experimental fica explícito em três lugares:

- a série `0.x`;
- o status do projeto na documentação;
- a marcação `Pre-release` do GitHub.

## Primeira release pública

A primeira versão preparada para distribuição é:

```text
v0.7.0
```

Os números `0.1` a `0.6` foram usados como marcos de roadmap durante o desenvolvimento inicial, mas não foram publicados como tags ou releases.

Não serão criadas releases retroativas apenas para preencher uma sequência numérica. O histórico real desse período permanece nos commits e Pull Requests.

## Notas de release

Cada versão distribuída precisa de um arquivo em:

```text
docs/releases/<tag>.md
```

Exemplo:

```text
docs/releases/v0.7.0.md
```

Esse arquivo é a fonte versionada da descrição pública da release.

As notas devem conter, quando aplicável:

- resumo da versão;
- principais funcionalidades;
- mudanças relevantes;
- plataformas e artefatos;
- limitações conhecidas;
- observações de compatibilidade ou dados;
- instruções para reportar problemas.

Evite transformar notas de release em um dump de commits.

## Preparação de uma release

Antes da tag:

1. confirmar que a branch de preparação parte da `main` atual;
2. sincronizar a versão nos três arquivos do projeto;
3. atualizar `CHANGELOG.md`;
4. criar `docs/releases/<tag>.md`;
5. atualizar README ou documentação caso a versão mude o comportamento público;
6. abrir Pull Request para `main`;
7. confirmar CI verde;
8. integrar a PR.

Depois do merge:

1. confirmar que `main` aponta para o commit esperado;
2. criar a tag `v0.x.y` nesse commit;
3. aguardar o workflow `Build desktop bundles`;
4. revisar a GitHub Release em draft;
5. validar os artefatos produzidos;
6. publicar a release como Pre-release.

## Título da release

Padrão:

```text
MeliponarioManager v0.x.y
```

Exemplo:

```text
MeliponarioManager v0.7.0
```

## Changelog

`CHANGELOG.md` registra a evolução relevante do produto.

A seção da versão deve estar pronta antes da tag. O changelog pode ser mais técnico e completo do que as notas públicas de release.

## Correções após uma release

Se `v0.7.0` revelar um bug compatível, a correção deve seguir para:

```text
v0.7.1
```

Se o próximo ciclo introduzir novas funcionalidades compatíveis:

```text
v0.8.0
```

Não é necessário reutilizar o mesmo número tentando transformar uma Pre-release anterior em uma release estável. Enquanto o projeto estiver nessa fase, cada versão pública continua sendo identificada como experimental no GitHub.

## Reprodutibilidade e assinatura

A ausência atual de lockfiles versionados e de assinatura de código Windows deve ser explicitada nas notas da release enquanto permanecer verdadeira.

Essas limitações não impedem testes públicos, mas fazem parte do estado técnico da distribuição e não devem ser escondidas.

Consulte [DISTRIBUTION.md](DISTRIBUTION.md) para detalhes do pipeline e dos bundles.
