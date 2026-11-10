//! Minimal MATLAB v5 reader for the USC complex recon array. Other shapes are rejected.
use super::*;
use std::io::{Read, Write};
fn u32le(b: &[u8]) -> Result<usize, String> {
    b.get(..4)
        .map(|b| u32::from_le_bytes(b.try_into().unwrap()) as usize)
        .ok_or("truncated MAT tag".into())
}
fn elements(mut b: &[u8]) -> Result<Vec<(usize, Vec<u8>)>, String> {
    let mut out = Vec::new();
    while b.len() >= 8 {
        let tag = u32le(b)?;
        let small = tag >> 16;
        let (kind, len, start, advance) = if small > 0 {
            (tag & 65535, small, 4, 8)
        } else {
            let n = u32le(&b[4..])?;
            (
                tag,
                n,
                8,
                if tag == 15 {
                    8 + n
                } else {
                    8 + n.div_ceil(8) * 8
                },
            )
        };
        let data = b.get(start..start + len).ok_or("truncated MAT element")?;
        out.push((kind, data.to_vec()));
        if advance > b.len() {
            if start + len == b.len() {
                break;
            }
            return Err("truncated MAT padding".into());
        }
        b = &b[advance..];
    }
    Ok(out)
}
fn array(bytes: &[u8]) -> Result<Vec<f32>, String> {
    if bytes.len() < 128 || &bytes[126..128] != b"IM" {
        return Err("expected little-endian MATLAB v5".into());
    }
    let mut elems = elements(&bytes[128..])?;
    let mut arrays = Vec::new();
    for (kind, b) in elems.drain(..) {
        if kind == 15 {
            let mut out = Vec::new();
            flate2::read::ZlibDecoder::new(&b[..])
                .take(64 * 1024 * 1024)
                .read_to_end(&mut out)
                .map_err(|e| e.to_string())?;
            arrays.extend(elements(&out)?);
        } else {
            arrays.push((kind, b));
        }
    }
    for (kind, b) in arrays {
        if kind != 14 {
            continue;
        }
        let fields = elements(&b)?;
        if fields.len() < 4 {
            continue;
        }
        if fields[2].1 != b"recon" {
            continue;
        }
        let dims: Vec<_> = fields[1]
            .1
            .chunks_exact(4)
            .map(|x| u32::from_le_bytes(x.try_into().unwrap()) as usize)
            .collect();
        if dims != [160, 160, 80] {
            return Err(format!("unexpected recon dimensions {dims:?}"));
        }
        let count = 160 * 160 * 80;
        let parse = |kind: usize, b: &[u8]| -> Result<Vec<f32>, String> {
            match kind {
                7 if b.len() == count * 4 => Ok(b
                    .chunks_exact(4)
                    .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
                    .collect()),
                9 if b.len() == count * 8 => Ok(b
                    .chunks_exact(8)
                    .map(|b| f64::from_le_bytes(b.try_into().unwrap()) as f32)
                    .collect()),
                _ => Err("expected float/double recon samples".into()),
            }
        };
        let mut re = parse(fields[3].0, &fields[3].1)?;
        if fields.len() > 4 {
            let im = parse(fields[4].0, &fields[4].1)?;
            for (i, r) in re.iter_mut().enumerate() {
                *r = r.hypot(im[i]);
            }
        } else {
            for r in &mut re {
                *r = r.abs();
            }
        }
        if re.iter().any(|x| !x.is_finite()) {
            return Err("nonfinite recon sample".into());
        }
        return Ok(re);
    }
    Err("recon array missing".into())
}
fn crc(b: &[u8]) -> u32 {
    let mut c = !0u32;
    for &v in b {
        c ^= v as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 {
                (c >> 1) ^ 0xedb88320
            } else {
                c >> 1
            };
        }
    }
    !c
}
fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], b: &[u8]) {
    out.extend_from_slice(&(b.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(b);
    let mut c = kind.to_vec();
    c.extend_from_slice(b);
    out.extend_from_slice(&crc(&c).to_be_bytes());
}
pub fn png(path: &Path, width: usize, height: usize, pixels: &[u8]) -> Result<(), String> {
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&(width as u32).to_be_bytes());
    ihdr.extend_from_slice(&(height as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 0, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &ihdr);
    let mut raw = Vec::new();
    for row in pixels.chunks(width) {
        raw.push(0);
        raw.extend_from_slice(row);
    }
    let mut z = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    z.write_all(&raw).map_err(|e| e.to_string())?;
    chunk(&mut out, b"IDAT", &z.finish().map_err(|e| e.to_string())?);
    chunk(&mut out, b"IEND", &[]);
    fs::write(path, out).map_err(|e| e.to_string())
}
pub fn run(args: &[String]) -> Result<(), String> {
    if args.len() < 2 {
        return Err("tract3d usc <volume.mat> <out-dir> [--ap INDEX]".into());
    }
    let data = fs::read(&args[0]).map_err(|e| e.to_string())?;
    let v = array(&data)?;
    let out = Path::new(&args[1]);
    mkdir(out)?;
    let ap: usize = option(args, "--ap", "64")
        .parse()
        .map_err(|_| "invalid coronal index")?;
    if ap >= 160 {
        return Err("coronal index outside 0..159".into());
    }
    let mut sorted = v.clone();
    sorted.sort_by(f32::total_cmp);
    let peak = sorted[sorted.len() * 995 / 1000].max(1e-12);
    let pixel = |x: f32| ((x / peak).clamp(0., 1.) * 255.) as u8;
    let mut mid = Vec::new();
    for y in 0..160 {
        for x in 0..160 {
            let sample = (v[y + x * 160 + 39 * 25600]
                + v[y + x * 160 + 40 * 25600]
                + v[y + x * 160 + 41 * 25600])
                / 3.;
            mid.push(pixel(sample));
        }
    }
    png(&out.join("midsagittal.png"), 160, 160, &mid)?;
    for (i, a) in [ap, ap.saturating_add(12).min(159)].iter().enumerate() {
        let mut cor = Vec::new();
        for y in 0..160 {
            for z in 0..80 {
                cor.push(pixel(v[y + a * 160 + z * 25600]));
            }
        }
        png(&out.join(format!("coronal-{i}.png")), 80, 160, &cor)?;
    }
    write(
        &out.join("planes.json"),
        &json!({"source":"doi:10.1038/s41597-021-00976-x","license":"CC BY 4.0","dimensions":[160,160,80],"spacingMm":1.5625,"coronalIndices":[ap,ap.saturating_add(12).min(159)],"quantitativeSegmentation":false}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compressed_blocks_have_no_alignment_padding() {
        let bytes = [
            15, 0, 0, 0, 3, 0, 0, 0, 1, 2, 3, 15, 0, 0, 0, 2, 0, 0, 0, 4, 5,
        ];
        let values = elements(&bytes).unwrap();
        assert_eq!(values, vec![(15, vec![1, 2, 3]), (15, vec![4, 5])]);
    }
    #[test]
    fn malformed_mat_is_rejected() {
        assert!(array(&[]).is_err());
        assert!(elements(&[9, 0, 0, 0, 200, 0, 0, 0]).is_err());
    }
}
