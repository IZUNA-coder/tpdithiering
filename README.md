# TP DitherPunk

## Auteur: Roshan GNANESWARAN

## Compilation

Pour compiler le projet, il suffit de lancer la commande suivante:

```bash
cargo build
```

## Utilisation

Pour utiliser le programme, il suffit de lancer la commande suivante:

Liste des commandes possibles:
```bash
cargo run -- ./img/iut.jpg seuil

cargo run -- ./img/iut.jpg palette --n-couleurs 7

cargo run -- ./img/iut.jpg tramage 

cargo run -- ./img/iut.jpg bayer --order 3 

cargo run -- ./img/iut.jpg diffusion

cargo run -- ./img/iut.jpg palette-diffusion --n-couleurs 5 --matrix jarvis-judice-ninke
```

Modes
- seuil : Convertit l'image en monochrome par seuillage.
-    palette : Convertit l'image en utilisant une palette de couleurs.
-    tramage : Convertit l'image en utilisant le tramage.
-    bayer : Convertit l'image en utilisant le tramage de Bayer.
-    diffusion : Convertit l'image en utilisant la diffusion d'erreur.
-    palette-diffusion : Convertit l'image en utilisant une palette de couleurs avec diffusion d'erreur.

### Réponses aux questions

## Question 3

Si l'image d'entrée contient un canal alpha, il sera ignoré lors de la conversion en RGB8. Donc pour une image png on aura une image avec un fond noir.

Exemple pour sauvegarder l'image :

```rust
rgb_image.save("output.png").expect("Erreur lors de la sauvegarde");
```

## Question 4

Afin de récupérer et afficher la couleur d’un pixel donné :

```rust
let pixel = rgb_image.get_pixel(32, 52);
println!("Couleur du pixel (32, 52): {:?}", pixel);
```

## Question 5
`
Pour passer une pixel sur deux en blanc, on peut utiliser le code suivant :
```rust
let mut new_image = rgb_image.clone();
for (x, y, pixel) in new_image.enumerate_pixels_mut() {
    if (x + y) % 2 == 0 {
        *pixel = image::Rgb([255, 255, 255]); // Blanc
    }
}
```

Par exemple sur mon image de test :

[Image avant](./img/shadow.png)

[Image après](./img/shadow_filtreBlanc.png)

## Question 6

La luminosité d’un pixel peut être calculée comme une moyenne pondérée des composantes R, G, et B, selon la formule standard pour la conversion en niveaux de gris.


```rust
fn luminosity(pixel: &image::Rgb<u8>) -> f32 {
    0.2126 * pixel[0] as f32 + 0.7152 * pixel[1] as f32 + 0.0722 * pixel[2] as f32
}
```

## Question 7

Pour chaque pixel, si la luminosité est supérieure à 50 %, le pixel est remplacé par blanc, sinon par noir.

```rust 
for (_x, _y, pixel) in rgb_img.enumerate_pixels_mut() {
    let lum = luminosity(pixel);
    *pixel = if lum > 128.0 {
        image::Rgb([255, 255, 255]) // Blanc
    } else {
        image::Rgb([0, 0, 0]) // Noir
    };
}

```

[Image avant](./ditherpunk/img/iut.jpg)

[Image après](./ditherpunk/image_monochrome.png)

## Question 9 

La distance entre deux couleurs dans l’espace RGB est calculée à l’aide de la distance euclidienne.

```rust
fn color_distance(c1: [u8; 3], c2: [u8; 3]) -> f32 {
    ((c1[0] as f32 - c2[0] as f32).powi(2) +
     (c1[1] as f32 - c2[1] as f32).powi(2) +
     (c1[2] as f32 - c2[2] as f32).powi(2))
    .sqrt()
}
```

## Question 10

```rust

let palette = [BLACK, WHITE, RED, GREEN, BLUE, YELLOW, MAGENTA, CYAN];
for (_x, _y, pixel) in rgb_img.enumerate_pixels_mut() {
    let mut min_distance = f32::MAX;
    let mut closest_color = WHITE;
    for &color in &palette[..opts.n_couleurs] {
        let distance = color_distance(pixel.0, color.0);
        if distance < min_distance {
            min_distance = distance;
            closest_color = color;
        }
    }
    *pixel = closest_color;
}

