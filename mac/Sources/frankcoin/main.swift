import Foundation
import FrankMinerCore

// frankcoin — the command-line miner.
//
// Same core as the app: the search runs on this machine and every proof is
// signed here. Installed from source, so there is no "trust this binary".

let rpcURL = URL(string: ProcessInfo.processInfo.environment["FRANKCOIN_RPC"]
                 ?? "https://api.devnet.solana.com")!
let fc = Frankcoin(rpc: Rpc(rpcURL))

let home = FileManager.default.homeDirectoryForCurrentUser
let defaultKey = home.appendingPathComponent(".config/solana/id.json")

func die(_ msg: String) -> Never {
    FileHandle.standardError.write(Data(("frankcoin: " + msg + "\n").utf8))
    exit(1)
}

func commas(_ n: Int) -> String { n.formatted(.number.grouping(.automatic)) }

func usage() {
    print("""
    frankcoin — mine franks on your own machine

    usage:
      frankcoin status                  the chain, and your position
      frankcoin register [--key PATH]   once per wallet; creates your challenge
      frankcoin mine     [--key PATH]   search, and claim what you find
      frankcoin address  [--key PATH]   print the wallet this would mine to

    options:
      --key PATH     Solana keypair file (default ~/.config/solana/id.json)
      --once         stop after one accepted proof
      --cores N      how many cores to use (default: all but one)

    environment:
      FRANKCOIN_RPC  RPC endpoint (default https://api.devnet.solana.com)

    Devnet only. These franks are test tokens and are worth nothing.
    """)
}

// ---------------------------------------------------------------- arguments
var args = Array(CommandLine.arguments.dropFirst())
guard let command = args.first else { usage(); exit(0) }
args.removeFirst()

func option(_ name: String) -> String? {
    guard let i = args.firstIndex(of: name), i + 1 < args.count else { return nil }
    return args[i + 1]
}
let keyPath = option("--key").map { URL(fileURLWithPath: ($0 as NSString).expandingTildeInPath) } ?? defaultKey
let once = args.contains("--once")
let cores = option("--cores").flatMap { Int($0) }

func loadWallet() -> Wallet {
    do { return try Wallet(contentsOf: keyPath) }
    catch { die("could not read \(keyPath.path) — \(error.localizedDescription)") }
}

// ------------------------------------------------------------------ commands
let done = DispatchSemaphore(value: 0)

Task {
    do {
        switch command {
        case "help", "-h", "--help":
            usage()

        case "address":
            print(loadWallet().pubkey.base58)

        case "status":
            let wallet = try? Wallet(contentsOf: keyPath)
            let st = try await fc.state(miner: wallet?.pubkey)
            guard st.deployed else { die("the program is not on this cluster") }
            print("program     \(Frankcoin.programId)")
            print("mint        \(st.mint)")
            print("difficulty  \(st.difficulty) bits")
            print("cooldown    \(st.cooldown)s between claims, per wallet")
            print("mined       \(commas(Int(st.totalMinted))) franks over \(commas(st.proofsAccepted)) proofs")
            print("reward      \(commas(Int(st.nextReward))) franks for the next proof")
            if let w = wallet {
                print("")
                print("wallet      \(w.pubkey.base58)")
                print("registered  \(st.registered ? "yes" : "no — run: frankcoin register")")
                if st.registered {
                    print("you hold    \(commas(Int(st.mined))) franks over \(commas(st.proofs)) proofs")
                }
            }

        case "register":
            let wallet = loadWallet()
            let st = try await fc.state(miner: wallet.pubkey)
            guard st.deployed else { die("the program is not on this cluster") }
            if st.registered { print("already registered."); break }
            print("registering \(wallet.pubkey.base58)…")
            let sig = try await fc.register(wallet: wallet)
            print("registered. \(sig)")
            print("this locks ~0.0078 SOL of rent as your mining bond; closing the account returns it.")

        case "mine":
            let wallet = loadWallet()
            var st = try await fc.state(miner: wallet.pubkey)
            guard st.deployed else { die("the program is not on this cluster") }
            guard st.registered else { die("not registered — run: frankcoin register") }

            let engine = cores.map { Miner(cores: $0) } ?? Miner()
            print("mining as \(wallet.pubkey.base58)")
            print("difficulty \(st.difficulty) bits on \(engine.cores) cores · cooldown \(st.cooldown)s\n")

            var claimed = 0
            while true {
                let m = cores.map { Miner(cores: $0) } ?? Miner()
                var lastLine = 0
                guard let found = m.grind(challenge: st.challenge, miner: wallet.pubkey.bytes,
                                          difficulty: st.difficulty,
                                          progress: { hashes, secs, _ in
                    let rate = Double(hashes) / max(secs, 0.001)
                    let line = "  searching · \(commas(Int(hashes))) hashes · \(commas(Int(rate)))/s · \(Int(secs))s"
                    let pad = max(0, lastLine - line.count)
                    lastLine = line.count
                    FileHandle.standardError.write(Data(("\r" + line + String(repeating: " ", count: pad)).utf8))
                }) else { break }

                FileHandle.standardError.write(Data("\r\u{1B}[K".utf8))
                print("  found nonce \(found.nonce) after \(commas(Int(found.hashes))) hashes in \(String(format: "%.1f", found.seconds))s")
                do {
                    let sig = try await fc.mine(wallet: wallet, nonce: found.nonce)
                    claimed += 1
                    print("  +\(commas(Int(st.nextReward))) franks · \(sig)")
                } catch {
                    let msg = error.localizedDescription
                    if msg.contains("Cooldown") {
                        print("  cooldown — waiting \(st.cooldown)s before the next claim")
                        try await Task.sleep(nanoseconds: UInt64(max(st.cooldown, 1)) * 1_000_000_000)
                    } else if msg.contains("FullyMined") {
                        print("  frankcoin is fully mined. there will be no more.")
                        break
                    } else {
                        print("  could not submit: \(msg)")
                    }
                }
                st = try await fc.state(miner: wallet.pubkey)
                if once && claimed > 0 { break }
                if st.cooldown > 0 && claimed > 0 {
                    print("  waiting out the \(st.cooldown)s cooldown…\n")
                    try await Task.sleep(nanoseconds: UInt64(st.cooldown) * 1_000_000_000)
                }
            }
            print("\nclaimed \(claimed) proof(s) this session.")

        default:
            die("unknown command '\(command)' — try: frankcoin help")
        }
    } catch {
        die(error.localizedDescription)
    }
    done.signal()
}
done.wait()
