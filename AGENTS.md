# Important thinking pattern

Think clearly before you respond; don't rush the output.
I would rather you ask me two critical questions than give me a pile of 'correct nonsense.'
I want specific, actionable, and tailored answers, not one-size-fits-all advice.
When answering, check our conversation history for background, search the web if possible, and
write code if necessary—utilize every capability you have.
If I say I am unsatisfied, do not just rephrase;
pivot your thinking and approach the problem from a new perspective.

# Programming methodology

### Rust programming style

1. Each class is defined in a dedicated file named after the class.
2. All functions must belong to a class, except test and script files located in src/tests and src/scripts.
3. Each function must have a compact docstring (no blank lines between sections) that describes its purpose, the data structure transformation it performs, and the key attributes (e.g., shape, type, role) of its parameters and return values.
4. Add a comment for every code block, especially those exceeding 5 lines.
5. Use snake_case for all variables, class methods, and package names; a leading underscore is permitted for private methods.

### Object-oriented programming principle

1. Each class has a single responsibility (CRC), centered around its core data members — the fundamental data that justifies the class's existence.
2. Complex tasks are decomposed into small, focused interactions among objects of different classes, in the way of small talk.
3. Every method body must not exceed 80 lines (excluding comments). If a method has long lines of codes,
   consider using the collector pattern to split the method.
4. Minimize class surface area — each method must justify its existence. If a responsibility can be absorbed by an existing method without harming clarity, do not create a new one.
5. Inside every method body, prefer guard-clause style: early returns, no else, and flat control flow.
6. Do not add trivial entry-point precondition checks (e.g., param == null) unless required
   by the method contract or a validated boundary/policy.
7. Constructor parameters must be essential — omit any option that can be derived, defaulted, or configured elsewhere (e.g., db_dir).
8. Each method follows a strict return contract: return a valid result or None for computation methods,
   and True/False for procedural methods. Thus, the caller only needs to check for None or False to decide whether to return early or proceed.
9. Log relevant data when important error conditions occur so failures are human-readable.
10. Keep the public interface of each class minimal to simplify usage by external objects.
11. Use short aliases for long member and local variable names,
    while preserving full names in class definitions and database schemas.
12. Follow TDD: write tests before implementing each new feature or function.
13. Use Pydantic for all config classes, and name each config's JSON file after its class (e.g., BackboneConfig.py → BackboneConfig.json).
14. Use 4-space indentation and type hints throughout; apply uv run ruff format or ruff check for formatting and linting.

### Human-Agent cooperated programming principle

1. human programmers' comments must be kept whenever we do refactor our code base.
