## 2024-05-14 - Password Visibility Toggle
**Learning:** In Jetpack Compose, password fields (`VisualTransformation.PasswordVisualTransformation`) lack a built-in visibility toggle. Implementing one requires manually adding a trailing `IconButton` and conditionally toggling `VisualTransformation.None`.
**Action:** Always verify if users need to confirm complex secrets (like API keys or bot tokens) and provide a visibility toggle to reduce input errors while maintaining default security.
