# Perguntas frequentes

## Caixa e colônia são a mesma coisa?

Não. A colônia é a unidade biológica e histórica. A caixa é o objeto físico que a abriga durante um período.

Veja [Conceitos básicos](Conceitos-basicos).

## Preciso criar outra colônia quando troco a caixa?

Não. Use o fluxo de troca de caixa para preservar a mesma colônia e encerrar/criar as ocupações correspondentes.

## Posso administrar mais de um meliponário?

Sim. O sistema suporta múltiplos meliponários e possui um seletor de contexto para trabalhar com uma unidade específica.

## A aplicação envia meus dados para a nuvem?

Não automaticamente. O MeliponarioManager é local-first e mantém banco, fotos e anexos no armazenamento local da aplicação.

## Exportar JSON é a mesma coisa que fazer backup?

Não. O JSON contém dados estruturados e metadados, mas não inclui os bytes das fotos e anexos. Para recuperação integral, use o backup completo.

## Exportar CSV é backup?

Não. CSV serve para planilhas e análise. Ele contém apenas os dados do relatório exportado.

## Posso salvar um relatório em PDF?

Quando o sistema operacional oferece a opção **Salvar como PDF** na janela de impressão, sim. A aplicação utiliza o mecanismo de impressão do sistema.

## Por que uma tarefa está atrasada se existe uma inspeção antiga?

A Agenda representa compromissos futuros. Uma tarefa só deixa de estar pendente quando o fluxo correspondente a conclui ou quando você toma outra decisão válida, como reagendar ou cancelar.

## Posso registrar uma inspeção antiga?

O sistema possui suporte a manejo retroativo dentro das regras históricas. Quando possível, ele resolve a caixa ocupada pela colônia na data do fato.

## O que acontece se eu apagar uma foto manualmente no disco?

O registro pode permanecer no banco e a aplicação passará a indicar **Arquivo não encontrado**. Use o diagnóstico de arquivos e seus backups para investigar.

## O MeliponarioManager emite GTA ou autorização oficial?

Não. Ele pode registrar documentos e referências relacionados a movimentações, mas não substitui sistemas oficiais e não certifica validade jurídica.

## O projeto é estável?

Ele está em desenvolvimento experimental na série `0.x`. Mantenha backups, especialmente antes de atualizações.

## Onde encontro ajuda para um problema?

Comece por [Solução de problemas](Solucao-de-problemas). Se o problema for reproduzível e persistir, consulte as Issues do repositório oficial sem expor dados privados.

---

[← Solução de problemas](Solucao-de-problemas) · [Voltar à Home](Home)