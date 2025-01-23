use argh::FromArgs;
use image::ImageError;
use rand::Rng;


#[derive(Debug, Clone, PartialEq, FromArgs)]
/// Convertit une image en monochrome ou vers une palette réduite de couleurs.
struct DitherArgs {

    /// le fichier d’entrée
    #[argh(positional)]
    input: String,

    /// le fichier de sortie (optionnel)
    #[argh(positional)]
    output: Option<String>,

    /// le mode d’opération
    #[argh(subcommand)]
    mode: Mode
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand)]
enum Mode {
    Seuil(OptsSeuil),
    Palette(OptsPalette),
    Tramage(OptsTramage),
    Bayer(OptsBayer),
    Diffusion(OptsDiffusion),
    PaletteDiffusion(OptsPaletteDiffusion),
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="seuil")]
/// Rendu de l’image par seuillage monochrome.
struct OptsSeuil {}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="palette")]
/// Rendu de l’image avec une palette contenant un nombre limité de couleurs
struct OptsPalette {

    /// le nombre de couleurs à utiliser, dans la liste [NOIR, BLANC, ROUGE, VERT, BLEU, JAUNE, CYAN, MAGENTA]
    #[argh(option)]
    n_couleurs: usize
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="tramage")]
/// Rendu de l’image par tramage aléatoire (dithering).
struct OptsTramage {}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="bayer")]
/// Rendu de l’image par tramage avec une matrice de Bayer.
struct OptsBayer {
    /// l’ordre de la matrice de Bayer (par défaut 2)
    #[argh(option, default = "2")]
    order: u32,
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="diffusion")]
/// Rendu de l’image par diffusion d’erreur.
struct OptsDiffusion {}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name="palette-diffusion")]
/// Rendu de l’image avec une palette et diffusion d’erreur.
struct OptsPaletteDiffusion {

    /// le nombre de couleurs à utiliser, dans la liste [NOIR, BLANC, ROUGE, VERT, BLEU, JAUNE, CYAN, MAGENTA]
    #[argh(option)]
    n_couleurs: usize,

    /// le type de matrice de diffusion d’erreurs à utiliser
    #[argh(option)]
    matrix: String,
}

const WHITE: image::Rgb<u8> = image::Rgb([255, 255, 255]);
const BLACK: image::Rgb<u8> = image::Rgb([0, 0, 0]);
const BLUE: image::Rgb<u8> = image::Rgb([0, 0, 255]);
const RED: image::Rgb<u8> = image::Rgb([255, 0, 0]);
const GREEN: image::Rgb<u8> = image::Rgb([0, 255, 0]);
const YELLOW: image::Rgb<u8> = image::Rgb([255, 255, 0]);
const MAGENTA: image::Rgb<u8> = image::Rgb([255, 0, 255]);
const CYAN: image::Rgb<u8> = image::Rgb([0, 255, 255]);

