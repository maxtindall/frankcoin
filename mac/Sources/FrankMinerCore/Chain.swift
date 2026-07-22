import Foundation
import CryptoKit

public struct AccountMeta {
    public let key: Pubkey
    public let isSigner: Bool
    public let isWritable: Bool
    public init(_ key: Pubkey, signer: Bool = false, writable: Bool = false) {
        self.key = key; self.isSigner = signer; self.isWritable = writable
    }
}

/// Legacy transaction encoding. Written out rather than pulled in so the app
/// has no third-party dependency it cannot audit.
public enum Tx {
    static func compactU16(_ n: Int) -> [UInt8] {
        var out: [UInt8] = [], v = n
        repeat {
            var b = UInt8(v & 0x7f)
            v >>= 7
            if v != 0 { b |= 0x80 }
            out.append(b)
        } while v != 0
        return out
    }

    /// Serialise the message, ordering accounts the way the runtime requires:
    /// writable signers, readonly signers, writable others, readonly others.
    static func message(payer: Pubkey, program: Pubkey, metas: [AccountMeta],
                        data: [UInt8], blockhash: [UInt8]) -> ([UInt8], [Pubkey]) {
        var merged: [(Pubkey, Bool, Bool)] = [(payer, true, true)]
        for m in metas {
            if let i = merged.firstIndex(where: { $0.0 == m.key }) {
                merged[i].1 = merged[i].1 || m.isSigner
                merged[i].2 = merged[i].2 || m.isWritable
            } else {
                merged.append((m.key, m.isSigner, m.isWritable))
            }
        }
        if !merged.contains(where: { $0.0 == program }) { merged.append((program, false, false)) }

        let ws = merged.filter { $0.1 && $0.2 }
        let rs = merged.filter { $0.1 && !$0.2 }
        let wn = merged.filter { !$0.1 && $0.2 }
        let rn = merged.filter { !$0.1 && !$0.2 }
        let ordered = ws + rs + wn + rn
        let keys = ordered.map { $0.0 }
        let index = { (k: Pubkey) -> UInt8 in UInt8(keys.firstIndex(of: k)!) }

        var msg: [UInt8] = [
            UInt8(ws.count + rs.count),   // required signatures
            UInt8(rs.count),              // readonly signed
            UInt8(rn.count),              // readonly unsigned
        ]
        msg += compactU16(keys.count)
        for k in keys { msg += k.bytes }
        msg += blockhash
        msg += compactU16(1)              // one instruction
        msg.append(index(program))
        msg += compactU16(metas.count)
        for m in metas { msg.append(index(m.key)) }
        msg += compactU16(data.count)
        msg += data
        return (msg, keys)
    }

    /// Sign and serialise. The key never leaves this process.
    public static func signed(payer: Pubkey, secret: Curve25519.Signing.PrivateKey,
                              program: Pubkey, metas: [AccountMeta],
                              data: [UInt8], blockhash: [UInt8]) throws -> [UInt8] {
        let (msg, _) = message(payer: payer, program: program, metas: metas,
                               data: data, blockhash: blockhash)
        let sig = try secret.signature(for: Data(msg))
        return compactU16(1) + [UInt8](sig) + msg
    }
}

public enum RpcError: Error, LocalizedError {
    case transport(String)
    case node(String)
    public var errorDescription: String? {
        switch self {
        case .transport(let s): return s
        case .node(let s): return s
        }
    }
}

public struct Rpc {
    public let url: URL
    public init(_ url: URL) { self.url = url }

    func call(_ method: String, _ params: [Any]) async throws -> Any {
        var req = URLRequest(url: url)
        req.httpMethod = "POST"
        req.setValue("application/json", forHTTPHeaderField: "Content-Type")
        req.httpBody = try JSONSerialization.data(withJSONObject: [
            "jsonrpc": "2.0", "id": 1, "method": method, "params": params,
        ])
        let (data, _) = try await URLSession.shared.data(for: req)
        guard let obj = try JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            throw RpcError.transport("the node sent something that was not JSON")
        }
        if let err = obj["error"] as? [String: Any] {
            throw RpcError.node((err["message"] as? String) ?? "\(err)")
        }
        guard let result = obj["result"] else { throw RpcError.node("no result") }
        return result
    }

    public func latestBlockhash() async throws -> [UInt8] {
        let r = try await call("getLatestBlockhash", [["commitment": "confirmed"]])
        guard let d = r as? [String: Any], let v = d["value"] as? [String: Any],
              let bh = v["blockhash"] as? String, let bytes = Base58.decode(bh)
        else { throw RpcError.node("could not read a blockhash") }
        return bytes
    }

    public func accounts(_ keys: [Pubkey]) async throws -> [[UInt8]?] {
        let r = try await call("getMultipleAccounts",
                               [keys.map { $0.base58 },
                                ["encoding": "base64", "commitment": "confirmed"]])
        guard let d = r as? [String: Any], let value = d["value"] as? [Any] else {
            throw RpcError.node("unexpected getMultipleAccounts response")
        }
        return value.map { entry in
            guard let e = entry as? [String: Any], let arr = e["data"] as? [Any],
                  let b64 = arr.first as? String, let data = Data(base64Encoded: b64)
            else { return nil }
            return [UInt8](data)
        }
    }

    public func send(_ raw: [UInt8]) async throws -> String {
        let r = try await call("sendTransaction",
                               [Data(raw).base64EncodedString(),
                                ["encoding": "base64", "preflightCommitment": "confirmed"]])
        guard let sig = r as? String else { throw RpcError.node("no signature returned") }
        return sig
    }

    /// Poll until the cluster confirms, or give up. Returns the on-chain error
    /// if the transaction landed but failed.
    public func confirm(_ signature: String, tries: Int = 40) async throws {
        for _ in 0..<tries {
            let r = try await call("getSignatureStatuses", [[signature]])
            if let d = r as? [String: Any], let v = d["value"] as? [Any],
               let first = v.first as? [String: Any] {
                if let err = first["err"], !(err is NSNull) {
                    throw RpcError.node("the program rejected it: \(err)")
                }
                if let status = first["confirmationStatus"] as? String,
                   status == "confirmed" || status == "finalized" { return }
            }
            try await Task.sleep(nanoseconds: 500_000_000)
        }
        throw RpcError.node("timed out waiting for confirmation")
    }
}
