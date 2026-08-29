# Transporte temporário

Este documento registra a decisão de domínio aplicada ao `movement_type = "transport"` no MeliponarioManager.

## Decisão

`transport` representa **transporte temporário com ciclo operacional**, e não apenas um fato histórico pontual.

A evidência já existente antes desta correção era consistente com essa interpretação:

- o backend descrevia explicitamente o tipo como **transporte temporário**;
- o transporte exigia um destino textual;
- o transporte deliberadamente não alterava o meliponário atual da colônia;
- o transporte deliberadamente não criava uma caixa de destino;
- o estado administrativo e a ocupação atual da colônia permaneciam inalterados;
- transferências internas e externas já possuíam semântica própria e consequências de estado distintas.

Portanto, a movimentação original registra a **saída temporária** e permanece preservada como fato histórico. O ciclo só é considerado concluído quando existe um retorno válido vinculado a esse movimento.

## Modelo de persistência

A migration `0016_transport_returns.sql` adiciona `transport_returns` sem modificar migrations anteriores.

Cada retorno preserva:

- identificador próprio;
- `movement_id` do transporte original;
- data e hora do retorno;
- observações do retorno;
- dados de eventual reabertura administrativa;
- data técnica de criação.

O movimento original em `colony_movements` nunca é apagado nem convertido em outro tipo.

Não há `DELETE` no fluxo de retorno.

## Estados operacionais

Um transporte temporário válido pode estar em dois estados derivados:

### Aberto

Existe um movimento válido com `movement_type = "transport"` e não existe um retorno ativo vinculado.

Enquanto estiver aberto:

- pode receber um retorno;
- pode ser anulado pelo fluxo administrativo existente, se o próprio movimento for inválido;
- impede a abertura de outro transporte temporário para a mesma colônia.

### Retornado

Existe um retorno ativo vinculado ao movimento original.

Enquanto estiver retornado:

- o movimento original continua preservado;
- a data e as observações do retorno ficam consultáveis;
- o transporte não pode ser anulado diretamente;
- uma correção que invalide o retorno deve primeiro **reabrir o transporte**.

## Reabertura do retorno

Reabrir não apaga o retorno anterior.

O registro de retorno recebe `reversed_at` e `reversal_reason`, e uma entrada de auditoria registra a mudança de `completed` para `open`.

Depois da reabertura, um novo retorno pode ser registrado. Assim, a trilha histórica preserva cada conclusão anterior em vez de reescrever o passado.

## Integridade

O backend e o SQLite protegem as seguintes regras:

- retorno somente pertence a movimento do tipo `transport` válido;
- retorno não pode ser anterior à saída;
- somente um retorno ativo pode existir por transporte;
- uma colônia não pode possuir dois transportes temporários simultaneamente abertos;
- transporte já retornado precisa ser reaberto antes de uma eventual anulação do movimento original.

O índice e os triggers de `0016_transport_returns.sql` funcionam como defesa de persistência mesmo que uma chamada futura contorne a interface.

## Auditoria

Registrar retorno cria ação de auditoria `complete_transport` para a entidade `movement`.

Reabrir retorno cria ação de auditoria `reopen_transport` com motivo obrigatório e snapshots do estado anterior e posterior.

O fluxo de reversão de transferências permanece separado:

- `internal_transfer` e `external_transfer` podem alterar plantel, situação ou ocupação e usam reversão transacional controlada;
- `transport` não altera esses estados e usa o ciclo próprio saída → retorno → eventual reabertura.

## Interface

A tela **Movimentações** apresenta o estado derivado do transporte:

- `Transporte aberto` quando aguarda retorno;
- `Retornado` com data de retorno quando concluído.

Ações disponíveis seguem o estado:

- transporte aberto: registrar retorno ou anular transporte;
- transporte retornado: reabrir transporte com motivo;
- transferências: fluxo de reversão já existente.

Essa separação evita usar “reversão” para dois conceitos de domínio diferentes.
