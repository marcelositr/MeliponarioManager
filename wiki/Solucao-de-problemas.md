# Solução de problemas

Esta página reúne verificações seguras para problemas comuns.

## A aplicação não abre

1. confirme que instalou um pacote correspondente ao seu sistema;
2. tente abrir novamente após encerrar instâncias anteriores;
3. verifique se o sistema operacional exibiu algum bloqueio ou aviso;
4. se o problema começou após uma atualização, preserve seus backups antes de qualquer tentativa de reinstalação.

## A interface apresenta problema gráfico no Linux

Algumas combinações de driver gráfico, desktop e WebKitGTK podem apresentar problemas de renderização.

Se a aplicação funcionava anteriormente, registre:

- distribuição e versão;
- ambiente gráfico;
- modelo/driver da GPU;
- versão do MeliponarioManager;
- o que aparece na tela.

Evite definir variáveis de ambiente globalmente ou alterar arquivos do sistema sem saber qual problema está sendo diagnosticado.

## Uma foto ou anexo aparece como “Arquivo não encontrado”

Use a área **Dados → Verificar arquivos**.

A aplicação preserva os metadados quando o arquivo físico está ausente. Se você possui um backup completo anterior, ele pode ajudar na recuperação.

## Meu relatório não mostra registros

Confira:

- intervalo de datas;
- meliponário selecionado;
- filtros adicionais;
- se os fatos foram anulados/revertidos;
- se você está consultando o relatório correto.

Um relatório vazio também pode representar corretamente um período sem movimentação.

## Uma tarefa está atrasada

Atraso significa que a tarefa ainda está pendente e sua data programada já passou.

Abra a Agenda e decida se deve:

- executar o manejo;
- reagendar;
- cancelar;
- ignorar, quando apropriado.

Use motivo quando a aplicação solicitar. Isso preserva a rastreabilidade.

## Registrei algo errado

Não tente “compensar” criando outro fato artificial.

Procure a função apropriada de:

- correção;
- anulação;
- reversão;
- reabertura;
- mudança de estado.

Esses fluxos existem para manter o histórico coerente.

## Antes de reinstalar

Se a aplicação ainda abre, faça um [backup completo](Backup-e-restauracao).

Se não abre, não apague manualmente o diretório de dados antes de copiar o material existente para um local seguro.

## Como relatar um problema

Ao abrir uma Issue pública, informe:

- versão do MeliponarioManager;
- sistema operacional;
- formato instalado;
- passos para reproduzir;
- resultado esperado e observado.

Remova dados pessoais, caminhos sensíveis e informações reais desnecessárias. Não publique banco de dados ou backup contendo seus registros.

---

[← Backup](Backup-e-restauracao) · [Próximo: FAQ →](FAQ)