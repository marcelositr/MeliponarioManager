# Política de segurança

## Versões suportadas

O MeliponarioManager permanece experimental na série `0.x`. Correções de segurança são destinadas somente à versão pública mais recente.

| Versão | Suporte |
| --- | --- |
| Última pre-release | Sim |
| Versões anteriores | Não |

## Como relatar uma vulnerabilidade

Não abra uma issue pública com detalhes que permitam explorar a falha, expor dados locais ou corromper backups.

Use **Security > Advisories > Report a vulnerability** neste repositório. Se essa opção não estiver disponível, abra uma issue contendo apenas um pedido de contato privado, sem prova de conceito, dados reais, caminhos locais ou detalhes de exploração.

Inclua no relato privado, quando aplicável:

- versão do MeliponarioManager e sistema operacional;
- impacto esperado nos dados ou arquivos locais;
- passos mínimos para reprodução com dados fictícios;
- possível mitigação;
- informação sobre divulgação prévia.

Relatos relevantes incluem, entre outros, leitura ou escrita fora da área gerenciada da aplicação, restauração insegura de backup, manipulação indevida de caminhos, importação maliciosa de arquivos e corrupção de dados causada por entrada não confiável.

O recebimento será confirmado assim que possível. A análise pode resultar em correção, mitigação documentada ou encerramento fundamentado quando o comportamento não representar uma vulnerabilidade.

## Escopo do produto

O aplicativo é local-first e não opera um serviço web próprio. Ele não substitui controles de acesso do sistema operacional, criptografia de disco, backup externo ou sistemas oficiais de rastreabilidade.
