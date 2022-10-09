// Copyright 2020-2022, Augustinas Lukauskas <augustinaslukauskas01@gmail.com>

mod vec3;
mod ray3;
mod color;
mod camera;
use camera::Camera;
mod cloud;
use cloud::Cloud;
mod background;
mod fast_rng;
mod renderer;

use std::{fs, path};
use std::fs::OpenOptions;
use std::io::BufWriter;
#[cfg(not(feature = "no-multithreading"))]
use std::sync::mpsc;
#[cfg(not(feature = "no-multithreading"))]
use std::thread;
use std::time::Instant;
use configparser::ini::Ini;
use rand::{RngCore, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;
use vec3::Vec3;
use background::Background;
use color::Color;
use renderer::Renderer;

#[cfg(not(feature = "no-multithreading"))]
#[inline]
fn compute_multithreaded(camera: &Camera, renderer: &Renderer, renderer_rng: &mut impl rand::Rng, img_data: &mut Vec<u8>, slice_count: usize, slice_height: usize, slice_length: usize) {
    let (tx, rx) = mpsc::channel();

    let mut transmitters = Vec::new();
    for _i in 0..slice_count - 1 {
        transmitters.push(mpsc::Sender::clone(&tx));
    }
    transmitters.push(tx);

    for (i, transmitter) in transmitters.into_iter().enumerate() {
        let frng_seed = renderer_rng.next_u64();
        let min_py = i * slice_height;

        let camera = camera.clone();

        let renderer = renderer.clone();

        thread::spawn(move || {
            let val = (i, renderer.render_slice(&camera, min_py, frng_seed));

            transmitter.send(val).unwrap();
        });
    }

    for received in rx {
        let (thread_index, slice) = received;

        let i1 = thread_index * slice_length;
        let i2 = i1 + slice_length;

        println!("\tSlice #{} complete", thread_index);

        img_data[i1..i2].copy_from_slice(&slice);
    }
}

#[cfg(feature = "no-multithreading")]
#[inline]
fn compute_multithreaded(_camera: &Camera, _renderer: &Renderer, _renderer_rng: &mut impl rand::Rng, _img_data: &mut Vec<u8>, _slice_count: usize, _slice_height: usize, _slice_length: usize) {}

fn main() {
    // ---------- Set configuration variables ----------

    let config_str = fs::read_to_string("config.ini").expect("Error while reading the configuration file.");
    let mut config = Ini::new();
    let _ = config.read(config_str);

    let image_width = config.getuint("images", "image_width").unwrap().unwrap() as usize;
    let image_height = config.getuint("images", "image_height").unwrap().unwrap() as usize;
    let color_byte_size = 6;
    let render_north_only = config.getbool("images", "render_north_only").unwrap().unwrap();
    let raw_color = config.getbool("images", "raw_color").unwrap().unwrap();

    let mut use_multithreading = config.getbool("slices", "use_multithreading").unwrap().unwrap();
    if cfg!(feature = "no-multithreading") && use_multithreading {
        println!("WARNING: \"use_multithreading\" flag is set to true in the configuration file but multithreading is not supported in this build. Ignoring the flag.");
        use_multithreading = false;
    }
    let slice_count = config.getuint("slices", "slice_count").unwrap().unwrap() as usize;
    assert!(!use_multithreading || image_height % slice_count == 0);
    let slice_height = image_height / slice_count;
    let slice_length = image_width * slice_height * color_byte_size;

    let min_height = config.getfloat("clouds", "min_height").unwrap().unwrap().into();
    let max_height = config.getfloat("clouds", "max_height").unwrap().unwrap().into();
    let cloud_threshold = config.getfloat("clouds", "cloud_threshold").unwrap().unwrap().into();
    let noise_levels = config.getuint("clouds", "noise_levels").unwrap().unwrap() as u32;
    let noise_scale = config.getfloat("clouds", "noise_scale").unwrap().unwrap() as f64;
    let cloud_seed = config.getuint("clouds", "cloud_seed").unwrap().unwrap();
    let min_fog_dist = config.getfloat("clouds", "min_fog_dist").unwrap().unwrap().into();
    let max_fog_dist = config.getfloat("clouds", "max_fog_dist").unwrap().unwrap().into();
    let step_size = config.getfloat("clouds", "step_size").unwrap().unwrap().into();
    let step_count  = config.getuint("clouds", "step_count").unwrap().unwrap() as usize;

    let sun_brightness = config.getfloat("background", "sun_brightness").unwrap().unwrap() as f64;
    let sun_size = config.getfloat("background", "sun_size").unwrap().unwrap().into();
    let sun_angle_phi = config.getfloat("background", "sun_angle_phi").unwrap().unwrap().into();
    let sun_angle_theta = config.getfloat("background", "sun_angle_theta").unwrap().unwrap().into();
    let sun_color = Color::from_str(&config.get("background", "sun_color").unwrap());
    let sky_colors: Vec<Color> = config.get("background", "sky_colors").unwrap().split(",").map(|str| (Color::from_str(str.trim()) * (1.0 / sun_brightness))).collect();
    let ground_color = Color::from_str(&config.get("background", "ground_color").unwrap()) * (1.0 / sun_brightness);

    let renderer_seed = config.getuint("quality", "renderer_seed").unwrap().unwrap();
    let pixel_width = config.getuint("quality", "pixel_width").unwrap().unwrap() as usize;
    let bundle_size = config.getuint("quality", "bundle_size").unwrap().unwrap() as usize;

    // ---------- Initialize ----------

    let mut scenes = vec![
        (
            "north",
            Camera::new(
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(0.0, 1.0, 0.0),
            )
        ),
        (
            "south",
            Camera::new(
                Vec3::new(0.0, 0.0, -1.0),
                Vec3::new(0.0, 1.0, 0.0),
            )
        ),
        (
            "east",
            Camera::new(
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            )
        ),
        (
            "west",
            Camera::new(
                Vec3::new(-1.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            )
        ),
        (
            "up",
            Camera::new(
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
            )
        ),
        (
            "down",
            Camera::new(
                Vec3::new(0.0, -1.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
            )
        ),
    ];

    if render_north_only {
        scenes.truncate(1);
    }

    let mut clouds_rng = Xoshiro256PlusPlus::seed_from_u64(cloud_seed);
    let cloud = Cloud::new(min_height, max_height, cloud_threshold, noise_levels, clouds_rng.next_u32() as i32, 256.0 / 2.0f64.powi(noise_levels as i32) * noise_scale);

    let background = Background::new(sun_size, sun_color, sky_colors.clone(), ground_color, sun_angle_phi, sun_angle_theta);

    let mut renderer_rng = Xoshiro256PlusPlus::seed_from_u64(renderer_seed);

    let renderer = Renderer::new(
        cloud,
        slice_length,
        color_byte_size,
        image_width,
        image_height,
        min_fog_dist,
        max_fog_dist,
        step_size,
        step_count,
        pixel_width,
        bundle_size,
        background,
        raw_color,
        sun_brightness
    );

    // ---------- Create images ----------

    let outer_now = Instant::now();

    for (name, camera) in scenes.iter() {
        println!("Computing \"{}\" face...", name);

        // ---------- Compute image data ----------

        let mut img_data = vec![0; image_width * image_height * color_byte_size];
        if use_multithreading {
            compute_multithreaded(camera, &renderer, &mut renderer_rng, &mut img_data, slice_count, slice_height, slice_length);
        } else {
            for thread_index in 0..slice_count {
                let frng_seed = renderer_rng.next_u64();
                let min_py = thread_index * slice_height;

                let val = (thread_index, renderer.render_slice(&camera, min_py, frng_seed));

                let (thread_index, slice) = val;

                let i1 = thread_index * slice_length;
                let i2 = i1 + slice_length;

                println!("\tSlice #{} complete", thread_index);

                img_data[i1..i2].copy_from_slice(&slice);
            }
        }

        // ---------- Write image file ----------

        let path_string = format!("output/{}.png", name);
        let path = path::Path::new(&path_string);
        let prefix = path.parent().unwrap();
        std::fs::create_dir_all(prefix).unwrap();
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(path)
            .unwrap();
        let ref mut w = BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, image_width as u32, image_height as u32);
        encoder.set_color(png::ColorType::RGB);
        encoder.set_depth(png::BitDepth::Sixteen);
        let mut writer = encoder.write_header().unwrap();

        writer.write_image_data(&img_data).unwrap(); // Save


        println!("\"{}\" face complete.\n", name);
    }

    // ---------- Ending tasks ----------

    let elapsed = outer_now.elapsed();
    println!("Duration: {:.2?}", elapsed);
}