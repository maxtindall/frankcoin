import Foundation

/// Keccak-256 (the original padding, not SHA3-256).
///
/// This must agree byte-for-byte with `solana_keccak_hasher` in the program and
/// with js-sha3 in the reference tooling. If it ever drifts, every proof this
/// app finds is rejected on-chain and the failure is silent — so the parity
/// vectors in KeccakTests are not optional decoration.
public enum Keccak {
    private static let rounds: [UInt64] = [
        0x0000000000000001, 0x0000000000008082, 0x800000000000808a, 0x8000000080008000,
        0x000000000000808b, 0x0000000080000001, 0x8000000080008081, 0x8000000000008009,
        0x000000000000008a, 0x0000000000000088, 0x0000000080008009, 0x000000008000000a,
        0x000000008000808b, 0x800000000000008b, 0x8000000000008089, 0x8000000000008003,
        0x8000000000008002, 0x8000000000000080, 0x000000000000800a, 0x800000008000000a,
        0x8000000080008081, 0x8000000000008080, 0x0000000080000001, 0x8000000080008008,
    ]
    private static let rotc: [Int] = [
        1, 3, 6, 10, 15, 21, 28, 36, 45, 55, 2, 14,
        27, 41, 56, 8, 25, 43, 62, 18, 39, 61, 20, 44,
    ]
    private static let piln: [Int] = [
        10, 7, 11, 17, 18, 3, 5, 16, 8, 21, 24, 4,
        15, 23, 19, 13, 12, 2, 20, 14, 22, 9, 6, 1,
    ]

    @inline(__always)
    private static func permute(_ a: inout [UInt64]) {
        var bc = [UInt64](repeating: 0, count: 5)
        for round in 0..<24 {
            // theta
            for i in 0..<5 { bc[i] = a[i] ^ a[i + 5] ^ a[i + 10] ^ a[i + 15] ^ a[i + 20] }
            for i in 0..<5 {
                let t = bc[(i + 4) % 5] ^ rotl(bc[(i + 1) % 5], 1)
                for j in stride(from: 0, to: 25, by: 5) { a[i + j] ^= t }
            }
            // rho + pi
            var t = a[1]
            for i in 0..<24 {
                let j = piln[i]
                let tmp = a[j]
                a[j] = rotl(t, rotc[i])
                t = tmp
            }
            // chi
            for j in stride(from: 0, to: 25, by: 5) {
                for i in 0..<5 { bc[i] = a[j + i] }
                for i in 0..<5 { a[j + i] ^= ~bc[(i + 1) % 5] & bc[(i + 2) % 5] }
            }
            // iota
            a[0] ^= rounds[round]
        }
    }

    @inline(__always)
    private static func rotl(_ x: UInt64, _ n: Int) -> UInt64 {
        (x << UInt64(n)) | (x >> UInt64(64 - n))
    }

    /// Digest of `input`, 32 bytes.
    public static func hash(_ input: [UInt8]) -> [UInt8] {
        let rate = 136                      // 1088 bits, the keccak-256 rate
        var state = [UInt64](repeating: 0, count: 25)
        var block = [UInt8](repeating: 0, count: rate)
        var offset = 0

        while offset + rate <= input.count {
            absorb(&state, Array(input[offset..<(offset + rate)]))
            offset += rate
        }
        // keccak padding: 0x01 … 0x80 (SHA3 would use 0x06 here)
        let tail = input.count - offset
        for i in 0..<rate { block[i] = 0 }
        for i in 0..<tail { block[i] = input[offset + i] }
        block[tail] ^= 0x01
        block[rate - 1] ^= 0x80
        absorb(&state, block)

        var out = [UInt8](repeating: 0, count: 32)
        for i in 0..<4 {
            let w = state[i]
            for b in 0..<8 { out[i * 8 + b] = UInt8((w >> UInt64(8 * b)) & 0xff) }
        }
        return out
    }

    @inline(__always)
    private static func absorb(_ state: inout [UInt64], _ block: [UInt8]) {
        for i in 0..<17 {
            var w: UInt64 = 0
            for b in 0..<8 { w |= UInt64(block[i * 8 + b]) << UInt64(8 * b) }
            state[i] ^= w
        }
        permute(&state)
    }

    /// Leading zero bits of a digest — the program's measure of work.
    public static func leadingZeroBits(_ digest: [UInt8]) -> Int {
        var z = 0
        for b in digest {
            if b == 0 { z += 8 } else { z += b.leadingZeroBitCount; break }
        }
        return z
    }
}
