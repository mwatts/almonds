-- ============================================
-- m20260304_080239_create_default_workspace
-- ============================================


        INSERT INTO workspaces (identifier, name, description, created_at, updated_at)
        SELECT '65be8e3a-7bc3-4c02-89fd-88bb2b85d029', 'default', 'Default workspace', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
        WHERE NOT EXISTS (
            SELECT 1 FROM workspaces WHERE name = 'default'
        );
        ;


        UPDATE todo SET workspace_identifier = '65be8e3a-7bc3-4c02-89fd-88bb2b85d029' WHERE workspace_identifier IS NULL;
        UPDATE notes SET workspace_identifier = '65be8e3a-7bc3-4c02-89fd-88bb2b85d029' WHERE workspace_identifier IS NULL;
        UPDATE bookmark SET workspace_identifier = '65be8e3a-7bc3-4c02-89fd-88bb2b85d029' WHERE workspace_identifier IS NULL;
        UPDATE recycle_bin SET workspace_identifier = '65be8e3a-7bc3-4c02-89fd-88bb2b85d029' WHERE workspace_identifier IS NULL;
        UPDATE reminder SET workspace_identifier = '65be8e3a-7bc3-4c02-89fd-88bb2b85d029' WHERE workspace_identifier IS NULL;
        UPDATE snippets SET workspace_identifier = '65be8e3a-7bc3-4c02-89fd-88bb2b85d029' WHERE workspace_identifier IS NULL;
        ;

