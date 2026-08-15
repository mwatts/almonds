-- ============================================
-- m20260304_080239_create_default_workspace
-- ============================================


        INSERT INTO workspaces (identifier, name, description, created_at, updated_at)
        SELECT '60de829c-9b65-4e2d-b2f8-bb7a6f18d98a', 'default', 'Default workspace', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
        WHERE NOT EXISTS (
            SELECT 1 FROM workspaces WHERE name = 'default'
        );
        ;


        UPDATE todo SET workspace_identifier = '60de829c-9b65-4e2d-b2f8-bb7a6f18d98a' WHERE workspace_identifier IS NULL;
        UPDATE notes SET workspace_identifier = '60de829c-9b65-4e2d-b2f8-bb7a6f18d98a' WHERE workspace_identifier IS NULL;
        UPDATE bookmark SET workspace_identifier = '60de829c-9b65-4e2d-b2f8-bb7a6f18d98a' WHERE workspace_identifier IS NULL;
        UPDATE recycle_bin SET workspace_identifier = '60de829c-9b65-4e2d-b2f8-bb7a6f18d98a' WHERE workspace_identifier IS NULL;
        UPDATE reminder SET workspace_identifier = '60de829c-9b65-4e2d-b2f8-bb7a6f18d98a' WHERE workspace_identifier IS NULL;
        UPDATE snippets SET workspace_identifier = '60de829c-9b65-4e2d-b2f8-bb7a6f18d98a' WHERE workspace_identifier IS NULL;
        ;

