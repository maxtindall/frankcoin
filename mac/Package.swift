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
        // Everything the miner needs, written out and tested: keccak-256,
        // base58, ed25519 curve arithmetic for PDAs, transaction signing.
        // No third-party dependencies, deliberately.
        .target(name: "FrankMinerCore"),
        .executableTarget(name: "frankcoin", dependencies: ["FrankMinerCore"]),
        .testTarget(name: "FrankMinerCoreTests", dependencies: ["FrankMinerCore"]),
    ]
)
