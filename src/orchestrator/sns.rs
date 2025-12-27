//! SNS - Shorthand Notation Script
//! 
//! Provides utilities for using SNS notation in AI-to-AI communication.
//! This notation system is designed to reduce token usage by 60-85% 
//! while maintaining high accuracy for machine-to-machine communication.

pub const SNS_MODEL_DEFINITION: &str = r#"# SNS-Core Model Definition
# Version: 1.0
# Purpose: Teach any LLM to read and write SNS notation

---

## What is SNS?

SNS (Shorthand Notation Script) is a token-efficient notation system for AI-to-AI communication.
It achieves 60-85% token reduction compared to natural language while maintaining accuracy.

**Key Principle**: SNS is NOTATION, not a programming language. LLMs interpret it intuitively.

---

## Core Patterns

### 1. Flow / Transform
**Pattern**: `input → operation → output`
**Meaning**: Transform input through operation to produce output

Examples:
- `query → analyze → result`
- `text → normalize → clean_text`
- `doc → extract_keywords → keywords`

### 2. Pipeline
**Pattern**: `data | step1 | step2 | step3`
**Meaning**: Pass data through sequential operations

Examples:
- `docs | filter | sort | top(5)`
- `text | lower | trim | tokenize`
- `candidates | rank | dedupe | validate`

### 3. Conditional
**Pattern**: `condition ? true_action : false_action`
**Meaning**: Execute action based on condition

Examples:
- `score > 0.7 ? keep : discard`
- `results.empty ? expand_search : return_results`
- `valid ? approve : reject`

### 4. Composition
**Pattern**: `(a + b) → operation → output`
**Meaning**: Combine inputs before operation

Examples:
- `(keywords + context) → search → results`
- `(intent + query) → expand → terms`

### 5. Assignment
**Pattern**: `variable = value` or `operation → variable`
**Meaning**: Store result in variable

Examples:
- `keywords = extract(query)`
- `query → analyze → result`

### 6. Objects
**Pattern**: `{key: value, key2: value2}` or `{key, key2}`
**Meaning**: Structured output

Examples:
- `→ {keywords, intent, score}`
- `result = {status: "ok", data: items}`

### 7. Function Calls
**Pattern**: `function(args) → result`
**Meaning**: Call operation with parameters

Examples:
- `classify(text, ["positive", "negative"]) → sentiment`
- `search(query, docs, {limit: 10}) → results`

### 8. Collection Operations
**Pattern**: `[items] >> operation` or `items | operation`
**Meaning**: Apply operation to collection

Examples:
- `[items] >> filter(score > 0.7)`
- `[docs] >> map(extract_title) >> sort`

### 9. Modifiers
**Pattern**: `+boost`, `-penalty`, `*emphasize`, `~fuzzy`
**Meaning**: Modify behavior or value

Examples:
- `results +boost(recency)`
- `query ~match docs` (fuzzy match)
- `score * 2` (emphasize)

---

## Common Abbreviations

Use these standard abbreviations:

- `q` = query
- `kw` = keywords
- `doc/docs` = document(s)
- `txt` = text
- `cat/cats` = category/categories
- `rel` = relevance
- `sim` = similarity
- `cls` = classify
- `ext` = extract
- `filt` = filter
- `res` = result(s)
- `temp` = temporary/template
- `param/params` = parameter(s)

---

## Symbols Reference

### Flow & Transform
- `→` : transform, flows to, maps to
- `|` : pipe through, then
- `>>` : apply operation, forward
- `?:` : conditional (ternary)
- `??` : null coalescing (use default if null)

### Logical
- `&&` : and
- `||` : or
- `!` : not
- `==` : equal
- `!=` : not equal
- `>`, `<`, `>=`, `<=` : comparisons

### Arithmetic & Modifiers
- `+` : add, combine, boost
- `-` : subtract, remove, penalty
- `*` : multiply, emphasize
- `/` : divide
- `%` : modulo
- `~` : approximately, fuzzy, similar

### Collections
- `∈` : element of, in
- `∉` : not in
- `∪` : union
- `∩` : intersection
- `&` : and/intersection
- `++` : concatenate, merge

### Special
- `@` : at location, in context
- `#` : count, number of
- `...` : spread, rest
- `.` : property access

### Emoji (Optional - Use for clarity)
- `🔍` : search
- `🎯` : target, precise
- `⚡` : boost, fast
- `⚖️` : rank, weigh
- `✂️` : trim, cut
- `✅` : validate, approve
- `❌` : reject, invalid
- `🚨` : urgent, alert
"#;

/// Returns the system prompt for SNS tasks.
pub fn get_sns_system_prompt() -> String {
    format!(
        "You are an SNS-native assistant. Read and write using SNS notation for efficiency. SNS (Shorthand Notation Script) achieves 60-85% token reduction.\n\n{}\n",
        SNS_MODEL_DEFINITION
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_sns_system_prompt() {
        let prompt = get_sns_system_prompt();
        assert!(prompt.contains("SNS-native assistant"));
        assert!(prompt.contains("→"));
        assert!(prompt.contains("|"));
    }
}
