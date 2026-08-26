# MeliponarioManager

Sistema experimental para gerenciamento de meliponários, com foco em colônias, caixas, inspeções, manejo, alimentação, produção, divisões, movimentações, histórico e rastreabilidade do plantel.

## Status do projeto

> Projeto experimental e em desenvolvimento contínuo.

O MeliponarioManager utiliza Versionamento Semântico (`vMAJOR.MINOR.PATCH`), mas permanecerá na série `0.x`. Não há previsão de uma versão `1.0.0`, pois o projeto é tratado como um laboratório permanente de evolução e testes.

## Objetivos

O sistema deve permitir, entre outras operações:

- cadastrar meliponários, espécies, colônias e caixas;
- registrar origem, instalação e localização das colônias;
- registrar inspeções e acompanhar a condição de cada colônia;
- registrar alimentação e suplementação;
- registrar divisões, multiplicações e genealogia;
- registrar produção de mel, pólen, própolis, cera e outros produtos;
- registrar eventos como enxameação, abandono, perda de rainha, ataques e transferências;
- registrar manutenção e troca de caixas;
- armazenar fotos associadas às inspeções;
- gerar alertas de manejo e acompanhamento;
- manter histórico completo e rastreável do plantel;
- apresentar um dashboard com a situação geral do meliponário.

## Filosofia de domínio

A colônia é tratada como uma entidade histórica. Uma troca de caixa, divisão, transferência ou baixa não apaga o que aconteceu anteriormente.

A caixa física e a colônia são entidades distintas: uma mesma colônia pode ocupar caixas diferentes ao longo do tempo.

A modelagem de movimentações e rastreabilidade considera como referência conceitual os fluxos utilizados por GEFAU, GEDAVE e GTA no Estado de São Paulo, sem transformar o MeliponarioManager em uma cópia desses sistemas ou impor burocracia oficial ao uso cotidiano.

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
