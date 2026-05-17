## 2024-05-14 - Password Visibility Toggle
**Learning:** In Jetpack Compose, password fields (`VisualTransformation.PasswordVisualTransformation`) lack a built-in visibility toggle. Implementing one requires manually adding a trailing `IconButton` and conditionally toggling `VisualTransformation.None`.
**Action:** Always verify if users need to confirm complex secrets (like API keys or bot tokens) and provide a visibility toggle to reduce input errors while maintaining default security.

## 2024-05-24 - Screen Reader Double-Announcement in Navigation Bar
**Learning:** When `NavigationBarItem` includes both an `icon` and a `label`, setting `contentDescription` on the `Icon` to the same text as the label causes screen readers to announce the item twice (e.g., "Chat, Chat").
**Action:** Explicitly set `contentDescription = null` for `Icon`s inside `NavigationBarItem` when a text label is already provided, treating the icon as purely decorative.

## 2024-05-28 - Bot Token Visibility Toggle in ChannelsScreen
**Learning:** Password fields (`VisualTransformation.PasswordVisualTransformation`) lack a built-in visibility toggle. Implementing one requires manually adding a trailing `IconButton` and conditionally toggling `VisualTransformation.None`.
**Action:** Always verify if users need to confirm complex secrets (like API keys or bot tokens) and provide a visibility toggle to reduce input errors while maintaining default security.
