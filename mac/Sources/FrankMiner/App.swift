import SwiftUI
import AppKit
import UniformTypeIdentifiers

@main
struct FrankMinerApp: App {
    var body: some Scene {
        Window("frankcoin miner", id: "main") {
            ContentView()
                .frame(minWidth: 620, minHeight: 560)
        }
        .windowResizability(.contentMinSize)
    }
}

@MainActor
final class Model: ObservableObject {
    @Published var wallet: Wallet?
    @Published var state = ChainState()
    @Published var status = "No wallet loaded."
    @Published var mining = false
    @Published var rate: Double = 0
    @Published var hashes: UInt64 = 0
    @Published var sessionFranks: Double = 0
    @Published var sessionProofs = 0
    @Published var log: [String] = []
    @Published var gate = Hardware.check()

    let fc = Frankcoin(rpc: Rpc(URL(string: "https://api.devnet.solana.com")!))
    private var miner: Miner?

    func say(_ s: String) {
        log.append(s)
        if log.count > 200 { log.removeFirst() }
    }

    func loadWallet() {
        let panel = NSOpenPanel()
        panel.message = "Choose the Solana keypair this miner will be tied to."
        panel.allowedContentTypes = [.json]
        panel.allowsMultipleSelection = false
        guard panel.runModal() == .OK, let url = panel.url else { return }
        do {
            let w = try Wallet(contentsOf: url)
            wallet = w
            say("Tied to \(w.pubkey.base58).")
            say("The key stays on this Mac. Every proof is signed here.")
            Task { await refresh() }
        } catch {
            say("Could not load that file: \(error.localizedDescription)")
        }
    }

    func refresh() async {
        do {
            state = try await fc.state(miner: wallet?.pubkey)
            status = state.deployed ? "Connected to devnet." : "The program is not on this cluster."
        } catch {
            status = "Cannot reach the network."
            say("Network error: \(error.localizedDescription)")
        }
    }

    func register() async {
        guard let w = wallet else { return }
        do {
            say("Registering this wallet — creating your challenge…")
            _ = try await fc.register(wallet: w)
            say("Registered. The challenge is yours alone.")
            await refresh()
        } catch {
            say("Registration failed: \(error.localizedDescription)")
        }
    }

    func toggleMining() {
        if mining { miner?.stop(); mining = false; say("Stopped."); return }
        guard gate.allowed else { say(gate.reason); return }
        guard let w = wallet, state.registered else { say("Load a wallet and register first."); return }
        mining = true
        sessionFranks = 0; sessionProofs = 0
        say("Mining on \(Miner().cores) cores at difficulty \(state.difficulty).")
        Task.detached { [weak self] in await self?.loop(w) }
    }

    private func loop(_ w: Wallet) async {
        while await MainActor.run(body: { self.mining }) {
            let st = await MainActor.run { self.state }
            guard st.registered else { break }
            let m = Miner()
            await MainActor.run { self.miner = m }

            let found: Miner.Found? = await withCheckedContinuation { cont in
                DispatchQueue.global(qos: .userInitiated).async {
                    let r = m.grind(challenge: st.challenge, miner: w.pubkey.bytes,
                                    difficulty: st.difficulty) { h, secs, _ in
                        Task { @MainActor in
                            self.hashes = h
                            self.rate = Double(h) / max(secs, 0.001)
                        }
                    }
                    cont.resume(returning: r)
                }
            }
            guard let found else { break }

            await MainActor.run {
                self.say("Found nonce \(found.nonce) after \(found.hashes) attempts in \(String(format: "%.1f", found.seconds))s.")
            }
            do {
                _ = try await fc.mine(wallet: w, nonce: found.nonce)
                await MainActor.run {
                    self.sessionProofs += 1
                    self.sessionFranks += st.nextReward
                    self.say("Minted \(Int(st.nextReward)) franks. Your challenge has rolled.")
                }
            } catch {
                let msg = error.localizedDescription
                await MainActor.run {
                    self.say("Could not submit: \(msg)")
                    if msg.contains("FullyMined") {
                        self.say("frankcoin is fully mined. There will be no more.")
                        self.mining = false
                    }
                }
                if msg.contains("FullyMined") { break }
            }
            await refresh()
        }
        await MainActor.run { self.mining = false; self.rate = 0 }
    }
}

// ---------------------------------------------------------------------------
// The open bot terminal look: black ground, phosphor green, typewriter face.
// ---------------------------------------------------------------------------

extension Color {
    static let term = Color.black
    static let phosphor = Color(red: 0.12, green: 0.61, blue: 0.26)   // body green
    static let phosphorBright = Color(red: 0.42, green: 1.0, blue: 0.60)
    static let phosphorDim = Color(red: 0.10, green: 0.38, blue: 0.19)
    static let alarm = Color(red: 1.0, green: 0.27, blue: 0.27)
}

private func mono(_ size: CGFloat, _ weight: Font.Weight = .regular) -> Font {
    .custom("Courier New", size: size).weight(weight)
}

/// A label/value row, as on the dashboard: dim tracked-out caps, bright value.
private struct Row: View {
    let label: String
    let value: String
    var bright = false
    var body: some View {
        HStack(alignment: .firstTextBaseline) {
            Text(label.uppercased())
                .font(mono(11))
                .tracking(2)
                .foregroundStyle(Color.phosphor)
            Spacer(minLength: 12)
            Text(value)
                .font(mono(13, bright ? .bold : .regular))
                .foregroundStyle(bright ? Color.phosphorBright : Color.phosphor)
                .textSelection(.enabled)
        }
        .padding(.vertical, 5)
        .overlay(alignment: .bottom) {
            Rectangle().fill(Color.phosphorDim.opacity(0.5)).frame(height: 1)
        }
    }
}

