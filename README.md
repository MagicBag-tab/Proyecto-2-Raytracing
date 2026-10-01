# La Puerta Trasera

## Description

**La Puerta Trasera** es un diorama japonés en miniatura renderizado con un ray tracer escrito en Rust. Durante el día parece un pequeño café rodeado por un jardín; al cambiar la hora aparecen pistas que conducen a una puerta trasera y a una habitación secreta.

La escena se construye con primitivas geométricas. No requiere modelos 3D externos. Las texturas de imagen son opcionales: mientras no estén disponibles, se usan patrones procedurales.

## Features

- Diorama con café, jardín, árboles, cerezos, mesas, sillas, cercas, flores y faroles.
- Orbitación de cámara con teclado y mouse, y zoom con la rueda.
- Picking por rayo y selección del objeto visible más cercano.
- Tres pistas con visibilidad ligada a día, atardecer y noche.
- Puerta trasera bloqueada hasta encontrar las tres pistas.
- Habitación secreta oculta que se revela al abrir la puerta.
- Caja seleccionable y movible sin física.
- Ciclo de iluminación día, atardecer y noche.
- Render paralelo por píxel y tres modos de render existentes.

## Controls

- `Left click`: seleccionar un objeto o interactuar; las pistas se recogen al hacer clic.
- `Left click + drag`: orbitar la cámara.
- `Mouse wheel`: acercar o alejar la cámara.
- `Arrow keys`: orbitar la cámara.
- `W`, `A`, `S`, `D`: mover la caja cuando está seleccionada.
- `Tab`: alternar entre día, atardecer y noche.
- `1`, `2`, `3`: cambiar el modo de render.
- `Esc`: cerrar la ventana.

La barra de título muestra la fase, el contador de pistas y el estado de la puerta. Los eventos de interacción también se imprimen en la consola.

## Ray Tracing Features

El motor incluye intersecciones de rayo con cubos, esferas, planos, cilindros, conos, pirámides y triángulos; transformaciones de objeto; sombreado ambiental, difuso y especular; sombras; emisión; reflexión; refracción; Fresnel y rayos recursivos con profundidad limitada. El render se calcula en paralelo con Rayon.

## Materials

Los presets principales son madera, piedra, metal, vidrio y papel; el césped tiene un material propio. Cada uno conserva su albedo y parámetros especulares. El metal tiene reflectividad y el vidrio tiene transparencia e índice de refracción. Los assets de imagen son opcionales y se cargan desde `assets/textures/`; si faltan, se usan texturas procedurales.

## Reflection

La habitación secreta contiene una superficie decorativa metálica con reflectividad elevada para mostrar los rayos reflejados en un contexto reconocible. Los objetos metálicos del diorama también usan reflexión.

## Refraction

Las ventanas y el recipiente de vidrio de la habitación secreta usan el preset de vidrio, con transparencia `0.94` e índice de refracción `1.5`. El trazador combina transmisión, reflexión de Fresnel y rayos recursivos.

## Skybox

El fondo predeterminado es un gradiente direccional con paletas para día, atardecer y noche. También se admiten imágenes equirectangulares independientes por fase:

- `assets/textures/sky_day.png`
- `assets/textures/sky_sunset.png`
- `assets/textures/sky_night.png`

Si un archivo no existe, se mantiene el gradiente correspondiente.

## Interactive Elements

- **Pista del jardín:** visible en cualquier fase.
- **Nota del café:** aparece en atardecer y noche.
- **Pista oculta:** aparece únicamente de noche.
- **Puerta trasera:** no se abre hasta encontrar las tres pistas; después, cada clic abre o cierra la entrada y revela u oculta la habitación.
- **Caja:** selecciónala y muévela con `W`, `A`, `S`, `D`.

La visibilidad de objetos ocultos también se respeta durante los rayos primarios y las pruebas de sombra.

## Scene

La escena conserva el jardín japonés construido con los builders existentes y añade una entrada trasera con abertura geométrica, pistas provisionales y una habitación secreta de escala reducida. La habitación incluye una superficie reflectante, un recipiente de vidrio y una luz cálida. Estos elementos están hechos con primitivas y sus materiales pueden sustituirse o ajustarse cuando estén listas las texturas y la dirección artística final.

