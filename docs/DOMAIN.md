# Modelo de domínio

Este documento registra os conceitos centrais do MeliponarioManager e as decisões que orientam a implementação.

## Entidades principais

### Meliponário

Unidade de criação onde as colônias e caixas são mantidas. O sistema suporta múltiplos meliponários desde o núcleo inicial.

Dados iniciais: nome, responsável, localização e observações.

### Espécie

Representa a espécie de abelha sem ferrão associada às colônias.

Dados iniciais: nome popular, nome científico, gênero e observações.

### Colônia

Entidade biológica e histórica. Mantém identidade própria ao longo do tempo, mesmo quando muda de caixa.

Dados iniciais: código, espécie, meliponário, origem, data de instalação, situação, colônia-mãe e observações.

### Caixa

Objeto físico que abriga uma colônia. Caixa e colônia não são a mesma entidade.

Dados iniciais: código, meliponário, modelo, material, posição/localização, situação física e observações.

### Ocupação de caixa

Relaciona uma colônia a uma caixa durante um intervalo de tempo.

Cada registro possui início e, quando houver mudança, fim. Assim, uma troca de caixa encerra a ocupação anterior e cria outra, sem apagar o histórico.

Regras iniciais:

- uma colônia só pode ocupar uma caixa por vez;
- uma caixa só pode abrigar uma colônia por vez;
- colônia e caixa precisam pertencer ao mesmo meliponário;
- uma troca de caixa mantém os registros anteriores;
- a data da troca não pode anteceder o início da ocupação atual.

### Inspeção

Registro de uma avaliação da colônia em determinada data, incluindo força, postura, alimento, crias, rainha, pragas, observações e fotos.

> Ainda não implementada no núcleo atual.

### Evento

Fato histórico relacionado a uma colônia, caixa ou meliponário. Exemplos: entrada, divisão, multiplicação, alimentação, manutenção, troca de caixa, transferência, enxameação, abandono, perda de rainha, ataque e baixa.

### Produção

Registro de retirada ou produção de mel, pólen, própolis, cera/cerume e outros produtos.

### Movimentação

Entrada, saída, transferência ou transporte de colônias. A movimentação preserva origem, destino e contexto das caixas quando aplicável.

Dados documentais relacionados a GTA, GEFAU, GEDAVE ou outros registros são associados à movimentação, mas não fazem parte da identidade da colônia nem alteram por si mesmos o estado do plantel.

### Documento de movimentação

Evidência ou referência associada a uma movimentação já registrada. Uma movimentação pode possuir zero, um ou vários documentos.

Dados iniciais:

- tipo do documento;
- número ou referência;
- sistema de origem opcional, como GEDAVE ou GEFAU;
- emissor opcional;
- data de emissão;
- validade opcional;
- caminho de arquivo opcional;
- observações.

Tipos iniciais: GTA, autorização, nota fiscal, recibo, declaração, protocolo, certificado e outros.

Regras iniciais:

- o documento sempre pertence a uma movimentação existente;
- a colônia do documento é derivada da movimentação e não é informada separadamente;
- a mesma combinação de movimentação, tipo e referência não é duplicada;
- quando emissão e validade estiverem presentes, a validade não pode anteceder a emissão;
- o sistema registra a referência, mas não afirma validade jurídica do documento;
- arquivos são referenciados por caminho e não armazenados como BLOB no SQLite.

O campo antigo `document_reference` de movimentação existe apenas por compatibilidade. A migration documental normaliza referências antigas e um trigger converte automaticamente novas gravações legadas para a entidade de documento.

## Princípios

### Histórico não é sobrescrito

Quando uma mudança possui significado histórico, ela deve ser representada por um novo registro ou evento.

A ocupação de caixas já segue esse princípio: a posição atual é consequência da sequência histórica de ocupações.

### Saldo é consequência

O total atual do plantel deve ser derivável das entradas, multiplicações, transferências, baixas e demais movimentações, evitando alterações manuais sem histórico.

### Genealogia

Uma divisão deve preservar a relação entre a colônia de origem e as colônias resultantes. O núcleo já reserva a relação de colônia-mãe para sustentar essa genealogia.

Exemplo:

```text
Jataí-01
└── Jataí-03
    └── Jataí-12
```

### Linguagem simples, dados corretos

A interface usa termos familiares ao meliponicultor, enquanto o modelo interno preserva consistência técnica e rastreabilidade.

## Referências conceituais

Os fluxos de plantel, origem, saldo, movimentação, transferência, baixa e transporte podem considerar como referência conceitual GEFAU, GEDAVE e GTA do Estado de São Paulo.

Essas referências ajudam a estruturar os dados, mas o MeliponarioManager não pretende reproduzir a burocracia, substituir sistemas oficiais ou decidir sozinho se um documento atende exigências legais vigentes.
