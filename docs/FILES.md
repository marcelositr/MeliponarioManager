# Arquivos gerenciados

O Bloco 5C introduz armazenamento local gerenciado para documentos associados aos meliponários e consolida o tratamento de fotos de inspeção como assets do aplicativo.

O objetivo é manter os binários fora do SQLite, preservar relações e metadados no banco e permitir backup/restauração coerentes do conjunto completo.

## Princípios

- o SQLite continua sendo a fonte de verdade para identidade, vínculo e metadados;
- arquivos binários ficam no diretório de dados da aplicação;
- caminhos persistidos no banco são relativos ao diretório de dados;
- nomes internos são gerados pela aplicação e não dependem do nome original;
- o nome original é preservado apenas como metadata de apresentação;
- não existe dependência permanente do caminho de origem escolhido pelo usuário;
- anexar um arquivo significa copiar uma versão para a área gerenciada;
- abrir ou revelar um arquivo usa APIs oficiais do desktop, sem executar shell construído pelo usuário.

## Anexos de meliponário

Os anexos ficam no Record Center do próprio meliponário. Não existe uma nova seção global na sidebar.

A estrutura física é:

```text
media/attachments/meliponaries/<meliponary-id>/<uuid>.<extensão>
```

A tabela `managed_attachments`, criada pela migration aditiva `0017_managed_attachments.sql`, guarda:

- ID do anexo;
- ID do meliponário;
- nome original;
- caminho relativo gerenciado;
- extensão e tipo MIME reconhecidos quando disponíveis;
- tamanho em bytes;
- descrição e observações opcionais;
- data de inclusão.

Arquivos diferentes com o mesmo nome original podem coexistir porque o nome físico usa UUID.

## Importação

O usuário seleciona o arquivo pelo Dialog nativo do sistema.

O backend:

1. valida o meliponário;
2. verifica se a origem é um arquivo regular;
3. gera ID e nome interno seguro;
4. cria a pasta de destino quando necessário;
5. copia o binário para a área gerenciada;
6. grava os metadados no SQLite;
7. remove a cópia recém-criada caso o INSERT falhe.

O arquivo de origem permanece intacto.

## Remoção

A remoção de um anexo é explícita e confirmada pela interface.

Para reduzir estados parcialmente removidos, a cópia gerenciada é renomeada para um tombstone temporário antes da exclusão do registro. Se a transação de banco falhar, a cópia é devolvida ao caminho original. Após commit bem-sucedido, o tombstone é eliminado.

O arquivo original usado na importação nunca é removido pelo aplicativo.

## Arquivo ausente

Se um binário registrado desaparecer do disco, a metadata não é apagada automaticamente.

A interface apresenta **Arquivo não encontrado**, desabilita ações que exigem o binário e preserva o registro para diagnóstico e recuperação por backup.

## Diagnóstico

A área **Dados** possui o comando **Verificar arquivos**.

Ele cruza os registros de:

- `inspection_photos`;
- `managed_attachments`;

com as árvores físicas:

- `media/inspections/`;
- `media/attachments/`.

O resultado informa:

- quantidade de arquivos registrados;
- quantidade encontrada;
- registros cujo binário está ausente;
- arquivos físicos sem referência no SQLite.

O diagnóstico é somente leitura. Ele não apaga automaticamente metadata nem arquivos órfãos. Falhas de permissão ou de acesso ao filesystem são tratadas pelos fluxos de leitura, importação, backup e restauração com erros públicos, sem transformar diagnóstico em ferramenta de reparo destrutivo.

## Fotos de inspeção

Fotos continuam pertencendo às inspeções e permanecem em:

```text
media/inspections/<inspection-id>/
```

O 5C acrescenta:

- seleção por Dialog nativo em vez de digitação de caminho;
- ações **Abrir** e **Mostrar no local**;
- contexto humano de inspeção, data e caixa em vez de fragmentos de UUID na UI;
- thumbnails lazy na lista;
- limite de 384 KiB para bytes carregados como prévia, evitando transportar imagens maiores só para renderizar listas;
- estado explícito quando a prévia está limitada, indisponível ou o arquivo não existe.

A imagem original continua sendo aberta externamente pelo sistema quando o usuário solicita.

## Segurança de caminhos

Caminhos internos aceitos precisam ser relativos, compostos por segmentos normais e começar em `media/`.

Operações específicas também verificam o prefixo esperado, por exemplo:

```text
media/attachments/meliponaries/
media/inspections/
```

Caminhos absolutos, `..` e outros escapes da área gerenciada são rejeitados. A conversão do caminho já resolvido para a API oficial do opener é explícita e falível; caminhos que não possam ser representados com segurança não são enviados ao sistema operacional.

## Backup completo

Backups novos são conjuntos versionados com:

```text
backup-<timestamp>-<id>/
├── meliponario.db
├── manifest.json
└── media/
```

O `manifest.json` v1 registra formato, versão do formato, versão do aplicativo, versão do schema e inventário de assets com caminho relativo, tamanho e checksum SHA-256. A restauração rejeita asset ausente, tamanho divergente, conteúdo alterado mesmo com o mesmo tamanho, caminho fora de `media/` ou árvore física diferente do inventário declarado.

O backup completo inclui fotos e anexos. Consulte [DATA-MANAGEMENT.md](DATA-MANAGEMENT.md) para as regras de validação e restauração.

## Exportação JSON

O JSON portátil é estrutural e versionado. Ele preserva IDs, relações e metadados de `inspection_photos` e `managed_attachments`, mas não incorpora os bytes dos arquivos.

Por isso:

- JSON não substitui backup completo;
- não existe importador JSON destrutivo no 5C;
- recuperação completa de uma instalação continua sendo responsabilidade do backup.

## Estado da janela

O desktop usa o plugin oficial `tauri-plugin-window-state` para restaurar estado da janela entre sessões. Isso é independente dos arquivos de domínio e não altera o SQLite.

## Validação técnica e teste de campo

O fechamento técnico do 5C depende dos gates de build, lint e testes automatizados no mesmo HEAD. A validação visual/runtime real permanece separada porque exige um desktop gráfico e interação com os dialogs do sistema.

Devem ser conferidos em teste de campo:

- 900×600, 1280×720, 1366×768 e 1920×1080;
- Light, Dark e System;
- navegação por teclado;
- pickers nativos;
- thumbnails;
- Abrir e Mostrar no local;
- arquivos ausentes;
- backup → restore com restart;
- persistência de `window-state`.

## Limitações atuais

- anexos gerais existem apenas no contexto do meliponário;
- não há sincronização em nuvem;
- não há editor embutido de documentos;
- não há thumbnails para anexos genéricos;
- o diagnóstico não repara automaticamente inconsistências;
- JSON não carrega binários e não possui importador;
- a validação visual final do desktop depende de execução em ambiente gráfico real.
