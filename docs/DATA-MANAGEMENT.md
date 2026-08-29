# Gerenciamento e segurança dos dados

O MeliponarioManager é uma aplicação local-first. Os dados principais permanecem no computador do usuário, no diretório de dados da aplicação.

Este documento descreve backup, restauração e exportação estrutural. A arquitetura de fotos e anexos é detalhada em [FILES.md](FILES.md).

## Onde os dados vivem

A aplicação mantém:

- banco SQLite com os dados estruturados e metadados;
- arquivos gerenciados fora do banco, abaixo de `media/`;
- preferências de janela tratadas pelo desktop, independentes do domínio.

Os binários de fotos e anexos não são armazenados como BLOB no SQLite.

Estruturas atuais:

```text
media/inspections/<inspection-id>/
media/attachments/meliponaries/<meliponary-id>/
```

Os caminhos persistidos no banco são relativos ao diretório de dados.

## Backup completo

O backup completo é o mecanismo de recuperação integral da instalação.

Backups novos usam um diretório próprio:

```text
backup-<timestamp>-<id>/
├── meliponario.db
├── manifest.json
└── media/
```

O banco é criado por uma cópia consistente do SQLite. A árvore `media/` inclui fotos de inspeção e anexos gerenciados.

### Manifest v1

`manifest.json` identifica explicitamente o formato do backup e registra:

- `format`;
- `formatVersion`;
- data de criação;
- versão do aplicativo;
- versão do schema;
- nome do banco;
- raiz da mídia;
- inventário dos assets com caminho relativo e tamanho em bytes.

O manifest torna possível distinguir um backup completo atual de um diretório arbitrário ou de formatos legados.

## Validação da restauração

Nenhum material é promovido diretamente para o estado ativo.

Antes do staging, a aplicação valida:

1. se o SQLite pode ser aberto;
2. `PRAGMA integrity_check`;
3. presença de tabelas essenciais do domínio;
4. versão registrada em `_sqlx_migrations`;
5. faixa de schema reconhecida pela aplicação;
6. quando existe manifest, formato e versão do manifest;
7. correspondência entre schema declarado e banco;
8. caminhos relativos e confinados à pasta `media/`;
9. presença e tamanho de cada asset declarado;
10. equivalência entre o inventário declarado e a árvore física de mídia.

Links simbólicos não são aceitos na árvore copiada para backup/restauração.

Backups legados podem ser aceitos quando o banco possui schema reconhecido e migrável. Eles são identificados como legados e não recebem a mesma garantia estrutural de um backup completo com manifest.

## Aplicação segura da restauração

A restauração não troca o banco enquanto a aplicação ainda está usando suas conexões.

Fluxo:

1. o conjunto é validado;
2. banco e mídia são copiados para staging;
3. a aplicação informa que a restauração será aplicada na próxima abertura;
4. no startup, o estado atual recebe uma cópia de segurança em `backups/pre-restore-<timestamp>/`;
5. banco e mídia atuais são movidos para áreas de rollback;
6. o conjunto staged é promovido;
7. se uma etapa de troca falhar, o estado anterior é restaurado;
8. somente após sucesso são removidos rollback e artefatos temporários.

Arquivos `-wal` e `-shm` remanescentes são removidos após a troca concluída.

## Exportação portátil em JSON

O 5C adota uma exportação **estrutural, versionada e abrangente**.

O JSON inclui:

- identificação de formato e versão;
- versão do aplicativo e schema;
- IDs e relações persistidos;
- tabelas de domínio e auditoria;
- metadata de `inspection_photos`;
- metadata de `managed_attachments`.

Os bytes de fotos e anexos **não são incorporados**. O documento declara `assetsEmbedded: false`.

Não existe importador JSON destrutivo no 5C. A recuperação integral permanece responsabilidade do backup completo. Essa separação evita prometer round-trip onde os binários não estão presentes.

## Relatórios e CSV

A Central de Relatórios é uma ferramenta de consulta operacional e gerencial separada da área **Dados**.

As finalidades são diferentes:

- **Backup completo**: recuperação do estado da instalação;
- **JSON portátil**: interoperabilidade e inspeção estrutural;
- **CSV de relatório**: análise humana e planilhas;
- **Relatório imprimível**: leitura, impressão ou PDF produzido pelo sistema operacional.

CSV e impressão não são mecanismos de backup nem novas fontes de verdade. Eles são derivados somente dos registros persistidos.

Os CSVs usam Save Dialog nativo e ficam no destino escolhido pelo usuário. Eles não passam automaticamente a integrar o armazenamento gerenciado.

Consulte [REPORTS.md](REPORTS.md).

## Diagnóstico de arquivos

A área **Dados** possui uma verificação explícita de consistência entre SQLite e filesystem.

Ela examina fotos e anexos e informa:

- quantidade registrada;
- quantidade fisicamente encontrada;
- registros cujo arquivo está ausente;
- arquivos físicos sob as áreas gerenciadas que não possuem referência no banco.

O diagnóstico é somente leitura. Nenhum registro ou arquivo órfão é apagado automaticamente.

## Arquivo ausente

Quando um asset registrado não é encontrado no disco:

- a metadata permanece no SQLite;
- a UI informa **Arquivo não encontrado**;
- ações que dependem do binário são desabilitadas;
- o usuário pode usar diagnóstico ou backup para investigar/recuperar o estado.

A ausência física não é tratada como autorização implícita para apagar histórico.

## Relatório gerencial legado em Markdown

O backend ainda contém a saída textual gerencial criada na série anterior para compatibilidade. A interface principal direciona consultas operacionais para a Central de Relatórios dedicada.

Essa saída é derivada e não constitui fonte de verdade.

## O que não acontece

- o MeliponarioManager não envia automaticamente o banco ou arquivos para nuvem;
- arquivos de mídia não viram BLOB no SQLite;
- JSON, CSV e impressão não alteram o estado do domínio;
- JSON não é apresentado como backup completo;
- não existe importação JSON destrutiva no 5C;
- restauração não troca silenciosamente o estado ativo sem validação e cópia de segurança;
- diagnóstico não apaga automaticamente arquivos ou registros;
- CSV exportado não é tratado como anexo gerenciado.

## Recomendações durante a fase experimental

- mantenha também backups externos do diretório de dados;
- valide restauração com dados de teste antes de depender de uma única cópia;
- preserve backups importantes em mídia diferente do disco principal;
- execute periodicamente o diagnóstico de arquivos;
- ao reportar falhas, registre versão do aplicativo, sistema operacional e sequência de passos sem expor dados pessoais desnecessários.

## Compatibilidade

O formato de backup e o formato JSON possuem versionamento próprio. Mudanças futuras que alterem seus contratos devem elevar a respectiva versão de formato e manter validação explícita de compatibilidade.

A versão do produto permanece independente desses números de formato.
