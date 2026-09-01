import Foundation
import HealthKit

/// Soft-optional ADR-018 `oxygen_saturation` DTO — emit only when HK has samples.
struct OxygenSaturationObservationDTO: Encodable {
    let id: String
    let timestamp: Int64
    let provider_id: String
    let data_type: String
    let payload: Payload
    let confidence: Double

    struct Payload: Encodable {
        let spo2_percent: Double
        let source: String?
    }
}

enum OxygenSaturationSample {
    static let providerId = "com.biofocus.applehealth"
    static let dataType = "oxygen_saturation"

    static var quantityType: HKQuantityType? {
        HKQuantityType.quantityType(forIdentifier: .oxygenSaturation)
    }

    /// Returns `nil` when no sample (soft-omit — never invent SpO2).
    static func latestObservation(store: HKHealthStore = HKHealthStore()) async throws -> OxygenSaturationObservationDTO? {
        guard let quantityType else { return nil }
        let samples = try await querySamples(store: store, type: quantityType, since: nil, limit: 1, ascending: false)
        return samples.first.map(dto(from:))
    }

    static func observations(
        since: Date?,
        limit: Int = 20,
        store: HKHealthStore = HKHealthStore()
    ) async throws -> [OxygenSaturationObservationDTO] {
        guard let quantityType else { return [] }
        let samples = try await querySamples(store: store, type: quantityType, since: since, limit: limit, ascending: true)
        return samples.map(dto(from:))
    }

    static func dto(from sample: HKQuantitySample) -> OxygenSaturationObservationDTO {
        var pct = sample.quantity.doubleValue(for: .percent()) * 100.0
        pct = min(100.0, max(0.0, pct))
        let source = sample.sourceRevision.source.name
        return OxygenSaturationObservationDTO(
            id: sample.uuid.uuidString,
            timestamp: Int64(sample.endDate.timeIntervalSince1970),
            provider_id: providerId,
            data_type: dataType,
            payload: .init(spo2_percent: pct, source: source),
            confidence: 0.85
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
