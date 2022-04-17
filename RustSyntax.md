
# Rust syntax

Zde jsem se snažil alespoň základně popsat syntax Rustu. </br>
Syntax Rustu je trochu zvláštní, protože kombinuje prvky C++, OCaml, Haskell a trochu i Ruby.

## Definice proměnných

```
let jmeno: datovy typ (není nutné uvádět) = hodnota;

let cislo = 42;
let jmeno: String = "Praha";
```

## Základní datové typy

Základní typy mají explicitní velikost.

```
Unsigned integers - u<počet bitů> - např. u32, u64 ... </br>
Signed integers - i<počet bitů> - např. i32, i64 ... </br>
Floating-point - f<32 / 64> </br>
```

Konverze mezi datovými typy musí být vždy explicitní:

```
let integer: u32 = 1;
let float: f64 = 5.0;

let vysledek = (integer as f64) * float;
// nebo
let vysledek = integer * (float as u32);
```

Rust podobně jako C++ používá název "vector" pro dynamické pole (typ `Vec`).
Pro vektory v matematickém smyslu používám typy `Vec2`, `Vec3`, `Vec4` z knihovny `glam`.

## Mutabilita

V Rustu se mutabilita rozlišuje 2 způsoby:

### Proměnné

```
let mut cislo = 1; // mohu měnit hodnotu proměnné
let cislo = 1;     // konstanta
```

### Reference

```
let mut cislo = 1;

let mut_ref = &mut cislo; // reference, pomocí které mohu hodnotu měnit
let ref = &cislo;         // reference, pomocí které mohu hodnotu pouze číst
```

## Funkce

```
fn jmeno_funkce(jmeno_parametru: typ_parametru, ...) -> návratový_typ {
    ...
}
```

## Typový systém

### Struktury

```
struct Vertex {
    position: Vec3,
    color: Vec4,
    ...
}
```

Metody (či "statické" funkce) které patří ke struktuře jsou definovány v `impl` bloku.
Metody mají narozdíl od "statických" funkcí jako první argument `self` referenci (`&mut self` nebo `&self`)

```
impl Vertex {
    pub fn dehomog(&mut self) {
        ...
    }
}
```

### Trait

Trait je obdoba interfacu. </br>
Např. ve standardní knihovně existuje trait `Debug`, který umožňuje vypsat hodnotu na standardní výstup pomocí standardních `print` funkcí.

V tomto projektu využívám trait např. pro podporu OpenGL UniformBufferů.


```
pub struct UniformBuffer<T: UniformBufferElement> {
    pub id: u32,
    pub inner: T,
}

pub trait UniformBufferElement {
    /// The binding port
    const BINDING: u32;
    /// Update buffer data using gl::BufferSubData
    fn update(&self);
    /// Allocate data for the element with gl::BufferData
    fn init_buffer(&self);
}
```

Struktura `UniformBuffer` je generická, přičemž do `T` je možné "dosadit" pouze typy, které implementují trait `UniformBufferElement`. Trait potom implementuju pro všechny typy, které chci používat jako UniformBuffer v shaderech.

Některé traity umí kompilátor (či makra) vygenerovat automaticky pomocí `Derive` syntaxe:

```
#[Derive(Debug)] // Kompilátor sám vygeneruje kód pro implementaci Debug traity pro danou strukturu
struct Point {
    x: f32,
    y: f32,
    z: f32,
}
```

### Sum types

V Rustu se dost využívají sum types, které jsou definovány pomocí enumů. </br>
Např.:

```
pub enum AnimationTransform {
    Translation(Vec3),
    Rotation(Quat),
    Scale(Vec3),
}
```

Toto je enum popisující, která transformace se má při animaci provést.
Každá varianta enumu obsahuje svoje vlastní typy dat.

Enumy se často používají ve spojení s pattern matching pomocí `match` (obdoba `switch`):

```
fn animate_point(point: &mut Point, transform: AnimationTransform) {
    match transform {
        Translations(translation) => {
            point.transform(translation) ...
        }
        Rotations(rotation) => {
            point.transform(rotation) ...
        }
        Scales(scale) => {
            point.transform(scale) ...
        }
    }
}
```

`match` má poměrně hodně možností zápisu

```
let number = 13;

match number {
    // Match a single value
    1 => println!("One!"),

    // Match several values
    2 | 3 | 5 | 7 | 11 => println!("This is a prime number"),

    // Match an inclusive range
    13..=19 => println!("A teenager"),

    // Handle the rest of cases
    _ => println!("Ain't special"),
}
```

Platí, že v `match` musí být zastoupeny všechny možné hodnoty daného typu, proto se občas používá `_` placeholder pro varianty, které v tu chvíli nejsou důležité.

## Error handling

Pro error handling jsou v Rustu používají obecně 3 možnosti:

### Option

Obdoba typu Optional z Javy. </br>
Option je definován jako enum s 2 variantami:

```
enum Option<T> {
    Some(T),
    None,
}
```

Pro přístup k vnitřní hodnotě je nutné buďto použít `match`, `if let` syntax nebo `unwrap`. </br>

```
let pole = ...;

// Předpokládám, že pole vrací Option, v závislosti na tom, jestli je index v mezích pole
let hodnota = pole.get(42);

match hodnota {
    Some(vintrni_hodnota) => ...,
    None => ...,
};

if let Some(vnitrni_hodnota) = hodnota {
    ...
};
```

### Result

Result je také definován jako enum:

```
enum Result<T, E> {
    Ok(T),
    Err(E).
}
```

Používá se v metodách, které buďto vrátí variantu `Ok` s ořekávaným výsledkem, nebo hodnotu `Err` s chybou. </br>
S Resulty se často používá `?` syntax, který v případě chyby vrací hodnotu chybu z momentální funkce volajícímu.

```
fn read_username_from_file() -> Result<String, io::Error> {
    let mut f = File::open("username.txt")?; // Pokud nastane chyba při otevření souboru, je funkce přerušena a error je vrácen volající funkci
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(s)
}
```

### Panic

Panic značí chybu, ze které se program nedokáže zotavit a je nutné co nejdříve program ukončit.

## Unsafe

Rust garantuje bezpečnost práce s pamětí a s více-vláknovými aplikacemi.
Pokud je ale potřeba low-level přístup, jako například k OpenGL, tak se musí využívat `unsafe` bloky.
