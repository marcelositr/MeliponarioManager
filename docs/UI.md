# Interface desktop

A interface da série `0.8.x` segue uma interpretação de **Material 3 com densidade enterprise e comportamento desktop**. O objetivo é parecer um sistema de gestão usado diariamente, não uma página web ampliada dentro do WebView.

## Princípios visuais

- superfícies hierarquizadas por borda, contraste e elevação discreta;
- tipografia de sistema, compacta e legível;
- controles com altura reduzida e espaçamento previsível;
- tabelas para conjuntos de registros e cards apenas para indicadores e resumos;
- cor de identidade em âmbar/ocre sóbrio;
- vermelho reservado a erro, criticidade e confirmação destrutiva;
- ícones SVG internos com o mesmo traço, sem mistura de emoji ou bibliotecas visuais.

## Temas

Há três preferências: `light`, `dark` e `system`. A preferência fica no `localStorage` e todo o aplicativo muda por tokens CSS compartilhados. O modo escuro usa grafite e cinzas profundos, sem preto absoluto ou cores neon.

Os temas possuem contratos explícitos de foreground e background. Em particular, superfícies, textos primário/secundário/muted, ações primárias e ações destrutivas usam tokens semânticos próprios para manter legibilidade em ambos os modos.

## Shell

O shell é composto por:

1. menu superior desktop;
2. barra de contexto da tela, incluindo o seletor de meliponário;
3. sidebar semântica e recolhível;
4. workspace principal rolável;
5. status bar inferior.

A sidebar agrupa Operação, Plantel, Manejo, Rastreabilidade e Administração. Fotos deixam de ser uma seção principal e permanecem acessíveis de forma transitória dentro de Manutenção até a consolidação das fichas completas.

## Meliponário ativo

O shell mantém um contexto global visual com `Todos os meliponários` ou uma unidade específica. A preferência é persistida localmente. Nesta etapa o contexto **não força filtros silenciosos** em módulos cujo backend ainda não oferece filtragem confiável; portanto o usuário sempre enxerga quando está em visão consolidada sem risco de dados desaparecerem apenas na interface.

## Dialogs e confirmações

Cadastros e operações principais deixam de ocupar permanentemente as telas e passam a abrir em dialogs reutilizáveis. Dialogs aceitam `Esc`, bloqueiam o fundo e não fecham por clique externo por padrão. Confirmações destrutivas descrevem a consequência antes da ação.

O ciclo de foco é inicializado uma vez por abertura do dialog. Renderizações normais provocadas por edição de campos não reinicializam autofocus. `Tab` e `Shift+Tab` permanecem contidos no dialog superior e, ao fechar, o foco retorna ao controle anterior quando ainda estiver disponível.

## Fichas internas

`RecordWorkspace` estabelece o padrão para abrir uma entidade em área interna. A primeira aplicação é a ficha-resumo da colônia, usando somente dados já existentes. Novas abas operacionais serão adicionadas quando houver conteúdo real, evitando telas vazias ou promessas de funcionalidade futura.

## Responsividade desktop

O layout continua desktop-first, mas não depende de uma resolução única.

- `1024x768` ou superior é a faixa de primeira classe, com composição completa;
- `800x600` é compatibilidade operacional: todas as operações essenciais devem permanecer acessíveis, mesmo com composição mais compacta;
- Full HD, 1440p e superiores preservam largura máxima de conteúdo quando isso melhora leitura, em vez de esticar painéis indefinidamente;
- a janela Tauri pode ser reduzida até `760x520` como margem de resiliência, sem transformar essa dimensão em alvo de primeira classe.

A linguagem de composição usa três estados principais: wide acima de `1199px`, medium entre `900px` e `1199px`, e compact abaixo de `900px`. Alturas reduzidas também recebem ajustes próprios para manter headers, corpos roláveis e ações de dialogs acessíveis.

A sidebar recolhe automaticamente no estado compact. Grids e formulários reduzem colunas conforme o espaço disponível, toolbars e grupos de ações aceitam quebra de linha e tabelas podem ter scroll horizontal local. A aplicação inteira não deve exigir scroll horizontal para acessar operações.

## Menus e seleção de texto

Menus de ações de registros são superfícies flutuantes ancoradas ao controle, renderizadas fora de containers de tabela para não deslocar linhas nem serem cortadas por `overflow` local. A posição é recalculada para permanecer dentro da viewport.

A superfície comum da aplicação usa comportamento de seleção típico de software desktop. Inputs, textareas, conteúdo editável e regiões explicitamente marcadas como `.selectable` continuam permitindo seleção e cópia.

## Estrutura de estilos

- `src/styles.css`: tokens, temas, shell, primitives compartilhados, dialogs, action groups e responsividade global;
- `src/styles/enterprise.css`: fichas, estados semânticos e componentes enterprise;
- `src/styles/operations.css`: ajustes dos workspaces operacionais;
- `src/styles/reports.css`: composição e impressão dos relatórios;
- `src/styles/files.css`: arquivos gerenciados, diagnósticos e composição de fotos.

As cores funcionais são referenciadas por tokens como `surface`, `surface-secondary`, `surface-raised`, `border`, `text-primary`, `text-secondary`, `text-muted`, `primary`, `on-primary`, `danger`, `on-danger`, `warning`, `success`, `info` e `focus-ring`.
