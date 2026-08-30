# Roadmap experimental

O MeliponarioManager permanecerá em desenvolvimento contínuo na série `0.x`.

Este roadmap registra a evolução planejada do produto, mas **não representa uma sequência de releases históricas publicadas**. Antes da primeira distribuição pública, o projeto percorreu internamente vários marcos de escopo sem criar tags intermediárias.

A primeira versão preparada para distribuição pública consolidou esse trabalho em `v0.7.0` como GitHub Pre-release. A `v0.8.0` consolida o ciclo seguinte de robustez operacional, interface desktop, Agenda, relatórios e gerenciamento local de arquivos.

## Escopo consolidado até v0.7.0

### Marco 0.1 - Núcleo utilizável

**Concluído no desenvolvimento pré-release.**

- meliponários;
- espécies;
- colônias;
- caixas físicas;
- inspeções;
- eventos;
- histórico por colônia;
- plantel atual derivado dos registros.

### Marco 0.2 - Manejo

**Concluído no desenvolvimento pré-release.**

- alimentação e suplementação;
- manutenção de caixas;
- troca de caixa preservando ocupações anteriores;
- alertas básicos de acompanhamento derivados dos registros.

### Marco 0.3 - Multiplicação e genealogia

**Núcleo concluído no desenvolvimento pré-release.**

- divisões e multiplicações;
- relação entre colônia de origem e descendentes;
- consulta de genealogia por gerações;
- interface de registro e histórico de divisões.

Acompanhamentos pós-divisão mais especializados podem evoluir em ciclos futuros a partir do uso real.

### Marco 0.4 - Movimentações e rastreabilidade

**Concluído no desenvolvimento pré-release.**

- transferências internas entre meliponários cadastrados;
- transferências externas;
- transportes temporários;
- baixas, perdas, inativações e reativações com histórico;
- documentos e referências vinculados às movimentações;
- estrutura inspirada conceitualmente em fluxos de GEFAU, GEDAVE e GTA sem substituir sistemas oficiais.

### Marco 0.5 - Produção

**Concluído no desenvolvimento pré-release.**

- mel;
- pólen;
- própolis;
- cera e cerume;
- outros produtos;
- histórico por colônia.

### Marco 0.6 - Fotos e evolução visual

**Base concluída no desenvolvimento pré-release.**

- fotos associadas às inspeções;
- armazenamento local gerenciado;
- consulta de fotos no contexto histórico da colônia.

Uma experiência dedicada de comparação visual lado a lado permanece como possibilidade futura e não é tratada como requisito já entregue.

### Marco 0.7 - Dashboard, operação e dados

**Concluído para a primeira pré-release pública.**

- dashboard operacional do plantel;
- situação e força das colônias;
- distribuição por espécie;
- ocupação de caixas;
- pendências e alertas;
- produção e movimentações recentes;
- interfaces operacionais para os módulos já implementados;
- backup do SQLite e da mídia;
- exportação JSON;
- relatório Markdown;
- restauração preparada com validação de integridade;
- hardening de CI e pipeline de bundles Linux e Windows.

## v0.8.0 - robustez operacional e informação gerencial

A `v0.8.0` amplia robustez operacional e informação gerencial sem transformar o aplicativo em serviço web ou BI pesado.

### Fundação, shell, operação e Agenda

**Concluído na v0.8.0.**

- integridade e hardening de domínio;
- shell desktop enterprise e acessibilidade;
- correções/reversões auditáveis;
- Agenda operacional;
- centros de registro de Colônia, Caixa e Meliponário;
- lifecycle completo de transporte temporário com retorno e reabertura auditada.

### Bloco 5A - Product hardening

**Concluído tecnicamente.**

- comportamento de menubar e navegação contextual refinados;
- dialogs e menus de ações endurecidos para teclado e foco;
- mensagens técnicas reduzidas na superfície do produto;
- Agenda protegida contra respostas obsoletas;
- semântica de transporte temporário fechada com no máximo um transporte aberto por colônia;
- criação e reabertura protegidas por backend e SQLite;
- testes de regressão para o lifecycle de transporte.

### Bloco 5B - Relatórios, CSV e impressão

**Concluído tecnicamente.**

- Central de Relatórios dedicada;
- visão operacional por período e contexto;
- produção sem mistura de unidades;
- custos limitados aos valores realmente persistidos;
- desempenho da Agenda com estados semanticamente explícitos;
- histórico operacional de colônia e modo completo/auditoria;
- consolidação por meliponário;
- CSV UTF-8 seguro para planilhas;
- impressão pela WebView/sistema operacional com layout próprio para papel.

Relatórios são derivados dos dados existentes e não criam uma nova fonte de verdade nem exigem migration própria.

### Bloco 5C - Arquivos, portabilidade e acabamento local

**Concluído na v0.8.0.**

- anexos gerenciados no contexto do Record Center de Meliponário;
- migration aditiva `0017_managed_attachments.sql`;
- armazenamento binário fora do SQLite com nomes internos por UUID e caminhos relativos;
- seleção de arquivos por Dialog nativo;
- abertura e revelação pelo plugin oficial do desktop;
- tratamento explícito de arquivos ausentes sem apagar metadata;
- diagnóstico entre registros SQLite e filesystem;
- fotos de inspeção com picker nativo, contexto humano, thumbnails lazy e limite de prévia;
- backup completo com manifest versionado e inventário de assets;
- restauração validada com staging, cópia de segurança e rollback da troca;
- exportação JSON estrutural versionada sem incorporar binários e sem importador destrutivo;
- restauração de estado da janela pelo plugin oficial `tauri-plugin-window-state`;
- testes de backend, migrations e estrutura frontend específicos do bloco.

A implementação acumulada da `v0.8.0` foi tecnicamente validada por revisão, testes automatizados, CI e build desktop. A validação prática em desktop e uso real permanece uma atividade separada de acompanhamento da pre-release.

Consulte [FILES.md](FILES.md), [DATA-MANAGEMENT.md](DATA-MANAGEMENT.md) e [REPORTS.md](REPORTS.md).

## Próximos ciclos 0.x

As próximas versões serão definidas pelo uso real do sistema e não por uma corrida artificial de números.

Temas prováveis de evolução:

- polimento de UX e fluxos de edição;
- filtros, busca e navegação em bases maiores;
- validação prática com dados reais de meliponários;
- testes de regressão e migração mais amplos;
- maior reprodutibilidade de builds e dependências;
- refinamento de backup, restauração e portabilidade entre versões;
- assinatura de código e amadurecimento da distribuição desktop;
- melhorias de análise e acompanhamento conforme surgirem necessidades reais de manejo.

## Política de versão

- o projeto permanece em `0.x` por decisão de produto;
- novas funcionalidades compatíveis incrementam `MINOR`;
- correções compatíveis incrementam `PATCH`;
- não existe meta de lançamento `v1.0.0`;
- versões anteriores a `v0.7.0` são marcos de planejamento interno, não releases públicas retroativas.

Consulte [RELEASES.md](RELEASES.md) para a política de tags e publicação.
