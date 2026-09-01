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
        if blocksLoopbackOnDevice(url) {
            throw IngestClientError.loopbackOnPhysicalDevice
        }
    }
}

/// Minimal HTTP client for ingest (`GET /v1/status`, `POST /v1/ingest`).
struct IngestClient {
    var baseURL: URL
    var token: String
    var statusSession: URLSession
    var ingestSession: URLSession

    init(baseURL: URL, token: String) {
        self.baseURL = baseURL
        self.token = token
        self.statusSession = Self.makeSession(requestTimeout: 5)
        self.ingestSession = Self.makeSession(requestTimeout: 10)
    }

    static func makeSession(requestTimeout: TimeInterval) -> URLSession {
        let config = URLSessionConfiguration.ephemeral
        config.timeoutIntervalForRequest = requestTimeout
        config.timeoutIntervalForResource = requestTimeout
        config.waitsForConnectivity = false
        return URLSession(configuration: config)
    }

    /// Preflight: `GET /v1/status` (unauthenticated) with ≤5s timeout.
    func fetchStatus() async throws -> IngestStatusResponse {
        try IngestURLPolicy.validateBaseURL(baseURL)
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

    func postObservations(_ body: Data) async throws {
        try IngestURLPolicy.validateBaseURL(baseURL)
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
