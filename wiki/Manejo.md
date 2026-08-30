# Manejo

O MeliponarioManager registra os principais trabalhos realizados com colônias e caixas preservando data, contexto e histórico.

## Inspeções

Uma inspeção pode registrar informações como:

- força ou condição observada;
- presença de rainha;
- postura;
- reservas de alimento;
- condição das crias;
- pragas ou inimigos;
- observações;
- ações realizadas;
- próxima inspeção sugerida.

Quando uma inspeção é registrada com data anterior, a aplicação procura relacioná-la à caixa que a colônia ocupava naquela data, e não simplesmente à caixa atual.

## Alimentação

Registre suplementações e alimentações com os dados disponíveis, como:

- data;
- tipo de alimento;
- quantidade e unidade;
- resposta observada;
- observações;
- próxima alimentação sugerida.

A próxima data pode gerar planejamento na [Agenda](Agenda-e-alertas).

## Produção

Registros de produção podem representar itens como:

- mel;
- pólen;
- própolis;
- cera;
- cerume;
- outros produtos.

Informe quantidade e unidade corretamente. Relatórios não somam automaticamente unidades diferentes, como `kg` e `g`.

## Manutenção de caixa

Manutenção pertence à **caixa física**.

Você pode registrar data, tipo de manutenção, descrição, responsável, custo, próxima manutenção e o contexto da colônia que ocupava a caixa naquele momento, quando aplicável.

Manutenção não é troca de caixa. Se a colônia realmente mudou de caixa, utilize o fluxo correspondente.

## Eventos

Eventos servem para registrar fatos relevantes que não possuem uma entidade mais específica, por exemplo observações biológicas ou operacionais importantes.

Evite duplicar como evento algo que já foi registrado como alimentação, produção, movimentação ou manutenção.

## Fotos de inspeção

Fotos pertencem à inspeção. A aplicação copia os arquivos selecionados para sua área local gerenciada e mantém no banco os metadados e vínculos.

Veja [Fotos e arquivos](Fotos-e-arquivos).

## Próximas datas

Ao informar uma próxima inspeção, alimentação ou manutenção, você está criando uma referência para **planejamento futuro**. A ação que ocorreu continua sendo o registro de manejo original.

A [Agenda](Agenda-e-alertas) transforma essas próximas datas em compromissos operacionais quando aplicável.

## Correções e anulações

Quando um registro histórico estiver incorreto, prefira sempre as funções de correção, anulação ou reversão oferecidas pela aplicação. Elas existem para preservar a trilha histórica em vez de simplesmente apagar o passado.

---

[← Caixas e colônias](Caixas-e-colonias) · [Próximo: Agenda e alertas →](Agenda-e-alertas)