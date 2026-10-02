# La Puerta Trasera

*Un diorama japonés interactivo donde cada día revela un nuevo secreto.*

**La Puerta Trasera** es una experiencia narrativa y visual construida desde cero utilizando un motor propio de **Ray Tracing por software en Rust**. A diferencia de una escena 3D estática convencional, este proyecto es un entorno vivo: combina la estética artesanal de un diorama de inspiración japonesa con iluminación dinámica, físicas ópticas (reflexiones, refracciones y sombras) y un sistema narrativo interactivo.

A medida que el usuario explora el jardín y el café, el paso de los días y las fases del cielo revelan pistas ocultas. Al recolectar todas las pistas, se desbloquea el acceso a un misterioso sótano secreto: una oficina de estilo Yakuza completamente explorable en primera persona.

---

## Demo visual

Mira el recorrido de la experiencia aquí: https://youtu.be/eH9zXlx_cFc

A lo largo de la experiencia, podrás descubrir:
* Exploración interactiva del jardín exterior y cambios de ciclo día/noche.
* Físicas de luz en tiempo real: faroles de papel emisivos, agua, metal y cristales refractantes.
* Transición al interior del café japonés con perspectiva en primera persona.
* Desbloqueo y descenso a la habitación secreta subterránea.

---

## Características principales

### Escena y exploración
* **Manejo de Cámara:** Control orbital fluido con ratón o teclado, zoom con la rueda del ratón.
* **Cámaras en Primera Persona (FPP):** Modos inmersivos para explorar el interior del café y la habitación secreta subterránea.
* **Interacción Espacial:** Sistema de *picking* mediante lanzamiento de rayos para interactuar con puertas, recolectar objetos narrativos (pistas) y entrar a edificios.
* **Primitivas Geométricas:** Escena construida puramente con matemáticas exactas usando cubos, esferas, cilindros, planos y pirámides.

### Ray tracing e iluminación
El motor fue desarrollado desde cero y renderiza por CPU:
* **Iluminación Dinámica:** Modelo de sombreado Phong (Ambiental, Difuso, Especular) con múltiples fuentes de luz puntuales y sombras duras direccionales.
* **Reflexión y Refracción:** Cálculos recursivos reales para espejos y vidrios (con Índice de Refracción / Fresnel aproximado).
* **Materiales Emisivos:** Lámparas de papel (*washi*) y faroles que actúan como emisores de color en la habitación.
* **Alpha Cutout:** Soporte real de texturas con canal alfa para follaje, hojas de sakura y bambú sin polígonos adicionales.
* **Multihilo:** Renderizado altamente paralelizado píxel a píxel utilizando 
ayon para mantener tasas de refresco fluidas.

### Materiales y texturas
* **Texturizado Híbrido:** Soporte para cargar imágenes planas (suelos de madera, paredes de piedra, tatami) combinadas con patrones procedurales.
* **Propiedades Físicas:** Control de reflectividad, transparencia, índice de refracción (IOR) y factor albedo.

### Experiencia narrativa
* **Sistema de Tiempo (7 Días):** La geometría de la escena evoluciona. Cada día aparecen nuevos objetos, plantas y vegetación.
* **Fases del Cielo (Skybox):** Amanecer, Día, Atardecer y Noche. La luz ambiental interactúa con el entorno.
* **Sistema de Pistas:** Elementos ocultos que el jugador debe encontrar haciendo clic sobre ellos.
* **El Café Japonés:** Entorno interior completamente modelado y explorable.
* **La Habitación Secreta:** Un espacio subterráneo (oficina Yakuza) con jardín zen interior, lámparas esquineras, caja fuerte y pergaminos, desbloqueable tras resolver el misterio.

---

## Arquitectura del proyecto

El motor se organiza en una arquitectura modular limpia, separando la matemática pura de trazado de rayos de la lógica de negocio y narrativa:

`	ext
La Puerta Trasera/
├── Cargo.toml
├── src/
│   ├── main.rs                 # Bucle principal, inicialización de ventana y control de input.
│   ├── core/                   # Lógica central del motor
│   │   ├── camera.rs           # Transformaciones de vista y proyección.
│   │   ├── framebuffer.rs      # Buffer de píxeles y utilidades de renderizado.
│   │   ├── hud.rs              # UI, fuentes bitmap, overlay de menús japoneses.
│   │   ├── interaction.rs      # Gestor de estado narrativo (días, pistas, desbloqueos).
│   │   ├── object.rs           # Definición de entidades 3D y Transform.
│   │   ├── picking.rs          # Interacción ratón-espacio 3D.
│   │   ├── ray_intersect.rs    # Trait fundamental para colisión Rayo-Primitiva.
│   │   └── scene.rs            # Contenedor del mundo, luces e interpolaciones.
│   ├── materials/              # Definición óptica (Color, texturas, presets físicos).
│   ├── shapes/                 # Matemáticas de intersección (Cubo, Esfera, Cilindro, etc).
│   └── assets/                 # Contenido del juego
│       ├── builders.rs         # Funciones procedurales que ensamblan la escena (Café, Sótano).
│       └── mod.rs
└── assets/                     # (Carpeta externa) Texturas, vegetación, UI y transiciones 2D.
`

