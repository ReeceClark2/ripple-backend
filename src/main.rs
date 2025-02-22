use warp::{Filter, Rejection, Reply, cors};
use bytes::Bytes;
use serde::Serialize;
use std::convert::TryInto;

#[derive(Debug, Serialize)]
struct ResponseData {
    response: String,
}

#[derive(Debug)]
struct Pixel {
    x: f32,
    y: f32,
    i: f32,
}


async fn handle_request(body: Bytes) -> Result<impl Reply, Rejection> {
    let raw_data = body.to_vec();
    
    if raw_data.len() % 12 != 0 {
        eprintln!("Received malformed data: {} bytes (not a multiple of 12)", raw_data.len());
        let error_msg = b"Malformed data";
        return Ok(warp::reply::with_header(
            error_msg.to_vec(),
            "Content-Type",
            "application/octet-stream",
        ));
    }

    let pixel_count = raw_data.len() / 12;
    let mut pixels = Vec::with_capacity(pixel_count);

    for chunk in raw_data.chunks_exact(12) {
        let x = f32::from_le_bytes(chunk[0..4].try_into().unwrap());
        let y = f32::from_le_bytes(chunk[4..8].try_into().unwrap());
        let i = f32::from_le_bytes(chunk[8..12].try_into().unwrap());

        pixels.push(Pixel { x, y, i });
    }

    // println!("Received {} pixels:", pixels.len());
    // for pixel in &pixels {
    //     println!("{:?}", pixel);
    // }

    let pixel_positions: Vec<(f32, f32)> = pixels.iter().map(|p| (p.x, p.y)).collect();
    let scale_factor = 1;

    // **New Vector to Store Updated Intensities**
    let mut new_intensities = vec![0.0; pixels.len()];

    for (index, pixel) in pixels.iter().enumerate() {  
        let mut arr: [f32; 9] = [0.0; 9];  // Use f32 instead of i32 for averaging

        for (i, (dx, dy)) in (-1..=1).flat_map(|j| (-1..=1).map(move |h| (j, h))).enumerate() {
            arr[i] = pixel_positions
                .iter()
                .position(|&(x, y)| x == pixel.x - (dx * scale_factor) as f32 && y == pixel.y - (dy * scale_factor) as f32)
                .map(|pos| pixels[pos].i)  // Get intensity from the original pixel set
                .unwrap_or(0.0);
        }

        let sum: f32 = arr.iter().sum();
        new_intensities[index] = sum / arr.len() as f32;
        if new_intensities[index] < 0.02 {
            new_intensities[index] = 0.0;
        }
    }

    // **Apply Updates After All Calculations**
    for (pixel, &new_i) in pixels.iter_mut().zip(new_intensities.iter()) {
        pixel.i = new_i;
    }

    let mut response_data = Vec::with_capacity(pixel_count * 12);
    for pixel in &pixels {
        response_data.extend_from_slice(&pixel.x.to_le_bytes());
        response_data.extend_from_slice(&pixel.y.to_le_bytes());
        response_data.extend_from_slice(&pixel.i.to_le_bytes());
    }

    Ok(warp::reply::with_header(
        response_data,
        "Content-Type",
        "application/octet-stream",
    ))
}


#[tokio::main]
async fn main() {
    let cors = warp::cors()
    .allow_any_origin()
    .allow_header("content-type")
    .allow_methods(vec!["POST"]);

    let route = warp::post()
        .and(warp::path("update"))
        .and(warp::body::bytes()) // Accept raw binary data
        .and_then(handle_request)
        .with(cors);

    println!("Server running on http://127.0.0.1:3000");
    warp::serve(route).run(([127, 0, 0, 1], 3000)).await;
}
