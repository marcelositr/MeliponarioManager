# Gerenciamento e segurança dos dados

O MeliponarioManager é uma aplicação local-first. Os dados principais permanecem no computador do usuário, no diretório de dados da aplicação.

Este documento descreve os recursos atuais de backup, exportação, relatório e restauração.

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

## Relatório gerencial em Markdown

O relatório Markdown fornece uma visão textual legível do estado do meliponário.

Seu objetivo é facilitar consulta, arquivamento e compartilhamento de informações sem exigir acesso direto ao SQLite.

Ele é um relatório derivado e não uma nova fonte de verdade para o sistema.

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
- arquivos de mídia não são convertidos em BLOB no SQLite.

## Recomendações para testes

Durante a fase experimental:

- mantenha backups externos periódicos do diretório de dados;
- teste restaurações usando uma cópia de trabalho antes de depender do fluxo em dados únicos;
- preserve exportações importantes fora do diretório principal da aplicação;
- ao reportar falhas de backup ou restauração, informe versão do aplicativo, sistema operacional e sequência exata de passos, evitando anexar dados pessoais ou sensíveis sem necessidade.

## Evolução futura

Compatibilidade de backup entre versões, validações adicionais e maior reprodutibilidade de migrações são áreas previstas para amadurecer conforme o aplicativo for usado em cenários reais.
