#!/bin/bash
set -e

pixelSort="horizontal"
fftSize="1024"
optparse=`getopt -o '' -l pixel-sort:,fft-size:,help -- "$@"`
if [ ! $? -eq 0 ]; then
	exit 1
fi
eval set -- $optparse
while true; do
	case $1 in
		-h|--help)
			echo >&2 "$(basename $0) [--pixel-sort vertical|horizontal|whole-frame] [--fft-size number] <infile> <outfile>"
			exit
			;;
		--pixel-sort)
			pixelSort="$2"
			shift 2
			;;
		--fft-size)
			fftSize="$2"
			shift 2
			;;
		--)
			shift
			break
			;;
		*)
			echo "Unknown option $1"
			exit 1
			;;
	esac
done
if [ ! $# -eq 2 ]; then
	echo >&2 "Missing input and/or output file arguments"
	exit 1
fi
infile="$1"
outfile="$2"

probeJson="$(ffprobe -hide_banner -of json -show_format -show_streams $infile 2>/dev/null)"
audioChannels="$(echo $probeJson | jq '.streams[] | select(.codec_type == "audio").channels')"
audioRate="$(echo $probeJson | jq '.streams[] | select(.codec_type == "audio").sample_rate | tonumber')"

cleanup() {
	kill $(jobs -p)
	exit 1
}
trap cleanup SIGINT

cargo build --workspace
rm -rfv *.sock
ffmpeg -hide_banner -i $infile -map 0:a:0 -f f32le -listen 1 unix:asrc.sock &
FREI0R_PATH="./target/debug" ffmpeg -hide_banner \
	-i $infile \
	-f f32le -ac $audioChannels -ar $audioRate -listen 1 -i unix:adest.sock \
	-filter_complex '[0:v]frei0r=libpixel_sorter[vout]' \
	-map "[vout]" -map 1:a \
	-c:a aac -b:a 192k \
	-c:v libx264 -profile:v high444 -preset:v veryslow -crf 20 -movflags +faststart \
	$outfile -y &
cargo run -p audio_sorter -- asrc.sock adest.sock \
	--channels $audioChannels --fft-size $fftSize &
wait
