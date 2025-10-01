
#![no_std]
#![no_main]

use core::fmt::Write;

const N: usize = 16; // Muss Potenz von 2 sein
type Sample = i16;

#[inline(always)]
fn avg(a: Sample, b: Sample) -> Sample {
    (a + b) / 2
}

#[inline(always)]
fn diff(a: Sample, b: Sample) -> Sample {
    (a - b) / 2
}

// In-place Haar-Wavelet 1D (rekursiv)
fn haar_wavelet_transform(data: &mut [Sample], levels: usize) {
    let mut n = data.len();
    for _ in 0..levels {
        let mut temp = [0i16; N];
        for i in 0..n / 2 {
            temp[i] = avg(data[2 * i], data[2 * i + 1]);         // Approximation
            temp[n / 2 + i] = diff(data[2 * i], data[2 * i + 1]); // Detail
        }
        data[..n].copy_from_slice(&temp[..n]);
        n /= 2;
    }
}

#[entry]
fn main() -> ! {
    let mut signal: [i16; N] = [2, 0, 1, 1, 1, 0, 2, 0, 1, 1, 1, 0, 2, 1, 1, 1];

    haar_wavelet_transform(&mut signal, 4);

    // Jetzt enthält `signal` zuerst Approximation, dann Details
    // Du kannst z. B. die Energie der Details auswerten (Bandanalyse)
    loop {}
}
