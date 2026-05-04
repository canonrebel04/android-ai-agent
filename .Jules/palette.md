## 2024-05-04 - Explicit Keyboard Types for Numeric Input
**Learning:** In Jetpack Compose, filtering input logically via `onValueChange` does not change the keyboard type displayed to the user. Omitting an explicit keyboard type results in a full text keyboard, slowing down numeric input.
**Action:** Always provide explicit `KeyboardOptions(keyboardType = KeyboardType.Decimal)` or `KeyboardType.Number` for numeric input fields to provide the most frictionless data entry experience.
