import Foundation
import HealthKit

/// DTO matching `docs/07-contracts.md` heart_rate Observation (encode as JSON object).
struct HeartRateObservationDTO: Encodable {
    let id: String
    let timestamp: Int64
    let provider_id: String
    let data_type: String
    let payload: Payload
    let confidence: Double

    struct Payload: Encodable {
        let bpm: Double
        let source: String?
    }
}

enum HeartRateSampleError: Error, LocalizedError {
    case healthDataUnavailable
    case notAuthorized
    case noSample

    var errorDescription: String? {
        switch self {
        case .healthDataUnavailable:
            return "Health data is not available on this device"
        case .notAuthorized:
            return "Heart-rate read was not authorized"
        case .noSample:
            return "No heart-rate sample yet — add one in Health, then try again"
        }
    }
}

enum HeartRateSample {
    static let providerId = "com.biofocus.applehealth"
    static let dataType = "heart_rate"

    /// One-shot latest heart-rate quantity → Observation DTO (no background poll loop).
    static func latestObservation(store: HKHealthStore = HKHealthStore()) async throws -> HeartRateObservationDTO {
        guard HKHealthStore.isHealthDataAvailable() else {
            throw HeartRateSampleError.healthDataUnavailable
        }
        guard let hrType = HKQuantityType.quantityType(forIdentifier: .heartRate) else {
            throw HeartRateSampleError.healthDataUnavailable
        }

        try await store.requestAuthorization(toShare: [], read: [hrType])

        let sample = try await withCheckedThrowingContinuation { (cont: CheckedContinuation<HKQuantitySample?, Error>) in
            let sort = NSSortDescriptor(key: HKSampleSortIdentifierEndDate, ascending: false)
            let query = HKSampleQuery(
                sampleType: hrType,
                predicate: nil,
                limit: 1,
                sortDescriptors: [sort]
            ) { _, samples, error in
                if let error {
                    cont.resume(throwing: error)
                    return
                }
                cont.resume(returning: samples?.first as? HKQuantitySample)
            }
            store.execute(query)
        }

        guard let sample else {
            throw HeartRateSampleError.noSample
        }

        let bpm = sample.quantity.doubleValue(for: HKUnit.count().unitDivided(by: .minute()))
        let source = sample.sourceRevision.source.name
        let ts = Int64(sample.endDate.timeIntervalSince1970)

        return HeartRateObservationDTO(
            id: UUID().uuidString,
            timestamp: ts,
            provider_id: providerId,
            data_type: dataType,
            payload: .init(bpm: bpm, source: source),
            confidence: 0.98
        )
    }
}
