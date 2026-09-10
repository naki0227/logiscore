use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::{Read, Write};

/// ソースコードのバイト列を zlib (level 6) で圧縮する。
///
/// # Errors
/// 圧縮に失敗した場合 `std::io::Error` を返す。
pub fn compress(input: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(input)?;
    encoder.finish()
}

/// zlib 圧縮されたバイト列を展開する。
///
/// # Errors
/// 展開に失敗した場合 `std::io::Error` を返す。
pub fn decompress(input: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut decoder = ZlibDecoder::new(input);
    let mut output = Vec::new();
    decoder.read_to_end(&mut output)?;
    Ok(output)
}

/// zlibデータを上限付きで展開し、decompression bombを拒否する。
pub fn decompress_limited(input: &[u8], max_length: usize) -> Result<Vec<u8>, std::io::Error> {
    let decoder = ZlibDecoder::new(input);
    let limit = u64::try_from(max_length)
        .map_err(|_| std::io::Error::other("decompression limit exceeds u64"))?
        .saturating_add(1);
    let mut output = Vec::new();
    decoder.take(limit).read_to_end(&mut output)?;
    if output.len() > max_length {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "decompressed data exceeds configured limit",
        ));
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_hello() {
        let input = b"Hello, Logiscore!";
        let compressed = compress(input).unwrap();
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(input.to_vec(), decompressed);
    }

    #[test]
    fn roundtrip_empty() {
        let input = b"";
        let compressed = compress(input).unwrap();
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(input.to_vec(), decompressed);
    }

    #[test]
    fn roundtrip_large() {
        let input: Vec<u8> = (0..10_000).map(|i| (i % 256) as u8).collect();
        let compressed = compress(&input).unwrap();
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(input, decompressed);
    }

    #[test]
    fn compression_reduces_size() {
        let input = "fn main() { println!(\"Hello\"); }\n".repeat(100);
        let compressed = compress(input.as_bytes()).unwrap();
        assert!(compressed.len() < input.len());
    }

    #[test]
    fn limited_decompression_rejects_oversized_output() {
        let compressed = compress(&[0; 1024]).unwrap();
        assert!(decompress_limited(&compressed, 1023).is_err());
        assert_eq!(decompress_limited(&compressed, 1024).unwrap().len(), 1024);
    }
}
