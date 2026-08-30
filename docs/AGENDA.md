# Agenda operacional

A Agenda do MeliponarioManager organiza **compromissos futuros** sem transformar planejamento em fato histórico.

Ela faz parte da Etapa 4 da série experimental `0.x` e trabalha em conjunto com inspeções, alimentação, manutenção, alertas, Dashboard e fichas operacionais.

## Regra principal

Uma tarefa diz **o que precisa ser feito**. Um registro de inspeção, alimentação, manutenção ou outro fato diz **o que realmente aconteceu**.

Esses conceitos não são intercambiáveis.

Exemplo:

```text
próxima inspeção sugerida
        ↓
tarefa pendente na Agenda
        ↓
execução da tarefa
        ↓
inspeção factual registrada
        ↓
tarefa concluída
```

A conclusão de uma tarefa especializada só ocorre junto do fato correspondente. O sistema não marca uma inspeção, alimentação ou manutenção como realizada sem criar o registro factual válido.

## Estados de tarefa

As tarefas possuem estados operacionais explícitos:

- `pending`: compromisso ainda aberto;
- `completed`: concluído por uma execução válida;
- `cancelled`: cancelado com motivo, preservando o registro original;
- `rescheduled`: substituído por um novo compromisso em outra data;
- `skipped`: deliberadamente ignorado com motivo.

Atraso não é um estado persistido separado. Uma tarefa pendente está atrasada quando `scheduled_for` é anterior à referência temporal atual.

## Tipos

A Agenda trabalha inicialmente com:

- `inspection`: inspeção;
- `feeding`: alimentação;
- `maintenance`: manutenção de caixa;
- `generic`: compromisso operacional que não cria automaticamente um fato especializado.

Tarefas genéricas podem ser concluídas diretamente. Tarefas de inspeção, alimentação e manutenção usam os fluxos de execução correspondentes.

## Prioridades

Prioridades disponíveis:

- `normal`;
- `attention`;
- `critical`.

A prioridade ajuda a ordenar o trabalho, mas não substitui data, contexto ou regras de domínio.

## Tarefas manuais e tarefas derivadas

Uma tarefa pode ser criada manualmente pelo usuário ou derivada de um campo `next_*` de um fato válido.

Fontes derivadas atuais:

- `next_inspection_at` de inspeções;
- `next_feeding_at` de alimentações;
- `next_maintenance_at` de manutenções.

A tarefa derivada mantém linhagem com o fato de origem. Essa linhagem permite que uma correção real do fato continue tendo autoridade sobre o planejamento derivado.

Um simples reagendamento operacional não apaga essa origem.

## Reconciliação

Na inicialização, o backend executa uma reconciliação idempotente da Agenda.

A reconciliação pode reconstruir uma tarefa derivada ausente quando o fato de origem continua válido e possui um `next_*` aplicável. Executar a reconciliação repetidamente não deve criar duplicatas.

Propriedade esperada:

```text
fato válido com next_*
        ↓
tarefa derivada ausente
        ↓
reconcile_all()
reconcile_all()
        ↓
exatamente uma tarefa pendente
```

Registros anulados não devem continuar produzindo compromissos operacionais válidos.

## Reagendamento

Reagendar preserva a tarefa original e cria a continuidade do compromisso.

O objetivo é manter a cadeia de decisão visível em vez de sobrescrever silenciosamente a data anterior.

A tarefa reagendada preserva a relação com a linhagem operacional necessária para que o fato de origem continue rastreável.

## Cancelar e ignorar

Cancelar e ignorar não apagam a tarefa.

Ambos exigem motivo e preservam o registro original para que o histórico mostre por que o compromisso deixou de estar pendente.

## Duplicar

Duplicar cria um novo compromisso a partir de outro, sem alterar o registro original.

É útil para repetir uma atividade planejada quando não existe um campo factual `next_*` apropriado para representar essa intenção.

## Execução

### Inspeção

Executar uma tarefa de inspeção cria uma inspeção factual usando as mesmas regras de integridade do fluxo normal. A tarefa só é concluída junto dessa operação válida.

### Alimentação

Executar uma tarefa de alimentação cria o registro factual de alimentação e permite informar a próxima alimentação, que pode gerar o compromisso derivado seguinte.

### Manutenção

Executar uma tarefa de manutenção registra a manutenção da caixa e pode definir a próxima manutenção.

### Tarefa geral

Uma tarefa `generic` pode ser concluída sem criar uma entidade factual especializada. Ela registra apenas a conclusão do próprio compromisso.

## Contexto de meliponário

O seletor de meliponário do shell aplica escopo operacional à Agenda.

- `Todos os meliponários`: visão consolidada;
- um meliponário selecionado: tarefas, resumo e opções de colônia/caixa ficam limitados àquela unidade.

Esse contexto é **filtro de interface**, não movimentação de domínio. Selecionar outro meliponário não altera vínculo de colônia, caixa ou tarefa.

## Alertas

Alertas de vencimento de inspeção, alimentação e manutenção usam as tarefas pendentes da Agenda como fonte operacional.

Isso evita manter, em paralelo, uma tarefa vencida e um segundo alerta de `next_*` calculado de forma independente para o mesmo compromisso.

A central de alertas pode apontar para:

- a Agenda, quando existe `task_id`;
- o fluxo recomendado de manejo, conforme `recommended_action`;
- ambos, quando o usuário precisa decidir entre revisar o compromisso e executar a ação factual.

## Dashboard

O Dashboard mostra o resumo da Agenda no contexto ativo:

- atrasadas;
- hoje;
- próximos sete dias;
- futuras.

Alertas exibidos no Dashboard preservam ações contextuais e não criam estados próprios.

## Fichas operacionais

As fichas de Colônia, Caixa e Meliponário utilizam projeções dedicadas do backend.

Elas podem apresentar tarefas e alertas relacionados, mas não copiam esses dados para tabelas próprias.

A ficha funciona como um **hub de leitura e navegação** para os fluxos especializados.

## Relação com fatos corrigidos ou anulados

A Etapa 4 preserva a política da Etapa 3:

- correção altera o fato válido com auditoria;
- anulação mantém o registro no histórico, mas o exclui das projeções operacionais;
- reversões seguras preservam o registro original;
- a Agenda deve refletir apenas fatos operacionais válidos para derivação futura.

## Fronteira da Etapa 4

A Agenda não é calendário externo, sincronização em nuvem, notificação push ou automação remota.

Nesta etapa ela é um planejador operacional local, integrado ao SQLite e aos fluxos Tauri existentes.

Funcionalidades posteriores pertencem a etapas futuras e não devem ser antecipadas dentro desta implementação.
