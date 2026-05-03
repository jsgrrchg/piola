# Diseño de Features

Este documento describe las features planificadas para Piola, su sintaxis propuesta, semántica y casos edge. Sirve como referencia de diseño antes de implementación. Cada feature puede cambiar antes de ser implementada, pero las decisiones registradas acá representan el estado actual del diseño.

---

## 1. Interpolación de strings

### Motivación

Concatenar strings con `+` funciona, pero es tedioso cuando hay múltiples variables involucradas.

```
altiro("Hola, " + nombre + ". Tienes " + edad + " años.")
altiro($"Hola, {nombre}. Tienes {edad} años.")
```

### Sintaxis

Un string interpolado lleva `$` antes de la comilla de apertura. Las expresiones van entre llaves `{}`. Dentro de `{}` puede ir cualquier expresión válida de Piola.

```
wea nombre = "Zalo"
wea edad = 69

altiro($"Hola, {nombre}.")
altiro($"El doble de {edad} es {edad * 2}.")
altiro($"Tipo: {cachar(nombre)}")
```

### Semántica

El intérprete expande un string interpolado en tiempo de ejecución a una concatenación de partes. `$"Hola, {nombre}."` equivale internamente a `"Hola, " + nombre + "."`. Cada expresión dentro de `{}` se evalúa y convierte a texto con la misma lógica que `altiro`.

### Casos edge

**Llaves literales:** Se escapan duplicándolas.

```
altiro($"El mapa usa llaves: {{ y }}")  // → "El mapa usa llaves: { y }"
```

**Expresiones vacías:** `$"texto {}"` es un error de sintaxis.

**Strings anidados:** Se permiten strings dentro de la interpolación usando comillas del otro tipo.

```
altiro($"El resultado es {'piola'}")
```

---

## 2. Funciones anónimas (lambdas)

### Motivación

Sin lambdas no se puede pasar comportamiento como valor. Esto bloquea patrones básicos como filtrar una lista, transformar elementos o definir callbacks.

### Sintaxis

Inspirada en Java. Parámetros entre paréntesis, cuerpo después de `->`. Si el cuerpo es una sola expresión, el resultado es el valor de esa expresión. Si necesita múltiples pasos, se usa un bloque con llaves.

```
wea duplicar = (x) -> x * 2
wea sumar    = (a, b) -> a + b

wea saludar = (nombre) -> {
  wea saludo = $"Wena, {nombre}"
  saludo
}

wea hablar = () -> altiro("piola")
```

### Semántica

Una lambda es un valor de tipo `pega`, equivalente a una función declarada con `pega`. Captura el entorno donde fue definida (closure léxico).

```
wea factor = 3
wea multiplicar = (x) -> x * factor   // captura 'factor'
altiro(multiplicar(5))                 // → 15
```

### Casos edge

**Lambda asignada a `duro`:**

```
duro duplicar = (x) -> x * 2
duplicar = (x) -> x * 3   // error: 'duplicar' es duro
```

**Recursión en lambdas:** Una lambda no puede referenciarse a sí misma por nombre. Para recursión, usar `pega` nombrada.

---

## 3. Métodos sobre colecciones

### Motivación

Las funciones globales (`largo(lista)`) son menos intuitivas que los métodos sobre el objeto (`lista.largo()`). Los métodos también permiten encadenamiento.

### Sintaxis

Acceso con punto para métodos de instancia. Acceso con `::` para constructores y funciones asociadas al tipo.

```
wea mi_lista = Lista::nueva()
wea mi_mapa  = Mapa::nuevo()

mi_lista.agregar(1)
mi_lista.largo()
mi_mapa.insertar("pais", "Chile")
mi_mapa.obtener("pais")
```

### Métodos de Lista

| Método                 | Descripción              | Retorna    |
|------------------------|--------------------------|------------|
| `Lista::nueva()`       | Crea una lista vacía     | `lista`    |
| `.agregar(valor)`      | Agrega al final          | `nada`     |
| `.largo()`             | Número de elementos      | `numero`   |
| `.obtener(i)`          | Elemento en índice `i`   | `valor`    |
| `.filtrar(fn)`         | Filtra con una lambda    | `lista`    |
| `.mapear(fn)`          | Transforma cada elemento | `lista`    |
| `.reducir(fn, inicio)` | Reduce a un valor        | `valor`    |
| `.contiene(valor)`     | Si el valor existe       | `booleano` |
| `.invertir()`          | Invierte el orden        | `lista`    |
| `.unir(sep)`           | Une elementos como texto | `texto`    |

