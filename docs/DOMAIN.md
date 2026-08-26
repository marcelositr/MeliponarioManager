# Modelo de domínio inicial

Este documento registra os conceitos centrais do MeliponarioManager antes da escolha final de tecnologia e implementação.

## Entidades principais

### Meliponário

Unidade de criação onde as colônias são mantidas. Deve permitir múltiplos meliponários no futuro.

### Espécie

Representa a espécie de abelha sem ferrão associada às colônias.

### Colônia

Entidade biológica e histórica. Mantém identidade própria ao longo do tempo, mesmo quando muda de caixa.

### Caixa

Objeto físico que abriga uma colônia. Caixa e colônia não são a mesma entidade.

### Inspeção

Registro de uma avaliação da colônia em determinada data, incluindo força, postura, alimento, crias, rainha, pragas, observações e fotos.

### Evento

Fato histórico relacionado a uma colônia, caixa ou meliponário. Exemplos: entrada, divisão, multiplicação, alimentação, manutenção, troca de caixa, transferência, enxameação, abandono, perda de rainha, ataque e baixa.

### Produção

Registro de retirada ou produção de mel, pólen, própolis, cera/cerume e outros produtos.

### Movimentação

Entrada, saída ou transferência de colônias. Pode conter dados documentais relacionados a GTA, GEFAU, GEDAVE ou outros registros quando aplicável.

## Princípios

### Histórico não é sobrescrito

Quando uma mudança possui significado histórico, ela deve ser representada por um novo registro ou evento.

### Saldo é consequência

O total atual do plantel deve ser derivável das entradas, multiplicações, transferências, baixas e demais movimentações, evitando alterações manuais sem histórico.

### Genealogia

Uma divisão deve preservar a relação entre a colônia de origem e as colônias resultantes.

Exemplo:

```text
Jataí-01
└── Jataí-03
    └── Jataí-12
```

### Linguagem simples, dados corretos

A interface deve usar termos familiares ao meliponicultor, enquanto o modelo interno preserva consistência técnica e rastreabilidade.

## Referências conceituais

Os fluxos de plantel, origem, saldo, movimentação, transferência, baixa e transporte podem considerar como referência conceitual GEFAU, GEDAVE e GTA do Estado de São Paulo.

Essas referências ajudam a estruturar os dados, mas o MeliponarioManager não pretende reproduzir a burocracia ou substituir sistemas oficiais.
