// #[cfg(test)]
// mod tests {
//     #[test]
//     fn check_if_density(imgbuf: &mut image::RgbImage, img_name: &String) {
//         dbg!("-----------------------------------------");
//         dbg!(" # Fluid::CHECK_IF_DENSITY");
//         dbg!("-----------------------------------------");
//         let imgx = N;
//         let imgy = N;
//         let mut num_different_pixels = 0u32;
//         let mut num_same_pixels = 0u32;
//         let first_pixel: &image::Rgb<u8> = imgbuf.get_pixel(0, 0);
    
//         for x in 0..imgx {
//             for y in 0..imgy {
//                 let cx = y as f32 - 1.5;
//                 let cy = x as f32 - 1.5;
    
//                 let pixel = imgbuf.get_pixel(x, y);
//                 if pixel != first_pixel {
//                     dbg!("different pixel found! x:{}, y:{}", x, y);
//                     num_different_pixels += 1;
//                 } else if pixel == first_pixel {
//                     num_same_pixels += 1;
//                 }
//             }
//         }
//         dbg!("Num different pixels: {}", num_different_pixels);
//         dbg!("Num different pixels: {}", num_same_pixels);
//     }
    

// }