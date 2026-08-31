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

### Avisos transitivos do stack Linux

O stack Linux atual do Tauri 2 ainda depende das bindings GTK3 da série `0.18`. Por isso, o `cargo-audit` pode exibir warnings RustSec de crates transitivas marcadas como `unmaintained` e o advisory de unsoundness de `glib 0.18.5` (`RUSTSEC-2024-0429`) sem considerar o audit como falho.

Esses warnings são acompanhados como dívida upstream. Não deve ser aplicado override manual para misturar `glib` ou bindings GTK de gerações incompatíveis apenas para silenciar o scanner. A correção deve ocorrer por atualização compatível do stack Tauri/GTK quando ela estiver disponível e validada no projeto.

Um resultado verde do job RustSec significa que não foi encontrada vulnerabilidade que o `cargo-audit` trate como falha. Ele não significa ausência total de warnings informativos ou de manutenção.

## Escopo do produto

O aplicativo é local-first e não opera um serviço web próprio. Ele não substitui controles de acesso do sistema operacional, criptografia de disco, backup externo ou sistemas oficiais de rastreabilidade.
