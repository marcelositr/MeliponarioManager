# Operação do repositório no GitHub

Este documento resume o fluxo usado para manter o MeliponarioManager produtivo sem adicionar burocracia desnecessária.

## Fluxo cotidiano

1. registre bugs e melhorias como Issues quando precisarem de acompanhamento;
2. crie uma branch curta a partir de `main`;
3. desenvolva e valide uma única mudança coerente;
4. abra um Pull Request para `main`;
5. aguarde o CI obrigatório;
6. integre por squash merge;
7. remova a branch integrada.

Branches temporárias não são linhas paralelas permanentes. O estado distribuível continua em `main`, e tags imutáveis identificam releases.

## O que cada recurso resolve

- **Issues:** backlog de bugs, melhorias e trabalho que precisa sobreviver a uma conversa;
- **Pull Requests:** revisão, registro da decisão e porta de entrada protegida para `main`;
- **Actions:** CI, testes, builds e bundles sem depender da máquina do mantenedor;
- **Dependabot:** Pull Requests semanais para dependências npm, Cargo e GitHub Actions;
- **Releases:** página pública, notas e instaladores associados a uma tag imutável;
- **Ruleset:** impede exclusão, force push e integração fora do fluxo definido para `main`;
- **Projects:** quadro opcional para organizar muitas Issues; não é necessário enquanto o backlog for pequeno.

## CI e reprodutibilidade

Os lockfiles `package-lock.json` e `src-tauri/Cargo.lock` fazem parte do código da aplicação. O CI usa `npm ci` e comandos Cargo com `--locked`, impedindo que uma validação resolva dependências diferentes das revisadas no Pull Request.

Execuções antigas de CI no mesmo Pull Request são canceladas quando um novo commit chega. Builds de release não são cancelados automaticamente.

## Configuração recomendada em Settings

O ruleset da branch padrão deve manter:

- Pull Request obrigatório;
- apenas squash merge;
- histórico linear;
- resolução obrigatória das conversas de revisão;
- bloqueio de exclusão e force push;
- status check `check` do workflow `CI` obrigatório antes do merge.

Nas configurações gerais do repositório, mantenha apenas squash merge e ative:

- exclusão automática da branch depois do merge;
- permissão para atualizar uma branch de Pull Request atrasada em relação a `main`.

Na área **Security**, habilite quando disponível:

- Dependency graph;
- Dependabot alerts;
- Dependabot security updates;
- private vulnerability reporting.

## Releases

O procedimento completo está em [RELEASES.md](RELEASES.md). Em resumo: a versão é preparada por Pull Request, integrada em `main`, marcada por uma tag `v0.x.y` e transformada em GitHub Pre-release depois da revisão dos bundles.

Nunca mova uma tag pública para esconder uma falha. Corrija em uma nova versão.
