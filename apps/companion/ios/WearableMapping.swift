import CryptoKit
import Foundation
import HealthKit

/// Closed `src.kind` set from ADR-030. Personal device names never land in the payload.
enum WearableKind {
    static let appleWatch = "apple_watch"
    static let iphone = "iphone"
    static let xiaomiMiFitness = "xiaomi_mi_fitness"
    static let zeppLife = "zepp_life"
    static let otherApp = "other_app"
    static let manual = "manual"

    /// Bundle table first, then Apple product type. A user-entered sample is `manual`.
    static func classify(
        bundleId: String,
        productType: String?,
        hardwareVersion: String?,
        wasUserEntered: Bool
    ) -> String {
        if wasUserEntered {
            return manual
        }
        switch bundleId {
        case "com.xiaomi.wearable":
            return xiaomiMiFitness
        case "com.huami.watch", "com.xiaomi.hm.health":
            return zeppLife
        default:
            break
        }
        let product = productType ?? ""
        let hardware = hardwareVersion ?? ""
        let appleHealth = bundleId == "com.apple.health" || bundleId.hasPrefix("com.apple.health.")
        if appleHealth && (product.hasPrefix("Watch") || hardware.hasPrefix("Watch")) {
            return appleWatch
        }
        if product.hasPrefix("iPhone") || hardware.hasPrefix("iPhone") {
            return iphone
        }
        return otherApp
    }

    /// App label from the bundle id. `HKSource.name` is not used — it can be a person's watch name.
    static func appName(forBundle bundleId: String) -> String? {
        switch bundleId {
        case "com.apple.health":
            return "Health"
        case "com.xiaomi.wearable":
            return "Mi Fitness"
        case "com.huami.watch", "com.xiaomi.hm.health":
            return "Zepp Life"
        default:
            return nil
        }
    }

    static func deviceLabelHash(label: String, salt: Data) -> String {
        var bytes = salt
        bytes.append(contentsOf: Data(label.utf8))
        let digest = SHA256.hash(data: bytes)
        return digest.map { String(format: "%02x", $0) }.joined()
    }
}

enum SleepStageMap {
    /// Core, deep, and REM stay separate. They are not folded into `asleep`.
    static func stage(hkValue: Int) -> String {
        if hkValue == HKCategoryValueSleepAnalysis.asleepCore.rawValue {
            return "asleep_core"
        }
        if hkValue == HKCategoryValueSleepAnalysis.asleepDeep.rawValue {
            return "asleep_deep"
        }
        if hkValue == HKCategoryValueSleepAnalysis.asleepREM.rawValue {
            return "asleep_rem"
        }
        if hkValue == HKCategoryValueSleepAnalysis.inBed.rawValue {
            return "in_bed"
        }
        if hkValue == HKCategoryValueSleepAnalysis.awake.rawValue {
            return "awake"
        }
        if hkValue == HKCategoryValueSleepAnalysis.asleepUnspecified.rawValue {
            return "asleep"
        }
        return "unknown"
    }

    static func stage(fixtureValue: String) -> String {
        switch fixtureValue {
        case "asleepCore":
            return "asleep_core"
        case "asleepDeep":
            return "asleep_deep"
        case "asleepREM":
            return "asleep_rem"
        case "asleepUnspecified", "asleep":
            return "asleep"
        case "inBed":
            return "in_bed"
        case "awake":
            return "awake"
        default:
            return "unknown"
        }
    }
}

struct SourceFacts: Equatable {
    var bundleId: String
    var appName: String?
    var deviceModel: String?
    var manufacturer: String?
    var hardwareVersion: String?
    var productType: String?
    var osVersion: String?
    /// Clear-text label used only to compute `device_label_hash`. Never encoded.
    var deviceLabel: String?
    var wasUserEntered: Bool
}

struct HealthRecord: Equatable {
    var uuid: String
    var identifier: String
    var start: Int64
    var end: Int64
    var value: Double?
    var categoryValue: Int?
    var categoryName: String?
    var activityType: String?
    var kcal: Double?
    var distanceMeters: Double?
    var source: SourceFacts
}

enum HealthTypeRegistry {
    static let heartRate = "HKQuantityTypeIdentifierHeartRate"
    static let restingHeartRate = "HKQuantityTypeIdentifierRestingHeartRate"
    static let walkingHeartRate = "HKQuantityTypeIdentifierWalkingHeartRateAverage"
    static let hrvSDNN = "HKQuantityTypeIdentifierHeartRateVariabilitySDNN"
    static let oxygenSaturation = "HKQuantityTypeIdentifierOxygenSaturation"
    static let respiratoryRate = "HKQuantityTypeIdentifierRespiratoryRate"
    static let wristTemperature = "HKQuantityTypeIdentifierAppleSleepingWristTemperature"
    static let vo2Max = "HKQuantityTypeIdentifierVO2Max"
    static let stepCount = "HKQuantityTypeIdentifierStepCount"
    static let distance = "HKQuantityTypeIdentifierDistanceWalkingRunning"
    static let activeEnergy = "HKQuantityTypeIdentifierActiveEnergyBurned"
    static let basalEnergy = "HKQuantityTypeIdentifierBasalEnergyBurned"
    static let exerciseTime = "HKQuantityTypeIdentifierAppleExerciseTime"
    static let standTime = "HKQuantityTypeIdentifierAppleStandTime"
    static let standHour = "HKCategoryTypeIdentifierAppleStandHour"
    static let mindfulSession = "HKCategoryTypeIdentifierMindfulSession"
    static let sleep = "HKCategoryTypeIdentifierSleepAnalysis"
    static let workout = "HKWorkoutTypeIdentifier"

