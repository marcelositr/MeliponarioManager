# Modelo de domínio

Este documento registra os conceitos centrais do MeliponarioManager e as decisões que orientam a implementação.

## Princípio central

A **colônia** é uma entidade biológica e histórica. A **caixa** é um objeto físico que pode abrigá-la durante um intervalo de tempo.

Esses conceitos não são intercambiáveis. Trocas de caixa, divisões, movimentações, perdas, inspeções, manutenções e demais fatos relevantes devem preservar o passado em vez de reescrevê-lo.

## Entidades principais

### Meliponário

Unidade de criação onde colônias e caixas são mantidas. O sistema suporta múltiplos meliponários.

Dados principais incluem nome, responsável, localização e observações.

### Espécie

Representa a espécie de abelha sem ferrão associada às colônias.

Dados principais incluem nome popular, nome científico, gênero e observações.

### Colônia

Entidade biológica e histórica que mantém identidade própria mesmo quando muda de caixa ou de localização.

Dados principais incluem código, espécie, meliponário, origem, data de instalação, situação, colônia-mãe e observações.

A entrada no plantel é derivada do próprio cadastro e da origem registrada. Não existe um segundo lançamento de entrada independente apenas para duplicar o mesmo fato.

### Caixa

Objeto físico que abriga uma colônia. Caixa e colônia são entidades distintas.

Dados principais incluem código, meliponário, modelo, material, posição ou localização, situação física e observações.

### Ocupação de caixa

Relaciona uma colônia a uma caixa durante um intervalo de tempo.

Cada ocupação possui início e, quando encerrada, fim. Uma troca de caixa encerra a ocupação anterior e cria outra.

Regras principais:

- uma colônia só pode ocupar uma caixa por vez;
- uma caixa só pode abrigar uma colônia por vez;
- colônia e caixa precisam pertencer ao mesmo meliponário para uma ocupação local;
- uma troca de caixa preserva todos os registros anteriores;
- a data de encerramento não pode invalidar a sequência histórica da ocupação.

### Inspeção

Registro de uma avaliação da colônia em determinada data.

Pode preservar força da colônia, presença de rainha, postura, reservas de alimento, condição das crias, pragas, observações, ações realizadas e próxima inspeção sugerida.

A inspeção guarda o contexto da caixa ocupada na data registrada. Inspeções retroativas resolvem a ocupação histórica correspondente, em vez de assumir a caixa atual.

### Foto de inspeção

Arquivo visual associado a uma inspeção já existente.

A inspeção fornece o vínculo com a colônia e o contexto temporal. A foto não mantém uma segunda referência independente à colônia.

Metadados principais:

- inspeção;
- caminho relativo do arquivo;
- nome original;
- tipo MIME;
- tamanho em bytes;
- data da captura;
- observações.

Regras principais:

- a foto precisa pertencer a uma inspeção existente;
- arquivos são copiados para `media/inspections/<inspection-id>/` no diretório de dados da aplicação;
- o SQLite guarda apenas metadados e o caminho relativo;
- nomes internos usam identificadores próprios para evitar colisões;
- os formatos iniciais aceitos são JPG, PNG e WebP;
- a exclusão protege o caminho para que apenas a área gerenciada de mídia possa ser afetada.

### Alimentação

Registro de alimentação ou suplementação associada a uma colônia.

Preserva data, tipo de alimento, quantidade e unidade quando informadas, resposta observada, observações, próxima alimentação sugerida e a caixa correspondente à data registrada.

Uma alimentação retroativa resolve a ocupação histórica da colônia naquela data.

### Produção

Registro de retirada ou produção associada a uma colônia.

Tipos iniciais incluem mel, pólen, própolis, cera, cerume e outros produtos. O registro preserva quantidade, unidade, finalidade opcional, observações e a caixa histórica da data.

### Evento de colônia

Fato operacional ou biológico registrado manualmente quando não pertence a uma entidade mais específica.

Exemplos iniciais incluem enxameação, abandono, perda de rainha, ataque, praga ou inimigo, recuperação, observação e outros fatos relevantes.

Eventos preservam data, severidade, detalhes e contexto de caixa quando aplicável.

Fatos que possuem entidade própria, como alimentação, produção, movimentação ou manutenção, não devem ser duplicados como eventos genéricos apenas para aparecer na timeline.

### Divisão e multiplicação

Registra uma tentativa de multiplicação de colônia e seu resultado.

Preserva colônia-mãe, data, resultado, caixa histórica da mãe e observações. Quando o resultado cria uma nova colônia, a descendente recebe vínculo genealógico com a origem.

Regras principais:

- resultados bem-sucedidos ou parciais podem criar uma colônia filha na mesma transação;
- a filha preserva espécie e contexto de origem da mãe;
- divisões malsucedidas não criam colônias fantasmas;
- colônias em situação incompatível com multiplicação não podem originar novas divisões.

### Movimentação

