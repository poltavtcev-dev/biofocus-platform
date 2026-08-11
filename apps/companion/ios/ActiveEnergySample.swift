import Foundation
import HealthKit

/// DTO matching ADR-018 `active_energy` Observation (kcal).
struct ActiveEnergyObservationDTO: Encodable {
    let id: String
    let timestamp: Int64
    let provider_id: String
    let data_type: String
    let payload: Payload
    let confidence: Double

    struct Payload: Encodable {
        let kcal: Double
        let source: String?
    }
}

enum ActiveEnergySample {
    static let providerId = "com.biofocus.applehealth"
    static let dataType = "active_energy"

    static var quantityType: HKQuantityType? {
        HKQuantityType.quantityType(forIdentifier: .activeEnergyBurned)
    }

    static func latestObservation(store: HKHealthStore = HKHealthStore()) async throws -> ActiveEnergyObservationDTO? {
        guard let quantityType else { return nil }
        let samples = try await querySamples(store: store, type: quantityType, since: nil, limit: 1, ascending: false)
        return samples.first.map(dto(from:))
    }

    static func observations(
        since: Date?,
        limit: Int = 40,
        store: HKHealthStore = HKHealthStore()
    ) async throws -> [ActiveEnergyObservationDTO] {
        guard let quantityType else { return [] }
        let samples = try await querySamples(store: store, type: quantityType, since: since, limit: limit, ascending: true)
        return samples.map(dto(from:))
    }

    static func dto(from sample: HKQuantitySample) -> ActiveEnergyObservationDTO {
        let kcal = sample.quantity.doubleValue(for: .kilocalorie())
        let source = sample.sourceRevision.source.name
        return ActiveEnergyObservationDTO(
            id: sample.uuid.uuidString,
            timestamp: Int64(sample.endDate.timeIntervalSince1970),
            provider_id: providerId,
            data_type: dataType,
            payload: .init(kcal: max(0, kcal), source: source),
            confidence: 0.9
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
