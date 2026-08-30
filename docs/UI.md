# Interface desktop

A interface combina princípios de Material 3, densidade de sistemas de gestão e comportamento desktop. O objetivo é oferecer operação diária compacta, legível e previsível dentro da WebView do Tauri.

## Estrutura principal

O shell contém:

1. menu superior;
2. barra de contexto, com seletor de meliponário;
3. sidebar agrupada e recolhível;
4. workspace principal rolável;
5. barra de status.

A sidebar organiza:

- Operação;
- Plantel;
- Manejo;
- Rastreabilidade;
- Administração.

Fotos de inspeção permanecem acessíveis no fluxo de Manutenção e nas fichas contextuais; não possuem item global próprio na navegação.

## Princípios visuais

- hierarquia por borda, contraste e elevação discreta;
- tipografia do sistema;
- controles compactos e com espaçamento previsível;
- tabelas para coleções de registros;
- cards para indicadores e resumos;
- âmbar/ocre como identidade;
- vermelho reservado a erro, criticidade e ação destrutiva;
- ícones SVG internos com linguagem consistente.

## Temas

Preferências disponíveis:

- `light`;
- `dark`;
- `system`.

Tema, estado da sidebar e meliponário ativo são preferências locais da interface. O plugin `tauri-plugin-window-state` mantém o estado da janela separadamente.

Cores devem usar tokens semânticos, incluindo `surface`, `text-primary`, `primary`, `danger`, `warning`, `success`, `info` e `focus-ring`. Componentes não devem depender de cores literais para comunicar estado.

## Contexto de meliponário

O seletor global oferece todos os meliponários ou uma unidade específica.

O contexto filtra módulos que possuem contrato confiável para isso, como Agenda, alertas, colônias, caixas e fluxos de manejo. Catálogos necessários a operações entre unidades continuam completos; uma transferência precisa enxergar destinos fora do meliponário ativo.

Quando uma projeção consolidada não pode ser corretamente filtrada, a interface informa a limitação em vez de misturar números globais com o contexto selecionado.

Selecionar um contexto não altera dados de domínio.

## Navegação

`WorkspaceRouter` escolhe a página ativa. Atalhos contextuais usam `NavigationIntent` para transportar, quando necessário:

- tarefa;
- colônia;
- caixa;
- meliponário;
- intenção de criar ou abrir.

Uma troca manual de meliponário remove intents incompatíveis, evitando aplicar contexto antigo sobre a nova seleção.

## Dialogs e confirmações

Dialogs reutilizáveis:

- bloqueiam interação com o fundo;
- não fecham por clique externo por padrão;
- aceitam `Esc`;
- mantêm `Tab` e `Shift+Tab` no dialog superior;
- restauram o foco anterior ao fechar;
- não reinicializam autofocus durante edição normal.

Confirmações destrutivas devem explicar a consequência da operação. Ações administrativas que exigem motivo usam um dialog próprio e não prompts genéricos do navegador.

## Fichas operacionais

`RecordWorkspace` hospeda fichas de Colônia, Caixa e Meliponário. As fichas consolidam projeções do backend e oferecem navegação para fluxos especializados.

Elas não criam estado paralelo e não devem exibir abas vazias ou funcionalidades prometidas mas ainda inexistentes.

## Menus e seleção de texto

Menus de ações são superfícies flutuantes ancoradas ao controle. Eles ficam fora de containers com `overflow` para não deslocar ou cortar linhas da tabela e reposicionam-se dentro da viewport.

A superfície geral segue o comportamento de seleção de um aplicativo desktop. Inputs, textareas, conteúdo editável e regiões `.selectable` continuam permitindo seleção e cópia.

## Responsividade

Faixas adotadas:

| Faixa | Comportamento |
| --- | --- |
| Acima de `1199px` | Composição ampla |
| `900px` a `1199px` | Composição intermediária |
| Abaixo de `900px` | Composição compacta e sidebar recolhida |
| `1024x768` ou superior | Alvo principal |
| `800x600` | Compatibilidade operacional |
| Mínimo da janela `760x520` | Margem de resiliência |

Toolbars e grupos de ação podem quebrar linha. Tabelas usam rolagem horizontal local quando necessário; o shell completo não deve exigir rolagem horizontal para alcançar operações essenciais.

## Estilos

| Arquivo | Responsabilidade |
| --- | --- |
| `src/styles.css` | Tokens, temas, shell, primitives, dialogs e responsividade global |
| `src/styles/enterprise.css` | Fichas, estados semânticos e componentes de gestão |
| `src/styles/operations.css` | Workspaces operacionais |
| `src/styles/reports.css` | Relatórios e impressão |
| `src/styles/files.css` | Arquivos, diagnóstico e fotos |

Mudanças visuais devem ser testadas em temas claro e escuro, com teclado e em pelo menos uma faixa compacta e uma faixa ampla.
