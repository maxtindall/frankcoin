// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "frankcoin",
    platforms: [.macOS(.v13)],
    products: [
        .executable(name: "frankcoin", targets: ["frankcoin"]),
        .library(name: "FrankMinerCore", targets: ["FrankMinerCore"]),
    ],
    targets: [
        // Everything the miner needs, written out: keccak-256, base58, ed25519
        // curve arithmetic for PDAs, transaction signing. No third-party deps.
        .target(name: "FrankMinerCore"),
        // The command-line miner (`frankcoin`). The SwiftUI GUI in
        // Sources/FrankMiner is built separately by build.sh into an .app and
        // is intentionally excluded from the SwiftPM build the Homebrew formula
        // uses, which ships only the CLI.
        .executableTarget(name: "frankcoin", dependencies: ["FrankMinerCore"]),
    ]
)
