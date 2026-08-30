# Conceitos básicos

Entender alguns conceitos antes de começar evita registros duplicados e mantém o histórico confiável.

## Meliponário

É a unidade de criação onde caixas e colônias são mantidas. O sistema suporta mais de um meliponário.

Você pode selecionar um meliponário como **contexto ativo**. Esse contexto filtra várias telas, como Agenda, alertas, colônias, caixas e manejos, sem alterar os dados cadastrados.

## Espécie

Representa a espécie de abelha sem ferrão associada à colônia. O cadastro pode reunir nome popular, nome científico, gênero e observações.

## Colônia

A **colônia** representa a unidade biológica e histórica.

Ela mantém sua identidade mesmo quando:

- muda de caixa;
- muda de posição;
- é transferida entre meliponários;
- passa por inspeções e alimentações;
- origina outra colônia por divisão;
- sofre uma alteração de ciclo de vida.

## Caixa

A **caixa** é o objeto físico que pode abrigar uma colônia.

Uma caixa pode estar livre, ocupada ou indisponível para nova ocupação conforme seu estado físico. Manutenções pertencem à caixa, não à identidade biológica da colônia.

## Colônia não é caixa

Esta é a regra mais importante do sistema.

Quando uma colônia muda da caixa `CX-01` para a `CX-08`, você não cria uma nova colônia só por causa da troca. O MeliponarioManager encerra a ocupação anterior e preserva a nova ocupação dentro do histórico da mesma colônia.

Isso permite responder depois perguntas como:

- Em quais caixas esta colônia já esteve?
- Qual caixa ela ocupava durante determinada inspeção?
- Qual colônia ocupava uma caixa quando houve uma manutenção?

## Fato histórico e planejamento

O sistema também separa **o que aconteceu** de **o que precisa acontecer**.

Exemplo:

- uma inspeção registrada é um fato histórico;
- a próxima inspeção sugerida pode gerar uma tarefa na Agenda;
- a tarefa não significa que a inspeção já aconteceu.

Veja [Agenda e alertas](Agenda-e-alertas).

## Registros atuais e histórico

O MeliponarioManager procura preservar os fatos anteriores em vez de sobrescrever o passado. Por isso, várias operações de correção, reversão e mudança de estado mantêm rastreabilidade.

Ao cadastrar ou corrigir algo, prefira sempre a operação específica disponível na aplicação em vez de tentar recriar manualmente um fato anterior.

---

[← Instalação](Instalacao) · [Próximo: Primeiros passos →](Primeiros-passos)