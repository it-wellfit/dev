use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;

/// Role management for RBAC (Role-Based Access Control)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub role_id: String,
    pub role_name: String,
    pub role_code: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl Role {
    /// Create a new role
    pub fn new(
        role_id: String,
        role_name: String,
        role_code: String,
        description: Option<String>,
    ) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            role_id,
            role_name,
            role_code,
            description,
            is_system: false,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Permission definition for granular access control
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Permission {
    pub permission_id: String,
    pub permission_name: String,
    pub permission_code: String,
    pub permission_type: String, // MENU, BUTTON, FIELD, DATA
    pub resource: String,
    pub action: String, // CREATE, READ, UPDATE, DELETE, EXECUTE
    pub description: Option<String>,
    pub is_active: bool,
}

impl Permission {
    /// Create a new permission
    pub fn new(
        permission_id: String,
        permission_name: String,
        permission_code: String,
        permission_type: String,
        resource: String,
        action: String,
        description: Option<String>,
    ) -> Self {
        Self {
            permission_id,
            permission_name,
            permission_code,
            permission_type,
            resource,
            action,
            description,
            is_active: true,
        }
    }
}

/// Role-Permission mapping
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RolePermission {
    pub role_id: String,
    pub permission_id: String,
    pub permission_code: String,
    pub granted_at: chrono::NaiveDateTime,
    pub granted_by: String,
}

impl RolePermission {
    /// Create a new role-permission mapping
    pub fn new(
        role_id: String,
        permission_id: String,
        permission_code: String,
        granted_by: String,
    ) -> Self {
        Self {
            role_id,
            permission_id,
            permission_code,
            granted_at: chrono::Utc::now().naive_utc(),
            granted_by,
        }
    }
}

/// User role assignment
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserRole {
    pub user_id: String,
    pub role_id: String,
    pub assigned_at: chrono::NaiveDateTime,
    pub assigned_by: String,
}

impl UserRole {
    /// Create a new user role assignment
    pub fn new(user_id: String, role_id: String, assigned_by: String) -> Self {
        Self {
            user_id,
            role_id,
            assigned_at: chrono::Utc::now().naive_utc(),
            assigned_by,
        }
    }
}

/// Permission checker for role-based access control
#[derive(Clone)]
pub struct PermissionChecker {
    role_permissions: HashMap<String, Vec<String>>, // role_id -> permission_codes
    user_roles: HashMap<String, Vec<String>>,       // user_id -> role_ids
}

impl PermissionChecker {
    /// Create a new permission checker
    pub fn new() -> Self {
        Self {
            role_permissions: HashMap::new(),
            user_roles: HashMap::new(),
        }
    }

    /// Add role-permission mapping
    pub fn add_role_permission(&mut self, role_permission: RolePermission) {
        self.role_permissions
            .entry(role_permission.role_id)
            .or_default()
            .push(role_permission.permission_code);
    }

    /// Assign role to user
    pub fn assign_role(&mut self, user_role: UserRole) {
        self.user_roles
            .entry(user_role.user_id)
            .or_default()
            .push(user_role.role_id);
    }

    /// Check if user has permission
    pub fn has_permission(&self, user_id: &str, permission_code: &str) -> bool {
        // Get user's roles
        let user_roles = match self.user_roles.get(user_id) {
            Some(roles) => roles,
            None => return false,
        };

        // Check if any of the user's roles has the permission
        for role_id in user_roles {
            if let Some(permissions) = self.role_permissions.get(role_id)
                && permissions.contains(&permission_code.to_string())
            {
                return true;
            }
        }

        false
    }

    /// Get all permissions for a user
    pub fn get_user_permissions(&self, user_id: &str) -> Vec<String> {
        let user_roles = match self.user_roles.get(user_id) {
            Some(roles) => roles,
            None => return Vec::new(),
        };

        let mut permissions = Vec::new();

        for role_id in user_roles {
            if let Some(role_perms) = self.role_permissions.get(role_id) {
                permissions.extend_from_slice(role_perms);
            }
        }

        // Remove duplicates
        permissions.sort();
        permissions.dedup();

        permissions
    }
}

impl Default for PermissionChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Permission manager for database operations
#[derive(Clone)]
pub struct PermissionManager {
    pool: sqlx::SqlitePool,
    checker: PermissionChecker,
}

impl PermissionManager {
    /// Create a new permission manager
    pub async fn new(pool: sqlx::SqlitePool) -> Self {
        let mut mgr = Self {
            pool,
            checker: PermissionChecker::new(),
        };
        // Auto-initialize defaults and load persisted mappings so has_permission works
        let _ = mgr.initialize_default().await;
        mgr
    }

    /// Initialize default permissions and roles
    pub async fn initialize_default(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let pool = &self.pool;

        // Create default roles
        let default_roles = vec![
            Role::new(
                "role-admin".to_string(),
                "Administrator".to_string(),
                "ADMIN".to_string(),
                Some("Full system administration rights".to_string()),
            ),
            Role::new(
                "role-finance".to_string(),
                "Finance Manager".to_string(),
                "FINANCE".to_string(),
                Some("Financial transaction management rights".to_string()),
            ),
            Role::new(
                "role-store".to_string(),
                "Store Manager".to_string(),
                "STORE".to_string(),
                Some("Store operation management rights".to_string()),
            ),
            Role::new(
                "role-viewer".to_string(),
                "Viewer".to_string(),
                "VIEWER".to_string(),
                Some("Read-only report viewer rights".to_string()),
            ),
            Role::new(
                "role-pos".to_string(),
                "POS Operator".to_string(),
                "POS".to_string(),
                Some("POS terminal operation rights".to_string()),
            ),
        ];

        for role in default_roles {
            sqlx::query(
                "INSERT OR IGNORE INTO roles (role_id, role_name, role_code, description, is_system, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&role.role_id)
            .bind(&role.role_name)
            .bind(&role.role_code)
            .bind(&role.description)
            .bind(role.is_system)
            .bind(role.created_at)
            .bind(role.updated_at)
            .execute(pool)
            .await?;
        }

        // Create default permissions
        let default_permissions = vec![
            Permission::new(
                "perm-ledger-write".to_string(),
                "Ledger Write".to_string(),
                "LEDGER_WRITE".to_string(),
                "BUTTON".to_string(),
                "ledger".to_string(),
                "CREATE".to_string(),
                Some("Create financial transactions".to_string()),
            ),
            Permission::new(
                "perm-ledger-read".to_string(),
                "Ledger Read".to_string(),
                "LEDGER_READ".to_string(),
                "MENU".to_string(),
                "ledger".to_string(),
                "READ".to_string(),
                Some("View financial transactions".to_string()),
            ),
            Permission::new(
                "perm-config-manage".to_string(),
                "Configuration Management".to_string(),
                "CONFIG_MANAGE".to_string(),
                "MENU".to_string(),
                "config".to_string(),
                "EXECUTE".to_string(),
                Some("Manage ERP configuration".to_string()),
            ),
            Permission::new(
                "perm-role-manage".to_string(),
                "Role Management".to_string(),
                "ROLE_MANAGE".to_string(),
                "MENU".to_string(),
                "permission".to_string(),
                "EXECUTE".to_string(),
                Some("Manage user roles and permissions".to_string()),
            ),
            Permission::new(
                "perm-report-view".to_string(),
                "Report View".to_string(),
                "REPORT_VIEW".to_string(),
                "MENU".to_string(),
                "report".to_string(),
                "READ".to_string(),
                Some("View financial reports".to_string()),
            ),
        ];

        for permission in default_permissions {
            sqlx::query(
                "INSERT OR IGNORE INTO permissions (permission_id, permission_name, permission_code, permission_type, resource, action, description, is_active) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&permission.permission_id)
            .bind(&permission.permission_name)
            .bind(&permission.permission_code)
            .bind(&permission.permission_type)
            .bind(&permission.resource)
            .bind(&permission.action)
            .bind(&permission.description)
            .bind(permission.is_active)
            .execute(pool)
            .await?;
        }

        // Create default role-permission mappings
        let default_role_permissions = vec![
            RolePermission::new(
                "role-admin".to_string(),
                "perm-ledger-write".to_string(),
                "LEDGER_WRITE".to_string(),
                "system".to_string(),
            ),
            RolePermission::new(
                "role-admin".to_string(),
                "perm-ledger-read".to_string(),
                "LEDGER_READ".to_string(),
                "system".to_string(),
            ),
            RolePermission::new(
                "role-admin".to_string(),
                "perm-config-manage".to_string(),
                "CONFIG_MANAGE".to_string(),
                "system".to_string(),
            ),
            RolePermission::new(
                "role-admin".to_string(),
                "perm-role-manage".to_string(),
                "ROLE_MANAGE".to_string(),
                "system".to_string(),
            ),
            RolePermission::new(
                "role-admin".to_string(),
                "perm-report-view".to_string(),
                "REPORT_VIEW".to_string(),
                "system".to_string(),
            ),
            RolePermission::new(
                "role-finance".to_string(),
                "perm-ledger-write".to_string(),
                "LEDGER_WRITE".to_string(),
                "system".to_string(),
            ),
            RolePermission::new(
                "role-finance".to_string(),
                "perm-ledger-read".to_string(),
                "LEDGER_READ".to_string(),
                "system".to_string(),
            ),
            RolePermission::new(
                "role-store".to_string(),
                "perm-ledger-read".to_string(),
                "LEDGER_READ".to_string(),
                "system".to_string(),
            ),
            RolePermission::new(
                "role-viewer".to_string(),
                "perm-report-view".to_string(),
                "REPORT_VIEW".to_string(),
                "system".to_string(),
            ),
            RolePermission::new(
                "role-pos".to_string(),
                "perm-ledger-read".to_string(),
                "LEDGER_READ".to_string(),
                "system".to_string(),
            ),
        ];

        for role_perm in default_role_permissions {
            sqlx::query(
                "INSERT OR IGNORE INTO role_permissions (role_id, permission_id, granted_at, granted_by) VALUES (?, ?, ?, ?)"
            )
            .bind(&role_perm.role_id)
            .bind(&role_perm.permission_id)
            .bind(role_perm.granted_at)
            .bind(&role_perm.granted_by)
            .execute(pool)
            .await?;

            // Update in-memory checker
            self.checker.add_role_permission(role_perm);
        }

        // Create default user assignments
        let default_user_roles = vec![
            UserRole::new(
                "user-admin".to_string(),
                "role-admin".to_string(),
                "system".to_string(),
            ),
            UserRole::new(
                "user-finance".to_string(),
                "role-finance".to_string(),
                "system".to_string(),
            ),
            UserRole::new(
                "user-store1".to_string(),
                "role-store".to_string(),
                "system".to_string(),
            ),
            UserRole::new(
                "user-pos1".to_string(),
                "role-pos".to_string(),
                "system".to_string(),
            ),
        ];

        for user_role in default_user_roles {
            sqlx::query(
                "INSERT OR IGNORE INTO user_roles (user_id, role_id, assigned_at, assigned_by) VALUES (?, ?, ?, ?)"
            )
            .bind(&user_role.user_id)
            .bind(&user_role.role_id)
            .bind(user_role.assigned_at)
            .bind(&user_role.assigned_by)
            .execute(pool)
            .await?;

            // Update in-memory checker
            self.checker.assign_role(user_role);
        }

        // Rebuild in-memory checker from persisted state (defaults + any pre-existing rows)
        self.checker = PermissionChecker::new();
        let role_perms: Vec<(String, String)> = sqlx::query_as(
            "SELECT rp.role_id, p.permission_code FROM role_permissions rp JOIN permissions p ON rp.permission_id = p.permission_id"
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default();
        for (role_id, code) in role_perms {
            self.checker.add_role_permission(RolePermission::new(
                role_id,
                String::new(),
                code,
                "system".to_string(),
            ));
        }
        let user_roles_rows: Vec<(String, String)> =
            sqlx::query_as("SELECT user_id, role_id FROM user_roles")
                .fetch_all(pool)
                .await
                .unwrap_or_default();
        for (user_id, role_id) in user_roles_rows {
            self.checker
                .assign_role(UserRole::new(user_id, role_id, "system".to_string()));
        }

        Ok(())
    }

    /// Check if user has specific permission
    pub fn has_permission(&self, user_id: &str, permission_code: &str) -> bool {
        self.checker.has_permission(user_id, permission_code)
    }

    /// Check a permission against the **live database** (not the startup
    /// snapshot). Use this on request paths so role assignments made after
    /// startup take effect immediately (e.g. the user administration UI).
    pub async fn has_permission_db(&self, user_id: &str, permission_code: &str) -> bool {
        let count: Result<(i64,), sqlx::Error> = sqlx::query_as(
            "SELECT COUNT(*) FROM user_roles ur \
             JOIN role_permissions rp ON ur.role_id = rp.role_id \
             JOIN permissions p ON rp.permission_id = p.permission_id \
             WHERE ur.user_id = ? AND p.permission_code = ? AND p.is_active = 1",
        )
        .bind(user_id)
        .bind(permission_code)
        .fetch_one(&self.pool)
        .await;
        matches!(count, Ok((n,)) if n > 0)
    }

    /// Get all permissions for a user
    pub fn get_user_permissions(&self, user_id: &str) -> Vec<String> {
        self.checker.get_user_permissions(user_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_checker() -> PermissionChecker {
        let mut checker = PermissionChecker::new();
        checker.add_role_permission(RolePermission::new(
            "role-admin".to_string(),
            "perm-ledger-write".to_string(),
            "LEDGER_WRITE".to_string(),
            "system".to_string(),
        ));
        checker.add_role_permission(RolePermission::new(
            "role-admin".to_string(),
            "perm-ledger-read".to_string(),
            "LEDGER_READ".to_string(),
            "system".to_string(),
        ));
        checker.add_role_permission(RolePermission::new(
            "role-store".to_string(),
            "perm-ledger-read".to_string(),
            "LEDGER_READ".to_string(),
            "system".to_string(),
        ));
        checker.assign_role(UserRole::new(
            "user-admin".to_string(),
            "role-admin".to_string(),
            "system".to_string(),
        ));
        checker.assign_role(UserRole::new(
            "user-store1".to_string(),
            "role-store".to_string(),
            "system".to_string(),
        ));
        checker
    }

    #[test]
    fn has_permission_matches_code_not_id() {
        let checker = build_checker();
        // 正確 code 應回傳 true
        assert!(checker.has_permission("user-admin", "LEDGER_WRITE"));
        assert!(checker.has_permission("user-admin", "LEDGER_READ"));
        // 該角色沒有寫入權限
        assert!(!checker.has_permission("user-store1", "LEDGER_WRITE"));
        // 以 permission_id 查詢不應匹配（bug 修正前會誤判）
        assert!(!checker.has_permission("user-admin", "perm-ledger-write"));
        // 不存在的使用者
        assert!(!checker.has_permission("user-unknown", "LEDGER_WRITE"));
    }

    #[test]
    fn get_user_permissions_returns_codes_deduped() {
        let checker = build_checker();
        let perms = checker.get_user_permissions("user-admin");
        assert!(perms.contains(&"LEDGER_WRITE".to_string()));
        assert!(perms.contains(&"LEDGER_READ".to_string()));
        // 不應包含 permission_id
        assert!(!perms.contains(&"perm-ledger-write".to_string()));
        // 不應重複
        assert_eq!(perms.iter().filter(|p| *p == "LEDGER_READ").count(), 1);
        // 無權限使用者回傳空
        assert!(checker.get_user_permissions("user-unknown").is_empty());
    }
}
