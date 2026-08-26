# Contribuindo com o MeliponarioManager

Este projeto utiliza um fluxo simples baseado em branches curtas e Pull Requests.

## Branches

Use nomes objetivos:

- `feat/...` para novas funcionalidades;
- `fix/...` para correções;
- `docs/...` para documentação;
- `refactor/...` para refatorações;
- `test/...` para testes;
- `chore/...` para manutenção do projeto.

Exemplos:

- `feat/colony-inspections`
- `fix/feeding-history`
- `docs/domain-model`

## Pull Requests

Evite trabalhar diretamente na `main` para alterações relevantes.

Cada Pull Request deve:

- resolver um objetivo claro;
- evitar misturar funcionalidades não relacionadas;
- explicar o que mudou e por quê;
- indicar testes realizados quando aplicável;
- manter compatibilidade com o modelo de domínio existente ou documentar explicitamente qualquer mudança.

## Commits

Preferimos mensagens curtas no estilo Conventional Commits:

- `feat: add colony inspection flow`
- `fix: preserve colony history on box transfer`
- `docs: document semantic versioning policy`
- `refactor: separate colony and hive box models`

O uso é uma convenção do projeto, não uma dependência rígida de ferramenta.

## Versionamento

O projeto segue Semantic Versioning no formato `vMAJOR.MINOR.PATCH`.

Enquanto permanecer experimental:

- novas funcionalidades compatíveis incrementam `MINOR`;
- correções compatíveis incrementam `PATCH`;
- o projeto permanecerá em `0.x` e não possui meta de chegar à versão `1.0.0`.

Exemplos:

- `v0.1.0`
- `v0.1.1`
- `v0.2.0`

## Modelo de domínio

A rastreabilidade é prioridade. Evite sobrescrever fatos históricos quando o correto for registrar um novo evento.

Exemplo: trocar uma colônia de caixa deve gerar uma movimentação/histórico, não apagar a caixa anterior como se ela nunca tivesse existido.

A interface pode usar linguagem popular e direta, mas o domínio e os dados devem permanecer tecnicamente consistentes.
