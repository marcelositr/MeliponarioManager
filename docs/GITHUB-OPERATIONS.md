# Operação do repositório no GitHub

Este documento define a rotina de manutenção do MeliponarioManager no GitHub. O objetivo é manter `main` integrável, rastreável e protegida sem criar burocracia desnecessária.

## Fluxo de trabalho

1. registre em uma Issue o trabalho que precisa de acompanhamento;
2. crie uma branch curta a partir da `main` atual;
3. implemente uma única mudança coerente;
4. abra um Pull Request para `main`;
5. atualize a branch caso `main` avance;
6. aguarde o status check obrigatório `check`;
7. resolva conversas de revisão;
8. integre por squash merge;
9. remova a branch temporária.

Mudanças pequenas e autocontidas podem começar diretamente em uma branch e ser explicadas no Pull Request, sem Issue separada.

## Responsabilidade de cada recurso

| Recurso | Uso |
| --- | --- |
| Issues | Bugs, melhorias e trabalho que precisa permanecer no backlog |
| Pull Requests | Revisão, decisão técnica e porta de entrada para `main` |
| Actions | CI, build dos bundles e evidência automatizada |
| Dependabot | Atualizações semanais de npm, Cargo e GitHub Actions |
| Releases | Notas e instaladores associados a uma tag imutável |
| Ruleset | Proteção da branch padrão e exigência do fluxo de integração |
| Projects | Organização opcional quando o volume do backlog justificar |

## Branches e Pull Requests

Use branches temporárias com os prefixos definidos em [CONTRIBUTING.md](../CONTRIBUTING.md). Não mantenha branches de desenvolvimento paralelas por tempo indeterminado.

O Pull Request deve declarar:

- problema e solução;
- validações executadas;
- impacto no domínio, schema e dados;
- documentação alterada;
- limitações ou riscos conhecidos.

O título deve ser adequado para virar a mensagem principal do squash commit.

## Proteção de `main`

O ruleset esperado para a branch padrão exige:

- Pull Request antes da atualização de `main`;
- status check `check` do workflow `CI`;
- branch atualizada com o topo de `main`;
- resolução das conversas de revisão;
- histórico linear;
- bloqueio de exclusão e force push.

A criação de uma nova branch pode ser permitida sem status prévio; a proteção é aplicada quando ela tenta atualizar a referência protegida.

Nas configurações gerais do repositório:

- permita apenas squash merge;
- sugira a atualização de branches atrasadas;
- exclua automaticamente a branch depois do merge;
- mantenha auto-merge desativado enquanto as atualizações ainda exigirem avaliação manual frequente.

Alterações nessas regras devem ser refletidas neste documento.

## CI e dependências reproduzíveis

Os lockfiles `package-lock.json` e `src-tauri/Cargo.lock` são versionados. O CI usa `npm ci` e comandos Cargo com `--locked`; manifests e lockfiles divergentes causam falha em vez de resolver versões não revisadas.

O workflow `CI` roda em Pull Requests e pushes para `main`. Novos commits cancelam execuções obsoletas da mesma referência. Builds de release não são cancelados automaticamente.

## Dependabot

O Dependabot abre grupos semanais para:

- dependências npm;
- crates Cargo;
- GitHub Actions.

### Triagem

1. confirme se a atualização é `PATCH`, `MINOR` ou `MAJOR`;
2. leia mudanças incompatíveis da dependência quando a atualização for relevante;
3. confira os manifests, lockfiles e o diff gerado;
4. use o CI como evidência, não como substituto da revisão;
5. faça squash merge apenas quando o impacto estiver entendido.

Se o CI falhar por incompatibilidade real, adapte o código na própria branch e adicione cobertura para o comportamento corrigido. Depois de uma alteração manual, o Dependabot deixa de garantir a manutenção automática de conflitos nessa branch.

Comandos como `@dependabot recreate` podem sobrescrever correções manuais. Use-os somente quando descartar essas alterações for intencional.

## Segurança

Mantenha habilitados, quando disponíveis:

- Dependency graph;
- Dependabot alerts;
- Dependabot security updates;
- private vulnerability reporting.

Vulnerabilidades não devem ser discutidas com detalhes exploráveis em Issues públicas. Consulte [SECURITY.md](../SECURITY.md).

## Releases

A versão é preparada em Pull Request, integrada em `main`, marcada por uma tag `v0.x.y` e publicada como GitHub Pre-release depois da revisão dos bundles.

Nunca mova uma tag pública para ocultar falha de empacotamento ou de produto. Prepare uma nova versão conforme [RELEASES.md](RELEASES.md).
