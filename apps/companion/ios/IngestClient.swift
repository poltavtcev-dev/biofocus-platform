import Foundation

/// Errors from companion → Desktop ingest (mirrors Rust `CompanionError`).
enum IngestClientError: Error, LocalizedError {
    case unauthorized
    case http(status: Int, body: String)
    case network(underlying: Error)
    case badResponse(String)

    var errorDescription: String? {
        switch self {
        case .unauthorized:
            return "Ingest unauthorized — check pairing token"
        case .http(let status, let body):
            return "Ingest HTTP \(status): \(body)"
        case .network(let underlying):
            return "Network error talking to ingest: \(underlying.localizedDescription)"
        case .badResponse(let message):
            return "Unexpected ingest response: \(message)"
        }
    }
}

/// Minimal HTTP client for `POST /v1/ingest`.
struct IngestClient {
    var baseURL: URL
    var token: String
    var session: URLSession = .shared

    func postObservations(_ body: Data) async throws {
        var request = URLRequest(url: baseURL.appendingPathComponent("v1/ingest"))
        request.httpMethod = "POST"
        request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = body

        let data: Data
        let response: URLResponse
        do {
            (data, response) = try await session.data(for: request)
        } catch {
            throw IngestClientError.network(underlying: error)
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
}