    /// HRV and sleep before cumulative counters, so a short sync tick reaches them first.
    static let syncPriority: [String] = [
        hrvSDNN,
        restingHeartRate,
        sleep,
        oxygenSaturation,
        respiratoryRate,
        heartRate,
        walkingHeartRate,
        wristTemperature,
        vo2Max,
        workout,
        stepCount,
        distance,
        activeEnergy,
        basalEnergy,
        exerciseTime,
        standTime,
        standHour,
        mindfulSession,
    ]

    static func sampleType(for identifier: String) -> HKSampleType? {
        switch identifier {
        case heartRate:
            return HKQuantityType.quantityType(forIdentifier: .heartRate)
        case restingHeartRate:
            return HKQuantityType.quantityType(forIdentifier: .restingHeartRate)
        case walkingHeartRate:
            return HKQuantityType.quantityType(forIdentifier: .walkingHeartRateAverage)
        case hrvSDNN:
            return HKQuantityType.quantityType(forIdentifier: .heartRateVariabilitySDNN)
        case oxygenSaturation:
            return HKQuantityType.quantityType(forIdentifier: .oxygenSaturation)
        case respiratoryRate:
            return HKQuantityType.quantityType(forIdentifier: .respiratoryRate)
        case wristTemperature:
            return HKQuantityType.quantityType(forIdentifier: .appleSleepingWristTemperature)
        case vo2Max:
            return HKQuantityType.quantityType(forIdentifier: .vo2Max)
        case stepCount:
            return HKQuantityType.quantityType(forIdentifier: .stepCount)
        case distance:
            return HKQuantityType.quantityType(forIdentifier: .distanceWalkingRunning)
        case activeEnergy:
            return HKQuantityType.quantityType(forIdentifier: .activeEnergyBurned)
        case basalEnergy:
            return HKQuantityType.quantityType(forIdentifier: .basalEnergyBurned)
        case exerciseTime:
            return HKQuantityType.quantityType(forIdentifier: .appleExerciseTime)
        case standTime:
            return HKQuantityType.quantityType(forIdentifier: .appleStandTime)
        case standHour:
            return HKCategoryType.categoryType(forIdentifier: .appleStandHour)
        case mindfulSession:
            return HKCategoryType.categoryType(forIdentifier: .mindfulSession)
        case sleep:
            return HKCategoryType.categoryType(forIdentifier: .sleepAnalysis)
        case workout:
            return HKObjectType.workoutType()
        default:
            return nil
        }
    }

    static var readTypes: Set<HKObjectType> {
        Set(syncPriority.compactMap { sampleType(for: $0) })
    }
}

enum WearableMapper {
    static let providerId = "com.biofocus.applehealth"

    static func jsonData(for record: HealthRecord, salt: Data) -> Data? {
        let object = observationObject(for: record, salt: salt)
        return try? JSONSerialization.data(withJSONObject: object, options: [.sortedKeys])
    }

    static func deletionJSON(targetId: String, timestamp: Int64) -> Data? {
        let object: [String: Any] = [
            "id": UUID().uuidString.lowercased(),
            "timestamp": timestamp,
            "provider_id": providerId,
            "data_type": "source_deletion",
            "payload": ["target_id": targetId.lowercased()],
            "confidence": 1,
        ]
        return try? JSONSerialization.data(withJSONObject: object, options: [.sortedKeys])
    }

    static func observationObject(for record: HealthRecord, salt: Data) -> [String: Any] {
        let id = record.uuid.lowercased()
        let timestamp = observationTimestamp(record)
        return [
            "id": id,
            "timestamp": timestamp,
            "provider_id": providerId,
            "data_type": dataType(for: record.identifier),
            "payload": payload(for: record, salt: salt),
            "confidence": 1,
        ]
    }

    static func dataType(for identifier: String) -> String {
        switch identifier {
        case HealthTypeRegistry.heartRate:
            return "heart_rate"
        case HealthTypeRegistry.restingHeartRate:
            return "resting_heart_rate"
        case HealthTypeRegistry.walkingHeartRate:
            return "walking_heart_rate_average"
        case HealthTypeRegistry.hrvSDNN:
            return "hrv"
        case HealthTypeRegistry.oxygenSaturation:
            return "oxygen_saturation"
        case HealthTypeRegistry.respiratoryRate:
            return "respiratory_rate"
        case HealthTypeRegistry.wristTemperature:
            return "sleeping_wrist_temperature"
        case HealthTypeRegistry.vo2Max:
            return "vo2_max"
        case HealthTypeRegistry.stepCount:
            return "step_count"
        case HealthTypeRegistry.distance:
            return "distance_walking_running"
        case HealthTypeRegistry.activeEnergy:
            return "active_energy"
        case HealthTypeRegistry.basalEnergy:
            return "basal_energy"
        case HealthTypeRegistry.exerciseTime:
            return "exercise_time"
        case HealthTypeRegistry.standTime:
            return "stand_time"
        case HealthTypeRegistry.standHour:
            return "stand_hour"
        case HealthTypeRegistry.mindfulSession:
            return "mindful_session"
        case HealthTypeRegistry.sleep:
            return "sleep_interval"
        case HealthTypeRegistry.workout:
            return "workout"
        default:
            return "heart_rate"
        }
    }

