# La Puerta Trasera

Diorama japonés interactivo renderizado por CPU con un ray tracer en Rust. Un café cambia durante siete días: aparecen flores, decoraciones y tres pistas que desbloquean una habitación secreta. La escena se construye con primitivas y reutiliza texturas locales o patrones procedurales.

## Ejecutar

Desde la raíz del repositorio:

```powershell
cargo run --release --offline
```

`--offline` requiere las dependencias ya descargadas. En una instalación nueva, ejecutar primero `cargo build --release` con acceso a la red.

## Controles

| Control | Acción |
| --- | --- |
| Flechas | Orbitar la cámara |
| Clic izquierdo + arrastre | Orbitar; superar 5 píxeles cancela la interacción |
| Rueda | Zoom |
| Clic izquierdo, al soltar | Recoger una pista o abrir/cerrar la puerta |
| F1 / F2 / F3 / F4 | Amanecer / día / atardecer / noche |
| F5–F11 | Seleccionar los días 1–7 y su fase narrativa |
| 8 | Render completo |
| 9 | Render sin overlay |
| 0 | Render sin textura base ni mapas normales/especulares |
| Esc | Salir |

La progresión temporal es manual. Cambiar de día conserva las pistas encontradas. Reiniciar la aplicación inicia una partida nueva. Los cambios de día o fase tienen un fundido de 0.55 segundos entre imágenes calculadas; calcular el nuevo render puede tardar antes de empezar el fundido.

## Recorrido de demostración

| Día / tecla | Fase inicial | Contenido acumulativo |
| --- | --- | --- |
| 1 / F5 | Día | Café, jardín, cerezos, bambú, faroles y patio |
| 2 / F6 | Día | Flores asagao y ayame |
| 3 / F7 | Día | Recipientes y maceta |
| 4 / F8 | Atardecer | Primera marca luminosa en la puerta trasera |
| 5 / F9 | Atardecer | Higanbana, primera pista en la esquina trasera del café |
| 6 / F10 | Noche | Placa/cuadro junto al farol derecho, segunda pista |
| 7 / F11 | Noche | Pergamino en el patio, tercera pista |

1. Comenzar en el día 1 y mostrar el café con órbita y zoom.
2. Avanzar con F6, F7 y F8 para mostrar flores, decoración y atardecer.
3. En F9, buscar la higanbana roja junto a la esquina trasera derecha del café. Orbitar permite verla de cerca.
4. En F10, recoger el cuadro junto al farol derecho.
5. En F11, recoger el pergamino del patio.
6. Con **PISTAS 3 / 3**, orbitar a la parte trasera y hacer clic en la puerta de madera situada en el centro del café.
7. La puerta se desplaza y aparece el interior como un corte abierto del diorama. La cámara encuadra automáticamente la habitación; siguen disponibles órbita y zoom.
8. Mostrar el espejo, la lámpara reflejada, el objeto metálico y el recipiente de vidrio que deforma el panel y su sello rojo.

La primera pista admite cualquier fase desde el día 5. La segunda requiere amanecer, atardecer o noche desde el día 6. La tercera requiere noche desde el día 7. Una pista recogida permanece oculta. La puerta no abre hasta reunir las tres; hacer clic de nuevo en su hoja desplazada la cierra y oculta el interior y su luz.

La UI usa una fuente bitmap embebida, sin dependencias adicionales: día en japonés, español debajo, contador desde 0/3, estado de la puerta y una indicación discreta. Texto amarillo con contorno negro.

## Habitación y materiales

Se integra `build_secret_room_interior` en la misma `Scene`. Contiene piso y paredes con `assets/paredes/pared_sótano.png`, marco de madera, espejo de metal con reflectividad 0.94, objeto metálico, vidrio con transparencia 0.94 e IOR 1.5, lámpara emisiva y tres recuerdos de las pistas. El panel detrás del vidrio reutiliza `cuadro_1.png`.

El espejo refleja objetos reales de la habitación. La forma curva del vidrio permite distinguir la refracción frente a las líneas del sello. La luz del interior solo se activa al abrir. Los paneles de papel de los faroles emiten luz visible al atardecer y de noche; dos luces puntuales acompañan los faroles.

## Render y arquitectura

- Rust 2021; `nalgebra-glm`, `minifb`, `image` y `rayon`.
- 800 × 600, perspectiva de 60°, render paralelo por píxel y actualización cuando cambia el estado.
- Cubos, esferas, planos limitados, cilindros y pirámides; cono disponible pero sin uso en la escena.
- Transformaciones, sombras duras, iluminación ambiental/difusa/especular, emisión, reflexión, refracción y Fresnel. Profundidad recursiva máxima configurada en 3.
- Color lineal y mapeo tonal; soporte opcional de mapas normales, especulares y overlays.
- Cuatro fases de skybox con cuatro capas de imagen por fase, transparencia y gradiente de respaldo.
- `main.rs`: composición, render, controles y comandos de captura.
- `core/scene.rs`: días, fases, visibilidad, puerta e interior.
- `core/interaction.rs` y `core/picking.rs`: progreso, gesto clic/arrastre y picking.
- `core/hud.rs`: texto bitmap y contador.
- `assets/builders.rs`: geometría del café, jardín, pistas e interior.
- `materials/` y `shapes/`: materiales, texturas e intersecciones existentes.

No se agregaron dependencias, primitivas, física ni un renderer alternativo. El motor conserva sus límites: una muestra por píxel, recorrido lineal de objetos, sombras opacas incluso para vidrio y luces puntuales sin atenuación por distancia. Los PNG con alfa solo se recortan por transparencia en el skybox, no en las superficies de objetos.

## Capturas reproducibles

```powershell
cargo build --release --offline
1..7 | ForEach-Object { & .\target\release\ray-tracer-cube.exe "--render-day$_" }
.\target\release\ray-tracer-cube.exe --render-secret-room
```

Los comandos generan `target/day1.png` a `target/day7.png`, con días/fases reales, UI y pistas todavía sin recoger. `--render-secret-room` prepara el día 7, recoge las pistas mediante `Scene::discover_clue`, abre con `Scene::toggle_secret_room` y guarda `target/secret-room.png` desde la cámara del interior. Es una captura de demostración, no una escena independiente.

También se conservan `--render-day1-flat`, `--render-day1-textured`, `--render-day1-rear`, `--test-shadows` y `--test-materials`.

## Validación

```powershell
cargo fmt --check
cargo check --offline
cargo test --offline
cargo build --release --offline
```

Las pruebas incluyen intersecciones, sombras, cámara, picking, materiales, fases y progreso. Se agregaron comprobaciones del recorrido completo con picking sobre la escena real, bloqueo de puerta, apertura/cierre y luz del interior, persistencia de pistas, separación clic/arrastre, extremos del fundido y diferencias entre renders diarios sin UI.

La compilación, las pruebas automatizadas y la inspección de PNG son verificaciones distintas: las pruebas no sustituyen una demostración manual en la ventana.
