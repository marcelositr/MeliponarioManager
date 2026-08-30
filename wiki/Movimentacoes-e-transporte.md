# Movimentações e transporte

O MeliponarioManager diferencia mudanças reais de plantel de transportes temporários.

## Transferência interna

Representa uma mudança entre meliponários cadastrados quando a colônia efetivamente passa para outra unidade.

Use o fluxo específico para que meliponário, ocupação e histórico sejam atualizados de forma consistente.

## Transferência externa

Representa a saída da colônia para fora do plantel acompanhado pela aplicação.

Essa operação possui impacto no estado da colônia e não deve ser registrada como simples transporte temporário.

## Transporte temporário

O transporte temporário registra uma saída transitória **sem mudar automaticamente**:

- o meliponário atual;
- a caixa atual;
- a ocupação;
- o saldo do plantel;
- a situação administrativa da colônia.

### Transporte aberto

Enquanto não houver retorno registrado, o transporte permanece aberto.

Nesse estado você pode registrar o retorno ou, quando permitido, anular o movimento pelo fluxo administrativo correspondente.

### Retorno

O retorno conclui o transporte temporário mantendo o movimento original no histórico.

A data de retorno não pode ser anterior à saída.

### Reabertura

Se um retorno tiver sido registrado incorretamente, o transporte pode ser reaberto quando o fluxo permitir. A reabertura não apaga o retorno anterior: ela preserva o histórico e exige motivo.

## Documentos de movimentação

Uma movimentação pode receber documentos ou referências, como:

- GTA;
- autorização;
- nota fiscal;
- recibo;
- declaração;
- protocolo;
- certificado;
- outros documentos pertinentes.

O MeliponarioManager registra essas informações para rastreabilidade, mas **não substitui sistemas oficiais e não certifica a validade jurídica de um documento**.

## Dica prática

Antes de registrar uma movimentação, pergunte:

> A colônia está mudando de plantel/local cadastrado ou apenas sairá e retornará?

Se vai sair e retornar sem alterar seu vínculo atual, normalmente o conceito apropriado é **transporte temporário**.

---

[← Agenda e alertas](Agenda-e-alertas) · [Próximo: Fotos e arquivos →](Fotos-e-arquivos)