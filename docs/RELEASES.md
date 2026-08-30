# Política de versões e releases

Este documento define como um estado aprovado de `main` se torna uma versão distribuível do MeliponarioManager.

## Versionamento

O projeto segue Semantic Versioning e permanece na série experimental `0.x`.

- `PATCH`: correção compatível;
- `MINOR`: funcionalidade compatível ou evolução relevante do produto;
- `MAJOR`: não usado enquanto o projeto permanecer em `0.x`.

A série `0.x` pode introduzir mudanças de formato ou comportamento entre versões. Compatibilidade de dados deve ser tratada explicitamente por migrations e validações, não presumida pelo número da versão.

## Identificadores da versão

A versão deve permanecer sincronizada em:

- `package.json`;
- `src-tauri/Cargo.toml`;
- `src-tauri/tauri.conf.json`.

`npm run version:check` valida essa consistência. Em builds disparados por tag, o mesmo script também compara a versão com o nome da tag.

## Tags

Tags distribuídas usam o formato:

```text
v0.x.y
```

A tag deve apontar para um commit já integrado em `main`. Tags públicas são imutáveis: não devem ser movidas, recriadas ou reaproveitadas depois de uma falha.

Se uma versão marcada apresentar problema compatível, prepare o próximo `PATCH`.

## GitHub Releases

Cada tag `v0.*` dispara o workflow `Build desktop bundles`. O workflow cria ou atualiza uma GitHub Release com:

- título `MeliponarioManager v0.x.y`;
- notas versionadas do repositório;
- status inicial `Draft`;
- marcação `Pre-release`;
- bundles Linux e Windows.

O draft só deve ser publicado depois da conferência das notas e dos artefatos.

## Notas de release

Cada versão precisa do arquivo:

```text
docs/releases/v0.x.y.md
```

As notas são o texto público daquela versão e devem conter, quando aplicável:

- resumo e destaques;
- mudanças de comportamento;
- compatibilidade e migração de dados;
- plataformas e artefatos;
- limitações conhecidas;
- orientação de backup e relato de problemas.

Notas de release não são dump de commits nem manual da versão atual. Depois da publicação, correções editoriais devem ser mínimas e não podem reescrever o comportamento histórico da versão.

## Changelog

`CHANGELOG.md` registra mudanças relevantes do produto. Durante o desenvolvimento, novas entradas ficam em `[Unreleased]`.

Ao preparar uma versão:

1. mova as entradas correspondentes para uma seção `[x.y.z] - AAAA-MM-DD`;
2. organize-as em categorias compatíveis com Keep a Changelog;
3. atualize os links de comparação no fim do arquivo;
4. confirme que a seção descreve o código que será marcado.

O changelog pode ser mais técnico e completo que as notas públicas.

## Preparação

Antes do merge da versão:

1. confirme a `main` de origem;
2. defina o incremento SemVer;
3. sincronize os três campos de versão;
4. execute `npm run version:check`;
5. atualize `CHANGELOG.md`;
6. crie `docs/releases/<tag>.md`;
7. atualize a documentação afetada;
8. abra um Pull Request de release;
9. valide o conjunto aplicável de testes e builds;
10. integre por squash merge.

Depois do merge:

1. confirme o commit exato em `main`;
2. crie a tag anotada `v0.x.y`;
3. envie a tag ao GitHub;
4. aguarde o workflow de bundles;
5. revise o draft da GitHub Release;
6. confira nomes, tamanhos e plataformas dos artefatos;
7. publique como Pre-release.

## Falha durante a distribuição

Se o pipeline falhar antes da publicação:

- preserve a tag;
- diagnostique o erro na execução do GitHub Actions;
- corrija em uma nova branch;
- prepare um novo `PATCH`;
- registre o incidente no changelog ou nas notas quando relevante para usuários.

Uma tag pode existir sem uma GitHub Release concluída. Isso não autoriza reescrever o histórico.

## Reprodutibilidade e assinatura

Os lockfiles npm e Cargo são versionados e exigidos pelos workflows. Atualizações de dependências devem incluir os lockfiles correspondentes.

Os instaladores Windows permanecem sem assinatura de código enquanto nenhum certificado estiver configurado. Essa limitação deve continuar explícita nas notas de cada release afetada.

Consulte [DISTRIBUTION.md](DISTRIBUTION.md) para detalhes do pipeline e dos bundles.
