import CryptoKit
import Foundation

/// HTTP status payload from `GET /v1/status` (snake_case JSON).
struct IngestStatusResponse: Decodable, Equatable {
    let version: String
    let db_status: String
    let db_error: String?
    let bind_mode: String
    let base_url_hints: [String]
}

/// Companion → Desktop ingest errors (mirrors Rust `CompanionError` + reachability UX).
enum IngestClientError: Error, LocalizedError, Equatable {
    case loopbackOnPhysicalDevice
    case timeout
    case unreachable
    case unauthorized
    case plainHttpOffLoopback
    case pinRequired
    case pinMismatch
    case http(status: Int, body: String)
    case network(String)
    case badResponse(String)

    var errorDescription: String? {
        switch self {
        case .loopbackOnPhysicalDevice:
            return "127.0.0.1 is this phone’s loopback — copy the LAN Base URL from Desktop Companion, not localhost."
        case .timeout:
            return "Desktop not reachable in time — same Wi‑Fi, enable LAN on Mac, check firewall."
        case .unreachable:
            return "Desktop not reachable — same Wi‑Fi, enable LAN bind on Mac, check firewall and Local Network permission."
        case .unauthorized:
            return "Ingest unauthorized — check pairing token on phone and Desktop."
        case .plainHttpOffLoopback:
            return "LAN sync uses https. Copy the Base URL from Desktop again."
        case .pinRequired:
            return "Paste the pairing QR so this phone can pin the Mac certificate."
        case .pinMismatch:
            return "Certificate fingerprint does not match this Mac. Scan the QR again."
        case .http(let status, let body):
            return "Ingest HTTP \(status): \(body)"
        case .network(let message):
            return message
        case .badResponse(let message):
            return "Unexpected ingest response: \(message)"
        }
    }
}

/// Loopback guard for physical iPhone (Simulator may use 127.0.0.1).
enum IngestURLPolicy {
    static func isLoopbackHost(_ host: String) -> Bool {
        let h = host.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        return h == "127.0.0.1" || h == "localhost" || h == "::1" || h == "[::1]"
    }

    static func blocksLoopbackOnDevice(_ url: URL) -> Bool {
        #if targetEnvironment(simulator)
        return false
        #else
        guard let host = url.host else { return false }
        return isLoopbackHost(host)
        #endif
    }

    static func validateBaseURL(_ url: URL) throws {
        try validate(url: url, pin: "unused-for-legacy")
    }

    static func validate(url: URL, pin: String) throws {
        if blocksLoopbackOnDevice(url) {
            throw IngestClientError.loopbackOnPhysicalDevice
        }
        let scheme = url.scheme?.lowercased() ?? ""
        if scheme == "http", let host = url.host, !isLoopbackHost(host) {
            throw IngestClientError.plainHttpOffLoopback
        }
        if scheme == "https", pin.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            throw IngestClientError.pinRequired
        }
    }
}

/// SHA-256 fingerprint of a certificate DER, compared without early exit.
enum TlsPin {
    static func sha256Hex(_ data: Data) -> String {
        SHA256.hash(data: data).map { String(format: "%02x", $0) }.joined()
    }

    static func matches(der: Data, expectedHex: String) -> Bool {
        let got = sha256Hex(der)
        let expected = expectedHex.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        guard got.count == expected.count, !got.isEmpty else { return false }
        var diff: UInt8 = 0
        for (left, right) in zip(got.utf8, expected.utf8) {
            diff |= left ^ right
        }
        return diff == 0
    }
}

/// Four-line pairing QR: version, URL, token, fingerprint or `-`.
struct PairingQr {
    let url: String
    let token: String
    let pin: String?

    static func parse(_ text: String) -> PairingQr? {
        let lines = text
            .split(whereSeparator: \.isNewline)
            .map { $0.trimmingCharacters(in: .whitespaces) }
            .filter { !$0.isEmpty }
        guard lines.count == 4, lines[0] == "biofocus:1" else { return nil }
        var url = lines[1]
        while url.hasSuffix("/") {
            url.removeLast()
        }
        guard url.hasPrefix("https://") || url.hasPrefix("http://"), !lines[2].isEmpty else { return nil }
        let pin = lines[3] == "-" ? nil : lines[3]
        return PairingQr(url: url, token: lines[2], pin: pin)
    }
}

/// Accepts the TLS handshake only when the leaf certificate matches `pin`.
final class CertPinDelegate: NSObject, URLSessionDelegate {
    let pin: String
    let rejected: PinFlag

    init(pin: String, rejected: PinFlag) {
        self.pin = pin
        self.rejected = rejected
    }

    func urlSession(
        _ session: URLSession,
        didReceive challenge: URLAuthenticationChallenge,
        completionHandler: @escaping (URLSession.AuthChallengeDisposition, URLCredential?) -> Void
    ) {
        guard challenge.protectionSpace.authenticationMethod == NSURLAuthenticationMethodServerTrust,
              let trust = challenge.protectionSpace.serverTrust,
              let chain = SecTrustCopyCertificateChain(trust) as? [SecCertificate],
              let leaf = chain.first
        else {
            rejected.mark()
            completionHandler(.cancelAuthenticationChallenge, nil)
            return
        }
        let der = SecCertificateCopyData(leaf) as Data
        if TlsPin.matches(der: der, expectedHex: pin) {
            completionHandler(.useCredential, URLCredential(trust: trust))
        } else {
            rejected.mark()
            completionHandler(.cancelAuthenticationChallenge, nil)
        }
    }
}

