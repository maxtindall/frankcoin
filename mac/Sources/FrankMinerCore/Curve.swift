import Foundation

/// Arithmetic mod p = 2^255 - 19, only as much as the PDA check needs.
///
/// A program-derived address is by definition a 32-byte value that is NOT a
/// valid ed25519 point — that is what guarantees no private key exists for it.
/// Deriving one therefore requires deciding whether a candidate lies on the
/// curve, which is why this file exists. CryptoKit cannot help: its public-key
/// initialiser traps rather than throwing on an invalid point.
enum Fe {
    static let p: [UInt64] = [0xFFFF_FFFF_FFFF_FFED, 0xFFFF_FFFF_FFFF_FFFF,
                              0xFFFF_FFFF_FFFF_FFFF, 0x7FFF_FFFF_FFFF_FFFF]
    /// d = -121665/121666, the Edwards curve constant.
    static let d: [UInt64] = [0x75EB_4DCA_1359_78A3, 0x0070_0A4D_4141_D8AB,
                              0x8CC7_4079_7779_E898, 0x5203_6CEE_2B6F_FE73]
    /// (p - 5) / 8 = 2^252 - 3, the exponent for the square-root trick.
    static let pm5d8: [UInt64] = [0xFFFF_FFFF_FFFF_FFFD, 0xFFFF_FFFF_FFFF_FFFF,
                                  0xFFFF_FFFF_FFFF_FFFF, 0x0FFF_FFFF_FFFF_FFFF]

    static let zero: [UInt64] = [0, 0, 0, 0]
    static let one: [UInt64] = [1, 0, 0, 0]

    /// a >= b, comparing 4-limb little-endian values.
    @inline(__always)
    static func gte(_ a: [UInt64], _ b: [UInt64]) -> Bool {
        var i = 3
        while i >= 0 {
            if a[i] != b[i] { return a[i] > b[i] }
            i -= 1
        }
        return true
    }

    @inline(__always)
    static func addNoReduce(_ a: [UInt64], _ b: [UInt64]) -> ([UInt64], UInt64) {
        var r = [UInt64](repeating: 0, count: 4)
        var carry: UInt64 = 0
        for i in 0..<4 {
            let (s1, o1) = a[i].addingReportingOverflow(b[i])
            let (s2, o2) = s1.addingReportingOverflow(carry)
            r[i] = s2
            carry = (o1 ? 1 : 0) + (o2 ? 1 : 0)
        }
        return (r, carry)
    }

    @inline(__always)
    static func subNoReduce(_ a: [UInt64], _ b: [UInt64]) -> ([UInt64], UInt64) {
        var r = [UInt64](repeating: 0, count: 4)
        var borrow: UInt64 = 0
        for i in 0..<4 {
            let (d1, o1) = a[i].subtractingReportingOverflow(b[i])
            let (d2, o2) = d1.subtractingReportingOverflow(borrow)
            r[i] = d2
            borrow = (o1 ? 1 : 0) + (o2 ? 1 : 0)
        }
        return (r, borrow)
    }

    /// Fold a value < 2p back below p.
    @inline(__always)
    static func weakReduce(_ a: [UInt64]) -> [UInt64] {
        if gte(a, p) { return subNoReduce(a, p).0 }
        return a
    }

    static func add(_ a: [UInt64], _ b: [UInt64]) -> [UInt64] {
        let (s, carry) = addNoReduce(a, b)
        var r = s
        if carry == 1 {
            // 2^256 ≡ 38 (mod p)
            r = addNoReduce(r, [38, 0, 0, 0]).0
        }
        return weakReduce(r)
    }

    static func sub(_ a: [UInt64], _ b: [UInt64]) -> [UInt64] {
        let (d, borrow) = subNoReduce(a, b)
        if borrow == 1 { return weakReduce(addNoReduce(d, p).0) }
        return weakReduce(d)
    }

    static func neg(_ a: [UInt64]) -> [UInt64] { sub(zero, a) }

