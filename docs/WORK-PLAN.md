# Plano de trabalho contínuo

Este arquivo é o ponto de continuidade do trabalho prático no MeliponarioManager.

Ele existe para que o desenvolvimento possa continuar entre conversas diferentes sem depender do histórico do chat. Toda sessão relevante deve atualizar este documento antes de encerrar uma etapa importante.

## Branch de trabalho

`work/field-testing-and-hardening`

Base inicial: `main` após o merge do PR #46 (`7640c69`).

## Objetivo desta fase

Sair do ciclo em que a aplicação foi majoritariamente construída e validada por automação e entrar em uma fase orientada por uso real:

- testar a aplicação manualmente com dados e fluxos reais;
- registrar qualquer comportamento estranho, regressão ou atrito observado;
- corrigir os problemas encontrados sem perder contratos já estabilizados;
- fortalecer testes automatizados sempre que um bug real revelar uma lacuna;
- melhorar a experiência de uso somente a partir de problemas observados;
- manter integridade, rastreabilidade e recuperação de dados como prioridades máximas.

## Regras de trabalho

- Não alterar migrations existentes.
- Mudanças de schema novas exigem migration nova e justificativa explícita.
- Não realizar restauração destrutiva contra a base real do usuário durante testes.
- Antes de corrigir um bug reproduzível, registrar o comportamento e, quando viável, adicionar teste de regressão.
- Preservar contratos IPC e formatos persistidos salvo decisão explícita registrada neste arquivo.
- Atualizar este plano ao concluir blocos relevantes de trabalho.
- Manter uma seção de continuidade suficientemente clara para outra conversa retomar o trabalho sem depender do chat anterior.

## Estado atual

- [x] PR #46 integrado na `main`.
- [x] Dívida técnica principal de backend/frontend reduzida.
- [x] Smoke desktop real via WebDriver adicionado ao pipeline da `main`.
- [x] Branch de trabalho prático criada.
- [x] Plano persistente de trabalho criado.
- [ ] Iniciar rodada sistemática de testes manuais.

## Fase 1 — Teste manual orientado por uso real

### Inicialização e navegação

- [ ] Iniciar a aplicação com a base de dados real existente.
- [ ] Confirmar carregamento normal da Visão geral.
- [ ] Percorrer todas as páginas principais sem mutações destrutivas.
- [ ] Registrar erros visuais, falhas de carregamento, travamentos ou mensagens inadequadas.
- [ ] Verificar comportamento em resolução compatível com o ambiente real de uso.

### Cadastros e estados principais

- [ ] Revisar meliponários.
- [ ] Revisar espécies.
- [ ] Revisar caixas.
- [ ] Revisar colônias.
- [ ] Verificar buscas, filtros, estados vazios e navegação entre registros.

### Manejo

- [ ] Testar inspeções.
- [ ] Testar alimentação.
- [ ] Testar produção.
- [ ] Testar eventos.
- [ ] Testar manutenção.
- [ ] Observar especialmente validações, mensagens de erro e dados exibidos após salvar.

### Agenda

- [ ] Revisar Agenda e alertas com dados existentes.
- [ ] Criar item manual descartável quando houver contexto seguro.
- [ ] Verificar conclusão/cancelamento conforme os fluxos atuais.
- [ ] Confirmar projeções derivadas e reconciliação sem duplicações aparentes.

### Movimentações e transporte

- [ ] Revisar histórico de movimentações.
- [ ] Revisar ciclo de transporte quando houver dados adequados.
- [ ] Verificar documentos e estados derivados.
- [ ] Confirmar que nenhuma ação permitida viola o estado atual da colônia/caixa.

### Fotos, anexos e arquivos gerenciados

- [ ] Abrir pelo menos uma foto existente.
- [ ] Abrir ou revelar pelo menos um arquivo gerenciado.
- [ ] Verificar estados de arquivo ausente ou inconsistente quando houver exemplo seguro.
- [ ] Confirmar que erros de sistema de arquivos chegam à UI de forma útil e sem vazamento técnico indevido.

### Relatórios e CSV

- [ ] Abrir cada relatório com dados reais.
- [ ] Verificar filtros e estados vazios.
- [ ] Exportar CSV pelo seletor nativo.
- [ ] Confirmar nome e destino escolhidos.
- [ ] Abrir o CSV resultante e conferir estrutura básica.
- [ ] Testar impressão quando houver ambiente adequado.

### Backup e recuperação

- [ ] Criar backup completo.
- [ ] Confirmar que o caminho criado é informado corretamente.
- [ ] Verificar existência física do backup.
- [ ] Inspecionar diagnóstico/validação de backup quando disponível.
- [ ] Preparar posteriormente uma cópia descartável para teste de restauração.
- [ ] Não restaurar sobre a base real durante esta fase inicial.

## Fase 2 — Correções guiadas pelos testes

Para cada problema real encontrado:

1. registrar na seção **Achados**;
2. definir severidade e impacto;
3. localizar a causa;
4. criar ou ampliar teste de regressão quando viável;
5. corrigir com a menor mudança coerente possível;
6. rodar validações afetadas;
7. marcar o achado como resolvido e registrar commit/PR correspondente.

## Fase 3 — Hardening após os testes

- [ ] Ampliar cenários de migrations a partir dos casos reais encontrados.
- [ ] Ampliar testes de backup/restore com bases descartáveis.
- [ ] Revisar integridade histórica e invariantes reveladas pelo uso real.
- [ ] Revisar UX de operações repetitivas com base em atritos observados.
- [ ] Revisar acessibilidade por teclado nos fluxos realmente utilizados.
- [ ] Ampliar smoke/E2E desktop apenas para cenários que tragam cobertura útil e estável.

## Achados

Nenhum achado registrado ainda.

Formato sugerido para novos achados:

### ACH-XXX — Título curto

- **Status:** aberto | investigando | corrigido | validado
- **Severidade:** crítica | alta | média | baixa
- **Área:**
- **Como reproduzir:**
- **Comportamento observado:**
- **Comportamento esperado:**
- **Causa:**
- **Correção:**
- **Teste de regressão:**
- **Commit/PR:**

## Decisões de trabalho

### 2026-08-31 — Início da fase prática

A partir desta branch, o desenvolvimento passa a ser guiado prioritariamente por testes manuais e uso real da aplicação. Automação continua sendo uma barreira de segurança e regressão, mas não substitui validação humana dos fluxos.

O trabalho será mantido neste arquivo para permitir continuidade entre conversas e reduzir dependência do hardware local do usuário.

## Próximo passo

Executar a primeira rodada de teste manual começando por **inicialização e navegação**, registrando imediatamente qualquer achado antes de iniciar correções.

## Handoff para a próxima conversa

Se uma nova conversa precisar continuar este trabalho:

1. abrir `docs/WORK-PLAN.md` na branch `work/field-testing-and-hardening`;
2. ler **Estado atual**, **Achados**, **Decisões de trabalho** e **Próximo passo**;
3. conferir os commits mais recentes da branch;
4. continuar do primeiro item não concluído;
5. atualizar este arquivo antes de encerrar uma etapa importante.
