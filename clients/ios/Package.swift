// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "HealthOS-HMS",
    platforms: [
        .iOS(.v16),
        .macOS(.v13),
        .watchOS(.v9),
        .tvOS(.v16)
    ],
    products: [
        .library(
            name: "HealthOS-HMS",
            targets: ["HealthOS-HMS"]
        ),
        .executable(
            name: "HealthOS-HMS-App",
            targets: ["HealthOS-HMS-App"]
        )
    ],
    dependencies: [
        // Network
        .package(url: "https://github.com/Alamofire/Alamofire.git", from: "5.8.0"),
        .package(url: "https://github.com/daltoniam/Starscream.git", from: "4.0.0"),
        
        // Security
        .package(url: "https://github.com/krzyzanowsky/CryptoSwift.git", from: "1.8.0"),
        
        // QR Code
        .package(url: "https://github.com/dmrschmidt/QRCode.git", from: "17.0.0"),
        
        // Charts
        .package(url: "https://github.com/danielgindi/Charts.git", from: "4.1.0"),
        
        // Keychain
        .package(url: "https://github.com/kishikawakatsumi/KeychainAccess.git", from: "4.2.0"),
        
        // UI Components
        .package(url: "https://github.com/onevcat/Kingfisher.git", from: "7.9.0"),
        .package(url: "https://github.com/SDWebImage/SDWebImageSwiftUI.git", from: "2.2.0"),
        
        // Local Database
        .package(url: "https://github.com/stephencelis/SQLite.swift.git", from: "0.14.0"),
        
        // Notifications
        .package(url: "https://github.com/Alamofire/Alamofire.git", from: "5.8.0"),
        
        // Biometrics
        .package(url: "https://github.com/vadymmarkov/LocalAuthentication.swift", from: "2.0.0")
    ],
    targets: [
        .target(
            name: "HealthOS-HMS",
            dependencies: [
                .product(name: "Alamofire", package: "Alamofire"),
                .product(name: "Starscream", package: "Starscream"),
                .product(name: "CryptoSwift", package: "CryptoSwift"),
                .product(name: "QRCode", package: "QRCode"),
                .product(name: "Charts", package: "Charts"),
                .product(name: "KeychainAccess", package: "KeychainAccess"),
                .product(name: "Kingfisher", package: "Kingfisher"),
                .product(name: "SDWebImageSwiftUI", package: "SDWebImageSwiftUI"),
                .product(name: "SQLite.swift", package: "SQLite.swift"),
                .product(name: "LocalAuthentication", package: "LocalAuthentication")
            ]
        ),
        .target(
            name: "HealthOS-HMS-App",
            dependencies: [
                .target(name: "HealthOS-HMS")
            ]
        )
    ]
)
