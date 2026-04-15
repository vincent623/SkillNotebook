# 2x2 Matrix Evaluation Framework

## Two Dimensions

> **用户视角**：第一个维度——就算不给 AI 任何技能，它自己能把这件事做到什么程度？（模型能力）第二个维度——技能作者写进去的工作流程、判断标准、领域知识，含金量有多高？（人类经验）

### Dimension 1: Model Capability Dependence

- **strong** (model is capable): Code generation, complex reasoning, architecture, API integration, multi-step workflows. Skill value = guiding direction and consistency.
- **weak** (model limited): Fixed templates, domain checklists, specific conventions, structured formats. Skill value = supplementing knowledge the model lacks.

### Dimension 2: Human Practice Quality

- **best** (high-quality practice): Error handling, security considerations, best practice references, rich examples, complete docs with references/.
- **weak** (thin content): Simple instructions, few examples, no error handling, short docs, no references.

## Four Quadrants

| Quadrant | Model | Practice | Strategy | Expectation emphasis |
|----------|:-----:|:--------:|----------|---------------------|
| **scaffolding** | weak | weak | assertion | Heavy **structure** |
| **leverage** | strong | weak | delta | Heavy **differential** |
| **codification** | weak | best | reference | Heavy **content** |
| **mastery** | strong | best | comparison | **Balanced** |

### Strategy descriptions

- **assertion** (scaffolding): Structured assertions verify output matches template. Focus: structural completeness > content quality.
- **delta** (leverage): Compare with_skill vs without_skill. Quantify incremental value. Focus: larger delta = more valuable.
- **reference** (codification): Compare against reference answers. Calculate deviation. Focus: accuracy > completeness.
- **comparison** (mastery): Blind A/B evaluation (auxiliary). Core measurement is still pass rate delta. Focus: balanced quality.

## Classification Thresholds

- Model capability score ≥ 50 → strong, < 50 → weak
- Practice quality score ≥ 50 → best, < 50 → weak
- Confidence < 0.6 → launch classifier agent for assisted confirmation