## Architecture

- `src/main.rs`: composición de escena, render, iluminación y bucle de entrada.
- `src/core/camera.rs`: cámara orbital, zoom y generación de rayos por píxel.
- `src/core/picking.rs`: picking y selección de la intersección más cercana.
- `src/core/interaction.rs`: identificadores, tipos interactivos y progreso narrativo.
- `src/core/scene.rs`: objetos, luces, fases, visibilidad, skybox y acciones de escena.
- `src/core/object.rs`: transformaciones e intersección de objetos.
- `src/materials/`: color, presets y carga de texturas.
- `src/shapes/`: primitivas geométricas.
- `src/assets/builders.rs`: builders de casa, árboles, faroles, mesas, sillas y cercas.

## How to Run

Desde la raíz del proyecto:

```powershell
cargo run
```

La primera compilación puede tardar mientras Cargo construye las dependencias.

## Video

Pendiente de añadir el enlace al video de demostración.

## Project Structure

```text
src/
  assets/
    builders.rs
  core/
    camera.rs
    framebuffer.rs
    interaction.rs
    light.rs
    object.rs
    picking.rs
    ray_intersect.rs
    scene.rs
    transform.rs
  materials/
    color.rs
    presets.rs
    texture.rs
  shapes/
  main.rs
assets/
  textures/   # Imágenes opcionales creadas para el proyecto
```

## Texturas que debo crear

Las imágenes se leen desde `assets/textures/`. Los mapas auxiliares son opcionales; deben llamarse con el sufijo indicado para que el material los conecte.

| Archivo | Uso |
| --- | --- |
| `wood.png` | Madera de casa, mesas, sillas, cercas y caja. |
| `stone.png` | Plataforma, piedras y superficies de la habitación secreta. |
| `metal.png` | Faroles, herrajes y espejo decorativo. |
| `glass.png` | Ventanas y recipiente de vidrio. |
| `paper.png` | Paredes, nota y detalles de papel. |
| `grass.png` | Suelo del jardín. |
| `<material>_normal.png` | Mapa normal opcional para un material, por ejemplo `wood_normal.png`. |
| `<material>_specular.png` | Mapa especular opcional, por ejemplo `metal_specular.png`. |
| `<material>_overlay.png` | Máscara de overlay opcional, por ejemplo `glass_overlay.png`; requiere su mapa normal asociado. |
| `<material>_overlay_normal.png` | Normal del overlay, por ejemplo `glass_overlay_normal.png`. |
| `sky_day.png` | Skybox equirectangular diurno. |
| `sky_sunset.png` | Skybox equirectangular de atardecer. |
| `sky_night.png` | Skybox equirectangular nocturno. |

El overlay solo se conecta si existen ambos archivos `_overlay.png` y `_overlay_normal.png` para uno de estos materiales: `wood`, `stone`, `metal`, `glass`, `paper` o `grass`.

## Rubric Mapping

- **Complejidad y creatividad:** escena compuesta, ciclo horario, pistas, puerta y habitación secreta; la valoración subjetiva depende de la presentación final.
- **Atractivo visual:** geometría low-poly, materiales diferenciados, sombras y cambios de iluminación; las texturas y el pulido artístico finales quedan abiertos.
- **Rotación del diorama y zoom:** orbitación con flechas o arrastre y zoom con rueda, con límites de distancia e inclinación.
- **Materiales:** presets de madera, piedra, metal, vidrio y papel, además de césped; cada material usa un patrón o imagen y parámetros propios.
- **Refracción:** vidrio transparente con índice de refracción y Fresnel en ventanas y recipiente.
- **Reflexión:** superficies metálicas reflectantes y un espejo contextual en la habitación.
- **Skybox:** gradiente direccional por fase y soporte para mapas equirectangulares diurnos, de atardecer y nocturnos.

La implementación deja preparados los sistemas para la demostración, pero la calificación final depende de la rúbrica y del resultado visual que se presente.
