import Foundation
import HealthKit

/// DTO matching ADR-018 `sleep_interval` Observation.
struct SleepIntervalObservationDTO: Encodable {
    let id: String
    let timestamp: Int64
    let provider_id: String
    let data_type: String
    let payload: Payload
    let confidence: Double

    struct Payload: Encodable {
        let start: Int64
        let end: Int64
        let stage: String?
        let source: String?
    }
}

enum SleepIntervalSample {
    static let providerId = "com.biofocus.applehealth"
    static let dataType = "sleep_interval"

    static var categoryType: HKCategoryType? {
        HKObjectType.categoryType(forIdentifier: .sleepAnalysis)
    }

    static func latestObservation(store: HKHealthStore = HKHealthStore()) async throws -> SleepIntervalObservationDTO? {
        guard let categoryType else { return nil }
        let samples = try await querySamples(store: store, type: categoryType, since: nil, limit: 1, ascending: false)
        return samples.first.map(dto(from:))
    }

    static func observations(
        since: Date?,
        limit: Int = 40,
        store: HKHealthStore = HKHealthStore()
    ) async throws -> [SleepIntervalObservationDTO] {
        guard let categoryType else { return [] }
        let samples = try await querySamples(store: store, type: categoryType, since: since, limit: limit, ascending: true)
        return samples.map(dto(from:))
    }

    static func dto(from sample: HKCategorySample) -> SleepIntervalObservationDTO {
        let start = Int64(sample.startDate.timeIntervalSince1970)
        let end = Int64(sample.endDate.timeIntervalSince1970)
        let stage = mapStage(sample.value)
        let source = sample.sourceRevision.source.name
        return SleepIntervalObservationDTO(
            id: sample.uuid.uuidString,
            timestamp: end,
            provider_id: providerId,
            data_type: dataType,
            payload: .init(
                start: start,
                end: max(start, end),
                stage: stage,
                source: source
            ),
            confidence: 0.85
        )
    }

    /// Maps HK sleep analysis values to ADR-018 closed-set stages (omit-friendly).
    static func mapStage(_ value: Int) -> String {
        if #available(iOS 16.0, *) {
            switch value {
            case HKCategoryValueSleepAnalysis.inBed.rawValue:
                return "in_bed"
            case HKCategoryValueSleepAnalysis.awake.rawValue:
                return "awake"
            case HKCategoryValueSleepAnalysis.asleepUnspecified.rawValue,
                 HKCategoryValueSleepAnalysis.asleepCore.rawValue,
                 HKCategoryValueSleepAnalysis.asleepDeep.rawValue,
                 HKCategoryValueSleepAnalysis.asleepREM.rawValue:
                return "asleep"
            default:
                return "unknown"
            }
        } else {
            switch value {
            case HKCategoryValueSleepAnalysis.inBed.rawValue:
                return "in_bed"
            case HKCategoryValueSleepAnalysis.asleep.rawValue:
                return "asleep"
            case HKCategoryValueSleepAnalysis.awake.rawValue:
                return "awake"
            default:
                return "unknown"
            }
        }
    }

    private static func querySamples(
        store: HKHealthStore,
        type: HKCategoryType,
        since: Date?,
        limit: Int,
        ascending: Bool
    ) async throws -> [HKCategorySample] {
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
                cont.resume(returning: (samples as? [HKCategorySample]) ?? [])
            }
            store.execute(query)
        }
    }
}
