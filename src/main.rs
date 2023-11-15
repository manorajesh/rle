use std::fs::File;
use std::io::{ self, Read, BufReader };
use std::sync::Arc;
use std::sync::atomic::{ AtomicUsize, Ordering };
use rayon::prelude::*;
use indicatif::{ ProgressBar, ProgressStyle };

fn run_length_encode(file_path: &str) -> io::Result<Vec<(u8, u64)>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let contents = reader.bytes().collect::<Result<Vec<_>, _>>()?;

    let chunk_size = 4096; // Adjust the chunk size as needed
    let num_chunks = (contents.len() + chunk_size - 1) / chunk_size;

    // Progress bar setup
    let progress_bar = ProgressBar::new(num_chunks as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
    );

    let progress_counter = Arc::new(AtomicUsize::new(0));

    // Process each chunk in parallel
    let results: Vec<_> = (0..num_chunks)
        .into_par_iter()
        .map(|i| {
            let start = i * chunk_size;
            let end = ((i + 1) * chunk_size).min(contents.len());
            let mut local_result = Vec::new();
            let mut count = 0;
            let mut last_byte = None;

            for &byte in &contents[start..end] {
                match last_byte {
                    Some(b) if b == byte => {
                        count += 1;
                    }
                    Some(b) => {
                        local_result.push((b, count));
                        last_byte = Some(byte);
                        count = 1;
                    }
                    None => {
                        last_byte = Some(byte);
                        count = 1;
                    }
                }
            }

            if let Some(b) = last_byte {
                local_result.push((b, count));
            }

            // Update progress
            let current_progress = progress_counter.fetch_add(1, Ordering::SeqCst);
            progress_bar.set_position((current_progress as u64) + 1);

            local_result
        })
        .collect();

    // Finalize progress bar
    progress_bar.finish_with_message("Compression complete");

    // Combine the results
    let mut combined_result = Vec::new();
    for result in results {
        combined_result.extend(result);
    }

    Ok(combined_result)
}

fn main() {
    let file_path = r#"C:\Users\mano\Downloads\KSP-OST-KSP_Soundtrack.zip"#;
    match run_length_encode(file_path) {
        Ok(encoded) => {
            println!(
                "\n\nLength of encoded data: {} KB",
                ((encoded.len() as f64) * (std::mem::size_of::<(u8, u64)>() as f64)) / 1024.0
            );
        }
        Err(e) => eprintln!("Failed to encode file: {}", e),
    }
}