### Métodos de Mapa

| Método                    | Descripción         | Retorna    |
|---------------------------|---------------------|------------|
| `Mapa::nuevo()`           | Crea un mapa vacío  | `mapa`     |
| `.insertar(clave, valor)` | Inserta o reemplaza | `nada`     |
| `.obtener(clave)`         | Valor por clave     | `valor`    |
| `.contiene(clave)`        | Si la clave existe  | `booleano` |
| `.eliminar(clave)`        | Elimina la clave    | `nada`     |
| `.claves()`               | Lista de claves     | `lista`    |
| `.valores()`              | Lista de valores    | `lista`    |
| `.largo()`                | Número de pares     | `numero`   |

### Métodos de Texto

| Método               | Descripción                  | Retorna    |
|----------------------|------------------------------|------------|
| `.largo()`           | Número de caracteres         | `numero`   |
| `.mayusculas()`      | Convierte a mayúsculas       | `texto`    |
| `.minusculas()`      | Convierte a minúsculas       | `texto`    |
| `.contiene(sub)`     | Si contiene el subtexto      | `booleano` |
| `.empieza_con(sub)`  | Si empieza con el subtexto   | `booleano` |
| `.termina_con(sub)`  | Si termina con el subtexto   | `booleano` |
| `.reemplazar(de, a)` | Reemplaza ocurrencias        | `texto`    |
| `.separar(sep)`      | Divide en lista              | `lista`    |
| `.recortar()`        | Quita espacios en los bordes | `texto`    |

### Semántica

Los métodos que reciben lambdas pasan el elemento como primer argumento, y el índice como segundo cuando se necesita.

```
wea lista = [1, 2, 3, 4, 5]

wea pares  = lista.filtrar((n) -> n % 2 == 0)
wea dobles = lista.mapear((n) -> n * 2)
wea suma   = lista.reducir((acc, n) -> acc + n, 0)
```

El encadenamiento es válido cuando el método retorna una colección.

```
wea resultado = lista
  .filtrar((n) -> n > 2)
  .mapear((n) -> n * 10)
```

### Casos edge

**`.obtener` fuera de rango:** Retorna `nada`. Para error explícito, usar el índice directo `lista[i]`.

**Métodos sobre literales:** Válido.

```
altiro([1, 2, 3].largo())    // → 3
altiro("piola".mayusculas()) // → "PIOLA"
```

---

## 4. Rangos

### Motivación

El patrón de loop con contador manual es verboso. Los rangos lo reemplazan con algo más directo.

```
// Antes
wea i = 0
mientras (i < 10) {
  altiro(i)
  i = i + 1
}

// Con rangos
para (i en 0..10) {
  altiro(i)
}
```

### Sintaxis

Inspirada en Rust.

```
0..10     // exclusivo: 0, 1, 2, ..., 9
0..=10    // inclusivo: 0, 1, 2, ..., 10
```

Los rangos son valores que se pueden asignar y pasar como argumentos.

```
wea rango = 1..=5
para (n en rango) {
  altiro(n)
}
```

### Semántica

Un rango es de tipo `rango`. Los rangos son siempre de números enteros; los extremos decimales se truncan.

```
para (i en 0..5)  { altiro(i) }   // 0, 1, 2, 3, 4
para (i en 0..=5) { altiro(i) }   // 0, 1, 2, 3, 4, 5
```

Conversión a lista:

```
wea lista = (1..=5).lista()   // → [1, 2, 3, 4, 5]
```

### Casos edge

**Rango invertido:** `10..0` es un rango vacío, no un rango que cuenta hacia atrás. Para iterar al revés, usar `.invertir()` sobre una lista.

**Rango con paso:** No está en el diseño inicial. Se resuelve con `mapear` sobre un rango base.

