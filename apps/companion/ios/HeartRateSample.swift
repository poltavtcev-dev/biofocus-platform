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

    static var hrType: HKQuantityType? {
        HKQuantityType.quantityType(forIdentifier: .heartRate)
    }

    /// One-shot latest heart-rate quantity → Observation DTO (no background poll loop).
    static func latestObservation(store: HKHealthStore = HKHealthStore()) async throws -> HeartRateObservationDTO {
        guard HKHealthStore.isHealthDataAvailable() else {
            throw HeartRateSampleError.healthDataUnavailable
        }
        guard let hrType else {
            throw HeartRateSampleError.healthDataUnavailable
        }

        try await store.requestAuthorization(toShare: [], read: [hrType])

        let samples = try await querySamples(store: store, type: hrType, since: nil, limit: 1, ascending: false)
        guard let sample = samples.first else {
            throw HeartRateSampleError.noSample
        }
        return dto(from: sample)
    }

    static func observations(
        since: Date?,
        limit: Int = 40,
        store: HKHealthStore = HKHealthStore()
    ) async throws -> [HeartRateObservationDTO] {
        guard let hrType else { return [] }
        let samples = try await querySamples(
            store: store,
            type: hrType,
            since: since,
            limit: limit,
            ascending: true
        )
        return samples.map(dto(from:))
    }

    static func dto(from sample: HKQuantitySample) -> HeartRateObservationDTO {
        let bpm = sample.quantity.doubleValue(for: HKUnit.count().unitDivided(by: .minute()))
        let source = sample.sourceRevision.source.name
        let ts = Int64(sample.endDate.timeIntervalSince1970)
        return HeartRateObservationDTO(
            id: sample.uuid.uuidString,
            timestamp: ts,
            provider_id: providerId,
            data_type: dataType,
            payload: .init(bpm: bpm, source: source),
            confidence: 0.98
        )
    }

    private static func querySamples(
        store: HKHealthStore,
        type: HKQuantityType,
        since: Date?,
        limit: Int,
        ascending: Bool
    ) async throws -> [HKQuantitySample] {
        try await withCheckedThrowingContinuation { cont in
            let predicate: NSPredicate?
            if let since {
                predicate = HKQuery.predicateForSamples(
                    withStart: since.addingTimeInterval(0.001),
                    end: nil,
                    options: [.strictStartDate]
                )
            } else {
                predicate = nil
            }
            let sort = NSSortDescriptor(key: HKSampleSortIdentifierEndDate, ascending: ascending)
            let query = HKSampleQuery(
                sampleType: type,
                predicate: predicate,
                limit: limit,
                sortDescriptors: [sort]
            ) { _, samples, error in
                if let error {
                    cont.resume(throwing: error)
                    return
                }
                cont.resume(returning: (samples as? [HKQuantitySample]) ?? [])
            }
            store.execute(query)
        }
    }
}
