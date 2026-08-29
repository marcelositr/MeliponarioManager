# Gerenciamento e segurança dos dados

O MeliponarioManager é uma aplicação local-first. Os dados principais permanecem no computador do usuário, no diretório de dados da aplicação.

Este documento descreve os recursos atuais de backup, exportação estrutural, relatórios e restauração.

## Onde os dados vivem

A aplicação mantém:

- banco SQLite com os dados estruturados;
- arquivos de mídia gerenciados no diretório local da aplicação;
- metadados de mídia no SQLite, sem armazenar arquivos binários como BLOB.

Fotos de inspeção são organizadas abaixo de:

```text
media/inspections/<inspection-id>/
```

## Backup completo

O backup completo é destinado a preservar o estado necessário para recuperar uma instalação.

Ele inclui:

- banco SQLite;
- arquivos de mídia gerenciados pela aplicação.

O backup deve ser tratado como uma cópia de segurança do estado do aplicativo, não como substituto de um histórico externo de versões ou de uma política pessoal de backup do computador.

## Exportação portátil em JSON

A exportação JSON oferece uma representação portátil dos dados do MeliponarioManager para inspeção, intercâmbio futuro e preservação independente do arquivo SQLite.

A exportação não substitui automaticamente um backup completo, porque arquivos de mídia e detalhes de restauração possuem necessidades próprias.

## Relatórios e CSV

A Central de Relatórios é uma ferramenta de consulta operacional e gerencial separada da área **Dados**.

Os conceitos possuem finalidades diferentes:

- **Backup completo** preserva o estado necessário para recuperação do sistema;
- **JSON portável** preserva a estrutura dos dados para interoperabilidade e inspeção;
- **CSV de relatório** apresenta conjuntos tabulares derivados para análise humana e planilhas;
- **Relatório imprimível** apresenta uma visão legível que pode ser impressa em papel ou salva como PDF pela impressão do sistema operacional.

CSV e impressão não são mecanismos de backup nem novas fontes de verdade. Eles são derivados somente dos registros persistidos e não alteram o domínio.

Os CSVs de relatórios usam Save Dialog nativo e ficam no destino escolhido pelo usuário. Esses arquivos são artefatos exportados e não passam a fazer parte automaticamente do armazenamento gerenciado da aplicação.

Consulte [REPORTS.md](REPORTS.md) para filtros, semântica, proteção contra formula injection e limitações dos relatórios.

## Relatório gerencial legado em Markdown

O backend ainda contém a saída textual gerencial criada na série anterior para compatibilidade. A interface principal passa a direcionar consultas operacionais para a Central de Relatórios dedicada.

Essa saída legada também é derivada e não constitui fonte de verdade.

## Restauração

A restauração é tratada como uma operação sensível.

O fluxo atual evita trocar o banco em uso no meio da execução:

1. o conjunto a restaurar é preparado;
2. a integridade é validada;
3. a restauração fica agendada;
4. a aplicação aplica a troca na próxima inicialização;
5. antes da substituição, o estado atual recebe um backup de segurança automático.

Essa estratégia reduz o risco de corrupção por substituição de arquivos enquanto conexões com o banco ainda estão abertas.

## Integridade

A restauração não deve ser aplicada cegamente a um arquivo arbitrário.

A implementação valida o material preparado antes da troca e preserva uma cópia de segurança do estado atual. Mesmo assim, versões experimentais devem ser testadas com dados não críticos antes de uso amplo.

## O que não acontece

- o MeliponarioManager não envia automaticamente o banco para um serviço em nuvem;
- exportações e relatórios não alteram o estado do plantel;
- a restauração não deve sobrescrever silenciosamente o estado atual sem criar a cópia de segurança prevista;
- arquivos de mídia não são convertidos em BLOB no SQLite;
- CSV exportado não é tratado como anexo gerenciado.

## Recomendações para testes

Durante a fase experimental:

- mantenha backups externos periódicos do diretório de dados;
- teste restaurações usando uma cópia de trabalho antes de depender do fluxo em dados únicos;
- preserve exportações importantes fora do diretório principal da aplicação;
- ao reportar falhas de backup ou restauração, informe versão do aplicativo, sistema operacional e sequência exata de passos, evitando anexar dados pessoais ou sensíveis sem necessidade.

## Evolução futura

Compatibilidade de backup entre versões, validações adicionais e maior reprodutibilidade de migrações são áreas previstas para amadurecer conforme o aplicativo for usado em cenários reais. O gerenciamento amplo de anexos e arquivos pertence ao Bloco 5C e não é antecipado pelas exportações CSV do 5B.