final class PinFlag: @unchecked Sendable {
    private let lock = NSLock()
    private var rejected = false

    func mark() {
        lock.lock()
        rejected = true
        lock.unlock()
    }

    var isRejected: Bool {
        lock.lock()
        defer { lock.unlock() }
        return rejected
    }
}

/// Minimal HTTP client for ingest (`GET /v1/status`, `POST /v1/ingest`).
struct IngestClient {
    var baseURL: URL
    var token: String
    var pin: String
    var statusSession: URLSession
    var ingestSession: URLSession
    var pinFlag: PinFlag?

    init(baseURL: URL, token: String, pin: String? = nil) {
        self.baseURL = baseURL
        self.token = token
        let resolved = pin ?? UserDefaults.standard.string(forKey: "biofocus.certPin") ?? ""
        self.pin = resolved
        if baseURL.scheme?.lowercased() == "https" {
            let flag = PinFlag()
            self.pinFlag = flag
            self.statusSession = Self.makeSession(
                requestTimeout: 5,
                delegate: CertPinDelegate(pin: resolved, rejected: flag)
            )
            self.ingestSession = Self.makeSession(
                requestTimeout: 10,
                delegate: CertPinDelegate(pin: resolved, rejected: flag)
            )
        } else {
            self.pinFlag = nil
            self.statusSession = Self.makeSession(requestTimeout: 5, delegate: nil)
            self.ingestSession = Self.makeSession(requestTimeout: 10, delegate: nil)
        }
    }

    static func makeSession(requestTimeout: TimeInterval, delegate: URLSessionDelegate?) -> URLSession {
        let config = URLSessionConfiguration.ephemeral
        config.timeoutIntervalForRequest = requestTimeout
        config.timeoutIntervalForResource = requestTimeout
        config.waitsForConnectivity = false
        if let delegate {
            return URLSession(configuration: config, delegate: delegate, delegateQueue: nil)
        }
        return URLSession(configuration: config)
    }

    /// Preflight: `GET /v1/status` (unauthenticated) with ≤5s timeout.
    func fetchStatus() async throws -> IngestStatusResponse {
        try IngestURLPolicy.validate(url: baseURL, pin: pin)
        let url = baseURL.appendingPathComponent("v1").appendingPathComponent("status")
        var request = URLRequest(url: url)
        request.httpMethod = "GET"

        let data: Data
        let response: URLResponse
        do {
            (data, response) = try await statusSession.data(for: request)
        } catch {
            throw mapURLError(error)
        }

        guard let http = response as? HTTPURLResponse else {
            throw IngestClientError.badResponse("non-HTTP response")
        }
        guard (200 ... 299).contains(http.statusCode) else {
            let text = String(data: data, encoding: .utf8) ?? ""
            throw IngestClientError.http(status: http.statusCode, body: text)
        }

        do {
            return try JSONDecoder().decode(IngestStatusResponse.self, from: data)
        } catch {
            throw IngestClientError.badResponse(error.localizedDescription)
        }
    }

    /// Progress only. The body must not carry health samples.
    func postCompanionStatus(_ body: Data) async throws {
        try IngestURLPolicy.validate(url: baseURL, pin: pin)
        let url = baseURL
            .appendingPathComponent("v1")
            .appendingPathComponent("companion")
            .appendingPathComponent("status")
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = body

        let data: Data
        let response: URLResponse
        do {
            (data, response) = try await ingestSession.data(for: request)
        } catch {
            throw mapURLError(error)
        }
        guard let http = response as? HTTPURLResponse else {
            throw IngestClientError.badResponse("non-HTTP response")
        }
        if http.statusCode == 401 {
            throw IngestClientError.unauthorized
        }
        guard (200 ... 299).contains(http.statusCode) else {
            let text = String(data: data, encoding: .utf8) ?? ""
            throw IngestClientError.http(status: http.statusCode, body: text)
        }
    }

    func postObservations(_ body: Data) async throws {
        try IngestURLPolicy.validate(url: baseURL, pin: pin)
        let url = baseURL.appendingPathComponent("v1").appendingPathComponent("ingest")
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = body

        let data: Data
        let response: URLResponse
        do {
            (data, response) = try await ingestSession.data(for: request)
        } catch {
            throw mapURLError(error)
        }

        guard let http = response as? HTTPURLResponse else {
            throw IngestClientError.badResponse("non-HTTP response")
        }
        if http.statusCode == 401 {
            throw IngestClientError.unauthorized
        }
        guard (200 ... 299).contains(http.statusCode) else {
            let text = String(data: data, encoding: .utf8) ?? ""
            throw IngestClientError.http(status: http.statusCode, body: text)
        }
    }

    private func mapURLError(_ error: Error) -> IngestClientError {
        if pinFlag?.isRejected == true {
            return .pinMismatch
        }
        if let urlError = error as? URLError {
            switch urlError.code {
            case .timedOut:
                return .timeout
            case .cannotConnectToHost, .networkConnectionLost, .notConnectedToInternet,
                 .cannotFindHost, .dnsLookupFailed:
                return .unreachable
            default:
                return .network(urlError.localizedDescription)
            }
        }
        return .network(error.localizedDescription)
    }
}

#if DEBUG
enum IngestURLPolicyTests {
    static func loopbackHostDetection() -> Bool {
        IngestURLPolicy.isLoopbackHost("127.0.0.1")
            && IngestURLPolicy.isLoopbackHost("localhost")
            && !IngestURLPolicy.isLoopbackHost("192.168.1.2")
    }
}
#endif
