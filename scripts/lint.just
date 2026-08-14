[working-directory:'server']
lint-server:
    sh ../scripts/format

[working-directory:'kernel']
lint-kernel:
    sh ../scripts/format


[working-directory:'console/src-tauri']
lint-console-tauri:
    sh ../../scripts/format


[working-directory:'console']
lint-console:
    npm run format
    npm run lint:fix

