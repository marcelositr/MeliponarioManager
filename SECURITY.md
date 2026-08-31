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

## Verificação automatizada de dependências

O repositório usa Dependabot para acompanhar atualizações de npm, Cargo e GitHub Actions. Além disso, o workflow independente `Dependency security audit` verifica os lockfiles atuais:

- `npm audit --audit-level=high` para as dependências Node registradas em `package-lock.json`;
- `cargo-audit` para `src-tauri/Cargo.lock`, usando a base de advisories RustSec.

O workflow roda quando os manifests ou lockfiles correspondentes mudam em um Pull Request para `main`, quando essas dependências mudam em `main`, semanalmente para detectar advisories publicados depois da última alteração de dependências e manualmente quando uma verificação adicional for necessária.

O audit de segurança é separado do status obrigatório `check`, para não alongar o ciclo normal de mudanças sem dependências. Uma falha nesse workflow deve ser tratada como sinal de segurança a investigar antes de uma nova release.

## Escopo do produto

O aplicativo é local-first e não opera um serviço web próprio. Ele não substitui controles de acesso do sistema operacional, criptografia de disco, backup externo ou sistemas oficiais de rastreabilidade.
