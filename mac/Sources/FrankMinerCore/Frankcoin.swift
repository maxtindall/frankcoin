import Foundation
import CryptoKit

public struct ChainState {
    public var deployed = false
    public var registered = false
    public var mint = ""
    public var difficulty = 0
    public var cooldown: Int64 = 0
    public var totalMinted: Double = 0
    public var proofsAccepted = 0
    public var nextReward: Double = 0
    public var challenge: [UInt8] = []
    public var mined: Double = 0
    public var proofs = 0
}

public struct Frankcoin {
    public static let programId = "CosvVR3aNvHcFPtyzZuD385kvBo2aVa3jZapttst1aqY"
    static let tokenProgram = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
    static let ataProgram = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
    static let systemProgram = "11111111111111111111111111111111"
    static let oneFrank: Double = 1_000_000_000
    static let cap: UInt64 = 1_000_000_000 * 1_000_000_000

    // First eight bytes of sha256("global:<name>"); fixed for the program's life.
    static let ixRegister: [UInt8] = [211, 124, 67, 15, 211, 194, 178, 240]
    static let ixMine: [UInt8] = [59, 22, 178, 213, 139, 197, 160, 196]

    public let rpc: Rpc
    public let program: Pubkey
    public init(rpc: Rpc) {
        self.rpc = rpc
        self.program = Pubkey(base58: Frankcoin.programId)!
    }

    func seeds(_ s: String) -> [UInt8] { Array(s.utf8) }

    public func configPda() -> Pubkey { Pda.find([seeds("config")], program: program)!.0 }
    public func mintPda() -> Pubkey { Pda.find([seeds("mint")], program: program)!.0 }
    public func proofPda(_ miner: Pubkey) -> Pubkey {
        Pda.find([seeds("proof"), miner.bytes], program: program)!.0
    }
    public func ataPda(_ miner: Pubkey) -> Pubkey {
        Pda.find([miner.bytes, Pubkey(base58: Frankcoin.tokenProgram)!.bytes, mintPda().bytes],
                 program: Pubkey(base58: Frankcoin.ataProgram)!)!.0
    }

    /// What the next accepted proof pays. Mirrors reward_for() in the program.
    public static func reward(forTotalMinted minted: UInt64) -> Double {
        var remaining = minted, tranche = cap / 2, reward: UInt64 = 500 * 1_000_000_000
        while remaining >= tranche && reward > 0 {
            remaining -= tranche
            tranche /= 2
            reward /= 2
        }
        return Double(reward) / oneFrank
    }

    static func u64(_ b: [UInt8], _ at: Int) -> UInt64 {
        var v: UInt64 = 0
        for i in 0..<8 { v |= UInt64(b[at + i]) << UInt64(8 * i) }
        return v
    }

    public func state(miner: Pubkey?) async throws -> ChainState {
        var keys = [configPda()]
        if let m = miner { keys.append(proofPda(m)) }
        let infos = try await rpc.accounts(keys)
        var s = ChainState()
        guard let cfg = infos[0] else { return s }
        s.deployed = true
        s.mint = Pubkey(Array(cfg[10..<42])).base58
        let minted = Frankcoin.u64(cfg, 42)
        s.totalMinted = Double(minted) / Frankcoin.oneFrank
        s.difficulty = Int(cfg[58])
        s.cooldown = Int64(bitPattern: Frankcoin.u64(cfg, 59))
        s.proofsAccepted = Int(Frankcoin.u64(cfg, 67))
        s.nextReward = Frankcoin.reward(forTotalMinted: minted)
        if infos.count > 1, let pr = infos[1] {
            s.registered = true
            s.challenge = Array(pr[40..<72])
            s.mined = Double(Frankcoin.u64(pr, 80)) / Frankcoin.oneFrank
            s.proofs = Int(Frankcoin.u64(pr, 88))
        }
        return s
    }

    public func register(wallet: Wallet) async throws -> String {
        let me = wallet.pubkey
        let metas = [
            AccountMeta(me, signer: true, writable: true),
            AccountMeta(configPda()),
            AccountMeta(proofPda(me), writable: true),
            AccountMeta(Pubkey(base58: Frankcoin.systemProgram)!),
        ]
        let bh = try await rpc.latestBlockhash()
        let raw = try Tx.signed(payer: me, secret: wallet.key, program: program,
                                metas: metas, data: Frankcoin.ixRegister, blockhash: bh)
        let sig = try await rpc.send(raw)
        try await rpc.confirm(sig)
        return sig
    }

    public func mine(wallet: Wallet, nonce: UInt64) async throws -> String {
        let me = wallet.pubkey
        var data = Frankcoin.ixMine
        for i in 0..<8 { data.append(UInt8((nonce >> UInt64(8 * i)) & 0xff)) }
        let metas = [
            AccountMeta(me, signer: true, writable: true),
            AccountMeta(configPda(), writable: true),
            AccountMeta(mintPda(), writable: true),
            AccountMeta(proofPda(me), writable: true),
            AccountMeta(ataPda(me), writable: true),
            AccountMeta(Pubkey(base58: Frankcoin.tokenProgram)!),
            AccountMeta(Pubkey(base58: Frankcoin.ataProgram)!),
            AccountMeta(Pubkey(base58: Frankcoin.systemProgram)!),
        ]
        let bh = try await rpc.latestBlockhash()
        let raw = try Tx.signed(payer: me, secret: wallet.key, program: program,
                                metas: metas, data: data, blockhash: bh)
        let sig = try await rpc.send(raw)
        try await rpc.confirm(sig)
        return sig
    }
}

/// The miner's wallet. Loaded from a keypair file the user chooses; the app
/// binds to one wallet and signs every proof with it, locally.
public struct Wallet {
    public let key: Curve25519.Signing.PrivateKey
    public let pubkey: Pubkey

    public init(secretKey raw: [UInt8]) throws {
        guard raw.count == 64 else {
            throw RpcError.transport("a Solana keypair file holds 64 bytes; this one has \(raw.count)")
        }
        key = try Curve25519.Signing.PrivateKey(rawRepresentation: Data(raw[0..<32]))
        pubkey = Pubkey(Array(raw[32..<64]))
        guard [UInt8](key.publicKey.rawRepresentation) == pubkey.bytes else {
            throw RpcError.transport("that keypair file is inconsistent — its public half does not match its secret half")
        }
    }

    public init(contentsOf url: URL) throws {
        let data = try Data(contentsOf: url)
        guard let arr = try JSONSerialization.jsonObject(with: data) as? [Int] else {
            throw RpcError.transport("expected a Solana keypair file: a JSON array of 64 numbers")
        }
        try self.init(secretKey: arr.map { UInt8($0) })
    }
}
