mod pixelflut;

use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::SystemTime;

use futures::{future, Future};
use image::GenericImageView;

use self::pixelflut::PixelFlut;

fn main() -> io::Result<()> {
    // Read the image
    let image = image::open("images/bougie.jpg").unwrap();
    let messages: Vec<String> = image
        .pixels()
        .map(|(x, y, color)| {
            let color = if color.data[3] == 255 {
                hex::encode(&color.data[..3])
            } else {
                hex::encode(color.data)
            };
            format!("PX {} {} {}", x, y, color)
        })
        .collect();
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(151, 217, 177, 136)), 1234);
    //let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1337);
    let mut pixelflut: Box<Future<Item = PixelFlut, Error = io::Error>> =
        Box::new(PixelFlut::connect(&addr)); //.and_then(PixelFlut::with_size));
    loop {
        // Get the time
        let start = SystemTime::now();
        // Draw the image
        pixelflut = Box::new(future::ok(
            pixelflut
                .and_then(|pixelflut| pixelflut.send_pixels(messages.clone()))
                .wait()
                .unwrap(),
        ));
        let elapsed_time = SystemTime::now().duration_since(start).unwrap();
        let secs = (elapsed_time.as_secs() * 1_000_000_000 + elapsed_time.subsec_nanos() as u64)
            as f64
            / 1_000_000_000.0;
        println!(
            "Finished drawing image after {:?} ({} pixels/s)",
            elapsed_time,
            messages.len() as f64 / secs
        );
    }
    Ok(())
}
