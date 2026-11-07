# Especificación de interfaz · Qui

**Revisión base:** «Paleta negro y azul», aprobada. Se conserva el wordmark Qui, Inter y la paleta negro/azul; la variante compacta usa el wordmark existente.  
**Revisión base:** «Paleta negro y azul», aprobada.
**Propuesta actual:** revisión nativa `1a1b7193d0c` («Formulario con scroll azul · revisión para aprobación»), pendiente de aprobación.  
**Plataforma:** escritorio Linux. Todas las pantallas de Qui: ventana flotante de 430 × 610 px lógicos.  
**Editable FigCraft:** [`qui.figcraft`](./qui.figcraft), páginas: `Selector principal`, `Selector · estados`, `Catálogo de UI`. Previews: [selector compacto](./previews/selector-compacto.png), [formulario de registro](./previews/anadir-cuenta.png), [éxito con clave pública copiable](./previews/clave-publica-copiable.png).

## Objetivo

Alternar globalmente perfiles SSH y la cuenta activa `gh`, y permitir registrar cuentas desde el selector con nombre visible, login de GitHub y ruta a una clave SSH privada ya existente. La clave nunca se lee ni se solicita como contenido. `gh` y SSH cambian globalmente; la identidad de autor de commits se conserva según las reglas `includeIf` actuales. El estado de autenticación `gh` aparece separado de SSH: cambiar el perfil no garantiza una sesión válida.

### Perfiles observados

- **Trabajo:** SSH Trabajo; `gh` tiene sesión válida.
- **Personal:** SSH Personal; `gh` tiene sesión válida.

## Flujo principal

1. Abrir el selector desde la aplicación; no se propone atajo global hasta comprobar conflictos.
2. Ver el perfil SSH actual y el estado `gh`; recordar que `includeIf` sigue controlando la autoría de commits.
3. Elegir Trabajo o Personal para seleccionar globalmente SSH y cuenta `gh` asociada.
4. Usar **Añadir cuenta** para abrir el formulario de registro.
5. Introducir nombre visible, login de GitHub para `gh` y elegir **Usar clave existente** o **Crear clave Ed25519**. La clave creada usa una ruta propia por perfil.
6. Al crear, Qui inicia `ssh-keygen -t ed25519` en Kitty de forma interactiva; `ssh-keygen` solicita la passphrase en el terminal. No pasarla como argumento. La clave privada queda local; Qui nunca la lee ni la muestra.
7. Guardar la ruta de la clave y volver al selector con la cuenta añadida disponible para activación explícita. El registro no cambia por sí solo el perfil global.
8. Mostrar el resultado SSH y el estado real de `gh`. Si falta sesión o el token no es válido, ofrecer **Iniciar sesión en gh** como acción deliberada e independiente.
9. Tras guardar perfil y crear la clave, mostrar el contenido íntegro del `.pub` en un campo seleccionable. **Copiar clave pública** copia la línea completa al portapapeles solo al pulsarlo; informar «Copiada» y mantener el campo visible.
10. Indicar: **Pégala en GitHub → Settings → SSH and GPG keys → New SSH key**. Solo se muestra y copia el `.pub`; la clave privada nunca se muestra ni copia.
11. Mostrar **Volver a …** para restaurar el perfil anterior tras un cambio.

## Pantallas

### Selector principal · 430 × 610

Ventana flotante vertical de referencia: **430 × 610 px lógicos**, con margen interior de 20 px. Dos filas compactas de perfil (390 × 84 px) muestran SSH y estado `gh`; Personal está seleccionada y rotulada **ACTIVA** con fondo y borde azul. Trabajo queda disponible para selección. Ambas cuentas muestran `gh · sesión válida`. **+ Añadir cuenta** ocupa el ancho de contenido. El aviso inferior recuerda que SSH y `gh` cambian globalmente y que la autoría Git sigue por directorio según `includeIf`. No hay dependencia del repositorio actual.

