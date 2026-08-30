# Documentação do MeliponarioManager

Este diretório concentra a documentação de **engenharia, arquitetura e manutenção** do projeto. Para instalação e uso cotidiano da aplicação, consulte a [Wiki do MeliponarioManager](https://github.com/marcelositr/MeliponarioManager/wiki).

O `README.md` da raiz apresenta o produto; os documentos abaixo registram o comportamento atual, as decisões de domínio e os procedimentos de manutenção.

## Guia de leitura

### Produto e domínio

- [Modelo de domínio](DOMAIN.md): entidades, invariantes, temporalidade, auditoria e projeções derivadas.
- [Agenda operacional](AGENDA.md): tarefas, estados, reconciliação e execução de manejos.
- [Transporte temporário](TRANSPORT.md): saída, retorno, reabertura e regras de integridade.
- [Interface desktop](UI.md): estrutura da interface, contexto ativo, temas e responsividade.

### Dados e recursos operacionais

- [Gerenciamento de dados](DATA-MANAGEMENT.md): armazenamento local, backup, restauração e exportação.
- [Arquivos gerenciados](FILES.md): fotos, anexos, caminhos, diagnóstico e integridade dos binários.
- [Relatórios](REPORTS.md): filtros, métricas, CSV, impressão e limitações conhecidas.

### Engenharia e manutenção

- [Arquitetura](ARCHITECTURE.md): componentes, fluxo de execução, persistência e estrutura do código.
- [Operação no GitHub](GITHUB-OPERATIONS.md): Issues, Pull Requests, ruleset, Dependabot e rotina do repositório.
- [Distribuição desktop](DISTRIBUTION.md): validações, bundles, tags e diagnóstico de empacotamento.
- [Política de releases](RELEASES.md): versionamento, preparação, publicação e correções.
- [Roadmap](ROADMAP.md): direções futuras, critérios de priorização e itens fora do escopo atual.

### Histórico

- [Changelog](../CHANGELOG.md): mudanças relevantes por versão.
- [Notas de release](releases/): textos publicados com cada versão distribuída.

### Políticas do repositório

- [Como contribuir](../CONTRIBUTING.md): branches, commits, validação e critérios para Pull Requests.
- [Política de segurança](../SECURITY.md): versões suportadas e relato responsável de vulnerabilidades.

## Fonte de verdade

Cada fonte possui uma responsabilidade:

| Assunto | Fonte autoritativa |
| --- | --- |
| Schema persistido | Migrations, constraints, índices e triggers do SQLite |
| Regras, validações e transações | Backend Rust |
| Interação e apresentação | Contratos TypeScript e componentes React |
| Comportamento público atual | Documentação temática, conferida contra a implementação |
| Evolução histórica | Changelog e notas de release |
| Trabalho futuro | Roadmap e Issues |

Contradição entre código e documentação é defeito a corrigir, não licença para escolher a versão mais conveniente. O roadmap descreve intenção e não substitui comportamento implementado. Notas de release registram a versão correspondente e não servem como manual da versão atual.

## Responsabilidade de atualização

| Tipo de mudança | Documentação mínima |
| --- | --- |
| Entidade, estado ou regra de integridade | `DOMAIN.md` e documento temático aplicável |
| Fluxo de Agenda | `AGENDA.md` |
| Backup, restauração ou exportação | `DATA-MANAGEMENT.md` |
| Fotos, anexos ou caminhos gerenciados | `FILES.md` |
| Relatório, métrica, CSV ou impressão | `REPORTS.md` |
| Estrutura técnica ou fronteira entre camadas | `ARCHITECTURE.md` |
| Interface, navegação ou responsividade | `UI.md` |
| CI, GitHub ou dependências automatizadas | `GITHUB-OPERATIONS.md` |
| Bundle, tag ou publicação | `DISTRIBUTION.md` e `RELEASES.md` |
| Mudança relevante para usuários | `CHANGELOG.md` |
| Preparação de versão | `CHANGELOG.md` e `releases/<tag>.md` |

## Convenções editoriais

- documente o estado implementado no presente;
- mantenha decisões históricas no changelog ou nas notas de release;
- use nomes de tabelas, campos, comandos e estados entre crases;
- prefira links relativos entre arquivos do repositório;
- não registre prompts, conversas, caminhos pessoais ou detalhes temporários de uma ferramenta de IA;
- não prometa funcionalidade futura em documentos que descrevem o produto atual;
- atualize exemplos quando o contrato que eles demonstram mudar.