Representa entrada, saída, transferência ou transporte de colônias quando há mudança ou registro de trânsito do plantel.

Tipos operacionais atuais incluem:

- transferência interna entre meliponários cadastrados;
- transferência externa para fora do plantel;
- transporte temporário sem alteração do plantel atual.

A movimentação preserva origem, destino e contexto das caixas quando aplicável. Operações que alteram plantel e ocupação são tratadas transacionalmente para evitar estado parcial.

### Documento de movimentação

Evidência ou referência associada a uma movimentação já registrada. Uma movimentação pode possuir zero, um ou vários documentos.

Dados principais:

- tipo do documento;
- número ou referência;
- sistema de origem opcional, como GEDAVE ou GEFAU;
- emissor opcional;
- data de emissão;
- validade opcional;
- caminho de arquivo opcional;
- observações.

Tipos iniciais incluem GTA, autorização, nota fiscal, recibo, declaração, protocolo, certificado e outros.

Regras principais:

- o documento sempre pertence a uma movimentação existente;
- a colônia é derivada da movimentação e não informada separadamente;
- a mesma combinação de movimentação, tipo e referência não é duplicada;
- validade não pode anteceder emissão quando ambas estiverem presentes;
- o sistema registra a evidência, mas não afirma validade jurídica;
- arquivos são referenciados por caminho e não armazenados como BLOB no SQLite.

O campo legado `document_reference` existe apenas por compatibilidade. Referências antigas são normalizadas para a estrutura documental, evitando duas fontes permanentes de verdade.

### Manutenção de caixa

Registro pertencente à caixa física.

Pode preservar data, tipo de manutenção, descrição, responsável, custo, próxima manutenção e a colônia que ocupava a caixa naquela data, quando houver.

Manutenção não é troca de caixa e não altera automaticamente a ocupação. O vínculo histórico com a colônia é contexto, não propriedade do registro.

### Transição de ciclo de vida

Registra mudanças explícitas de situação da colônia.

Transições atuais incluem:

- baixa por perda;
- inativação;
- reativação.

Cada transição preserva situação anterior, nova situação, data, motivo, observações e caixa histórica quando aplicável.

Perda e inativação encerram uma ocupação ativa quando necessário. Reativação não reabre automaticamente a caixa antiga.

## Visões derivadas

### Timeline unificada

A timeline agrega fatos de diferentes fontes sem duplicá-los.

Pode reunir:

- entrada no plantel derivada do cadastro;
- ocupações e trocas de caixa;
- inspeções;
- eventos manuais;
- alimentação;
- produção;
- divisões;
- movimentações;
- manutenção;
- transições de ciclo de vida.

A timeline é uma projeção de leitura. As fontes originais continuam sendo as tabelas e entidades correspondentes.

### Alertas

Alertas de manejo são derivados dos registros mais recentes e não persistidos como um segundo estado independente.

A implementação atual considera, entre outros casos:

- inspeção pendente;
- alimentação pendente;
- colônia fraca.

Registros mais novos substituem pendências anteriores quando representam o mesmo acompanhamento.

### Dashboard

O dashboard é uma visão operacional derivada pelo backend.

Ele combina informações como situação das colônias, força da inspeção mais recente, distribuição por espécie, ocupação de caixas, alertas, produção recente e movimentações recentes.

## Serviços de dados

Backup, exportação, relatório e restauração são serviços operacionais e **não entidades do domínio**.

Eles trabalham sobre o estado persistido sem criar uma segunda fonte de verdade. A restauração é preparada, validada e aplicada de forma controlada, com backup de segurança do estado atual antes da troca.

Consulte [DATA-MANAGEMENT.md](DATA-MANAGEMENT.md).

## Princípios

### Histórico não é sobrescrito

Quando uma mudança possui significado histórico, ela deve gerar um novo registro, uma nova ocupação ou uma transição explícita.

### Saldo é consequência

O estado atual do plantel deve ser derivável dos cadastros, multiplicações, transferências, baixas e demais fatos registrados. Alterações manuais sem histórico devem ser evitadas.

### Genealogia é rastreável

Divisões preservam a relação entre origem e descendentes.

Exemplo:

```text
Jataí-01
└── Jataí-03
    └── Jataí-12
```

### Linguagem simples, dados corretos

A interface pode usar termos familiares ao meliponicultor. O modelo interno preserva nomenclatura técnica, integridade e rastreabilidade.

### Estado derivado não deve virar estado paralelo

Timeline, alertas e dashboard devem ser calculados a partir dos registros reais sempre que possível, evitando duplicação e inconsistência.

## Referências conceituais

Fluxos de plantel, origem, movimentação, transferência, baixa e transporte podem considerar como referência conceitual GEFAU, GEDAVE e GTA do Estado de São Paulo.

Essas referências ajudam a estruturar os dados, mas o MeliponarioManager não pretende reproduzir a burocracia desses sistemas, substituí-los ou decidir sozinho se um documento atende exigências legais vigentes.
