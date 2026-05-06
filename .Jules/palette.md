## 2024-05-04 - Explicit Keyboard Types for Numeric Input
**Learning:** In Jetpack Compose, filtering input logically via `onValueChange` does not change the keyboard type displayed to the user. Omitting an explicit keyboard type results in a full text keyboard, slowing down numeric input.
**Action:** Always provide explicit `KeyboardOptions(keyboardType = KeyboardType.Decimal)` or `KeyboardType.Number` for numeric input fields to provide the most frictionless data entry experience.

## 2025-05-15 - Virtual Keyboard Submission Actions
**Learning:** Forcing users to tap a submit button for a single-line input field on mobile is cumbersome. Providing native keyboard submission actions (`ImeAction.Send`) creates a much smoother experience.
**Action:** Always add `keyboardOptions = KeyboardOptions(imeAction = ImeAction.Send)` and handle `keyboardActions` (with focus clearing) for primary text inputs in mobile apps.

## 2026-05-06 - Semantics Content Description for UI Switches
**Learning:** In Jetpack Compose, `Switch` components without semantic content descriptions are announced poorly or not at all by screen readers like TalkBack, lacking context about what setting the user is toggling.
**Action:** Use `modifier = Modifier.semantics { contentDescription = "..." }` on switches to associate them with the setting they control.

## 2026-05-06 - Accurate Semantics for Reused Switch Components
**Learning:** When copying and pasting Jetpack Compose UI components (like `Card` blocks containing a `Switch`), it is extremely common for developers to forget to update the `modifier = Modifier.semantics { contentDescription = "..." }`. This leads to screen readers announcing incorrect toggle names, causing significant confusion for visually impaired users.
**Action:** Always verify that `contentDescription` inside `semantics` modifiers perfectly matches the actual purpose of the interactive element, especially after copy-pasting component structures.