private struct SectionLabel: View {
    let text: String
    var body: some View {
        Text(text.uppercased())
            .font(mono(11))
            .tracking(3)
            .foregroundStyle(Color.phosphor)
            .padding(.bottom, 4)
            .frame(maxWidth: .infinity, alignment: .leading)
    }
}

/// Bordered uppercase button, matching the dashboard's .btn-line.
private struct TermButton: ButtonStyle {
    var accent: Color = .phosphor
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(mono(11))
            .tracking(2)
            .foregroundStyle(accent)
            .padding(.horizontal, 14)
            .padding(.vertical, 8)
            .overlay(Rectangle().stroke(accent, lineWidth: 1))
            .opacity(configuration.isPressed ? 0.55 : 1)
    }
}

struct ContentView: View {
    @StateObject private var m = Model()

    var body: some View {
        ZStack {
            Color.term.ignoresSafeArea()
            ScrollView {
                VStack(alignment: .leading, spacing: 22) {
                    masthead
                    if !m.gate.allowed { gateBanner }
                    chainSection
                    positionSection
                    controls
                    logSection
                    footer
                }
                .padding(22)
            }
        }
        .task { await m.refresh() }
    }

    private var masthead: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("frankcoin")
                .font(mono(38, .bold))
                .foregroundStyle(Color.phosphorBright)
                .shadow(color: Color.phosphorBright.opacity(0.55), radius: 12)
            HStack(spacing: 6) {
                Text("mined on this mac, by you")
                    .font(mono(13))
                    .foregroundStyle(Color.phosphor)
                Rectangle()                       // the terminal cursor
                    .fill(Color.phosphorBright)
                    .frame(width: 8, height: 14)
            }
        }
    }

    private var gateBanner: some View {
        Text(m.gate.reason.uppercased())
            .font(mono(11))
            .tracking(1.5)
            .foregroundStyle(Color.alarm)
            .padding(10)
            .frame(maxWidth: .infinity, alignment: .leading)
            .overlay(Rectangle().stroke(Color.alarm, lineWidth: 1))
    }

    private var chainSection: some View {
        VStack(alignment: .leading, spacing: 0) {
            SectionLabel(text: "the chain · devnet")
            Row(label: "program", value: m.status)
            Row(label: "mined so far",
                value: m.state.deployed ? "\(Int(m.state.totalMinted).formatted()) franks" : "—",
                bright: true)
            Row(label: "proofs accepted",
                value: m.state.deployed ? "\(m.state.proofsAccepted)" : "—")
            Row(label: "next reward",
                value: m.state.deployed ? "\(Int(m.state.nextReward)) franks per proof" : "—")
            Row(label: "difficulty",
                value: m.state.deployed ? "\(m.state.difficulty) bits" : "—")
            Row(label: "this machine", value: Hardware.machineSummary)
        }
    }

    private var positionSection: some View {
        VStack(alignment: .leading, spacing: 0) {
            SectionLabel(text: "your position")
            Row(label: "wallet",
                value: m.wallet.map { String($0.pubkey.base58.prefix(16)) + "…" } ?? "no wallet loaded")
            Row(label: "you have mined",
                value: m.wallet == nil ? "—" : "\(Int(m.state.mined).formatted()) franks over \(m.state.proofs) proofs",
                bright: m.state.mined > 0)
            Row(label: "this session",
                value: "\(Int(m.sessionFranks).formatted()) franks · \(m.sessionProofs) proofs")
            Row(label: "hash rate",
                value: m.mining ? "\(Int(m.rate).formatted()) hashes/sec" : "idle",
                bright: m.mining)
            Row(label: "work done", value: "\(m.hashes.formatted()) hashes this session")
        }
    }

    private var controls: some View {
        HStack(spacing: 10) {
            Button("load wallet") { m.loadWallet() }
                .buttonStyle(TermButton())
            if m.wallet != nil && !m.state.registered {
                Button("register once") { Task { await m.register() } }
                    .buttonStyle(TermButton())
            }
            Button(m.mining ? "stop" : "start mining") { m.toggleMining() }
                .buttonStyle(TermButton(accent: m.mining ? .alarm : .phosphorBright))
                .disabled(m.wallet == nil || !m.state.registered || !m.gate.allowed)
            Button("refresh") { Task { await m.refresh() } }
                .buttonStyle(TermButton())
            Spacer()
        }
        .buttonStyle(.plain)
    }

    private var logSection: some View {
        VStack(alignment: .leading, spacing: 0) {
            SectionLabel(text: "log")
            ScrollViewReader { proxy in
                ScrollView {
                    VStack(alignment: .leading, spacing: 3) {
                        ForEach(Array(m.log.enumerated()), id: \.offset) { i, line in
                            Text(line)
                                .font(mono(12))
                                .foregroundStyle(Color.phosphor)
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .id(i)
                        }
                    }
                    .padding(10)
                }
                .frame(minHeight: 140)
                .overlay(Rectangle().stroke(Color.phosphorDim, lineWidth: 1))
                .onChange(of: m.log.count) { _, n in
                    withAnimation { proxy.scrollTo(n - 1, anchor: .bottom) }
                }
            }
        }
    }

    private var footer: some View {
        Text("frankcoin is mined, not sold · no pre-mine · devnet test tokens, worth nothing")
            .font(mono(10))
            .tracking(1)
            .foregroundStyle(Color.phosphorDim)
    }
}
