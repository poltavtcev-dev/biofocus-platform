import Foundation
import HealthKit

/// DTO matching `docs/07-contracts.md` hrv Observation (Apple HK SDNN → `sdnn_ms`, ADR-016).
struct HrvObservationDTO: Encodable {
    let id: String
    let timestamp: Int64
    let provider_id: String
    let data_type: String
    let payload: Payload
    let confidence: Double

    struct Payload: Encodable {
        let sdnn_ms: Double
        let source: String?
    }
}

enum HrvSampleError: Error, LocalizedError {
    case healthDataUnavailable
    case noSample

    var errorDescription: String? {
        switch self {
        case .healthDataUnavailable:
            return "Health data is not available on this device"
        case .noSample:
            return "No HRV sample yet — HealthKit SDNN may be sparse (often overnight). Heart rate can still sync."
        }
    }
}

enum HrvSample {
    static let providerId = "com.biofocus.applehealth"
    static let dataType = "hrv"

    static var hrvType: HKQuantityType? {
        HKQuantityType.quantityType(forIdentifier: .heartRateVariabilitySDNN)
    }

    /// Latest HRV SDNN quantity → Observation DTO (no busy-loop).
    static func latestObservation(store: HKHealthStore = HKHealthStore()) async throws -> HrvObservationDTO {
        guard HKHealthStore.isHealthDataAvailable() else {
            throw HrvSampleError.healthDataUnavailable
        }
        guard let hrvType else {
            throw HrvSampleError.healthDataUnavailable
        }

        try await store.requestAuthorization(toShare: [], read: [hrvType])

        let sample = try await queryLatest(store: store, type: hrvType)
        guard let sample else {
            throw HrvSampleError.noSample
        }
        return dto(from: sample)
    }

    /// Samples ending after `since` (exclusive), oldest first — for background catch-up.
    static func observations(
        since: Date?,
        limit: Int = 20,
        store: HKHealthStore = HKHealthStore()
    ) async throws -> [HrvObservationDTO] {
        guard let hrvType else { return [] }
        let samples = try await querySamples(store: store, type: hrvType, since: since, limit: limit)
        return samples.map(dto(from:))
    }

    static func dto(from sample: HKQuantitySample) -> HrvObservationDTO {
        let sdnn = sample.quantity.doubleValue(for: .secondUnit(with: .milli))
        let source = sample.sourceRevision.source.name
        let ts = Int64(sample.endDate.timeIntervalSince1970)
        let id = sample.uuid.uuidString
        return HrvObservationDTO(
            id: id,
            timestamp: ts,
            provider_id: providerId,
            data_type: dataType,
            payload: .init(sdnn_ms: sdnn, source: source),
            confidence: 0.95
        )
    }

    private static func queryLatest(
        store: HKHealthStore,
        type: HKQuantityType
    ) async throws -> HKQuantitySample? {
        try await withCheckedThrowingContinuation { cont in
            let sort = NSSortDescriptor(key: HKSampleSortIdentifierEndDate, ascending: false)
            let query = HKSampleQuery(
                sampleType: type,
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
    }

    private static func querySamples(
        store: HKHealthStore,
        type: HKQuantityType,
        since: Date?,
        limit: Int
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
            let sort = NSSortDescriptor(key: HKSampleSortIdentifierEndDate, ascending: true)
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
