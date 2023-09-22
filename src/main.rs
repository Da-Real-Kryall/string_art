use core::panic;

use image::{DynamicImage, GenericImageView};
use minifb::{Window, WindowOptions};
use image::GenericImage;
use std::time::Duration;

///lower bound on the change in closeness to the ideal image. if all potential lines have a diff lower than this, the program will stop.
const DELTA_DIFF_THRESHOLD: f32 = -0.5;
///the size of the output image, in pixels. the output image is square.
const SIZE: u32 = 300;
///the number of pixels of padding on each side of the image. this is used to center the image in the window when in fullscreen.
const SIDE_PADDING: u32 = (SIZE as f32 * 0.3) as u32;
///the number of screws distributed around the circle's edge. there is always one at the top.
const NUM_SCREWS: usize = 271;
///Whether to use an algorithm that does not require all lines to be connected, this method is much slower but yields better images.
const SLOW_BETTER_MODE: bool = false;
///Crops the image to a circle. The circle is centered in the image and has a diameter equal to the smaller of the image's width and height.
const CROP_TO_CIRCLE: bool = true;
fn crop_to_circle(image: &DynamicImage) -> DynamicImage {
    let (width, height) = image.dimensions();
    let radius = width.min(height) / 2;
    let mut circle = DynamicImage::new_luma8(radius * 2, radius * 2);
    for x in 0..radius * 2 {
        for y in 0..radius * 2 {
            let dx = x as i32 - radius as i32;
            let dy = y as i32 - radius as i32;
            if dx * dx + dy * dy <= (radius * radius) as i32 {
                circle.put_pixel(x, y, image.get_pixel(x, y));
            }
        }
    }
    circle
}

///Returns resulting pixel darkness based on distance from the pixel to the line.
fn point_profile(distance: f32) -> f32 {
    (0.6-(distance).abs()).max(0.0).min(0.05)
}

///Generates the positions of the screws around the circle. The first screw is always at the top. The data is returned as an array of (x, y) coordinates corresponding to the pixel coordinates they occupy on the image.
fn generate_screw_locations() -> [(f32, f32); NUM_SCREWS] {
    if CROP_TO_CIRCLE {

        let mut circle_screws = [(0.0, 0.0); NUM_SCREWS];
        for i in 0..NUM_SCREWS {
            let angle = i as f32 * 2.0 * std::f32::consts::PI / NUM_SCREWS as f32;
            circle_screws[i] = (
                (SIZE as f32 / 2.0 + angle.cos() * SIZE as f32 / 2.0).round(),
                (SIZE as f32 / 2.0 + angle.sin() * SIZE as f32 / 2.0).round()
            );
        }
        circle_screws
    } else {
        //place square screws around the edge of the image, for testing
        let mut square_screws = [(0.0, 0.0); NUM_SCREWS];
        for i in 0..NUM_SCREWS / 4 {
            square_screws[i] = (
                i as f32 * SIZE as f32 / (NUM_SCREWS / 4) as f32,
                SIZE as f32
            );
            square_screws[i + NUM_SCREWS / 4] = (
                SIZE as f32,
                SIZE as f32 - i as f32 * SIZE as f32 / (NUM_SCREWS / 4) as f32
            );
            square_screws[i + NUM_SCREWS / 2] = (
                SIZE as f32 - i as f32 * SIZE as f32 / (NUM_SCREWS / 4) as f32,
                0.0
            );
            square_screws[i + NUM_SCREWS / 4 * 3] = (
                0.0,
                i as f32 * SIZE as f32 / (NUM_SCREWS / 4) as f32
            );
        }
        square_screws  
    }
}
    
