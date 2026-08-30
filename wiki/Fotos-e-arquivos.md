# Fotos e arquivos

O MeliponarioManager mantém fotos de inspeção e anexos em uma área local gerenciada pela aplicação.

## O que acontece ao anexar um arquivo

Quando você seleciona um arquivo compatível:

1. a aplicação lê o arquivo original;
2. cria uma cópia na área gerenciada;
3. registra seus metadados e vínculo no banco;
4. mantém o arquivo original no local onde estava.

Depois da importação, a aplicação trabalha com a cópia gerenciada. Mover o arquivo original não deve quebrar o anexo já importado.

## Fotos de inspeção

Fotos ficam associadas a uma inspeção existente.

A interface pode oferecer:

- prévia;
- abrir o arquivo;
- mostrar o arquivo no local;
- contexto da inspeção, data e caixa correspondente.

Formatos de imagem suportados inicialmente incluem JPG, PNG e WebP.

## Anexos do meliponário

Documentos administrativos podem ser associados ao próprio meliponário. Eles complementam os dados do sistema, mas não substituem registros de manejo ou movimentação.

## Arquivo não encontrado

Se uma cópia gerenciada desaparecer do disco, a aplicação preserva os metadados históricos e informa que o arquivo não foi encontrado.

Ela não deve apagar automaticamente o registro só porque o binário sumiu.

## Verificar arquivos

Na área **Dados**, o comando de verificação cruza registros do banco com os arquivos físicos e pode informar:

- quantidade registrada;
- quantidade encontrada;
- registros cujo arquivo está ausente;
- arquivos físicos sem referência no banco.

Essa verificação é de diagnóstico. Ela não remove automaticamente registros ou arquivos órfãos.

## Backup

Fotos e anexos fazem parte do [backup completo](Backup-e-restauracao).

A exportação JSON preserva metadados, mas **não incorpora os bytes desses arquivos**. Por isso JSON não substitui backup.

---

[← Movimentações](Movimentacoes-e-transporte) · [Próximo: Relatórios →](Relatorios)