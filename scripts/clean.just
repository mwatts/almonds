[working-directory: 'console']
clean-console:
    rm -rf node_modules dist .nuxt
    cd src-tauri && cargo clean

[working-directory: 'server']
clean-server:
    cargo clean

clean target:
    #!/usr/bin/env sh
    echo "Cleaning {{target}} build assets"
    if [ "{{target}}" = "console" ]; then
        just clean-console
    elif [ "{{target}}" = "server" ]; then
        just clean-server
    elif [ "{{target}}" = "all" ]; then
        just clean-console
        just clean-server
    else
        echo "Invalid target '{{target}}'. Use one of: console, server, all"
    fi