```
wea pares = (0..=5).lista().mapear((n) -> n * 2)
```

---

## 5. Pattern matching (`según`)

### Motivación

Los `cachai` anidados para comparar un valor contra múltiples posibilidades son difíciles de leer. `según` los reemplaza con algo limpio y exhaustivo.

### Sintaxis

El valor a comparar va después de `según`. Cada rama tiene un patrón seguido de `->` y el cuerpo.

```
según tipo {
  "numero" -> altiro("es número")
  "texto"  -> altiro("es texto")
  _        -> altiro("otra cosa")
}
```

Cuerpos de múltiples instrucciones usan llaves:

```
según x {
  1 -> altiro("uno")
  2 -> {
    wea doble = x * 2
    altiro($"dos, el doble es {doble}")
  }
  _ -> altiro("otro")
}
```

`según` es una expresión y retorna el valor de la rama ejecutada:

```
wea mensaje = según cachar(x) {
  "numero" -> $"el número es {x}"
  "texto"  -> $"el texto es {x}"
  _        -> "tipo desconocido"
}
```

### Patrones soportados

**Valor literal:**

```
según x {
  1      -> altiro("uno")
  "hola" -> altiro("saludo")
  verdad -> altiro("es verdad")
  nada   -> altiro("no hay nada")
  _      -> altiro("otro")
}
```

**Múltiples valores en una rama**, separados por `|`:

```
según x {
  1 | 2 | 3 -> altiro("entre 1 y 3")
  _          -> altiro("otro")
}
```

**Guardas con condición**, usando `cachai`:

```
según x {
  n cachai (n > 0) -> altiro("positivo")
  n cachai (n < 0) -> altiro("negativo")
  _                -> altiro("cero")
}
```

**Destructuring de `Opcion` y `Resultado`:**

```
según buscar_usuario(id) {
  Opcion::algo(u) -> altiro($"Encontré a {u}")
  Opcion::nada()  -> altiro("No existe")
}
```

### Semántica

`según` evalúa el valor una sola vez y compara con cada patrón en orden. Ejecuta la primera rama que coincide. No hay fall-through.

**Exhaustividad:** El comodín `_` es obligatorio cuando los patrones no cubren todos los casos posibles. Sin `_` y sin coincidencia, es un error en tiempo de ejecución.

**Exhaustividad automática para `Opcion` y `Resultado`:** Si el valor evaluado es de tipo `Opcion` o `Resultado`, el intérprete verifica que ambas variantes estén cubiertas antes de ejecutar. Si falta alguna, avisa con un error descriptivo en vez de ejecutar y fallar silenciosamente.

```
// Esto genera un aviso antes de correr:
según mi_opcion {
  Opcion::algo(v) -> altiro(v)
  // falta Opcion::nada(), el intérprete lo detecta
}

Error: El 'según' sobre un Opcion no cubre el caso Opcion::nada().
       Agrega esa rama o usa el comodín '_'.
```

Esto refuerza el aprendizaje de tipos algebraicos: el lenguaje te enseña a pensar en todos los casos posibles.

### Casos edge

**Comodín no al final:** Error de sintaxis.

**`_` y exhaustividad automática:** Si el valor es `Opcion` o `Resultado`, el comodín `_` sí satisface la exhaustividad automática, pero el intérprete puede emitir un aviso sugiriendo ser explícito.

---

## 6. Parámetros con valor por defecto

### Motivación

Funciones llamadas frecuentemente con los mismos argumentos para ciertos parámetros. Sin defaults hay que repetir el argumento en cada llamada o crear una función wrapper.

### Sintaxis

El valor por defecto se asigna con `=` en la declaración del parámetro. Los parámetros con default van siempre al final.

```
pega saludar(nombre, saludo = "wena") {
  altiro($"{saludo}, {nombre}!")
}

saludar("Zalo")          // → "wena, Zalo!"
saludar("Zalo", "oe")    // → "oe, Zalo!"
```

```
pega conectar(host, puerto = 8080, seguro = falso) {
  // ...
}

conectar("localhost")
conectar("localhost", 3000)
conectar("localhost", 443, verdad)
```

