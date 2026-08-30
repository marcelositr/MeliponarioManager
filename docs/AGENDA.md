# Agenda operacional

A Agenda organiza compromissos futuros sem confundir planejamento com fatos históricos de manejo.

## Conceito

Uma tarefa informa o que precisa ser feito. Inspeções, alimentações e manutenções registram o que realmente aconteceu.

```text
fato com próxima data sugerida
        ↓
tarefa pendente
        ↓
execução da tarefa
        ↓
novo fato registrado
        ↓
tarefa concluída
```

Uma tarefa especializada só é concluída na mesma operação que cria o fato correspondente. A Agenda não pode marcar um manejo como realizado sem o registro factual válido.

## Tipos de tarefa

| Tipo | Finalidade | Forma de conclusão |
| --- | --- | --- |
| `inspection` | Inspeção de colônia | Registra uma inspeção |
| `feeding` | Alimentação ou suplementação | Registra uma alimentação |
| `maintenance` | Manutenção de caixa | Registra uma manutenção |
| `generic` | Compromisso operacional geral | Conclui a própria tarefa |

A prioridade pode ser `normal`, `attention` ou `critical`. Prioridade influencia a ordenação, mas não altera regras de data ou de domínio.

## Estados

| Estado | Significado |
| --- | --- |
| `pending` | Compromisso aberto |
| `completed` | Concluído por uma execução válida |
| `cancelled` | Cancelado com motivo |
| `rescheduled` | Substituído por outro compromisso |
| `skipped` | Ignorado deliberadamente com motivo |

Atraso é derivado: uma tarefa `pending` está atrasada quando `scheduled_for` é anterior ao horário atual. Não existe estado persistido `overdue`.

## Origem

Tarefas podem ser:

- manuais, criadas diretamente pelo usuário;
- derivadas de `next_inspection_at`;
- derivadas de `next_feeding_at`;
- derivadas de `next_maintenance_at`.

Uma tarefa derivada preserva a relação com o fato de origem. Essa linhagem permite atualizar o planejamento quando o fato é corrigido ou anulado.

## Reconciliação

Na inicialização, o backend executa `reconcile_all()`.

A reconciliação recria uma tarefa derivada ausente quando:

- o fato de origem continua válido;
- existe um `next_*` aplicável;
- nenhuma tarefa correspondente já representa o compromisso.

O processo é idempotente. Executá-lo várias vezes produz no máximo uma tarefa pendente para a mesma origem e finalidade.

Fatos anulados ou revertidos não podem sustentar uma tarefa derivada válida.

## Operações

### Reagendamento

Reagendar encerra a tarefa original como `rescheduled` e cria uma sucessora ligada por `rescheduled_from_id`. A data anterior e a decisão permanecem consultáveis.

### Cancelamento e tarefa ignorada

Cancelar ou ignorar exige motivo. A tarefa original permanece armazenada com o estado correspondente.

### Duplicação

Duplicar cria outro compromisso a partir de uma tarefa existente sem alterar a original. Essa operação é adequada para repetição intencional que não possui um campo factual `next_*`.

### Execução especializada

A execução de inspeção, alimentação ou manutenção:

1. valida a tarefa e seu contexto;
2. aplica as regras do fluxo factual correspondente;
3. cria o fato;
4. cria eventual compromisso derivado da próxima data;
5. conclui a tarefa original;
6. confirma tudo na mesma transação.

Se qualquer etapa falhar, a tarefa permanece aberta e nenhum fato parcial é promovido.

### Execução genérica

Uma tarefa `generic` pode ser concluída diretamente. Ela registra apenas o cumprimento do compromisso e não cria uma entidade de manejo.

## Contexto de meliponário

O meliponário ativo no shell filtra:

- lista e resumo da Agenda;
- opções de colônia e caixa;
- alertas e atalhos relacionados;
- fichas operacionais que exibem tarefas.

Esse contexto é um filtro de leitura e operação. Ele não altera a propriedade de colônias, caixas ou tarefas.

## Integração com alertas e Dashboard

Alertas de inspeção, alimentação e manutenção vencidas usam tarefas pendentes como fonte operacional. Isso evita representar o mesmo vencimento por uma tarefa e por um segundo cálculo independente.

Alertas podem carregar `task_id`, colônia, caixa, meliponário e ação recomendada. A interface usa esse contexto para abrir a Agenda ou o fluxo de manejo correto.

O Dashboard resume tarefas atrasadas, de hoje, dos próximos sete dias e futuras no contexto ativo.

## Fichas operacionais

As fichas de Colônia, Caixa e Meliponário consultam projeções do backend para exibir tarefas e alertas relacionados. Elas não copiam esses registros para tabelas próprias.

## Integridade

- planejamento e fato histórico são conceitos distintos;
- conclusão especializada exige criação factual válida;
- transições preservam a tarefa original;
- tarefas derivadas mantêm vínculo com a origem;
- reconciliação não cria duplicatas;
- fatos anulados ou revertidos deixam de produzir planejamento válido;
- projeções da Agenda não substituem as entidades de domínio.

## Limites atuais

A Agenda é local e integrada ao SQLite. Não há:

- sincronização com calendário externo;
- notificações push;
- automação remota;
- serviço em nuvem.

Esses recursos não devem ser presumidos por outros documentos ou pela interface.
