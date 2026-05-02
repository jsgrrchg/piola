// Variables y constantes
wea x = 10
duro PI = 3.1415

// Operadores aritméticos
wea suma = x + 5
wea resta = x - 3
wea producto = x * 2
wea division = x / 4
wea modulo = x % 3

altiro("Suma: " + suma)
altiro("Resta: " + resta)
altiro("Producto: " + producto)
altiro("División: " + division)
altiro("Módulo: " + modulo)

// Booleanos y condicionales
wea edad = 20
cachai (edad >= 18 y edad < 65) {
  altiro("Estai en edad pa trabajar")
} si no {
  altiro("No estai en edad pa trabajar")
}

// Bucle mientras
wea contador = 0
mientras (contador < 3) {
  altiro("Contador: " + contador)
  contador = contador + 1
}

// Listas
wea mi_lista = ["uno", "dos", "tres"]
altiro("Largo lista: " + largo(mi_lista))
para (item en mi_lista) {
  altiro(item)
}

// Mapas
wea persona = {"nombre": "Zalo", "edad": 69}
altiro("Nombre: " + persona["nombre"])
altiro("Edad: " + persona["edad"])

// Tipos
altiro(cachar(42))
altiro(cachar("wena"))
altiro(cachar(verdad))
altiro(cachar(nada))
altiro(cachar([1, 2]))

// Manejo de errores
ojo {
  wea resultado = 10 / 0
} cago(error) {
  altiro("Capturé error: " + error)
}