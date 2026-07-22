import Foundation

/// The search itself. Splits the nonce space across cores and stops the moment
/// one of them finds a proof.
public final class Miner: @unchecked Sendable {
    public struct Found { public let nonce: UInt64; public let hashes: UInt64; public let seconds: Double }

    private let lock = NSLock()
    private var stopped = false
    private var total: UInt64 = 0
    private var sample: UInt64 = 0   // a nonce lane 0 actually tried, for the tape

    public private(set) var cores: Int
    public init(cores: Int = max(1, ProcessInfo.processInfo.activeProcessorCount - 1)) {
        self.cores = cores
    }

    public func stop() { lock.lock(); stopped = true; lock.unlock() }
    public var hashesSoFar: UInt64 { lock.lock(); defer { lock.unlock() }; return total }
    public var sampleNonce: UInt64 { lock.lock(); defer { lock.unlock() }; return sample }

    /// Grind until a nonce meets `difficulty` or `stop()` is called.
    /// `progress` is called on a background queue roughly twice a second.
    public func grind(challenge: [UInt8], miner: [UInt8], difficulty: Int,
                      progress: @escaping (UInt64, Double, UInt64) -> Void) -> Found? {
        lock.lock(); stopped = false; total = 0; lock.unlock()

        let start = Date()
        let result = NSLock()
        var found: Found?

        let ticker = DispatchQueue(label: "frank.progress")
        var ticking = true
        ticker.async { [weak self] in
            while ticking, let self {
                progress(self.hashesSoFar, Date().timeIntervalSince(start), self.sampleNonce)
                Thread.sleep(forTimeInterval: 0.5)
            }
        }

        DispatchQueue.concurrentPerform(iterations: cores) { lane in
            // Each lane walks its own arithmetic sequence, so no two cores ever
            // try the same nonce and no coordination is needed.
            var nonce = UInt64(lane)
            let stride = UInt64(cores)
            var buf = challenge + miner + [UInt8](repeating: 0, count: 8)
            let nonceAt = challenge.count + miner.count
            var localHashes: UInt64 = 0

            while true {
                if localHashes & 0x3FFF == 0 {
                    lock.lock()
                    let quit = stopped
                    total &+= localHashes
                    if lane == 0 { sample = nonce }
                    lock.unlock()
                    localHashes = 0
                    if quit { return }
                }
                for i in 0..<8 { buf[nonceAt + i] = UInt8((nonce >> UInt64(8 * i)) & 0xff) }
                if Keccak.leadingZeroBits(Keccak.hash(buf)) >= difficulty {
                    result.lock()
                    if found == nil {
                        found = Found(nonce: nonce,
                                      hashes: self.hashesSoFar &+ localHashes,
                                      seconds: Date().timeIntervalSince(start))
                    }
                    result.unlock()
                    self.stop()
                    return
                }
                nonce &+= stride
                localHashes &+= 1
            }
        }

        ticking = false
        result.lock(); defer { result.unlock() }
        return found
    }
}