    static func mul(_ a: [UInt64], _ b: [UInt64]) -> [UInt64] {
        // schoolbook 4x4 -> 8 limbs
        var t = [UInt64](repeating: 0, count: 8)
        for i in 0..<4 {
            var carry: UInt64 = 0
            for j in 0..<4 {
                let (hi, lo) = a[i].multipliedFullWidth(by: b[j])
                let (s1, o1) = t[i + j].addingReportingOverflow(lo)
                let (s2, o2) = s1.addingReportingOverflow(carry)
                t[i + j] = s2
                carry = hi &+ (o1 ? 1 : 0) &+ (o2 ? 1 : 0)
            }
            var k = i + 4
            while carry != 0 && k < 8 {
                let (s, o) = t[k].addingReportingOverflow(carry)
                t[k] = s
                carry = o ? 1 : 0
                k += 1
            }
        }
        // fold the high half: 2^256 ≡ 38 (mod p)
        var lo = Array(t[0..<4])
        let hi = Array(t[4..<8])
        var acc = [UInt64](repeating: 0, count: 5)
        var carry: UInt64 = 0
        for i in 0..<4 {
            let (h, l) = hi[i].multipliedFullWidth(by: 38)
            let (s, o) = l.addingReportingOverflow(carry)
            acc[i] = s
            carry = h &+ (o ? 1 : 0)
        }
        acc[4] = carry
        let (sum, c2) = addNoReduce(lo, Array(acc[0..<4]))
        lo = sum
        var top = acc[4] &+ c2
        while top != 0 {
            let (h, l) = top.multipliedFullWidth(by: 38)
            let (s, c3) = addNoReduce(lo, [l, 0, 0, 0])
            lo = s
            top = h &+ c3
        }
        return weakReduce(weakReduce(lo))
    }

    static func sqr(_ a: [UInt64]) -> [UInt64] { mul(a, a) }

    static func pow(_ base: [UInt64], _ exp: [UInt64]) -> [UInt64] {
        var result = one
        var b = base
        for limb in 0..<4 {
            for bit in 0..<64 {
                if (exp[limb] >> UInt64(bit)) & 1 == 1 { result = mul(result, b) }
                b = sqr(b)
            }
        }
        return result
    }

    static func eq(_ a: [UInt64], _ b: [UInt64]) -> Bool {
        weakReduce(a) == weakReduce(b)
    }

    static func fromBytes(_ bytes: [UInt8]) -> [UInt64] {
        var r = [UInt64](repeating: 0, count: 4)
        for i in 0..<4 {
            var w: UInt64 = 0
            for b in 0..<8 { w |= UInt64(bytes[i * 8 + b]) << UInt64(8 * b) }
            r[i] = w
        }
        return r
    }
}

enum Ed25519 {
    /// Is this 32-byte value a valid compressed Edwards point?
    ///
    /// Mirrors dalek's `CompressedEdwardsY::decompress`: with y taken from the
    /// low 255 bits, a point exists iff x² = (y²-1)/(dy²+1) has a square root.
    /// Solana's `find_program_address` skips any candidate for which this is
    /// true — an address with a possible private key would not be *derived*.
    static func isOnCurve(_ bytes: [UInt8]) -> Bool {
        guard bytes.count == 32 else { return false }
        var b = bytes
        b[31] &= 0x7F                     // drop the sign bit; y is the rest
        let y = Fe.fromBytes(b)

        let yy = Fe.sqr(y)
        let u = Fe.sub(yy, Fe.one)                    // y² - 1
        let v = Fe.add(Fe.mul(Fe.d, yy), Fe.one)      // d·y² + 1

        let v3 = Fe.mul(Fe.sqr(v), v)
        let v7 = Fe.mul(Fe.sqr(v3), v)
        let uv7 = Fe.mul(u, v7)
        let r = Fe.mul(Fe.mul(u, v3), Fe.pow(uv7, Fe.pm5d8))

        let check = Fe.mul(v, Fe.sqr(r))
        return Fe.eq(check, u) || Fe.eq(check, Fe.neg(u))
    }
}