### Flujo de renderizado (Render Pipeline)

`mermaid
graph TD;
    Input[Input del Usuario] --> State[Actualización de GameState];
    State --> Scene[Transformación de Escena];
    Scene --> Raycast[Lanzamiento de Rayos Primarios];
    
    Raycast --> Intersect{Intersección?};
    Intersect -- Sí --> Bounce[Rayos Secundarios: Sombra, Reflejo, Refracción];
    Bounce --> Color[Cálculo Phong + Textura];
    Intersect -- No --> Skybox[Muestreo de Skybox];
    
    Color --> Buffer[Framebuffer];
    Skybox --> Buffer;
    
    Buffer --> UI[Overlay de UI y Menú];
    UI --> Window[Presentación en Pantalla];
`

---

## Requisitos e instalación

El proyecto está escrito íntegramente en Rust. No requiere bibliotecas gráficas pesadas (Vulkan/OpenGL/DirectX), ya que todo el cálculo de luz se ejecuta matemáticamente en el procesador mediante la CPU.

### Prerrequisitos
* [Rust y Cargo](https://rustup.rs/) (Edición 2021 o superior).

### Instalación
1. Clona el repositorio:
   `ash
   git clone <URL_DEL_REPOSITORIO>
   cd Proyecto-2-Raytracing
   `

2. Ejecuta el juego en modo Release (obligatorio para un rendimiento fluido debido al costo de cómputo del ray tracing):
   `ash
   cargo run --release
   `

3. **Pruebas unitarias:** Para verificar las matemáticas vectoriales, físicas de intersección y lógicas narrativas del motor:
   `ash
   cargo test --release
   `

*(Nota: Ejecutar el código sin --release provocará caídas extremas de fotogramas, ya que las optimizaciones del compilador son cruciales para un raytracer por software).*

---

## Tabla de controles

Todas las interacciones de cámara y narrativa ocurren directamente en la ventana activa.

| Acción | Control | Condiciones / Notas |
| :--- | :--- | :--- |
| **Orbitar Cámara** | Flechas direccionales o Clic Izquierdo + Arrastre | Disponible en modos de exploración exterior. |
| **Zoom** | Rueda del ratón | Acerque o aleje la lente orbital. |
| **Interactuar / Recoger Pistas** | Clic Izquierdo (soltar) | Usado para puertas, menú y pistas ocultas. |
| **Entrar/Salir del Café** | Tecla C | Intercambia instantáneamente entre cámara de jardín y vista FPP en la barra del café. |
| **Primera Persona en Sótano** | Tecla T | Requiere que el sótano esté abierto. Cambia entre cámara isométrica y visión humana en la habitación. |
| **Forzar Desbloqueo de Sótano** | Tecla F12 | *Modo Debug:* Salta el requerimiento de encontrar las 3 pistas. |
| **Cambiar Día (Narrativa)** | Teclas F1 a F7 | Avanza el estado geométrico (1 = Inicio, 7 = Final). |
| **Cambiar Fase del Cielo** | Teclas F8 a F11 | Amanecer (F8), Día (F9), Atardecer (F10), Noche (F11). |
| **Ocultar Interfaz (UI)** | Tecla 8, 9 | Alternar modos de depuración sin UI/overlays. |
| **Salir del Juego** | Esc | Cierra la aplicación de inmediato. |

---

## Documentación académica y créditos

Este proyecto explora aplicaciones fundamentales en **Gráficos por Computadora**, implementando desde cero:
* **Matemáticas espaciales lineales:** Transformaciones vectoriales con 
algebra-glm.
* **Algoritmos de intersección:** Cálculo de raíces algebraicas (cuadráticas) para esferas y cilindros; y cálculo delimitado por losas (Slab method) para paralelepípedos.
* **Modelo físico de iluminación de Phong:** Suma analítica de componentes emisivas, luz ambiental, reflexión difusa lambertiana y destellos especulares.
* **Fenómenos recursivos:** La función cast_ray se llama a sí misma para simular rebotes continuos de luz en espejos y refracción a través de sólidos transparentes (vidrio/agua), incluyendo corrección por índice de refracción (IOR).

**Créditos:** Texturas fotográficas obtenidas de repositorios libres (ej. Unsplash) procesadas para su integración volumétrica en el diorama. Las fuentes y el menú han sido traducidos y estilizados mediante herramientas 2D en local.

---

## Limitaciones y trabajo futuro

* **Rendimiento atado a CPU:** Al carecer de aceleración por GPU y depender puramente de 
ayon, la tasa de fotogramas es sensible a la resolución de pantalla y la cantidad de luces activas simultáneamente en el interior.
* **Anti-Aliasing:** En su estado actual solo se procesa un rayo por píxel en la región central (sin Multisampling), lo que genera bordes dentados nativos.
* **Trabajo Futuro:** Implementación de estructuras de aceleración de rayos (BVH o Grillas espaciales) para permitir cargar miles de primitivas sin penalización matemática exponencial, así como texturas de mapas normales en superficies no planas.

