# Backup e restauração

O MeliponarioManager mantém seus dados localmente. Um backup atualizado é a principal proteção contra perda por falha de disco, exclusão acidental ou problema na instalação.

## O que entra no backup completo

Um backup completo reúne:

- banco de dados SQLite;
- fotos de inspeção;
- anexos gerenciados;
- manifesto com informações de integridade.

A estrutura atual é semelhante a:

```text
backup-<data>-<id>/
├── meliponario.db
├── manifest.json
└── media/
```

## Criando um backup

Na área **Dados**, utilize a função de backup completo. O MeliponarioManager cria automaticamente o backup na área de dados da aplicação e, ao concluir, informa o caminho onde a cópia foi armazenada.

Recomendações:

- mantenha pelo menos uma cópia fora do disco principal;
- não guarde todas as cópias dentro da mesma pasta da aplicação;
- faça backup antes de atualizar;
- faça backup antes de uma restauração importante;
- teste periodicamente se você consegue localizar e selecionar suas cópias.

## O que o sistema valida

Backups atuais possuem informações que permitem conferir itens como:

- integridade do SQLite;
- versão do formato;
- versão do schema;
- arquivos esperados;
- tamanho dos arquivos;
- hash SHA-256 dos arquivos de mídia.

A validação existe para evitar substituir seus dados ativos por um conjunto incompleto ou corrompido.

## Restaurando

A restauração é preparada primeiro e aplicada de maneira controlada.

Em linhas gerais:

1. você seleciona o backup;
2. a aplicação valida o conjunto;
3. prepara os arquivos para restauração;
4. informa quando é necessário reiniciar;
5. na próxima abertura, o estado atual recebe uma cópia de segurança;
6. o conjunto restaurado é promovido;
7. se a troca falhar, a aplicação procura recuperar o estado anterior.

Não interrompa deliberadamente o processo durante a aplicação da restauração.

## JSON não é backup

A exportação JSON serve para interoperabilidade e inspeção estrutural.

Ela contém registros e metadados, mas **não incorpora as fotos e anexos**. Portanto não substitui o backup completo.

## CSV também não é backup

CSV é uma saída para planilhas e análise humana. Ele não contém o estado completo da aplicação.

## Diagnóstico de arquivos

Use **Verificar arquivos** periodicamente para identificar arquivos registrados que estejam ausentes e arquivos físicos sem referência no banco.

## Estratégia recomendada

Para dados importantes, mantenha pelo menos:

- uma cópia recente no computador;
- uma cópia em outro dispositivo ou mídia;
- uma rotina periódica de backup.

---

[← Relatórios](Relatorios) · [Próximo: Solução de problemas →](Solucao-de-problemas)