fn main() -> Result<(), ImageError> {
    let args: DitherArgs = argh::from_env();
    let _img = image::open(&args.input).expect("Image non trouvée");
    let mut rgb_img = _img.to_rgb8(); // Déclarer comme mutable
    
    let pixel = rgb_img.get_pixel(32, 52);
    println!("Pixel at (32, 52): {:?}", pixel);

    match args.mode {
        Mode::Seuil(_) => {
            for (_x, _y, pixel) in rgb_img.enumerate_pixels_mut() {
                let r = pixel[0];
                let g = pixel[1];
                let b = pixel[2];
                let luminance = 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
                *pixel = if luminance > 128.0 {
                    image::Rgb([255, 255, 255])
                } else {
                    image::Rgb([0, 0, 0])
                };
            }
            rgb_img.save(&args.output.clone().unwrap_or_else(|| "image_monochrome.png".to_string())).expect("Erreur lors de la sauvegarde de l'image");
        },
        Mode::Palette(opts) => {
            let palette = [WHITE, BLACK,  RED, GREEN,BLUE, YELLOW, MAGENTA, CYAN];
            
            if opts.n_couleurs == 0 {
                eprintln!("Erreur : La palette doit contenir au moins une couleur.");
                std::process::exit(1);
            }
            
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
            rgb_img.save(&args.output.unwrap_or_else(|| "image_palette.png".to_string())).expect("Erreur lors de la sauvegarde de l'image");
        }

        Mode::Tramage(_) => {
            // Implémentation du tramage aléatoire
            let mut rng = rand::thread_rng();
            for (_x, _y, pixel) in rgb_img.enumerate_pixels_mut() {
                let r = pixel[0];
                let g = pixel[1];
                let b = pixel[2];
                let luminance = 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
                let threshold: f32 = rng.gen(); // Tirage aléatoire entre 0 et 1
                *pixel = if luminance / 255.0 > threshold {
                    image::Rgb([255, 255, 255])
                } else {
                    image::Rgb([0, 0, 0])
                };
            }
            rgb_img.save(&args.output.unwrap_or_else(|| "image_tramage.png".to_string())).expect("Erreur lors de la sauvegarde de l'image");
        }

        Mode::Bayer(opts) => {
            let bayer_matrix = generate_bayer_matrix(opts.order);
            let matrix_size = bayer_matrix.len() as u32;
        
            for (x, y, pixel) in rgb_img.enumerate_pixels_mut() {
                let r = pixel[0];
                let g = pixel[1];
                let b = pixel[2];
                let luminance = 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
        
                // Valeur de seuil issue de la matrice de Bayer
                let threshold = bayer_matrix[(y % matrix_size) as usize][(x % matrix_size) as usize] as f32;
        
                // Normalisation du seuil entre 0 et 255
                let normalized_threshold = threshold * (255.0 / (matrix_size * matrix_size) as f32);
        
                // Tramage : blanc ou noir
                *pixel = if luminance > normalized_threshold {
                    image::Rgb([255, 255, 255]) // Blanc
                } else {
                    image::Rgb([0, 0, 0]) // Noir
                };
            }
        
            rgb_img.save(&args.output.unwrap_or_else(|| "image_bayer.png".to_string())).expect("Erreur lors de la sauvegarde de l'image");
        }

        Mode::Diffusion(_) => {
            // Conversion manuelle en niveaux de gris
            let mut gray_img = image::GrayImage::new(rgb_img.width(), rgb_img.height());
            for (x, y, pixel) in rgb_img.enumerate_pixels() {
                let luminance = (0.2126 * pixel[0] as f32 + 0.7152 * pixel[1] as f32 + 0.0722 * pixel[2] as f32) as u8;
                gray_img.put_pixel(x, y, image::Luma([luminance]));
            }
        
            // Appliquer la diffusion d'erreur
            error_diffusion(&mut gray_img);
        
            // Sauvegarder l'image résultante
            gray_img.save(&args.output.unwrap_or_else(|| "image_diffusion.png".to_string()))
                .expect("Erreur lors de la sauvegarde de l'image");
        }

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
            }
            rgb_img.save(&args.output.unwrap_or_else(|| "image_palette_diffusion.png".to_string())).expect("Erreur lors de la sauvegarde de l'image");
        }
    }

    println!("Fin du programme");
    Ok(())
}

fn color_distance(c1: [u8; 3], c2: [u8; 3]) -> f32 {
    ((c1[0] as f32 - c2[0] as f32).powi(2) +
     (c1[1] as f32 - c2[1] as f32).powi(2) +
     (c1[2] as f32 - c2[2] as f32).powi(2))
    .sqrt()
}

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

fn error_diffusion(image: &mut image::GrayImage) {
    let width = image.width() as usize;
    let height = image.height() as usize;

    // Convertir les pixels en valeurs flottantes pour suivre les erreurs
    let mut buffer: Vec<Vec<f32>> = image
        .pixels()
        .map(|p| p[0] as f32 / 255.0) // Normaliser entre 0 et 1
        .collect::<Vec<f32>>()
        .chunks(width)
        .map(|chunk| chunk.to_vec())
        .collect();

    for y in 0..height {
        for x in 0..width {
            // Valeur actuelle
            let old_value = buffer[y][x];
            let new_value = if old_value > 0.5 { 1.0 } else { 0.0 }; // Blanc ou Noir
            let error = old_value - new_value;

            // Mettre à jour le pixel
            image.put_pixel(x as u32, y as u32, image::Luma([(new_value * 255.0) as u8]));

            // Diffuser l'erreur
            if x + 1 < width {
                buffer[y][x + 1] += error * 0.5; // Voisin de droite
            }
            if y + 1 < height {
                buffer[y + 1][x] += error * 0.5; // Voisin en dessous
            }
        }
    }
}