# Inventario UI · Qui

**Estado:** selector compacto implementado; se conserva el wordmark Qui y sus tokens. El formulario de cuenta y la copia manual de clave pública también están implementados. Referencias: [especificación](./interface-spec.md), [`qui.figcraft`](./qui.figcraft).

| ID | Elemento | Uso y estados | Interacción | Ventana/movimiento |
|---|---|---|---|---|
| UI-01 | Marca Qui | Wordmark textual existente; Inter y paleta negra/azul | Sin acción asignada | Cabecera compacta |
| UI-02 | Fila de perfil | Personal/Trabajo; SSH + sesión `gh` válida en ambas; Personal activa | Activar al seleccionar; mostrar `ACTIVA` en texto | Selector y ventanas del flujo 430 × 610 px; ventana flotante vertical, botones accesibles por teclado |
| UI-03 | Aviso global y autoría `includeIf` | SSH y `gh` globales; autoría por directorio | Solo lectura | Panel inferior 390 × 98 px; envolver texto |
| UI-04 | Lista de perfiles | Filas compactas, selección azul; Personal activa en preview | `Tab` recorre, `Enter` activa; foco visible | Ancho interior 390 px; filas 390 × 84 px |
| UI-05 | Aviso de resultado/error | Resultado SSH y estado de `gh`; éxito, error, falta de sesión | Anunciar sin robar foco | Envolver; movimiento reducido |
| UI-06 | Iniciar sesión `gh` | Visible si falta sesión o token inválido | Login solo tras activación explícita | Junto o debajo del aviso |
| UI-07 | Volver al perfil anterior | Acción secundaria tras cambio | Revertir selección y anunciar resultado | Junto al aviso temporal |
| UI-08 | Teclado/cierre | Perfiles, Añadir cuenta, cierre | `Tab` recorre, `Enter` activa/abre, `Escape` cierra; sin atajo global | Ventana compacta |
| UI-09 | Formulario de cuenta | Nombre visible, login `gh`, usar clave existente o crear Ed25519 | Elegir ruta o iniciar `ssh-keygen` interactivo en Kitty; passphrase solo en terminal. No leer/mostrar clave privada | 430 × 610; contenido inferior accesible con scroll vertical |
| UI-10 | Aviso de alcance | `gh` y SSH globales; autoría por `includeIf` | Solo lectura | Envolver; azul claro |
| UI-11 | Guardar/cancelar | Añade perfil o vuelve al selector | Guardar no activa el perfil | Acciones al pie; estáticas en maqueta |
| UI-12 | Copiar clave pública | Éxito tras guardar perfil y crear clave; muestra línea completa obtenida del `.pub` | Botón copia línea íntegra al portapapeles del sistema solo al pulsarlo; sin permiso adicional; confirmar «Copiada» y mantener visible. No copiar privada | Frame 430 × 610 |
| UI-13 | Scroll vertical | Formulario con contenido bajo el pliegue | Rueda/trackpad o arrastrar thumb; sin scroll horizontal | Gutter 7 px; track 6×570 `#EAF1FF`, thumb 6×494 `#2563EB`, ambos radio 3 px |

## Tokens

Fondo `#F3F7FC`; superficie `#FFFFFF`; texto `#111827`; secundario `#52627A`; azul principal `#2563EB`; azul claro `#EAF1FF`; error `#B42318`. Inter: títulos 20–24 px, cuerpo 12–15 px, rótulos 11 px. Espaciado base 8 px, radios 8–12 px, sin elevación. Estados siempre etiquetados con texto.

**Regla funcional:** selector, formulario y estados usan ventana de 430 × 610 px. Seleccionar globalmente la clave SSH de `github.com` y la cuenta `gh`; normalizar los remotos con alias y conservar la identidad de commits definida por las reglas `includeIf` actuales. La sesión `gh` es un estado aparte. El acceso al selector no usa un atajo global sin verificación.
