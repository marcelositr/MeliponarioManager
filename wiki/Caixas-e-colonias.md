# Caixas e colônias

Esta área é o coração do cadastro do plantel.

## Colônia

A colônia mantém a identidade biológica e histórica. Entre as informações cadastráveis estão código, espécie, origem, data de instalação, situação, relação com colônia-mãe e observações.

## Caixa

A caixa representa o objeto físico. Pode reunir código, modelo, material, posição ou localização, estado físico e observações.

## Ocupação

O vínculo entre uma colônia e uma caixa é uma **ocupação** com início e, quando encerrada, fim.

Isso permite preservar trocas de caixa sem apagar o passado.

### Exemplo

```text
Colônia JATAI-014

01/02 → 15/05   Caixa CX-03
15/05 → atual   Caixa CX-11
```

A colônia continua sendo `JATAI-014`. O sistema apenas registra que sua caixa mudou.

## Troca de caixa

Use o fluxo específico de troca de caixa quando uma colônia for transferida para outra caixa física.

A operação deve preservar:

- a ocupação anterior;
- a data da mudança;
- a nova ocupação;
- o histórico de manejos realizados em cada período.

Não crie uma nova colônia para representar uma simples troca de caixa.

## Divisão de colônia

Uma divisão ou multiplicação é diferente de uma troca de caixa.

Quando uma divisão gera uma descendente, o sistema pode criar a nova colônia mantendo a relação genealógica com a colônia-mãe. Tentativas sem sucesso não devem criar colônias inexistentes.

## Situação da colônia

Mudanças importantes, como perda, inativação e reativação, possuem fluxos específicos para preservar o ciclo de vida.

A condição observada durante o manejo pode ser registrada em inspeções sem necessariamente significar uma mudança administrativa permanente da colônia.

## Estado físico da caixa

Uma caixa pode ficar indisponível para nova ocupação durante manutenção ou após sua retirada de uso. Uma caixa ocupada precisa ter sua ocupação resolvida pelo fluxo apropriado antes de determinadas mudanças de estado.

## Histórico

Para descobrir o que aconteceu com uma colônia ao longo do tempo, use sua ficha ou o [Histórico de colônia nos Relatórios](Relatorios).

---

[← Meliponários](Meliponarios) · [Próximo: Manejo →](Manejo)