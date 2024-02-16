#![allow(unused, non_snake_case)]

use std::{collections::VecDeque, mem::size_of, path::PathBuf};

use anyhow::Result as AResult;
use clap::Parser;
use rustfft::{num_complex::Complex32, FftPlanner};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::UnixStream};

#[derive(Parser)]
struct Args {
	srcSocket: PathBuf,
	destSocket: PathBuf,

	#[arg(long)]
	channels: usize,

	#[arg(long)]
	fftSize: usize,
}

#[tokio::main]
async fn main() -> AResult<()> {
	let args = Args::parse();
	let batchBytes = args.channels * args.fftSize * size_of::<f32>();

	let mut srcSocket = UnixStream::connect(args.srcSocket).await?;
	let mut destSocket = UnixStream::connect(args.destSocket).await?;

	let mut buf = [0; 8192];
	let mut queue = VecDeque::<u8>::new();
	let mut batch = Vec::<u8>::with_capacity(batchBytes);
	let mut channels: Vec<Vec<Complex32>> = vec![Vec::with_capacity(args.fftSize); args.channels];
	let mut planner = FftPlanner::<f32>::new();

	let mut done = false;
	while !done {
		let len = match srcSocket.read(&mut buf).await {
			Ok(len) if len == 0 => {
				done = true;
				0
			},
			Ok(len) => len,
			Err(err) => return Err(err)?,
		};
		queue.extend(&buf[.. len]);

		while queue.len() >= batchBytes {
			batch.clear();
			batch.extend(queue.drain(.. batchBytes.min(queue.len())));

			channels.iter_mut().for_each(|c| c.clear());
			let mut chunks = batch.chunks_exact(size_of::<f32>());
			assert!(chunks.remainder().len() == 0);
			let mut floats = chunks.map(|c| {
				let mut buf = [0; 4];
				buf.copy_from_slice(c);
				f32::from_le_bytes(buf)
			});
			for (i, sample) in floats.enumerate() {
				let channel = &mut channels[i % args.channels];
				channel.push(Complex32::new(sample, 0.0));
			}

			let fftLen = channels[0].len();
			let fft = planner.plan_fft_forward(fftLen);
			channels.iter_mut().for_each(|c| fft.process(c));
			for channel in &mut channels {
				channel.sort_by(|l, r| l.re.total_cmp(&r.re));
			}
			let fft = planner.plan_fft_inverse(fftLen);
			channels.iter_mut().for_each(|c| fft.process(c));

			batch.clear();
			for i in 0 .. fftLen {
				for channel in &channels {
					batch.extend((channel[i].re / fftLen as f32).to_le_bytes());
				}
			}
			destSocket.write_all(&batch).await?;
		}
	}
	Ok(())
}