    private static func observationTimestamp(_ record: HealthRecord) -> Int64 {
        switch record.identifier {
        case HealthTypeRegistry.sleep, HealthTypeRegistry.standHour, HealthTypeRegistry.mindfulSession:
            return record.start
        default:
            return record.end
        }
    }

    private static func payload(for record: HealthRecord, salt: Data) -> [String: Any] {
        var payload: [String: Any] = [:]
        switch record.identifier {
        case HealthTypeRegistry.heartRate, HealthTypeRegistry.restingHeartRate, HealthTypeRegistry.walkingHeartRate:
            payload["bpm"] = jsonNumber(record.value ?? 0)
        case HealthTypeRegistry.hrvSDNN:
            payload["method"] = "sdnn"
            payload["sdnn_ms"] = jsonNumber(record.value ?? 0)
        case HealthTypeRegistry.oxygenSaturation:
            payload["spo2_percent"] = jsonNumber(spo2Percent(record.value ?? 0))
        case HealthTypeRegistry.respiratoryRate:
            payload["breaths_per_min"] = jsonNumber(record.value ?? 0)
        case HealthTypeRegistry.wristTemperature:
            payload["celsius"] = jsonNumber(record.value ?? 0)
        case HealthTypeRegistry.vo2Max:
            payload["ml_kg_min"] = jsonNumber(record.value ?? 0)
        case HealthTypeRegistry.stepCount:
            payload["count"] = jsonNumber(record.value ?? 0)
        case HealthTypeRegistry.distance:
            payload["meters"] = jsonNumber(record.value ?? 0)
        case HealthTypeRegistry.activeEnergy, HealthTypeRegistry.basalEnergy:
            payload["kcal"] = jsonNumber(record.value ?? 0)
        case HealthTypeRegistry.exerciseTime, HealthTypeRegistry.standTime:
            payload["minutes"] = jsonNumber(record.value ?? 0)
        case HealthTypeRegistry.standHour, HealthTypeRegistry.mindfulSession:
            payload["start"] = record.start
            payload["end"] = max(record.start, record.end)
        case HealthTypeRegistry.sleep:
            payload["start"] = record.start
            payload["end"] = max(record.start, record.end)
            if let categoryValue = record.categoryValue {
                payload["stage"] = SleepStageMap.stage(hkValue: categoryValue)
            } else if let categoryName = record.categoryName {
                payload["stage"] = SleepStageMap.stage(fixtureValue: categoryName)
            }
        case HealthTypeRegistry.workout:
            payload["activity_type"] = record.activityType ?? "other"
            payload["start"] = record.start
            payload["end"] = max(record.start, record.end)
            if let kcal = record.kcal {
                payload["kcal"] = jsonNumber(kcal)
            }
            if let meters = record.distanceMeters {
                payload["distance_m"] = jsonNumber(meters)
            }
        default:
            break
        }
        payload["src"] = srcObject(record.source, salt: salt)
        return payload
    }

    /// HealthKit SpO2 is a fraction (0.97 → 97). Values already on a 0–100 scale stay put.
    static func spo2Percent(_ raw: Double) -> Double {
        if raw <= 1 {
            return raw * 100
        }
        return raw
    }

    static func srcObject(_ source: SourceFacts, salt: Data) -> [String: Any] {
        var src: [String: Any] = [
            "bundle_id": source.bundleId,
            "was_user_entered": source.wasUserEntered,
            "kind": WearableKind.classify(
                bundleId: source.bundleId,
                productType: source.productType,
                hardwareVersion: source.hardwareVersion,
                wasUserEntered: source.wasUserEntered
            ),
        ]
        if let appName = source.appName {
            src["app_name"] = appName
        }
        if let model = source.deviceModel {
            src["device_model"] = model
        }
        if let manufacturer = source.manufacturer {
            src["device_manufacturer"] = manufacturer
        }
        if let hardware = source.hardwareVersion {
            src["hardware_version"] = hardware
        }
        if let product = source.productType {
            src["product_type"] = product
        }
        if let os = source.osVersion {
            src["os_version"] = os
        }
        if let label = source.deviceLabel, !label.isEmpty {
            src["device_label_hash"] = WearableKind.deviceLabelHash(label: label, salt: salt)
        }
        return src
    }

    static func jsonNumber(_ value: Double) -> Any {
        if value.isFinite, value.rounded() == value, abs(value) < 1e15 {
            return Int(value)
        }
        return value
    }
}
