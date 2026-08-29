# Relatórios

A Central de Relatórios transforma os registros operacionais do MeliponarioManager em consultas por período e contexto. Ela é separada de **Dados**, que continua responsável por backup, restauração e exportação estrutural.

## Período e contexto

Os filtros globais usam:

- data inicial inclusiva;
- data final inclusiva;
- meliponário opcional.

A persistência continua em `YYYY-MM-DD HH:MM:SS`. Para filtros por data, o backend resolve o início como `00:00:00` e o final como `23:59:59`, preservando o comportamento visível inclusivo no horário local já adotado pelo projeto.

Quando existe um meliponário ativo no shell, ele é usado como contexto inicial. O usuário pode escolher **Todos os meliponários** para relatórios consolidados quando o tipo de relatório permitir.

## Regra de estado efetivo

Relatórios operacionais não tratam fatos anulados ou revertidos como válidos.

- `voided_at IS NOT NULL`: excluído de totais e listas efetivas;
- `reversed_at IS NOT NULL`: excluído do estado efetivo;
- registros corrigidos permanecem válidos com os valores atuais;
- o histórico de colônia possui um modo explícito **Histórico completo / auditoria**, no qual anulados, revertidos e registros de auditoria relacionados podem ser apresentados com estado claro.

O frontend não recalcula essas regras a partir de arrays já carregados. Consultas e agregações são feitas pelos serviços Rust de relatórios.

## Relatórios disponíveis

### Visão operacional

Combina:

- plantel atual: colônias, estados, caixas ativas e ocupações atuais;
- manejo no período: inspeções, alimentações, manutenções e eventos efetivos;
- produção agrupada por produto e unidade;
- transferências;
- transportes temporários iniciados;
- retornos concluídos;
- transportes temporários ainda abertos no fim do período;
- métricas da Agenda.

O bloco de plantel é uma fotografia atual. Os demais blocos são fatos do período. Isso evita sugerir uma reconstrução histórica de plantel que o modelo atual não armazena integralmente.

### Produção

Filtros adicionais disponíveis:

- espécie;
- colônia;
- produto.

A listagem apresenta data, meliponário, colônia, espécie, produto, quantidade, unidade e observações. As agregações são produzidas no backend por:

- produto e unidade;
- colônia;
- meliponário;
- espécie.

Unidades nunca são somadas entre si. `kg`, `g`, `L`, `mL` e outras unidades permanecem grupos independentes.

### Custos

O modelo atual possui custo persistido apenas em `box_maintenance_records.cost`. Portanto a primeira versão do relatório financeiro é deliberadamente limitada a **custos de manutenção registrados**.

Não são inferidos:

- preços de mercado;
- custos de alimentação;
- custos de produção;
- custos de movimentação;
- valores estimados.

A interface existente já apresenta o campo de manutenção em reais. O relatório mantém essa convenção como BRL, mas documenta a limitação de que o banco ainda não possui uma coluna de moeda.

Se não houver valores no período, o relatório informa que não há custos registrados em vez de apresentar um total financeiro artificialmente completo.

### Agenda

As métricas são contagens explícitas, sem percentual arbitrário:

- **Criadas**: tarefas cujo `created_at` está dentro do período;
- **Agendadas no período**: tarefas cujo `scheduled_for` está no período;
- **Concluídas**: estado final `completed`;
- **No prazo**: concluídas com `completed_at <= scheduled_for`;
- **Após o prazo**: concluídas com `completed_at > scheduled_for`;
- **Canceladas**: estado final `cancelled`;
- **Ignoradas**: estado final `skipped`;
- **Reagendadas**: estado final `rescheduled`;
- **Pendentes atrasadas**: `pending` com horário agendado anterior ao momento de geração.

Uma tarefa marcada como `rescheduled` é um registro terminal próprio. A sucessora é outro compromisso, ligado por `rescheduled_from_id`. A original não é automaticamente tratada como falha e não é contada como concluída.

### Histórico de colônia

Reúne, cronologicamente e dentro do período:

- ocupações;
- inspeções;
- alimentações;
- produção;
- eventos;
- manutenção associada;
- divisões/genealogia;
- movimentações;
- lifecycle;
- transporte temporário e retorno;
- Agenda associada;
- contagem de fotos válidas de inspeção.

A identificação da colônia mostra espécie, situação, caixa atual, origem e colônia-mãe quando existente.

No modo operacional, anulados e revertidos são removidos. No modo **Histórico completo / auditoria**, eles permanecem marcados e alterações de auditoria relacionadas aos fatos apresentados são acrescentadas ao histórico.

### Meliponário

Exige um meliponário específico. Consolida a visão operacional do período e o total de custos de manutenção registrados. Diferentemente do Dashboard, seu objetivo é período e consolidação, não somente a situação imediata.

## CSV

Há CSV para:

- Produção;
- Agenda;
- Histórico de colônia;
- Custos de manutenção.

Características:

- UTF-8;
- separador `;`, escolhido para abertura direta mais previsível em Excel/LibreOffice no locale brasileiro;
- cabeçalhos humanos e estáveis;
- ordem determinística;
- datas estruturadas de forma consistente;
- sem JSON dentro de células;
- códigos humanos usados no lugar de UUIDs como identificação principal;
- aspas, delimitadores e quebras de linha escapados conforme CSV.

### Proteção contra formula injection

Campos textuais potencialmente controlados pelo usuário que começam, depois de espaços iniciais, com:

- `=`;
- `+`;
- `-`;
- `@`;

recebem prefixo `'` antes da codificação CSV. Campos numéricos gerados pelo sistema não passam por essa proteção, portanto números negativos legítimos não são alterados.

A gravação usa o path escolhido pelo Save Dialog nativo, sem montar comandos de shell.

CSV é saída para planilha e análise humana. Ele **não substitui** backup nem JSON portável.

## Impressão

Os relatórios exibidos podem usar **Imprimir…**, que chama `window.print()` da WebView. Isso permite utilizar a impressão do sistema, inclusive a opção do sistema operacional para salvar como PDF, sem incorporar motor PDF pesado.

O CSS `@media print`:

- remove menu, sidebar, barra de contexto, status, filtros, botões e controles;
- força fundo claro e texto escuro mesmo quando o aplicativo está em dark mode;
- mantém título, período, meliponário, data de geração, totais e tabelas;
- repete cabeçalhos de tabela quando suportado pela WebView;
- evita quebra de linhas e seções quando possível.

## Estados vazios, loading e erros

Relatório sem dados é um resultado válido e recebe mensagem explícita.

Durante consultas e exportações os controles correspondentes ficam desabilitados para evitar execução duplicada. Erros passam pela camada pública já usada no produto, evitando exposição de SQL, constraints, panic, caminhos internos ou detalhes de Rust.

CSV sem linhas mantém o cabeçalho e retorna contagem zero.

## Segurança e auditoria

Consultas de relatório são somente leitura. Visualizar, imprimir ou exportar relatório não cria `audit_records`, porque são ações de consulta e não mudanças de domínio.

## Limitações atuais

- o plantel da visão operacional é uma fotografia atual, não uma reconstrução histórica completa na data final;
- custos estão limitados ao campo de manutenção já existente;
- não há conversão automática entre unidades;
- não há XLSX;
- não há gráficos obrigatórios;
- não há motor próprio de PDF;
- arquivos CSV exportados são artefatos do usuário e não entram no futuro gerenciamento de anexos do Bloco 5C.
