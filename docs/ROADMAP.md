# Roadmap

O roadmap registra direções de evolução do MeliponarioManager. Ele não é promessa de prazo, sequência obrigatória de versões nem fonte de verdade sobre funcionalidades já entregues.

O comportamento atual está documentado nos manuais temáticos. O histórico de entregas está no [changelog](../CHANGELOG.md) e nas [notas de release](releases/).

## Base atual

A linha `0.8.x` oferece uma aplicação desktop local-first com:

- cadastros de meliponários, espécies, colônias e caixas;
- inspeções, alimentação, produção, eventos e manutenção;
- ocupações, divisões, genealogia, movimentações e ciclo de vida;
- Agenda, alertas, Dashboard e fichas operacionais;
- relatórios, CSV e impressão;
- fotos e anexos gerenciados;
- backup completo, restauração validada e exportação estrutural;
- auditoria, correções, anulações e reversões;
- bundles experimentais para Linux e Windows.

Essa lista estabelece o ponto de partida. Detalhes e limitações pertencem aos documentos temáticos.

## Critérios de priorização

Novos trabalhos devem ser avaliados nesta ordem:

1. integridade e recuperação dos dados;
2. correção de regras de domínio e rastreabilidade;
3. falhas que impedem operações de manejo;
4. usabilidade dos fluxos existentes;
5. desempenho com bases maiores;
6. novas funcionalidades comprovadas por uso real.

Uma ideia não entra no produto apenas por ser tecnicamente possível. Ela precisa resolver um problema observável sem criar uma segunda fonte de verdade ou comprometer o uso local-first.

## Próximas frentes

### Validação prática

- testar atualização e restauração com bases reais e cópias de trabalho;
- ampliar cenários de migrations e regressão;
- validar fluxos completos nos bundles distribuídos;
- registrar problemas reproduzíveis como Issues.

### Usabilidade e escala

- melhorar busca, filtros e navegação em conjuntos maiores;
- reduzir atrito em correções e operações repetitivas;
- aprimorar acessibilidade por teclado;
- revisar comportamento em diferentes resoluções e ambientes gráficos.

### Dados e portabilidade

- ampliar testes de backup, rollback e diagnóstico de arquivos;
- documentar compatibilidade entre formatos quando eles evoluírem;
- avaliar importações adicionais somente com contratos seguros e não destrutivos;
- manter exportações derivadas separadas dos mecanismos de recuperação.

### Distribuição

- amadurecer os testes dos instaladores Linux e Windows;
- avaliar assinatura de código para Windows;
- melhorar diagnóstico de falhas específicas de plataforma;
- revisar suporte a plataformas conforme houver ambiente real de teste.

### Informação operacional

- evoluir relatórios a partir de perguntas reais de manejo;
- considerar comparação visual de inspeções quando o fluxo estiver definido;
- evitar agregações que misturem unidades, períodos ou estados incompatíveis;
- não introduzir BI ou sincronização remota sem necessidade demonstrada.

## Fora do escopo atual

Não fazem parte do compromisso atual:

- serviço em nuvem ou sincronização entre máquinas;
- aplicação web multiusuário;
- integração automática com GEFAU, GEDAVE, GTA ou sistemas equivalentes;
- emissão ou certificação jurídica de documentos;
- notificações push ou calendário externo;
- editor de documentos;
- conversão automática entre unidades de produção;
- reconstrução histórica completa do plantel sem suporte suficiente no schema.

Esses itens só devem ser reavaliados com requisitos concretos, impacto de segurança conhecido e estratégia de compatibilidade de dados.

## Gestão do roadmap

Trabalho executável deve ser registrado em Issues. Quando o volume justificar, um GitHub Project pode organizar prioridade e estado.

Este arquivo deve permanecer curto e estratégico. Planos concluídos migram para changelog/notas de release; decisões técnicas permanentes migram para o documento temático correspondente.
