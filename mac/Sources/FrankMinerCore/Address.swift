import Foundation
import CryptoKit

public enum Base58 {
    static let alphabet = Array("123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz")

    public static func encode(_ bytes: [UInt8]) -> String {
        var digits: [UInt8] = []
        for byte in bytes {
            var carry = Int(byte)
            for i in 0..<digits.count {
                carry += Int(digits[i]) << 8
                digits[i] = UInt8(carry % 58)
                carry /= 58
            }
            while carry > 0 { digits.append(UInt8(carry % 58)); carry /= 58 }
        }
        var out = ""
        for byte in bytes { if byte == 0 { out += "1" } else { break } }
        for d in digits.reversed() { out.append(alphabet[Int(d)]) }
        return out
    }

    public static func decode(_ s: String) -> [UInt8]? {
        var bytes: [UInt8] = []
        for ch in s {
            guard let idx = alphabet.firstIndex(of: ch) else { return nil }
            var carry = idx
            for i in 0..<bytes.count {
                carry += Int(bytes[i]) * 58
                bytes[i] = UInt8(carry & 0xff)
                carry >>= 8
            }
            while carry > 0 { bytes.append(UInt8(carry & 0xff)); carry >>= 8 }
        }
        var leading = 0
        for ch in s { if ch == "1" { leading += 1 } else { break } }
        return [UInt8](repeating: 0, count: leading) + bytes.reversed()
    }
}

public struct Pubkey: Equatable, Hashable {
    public let bytes: [UInt8]
    public init(_ b: [UInt8]) { bytes = b }
    public init?(base58: String) {
        guard let b = Base58.decode(base58), b.count == 32 else { return nil }
        bytes = b
    }
    public var base58: String { Base58.encode(bytes) }
}

public enum Pda {
    static let marker = Array("ProgramDerivedAddress".utf8)

    /// Solana's find_program_address: walk the bump from 255 down and take the
    /// first candidate that is NOT on the curve.
    public static func find(_ seeds: [[UInt8]], program: Pubkey) -> (Pubkey, UInt8)? {
        var bump: Int = 255
        while bump >= 0 {
            var input: [UInt8] = []
            for s in seeds { input += s }
            input.append(UInt8(bump))
            input += program.bytes
            input += marker
            let h = [UInt8](SHA256.hash(data: Data(input)))
            if !Ed25519.isOnCurve(h) { return (Pubkey(h), UInt8(bump)) }
            bump -= 1
        }
        return nil
    }
}
