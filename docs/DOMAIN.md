# Modelo de domínio

Este documento registra os conceitos centrais do MeliponarioManager e as decisões que orientam a implementação.

## Princípio central

A **colônia** é uma entidade biológica e histórica. A **caixa** é um objeto físico que pode abrigá-la durante um intervalo de tempo.

Esses conceitos não são intercambiáveis. Trocas de caixa, divisões, movimentações, perdas, inspeções, manutenções e demais fatos relevantes devem preservar o passado em vez de reescrevê-lo.

## Entidades principais

### Meliponário

Unidade de criação onde colônias e caixas são mantidas. O sistema suporta múltiplos meliponários.

Dados principais incluem nome, responsável, localização e observações.

### Espécie

Representa a espécie de abelha sem ferrão associada às colônias.

Dados principais incluem nome popular, nome científico, gênero e observações.

### Colônia

Entidade biológica e histórica que mantém identidade própria mesmo quando muda de caixa ou de localização.

Dados principais incluem código, espécie, meliponário, origem, data de instalação, situação, colônia-mãe e observações.

A entrada no plantel é derivada do próprio cadastro e da origem registrada. Não existe um segundo lançamento de entrada independente apenas para duplicar o mesmo fato.

#### Situação administrativa e condição observada

O campo histórico `colonies.status` admite `active`, `weak`, `recovering`, `inactive`, `lost` e `transferred`. Para compatibilidade com bancos anteriores, `weak` e `recovering` continuam aceitos, mas são tratados como **administrativamente ativos** quando o sistema precisa decidir se a colônia pode ser manejada.

A condição de manejo, especialmente a condição de colônia fraca, deve ser derivada da inspeção mais recente. Código novo não deve gravar `weak` ou `recovering` como novos caminhos de situação administrativa nem depender desses valores para determinar a força atual da colônia.

Assim, a projeção administrativa considera:

- `active`, `weak` e `recovering`: colônia operacionalmente manejável, respeitando o histórico da data do fato;
- `inactive`: indisponível durante o intervalo de inativação;
- `lost`: indisponível a partir da baixa por perda;
- `transferred`: indisponível a partir da transferência externa.

### Caixa

Objeto físico que abriga uma colônia. Caixa e colônia são entidades distintas.

Dados principais incluem código, meliponário, modelo, material, posição ou localização, situação física e observações.

Estados físicos operacionais:

- `active`: pode receber nova ocupação;
- `maintenance`: não pode receber nova ocupação;
- `retired`: não pode receber nova ocupação e é terminal no fluxo operacional normal desta fase.

Mudanças de estado relevantes geram registros em histórico próprio com data, estado anterior, novo estado, motivo e observações. Uma caixa ocupada não pode entrar em manutenção nem ser aposentada; a ocupação deve ser encerrada pelo fluxo adequado antes da mudança.

### Ocupação de caixa

Relaciona uma colônia a uma caixa durante um intervalo de tempo.

Cada ocupação possui início e, quando encerrada, fim. Uma troca de caixa encerra a ocupação anterior e cria outra.

Regras principais:

- uma colônia só pode ocupar uma caixa por vez;
- uma caixa só pode abrigar uma colônia por vez;
- colônia e caixa precisam pertencer ao mesmo meliponário para uma ocupação local;
- somente uma caixa em estado `active` pode receber nova ocupação;
- essa restrição existe tanto no backend quanto no SQLite;
- uma troca de caixa preserva todos os registros anteriores;
- a data de encerramento não pode invalidar a sequência histórica da ocupação.

### Inspeção

Registro de uma avaliação da colônia em determinada data.

Pode preservar força da colônia, presença de rainha, postura, reservas de alimento, condição das crias, pragas, observações, ações realizadas e próxima inspeção sugerida.

A inspeção guarda o contexto da caixa ocupada na data registrada. Inspeções retroativas resolvem a ocupação histórica correspondente, em vez de assumir a caixa atual.

### Foto de inspeção

Arquivo visual associado a uma inspeção já existente.

A inspeção fornece o vínculo com a colônia e o contexto temporal. A foto não mantém uma segunda referência independente à colônia.

Metadados principais:

- inspeção;
- caminho relativo do arquivo;
- nome original;
- tipo MIME;
- tamanho em bytes;
- data da captura;
- observações.

Regras principais:

- a foto precisa pertencer a uma inspeção existente;
- arquivos são copiados para `media/inspections/<inspection-id>/` no diretório de dados da aplicação;
- o SQLite guarda apenas metadados e o caminho relativo;
- nomes internos usam identificadores próprios para evitar colisões;
- os formatos iniciais aceitos são JPG, PNG e WebP;
- a exclusão protege o caminho para que apenas a área gerenciada de mídia possa ser afetada;
- a interface pode carregar uma prévia limitada e sob demanda, sem transformar o binário em estado persistido do frontend;
- ausência física do arquivo não autoriza apagar automaticamente os metadados históricos.

### Anexo gerenciado de meliponário

Documento ou arquivo associado ao contexto administrativo de um meliponário.

O anexo não é uma nova fonte de verdade para fatos de manejo. Ele complementa o Record Center da unidade com material externo, enquanto o vínculo e os metadados permanecem no SQLite.

Metadados principais:

- meliponário;
- nome original;
- caminho relativo gerenciado;
- extensão e tipo MIME quando reconhecidos;
- tamanho em bytes;
- descrição e observações;
- data de inclusão.

Regras principais:

- o arquivo precisa pertencer a um meliponário existente;
- a aplicação copia a origem para `media/attachments/meliponaries/<meliponary-id>/`;
- o nome físico é gerado por UUID, permitindo arquivos distintos com o mesmo nome original;
- o caminho de origem não é mantido como dependência permanente;
- a remoção afeta apenas a cópia gerenciada e seus metadados, nunca o arquivo original usado na importação;
- a ausência física preserva os metadados e é exposta pelo diagnóstico de arquivos;
- paths absolutos, travessia de diretório e escapes da área `media/` são rejeitados.

Consulte [FILES.md](FILES.md) para a política completa de armazenamento, diagnóstico, backup e abertura dos arquivos.

### Alimentação

Registro de alimentação ou suplementação associada a uma colônia.

Preserva data, tipo de alimento, quantidade e unidade quando informadas, resposta observada, observações, próxima alimentação sugerida e a caixa correspondente à data registrada.

Uma alimentação retroativa resolve a ocupação histórica da colônia naquela data.

### Produção

Registro de retirada ou produção associada a uma colônia.

Tipos iniciais incluem mel, pólen, própolis, cera, cerume e outros produtos. O registro preserva quantidade, unidade, finalidade opcional, observações e a caixa histórica da data.

### Evento de colônia

Fato operacional ou biológico registrado manualmente quando não pertence a uma entidade mais específica.

Exemplos iniciais incluem enxameação, abandono, perda de rainha, ataque, praga ou inimigo, recuperação, observação e outros fatos relevantes.

Eventos preservam data, severidade, detalhes e contexto de caixa quando aplicável.

Fatos que possuem entidade própria, como alimentação, produção, movimentação ou manutenção, não devem ser duplicados como eventos genéricos apenas para aparecer na timeline.

### Divisão e multiplicação

Registra uma tentativa de multiplicação de colônia e seu resultado.

Preserva colônia-mãe, data, resultado, caixa histórica da mãe e observações. Quando o resultado cria uma nova colônia, a descendente recebe vínculo genealógico com a origem.

Regras principais:

- resultados bem-sucedidos ou parciais podem criar uma colônia filha na mesma transação;
- a filha preserva espécie e contexto de origem da mãe;
- divisões malsucedidas não criam colônias fantasmas;
- colônias em situação incompatível com multiplicação não podem originar novas divisões.

### Movimentação

Representa entrada, saída, transferência ou transporte de colônias quando há mudança ou registro de trânsito do plantel.

Tipos operacionais atuais incluem:

- transferência interna entre meliponários cadastrados;
- transferência externa para fora do plantel;
- transporte temporário sem alteração do plantel atual.

A movimentação preserva origem, destino e contexto das caixas quando aplicável. Operações que alteram plantel e ocupação são tratadas transacionalmente para evitar estado parcial.

#### Transporte temporário e retorno