```

[Image avant](./ditherpunk/img/iut.jpg)

[Image après](./ditherpunk/image_palette.png)

## Question 11

Si la palette choisi est vide, nous générons une erreur.

```rust
// Vérification si la palette demandée est vide
    if opts.n_couleurs == 0 {
        eprintln!("Erreur : La palette doit contenir au moins une couleur.");
        std::process::exit(1);
    }
```

## Question 12

Le tramage aléatoire remplace chaque pixel par blanc ou noir en fonction d’un seuil généré aléatoirement.

Code :

```rust
let mut rng = rand::thread_rng();
for (_x, _y, pixel) in rgb_img.enumerate_pixels_mut() {
    let lum = luminosity(pixel);
    let threshold: f32 = rng.gen(); // Nombre aléatoire entre 0 et 1
    *pixel = if lum / 255.0 > threshold {
        WHITE
    } else {
        BLACK
    };
}

```

[Image avant](./ditherpunk/img/iut.jpg)

[Image après](./ditherpunk/image_tramage.png)

## Question 13

La matrice de Bayer est générée de manière récursive. Voici un exemple pour générer une matrice d’ordre arbitraire :

```rust
fn generate_bayer_matrix(order: u32) -> Vec<Vec<u8>> {
    if order == 0 {
        return vec![vec![0]]; // Matrice de base B_0
    }

    let prev_matrix = generate_bayer_matrix(order - 1);
    let size = 1 << (order - 1); // Taille 2^(n-1)
    let mut matrix = vec![vec![0; size * 2]; size * 2]; // Matrice 2^n x 2^n

    for i in 0..size {
        for j in 0..size {
            let base = prev_matrix[i][j] * 4;
            matrix[i][j] = base;
            matrix[i][j + size] = base + 3;
            matrix[i + size][j] = base + 2;
            matrix[i + size][j + size] = base + 1;
        }
    }

    matrix
}
```

Exemple pour une matrice d’ordre 3 :

[Image avant](./ditherpunk/img/iut.jpg)

[Image après](./ditherpunk/image_bayer.png)



## Question 16

La diffusion d’erreur de Floyd-Steinberg est implémentée de la manière suivante :

```rust
fn error_diffusion(image: &mut image::GrayImage) {
    let width = image.width() as usize;
    let height = image.height() as usize;

    let mut buffer: Vec<Vec<f32>> = image
        .pixels()
        .map(|p| p[0] as f32 / 255.0) // Normaliser entre 0 et 1
        .collect::<Vec<f32>>()
        .chunks(width)
        .map(|chunk| chunk.to_vec())
        .collect();

    for y in 0..height {
        for x in 0..width {
            let old_value = buffer[y][x];
            let new_value = if old_value > 0.5 { 1.0 } else { 0.0 };
            let error = old_value - new_value;

            image.put_pixel(x as u32, y as u32, image::Luma([(new_value * 255.0) as u8]));

            if x + 1 < width {
                buffer[y][x + 1] += error * 0.5; // Droite
            }
            if y + 1 < height {
                buffer[y + 1][x] += error * 0.5; // Bas
            }
        }
    }
}
```

[Image avant](./ditherpunk/img/iut.jpg)

[Image après](./ditherpunk/image_diffusion.png)

## Question 17

Représentation de l'erreur commise à chaque pixel

Lors de la conversion d'une image en une palette réduite, l'erreur commise pour chaque pixel correspond à la différence entre la couleur d'origine et la couleur sélectionnée dans la palette. Cette erreur est un vecteur à trois composantes, correspondant aux canaux R, G et B.

Formule pour l'erreur :

Erreur = [R_pixel_original - R_palette, G_pixel_original - G_palette, B_pixel_original - B_palette]

Diffusion de l'erreur

L'erreur est propagée aux pixels voisins qui n'ont pas encore été traités. Les proportions de l'erreur sont définies par une matrice de diffusion, par exemple celle de Floyd-Steinberg :

    [  0    0   7/16  ]
    [  3/16 5/16 1/16 ]

Cela signifie que :

7/16 de l'erreur est propagée au pixel à droite.
3/16 est diffusée au pixel en bas à gauche.
5/16 au pixel en bas.
1/16 au pixel en bas à droite.
Pour une palette, chaque composante de l'erreur (R, G, B) est propagée indépendamment.


## Question 18 / 19 / 20

```rust
Mode::PaletteDiffusion(opts) => {
            let matrix = match opts.matrix.as_str() {
                "floyd-steinberg" => vec![
                    (1, 0, 7.0 / 16.0),
                    (-1, 1, 3.0 / 16.0),
                    (0, 1, 5.0 / 16.0),
                    (1, 1, 1.0 / 16.0),
                ],
                "jarvis-judice-ninke" => vec![
                    (1, 0, 7.0 / 48.0), (2, 0, 5.0 / 48.0),
                    (-2, 1, 3.0 / 48.0), (-1, 1, 5.0 / 48.0), (0, 1, 7.0 / 48.0), (1, 1, 5.0 / 48.0), (2, 1, 3.0 / 48.0),
                    (-2, 2, 1.0 / 48.0), (-1, 2, 3.0 / 48.0), (0, 2, 5.0 / 48.0), (1, 2, 3.0 / 48.0), (2, 2, 1.0 / 48.0),
                ],
                "atkinson" => vec![
                    (1, 0, 1.0 / 8.0), (2, 0, 1.0 / 8.0),
                    (-1, 1, 1.0 / 8.0), (0, 1, 1.0 / 8.0), (1, 1, 1.0 / 8.0),
                    (0, 2, 1.0 / 8.0),
                ],
                _ => vec![],
            };

            let palette = [BLACK, WHITE, RED, BLUE, GREEN];
            for y in 0..rgb_img.height() {
                for x in 0..rgb_img.width() {
                    let pixel = rgb_img.get_pixel_mut(x, y);
                    let old_pixel = *pixel;
                    let mut min_distance = f32::MAX;
                    let mut closest_color = WHITE;
                    for &color in &palette[..opts.n_couleurs] {
                        let distance = color_distance(old_pixel.0, color.0);
                        if distance < min_distance {
                            min_distance = distance;
                            closest_color = color;
                        }
                    }
                    *pixel = closest_color;
                    let error = [
                        old_pixel[0] as f32 - closest_color[0] as f32,
                        old_pixel[1] as f32 - closest_color[1] as f32,
                        old_pixel[2] as f32 - closest_color[2] as f32,
                    ];
                    for &(dx, dy, factor) in &matrix {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && nx < rgb_img.width() as i32 && ny >= 0 && ny < rgb_img.height() as i32 {
                            let neighbor_pixel = rgb_img.get_pixel_mut(nx as u32, ny as u32);
                            for i in 0..3 {
                                neighbor_pixel[i] = (neighbor_pixel[i] as f32 + error[i] * factor).clamp(0.0, 255.0) as u8;
                            }
                        }
                    }
        }
```

[Image avant](./ditherpunk/img/iut.jpg)

[Image après](./ditherpunk/image_palette_diffusion.png)

## Bilan

| Transformation           | Image Avant                      | Image Après                        |
|--------------------------|-----------------------------------|-------------------------------------|
| Seuillage (Question 7)   | ![Image avant](./img/iut.jpg)    | ![Image après](./image_monochrome.png) |
| Palette (Question 10)    | ![Image avant](./img/iut.jpg)    | ![Image après](./image_palette.png)    |
| Tramage (Question 12)    | ![Image avant](./img/iut.jpg)    | ![Image après](./image_tramage.png)    |
| Bayer (Question 13)      | ![Image avant](./img/iut.jpg)    | ![Image après](./image_bayer.png)      |
| Diffusion d’erreur (Q16) | ![Image avant](./img/iut.jpg)    | ![Image après](./image_diffusion.png)  |
| Palette + Diffusion (Q18)| ![Image avant](./img/iut.jpg)    | ![Image après](./image_palette_diffusion.png)  |