### Semántica

El valor por defecto se evalúa en el momento de la definición de la función, no en cada llamada.

### Casos edge

**Parámetro sin default después de uno con default:** Error de sintaxis.

```
pega malo(a = 1, b) { }   // error
```

**Default que depende de otro parámetro:** No soportado.

---

## 7. Null coalescing (`??`)

### Motivación

El patrón de "usa este valor si existe, si no usa este otro" es muy común con `nada`.

```
wea nombre = obtener_nombre() ?? "invitado"
```

### Sintaxis

Operador binario `??`. Si el lado izquierdo es `nada`, retorna el lado derecho. Si no, retorna el lado izquierdo.

```
wea resultado = valor ?? "por defecto"
wea x = a ?? b ?? c ?? "último recurso"
```

### Semántica

`??` tiene cortocircuito: el lado derecho no se evalúa si el izquierdo no es `nada`. Tiene menor precedencia que los operadores aritméticos y de comparación, pero mayor que `y` y `o`.

### Casos edge

**`falso` y `0` no son `nada`:**

```
wea x = falso ?? "otro"   // → falso
wea y = 0 ?? "otro"       // → 0
wea z = nada ?? "otro"    // → "otro"
```

---

## 8. Destructuring en `para`

### Motivación

Al iterar sobre listas de pares, extraer los elementos manualmente es verboso.

```
// Sin destructuring
para (par en pares) {
  wea num   = par[0]
  wea texto = par[1]
  altiro($"{num}: {texto}")
}

// Con destructuring
para ([num, texto] en pares) {
  altiro($"{num}: {texto}")
}
```

### Sintaxis

El patrón va donde normalmente va la variable de iteración.

```
wea coordenadas = [[1, 2], [3, 4], [5, 6]]
para ([x, y] en coordenadas) {
  altiro($"x={x}, y={y}")
}
```

Al iterar sobre un mapa, cada elemento es un par `[clave, valor]`:

```
wea persona = {"nombre": "Zalo", "edad": 69}
para ([clave, valor] en persona) {
  altiro($"{clave}: {valor}")
}
```

Ignorar elementos con `_`:

```
para ([_, y] en coordenadas) {
  altiro(y)
}
```

### Casos edge

**Número de elementos no coincide:** Error en tiempo de ejecución.

```
Error: No podi desestructurar un elemento de largo 3 en un patrón de largo 2.
```

**Anidamiento:** No está en el diseño inicial. Un solo nivel es suficiente para la mayoría de los casos.

---

## 9. Módulos (`meter`)

### Motivación

Sin módulos, Piola no escala más allá de un archivo.

### Sintaxis

`meter` importa otro archivo `.cl`. La extensión es implícita. La ruta es relativa al archivo actual.

```
meter utils
meter math/vectores
```

Para evitar colisiones de nombres, se importa con alias:

```
meter utils como u
u::saludar("Zalo")
```

### Semántica

`meter` ejecuta el archivo importado una sola vez, aunque sea referenciado múltiples veces. El resultado queda cacheado.

**Política de colisión: falla fuerte.** Si el intérprete detecta que un nombre exportado por el módulo importado ya existe en el scope actual, lanza un error y obliga a usar alias. Esto es intencional: en un lenguaje educativo, dejar que una función sobrescriba silenciosamente a otra es una fuente de bugs frustrantes y difíciles de rastrear.

```
// Si 'saludar' ya existe en el scope actual:
meter utils   // error: el nombre 'saludar' ya existe

meter utils como u   // correcto
u::saludar("Zalo")
```

Las importaciones circulares son un error:

```
Error: Importación circular detectada: a.cl → b.cl → a.cl.
```

### Evolución futura

Una versión posterior podría soportar imports específicos al estilo Rust:

```
meter utils::saludar
meter utils::{ saludar, despedir }
```

Esta sintaxis requiere que el intérprete entienda qué símbolos exporta cada módulo antes de ejecutarlo, lo que implica un análisis previo más complejo.

### Casos edge

**Archivo no encontrado:** Error con la ruta que se intentó resolver.

**Sin alias, con colisión:** Error explícito que indica qué nombre colisiona y sugiere usar `como`.

