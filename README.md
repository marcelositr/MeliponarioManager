# MeliponarioManager

Sistema experimental para gerenciamento de meliponários, com foco em colônias, caixas, inspeções, manejo, alimentação, produção, divisões, movimentações, histórico e rastreabilidade do plantel.

## Status do projeto

> Projeto experimental e em desenvolvimento contínuo.

O MeliponarioManager utiliza Versionamento Semântico (`vMAJOR.MINOR.PATCH`), mas permanecerá na série `0.x`. Não há previsão de uma versão `1.0.0`, pois o projeto é tratado como um laboratório permanente de evolução e testes.

## Tecnologias

A base inicial utiliza:

- Rust;
- Tauri 2;
- React + TypeScript;
- Vite;
- SQLite;
- SQLx.

A aplicação é desktop e local-first. O frontend não acessa o banco diretamente: persistência e regras de domínio ficam no backend Rust.

## Interface operacional atual

A primeira interface de operação já permite trabalhar com a base do plantel sem acessar o banco manualmente:

- navegar entre visão geral, meliponários, espécies, colônias e caixas;
- cadastrar e listar meliponários;
- cadastrar e listar espécies;
- cadastrar e listar caixas físicas;
- cadastrar e listar colônias;
- relacionar colônia-mãe quando aplicável;
- colocar ou mover uma colônia para uma caixa livre usando o histórico de ocupação já existente;
- acompanhar na tela a caixa atual, o meliponário, a espécie e a situação de cada colônia.

A interface usa componentes próprios e chamadas Tauri diretas, sem adicionar um framework de UI pesado.

## Objetivos

O sistema deve permitir, entre outras operações:

- cadastrar meliponários, espécies, colônias e caixas;
- registrar origem, instalação e localização das colônias;
- registrar inspeções e acompanhar a condição de cada colônia;
- registrar alimentação e suplementação;
- registrar divisões, multiplicações e genealogia;
- registrar produção de mel, pólen, própolis, cera e outros produtos;
- registrar eventos como enxameação, abandono, perda de rainha, ataques e transferências;
- registrar movimentações com documentos e referências de rastreabilidade quando aplicável;
- registrar manutenção e troca de caixas;
- armazenar fotos associadas às inspeções;
- gerar alertas de manejo e acompanhamento;
- manter histórico completo e rastreável do plantel;
- apresentar um dashboard com a situação geral do meliponário.

## Filosofia de domínio

A colônia é tratada como uma entidade histórica. Uma troca de caixa, divisão, transferência ou baixa não apaga o que aconteceu anteriormente.

A entrada no plantel é derivada do próprio cadastro e da origem da colônia, sem criar um segundo registro redundante. Perdas, inativações e reativações são transições explícitas, preservando status anterior, novo status, data, motivo e o contexto da caixa quando aplicável.

A caixa física e a colônia são entidades distintas: uma mesma colônia pode ocupar caixas diferentes ao longo do tempo.

Manutenções pertencem à caixa física e preservam o contexto da colônia que a ocupava na data registrada, quando houver. Assim, reparar ou limpar uma caixa não é tratado como troca de caixa nem como evento genérico da colônia.

Fotos de inspeção pertencem à inspeção que lhes dá contexto. O SQLite armazena apenas metadados e um caminho relativo; o arquivo é copiado para o diretório de dados da aplicação em `media/inspections/`. A colônia da foto é sempre derivada da inspeção, evitando um segundo vínculo independente. A primeira versão aceita JPG, PNG e WebP.

Documentos de movimentação pertencem ao registro de movimentação e funcionam como evidência de rastreabilidade. Uma movimentação pode possuir vários documentos ou referências, com tipo, número, sistema de origem, emissor, emissão, validade, caminho de arquivo e observações. O sistema registra esses dados, mas não presume nem certifica validade jurídica.

Referências simples gravadas pelo campo legado `document_reference` são normalizadas automaticamente para a estrutura documental, evitando duas fontes permanentes de verdade.

Os alertas de manejo são derivados dos registros mais recentes de inspeção, alimentação e condição da colônia. Eles não são persistidos como um segundo estado independente, evitando alertas desatualizados quando um novo manejo substitui uma pendência anterior.

A modelagem de movimentações e rastreabilidade considera como referência conceitual os fluxos utilizados por GEFAU, GEDAVE e GTA no Estado de São Paulo, sem transformar o MeliponarioManager em uma cópia desses sistemas ou impor burocracia oficial ao uso cotidiano.

## Desenvolvimento

Pré-requisitos principais:

- Node.js 22 ou compatível com Vite 8;
- Rust 1.94 ou superior;
- dependências de sistema exigidas pelo Tauri para a plataforma utilizada.

Instale as dependências do frontend:

```bash
npm install
```

Execute apenas o frontend:

```bash
npm run dev
```

Execute a aplicação desktop:

```bash
npm run tauri dev
```

O banco SQLite é criado automaticamente no diretório de dados da aplicação e recebe as migrations durante a inicialização. Fotos importadas pelo backend são mantidas no subdiretório `media/inspections/` do mesmo diretório de dados.

## Fluxo de desenvolvimento

- `main`: estado integrado do projeto;
- branches curtas para cada alteração;
- alterações relevantes entram por Pull Request;
- versões são identificadas por tags SemVer no formato `v0.x.y`;
- correções incrementam `PATCH`;
- novas funcionalidades compatíveis incrementam `MINOR`.

Mais detalhes em [CONTRIBUTING.md](CONTRIBUTING.md).

## Licença

Distribuído sob a licença MIT. Consulte [LICENSE](LICENSE).