///Generates the grids of resulting added darkness for each potential line drawn between screws. masks\[0]\[4] returns an image of the line drawn between the top screw and the screw 4 notches clockwise from it.
fn generate_line_mask(i: usize, j: usize, screws: [(f32, f32); NUM_SCREWS]) -> [[f32; SIZE as usize]; SIZE as usize] {
    //let mut mask = [[0.0; SIZE as usize]; SIZE as usize];
    //let x1 = screws[i].0 as usize;
    //let y1 = screws[i].1 as usize;
    //let x2 = screws[j].0 as usize;
    //let y2 = screws[j].1 as usize;
    //let going_left = x1 < x2;
    //let going_up = y1 < y2;
    //println!("Going up: {}, Going left: {}", going_up, going_left);
    //let mut x = x1;
    //let mut y = y1;
    //let dx: f32  = x2 as f32 - x1 as f32;
    //let dy: f32 = y2 as f32- y1 as f32;
//
    //let mut d = 2.0 * dy - dx; // discriminator
//
    //println!("({}, {}) to ({}, {})", x1, y1, x2, y2);
    //
    //// Euclidean distance of point (x,y) from line (signed)
    //let mut D = 0.0; 
    //
    //// Euclidean distance between points (x1, y1) and (x2, y2)
    //let length = ((dx * dx + dy * dy) as f32).sqrt();
//
    //let sin = dx as f32 / length;     
    //let cos = dy as f32 / length;
    //println!("{}, {}", sin, cos);
    //while {
    //    if going_left {
    //        x <= x2
    //    } else {
    //        x >= x2
    //    }
    //} {
    //    mask[y - 1][x] += D + cos;
    //    mask[y][x] += D;
    //    mask[y + 1][x] += D - cos;
    //    println!("val {}, D {}, d {}, ({}, {})", mask[y][x], D, d, x, y);
    //    x = {
    //        if going_left {
    //            x + 1
    //        } else {
    //            x - 1
    //        }
    //    };
    //    if d <= 0.0 {
    //        D += sin;
    //        d += 2.0 * dy;
    //    } else {
    //        D += sin - cos;
    //        d += 2.0 * (dy - dx);
    //        y = {
    //            if going_up {
    //                y + 1
    //            } else {
    //                y - 1
    //            }
    //        };
    //    }
    //}
//
    //mask

    let mut mask = [[0.0; SIZE as usize]; SIZE as usize];
    let (x1, y1) = screws[i];
    let (x2, y2) = screws[j];
    let m = (y2 - y1) / (x2 - x1);
    let c = y1 - m * x1;
    for x in 0..SIZE {
        for y in 0..SIZE {
            //if the pixel is further away from the center of the image than the circle's radius, just make it white
            if ((x as f32 - SIZE as f32 / 2.0).powi(2) + (y as f32 - SIZE as f32 / 2.0).powi(2) > (SIZE as f32 / 2.0).powi(2)) & CROP_TO_CIRCLE {
                mask[y as usize][x as usize] = 0.0;
                continue;
            }
            let distance = (m * x as f32 - y as f32 + c).abs() / (m * m + 1.0).sqrt();
            mask[y as usize][x as usize] = point_profile(distance);
        }
    }
    mask
}

//fn luminance_to_ansi(luminance: f32) -> u8 {
//    let chart: Vec<u8> = vec![
//        0,
//        232,
//        233,
//        234,
//        235,
//        236,
//        237,
//        238,
//        239,
//        240,
//        241,
//        241,
//        242,
//        243,
//        244,
//        245,
//        246,
//        247,
//        248,
//        249,
//        250,
//        251,
//        252,
//        253,
//        254,
//        255,
//        15
//    ];
//    let chart_length = chart.len();
//    return chart[(luminance * (chart_length as f32)).round().min(chart_length as f32 - 1.0) as usize];
//
//}

/// Takes a grid of floats and prints them based on a luminance chart.
fn print_image(buffer: &mut Vec<u32>, image: [[f32; SIZE as usize]; SIZE as usize]) {
    for y in 0..SIZE {
        for x in 0..SIZE {
            let pixel = image[y as usize][x as usize];
            //let prev_pixel = prev_image[y as usize][x as usize];
            let luminance = (pixel * 255.0) as u8;//luminance_to_ansi(pixel);
            //let prev_luminance = luminance_to_ansi(prev_pixel);
            buffer[((y+1) * (SIZE + SIDE_PADDING * 2) + x + SIDE_PADDING) as usize] = (luminance as u32) << 16 | (luminance as u32) << 8 | luminance as u32;
        }
    };




    //let mut print_buffer = String::new();
    //for y in 0..SIZE/2 {
    //    let mut usegoto = false;
//
    //    let y = (SIZE/2 - y - 1);
    //    
    //    //goto the start of the line
    //    print_buffer += &format!("\x1B[{};{}H", y+1, 1);
    //    for x in 0..SIZE {
    //        if usegoto {
    //            print_buffer += &format!("\x1B[{};{}H", y+1, x+1);
    //            usegoto = false;
    //        }
    //        let append_text = format!("\x1B[38;5;{};48;5;{}m▀",
    //            //set the foreground colour to the pixel's colour
    //            luminance_to_ansi(image[2*y as usize][x as usize]),
    //            //set the background colour to the pixel's colour
    //            luminance_to_ansi(image[1+2*y as usize][x as usize])
    //        );//;}
    //        let old_text = format!("\x1B[38;5;{};48;5;{}m▀",
    //            //set the foreground colour to the pixel's colour
    //            luminance_to_ansi(prev_image[2*y as usize][x as usize]),
    //            //set the background colour to the pixel's colour
    //            luminance_to_ansi(prev_image[1+2*y as usize][x as usize])
    //        );//;}
    //        if append_text != old_text {
    //            print_buffer += &append_text;
    //        } else {
    //            usegoto = true;
    //        }
    //    }
    //}
    //print!("{}", print_buffer);
}