Um movimento `transport` representa a saída para transporte temporário e não altera o meliponário atual, a caixa atual nem o saldo do plantel. O movimento original permanece preservado como fato histórico.

A conclusão do transporte é registrada separadamente em `transport_returns`. Enquanto não existe retorno ativo, o transporte está aberto. Um retorno ativo marca sua conclusão e preserva data e observações.

Reabrir um transporte não apaga o retorno anterior: o retorno é revertido com data e motivo, mantendo a trilha histórica e de auditoria. Uma colônia pode ter no máximo um transporte temporário aberto por vez, inclusive quando a abertura decorre de uma reabertura.

### Documento de movimentação

Evidência ou referência associada a uma movimentação já registrada. Uma movimentação pode possuir zero, um ou vários documentos.

Dados principais:

- tipo do documento;
- número ou referência;
- sistema de origem opcional, como GEDAVE ou GEFAU;
- emissor opcional;
- data de emissão;
- validade opcional;
- caminho de arquivo opcional;
- observações.

Tipos iniciais incluem GTA, autorização, nota fiscal, recibo, declaração, protocolo, certificado e outros.

Regras principais:

- o documento sempre pertence a uma movimentação existente;
- a colônia é derivada da movimentação e não informada separadamente;
- a mesma combinação de movimentação, tipo e referência não é duplicada;
- validade não pode anteceder emissão quando ambas estiverem presentes;
- o sistema registra a evidência, mas não afirma validade jurídica;
- arquivos são referenciados por caminho e não armazenados como BLOB no SQLite.

O campo legado `document_reference` existe apenas por compatibilidade. Referências antigas são normalizadas para a estrutura documental, evitando duas fontes permanentes de verdade.

### Manutenção de caixa

Registro pertencente à caixa física.

Pode preservar data, tipo de manutenção, descrição, responsável, custo, próxima manutenção e a colônia que ocupava a caixa naquela data, quando houver.

Manutenção não é troca de caixa e não altera automaticamente a ocupação. O vínculo histórico com a colônia é contexto, não propriedade do registro.

### Transição de ciclo de vida

Registra mudanças explícitas de situação da colônia.

Transições atuais incluem:

- baixa por perda;
- inativação;
- reativação.

Cada transição preserva situação anterior, nova situação, data, motivo, observações e caixa histórica quando aplicável.

Perda e inativação encerram uma ocupação ativa quando necessário. Reativação não reabre automaticamente a caixa antiga.

## Política temporal da série 0.x

Timestamps operacionais são tratados como horário local da máquina e persistidos no formato canônico `YYYY-MM-DD HH:MM:SS`.

A fronteira backend normaliza valores recebidos de controles `datetime-local`, aceita o formato já persistido e rejeita timestamps claramente inválidos. Quando um timestamp operacional é omitido, o backend obtém o horário local pela mesma referência SQLite usada nas comparações de vencimento.

Registros históricos existentes não são reescritos em massa. `created_at` técnico pode permanecer com a semântica histórica do schema; a política central se aplica principalmente aos timestamps de domínio usados para ordenação, contexto histórico e alertas.

Os campos sugeridos `next_inspection_at`, `next_feeding_at` e `next_maintenance_at` não podem anteceder o fato que os originou.

## Manejo retroativo e disponibilidade histórica

A disponibilidade para inspeção, alimentação e produção é avaliada **na data do fato**, não apenas pelo estado atual da linha em `colonies`.

A avaliação considera entrada da colônia, inativação, reativação, baixa por perda e transferência externa. Isso permite, por exemplo, lançar hoje uma inspeção antiga ocorrida antes de uma baixa já registrada, mas rejeita um manejo datado depois da baixa ou durante um período de inativação.

A regra principal vive no backend Rust e não depende da interface para proteger a integridade.

## Administração, correção e reversão

Cadastros mestres e fatos históricos possuem políticas diferentes.

Meliponários e espécies podem ser arquivados e reativados. Exclusão física é reservada a cadastros nunca utilizados. Caixas e colônias seguem a mesma proteção contra exclusão quando já possuem dependências históricas.

