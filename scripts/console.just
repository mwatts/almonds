[working-directory: 'console']
watch-frontend:
    npm run dev

[working-directory: 'console']
watch-tauri:
    npm run tauri dev

[working-directory: 'console']
watch-mobile:
    npm run tauri android dev 

[working-directory: 'console']
watch-mobile-device DEVICE HOST:
    npm run tauri android dev {{DEVICE}} --host {{HOST}}

[working-directory: 'console']
watch-ios:
    npm run tauri ios dev

[working-directory: 'console']
watch-console:
    npm run dev & npm run tauri dev

[working-directory: 'console']
build-console:
    npm run tauri build

[working-directory: 'console']
install-frontend-dependencies:
    npm install

[working-directory: 'console']
test-console:
    echo "TBD"

[working-directory: 'console/src-tauri']
test-tauri:
    echo "TBD"