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
    let _img = image::open(&args.input).expect("Failed to open image");
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
            rgb_img.save(&args.output.clone().unwrap_or_else(|| "image_monochrome.png".to_string())).expect("Failed to save image");
        },
        Mode::Palette(opts) => {
            let palette = [WHITE, BLACK,  RED, GREEN,BLUE, YELLOW, MAGENTA, CYAN];
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
            rgb_img.save(&args.output.unwrap_or_else(|| "image_palette.png".to_string())).expect("Failed to save image");
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
            rgb_img.save(&args.output.unwrap_or_else(|| "image_tramage.png".to_string())).expect("Failed to save image");
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