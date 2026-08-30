# Transporte temporário

Este documento define o ciclo de `movement_type = "transport"`.

## Semântica

Um transporte temporário registra a saída transitória de uma colônia sem alterar:

- o meliponário atual;
- a caixa atual;
- a ocupação;
- o saldo do plantel;
- a situação administrativa da colônia.

Transferências internas e externas possuem regras próprias e não devem ser modeladas como transporte temporário.

## Persistência

A saída fica em `colony_movements`. Retornos ficam em `transport_returns` e sempre pertencem ao movimento original.

Cada retorno preserva:

- identificador;
- `movement_id`;
- data e hora;
- observações;
- data e motivo de eventual reabertura;
- data técnica de criação.

O movimento original e os retornos anteriores não são apagados.

## Estado derivado

### Aberto

Um transporte está aberto quando o movimento é válido e não existe retorno ativo.

Enquanto estiver aberto:

- pode receber retorno;
- pode ser anulado pelo fluxo administrativo, quando o movimento for inválido;
- impede outro transporte aberto para a mesma colônia.

### Retornado

Um transporte está retornado quando existe um retorno ativo.

Enquanto estiver retornado:

- a saída original permanece consultável;
- a data e as observações do retorno permanecem disponíveis;
- o movimento não pode ser anulado diretamente;
- o retorno precisa ser reaberto antes de outra conclusão ou anulação aplicável.

Os estados `open` e `completed` são projeções do movimento e do retorno ativo; não constituem uma segunda fonte persistida de verdade.

## Registro de retorno

A conclusão valida:

1. existência e validade do movimento;
2. tipo `transport`;
3. ausência de retorno ativo;
4. data de retorno igual ou posterior à saída;
5. inexistência de conflito operacional para a colônia.

O retorno e sua auditoria são gravados de forma transacional.

## Reabertura

Reabrir um transporte não exclui o retorno.

O retorno ativo recebe `reversed_at` e `reversal_reason`. A auditoria registra a transição de concluído para aberto. Depois disso, outro retorno pode ser registrado.

A reabertura exige motivo e não pode produzir dois transportes abertos para a mesma colônia.

## Integridade

Backend, índices e triggers protegem:

- retorno somente para movimento `transport` válido;
- retorno nunca anterior à saída;
- no máximo um retorno ativo por transporte;
- no máximo um transporte aberto por colônia;
- reabertura antes da anulação de um transporte já retornado.

Essas proteções permanecem válidas mesmo quando uma chamada futura contornar a interface.

## Auditoria

- registrar retorno: ação `complete_transport` sobre `movement`;
- reabrir retorno: ação `reopen_transport`, com motivo e estados anterior/posterior;
- anular transporte aberto: fluxo administrativo de anulação;
- reverter `internal_transfer` ou `external_transfer`: fluxo transacional de reversão de transferência.

Retorno e reversão de transferência são conceitos diferentes.

## Interface

A tela Movimentações apresenta:

- **Transporte aberto**: permite registrar retorno ou anular o transporte;
- **Retornado**: mostra a data e permite reabrir com motivo;
- **Transferência**: oferece as ações específicas de reversão quando permitidas.

A interface deriva essas ações do estado retornado pelo backend e não tenta reconstruir as regras por conta própria.

Consulte [DOMAIN.md](DOMAIN.md) para o papel das movimentações no modelo geral.