fn main() {
    let options = WindowOptions {
        resize: true,
        ..WindowOptions::default()
    };
    let mut window = Window::new("String Art", (SIZE + SIDE_PADDING * 2) as usize, SIZE as usize+2, options)
        .unwrap_or_else(|e| {
            panic!("{}", e);
    });
    let mut buffer: Vec<u32> = vec![0; ((SIZE+2) * (SIZE + SIDE_PADDING * 2)) as usize];

    let ideal_image = {
        let image = //crop_to_circle(
        image::open("input.jpg")
            .unwrap()
            .resize_exact(SIZE, SIZE, image::imageops::FilterType::Nearest)
            .grayscale()
        //)
        ;
        let image = if CROP_TO_CIRCLE {
            crop_to_circle(&image)
        } else {
            image
        };
        let mut grid: [[f32; SIZE as usize]; SIZE as usize] = [[0.0; SIZE as usize]; SIZE as usize];
        for x in 0..SIZE {
            for y in 0..SIZE {
                let pixel = image.get_pixel(x, y)[0];
                grid[y as usize][x as usize] = pixel as f32 / 255.0;
            }
        }
        grid
    };

    

    let mut image: [[f32; SIZE as usize]; SIZE as usize] = [[0.0; SIZE as usize]; SIZE as usize];
    let mut i = 100;
    let mut old_closeness: f32 = 100000.0;
    let screw_locations = generate_screw_locations();


    while window.is_open() {
        let mut permutations: Vec<(usize, usize, f32)> = Vec::new();
        
        for i in {
            if SLOW_BETTER_MODE {
                0..NUM_SCREWS
            } else {
                i..i+1
            }
        } {
            //permutations is the list of the different lines drawn between screws and the resulting change in closeness to the ideal image. Higher third value means closer to the ideal image.
            for j in 0..NUM_SCREWS {
                //if there are at least 2 pegs between the screws, draw the line
                if (j as i32 - i as i32).abs() > NUM_SCREWS as i32 / 15 {
                    let mask = generate_line_mask(i, j, screw_locations);
                    //print_image(&mut buffer, mask);
                    //window
                    //    .update_with_buffer(&buffer, (SIZE+SIDE_PADDING*2) as usize, SIZE as usize)
                    //    .unwrap();
                    let mut temp_image = image;
                    for x in 0..SIZE {
                        for y in 0..SIZE {
                            temp_image[y as usize][x as usize] += mask[y as usize][x as usize];
                        }
                    }
                    //if temp_image == image {
                    //    continue;
                    //}
                    let mut closeness = 0.0;
                    let mut skip = false;
                    for x in 0..SIZE {
                        for y in 0..SIZE {
                            closeness += (ideal_image[y as usize][x as usize] - image[y as usize][x as usize] - mask[y as usize][x as usize]).powi(2);
                            if old_closeness - closeness < DELTA_DIFF_THRESHOLD {
                                skip = true;
                                //println!("skipped");
                                break;
                            }
                        }
                        if skip {
                            //println!("skipped");
                            break;
                        }
                    }
                    if !skip {
                        permutations.push((i, j, closeness));
                    }
                    //std::thread::sleep(Duration::from_millis(100));
                }
            }
        }
        permutations.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

        //println!("{:?}", permutations);
        if permutations.len() == 0 {
            break;
        }
        
        
        let mut closeness = permutations[0].2;
        let mut x = 0;
        while old_closeness - closeness < DELTA_DIFF_THRESHOLD {
            println!("{}, {}, {}", closeness, old_closeness, old_closeness - closeness);
            x += 1;
            closeness = permutations[x].2;
            if x >= permutations.len() - 1 {
                break;
            }
        }
        if x != 0 {
            panic!("x is not 0");
        }
        //println!("{}, {}, {}, {}, {}", permutations[x].2, old_closeness, permutations.len(), x, old_closeness - closeness);
        old_closeness = closeness;
        let mask;
        if SLOW_BETTER_MODE {
            mask = generate_line_mask(permutations[x].1, permutations[x].0, screw_locations);
        } else {
            i = permutations[x].1;
            mask = generate_line_mask(i, permutations[x].0, screw_locations);
        }
        //apply the line to the image
        for x in 0..SIZE {
            for y in 0..SIZE {
                image[y as usize][x as usize] += mask[y as usize][x as usize];
            }
        }
        print_image(&mut buffer, image);
        window
            .update_with_buffer(&buffer, (SIZE+SIDE_PADDING*2) as usize, SIZE as usize)
            .unwrap();
    }
    println!("done");
    //save the image
    let mut save_image = DynamicImage::new_luma8(SIZE, SIZE)
        .to_luma8();
    for x in 0..SIZE {
        for y in 0..SIZE {
            save_image.put_pixel(x, y, image::Luma([(image[y as usize][x as usize] * 255.0) as u8]));
        }
    }
    save_image.save("output.png").unwrap();
    
    //maintain the window until it is closed
    while window.is_open() {
        window
            .update_with_buffer(&buffer, (SIZE+SIDE_PADDING*2) as usize, SIZE as usize)
            .unwrap();
        std::thread::sleep(Duration::from_millis(100));
    }


}
