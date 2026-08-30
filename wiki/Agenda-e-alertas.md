# Agenda e alertas

A Agenda organiza **o que precisa ser feito**. Os registros de manejo documentam **o que realmente aconteceu**.

## Como funciona

Um fluxo típico é:

```text
Inspeção realizada
      ↓
Próxima inspeção informada
      ↓
Tarefa aparece na Agenda
      ↓
Usuário executa a tarefa
      ↓
Nova inspeção é registrada
      ↓
Tarefa anterior é concluída
```

A aplicação evita marcar um manejo especializado como concluído sem criar o registro correspondente.

## Tipos de tarefa

A Agenda pode trabalhar com compromissos de:

- inspeção;
- alimentação;
- manutenção;
- tarefa genérica.

## Estados

Uma tarefa pode estar:

- pendente;
- concluída;
- cancelada;
- reagendada;
- ignorada.

Uma tarefa pendente cuja data já passou é apresentada como atrasada. O atraso é calculado a partir da data, não é um segundo registro separado.

## Prioridade

As tarefas podem possuir níveis de prioridade para ajudar na ordenação do trabalho. Prioridade não altera as regras históricas ou os dados da colônia.

## Reagendar

Ao reagendar, a tarefa anterior é preservada e uma sucessora representa o novo compromisso. Isso mantém o histórico da decisão.

## Cancelar ou ignorar

Essas ações exigem motivo e preservam a tarefa original com seu estado final.

## Tarefas derivadas

Algumas tarefas são criadas a partir das próximas datas informadas nos manejos. A aplicação mantém a relação entre o fato que originou o planejamento e a tarefa correspondente.

## Alertas

Os alertas ajudam a chamar atenção para compromissos de inspeção, alimentação e manutenção. Eles podem direcionar você para a Agenda ou para o fluxo de manejo correspondente.

## Dashboard

O Dashboard pode resumir tarefas atrasadas, de hoje, dos próximos dias e futuras dentro do contexto ativo.

## Limitações atuais

A Agenda é local. Atualmente ela não deve ser confundida com:

- calendário externo;
- notificações push em nuvem;
- automação remota;
- sincronização online.

---

[← Manejo](Manejo) · [Próximo: Movimentações e transporte →](Movimentacoes-e-transporte)