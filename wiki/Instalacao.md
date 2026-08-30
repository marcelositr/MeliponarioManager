# Instalação

O MeliponarioManager é distribuído como aplicação desktop para **Linux** e **Windows**.

> O projeto ainda está na série experimental `0.x`. Faça backup dos seus dados antes de atualizar para uma nova versão.

## Formatos disponíveis

### Linux

Os builds de distribuição podem ser disponibilizados em:

- `.deb`;
- AppImage.

Use o formato mais adequado à sua distribuição. O AppImage é portátil; o pacote `.deb` integra a instalação ao sistema em distribuições compatíveis.

### Windows

Os builds podem ser disponibilizados como:

- instalador NSIS;
- instalador MSI.

Os instaladores Windows do período experimental podem não possuir assinatura de código. O próprio sistema operacional pode, portanto, exibir avisos adicionais antes da instalação.

## Onde obter uma versão

Use a seção **Releases** do repositório oficial do MeliponarioManager. As versões experimentais são publicadas como **Pre-release**.

Evite instalar executáveis redistribuídos por fontes desconhecidas.

## Primeira abertura

Na primeira execução, a aplicação prepara automaticamente seu armazenamento local e o banco de dados necessário.

Depois disso, siga [Primeiros passos](Primeiros-passos) para criar o primeiro meliponário, cadastrar espécies, caixas e colônias.

## Atualização

Antes de atualizar:

1. abra a área **Dados**;
2. crie um [backup completo](Backup-e-restauracao);
3. guarde a cópia em local diferente do diretório de trabalho da aplicação;
4. instale a nova versão;
5. abra a aplicação e confira os registros principais.

As atualizações podem executar migrações internas do banco automaticamente. Não altere manualmente os arquivos do banco para tentar adaptar versões.

## Problemas de instalação

Se a aplicação não abrir, apresentar falha gráfica ou o instalador for bloqueado, consulte [Solução de problemas](Solucao-de-problemas).

---

[← Home](Home) · [Próximo: Conceitos básicos →](Conceitos-basicos)