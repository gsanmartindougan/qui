# Qui

Selector de cuentas SSH y GitHub CLI. Detecta los perfiles Trabajo y Personal existentes y permite guardar cuentas adicionales desde la aplicación.

## Requisitos

- Rust 1.85 o posterior y Cargo.
- `git` y GitHub CLI (`gh`).
- Las identidades de commit siguen usando las reglas `includeIf` configuradas en Git.
- Kitty para abrir autenticación de `gh` y la creación interactiva de claves; `x-terminal-emulator` sirve como alternativa.
- En Linux, un portal XDG de archivos (GTK, GNOME o KDE) para elegir claves existentes y rutas de destino.

## Ejecutar

```sh
cargo run
```

En Linux x86-64 también puedes iniciar el binario [Qui](outputs/qui-linux-x86_64).

Qui detecta los perfiles Trabajo y Personal a partir de las reglas `includeIf` y los alias SSH existentes. Desde **Añadir cuenta** puedes guardar un login de GitHub y seleccionar una clave privada existente o crear una clave Ed25519 con `ssh-keygen` en Kitty. La clave privada permanece local. Tras guardar, Qui muestra el archivo `.pub` y permite copiar su línea pública completa para pegarla en GitHub. Los perfiles añadidos se guardan en `$XDG_CONFIG_HOME/qui/profiles.json` (o `~/.config/qui/profiles.json`).

Al activar un perfil, Qui configura `github.com` con su clave y hace que Git normalice remotos de GitHub con alias al host canónico `git@github.com:`. Añade una línea `Include` a `~/.ssh/config` y un `include.path` global de Git hacia los archivos administrados bajo `$XDG_CONFIG_HOME/qui/` (o `~/.config/qui/`). No cambia `user.name`, `user.email`, URLs remotas guardadas ni credenciales; los sufijos `-work` y `-personal` no aparecen en las URLs efectivas. Las credenciales de `gh` siguen administradas por GitHub CLI. Si falta iniciar sesión, Qui intenta abrir Kitty y usa `x-terminal-emulator` como alternativa. Después, activa de nuevo el perfil para refrescar su estado.

## Comprobar la cuenta activa

Dentro de un repositorio, ejecuta:

```sh
git remote get-url origin
ssh -T git@github.com
gh auth status --hostname github.com
gh api user --jq .login
git var GIT_AUTHOR_IDENT
```

El remoto efectivo debería usar `git@github.com:` sin sufijo. El saludo de SSH indica qué cuenta tiene la clave seleccionada; `gh api` indica el usuario activo de `gh`. `git var` muestra por separado la identidad de autor que corresponde al directorio. Git expande las reglas `insteadOf` al mostrar `git remote get-url`.

Para revertir Qui, elimina de `~/.ssh/config` su línea `Include` y de la configuración global de Git el `include.path` que apunta a `ssh-profile.gitconfig`; después puedes borrar los dos archivos administrados en `$XDG_CONFIG_HOME/qui/`.

El cambio de SSH es independiente del inicio de sesión de `gh`: Qui puede dejar SSH cambiado y mostrar que `gh` no pudo cambiar de cuenta. La identidad de autor de commits sigue dependiendo de las reglas `includeIf` actuales.