---

## 10. Propagación de errores (`?`)

### Motivación

El patrón `ojo`/`cago` es explícito, pero verboso cuando solo se necesita propagar un error hacia arriba sin manejarlo.

```
// Sin propagación
pega leer_config(ruta) {
  ojo {
    wea contenido = abrir_archivo(ruta)
    contenido
  } cago(e) {
    pifia e
  }
}

// Con propagación
pega leer_config(ruta) {
  wea contenido = abrir_archivo(ruta)?
  contenido
}
```

### Sintaxis

El operador `?` se coloca al final de una expresión que puede fallar.

```
pega procesar(ruta) {
  wea datos  = leer_archivo(ruta)?
  wea parsed = parsear(datos)?
  parsed
}
```

### Semántica

`?` solo es válido dentro de una `pega`. Si la expresión produce un error, la función retorna ese error inmediatamente. Si no hay error, el valor continúa.

### Casos edge

**`?` fuera de una pega:** Error de sintaxis.

**`?` sobre un `Resultado`:** Si el resultado es `Resultado::error(e)`, propaga `e`. Si es `Resultado::ok(v)`, devuelve `v`.

---

## 11. Resultado y Opción como valores

### Motivación

A veces una función naturalmente puede retornar "algo o nada" sin que eso sea un error. `Opcion` modela ese caso. `Resultado` modela operaciones que pueden fallar con información del error.

### Sintaxis

```
wea encontrado = Opcion::algo(42)
wea vacio      = Opcion::nada()

wea ok    = Resultado::ok(100)
wea error = Resultado::error("algo salió mal")
```

Se combinan con `según`:

```
según buscar_usuario(id) {
  Opcion::algo(u) -> altiro($"Encontré a {u}")
  Opcion::nada()  -> altiro("No existe")
}

según dividir(a, b) {
  Resultado::ok(v)    -> altiro($"Resultado: {v}")
  Resultado::error(e) -> altiro($"Error: {e}")
}
```

### Semántica

`Opcion` y `Resultado` son tipos de primera clase. Se pueden asignar, pasar como argumentos y retornar desde funciones.

`Opcion::nada()` es distinto de `nada`. `nada` es la ausencia de valor. `Opcion::nada()` es un valor que representa explícitamente la ausencia dentro de un contexto donde también podría haber algo.

El operador `?` funciona con `Resultado`: si es `Resultado::error(e)`, propaga el error. Si es `Resultado::ok(v)`, devuelve `v`.

---

## 12. Lanzar errores (`pifia`)

### Motivación

`ojo`/`cago` maneja errores que ocurren, pero no había forma de lanzar un error intencionalmente desde código Piola. `pifia` llena ese hueco.

En Chile, una "pifia" es un error, un defecto, algo que salió mal. Es la palabra exacta.

### Sintaxis

```
pifia "Mensaje del error"
pifia $"El valor {x} no es válido"
```

Dentro de una `pega` con `?`:

```
pega dividir(a, b) {
  cachai (b == 0) {
    pifia "No se puede dividir por cero, weon"
  }
  a / b
}
```

### Semántica

`pifia` lanza un error que interrumpe la ejecución normal. Ese error puede ser capturado con `ojo`/`cago` en cualquier nivel de la cadena de llamadas, o propagado con `?`.

El mensaje de error en la consola sigue el tono del lenguaje:

```
Error: Hubo una pifia: No se puede dividir por cero, weon.
```

`pifia` acepta cualquier expresión que evalúe a texto. Si recibe un `Resultado::error(e)`, usa el mensaje de `e`.

### Casos edge

**`pifia` dentro de `ojo`:** El error lanzado es capturado por el `cago` del mismo bloque `ojo` que lo contiene.

**`pifia` sin mensaje:** Error de sintaxis. Siempre requiere un texto.

---

## 13. Documentación (`copucha`)

### Motivación

Un lenguaje sin forma de documentar su código se documenta en comentarios sueltos que nadie lee. `copucha` es el sistema de documentación oficial de Piola.

En Chile, "copucha" es el detalle de algo, el chisme, lo que vale la pena saber. Es exactamente lo que hace la documentación: te cuenta la copucha de cómo funciona algo.

