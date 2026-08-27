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

## Fichas internas

`RecordWorkspace` estabelece o padrão para abrir uma entidade em área interna. A primeira aplicação é a ficha-resumo da colônia, usando somente dados já existentes. Novas abas operacionais serão adicionadas quando houver conteúdo real, evitando telas vazias ou promessas de funcionalidade futura.

## Responsividade desktop

O layout é desktop-first e considera aproximadamente `900x600` como piso operacional. A sidebar se recolhe automaticamente em larguras menores, grids passam a uma coluna e toolbars reorganizam seus controles. Em janelas maiores, sidebar, tabelas e toolbars usam a densidade completa.

## Estrutura de estilos

- `src/styles.css`: tokens, temas e fundação do shell;
- `src/styles/enterprise.css`: fichas, estados semânticos e componentes enterprise;
- `src/styles/operations.css`: ajustes dos workspaces operacionais.

As cores funcionais são referenciadas por tokens como `surface`, `border`, `text-primary`, `primary`, `danger`, `warning`, `success`, `info` e `focus-ring`.