Fatos como inspeção, alimentação, produção, manutenção e eventos são corrigidos ou anulados sem apagar a linha original. Correções exigem motivo e registram auditoria com estado anterior e posterior. Anulações preservam o fato para consulta histórica e o excluem das projeções operacionais válidas.

Movimentações, divisões, ocupações e ciclo de vida usam operações específicas quando uma alteração possui consequência histórica. Reversões só são permitidas quando o estado posterior ainda torna a operação segura e restaurável.

Arquivamento administrativo de cadastro não deve ser confundido com estado físico da caixa nem com ciclo de vida da colônia.

## Agenda operacional

A Agenda representa **planejamento**, não fato histórico.

Uma tarefa pendente diz que algo precisa ser feito. Uma inspeção, alimentação ou manutenção diz que algo efetivamente aconteceu. Concluir uma tarefa especializada registra o fato correspondente e conclui o compromisso dentro do fluxo protegido pelo backend.

Estados de tarefa atuais:

- `pending`;
- `completed`;
- `cancelled`;
- `rescheduled`;
- `skipped`.

Atraso é derivado de `scheduled_for` e da referência temporal atual. Não existe um segundo estado persistido `overdue`.

Tarefas podem ser manuais ou derivadas de `next_inspection_at`, `next_feeding_at` e `next_maintenance_at`. A tarefa derivada preserva a linhagem do fato de origem para que uma correção factual continue tendo autoridade sobre o planejamento futuro.

Reagendar, cancelar, ignorar ou duplicar preserva a tarefa original. A reconciliação executada na inicialização pode recuperar tarefas derivadas ausentes e é idempotente: repetir `reconcile_all()` não deve criar duplicatas.

Consulte [AGENDA.md](AGENDA.md) para a política operacional completa.

## Contexto de meliponário na interface

O meliponário ativo na estrutura principal da interface é um **escopo de leitura e operação**.

Selecionar uma unidade filtra Agenda, alertas, Dashboard, colônias, caixas e fluxos de manejo compatíveis. A seleção não altera a propriedade de entidades nem registra movimentação.

Fluxos que precisam conhecer destinos externos ao contexto, como transferência entre meliponários cadastrados, continuam recebendo o catálogo completo de destinos. Assim, o filtro de origem não reduz artificialmente as opções necessárias à regra de domínio.

## Fichas operacionais

As fichas de Colônia, Caixa e Meliponário são projeções de leitura construídas pelo backend e usadas como pontos centrais de navegação.

Elas consolidam somente dados já existentes em suas fontes:

- Colônia: situação, caixa atual, inspeção e alimentação recentes, Agenda e alertas;
- Caixa: estado físico, ocupação, manutenção, Agenda e fotos cujo contexto histórico pertence à caixa;
- Meliponário: plantel, caixas, Agenda, atrasos, alertas, produção recente e arquivos administrativos associados ao próprio meliponário.

Essas fichas não criam tabelas paralelas nem se tornam novas fontes de verdade. Arquivos exibidos na ficha do meliponário permanecem entidades de suporte, com metadados próprios e binários gerenciados fora do SQLite.

## Visões derivadas

### Timeline unificada

A timeline agrega fatos de diferentes fontes sem duplicá-los.

Pode reunir:

- entrada no plantel derivada do cadastro;
- ocupações e trocas de caixa;
- inspeções;
- eventos manuais;
- alimentação;
- produção;
- divisões;
- movimentações;
- manutenção;
- transições de ciclo de vida.

A timeline é uma projeção de leitura. As fontes originais continuam sendo as tabelas e entidades correspondentes.

### Alertas

Alertas de manejo são derivados dos registros mais recentes e não persistidos como um segundo estado independente.

A implementação atual considera, entre outros casos:

- tarefas vencidas de inspeção;
- tarefas vencidas de alimentação;
- tarefas vencidas de manutenção;
- colônia fraca derivada da inspeção mais recente.

Quando uma pendência temporal possui tarefa derivada na Agenda, essa tarefa é a fonte operacional do vencimento. Isso evita manter simultaneamente um alerta independente de `next_*` para o mesmo compromisso.

Alertas carregam contexto suficiente para orientar a interface, incluindo tarefa relacionada quando houver e ação recomendada. A UI pode abrir a Agenda ou o fluxo factual indicado sem persistir estado adicional.