La dirección toma de la [vista List de Faugus Launcher](https://flathub.org/en/apps/io.github.Faugus.faugus-launcher) la lista vertical compacta y la selección azul; conserva la marca, tipografía Inter, textos y paleta de Qui. No reutiliza assets de Faugus.

Teclado: `Tab` recorre perfiles y **Añadir cuenta**, `Enter` activa o abre, `Escape` cierra; foco visible y nombre accesible. El acceso inicial queda vinculado a la UI principal, sin atajo global. La composición se revisó a 430 × 610 px; sin variante adicional para tablet o móvil.

### Añadir cuenta · 430 × 610

Frame `Añadir cuenta · formulario` en `Selector principal`, revisión nativa `1a1b7193d0c`. Acceso desde **+ Añadir cuenta**. Campos requeridos: nombre visible, login GitHub para `gh`, método y ruta de clave. **Usar clave existente** permite elegir una clave local; **Crear clave Ed25519** usa una ruta propia para el perfil e inicia `ssh-keygen -t ed25519` en Kitty con prompt interactivo de passphrase. No pasar la passphrase como argumento. La clave privada queda local; Qui nunca la lee ni muestra. El aviso fija el alcance global de `gh` y SSH y la autoría por directorio según `includeIf`. **Guardar cuenta** vuelve al selector sin activar el perfil automáticamente. Controles estáticos; interacción no probada. El formulario muestra contenido posterior a los botones fuera de la ventana; la nota «Después de guardar» queda tras el pliegue. Scroll vertical: gutter de 7 px respecto al contenido y al borde derecho; track de 6 × 570 px, `#EAF1FF`, radio 3 px; thumb de 6 × 494 px, `#2563EB`, radio 3 px. Rueda/trackpad o arrastre del thumb; sin scroll horizontal. En FigCraft el thumb es una anotación visual estática, no se probó desplazamiento.

Validación propuesta: campos requeridos y existencia de la clave elegida o disponibilidad de la ruta de destino; error junto al campo, conservando valores. El estado de error no tiene frame visual todavía.

### Clave pública tras guardar perfil · 430 × 610

Frame `Clave pública lista · éxito` en `Selector · estados`, revisión nativa `1a1b70ea6f9`. Tras guardar perfil y crear clave, presenta la ruta del `.pub` y su línea completa en un campo seleccionable. La cadena del frame es un ejemplo visual; la app debe cargar el contenido íntegro del `.pub` del perfil. **Copiar clave pública** escribe esa línea al portapapeles del sistema solo tras activación explícita, sin diálogo adicional de permisos; después confirma «Copiada» y mantiene el campo disponible. Instrucción: **Pégala en GitHub → Settings → SSH and GPG keys → New SSH key**. La clave privada nunca se muestra ni se copia. El upload automático está omitido; copia manual es el flujo principal. Controles estáticos; interacción no probada.

### Cambio realizado · 430 × 610

Confirma «Perfil SSH cambiado a Trabajo. gh sigue sin sesión». **Iniciar sesión en gh** solo inicia el flujo de autenticación al activarse. **Volver a Personal** restaura el perfil anterior. Los controles son rótulos estáticos en esta maqueta; no se probó interacción.

### Error

No tiene frame visual. Ante error, conservar el perfil anterior, especificar si falló SSH o la selección/autenticación de `gh`, y ofrecer reintento. No inferir que `gh` está autenticado por el cambio de SSH.

## Ventana y estilo

El selector compacto conserva el wordmark `qui` del diseño existente; mantiene Inter, texto negro y acento azul. Los activos de identidad del proyecto están en [`qui-logo.png`](./qui-logo.png), [`qui-icon.png`](./qui-icon.png) y la fuente editable [`qui-brand.vectorcraft`](./qui-brand.vectorcraft).

Selector, registro, cambio y éxito comparten ventana de 430 × 610 px lógicos. El formulario conserva contenido posterior a la ventana para revisar el scroll vertical. Gutter visible, track azul muy claro y thumb azul redondeado; no hay scroll horizontal. Escritorio solamente. Sin animación requerida; cambio de perfil inmediato y foco visible, respetando movimiento reducido.

Paleta de negro y azules: fondo `#F3F7FC`, superficie `#FFFFFF`, texto `#111827`, secundario `#52627A`, azul principal `#2563EB`, azul claro `#EAF1FF`, error `#B42318`; Inter; títulos 20–24 px, cuerpo 12–15 px, rótulos 11 px; espacio base 8 px, radios 8–12 px, sin sombras. Estado expresado con texto, no solo color.

## Revisión y límites

La revisión FigCraft `1a1b5f5ac75` contiene el selector global SSH + `gh`, conserva la autoría `includeIf` y presenta por separado el estado de autenticación. La revisión `1a1b64619b0` añade el éxito tras registrar cuenta con campo copiable del `.pub` e instrucciones de GitHub. La revisión `1a1b70ea6f9` adapta selector, formulario y estados a 430 × 610 px; la revisión `1a1b7193d0c` especifica el gutter y scroll azul del formulario; refleja Personal activa y ambas sesiones `gh` válidas. Es propuesta pendiente de aprobación; no se ha trasladado al código. Error documentado, sin marco visual. La autenticación real depende de `gh` y las sesiones configuradas. Si no existe `x-terminal-emulator`, se muestra el comando de autenticación para ejecutarlo manualmente.