### Sintaxis

Los comentarios de documentación usan `#?` y van inmediatamente antes de la declaración que documentan.

```
#? Suma dos números y retorna el resultado.
pega sumar(a, b) {
  a + b
}

#? El número pi con suficiente precisión pa' la mayoría de los casos.
duro PI = 3.14159

#? Busca un usuario por id. Retorna Opcion::nada() si no existe.
pega buscar_usuario(id) {
  // ...
}
```

Para documentación de múltiples líneas, se usan varios `#?` consecutivos:

```
#? Calcula el factorial de n.
#? n debe ser un número entero no negativo.
#? Si n es negativo, lanza una pifia.
pega factorial(n) {
  // ...
}
```

### La herramienta `piola-copucha`

`piola-copucha` es una herramienta CLI separada que extrae todos los comentarios `#?` de los archivos `.cl` y genera documentación estática en HTML.

```
piola-copucha src/           # genera docs/ con la documentación del proyecto
piola-copucha src/ --salida docs_publicas/
```

La salida es un sitio HTML estático sin dependencias externas: un archivo por módulo, con navegación entre funciones y constantes.

### Casos edge

**`#?` sin declaración a continuación:** Se ignora con un aviso.

**Múltiples `#?` no consecutivos:** Solo el bloque inmediatamente anterior a la declaración se considera documentación de esa declaración.

---

## 14. Referencia al objeto actual (`voh_mismo`)

### Motivación

Cuando Piola implemente estructuras con métodos de instancia, se necesita una forma de referirse al objeto actual desde dentro de un método. `self` y `this` suenan a cualquier otro lenguaje. `voh_mismo` es la traducción directa y natural.

### Sintaxis

```
pega inicializar(nombre) {
  voh_mismo.nombre = nombre
}

pega saludar() {
  altiro($"Wena, soy {voh_mismo.nombre}")
}
```

### Semántica

`voh_mismo` es una variable especial disponible solo dentro de métodos de instancia. Referencia al objeto sobre el que se está llamando el método.

Fuera de un método de instancia, `voh_mismo` no existe y su uso es un error.

```
Error: 'voh_mismo' solo existe dentro de un método de instancia, weon.
```

### Nota de implementación

Esta feature depende de que Piola implemente estructuras o tipos definidos por el usuario, lo que no está en el alcance de la versión actual. Está documentada acá para reservar la keyword y el concepto antes de que haya colisión con otros nombres.

---

## 15. Control de flujo en bucles (`cortala` y `sigue`)

### Motivación

`break` y `continue` suenan a términos técnicos. `cortala` y `sigue` son comandos directos que cualquier chileno usaría para detener una acción o pedir que se continúe.

### Sintaxis

```
para (i en 0..10) {
  cachai (i == 5) { cortala }   // detiene el bucle
  cachai (i < 2)  { sigue }     // salta a la siguiente iteración
  altiro(i)
}
// imprime: 2, 3, 4
```

Funcionan igual en `mientras`:

```
wea i = 0
mientras (verdad) {
  cachai (i >= 5) { cortala }
  altiro(i)
  i = i + 1
}
```

### Semántica

**`cortala`:** Termina el bucle más cercano inmediatamente. La ejecución continúa después del bloque del bucle.

**`sigue`:** Salta el resto del cuerpo del bucle actual y pasa a la siguiente iteración.

Ambas palabras solo son válidas dentro de un `para` o un `mientras`. Fuera de un bucle, son un error de sintaxis.

```
Error: 'cortala' solo tiene sentido dentro de un bucle, po.
```

### Casos edge

**Bucles anidados:** `cortala` y `sigue` afectan al bucle más inmediato, no al exterior.

```
para (i en 0..3) {
  para (j en 0..3) {
    cachai (j == 1) { cortala }   // termina el bucle de j, no el de i
    altiro($"{i},{j}")
  }
}
```

**`cortala` dentro de `ojo`:** Si `cortala` está dentro de un `ojo` que a su vez está dentro de un bucle, el comportamiento es que `cortala` sale del bucle. Él `ojo` no interfiere.