use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessDecision {
    pub allowed: bool,
    pub reason: String,
}

pub struct AndroidPermissionGuard {
    blocked_packages: HashSet<String>,
    safe_packages: HashSet<String>,
    blocked_actions: HashSet<String>,
}

impl AndroidPermissionGuard {
    pub fn new() -> Self {
        let blocked_packages: HashSet<String> = [
            "com.android.settings",
            "com.android.vending",
            "com.google.android.gms.supervision",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        let safe_packages: HashSet<String> = [
            "com.google.android.calendar",
            "com.google.android.contacts",
            "com.google.android.gm",
            "com.google.android.apps.messaging",
            "org.telegram.messenger",
            "com.whatsapp",
            "com.android.chrome",
            "com.android.calculator2",
            "com.termux",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        let blocked_actions: HashSet<String> = [
            "uninstall_app",
            "disable_app",
            "clear_app_data",
            "factory_reset",
            "change_password",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        Self {
            blocked_packages,
            safe_packages,
            blocked_actions,
        }
    }

    pub fn can_interact_with_app(&self, package: &str) -> AccessDecision {
        if self.blocked_packages.contains(package) {
            AccessDecision {
                allowed: false,
                reason: format!("'{}' is blocked", package),
            }
        } else {
            AccessDecision {
                allowed: true,
                reason: String::new(),
            }
        }
    }

    pub fn can_perform_action(&self, action: &str) -> AccessDecision {
        if self.blocked_actions.contains(action) {
            AccessDecision {
                allowed: false,
                reason: format!("Action '{}' is blocked", action),
            }
        } else {
            AccessDecision {
                allowed: true,
                reason: String::new(),
            }
        }
    }

    pub fn is_safe_package(&self, package: &str) -> bool {
        self.safe_packages.contains(package)
    }
}

impl Default for AndroidPermissionGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocked_package_denied() {
        let guard = AndroidPermissionGuard::new();
        let decision = guard.can_interact_with_app("com.android.settings");
        assert!(!decision.allowed);
        assert!(decision.reason.contains("blocked"));
    }

    #[test]
    fn safe_package_allowed() {
        let guard = AndroidPermissionGuard::new();
        let decision = guard.can_interact_with_app("com.whatsapp");
        assert!(decision.allowed);
        assert!(decision.reason.is_empty());
    }

    #[test]
    fn blocked_action_denied() {
        let guard = AndroidPermissionGuard::new();
        let decision = guard.can_perform_action("factory_reset");
        assert!(!decision.allowed);
        assert!(decision.reason.contains("blocked"));
    }

    #[test]
    fn is_safe_package_check() {
        let guard = AndroidPermissionGuard::new();
        assert!(guard.is_safe_package("com.termux"));
        assert!(!guard.is_safe_package("com.evil.malware"));
    }
}