Colônias inativas, perdidas ou transferidas não geram alertas operacionais atuais. Valores legados `weak` e `recovering` não são fonte de verdade para a condição de fraqueza.

Registros mais novos substituem pendências anteriores quando representam o mesmo acompanhamento.

### Dashboard

O dashboard é uma visão operacional derivada pelo backend e complementada pela Agenda no contexto ativo.

Ele combina informações como situação administrativa das colônias, força da inspeção mais recente, distribuição por espécie, ocupação de caixas, alertas, produção recente e movimentações recentes. Valores legados `weak` e `recovering` são consolidados na projeção administrativa ativa, enquanto a força permanece derivada das inspeções.

Quando um meliponário específico está selecionado, a interface só apresenta como contextuais os indicadores que podem ser corretamente escopados. Dados consolidados que não possuem contrato de escopo não são misturados silenciosamente com a visão filtrada.

### Relatórios operacionais

A Central de Relatórios é uma projeção de leitura sobre as entidades e fatos existentes. Ela não cria tabelas paralelas, saldos independentes nem novos fatos de domínio.

Os relatórios usam período inclusivo e contexto opcional de meliponário. Fatos anulados ou operações revertidas são excluídos das projeções operacionais efetivas. O histórico de colônia pode ativar explicitamente um modo completo/auditoria para mostrar esses registros com seu estado, sem transformá-los novamente em fatos válidos.

A visão operacional combina uma fotografia atual do plantel com fatos ocorridos no período. Ela não afirma reconstruir historicamente o plantel na data final quando o schema atual não oferece informação suficiente para isso.

Produção é agregada no backend e sempre preserva a unidade. Quantidades com unidades diferentes não são somadas entre si.

Custos são limitados ao que está realmente persistido em `box_maintenance_records.cost`. O sistema não infere custos de alimentação, produção, movimentação ou preços de mercado. A interface atual trata esse campo como reais, embora o schema ainda não possua coluna própria de moeda.

Métricas da Agenda são contagens nomeadas de tarefas criadas, agendadas e dos estados finais. `rescheduled` permanece um estado terminal próprio e a tarefa sucessora é outro registro; reagendamento não é reinterpretado automaticamente como falha.

CSV e impressão são formatos de saída dessas projeções. Eles não alteram o banco, não geram fatos de auditoria e não substituem backup ou exportação estrutural. Consulte [REPORTS.md](REPORTS.md) para filtros, relatórios disponíveis, CSV e impressão.

## Serviços de dados

Backup, exportação estrutural, relatórios, diagnóstico de arquivos e restauração são serviços operacionais e **não entidades do domínio**.

Eles trabalham sobre o estado persistido sem criar uma segunda fonte de verdade. A restauração é preparada, validada e aplicada de forma controlada, com backup de segurança do estado atual antes da troca. CSV e impressão são saídas de consulta; exportação JSON permanece uma representação estrutural e backup completo continua sendo o mecanismo de recuperação dos dados e binários gerenciados.

Consulte [DATA-MANAGEMENT.md](DATA-MANAGEMENT.md), [FILES.md](FILES.md) e [REPORTS.md](REPORTS.md).

## Princípios

### Histórico não é sobrescrito

Quando uma mudança possui significado histórico, ela deve gerar um novo registro, uma nova ocupação ou uma transição explícita.

### Saldo é consequência

O estado atual do plantel deve ser derivável dos cadastros, multiplicações, transferências, baixas e demais fatos registrados. Alterações manuais sem histórico devem ser evitadas.

### Genealogia é rastreável

Divisões preservam a relação entre origem e descendentes.

Exemplo:

```text
Jataí-01
└── Jataí-03
    └── Jataí-12
```

### Linguagem simples, dados corretos

A interface pode usar termos familiares ao meliponicultor. O modelo interno preserva nomenclatura técnica, integridade e rastreabilidade.

### Estado derivado não deve virar estado paralelo

Timeline, alertas, dashboard, Agenda, fichas operacionais e relatórios devem ser calculados a partir dos registros reais sempre que possível, evitando duplicação e inconsistência.
