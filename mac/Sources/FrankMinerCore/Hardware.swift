import Foundation

/// Mining is meant to happen on the miner's own machine. This refuses the
/// obvious ways of not doing that.
///
/// Being straight about the limits: a virtual machine is detectable, and a
/// headless daemon is detectable. A *rented physical* Mac is not — it is real
/// Apple hardware with a real display session, and nothing the app can read
/// distinguishes it from one on a desk. These checks raise the effort; they do
/// not make cloud mining impossible, and nothing here should be described as
/// though they do.
public enum Hardware {
    public struct Verdict {
        public let allowed: Bool
        public let reason: String
    }

    static func sysctlInt(_ name: String) -> Int64? {
        var value: Int64 = 0
        var size = MemoryLayout<Int64>.size
        if sysctlbyname(name, &value, &size, nil, 0) == 0 { return value }
        var v32: Int32 = 0
        var s32 = MemoryLayout<Int32>.size
        if sysctlbyname(name, &v32, &s32, nil, 0) == 0 { return Int64(v32) }
        return nil
    }

    static func sysctlString(_ name: String) -> String? {
        var size = 0
        guard sysctlbyname(name, nil, &size, nil, 0) == 0, size > 0 else { return nil }
        var buf = [CChar](repeating: 0, count: size)
        guard sysctlbyname(name, &buf, &size, nil, 0) == 0 else { return nil }
        return String(cString: buf)
    }

    public static var isVirtualMachine: Bool {
        if let hv = sysctlInt("kern.hv_vmm_present"), hv != 0 { return true }
        if let brand = sysctlString("machdep.cpu.brand_string"),
           brand.contains("QEMU") || brand.contains("Virtual") { return true }
        return false
    }

    /// A GUI login session. Absent when run over SSH or as a daemon — which is
    /// what a rented builder or a headless fleet looks like.
    public static var hasInteractiveSession: Bool {
        ProcessInfo.processInfo.environment["SSH_CONNECTION"] == nil
            && NSClassFromString("NSApplication") != nil
    }

    public static func check() -> Verdict {
        if isVirtualMachine {
            return Verdict(allowed: false,
                           reason: "This looks like a virtual machine. frankcoin is mined on your own Mac.")
        }
        if ProcessInfo.processInfo.environment["SSH_CONNECTION"] != nil {
            return Verdict(allowed: false,
                           reason: "This is an SSH session. Mine from the Mac in front of you.")
        }
        return Verdict(allowed: true, reason: "Running locally on this Mac.")
    }

    public static var machineSummary: String {
        let chip = sysctlString("machdep.cpu.brand_string") ?? "unknown CPU"
        let cores = ProcessInfo.processInfo.activeProcessorCount
        return "\(chip) · \(cores) cores"
    }
}
